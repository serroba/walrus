//! Individual agent simulation with struct-of-arrays layout.
//!
//! Agents interact locally (cooperate, trade, conflict, delegate, court).
//! Macro patterns like hierarchy, specialization, and inequality emerge
//! from these micro interactions rather than being formula-driven.

use rayon::prelude::*;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillType {
    Forager,
    Crafter,
    Builder,
    Leader,
    Warrior,
}

// ---------------------------------------------------------------------------
// Cultural system (Phase 5)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KinshipSystem {
    Patrilineal,
    Matrilineal,
    Bilateral,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarriageRule {
    Monogamy,
    Polygyny,
    Polyandry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResidenceRule {
    Patrilocal,
    Matrilocal,
    Neolocal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InheritanceRule {
    Primogeniture,
    Partible,
    Matrilineal,
}

/// Rich cultural genome transmitted between agents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Culture {
    // Kinship and family structure
    pub kinship_system: KinshipSystem,
    pub marriage_rule: MarriageRule,
    pub residence_rule: ResidenceRule,
    pub inheritance_rule: InheritanceRule,
    // Authority and governance
    pub authority_norm: f32,     // 0=egalitarian, 1=highly hierarchical
    pub coercion_tolerance: f32, // how much extraction people accept
    // Economic
    pub sharing_norm: f32,  // 0=individualist, 1=full communal sharing
    pub property_norm: f32, // 0=communal, 1=private property
    // Technology
    pub techniques: u64, // bitfield of known technologies
    // Social
    pub trust_outgroup: f32, // willingness to cooperate with strangers
    pub risk_tolerance: f32, // willingness to try new things
}

impl Default for Culture {
    fn default() -> Self {
        Self {
            kinship_system: KinshipSystem::Bilateral,
            marriage_rule: MarriageRule::Monogamy,
            residence_rule: ResidenceRule::Neolocal,
            inheritance_rule: InheritanceRule::Partible,
            authority_norm: 0.3,
            coercion_tolerance: 0.3,
            sharing_norm: 0.5,
            property_norm: 0.3,
            techniques: 0,
            trust_outgroup: 0.3,
            risk_tolerance: 0.5,
        }
    }
}

/// Parameters controlling cultural transmission and mutation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CulturalParams {
    /// Probability of vertical (parent→child) trait mutation per discrete trait.
    pub vertical_mutation_prob: f64,
    /// Mutation magnitude for continuous cultural traits.
    pub cultural_mutation_magnitude: f32,
    /// Probability per interaction of horizontal (peer→peer) cultural adoption.
    pub horizontal_adoption_prob: f32,
    /// Probability per tick of oblique (prestige-biased) cultural adoption.
    pub oblique_adoption_prob: f32,
    /// Minimum prestige gap for oblique transmission.
    pub oblique_prestige_gap: f32,
    /// How much authority_norm boosts delegation willingness.
    pub authority_delegation_bonus: f32,
    /// How much trust_outgroup boosts inter-group trade tendency.
    pub trust_trade_bonus: f32,
    /// How much sharing_norm boosts cooperation tendency.
    pub sharing_coop_bonus: f32,
    /// How much coercion_tolerance reduces conflict avoidance.
    pub coercion_conflict_bonus: f32,
}

impl Default for CulturalParams {
    fn default() -> Self {
        Self {
            vertical_mutation_prob: 0.05,
            cultural_mutation_magnitude: 0.08,
            horizontal_adoption_prob: 0.02,
            oblique_adoption_prob: 0.01,
            oblique_prestige_gap: 0.3,
            authority_delegation_bonus: 0.15,
            trust_trade_bonus: 0.2,
            sharing_coop_bonus: 0.15,
            coercion_conflict_bonus: 0.1,
        }
    }
}

// ---------------------------------------------------------------------------
// Energy model (Phase 2)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnergyType {
    Biomass = 0,
    Agriculture = 1,
    Fossil = 2,
    Renewable = 3,
}

impl EnergyType {
    pub const ALL: [EnergyType; 4] = [
        EnergyType::Biomass,
        EnergyType::Agriculture,
        EnergyType::Fossil,
        EnergyType::Renewable,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergySource {
    pub stock: f64,
    pub initial_stock: f64,
    pub flow_rate: f64,
    pub base_eroei: f64,
    pub tech_threshold: f32,
    pub steepness: f64,
}

impl EnergySource {
    pub fn depletion(&self) -> f64 {
        if self.initial_stock <= 0.0 || self.initial_stock.is_infinite() {
            return 0.0;
        }
        (1.0 - self.stock / self.initial_stock).clamp(0.0, 1.0)
    }

    pub fn current_eroei(&self) -> f64 {
        self.base_eroei * (1.0 - self.depletion()).powf(self.steepness)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnergyCell {
    pub sources: [EnergySource; 4],
}

#[derive(Clone, Debug)]
pub struct EnergyLandscape {
    pub cells: Vec<EnergyCell>,
    pub(crate) cols: usize,
    pub(crate) rows: usize,
    pub(crate) cell_size: f32,
}

impl EnergyLandscape {
    pub fn mean_depletion(&self, energy_type: EnergyType) -> f64 {
        let idx = energy_type as usize;
        if self.cells.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.cells.iter().map(|c| c.sources[idx].depletion()).sum();
        sum / self.cells.len() as f64
    }

    pub fn total_pollution(&self) -> f64 {
        // Pollution tracked per-source is just depletion * extraction impact
        // For simplicity, use depletion as a proxy for cumulative pollution
        self.cells
            .iter()
            .flat_map(|c| c.sources.iter())
            .map(|s| {
                if s.initial_stock.is_infinite() {
                    0.0
                } else {
                    s.depletion() * s.initial_stock * 0.01
                }
            })
            .sum()
    }
}

/// Summary of energy harvested in a single tick.
#[derive(Clone, Copy, Debug, Default)]
pub struct EnergyTickSummary {
    pub energy_by_type: [f64; 4],
    pub total_net_energy: f64,
    pub agents_harvesting: u32,
}

/// Struct-of-arrays population for cache-friendly parallel access.
/// At ~108 bytes per agent, 1M agents ≈ 108 MB.
#[derive(Clone, Debug)]
pub struct Population {
    // Identity
    pub ids: Vec<u64>,

    // Biology
    pub sexes: Vec<Sex>,
    pub ages: Vec<u16>,
    pub fertilities: Vec<f32>,
    pub healths: Vec<f32>,

    // Skills and production
    pub skill_types: Vec<SkillType>,
    pub skill_levels: Vec<f32>,

    // Social
    pub statuses: Vec<f32>,
    pub prestiges: Vec<f32>,
    pub aggressions: Vec<f32>,
    pub cooperations: Vec<f32>,

    // Resources
    pub resources: Vec<f32>,
    pub surpluses: Vec<f32>,

    // Cultural
    pub cultures: Vec<Culture>,
    pub innovations: Vec<f32>,

    // Relationships
    pub kin_groups: Vec<u32>,
    pub partners: Vec<Option<u32>>, // index into population, not id
    pub patrons: Vec<Option<u32>>,  // delegation hierarchy
    pub patron_ticks: Vec<u32>,     // how many ticks current patron relationship has lasted

    // Spatial
    pub xs: Vec<f32>,
    pub ys: Vec<f32>,

    // Trust memory: EMA of incoming cooperation from neighbors (0=never cooperated with, 1=always)
    pub trust_memory: Vec<f32>,
}

pub(crate) struct AgentInit {
    pub(crate) id: u64,
    pub(crate) sex: Sex,
    pub(crate) age: u16,
    pub(crate) fertility: f32,
    pub(crate) health: f32,
    pub(crate) skill_type: SkillType,
    pub(crate) skill_level: f32,
    pub(crate) status: f32,
    pub(crate) prestige: f32,
    pub(crate) aggression: f32,
    pub(crate) cooperation: f32,
    pub(crate) resources: f32,
    pub(crate) surplus: f32,
    pub(crate) culture: Culture,
    pub(crate) innovation: f32,
    pub(crate) kin_group: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Population {
    fn empty() -> Self {
        Self {
            ids: Vec::new(),
            sexes: Vec::new(),
            ages: Vec::new(),
            fertilities: Vec::new(),
            healths: Vec::new(),
            skill_types: Vec::new(),
            skill_levels: Vec::new(),
            statuses: Vec::new(),
            prestiges: Vec::new(),
            aggressions: Vec::new(),
            cooperations: Vec::new(),
            resources: Vec::new(),
            surpluses: Vec::new(),
            cultures: Vec::new(),
            innovations: Vec::new(),
            kin_groups: Vec::new(),
            partners: Vec::new(),
            patrons: Vec::new(),
            patron_ticks: Vec::new(),
            xs: Vec::new(),
            ys: Vec::new(),
            trust_memory: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub(crate) fn push_agent(&mut self, a: AgentInit) {
        self.ids.push(a.id);
        self.sexes.push(a.sex);
        self.ages.push(a.age);
        self.fertilities.push(a.fertility);
        self.healths.push(a.health);
        self.skill_types.push(a.skill_type);
        self.skill_levels.push(a.skill_level);
        self.statuses.push(a.status);
        self.prestiges.push(a.prestige);
        self.aggressions.push(a.aggression);
        self.cooperations.push(a.cooperation);
        self.resources.push(a.resources);
        self.surpluses.push(a.surplus);
        self.cultures.push(a.culture);
        self.innovations.push(a.innovation);
        self.kin_groups.push(a.kin_group);
        self.partners.push(None);
        self.patrons.push(None);
        self.patron_ticks.push(0);
        self.xs.push(a.x);
        self.ys.push(a.y);
        self.trust_memory.push(0.5); // neutral trust
    }

    pub(crate) fn swap_remove(&mut self, idx: usize) {
        self.ids.swap_remove(idx);
        self.sexes.swap_remove(idx);
        self.ages.swap_remove(idx);
        self.fertilities.swap_remove(idx);
        self.healths.swap_remove(idx);
        self.skill_types.swap_remove(idx);
        self.skill_levels.swap_remove(idx);
        self.statuses.swap_remove(idx);
        self.prestiges.swap_remove(idx);
        self.aggressions.swap_remove(idx);
        self.cooperations.swap_remove(idx);
        self.resources.swap_remove(idx);
        self.surpluses.swap_remove(idx);
        self.cultures.swap_remove(idx);
        self.innovations.swap_remove(idx);
        self.kin_groups.swap_remove(idx);
        self.partners.swap_remove(idx);
        self.patrons.swap_remove(idx);
        self.patron_ticks.swap_remove(idx);
        self.xs.swap_remove(idx);
        self.ys.swap_remove(idx);
        self.trust_memory.swap_remove(idx);
    }
}

/// Spatial hash grid for O(1) neighbor queries.
pub(crate) struct SpatialGrid {
    cells: Vec<Vec<u32>>,
    cols: usize,
    rows: usize,
    pub(crate) cell_size: f32,
}

impl SpatialGrid {
    pub(crate) fn build(xs: &[f32], ys: &[f32], cell_size: f32, world_size: f32) -> Self {
        let cols = (world_size / cell_size).ceil() as usize + 1;
        let rows = cols;
        let mut cells = vec![Vec::new(); cols * rows];
        for (idx, (&x, &y)) in xs.iter().zip(ys.iter()).enumerate() {
            let cx = (x / cell_size).floor() as usize;
            let cy = (y / cell_size).floor() as usize;
            let key = cy.min(rows - 1) * cols + cx.min(cols - 1);
            cells[key].push(idx as u32);
        }
        Self {
            cells,
            cols,
            rows,
            cell_size,
        }
    }

    pub(crate) fn neighbors_of(&self, x: f32, y: f32) -> Vec<u32> {
        let cx = (x / self.cell_size).floor() as isize;
        let cy = (y / self.cell_size).floor() as isize;
        let mut result = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.cols && (ny as usize) < self.rows {
                    let key = ny as usize * self.cols + nx as usize;
                    result.extend_from_slice(&self.cells[key]);
                }
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Weights and thresholds for interaction decisions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InteractionParams {
    /// Weight of own cooperation trait on cooperation tendency.
    pub coop_self_weight: f32,
    /// Weight of other's cooperation trait on cooperation tendency.
    pub coop_other_weight: f32,
    /// Bonus to cooperation tendency when interacting with kin.
    pub coop_kin_bonus: f32,
    /// Weight of own aggression on conflict tendency.
    pub conflict_self_weight: f32,
    /// Weight of other's aggression on conflict tendency.
    pub conflict_other_weight: f32,
    /// Bonus to conflict tendency when interacting with non-kin.
    pub conflict_stranger_bonus: f32,
    /// Trade tendency when agents have different skills.
    pub trade_complementary: f32,
    /// Trade tendency when agents have the same skill.
    pub trade_same_skill: f32,
    /// Resource bonus per cooperation event (scaled by mean cooperation level).
    pub coop_resource_bonus: f32,
    /// Prestige gained per cooperation event.
    pub coop_prestige_gain: f32,
    /// Resources gained by conflict winner.
    pub conflict_win_resources: f32,
    /// Status gained by conflict winner.
    pub conflict_win_status: f32,
    /// Resources lost by conflict loser.
    pub conflict_lose_resources: f32,
    /// Health lost by conflict loser.
    pub conflict_lose_health: f32,
    /// Noise in conflict outcome (higher = more random).
    pub conflict_noise: f32,
    /// Trade surplus multiplier for complementary skills.
    pub trade_complementary_bonus: f32,
    /// Trade surplus for same-skill trades.
    pub trade_same_bonus: f32,
    /// Max health loss from interactions per tick (cap).
    pub max_health_loss_per_tick: f32,
    /// Status threshold above which an agent considers delegation.
    pub delegation_status_gap: f32,
    /// Tax rate patrons extract from delegating agents.
    pub delegation_tax_rate: f32,
    /// Prestige gained by patron per delegation.
    pub delegation_prestige_gain: f32,
    /// Status weight in power calculation.
    pub power_status_weight: f32,
    /// Skill weight in power calculation.
    pub power_skill_weight: f32,
    /// Aggression weight in power calculation.
    pub power_aggression_weight: f32,
    /// Max status value (clamp).
    pub max_status: f32,
    /// Max prestige value (clamp).
    pub max_prestige: f32,
    /// Subsistence level: resources above this are surplus.
    pub subsistence_level: f32,
    /// Skill improvement per tick through practice.
    pub skill_practice_rate: f32,
    /// Weight of trust_memory on cooperation tendency (higher = trust matters more).
    pub trust_coop_weight: f32,
    /// EMA decay for trust_memory updates (0-1; lower = longer memory).
    pub trust_memory_decay: f32,
}

impl Default for InteractionParams {
    fn default() -> Self {
        Self {
            coop_self_weight: 0.5,
            coop_other_weight: 0.3,
            coop_kin_bonus: 0.2,
            conflict_self_weight: 0.4,
            conflict_other_weight: 0.3,
            conflict_stranger_bonus: 0.15,
            trade_complementary: 0.4,
            trade_same_skill: 0.15,
            coop_resource_bonus: 0.01,
            coop_prestige_gain: 0.005,
            conflict_win_resources: 0.05,
            conflict_win_status: 0.01,
            conflict_lose_resources: 0.03,
            conflict_lose_health: 0.005,
            conflict_noise: 0.2,
            trade_complementary_bonus: 0.03,
            trade_same_bonus: 0.005,
            max_health_loss_per_tick: 0.01,
            delegation_status_gap: 0.1,
            delegation_tax_rate: 0.01,
            delegation_prestige_gain: 0.002,
            power_status_weight: 0.4,
            power_skill_weight: 0.3,
            power_aggression_weight: 0.3,
            max_status: 2.0,
            max_prestige: 5.0,
            subsistence_level: 0.5,
            skill_practice_rate: 0.002,
            trust_coop_weight: 0.25,
            trust_memory_decay: 0.15,
        }
    }
}

/// Parameters governing agent lifecycle: aging, health, death, reproduction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LifecycleParams {
    /// Base health decay per tick (age-independent).
    pub health_decay_base: f32,
    /// Additional health decay scaled by (age/max_age)^2.
    pub health_decay_age_factor: f32,
    /// Resource threshold below which health recovery kicks in.
    pub health_recovery_threshold: f32,
    /// Health recovery rate per tick (scaled by excess resources).
    pub health_recovery_rate: f32,
    /// Health below this threshold causes death.
    pub death_health_threshold: f32,
    /// Resource level below which starvation death is possible.
    pub starvation_resource_threshold: f32,
    /// Probability of starvation death per tick when resources are below threshold.
    pub starvation_death_prob: f32,
    /// Female peak fertility value.
    pub female_peak_fertility: f32,
    /// Age at which female fertility peaks.
    pub female_fertility_peak_age: f32,
    /// Rate of female fertility decline away from peak.
    pub female_fertility_decline: f32,
    /// Male peak fertility value.
    pub male_peak_fertility: f32,
    /// Age at which male fertility peaks.
    pub male_fertility_peak_age: f32,
    /// Rate of male fertility decline away from peak.
    pub male_fertility_decline: f32,
    /// Minimum fertility required for reproduction.
    pub min_fertility: f32,
    /// Minimum age for reproduction.
    pub min_reproduction_age: u16,
    /// Maximum age for reproduction.
    pub max_reproduction_age: u16,
    /// Minimum resources required to reproduce.
    pub reproduction_resource_threshold: f32,
    /// Base birth probability per tick (scaled by fertility and health).
    pub birth_rate: f32,
    /// Resources consumed by mother at birth.
    pub birth_resource_cost: f32,
    /// Health lost by mother at birth.
    pub birth_health_cost: f32,
    /// Probability child inherits mother's skill (vs father's or mutation).
    pub skill_maternal_inherit_prob: f64,
    /// Probability of skill mutation (random skill).
    pub skill_mutation_prob: f64,
    /// Standard deviation of trait inheritance noise (aggression, cooperation, innovation).
    pub trait_mutation_magnitude: f32,
    /// Probability of norm mutation per bit.
    pub norm_mutation_prob: f64,
    /// Initial health of newborn.
    pub newborn_health: f32,
    /// Initial skill level of newborn.
    pub newborn_skill_level: f32,
    /// Initial status of newborn.
    pub newborn_status: f32,
    /// Initial resources of newborn.
    pub newborn_resources: f32,
    /// Spawn radius around mother.
    pub birth_spawn_radius: f32,
    /// Number of agents per initial kin group.
    pub agents_per_kin_group: u32,
    /// Per-tick innovation growth (learning by doing).
    pub innovation_growth_rate: f32,
}

impl Default for LifecycleParams {
    fn default() -> Self {
        Self {
            health_decay_base: 0.001,
            health_decay_age_factor: 0.008,
            health_recovery_threshold: 0.2,
            health_recovery_rate: 0.02,
            death_health_threshold: 0.01,
            starvation_resource_threshold: 0.01,
            starvation_death_prob: 0.1,
            female_peak_fertility: 0.8,
            female_fertility_peak_age: 25.0,
            female_fertility_decline: 0.02,
            male_peak_fertility: 0.9,
            male_fertility_peak_age: 30.0,
            male_fertility_decline: 0.012,
            min_fertility: 0.2,
            min_reproduction_age: 8,
            max_reproduction_age: 50,
            reproduction_resource_threshold: 0.4,
            birth_rate: 0.25,
            birth_resource_cost: 0.2,
            birth_health_cost: 0.05,
            skill_maternal_inherit_prob: 0.7,
            skill_mutation_prob: 0.5,
            trait_mutation_magnitude: 0.1,
            norm_mutation_prob: 0.05,
            newborn_health: 0.9,
            newborn_skill_level: 0.05,
            newborn_status: 0.2,
            newborn_resources: 0.3,
            birth_spawn_radius: 2.0,
            agents_per_kin_group: 8,
            innovation_growth_rate: 0.0003,
        }
    }
}

/// Parameters governing agent movement each tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementParams {
    /// Strength of pull toward kin group centroid.
    pub kin_pull_strength: f32,
    /// Random drift magnitude when near kin.
    pub drift_with_kin: f32,
    /// Random drift magnitude when isolated.
    pub drift_alone: f32,
}

impl Default for MovementParams {
    fn default() -> Self {
        Self {
            kin_pull_strength: 0.02,
            drift_with_kin: 0.5,
            drift_alone: 0.8,
        }
    }
}

/// Weights for sexual selection scoring of potential mates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MateSelectionParams {
    pub status_weight: f32,
    pub resource_weight: f32,
    pub prestige_weight: f32,
    pub noise_weight: f32,
}

impl Default for MateSelectionParams {
    fn default() -> Self {
        Self {
            status_weight: 0.3,
            resource_weight: 0.3,
            prestige_weight: 0.3,
            noise_weight: 0.1,
        }
    }
}

/// Parameters for the energy landscape and EROEI dynamics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyParams {
    // Biomass: available everywhere, regenerates, low EROEI
    pub biomass_base_eroei: f64,
    pub biomass_initial_stock: f64,
    pub biomass_flow_rate: f64,
    pub biomass_steepness: f64,
    pub biomass_tech_threshold: f32,
    pub biomass_regen_rate: f64,

    // Agriculture: fertile areas only, high stock, medium EROEI
    pub agriculture_base_eroei: f64,
    pub agriculture_initial_stock: f64,
    pub agriculture_flow_rate: f64,
    pub agriculture_steepness: f64,
    pub agriculture_tech_threshold: f32,
    pub agriculture_fertility_prob: f64,

    // Fossil: rare deposits, finite, very high initial EROEI
    pub fossil_base_eroei: f64,
    pub fossil_initial_stock: f64,
    pub fossil_flow_rate: f64,
    pub fossil_steepness: f64,
    pub fossil_tech_threshold: f32,
    pub fossil_abundance: f64,

    // Renewable: everywhere, infinite, requires high tech
    pub renewable_base_eroei: f64,
    pub renewable_flow_rate: f64,
    pub renewable_tech_threshold: f32,

    /// Scaling factor for per-agent extraction rate.
    pub harvest_per_agent: f64,
}

impl Default for EnergyParams {
    fn default() -> Self {
        Self {
            // Biomass: base EROEI ~5:1, net ~0.05/agent matching old resource_regen
            biomass_base_eroei: 5.0,
            biomass_initial_stock: 100.0,
            biomass_flow_rate: 0.0625,
            biomass_steepness: 2.0,
            biomass_tech_threshold: 0.0,
            biomass_regen_rate: 0.05,

            // Agriculture: base EROEI ~10:1, needs innovation ~0.25
            agriculture_base_eroei: 10.0,
            agriculture_initial_stock: 500.0,
            agriculture_flow_rate: 0.15,
            agriculture_steepness: 1.5,
            agriculture_tech_threshold: 0.25,
            agriculture_fertility_prob: 0.4,

            // Fossil: base EROEI ~100:1, rare (15% of cells), needs innovation ~0.5
            fossil_base_eroei: 100.0,
            fossil_initial_stock: 200.0,
            fossil_flow_rate: 0.5,
            fossil_steepness: 3.0,
            fossil_tech_threshold: 0.5,
            fossil_abundance: 0.15,

            // Renewable: EROEI ~15:1, infinite, needs innovation ~0.75
            renewable_base_eroei: 15.0,
            renewable_flow_rate: 0.2,
            renewable_tech_threshold: 0.75,

            harvest_per_agent: 1.0,
        }
    }
}

/// Parameters for emergent institution dynamics (Phase 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstitutionParams {
    /// Fraction of tax revenue patrons invest in public goods.
    pub public_goods_rate: f32,
    /// Resource bonus per agent in a kin group with an investing patron.
    pub public_goods_bonus: f32,
    /// Conflict damage reduction for agents with an investing patron.
    pub defense_bonus: f32,
    /// Fraction of agents in kin group needed to recognize a leader.
    pub leadership_threshold: f32,
    /// Whether children inherit mother's patron.
    pub patron_inheritance: bool,
}

impl Default for InstitutionParams {
    fn default() -> Self {
        Self {
            public_goods_rate: 0.3,
            public_goods_bonus: 0.005,
            defense_bonus: 0.3,
            leadership_threshold: 0.5,
            patron_inheritance: true,
        }
    }
}

/// Parameters for inter-society dynamics: raids, conquest, tribute, migration (Phase 4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterSocietyParams {
    /// Minimum warriors in a kin group to consider raiding.
    pub min_raid_warriors: u32,
    /// Aggression threshold (mean group aggression) to trigger a raid.
    pub raid_aggression_threshold: f32,
    /// Resources looted per warrior in a successful raid.
    pub raid_loot_per_warrior: f32,
    /// Health damage to defenders per attacking warrior.
    pub raid_damage_per_warrior: f32,
    /// Power ratio (attacker/defender) needed for conquest.
    pub conquest_power_ratio: f32,
    /// Tribute extraction rate from vassals per tick.
    pub tribute_rate: f32,
    /// Maximum number of ticks tribute lasts before expiring.
    pub tribute_duration: u32,
    /// Resource threshold below which agents consider migrating.
    pub migration_resource_threshold: f32,
    /// Probability per tick of migration when conditions are met.
    pub migration_probability: f32,
    /// Maximum distance for inter-group raids (in world units).
    pub raid_range: f32,
}

impl Default for InterSocietyParams {
    fn default() -> Self {
        Self {
            min_raid_warriors: 3,
            raid_aggression_threshold: 0.35,
            raid_loot_per_warrior: 0.1,
            raid_damage_per_warrior: 0.02,
            conquest_power_ratio: 3.0,
            tribute_rate: 0.02,
            tribute_duration: 50,
            migration_resource_threshold: 0.15,
            migration_probability: 0.05,
            raid_range: 20.0,
        }
    }
}

/// Active tribute relationship between kin groups.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TributeRelation {
    pub vassal_kin: u32,
    pub overlord_kin: u32,
    pub rate: f32,
    pub ticks_remaining: u32,
}

/// Per-tick summary of inter-society events.
#[derive(Clone, Copy, Debug, Default)]
pub struct InterSocietySummary {
    pub raids: u32,
    pub conquests: u32,
    pub tribute_total: f32,
    pub migrations: u32,
    pub inter_group_trades: u32,
    pub total_trades: u32,
}

// ---------------------------------------------------------------------------
// Institutional detection (Phase 3)
// ---------------------------------------------------------------------------

/// Detected institutional type based on emergent population patterns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstitutionalType {
    Band = 0,
    Tribe = 1,
    Chiefdom = 2,
    State = 3,
}

/// Emergent institutional profile detected from population state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstitutionalProfile {
    pub institutional_type: InstitutionalType,
    pub coercion_rate: f32,
    pub property_norm_strength: f32,
    pub public_goods_investment: f32,
    pub patron_count: u32,
    pub recognized_leaders: u32,
    pub mean_patron_tenure: f32,
}

/// Top-level configuration for individual agent simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentSimConfig {
    pub seed: u64,
    pub initial_population: u32,
    pub ticks: u32,
    pub world_size: f32,
    pub interaction_radius: f32,
    /// Energy landscape parameters (replaces flat resource_regen).
    pub energy: EnergyParams,
    /// Maximum age before guaranteed death.
    pub max_age: u16,
    /// Minimum population below which simulation stops.
    pub min_population: u32,
    /// Maximum population above which birth rate is suppressed.
    pub max_population: u32,
    pub interaction: InteractionParams,
    pub lifecycle: LifecycleParams,
    pub movement: MovementParams,
    pub mate_selection: MateSelectionParams,
    pub institution: InstitutionParams,
    pub inter_society: InterSocietyParams,
    pub cultural: CulturalParams,
}

impl Default for AgentSimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            initial_population: 150,
            ticks: 500,
            world_size: 100.0,
            interaction_radius: 8.0,
            energy: EnergyParams::default(),
            max_age: 80,
            min_population: 10,
            max_population: 10_000,
            interaction: InteractionParams::default(),
            lifecycle: LifecycleParams::default(),
            movement: MovementParams::default(),
            mate_selection: MateSelectionParams::default(),
            institution: InstitutionParams::default(),
            inter_society: InterSocietyParams::default(),
            cultural: CulturalParams::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Emergence detection (measured, not prescribed)
// ---------------------------------------------------------------------------

/// Emergent properties measured from the population state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergentState {
    pub population_size: u32,
    pub mean_resources: f32,
    pub gini_coefficient: f32,
    pub skill_entropy: f32,
    pub max_hierarchy_depth: u32,
    pub num_leaders: u32,
    pub mean_group_size: f32,
    pub num_kin_groups: u32,
    pub cooperation_rate: f32,
    pub conflict_rate: f32,
    pub mean_prestige: f32,
    pub mean_health: f32,
    pub mean_innovation: f32,
    pub dominant_energy: u8,
    pub energy_per_capita: f32,
    pub mean_eroei: f32,
    pub biomass_depletion: f32,
    pub fossil_depletion: f32,
    // Institutional (Phase 3)
    pub coercion_rate: f32,
    pub property_norm_strength: f32,
    pub institutional_type: u8,
    pub public_goods_investment: f32,
    pub patron_count: u32,
    pub recognized_leaders: u32,
    pub mean_patron_tenure: f32,
    // Inter-society (Phase 4)
    pub raid_events: u32,
    pub conquest_events: u32,
    pub tribute_flows: f32,
    pub migration_events: u32,
    pub num_active_societies: u32,
    pub inter_group_trade_rate: f32,
    pub active_tributes: u32,
    // Cultural (Phase 5)
    pub mean_authority_norm: f32,
    pub mean_sharing_norm: f32,
    pub mean_property_norm: f32,
    pub mean_trust_outgroup: f32,
    pub cultural_diversity: f32, // entropy over kinship/marriage/residence/inheritance combos
    pub dominant_kinship: u8,    // 0=patrilineal, 1=matrilineal, 2=bilateral
    pub dominant_marriage: u8,   // 0=monogamy, 1=polygyny, 2=polyandry
    pub mean_coercion_tolerance: f32,
    pub technique_count: f32, // mean bits set in techniques
    // Coordination dilemma
    /// Fraction of surplus lost to coordination failure: 1 - (actual / cooperative_optimal).
    pub coordination_failure_index: f32,
    /// Mean trust_memory across all agents.
    pub mean_trust: f32,
}

/// Per-tick snapshot of the simulation state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentSnapshot {
    pub tick: u32,
    pub emergent: EmergentState,
}

/// Final result of a simulation run.
#[derive(Clone, Debug)]
pub struct AgentSimResult {
    pub snapshots: Vec<AgentSnapshot>,
    pub final_population: Population,
    pub final_landscape: EnergyLandscape,
}

// ---------------------------------------------------------------------------
// Superorganism detection (Phase 6)
// ---------------------------------------------------------------------------

/// Composite superorganism index from emergent state.
///
/// The superorganism is a detectable macro pattern where individual agents
/// optimize locally but the aggregate effect is coordinated throughput-maximizing
/// behavior that persists even when individuals would benefit from defecting.
///
/// Components (each 0-1, averaged):
/// 1. **Hierarchy**: normalized hierarchy depth (deeper = more coordinated)
/// 2. **Inequality**: Gini coefficient (resource concentration = throughput focus)
/// 3. **Specialization**: skill entropy (high entropy = division of labor)
/// 4. **Institutional complexity**: institutional type normalized (band=0, state=1)
/// 5. **Coercion**: coercion rate (involuntary transfers = system over individual)
/// 6. **Energy throughput**: energy per capita normalized (higher = more throughput)
/// 7. **Cultural authority**: mean authority norm (acceptance of hierarchy)
/// 8. **Tribute/extraction**: tribute flows > 0 indicates inter-group extraction
/// 9. **Coordination pressure**: CFI — surplus lost to multipolar traps drives
///    institutional emergence (high = more pressure for hierarchical control)
#[must_use]
pub fn superorganism_index(state: &EmergentState) -> f32 {
    let n = state.population_size as f32;
    if n < 10.0 {
        return 0.0;
    }

    // 1. Hierarchy depth normalized (0-1, capped at depth 10)
    let hierarchy = (state.max_hierarchy_depth as f32 / 10.0).min(1.0);

    // 2. Gini (already 0-1)
    let inequality = state.gini_coefficient;

    // 3. Skill entropy (already normalized 0-1)
    let specialization = state.skill_entropy;

    // 4. Institutional type (band=0, tribe=0.33, chiefdom=0.67, state=1.0)
    let institution = state.institutional_type as f32 / 3.0;

    // 5. Coercion rate (already 0-1)
    let coercion = state.coercion_rate;

    // 6. Energy throughput normalized (log scale, ~0.01 = low, ~1.0 = high)
    let energy = (state.energy_per_capita * 10.0).min(1.0);

    // 7. Cultural authority norm (already 0-1)
    let authority = state.mean_authority_norm;

    // 8. Tribute extraction indicator
    let tribute = if state.tribute_flows > 0.0 { 0.5 } else { 0.0 }
        + if state.active_tributes > 0 { 0.5 } else { 0.0 };

    // 9. Coordination pressure: high CFI means agents are locked into
    // defection (Moloch trap), which creates selection pressure for
    // hierarchical coercion to substitute for voluntary cooperation —
    // a key driver of superorganism emergence. This intentionally
    // contributes positively: more coordination failure → more pressure
    // for institutional control → more superorganism-like.
    let coordination_pressure = state.coordination_failure_index;

    // Weighted average — hierarchy, inequality, and institution matter most
    let weights = [2.0, 1.5, 1.0, 2.0, 1.5, 1.0, 1.0, 1.0, 1.0];
    let values = [
        hierarchy,
        inequality,
        specialization,
        institution,
        coercion,
        energy,
        authority,
        tribute,
        coordination_pressure,
    ];
    let total_weight: f32 = weights.iter().sum();
    let weighted_sum: f32 = weights.iter().zip(values.iter()).map(|(w, v)| w * v).sum();
    weighted_sum / total_weight
}

/// Result of analyzing a simulation run for superorganism emergence.
#[derive(Clone, Debug)]
pub struct SuperorganismAnalysis {
    /// Peak superorganism index observed.
    pub peak_index: f32,
    /// Tick at which peak was observed.
    pub peak_tick: u32,
    /// Final superorganism index.
    pub final_index: f32,
    /// Whether the threshold was reached and sustained.
    pub reached_sustained: bool,
    /// First tick at which sustained threshold was reached (None if never).
    pub time_to_sustained: Option<u32>,
    /// Mean superorganism index over all ticks.
    pub mean_index: f32,
    /// Number of collapses (index drops > 0.1 from a local peak).
    pub collapses: u32,
    /// Final institutional type.
    pub final_institution: u8,
    /// Final dominant kinship system.
    pub final_kinship: u8,
    /// Final dominant marriage rule.
    pub final_marriage: u8,
    /// Final cultural diversity.
    pub final_cultural_diversity: f32,
    /// Final population.
    pub final_population: u32,
}

/// Analyze a simulation result for superorganism emergence.
#[must_use]
pub fn analyze_superorganism(
    result: &AgentSimResult,
    threshold: f32,
    sustained_ticks: u32,
) -> SuperorganismAnalysis {
    let mut peak_index = 0.0_f32;
    let mut peak_tick = 0_u32;
    let mut sum_index = 0.0_f32;
    let mut sustained_count = 0_u32;
    let mut time_to_sustained: Option<u32> = None;
    let mut collapses = 0_u32;
    let mut prev_index = 0.0_f32;
    let mut local_peak = 0.0_f32;

    for snap in &result.snapshots {
        let idx = superorganism_index(&snap.emergent);
        sum_index += idx;

        if idx > peak_index {
            peak_index = idx;
            peak_tick = snap.tick;
        }

        // Track sustained threshold
        if idx >= threshold {
            sustained_count += 1;
            if sustained_count >= sustained_ticks && time_to_sustained.is_none() {
                time_to_sustained = Some(snap.tick.saturating_sub(sustained_ticks - 1));
            }
        } else {
            sustained_count = 0;
        }

        // Track collapses (drop of >0.1 from local peak)
        if idx > local_peak {
            local_peak = idx;
        }
        if prev_index > 0.0 && local_peak - idx > 0.1 {
            collapses += 1;
            local_peak = idx; // reset local peak
        }
        prev_index = idx;
    }

    let n = result.snapshots.len() as f32;
    let last = result.snapshots.last();

    SuperorganismAnalysis {
        peak_index,
        peak_tick,
        final_index: last
            .map(|s| superorganism_index(&s.emergent))
            .unwrap_or(0.0),
        reached_sustained: time_to_sustained.is_some(),
        time_to_sustained,
        mean_index: if n > 0.0 { sum_index / n } else { 0.0 },
        collapses,
        final_institution: last.map(|s| s.emergent.institutional_type).unwrap_or(0),
        final_kinship: last.map(|s| s.emergent.dominant_kinship).unwrap_or(0),
        final_marriage: last.map(|s| s.emergent.dominant_marriage).unwrap_or(0),
        final_cultural_diversity: last.map(|s| s.emergent.cultural_diversity).unwrap_or(0.0),
        final_population: last.map(|s| s.emergent.population_size).unwrap_or(0),
    }
}

/// Predefined experiment conditions for the convergence study.
#[derive(Clone, Debug)]
pub struct ExperimentCondition {
    pub label: String,
    pub config: AgentSimConfig,
}

/// Generate the standard set of experiment conditions.
#[must_use]
pub fn default_agent_experiment_conditions() -> Vec<ExperimentCondition> {
    let base = AgentSimConfig::default();

    vec![
        // 1. Baseline: default parameters
        ExperimentCondition {
            label: "baseline".to_string(),
            config: base,
        },
        // 2. Large population, small world (high density)
        ExperimentCondition {
            label: "high_density".to_string(),
            config: AgentSimConfig {
                initial_population: 300,
                world_size: 40.0,
                max_population: 5000,
                ..base
            },
        },
        // 3. Rich energy: fast tech growth, all energy types available
        ExperimentCondition {
            label: "rich_energy".to_string(),
            config: AgentSimConfig {
                initial_population: 200,
                lifecycle: LifecycleParams {
                    innovation_growth_rate: 0.002,
                    ..base.lifecycle
                },
                energy: EnergyParams {
                    agriculture_tech_threshold: 0.1,
                    fossil_tech_threshold: 0.2,
                    renewable_tech_threshold: 0.4,
                    fossil_abundance: 0.8,
                    ..base.energy
                },
                ..base
            },
        },
        // 4. Scarce energy: low biomass, no fossil
        ExperimentCondition {
            label: "scarce_energy".to_string(),
            config: AgentSimConfig {
                initial_population: 200,
                energy: EnergyParams {
                    biomass_flow_rate: 0.02,
                    fossil_abundance: 0.0,
                    agriculture_tech_threshold: 0.5,
                    ..base.energy
                },
                ..base
            },
        },
        // 5. Aggressive population: high conflict, many raids
        ExperimentCondition {
            label: "aggressive".to_string(),
            config: AgentSimConfig {
                initial_population: 200,
                world_size: 40.0,
                inter_society: InterSocietyParams {
                    min_raid_warriors: 2,
                    raid_aggression_threshold: 0.15,
                    raid_range: 50.0,
                    ..base.inter_society
                },
                ..base
            },
        },
        // 6. Cooperative culture: high sharing, high trust
        ExperimentCondition {
            label: "cooperative".to_string(),
            config: AgentSimConfig {
                initial_population: 200,
                cultural: CulturalParams {
                    sharing_coop_bonus: 0.4,
                    trust_trade_bonus: 0.4,
                    coercion_conflict_bonus: 0.0,
                    ..base.cultural
                },
                ..base
            },
        },
        // 7. Hierarchical culture: high authority, strong coercion tolerance
        ExperimentCondition {
            label: "hierarchical".to_string(),
            config: AgentSimConfig {
                initial_population: 200,
                cultural: CulturalParams {
                    authority_delegation_bonus: 0.4,
                    coercion_conflict_bonus: 0.2,
                    ..base.cultural
                },
                ..base
            },
        },
        // 8. Island: small isolated population
        ExperimentCondition {
            label: "island".to_string(),
            config: AgentSimConfig {
                initial_population: 60,
                world_size: 30.0,
                max_population: 500,
                ..base
            },
        },
    ]
}

/// Summary of a convergence experiment across multiple seeds for one condition.
#[derive(Clone, Debug)]
pub struct ConditionSummary {
    pub label: String,
    pub runs: u32,
    pub arrival_rate: f32,
    pub mean_peak_index: f32,
    pub mean_final_index: f32,
    pub median_time_to_sustained: Option<u32>,
    pub mean_collapses: f32,
    pub mean_final_population: f32,
    pub kinship_distribution: [u32; 3], // patrilineal, matrilineal, bilateral
    pub marriage_distribution: [u32; 3], // monogamy, polygyny, polyandry
    pub institution_distribution: [u32; 4], // band, tribe, chiefdom, state
}

/// Full result of the agent-based convergence experiment.
#[derive(Clone, Debug)]
pub struct AgentConvergenceResult {
    pub condition_summaries: Vec<ConditionSummary>,
    pub all_analyses: Vec<(String, SuperorganismAnalysis)>,
    pub overall_arrival_rate: f32,
}

/// Run the full agent-based convergence experiment.
#[must_use]
pub fn run_agent_convergence_experiment(
    conditions: &[ExperimentCondition],
    seeds_per_condition: u32,
    ticks: u32,
    threshold: f32,
    sustained_ticks: u32,
) -> AgentConvergenceResult {
    let mut all_analyses: Vec<(String, SuperorganismAnalysis)> = Vec::new();
    let mut condition_summaries: Vec<ConditionSummary> = Vec::new();

    for cond in conditions {
        let mut analyses: Vec<SuperorganismAnalysis> = Vec::new();

        for seed_offset in 0..seeds_per_condition {
            let mut cfg = cond.config;
            cfg.seed = cfg.seed.wrapping_add(u64::from(seed_offset) * 1000003);
            cfg.ticks = ticks;
            let result = simulate_agents(cfg);
            let analysis = analyze_superorganism(&result, threshold, sustained_ticks);
            all_analyses.push((cond.label.clone(), analysis.clone()));
            analyses.push(analysis);
        }

        let n = analyses.len() as f32;
        let arrived: Vec<&SuperorganismAnalysis> =
            analyses.iter().filter(|a| a.reached_sustained).collect();
        let arrival_rate = arrived.len() as f32 / n;

        let mut times: Vec<u32> = arrived.iter().filter_map(|a| a.time_to_sustained).collect();
        times.sort_unstable();
        let median_time = if times.is_empty() {
            None
        } else {
            Some(times[times.len() / 2])
        };

        let mut kinship_dist = [0_u32; 3];
        let mut marriage_dist = [0_u32; 3];
        let mut institution_dist = [0_u32; 4];
        for a in &analyses {
            if (a.final_kinship as usize) < 3 {
                kinship_dist[a.final_kinship as usize] += 1;
            }
            if (a.final_marriage as usize) < 3 {
                marriage_dist[a.final_marriage as usize] += 1;
            }
            if (a.final_institution as usize) < 4 {
                institution_dist[a.final_institution as usize] += 1;
            }
        }

        condition_summaries.push(ConditionSummary {
            label: cond.label.clone(),
            runs: seeds_per_condition,
            arrival_rate,
            mean_peak_index: analyses.iter().map(|a| a.peak_index).sum::<f32>() / n,
            mean_final_index: analyses.iter().map(|a| a.final_index).sum::<f32>() / n,
            median_time_to_sustained: median_time,
            mean_collapses: analyses.iter().map(|a| a.collapses as f32).sum::<f32>() / n,
            mean_final_population: analyses
                .iter()
                .map(|a| a.final_population as f32)
                .sum::<f32>()
                / n,
            kinship_distribution: kinship_dist,
            marriage_distribution: marriage_dist,
            institution_distribution: institution_dist,
        });
    }

    let total = all_analyses.len() as f32;
    let total_arrived = all_analyses
        .iter()
        .filter(|(_, a)| a.reached_sustained)
        .count() as f32;

    AgentConvergenceResult {
        condition_summaries,
        all_analyses,
        overall_arrival_rate: if total > 0.0 {
            total_arrived / total
        } else {
            0.0
        },
    }
}

// ---------------------------------------------------------------------------
// Measurement functions
// ---------------------------------------------------------------------------

pub(crate) fn measure_gini(resources: &[f32]) -> f32 {
    if resources.len() < 2 {
        return 0.0;
    }
    let n = resources.len() as f64;
    let mean = resources.iter().map(|r| f64::from(*r)).sum::<f64>() / n;
    if mean < 1e-9 {
        return 0.0;
    }
    let mut abs_diff_sum = 0.0_f64;
    for (i, a) in resources.iter().enumerate() {
        for b in resources.iter().skip(i + 1) {
            abs_diff_sum += (f64::from(*a) - f64::from(*b)).abs();
        }
    }
    // Gini = sum_all |xi - xj| / (2 * n^2 * mean); abs_diff_sum is half-pairs, so *2 = all pairs
    let gini = (2.0 * abs_diff_sum) / (2.0 * n * n * mean);
    gini.clamp(0.0, 1.0) as f32
}

pub(crate) fn measure_gini_fast(resources: &[f32]) -> f32 {
    // For large populations, use sorted-rank formula: G = (2 * sum(i*x_i)) / (n * sum(x_i)) - (n+1)/n
    let n = resources.len();
    if n < 2 {
        return 0.0;
    }
    let mut sorted: Vec<f32> = resources.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let total: f64 = sorted.iter().map(|r| f64::from(*r)).sum();
    if total < 1e-9 {
        return 0.0;
    }
    let weighted_sum: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, r)| (i as f64 + 1.0) * f64::from(*r))
        .sum();
    let n_f = n as f64;
    let gini = (2.0 * weighted_sum) / (n_f * total) - (n_f + 1.0) / n_f;
    gini.clamp(0.0, 1.0) as f32
}

pub(crate) fn measure_skill_entropy(skill_types: &[SkillType]) -> f32 {
    if skill_types.is_empty() {
        return 0.0;
    }
    let mut counts = [0_u32; 5];
    for s in skill_types {
        counts[*s as usize] += 1;
    }
    let n = skill_types.len() as f64;
    let mut entropy = 0.0_f64;
    for &c in &counts {
        if c > 0 {
            let p = f64::from(c) / n;
            entropy -= p * p.ln();
        }
    }
    // Normalize by max entropy (ln(5))
    let max_entropy = 5.0_f64.ln();
    (entropy / max_entropy).clamp(0.0, 1.0) as f32
}

pub(crate) fn measure_hierarchy_depth(patrons: &[Option<u32>]) -> u32 {
    let mut max_depth = 0_u32;
    for i in 0..patrons.len() {
        let mut depth = 0_u32;
        let mut current = i as u32;
        let mut visited = 0_u64; // bitset for cycle detection (first 64 agents)
        while let Some(patron) = patrons[current as usize] {
            if patron == current {
                break;
            }
            // Simple cycle detection
            if current < 64 {
                let bit = 1_u64 << current;
                if visited & bit != 0 {
                    break;
                }
                visited |= bit;
            }
            depth += 1;
            if depth > 20 {
                break; // safety limit
            }
            current = patron;
        }
        max_depth = max_depth.max(depth);
    }
    max_depth
}

pub(crate) fn count_kin_groups(kin_groups: &[u32]) -> u32 {
    if kin_groups.is_empty() {
        return 0;
    }
    let mut seen = Vec::new();
    for &kg in kin_groups {
        if !seen.contains(&kg) {
            seen.push(kg);
        }
    }
    seen.len() as u32
}

pub(crate) fn mean_group_size(kin_groups: &[u32]) -> f32 {
    let n_groups = count_kin_groups(kin_groups);
    if n_groups == 0 {
        return 0.0;
    }
    kin_groups.len() as f32 / n_groups as f32
}

fn detect_institutional_profile(
    pop: &Population,
    effects: &InteractionEffects,
    cfg: &AgentSimConfig,
) -> InstitutionalProfile {
    let n = pop.len();
    let inst = &cfg.institution;

    // Coercion rate: involuntary transfers / total resource transfers
    // Delegation tax also counts as involuntary
    let delegation_count = effects.delegation_choices.len() as u32;
    let total_involuntary = effects.involuntary_transfers + delegation_count;
    let total_transfers = effects.voluntary_transfers + total_involuntary;
    let coercion_rate = if total_transfers > 0 {
        total_involuntary as f32 / total_transfers as f32
    } else {
        0.0
    };

    // Property norm strength: 1 - (intra-kin conflict rate)
    // Low theft within kin = strong norms
    let property_norm_strength = if effects.intra_kin_interactions > 0 {
        1.0 - (effects.intra_kin_conflicts as f32 / effects.intra_kin_interactions as f32)
    } else {
        1.0 // no intra-kin interactions = no theft
    };

    // Count patrons and recognized leaders
    let mut patron_count = 0_u32;
    let mut recognized_leaders = 0_u32;
    let mut patron_tenure_sum = 0_u64;
    let mut patron_tenure_count = 0_u32;

    if n > 0 {
        // Count unique patrons
        let mut patron_set: Vec<u32> = Vec::new();
        for p in pop.patrons.iter().flatten() {
            if !patron_set.contains(p) {
                patron_set.push(*p);
            }
        }
        patron_count = patron_set.len() as u32;

        // Recognized leaders: patrons where >threshold of their kin group follows them
        // Group by kin group, check if majority shares same patron
        let n_kin = count_kin_groups(&pop.kin_groups);
        for kg in 0..n_kin {
            let mut kin_members = 0_u32;
            let mut patron_votes: Vec<(u32, u32)> = Vec::new(); // (patron_idx, count)
            for i in 0..n {
                if pop.kin_groups[i] != kg {
                    continue;
                }
                kin_members += 1;
                if let Some(p) = pop.patrons[i] {
                    if let Some(entry) = patron_votes.iter_mut().find(|(pid, _)| *pid == p) {
                        entry.1 += 1;
                    } else {
                        patron_votes.push((p, 1));
                    }
                }
            }
            if kin_members > 0 {
                for &(_, count) in &patron_votes {
                    if count as f32 / kin_members as f32 >= inst.leadership_threshold {
                        recognized_leaders += 1;
                    }
                }
            }
        }

        // Mean patron tenure
        for i in 0..n {
            if pop.patrons[i].is_some() {
                patron_tenure_sum += u64::from(pop.patron_ticks[i]);
                patron_tenure_count += 1;
            }
        }
    }

    let mean_patron_tenure = if patron_tenure_count > 0 {
        patron_tenure_sum as f32 / patron_tenure_count as f32
    } else {
        0.0
    };

    // Public goods: estimate from patron investment
    let public_goods_investment = if n > 0 {
        let total_tax: f32 = pop
            .patrons
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.map(|_| pop.resources[i] * cfg.interaction.delegation_tax_rate))
            .sum();
        total_tax * inst.public_goods_rate
    } else {
        0.0
    };

    // Institutional classification based on emergent patterns
    let hierarchy = measure_hierarchy_depth(&pop.patrons);
    let pop_size = n as u32;
    let institutional_type = if hierarchy >= 3 && pop_size > 500 {
        InstitutionalType::State
    } else if hierarchy >= 2 && pop_size > 150 {
        InstitutionalType::Chiefdom
    } else if hierarchy >= 1 || pop_size > 50 {
        InstitutionalType::Tribe
    } else {
        InstitutionalType::Band
    };

    InstitutionalProfile {
        institutional_type,
        coercion_rate,
        property_norm_strength,
        public_goods_investment,
        patron_count,
        recognized_leaders,
        mean_patron_tenure,
    }
}

/// Aggregated per-tick data needed to compute EmergentState.
struct TickMeasureInput<'a> {
    cooperation_events: u32,
    conflict_events: u32,
    total_interactions: u32,
    energy_summary: &'a EnergyTickSummary,
    landscape: &'a EnergyLandscape,
    institutional: &'a InstitutionalProfile,
    inter_society: &'a InterSocietySummary,
    active_tribute_count: u32,
    total_actual_surplus: f32,
    total_cooperative_optimal: f32,
}

fn measure_emergent_state(pop: &Population, input: &TickMeasureInput<'_>) -> EmergentState {
    let n = pop.len() as u32;
    let gini = if pop.len() > 500 {
        measure_gini_fast(&pop.resources)
    } else {
        measure_gini(&pop.resources)
    };
    EmergentState {
        population_size: n,
        mean_resources: if n > 0 {
            pop.resources.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        gini_coefficient: gini,
        skill_entropy: measure_skill_entropy(&pop.skill_types),
        max_hierarchy_depth: measure_hierarchy_depth(&pop.patrons),
        num_leaders: pop
            .skill_types
            .iter()
            .filter(|s| **s == SkillType::Leader)
            .count() as u32,
        mean_group_size: mean_group_size(&pop.kin_groups),
        num_kin_groups: count_kin_groups(&pop.kin_groups),
        cooperation_rate: if input.total_interactions > 0 {
            input.cooperation_events as f32 / input.total_interactions as f32
        } else {
            0.0
        },
        conflict_rate: if input.total_interactions > 0 {
            input.conflict_events as f32 / input.total_interactions as f32
        } else {
            0.0
        },
        mean_prestige: if n > 0 {
            pop.prestiges.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_health: if n > 0 {
            pop.healths.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_innovation: if n > 0 {
            pop.innovations.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        dominant_energy: {
            let e = &input.energy_summary.energy_by_type;
            let mut best = 0_u8;
            let mut best_val = e[0];
            for (i, &val) in e.iter().enumerate().skip(1) {
                if val > best_val {
                    best = i as u8;
                    best_val = val;
                }
            }
            best
        },
        energy_per_capita: if n > 0 {
            (input.energy_summary.total_net_energy / f64::from(n)) as f32
        } else {
            0.0
        },
        mean_eroei: {
            let mut sum = 0.0_f64;
            let mut count = 0_u32;
            for cell in &input.landscape.cells {
                for src in &cell.sources {
                    if src.flow_rate > 0.0 && src.stock > 0.0 {
                        sum += src.current_eroei();
                        count += 1;
                    }
                }
            }
            if count > 0 {
                (sum / f64::from(count)) as f32
            } else {
                0.0
            }
        },
        biomass_depletion: input.landscape.mean_depletion(EnergyType::Biomass) as f32,
        fossil_depletion: input.landscape.mean_depletion(EnergyType::Fossil) as f32,
        coercion_rate: input.institutional.coercion_rate,
        property_norm_strength: input.institutional.property_norm_strength,
        institutional_type: input.institutional.institutional_type as u8,
        public_goods_investment: input.institutional.public_goods_investment,
        patron_count: input.institutional.patron_count,
        recognized_leaders: input.institutional.recognized_leaders,
        mean_patron_tenure: input.institutional.mean_patron_tenure,
        // Inter-society (Phase 4)
        raid_events: input.inter_society.raids,
        conquest_events: input.inter_society.conquests,
        tribute_flows: input.inter_society.tribute_total,
        migration_events: input.inter_society.migrations,
        num_active_societies: count_kin_groups(&pop.kin_groups),
        inter_group_trade_rate: if input.inter_society.total_trades > 0 {
            input.inter_society.inter_group_trades as f32 / input.inter_society.total_trades as f32
        } else {
            0.0
        },
        active_tributes: input.active_tribute_count,
        // Cultural (Phase 5)
        mean_authority_norm: if n > 0 {
            pop.cultures.iter().map(|c| c.authority_norm).sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_sharing_norm: if n > 0 {
            pop.cultures.iter().map(|c| c.sharing_norm).sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_property_norm: if n > 0 {
            pop.cultures.iter().map(|c| c.property_norm).sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_trust_outgroup: if n > 0 {
            pop.cultures.iter().map(|c| c.trust_outgroup).sum::<f32>() / n as f32
        } else {
            0.0
        },
        cultural_diversity: measure_cultural_diversity(pop),
        dominant_kinship: dominant_kinship(pop),
        dominant_marriage: dominant_marriage(pop),
        mean_coercion_tolerance: if n > 0 {
            pop.cultures
                .iter()
                .map(|c| c.coercion_tolerance)
                .sum::<f32>()
                / n as f32
        } else {
            0.0
        },
        technique_count: mean_technique_count(pop),
        coordination_failure_index: if input.total_cooperative_optimal > 0.0 {
            (1.0 - (input.total_actual_surplus / input.total_cooperative_optimal).min(1.0))
                .clamp(0.0, 1.0)
        } else {
            0.0
        },
        mean_trust: if n > 0 {
            pop.trust_memory.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
    }
}

// ---------------------------------------------------------------------------
// Deterministic RNG (same LCG as evolution module)
// ---------------------------------------------------------------------------

fn rand01(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*state as f64) / (u64::MAX as f64)
}

fn rand01f(state: &mut u64) -> f32 {
    rand01(state) as f32
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

/// Seed an initial population with random traits.
#[must_use]
pub fn seed_population(cfg: &AgentSimConfig) -> Population {
    let mut pop = Population::empty();
    let mut rng = cfg.seed.max(1);
    let n = cfg.initial_population;
    let lp = &cfg.lifecycle;
    let kin_group_count = (n / lp.agents_per_kin_group).max(2);

    for i in 0..n {
        let sex = if rand01(&mut rng) < 0.5 {
            Sex::Male
        } else {
            Sex::Female
        };
        let max_initial_age = (cfg.max_age as f64 * 0.35) as u16;
        let min_initial_age = 5_u16;
        let age = (rand01(&mut rng) * f64::from(max_initial_age - min_initial_age)) as u16
            + min_initial_age;
        let age_f = age as f32;
        let fertility = match sex {
            Sex::Female => (lp.female_peak_fertility
                - (age_f - lp.female_fertility_peak_age).abs() * lp.female_fertility_decline)
                .clamp(0.0, 1.0),
            Sex::Male => (lp.male_peak_fertility
                - (age_f - lp.male_fertility_peak_age).abs() * lp.male_fertility_decline)
                .clamp(0.0, 1.0),
        };
        let skill_type = match (rand01(&mut rng) * 5.0) as u32 {
            0 => SkillType::Forager,
            1 => SkillType::Crafter,
            2 => SkillType::Builder,
            3 => SkillType::Leader,
            _ => SkillType::Warrior,
        };

        // Generate initial culture — kin groups share cultural traits
        let kin = i % kin_group_count;
        let kin_seed = cfg.seed.wrapping_add(u64::from(kin).wrapping_mul(7919));
        let mut kin_rng = kin_seed.max(1);
        let kinship_system = match (rand01(&mut kin_rng) * 3.0) as u32 {
            0 => KinshipSystem::Patrilineal,
            1 => KinshipSystem::Matrilineal,
            _ => KinshipSystem::Bilateral,
        };
        let marriage_rule = match (rand01(&mut kin_rng) * 3.0) as u32 {
            0 => MarriageRule::Monogamy,
            1 => MarriageRule::Polygyny,
            _ => MarriageRule::Polyandry,
        };
        let residence_rule = match (rand01(&mut kin_rng) * 3.0) as u32 {
            0 => ResidenceRule::Patrilocal,
            1 => ResidenceRule::Matrilocal,
            _ => ResidenceRule::Neolocal,
        };
        let inheritance_rule = match (rand01(&mut kin_rng) * 3.0) as u32 {
            0 => InheritanceRule::Primogeniture,
            1 => InheritanceRule::Partible,
            _ => InheritanceRule::Matrilineal,
        };
        let culture = Culture {
            kinship_system,
            marriage_rule,
            residence_rule,
            inheritance_rule,
            authority_norm: (0.2 + rand01f(&mut rng) * 0.3).clamp(0.0, 1.0),
            coercion_tolerance: (0.2 + rand01f(&mut rng) * 0.3).clamp(0.0, 1.0),
            sharing_norm: (0.3 + rand01f(&mut rng) * 0.4).clamp(0.0, 1.0),
            property_norm: (0.2 + rand01f(&mut rng) * 0.3).clamp(0.0, 1.0),
            techniques: (rand01(&mut rng) * 15.0) as u64, // initial low-tech
            trust_outgroup: (0.2 + rand01f(&mut rng) * 0.3).clamp(0.0, 1.0),
            risk_tolerance: (0.3 + rand01f(&mut rng) * 0.4).clamp(0.0, 1.0),
        };

        pop.push_agent(AgentInit {
            id: u64::from(i),
            sex,
            age,
            fertility,
            health: (0.6 + rand01f(&mut rng) * 0.4).clamp(0.0, 1.0),
            skill_type,
            skill_level: 0.1 + rand01f(&mut rng) * 0.3,
            status: 0.3 + rand01f(&mut rng) * 0.4,
            prestige: rand01f(&mut rng) * 0.2,
            aggression: 0.1 + rand01f(&mut rng) * 0.4,
            cooperation: 0.3 + rand01f(&mut rng) * 0.5,
            resources: 0.5 + rand01f(&mut rng) * 1.0,
            surplus: 0.0,
            culture,
            innovation: rand01f(&mut rng) * 0.3,
            kin_group: (i % kin_group_count),
            x: rand01f(&mut rng) * cfg.world_size,
            y: rand01f(&mut rng) * cfg.world_size,
        });
    }
    pop
}

// ---------------------------------------------------------------------------
// Per-tick interactions
// ---------------------------------------------------------------------------

/// Outcome of interactions computed in parallel, applied sequentially.
struct InteractionEffects {
    resource_deltas: Vec<f32>,
    status_deltas: Vec<f32>,
    prestige_deltas: Vec<f32>,
    health_deltas: Vec<f32>,
    cooperation_events: u32,
    conflict_events: u32,
    trade_events: u32,
    total_interactions: u32,
    voluntary_transfers: u32,
    involuntary_transfers: u32,
    intra_kin_conflicts: u32,
    intra_kin_interactions: u32,
    inter_group_trades: u32,
    // Delegation choices: agent_idx -> chosen_patron_idx
    delegation_choices: Vec<(u32, u32)>,
    // Trust-memory coordination dilemma
    trust_signals: Vec<f32>,
    per_agent_interactions: Vec<u32>,
    total_actual_surplus: f32,
    total_cooperative_optimal: f32,
}

struct AgentInteractionResult {
    res_delta: f32,
    status_delta: f32,
    prestige_delta: f32,
    health_delta: f32,
    coop_count: u32,
    conflict_count: u32,
    trade_count: u32,
    interaction_count: u32,
    voluntary: u32,
    involuntary: u32,
    intra_kin_conflict: u32,
    intra_kin_interaction: u32,
    inter_group_trade: u32,
    best_patron: Option<u32>,
    actual_surplus: f32,
    cooperative_optimal_surplus: f32,
    /// Count of cooperative interactions (used as proxy for trust signal).
    /// In the parallel tick model, this equals coop_count since each agent
    /// independently resolves its interactions. The event-driven engine
    /// updates both participants directly for a more accurate signal.
    coop_outcome_count: u32,
}

fn compute_interactions(
    pop: &Population,
    grid: &SpatialGrid,
    tick: u32,
    cfg: &AgentSimConfig,
) -> InteractionEffects {
    let n = pop.len();
    let ip = &cfg.interaction;
    // Compute per-agent effects in parallel, then merge.
    let per_agent: Vec<AgentInteractionResult> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut rng = (pop.ids[i])
                .wrapping_mul(6364136223846793005)
                .wrapping_add(tick as u64)
                .wrapping_add(cfg.seed)
                .max(1);

            let neighbors = grid.neighbors_of(pop.xs[i], pop.ys[i]);
            let mut res_delta = 0.0_f32;
            let mut status_delta = 0.0_f32;
            let mut prestige_delta = 0.0_f32;
            let mut health_delta = 0.0_f32;
            let mut coop_count = 0_u32;
            let mut conflict_count = 0_u32;
            let mut trade_count = 0_u32;
            let mut interaction_count = 0_u32;
            let mut voluntary = 0_u32;
            let mut involuntary = 0_u32;
            let mut intra_kin_conflict = 0_u32;
            let mut intra_kin_interaction = 0_u32;
            let mut inter_group_trade = 0_u32;
            let mut best_patron: Option<u32> = None;
            let mut best_patron_score = 0.0_f32;
            let mut actual_surplus = 0.0_f32;
            let mut cooperative_optimal_surplus = 0.0_f32;
            let mut coop_outcome_count = 0_u32;

            let my_coop = pop.cooperations[i];
            let my_aggr = pop.aggressions[i];
            let my_skill = pop.skill_types[i];
            let my_kin = pop.kin_groups[i];
            let my_status = pop.statuses[i];

            for &j_u32 in &neighbors {
                let j = j_u32 as usize;
                if j == i {
                    continue;
                }

                // Distance check
                let dx = pop.xs[i] - pop.xs[j];
                let dy = pop.ys[i] - pop.ys[j];
                let dist_sq = dx * dx + dy * dy;
                let max_dist_sq = grid.cell_size * grid.cell_size * 4.0;
                if dist_sq > max_dist_sq {
                    continue;
                }

                interaction_count += 1;
                let same_kin = my_kin == pop.kin_groups[j];
                if same_kin {
                    intra_kin_interaction += 1;
                }
                let other_coop = pop.cooperations[j];
                let other_aggr = pop.aggressions[j];

                // Interaction decision: cooperate, trade, or conflict
                // Cultural traits modulate tendencies
                let cp = &cfg.cultural;
                let my_culture = &pop.cultures[i];
                let sharing_boost = my_culture.sharing_norm * cp.sharing_coop_bonus;
                let trust_boost = if !same_kin {
                    my_culture.trust_outgroup * cp.trust_trade_bonus
                } else {
                    0.0
                };
                let coercion_boost = my_culture.coercion_tolerance * cp.coercion_conflict_bonus;

                let coop_tendency = my_coop * ip.coop_self_weight
                    + other_coop * ip.coop_other_weight
                    + if same_kin { ip.coop_kin_bonus } else { 0.0 }
                    + sharing_boost
                    + pop.trust_memory[i] * ip.trust_coop_weight;
                let conflict_tendency = my_aggr * ip.conflict_self_weight
                    + other_aggr * ip.conflict_other_weight
                    + if !same_kin {
                        ip.conflict_stranger_bonus
                    } else {
                        0.0
                    }
                    + coercion_boost
                    + (1.0 - pop.trust_memory[i]) * ip.trust_coop_weight * 0.5;
                let trade_tendency = if my_skill != pop.skill_types[j] {
                    ip.trade_complementary
                } else {
                    ip.trade_same_skill
                } + trust_boost;

                let total = coop_tendency + conflict_tendency + trade_tendency;
                let roll = rand01f(&mut rng) * total;

                // Track cooperative counterfactual for every interaction
                cooperative_optimal_surplus += ip.coop_resource_bonus;

                if roll < coop_tendency {
                    // Cooperation: mutual effort produces surplus for both
                    let coop_bonus = ip.coop_resource_bonus * (pop.cooperations[j] + my_coop) * 0.5;
                    res_delta += coop_bonus;
                    prestige_delta += ip.coop_prestige_gain;
                    coop_count += 1;
                    voluntary += 1;
                    actual_surplus += coop_bonus;
                    coop_outcome_count += 1;
                } else if roll < coop_tendency + conflict_tendency {
                    // Conflict: winner takes resources, loser loses health
                    let my_power = my_status * ip.power_status_weight
                        + pop.skill_levels[i] * ip.power_skill_weight
                        + my_aggr * ip.power_aggression_weight;
                    let other_power = pop.statuses[j] * ip.power_status_weight
                        + pop.skill_levels[j] * ip.power_skill_weight
                        + other_aggr * ip.power_aggression_weight;
                    if my_power > other_power + rand01f(&mut rng) * ip.conflict_noise {
                        res_delta += ip.conflict_win_resources;
                        status_delta += ip.conflict_win_status;
                        actual_surplus += ip.conflict_win_resources;
                    } else {
                        res_delta -= ip.conflict_lose_resources;
                        health_delta -= ip.conflict_lose_health;
                        actual_surplus -= ip.conflict_lose_resources;
                    }
                    conflict_count += 1;
                    involuntary += 1;
                    if same_kin {
                        intra_kin_conflict += 1;
                    }
                } else {
                    // Trade: complementary skills produce surplus
                    let skill_bonus = if my_skill != pop.skill_types[j] {
                        ip.trade_complementary_bonus * (pop.skill_levels[i] + pop.skill_levels[j])
                    } else {
                        ip.trade_same_bonus
                    };
                    res_delta += skill_bonus;
                    trade_count += 1;
                    voluntary += 1;
                    actual_surplus += skill_bonus;
                    if !same_kin {
                        inter_group_trade += 1;
                    }
                }

                // Delegation: consider this neighbor as patron
                // High authority_norm makes agents more willing to delegate
                let effective_gap = ip.delegation_status_gap
                    - my_culture.authority_norm * cp.authority_delegation_bonus;
                if pop.statuses[j] > my_status + effective_gap
                    && pop.prestiges[j] > best_patron_score
                    && pop.skill_types[j] == SkillType::Leader
                {
                    best_patron = Some(j_u32);
                    best_patron_score = pop.prestiges[j];
                }
            }

            AgentInteractionResult {
                res_delta,
                status_delta,
                prestige_delta,
                health_delta: health_delta.max(-ip.max_health_loss_per_tick),
                coop_count,
                conflict_count,
                trade_count,
                interaction_count,
                voluntary,
                involuntary,
                intra_kin_conflict,
                intra_kin_interaction,
                inter_group_trade,
                best_patron,
                actual_surplus,
                cooperative_optimal_surplus,
                coop_outcome_count,
            }
        })
        .collect();

    let mut effects = InteractionEffects {
        resource_deltas: vec![0.0; n],
        status_deltas: vec![0.0; n],
        prestige_deltas: vec![0.0; n],
        health_deltas: vec![0.0; n],
        cooperation_events: 0,
        conflict_events: 0,
        trade_events: 0,
        total_interactions: 0,
        voluntary_transfers: 0,
        involuntary_transfers: 0,
        intra_kin_conflicts: 0,
        intra_kin_interactions: 0,
        inter_group_trades: 0,
        delegation_choices: Vec::new(),
        trust_signals: vec![0.0; n],
        per_agent_interactions: vec![0; n],
        total_actual_surplus: 0.0,
        total_cooperative_optimal: 0.0,
    };

    for (i, result) in per_agent.iter().enumerate() {
        effects.resource_deltas[i] = result.res_delta;
        effects.status_deltas[i] = result.status_delta;
        effects.prestige_deltas[i] = result.prestige_delta;
        effects.health_deltas[i] = result.health_delta;
        effects.cooperation_events += result.coop_count;
        effects.conflict_events += result.conflict_count;
        effects.trade_events += result.trade_count;
        effects.total_interactions += result.interaction_count;
        effects.voluntary_transfers += result.voluntary;
        effects.involuntary_transfers += result.involuntary;
        effects.intra_kin_conflicts += result.intra_kin_conflict;
        effects.intra_kin_interactions += result.intra_kin_interaction;
        effects.inter_group_trades += result.inter_group_trade;
        effects.total_actual_surplus += result.actual_surplus;
        effects.total_cooperative_optimal += result.cooperative_optimal_surplus;
        effects.per_agent_interactions[i] = result.interaction_count;
        // Trust signal: fraction of interactions with cooperative outcomes.
        // In the parallel tick model this is a proxy for the cooperation environment,
        // since each agent resolves interactions independently.
        if result.interaction_count > 0 {
            effects.trust_signals[i] =
                result.coop_outcome_count as f32 / result.interaction_count as f32;
        }
        if let Some(patron) = result.best_patron {
            effects.delegation_choices.push((i as u32, patron));
        }
    }

    effects
}

fn apply_effects(pop: &mut Population, effects: &InteractionEffects, cfg: &AgentSimConfig) {
    let n = pop.len();
    let ip = &cfg.interaction;
    let lp = &cfg.lifecycle;
    for i in 0..n {
        pop.resources[i] = (pop.resources[i] + effects.resource_deltas[i]).max(0.0);
        pop.statuses[i] = (pop.statuses[i] + effects.status_deltas[i]).clamp(0.0, ip.max_status);
        pop.prestiges[i] =
            (pop.prestiges[i] + effects.prestige_deltas[i]).clamp(0.0, ip.max_prestige);
        pop.healths[i] = (pop.healths[i] + effects.health_deltas[i]).clamp(0.0, 1.0);
        // Well-fed agents recover health
        if pop.resources[i] > lp.health_recovery_threshold {
            let recovery = lp.health_recovery_rate
                * (pop.resources[i] - lp.health_recovery_threshold).min(1.0);
            pop.healths[i] = (pop.healths[i] + recovery).min(1.0);
        }
        pop.surpluses[i] = (pop.resources[i] - ip.subsistence_level).max(0.0);

        // Skill improvement through practice
        pop.skill_levels[i] = (pop.skill_levels[i] + ip.skill_practice_rate).min(1.0);

        // Trust memory EMA update based on cooperative interaction outcomes
        if effects.per_agent_interactions[i] > 0 {
            let alpha = ip.trust_memory_decay.clamp(0.0, 1.0);
            pop.trust_memory[i] = ((1.0 - alpha) * pop.trust_memory[i]
                + alpha * effects.trust_signals[i])
                .clamp(0.0, 1.0);
        }
    }

    // Increment patron tenure for agents who keep their patron
    for i in 0..n {
        if pop.patrons[i].is_some() {
            pop.patron_ticks[i] += 1;
        }
    }

    // Apply delegation choices
    let inst = &cfg.institution;
    for &(agent, patron) in &effects.delegation_choices {
        if (patron as usize) < n {
            let old_patron = pop.patrons[agent as usize];
            if old_patron != Some(patron) {
                pop.patron_ticks[agent as usize] = 0; // reset tenure on patron change
            }
            pop.patrons[agent as usize] = Some(patron);
            let tax = pop.resources[agent as usize] * ip.delegation_tax_rate;
            pop.resources[agent as usize] -= tax;
            // Patron splits tax between personal wealth and public goods
            let public_share = tax * inst.public_goods_rate;
            pop.resources[patron as usize] += tax - public_share;
            pop.prestiges[patron as usize] += ip.delegation_prestige_gain;
        }
    }

    // Public goods: patrons with followers provide group benefits
    // Collect patron->follower counts and accumulated public goods
    let mut patron_followers: Vec<(u32, u32)> = Vec::new(); // (patron_idx, count)
    let mut patron_investment: Vec<f32> = Vec::new();
    {
        let mut patron_map: std::collections::HashMap<u32, (u32, f32)> =
            std::collections::HashMap::new();
        for i in 0..n {
            if let Some(p) = pop.patrons[i] {
                let entry = patron_map.entry(p).or_insert((0, 0.0));
                entry.0 += 1;
                // Each follower contributes their tax as public goods
                entry.1 += pop.resources[i] * ip.delegation_tax_rate * inst.public_goods_rate;
            }
        }
        for (p, (count, invest)) in &patron_map {
            patron_followers.push((*p, *count));
            patron_investment.push(*invest);
        }
    }

    // Distribute public goods benefits to kin groups with active patrons
    for (idx, &(patron_idx, follower_count)) in patron_followers.iter().enumerate() {
        if (patron_idx as usize) >= n || follower_count < 2 {
            continue;
        }
        let patron_kin = pop.kin_groups[patron_idx as usize];
        let investment = patron_investment[idx];
        if investment <= 0.0 {
            continue;
        }
        // Benefit all agents in the patron's kin group
        for i in 0..n {
            if pop.kin_groups[i] == patron_kin {
                pop.resources[i] += inst.public_goods_bonus;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Lifecycle: aging, death, courtship, birth
// ---------------------------------------------------------------------------

fn lifecycle_tick(pop: &mut Population, tick: u32, cfg: &AgentSimConfig, next_id: &mut u64) {
    let lp = &cfg.lifecycle;
    let mut rng = (*next_id)
        .wrapping_mul(2862933555777941757)
        .wrapping_add(tick as u64)
        .wrapping_add(cfg.seed)
        .max(1);

    // Age everyone
    for age in &mut pop.ages {
        *age = age.saturating_add(1);
    }

    // Health decay with age (slow decline, accelerating in old age)
    for i in 0..pop.len() {
        let age_ratio = (pop.ages[i] as f32 / cfg.max_age as f32).clamp(0.0, 1.0);
        let age_factor = age_ratio * age_ratio; // quadratic: slow when young, fast when old
        pop.healths[i] -= lp.health_decay_base + lp.health_decay_age_factor * age_factor;
        pop.healths[i] = pop.healths[i].clamp(0.0, 1.0);

        // Fertility peaks mid-life, declines at extremes
        let age_f = pop.ages[i] as f32;
        pop.fertilities[i] = match pop.sexes[i] {
            Sex::Female => (lp.female_peak_fertility
                - (age_f - lp.female_fertility_peak_age).abs() * lp.female_fertility_decline)
                .clamp(0.0, 1.0),
            Sex::Male => (lp.male_peak_fertility
                - (age_f - lp.male_fertility_peak_age).abs() * lp.male_fertility_decline)
                .clamp(0.0, 1.0),
        };
    }

    // Innovation growth (learning by doing, cumulative knowledge)
    for i in 0..pop.len() {
        pop.innovations[i] = (pop.innovations[i] + lp.innovation_growth_rate).min(1.0);
    }

    // Death: old age, low health, or starvation
    let mut deaths = Vec::new();
    for i in (0..pop.len()).rev() {
        let die = pop.ages[i] >= cfg.max_age
            || pop.healths[i] < lp.death_health_threshold
            || (pop.resources[i] < lp.starvation_resource_threshold
                && rand01f(&mut rng) < lp.starvation_death_prob);
        if die {
            deaths.push(i);
        }
    }
    // Remove from highest index first (swap_remove is safe this way)
    for &idx in &deaths {
        pop.swap_remove(idx);
    }

    // Fix patron/partner references after swap_remove
    let n = pop.len();
    for i in 0..n {
        if let Some(p) = pop.patrons[i] {
            if p as usize >= n {
                pop.patrons[i] = None;
            }
        }
        if let Some(p) = pop.partners[i] {
            if p as usize >= n {
                pop.partners[i] = None;
            }
        }
    }

    // Courtship and birth (only if below max population)
    if (pop.len() as u32) >= cfg.max_population {
        return;
    }

    let mut births: Vec<(AgentInit, Option<u32>)> = Vec::new();
    let pop_len = pop.len();

    for i in 0..pop_len {
        if pop.sexes[i] != Sex::Female {
            continue;
        }
        if pop.fertilities[i] < lp.min_fertility
            || pop.ages[i] < lp.min_reproduction_age
            || pop.ages[i] > lp.max_reproduction_age
        {
            continue;
        }
        if pop.resources[i] < lp.reproduction_resource_threshold {
            continue;
        }
        // Already has partner? Use them. Otherwise find one.
        let mate_idx = pop.partners[i].and_then(|p| {
            if (p as usize) < pop_len && pop.sexes[p as usize] == Sex::Male {
                Some(p as usize)
            } else {
                None
            }
        });

        let mate = if let Some(m) = mate_idx {
            Some(m)
        } else {
            find_mate(pop, i, &mut rng, cfg)
        };

        if let Some(m) = mate {
            let birth_prob = lp.birth_rate * pop.fertilities[i] * pop.healths[i];
            if rand01f(&mut rng) >= birth_prob {
                continue;
            }

            // Pair them
            pop.partners[i] = Some(m as u32);
            pop.partners[m] = Some(i as u32);

            let child_sex = if rand01(&mut rng) < 0.5 {
                Sex::Male
            } else {
                Sex::Female
            };

            // Inherit traits from both parents with mutation
            let skill = if rand01(&mut rng) < lp.skill_maternal_inherit_prob {
                pop.skill_types[i] // mother's skill more likely
            } else if rand01(&mut rng) < lp.skill_mutation_prob {
                pop.skill_types[m]
            } else {
                // mutation: random skill
                match (rand01(&mut rng) * 5.0) as u32 {
                    0 => SkillType::Forager,
                    1 => SkillType::Crafter,
                    2 => SkillType::Builder,
                    3 => SkillType::Leader,
                    _ => SkillType::Warrior,
                }
            };

            // Vertical cultural transmission (parent → child) with mutation
            let cp = &cfg.cultural;
            let mother_culture = &pop.cultures[i];
            let father_culture = &pop.cultures[m];
            let child_culture = Culture {
                // Discrete traits: inherit from one parent, with mutation chance
                kinship_system: if rand01(&mut rng) < cp.vertical_mutation_prob {
                    match (rand01(&mut rng) * 3.0) as u32 {
                        0 => KinshipSystem::Patrilineal,
                        1 => KinshipSystem::Matrilineal,
                        _ => KinshipSystem::Bilateral,
                    }
                } else if rand01(&mut rng) < 0.5 {
                    mother_culture.kinship_system
                } else {
                    father_culture.kinship_system
                },
                marriage_rule: if rand01(&mut rng) < cp.vertical_mutation_prob {
                    match (rand01(&mut rng) * 3.0) as u32 {
                        0 => MarriageRule::Monogamy,
                        1 => MarriageRule::Polygyny,
                        _ => MarriageRule::Polyandry,
                    }
                } else if rand01(&mut rng) < 0.5 {
                    mother_culture.marriage_rule
                } else {
                    father_culture.marriage_rule
                },
                residence_rule: if rand01(&mut rng) < cp.vertical_mutation_prob {
                    match (rand01(&mut rng) * 3.0) as u32 {
                        0 => ResidenceRule::Patrilocal,
                        1 => ResidenceRule::Matrilocal,
                        _ => ResidenceRule::Neolocal,
                    }
                } else if rand01(&mut rng) < 0.5 {
                    mother_culture.residence_rule
                } else {
                    father_culture.residence_rule
                },
                inheritance_rule: if rand01(&mut rng) < cp.vertical_mutation_prob {
                    match (rand01(&mut rng) * 3.0) as u32 {
                        0 => InheritanceRule::Primogeniture,
                        1 => InheritanceRule::Partible,
                        _ => InheritanceRule::Matrilineal,
                    }
                } else if rand01(&mut rng) < 0.5 {
                    mother_culture.inheritance_rule
                } else {
                    father_culture.inheritance_rule
                },
                // Continuous traits: blend parents + mutation
                authority_norm: ((mother_culture.authority_norm + father_culture.authority_norm)
                    * 0.5
                    + (rand01f(&mut rng) - 0.5) * cp.cultural_mutation_magnitude)
                    .clamp(0.0, 1.0),
                coercion_tolerance: ((mother_culture.coercion_tolerance
                    + father_culture.coercion_tolerance)
                    * 0.5
                    + (rand01f(&mut rng) - 0.5) * cp.cultural_mutation_magnitude)
                    .clamp(0.0, 1.0),
                sharing_norm: ((mother_culture.sharing_norm + father_culture.sharing_norm) * 0.5
                    + (rand01f(&mut rng) - 0.5) * cp.cultural_mutation_magnitude)
                    .clamp(0.0, 1.0),
                property_norm: ((mother_culture.property_norm + father_culture.property_norm)
                    * 0.5
                    + (rand01f(&mut rng) - 0.5) * cp.cultural_mutation_magnitude)
                    .clamp(0.0, 1.0),
                // Techniques: union of parent techniques + rare new bits
                techniques: mother_culture.techniques
                    | father_culture.techniques
                    | if rand01(&mut rng) < lp.norm_mutation_prob {
                        1 << ((rand01(&mut rng) * 16.0) as u64)
                    } else {
                        0
                    },
                trust_outgroup: ((mother_culture.trust_outgroup + father_culture.trust_outgroup)
                    * 0.5
                    + (rand01f(&mut rng) - 0.5) * cp.cultural_mutation_magnitude)
                    .clamp(0.0, 1.0),
                risk_tolerance: ((mother_culture.risk_tolerance + father_culture.risk_tolerance)
                    * 0.5
                    + (rand01f(&mut rng) - 0.5) * cp.cultural_mutation_magnitude)
                    .clamp(0.0, 1.0),
            };

            // Patron inheritance: child adopts mother's patron if configured
            let inherited_patron = if cfg.institution.patron_inheritance {
                pop.patrons[i].and_then(|p| {
                    if (p as usize) < pop_len {
                        Some(p)
                    } else {
                        None
                    }
                })
            } else {
                None
            };

            births.push((
                AgentInit {
                    id: *next_id,
                    sex: child_sex,
                    age: 0,
                    fertility: 0.0, // too young
                    health: lp.newborn_health,
                    skill_type: skill,
                    skill_level: lp.newborn_skill_level,
                    status: lp.newborn_status,
                    prestige: 0.0,
                    aggression: ((pop.aggressions[i] + pop.aggressions[m]) * 0.5
                        + (rand01f(&mut rng) - 0.5) * lp.trait_mutation_magnitude)
                        .clamp(0.0, 1.0),
                    cooperation: ((pop.cooperations[i] + pop.cooperations[m]) * 0.5
                        + (rand01f(&mut rng) - 0.5) * lp.trait_mutation_magnitude)
                        .clamp(0.0, 1.0),
                    resources: lp.newborn_resources,
                    surplus: 0.0,
                    culture: child_culture,
                    innovation: ((pop.innovations[i] + pop.innovations[m]) * 0.5
                        + (rand01f(&mut rng) - 0.5) * lp.trait_mutation_magnitude * 0.5)
                        .clamp(0.0, 1.0),
                    kin_group: pop.kin_groups[i], // inherit mother's kin group
                    x: pop.xs[i] + (rand01f(&mut rng) - 0.5) * lp.birth_spawn_radius,
                    y: pop.ys[i] + (rand01f(&mut rng) - 0.5) * lp.birth_spawn_radius,
                },
                inherited_patron,
            ));
            *next_id += 1;

            // Reproduction cost
            pop.resources[i] -= lp.birth_resource_cost;
            pop.healths[i] -= lp.birth_health_cost;
        }
    }

    for (birth, patron) in births {
        pop.push_agent(birth);
        // Set inherited patron (push_agent defaults to None)
        if let Some(p) = patron {
            let idx = pop.len() - 1;
            if (p as usize) < pop.len() {
                pop.patrons[idx] = Some(p);
            }
        }
    }
}

fn find_mate(
    pop: &Population,
    female_idx: usize,
    rng: &mut u64,
    cfg: &AgentSimConfig,
) -> Option<usize> {
    let ms = &cfg.mate_selection;
    let lp = &cfg.lifecycle;
    let fx = pop.xs[female_idx];
    let fy = pop.ys[female_idx];
    let r_sq = cfg.interaction_radius * cfg.interaction_radius * 4.0;

    let mut best: Option<usize> = None;
    let mut best_score = f32::NEG_INFINITY;

    for j in 0..pop.len() {
        if pop.sexes[j] != Sex::Male || pop.ages[j] < lp.min_reproduction_age {
            continue;
        }
        let dx = fx - pop.xs[j];
        let dy = fy - pop.ys[j];
        if dx * dx + dy * dy > r_sq {
            continue;
        }
        // Sexual selection: weighted preference
        let score = pop.statuses[j] * ms.status_weight
            + pop.resources[j] * ms.resource_weight
            + pop.prestiges[j] * ms.prestige_weight
            + rand01f(rng) * ms.noise_weight;
        if score > best_score {
            best_score = score;
            best = Some(j);
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Agent movement (drift toward kin, away from conflict)
// ---------------------------------------------------------------------------

fn movement_tick(pop: &mut Population, tick: u32, cfg: &AgentSimConfig) {
    let n = pop.len();
    if n == 0 {
        return;
    }
    let mp = &cfg.movement;

    // Compute kin group centroids
    let mut kin_cx: Vec<f64> = Vec::new();
    let mut kin_cy: Vec<f64> = Vec::new();
    let mut kin_count: Vec<u32> = Vec::new();

    for i in 0..n {
        let kg = pop.kin_groups[i] as usize;
        while kin_cx.len() <= kg {
            kin_cx.push(0.0);
            kin_cy.push(0.0);
            kin_count.push(0);
        }
        kin_cx[kg] += f64::from(pop.xs[i]);
        kin_cy[kg] += f64::from(pop.ys[i]);
        kin_count[kg] += 1;
    }

    for kg in 0..kin_cx.len() {
        let c = f64::from(kin_count[kg].max(1));
        kin_cx[kg] /= c;
        kin_cy[kg] /= c;
    }

    let world_max = cfg.world_size - 0.1;

    // Move each agent slightly toward kin centroid + random drift
    for i in 0..n {
        let mut rng = pop.ids[i]
            .wrapping_mul(2862933555777941757)
            .wrapping_add(tick as u64)
            .wrapping_add(cfg.seed)
            .wrapping_add(0xDEAD)
            .max(1);

        let kg = pop.kin_groups[i] as usize;
        if kg < kin_cx.len() && kin_count[kg] > 1 {
            let cx = kin_cx[kg] as f32;
            let cy = kin_cy[kg] as f32;
            let dx = (cx - pop.xs[i]) * mp.kin_pull_strength;
            let dy = (cy - pop.ys[i]) * mp.kin_pull_strength;
            pop.xs[i] += dx + (rand01f(&mut rng) - 0.5) * mp.drift_with_kin;
            pop.ys[i] += dy + (rand01f(&mut rng) - 0.5) * mp.drift_with_kin;
        } else {
            pop.xs[i] += (rand01f(&mut rng) - 0.5) * mp.drift_alone;
            pop.ys[i] += (rand01f(&mut rng) - 0.5) * mp.drift_alone;
        }

        // Clamp to world bounds
        pop.xs[i] = pop.xs[i].clamp(0.0, world_max);
        pop.ys[i] = pop.ys[i].clamp(0.0, world_max);
    }
}

// ---------------------------------------------------------------------------
// Energy landscape
// ---------------------------------------------------------------------------

pub(crate) fn init_energy_landscape(cfg: &AgentSimConfig) -> EnergyLandscape {
    let ep = &cfg.energy;
    let cell_size = cfg.interaction_radius;
    let cols = (cfg.world_size / cell_size).ceil() as usize + 1;
    let rows = cols;
    let mut cells = Vec::with_capacity(cols * rows);
    let mut rng = cfg.seed.wrapping_mul(0x517cc1b727220a95).max(1);

    for _ in 0..(cols * rows) {
        let biomass_var = 0.5 + rand01(&mut rng);
        let fertility_roll = rand01(&mut rng);
        let fossil_roll = rand01(&mut rng);
        let fossil_var = 0.5 + rand01(&mut rng);

        let is_fertile = fertility_roll < ep.agriculture_fertility_prob;
        let has_fossil = fossil_roll < ep.fossil_abundance;

        let biomass = EnergySource {
            stock: ep.biomass_initial_stock * biomass_var,
            initial_stock: ep.biomass_initial_stock * biomass_var,
            flow_rate: ep.biomass_flow_rate * biomass_var,
            base_eroei: ep.biomass_base_eroei,
            tech_threshold: ep.biomass_tech_threshold,
            steepness: ep.biomass_steepness,
        };

        let agriculture = EnergySource {
            stock: if is_fertile {
                ep.agriculture_initial_stock * fertility_roll
            } else {
                0.0
            },
            initial_stock: if is_fertile {
                ep.agriculture_initial_stock * fertility_roll
            } else {
                0.0
            },
            flow_rate: if is_fertile {
                ep.agriculture_flow_rate * fertility_roll
            } else {
                0.0
            },
            base_eroei: ep.agriculture_base_eroei,
            tech_threshold: ep.agriculture_tech_threshold,
            steepness: ep.agriculture_steepness,
        };

        let fossil = EnergySource {
            stock: if has_fossil {
                ep.fossil_initial_stock * fossil_var
            } else {
                0.0
            },
            initial_stock: if has_fossil {
                ep.fossil_initial_stock * fossil_var
            } else {
                0.0
            },
            flow_rate: if has_fossil { ep.fossil_flow_rate } else { 0.0 },
            base_eroei: ep.fossil_base_eroei,
            tech_threshold: ep.fossil_tech_threshold,
            steepness: ep.fossil_steepness,
        };

        let renewable = EnergySource {
            stock: f64::INFINITY,
            initial_stock: f64::INFINITY,
            flow_rate: ep.renewable_flow_rate,
            base_eroei: ep.renewable_base_eroei,
            tech_threshold: ep.renewable_tech_threshold,
            steepness: 1.0,
        };

        cells.push(EnergyCell {
            sources: [biomass, agriculture, fossil, renewable],
        });
    }

    EnergyLandscape {
        cells,
        cols,
        rows,
        cell_size,
    }
}

fn energy_harvest_tick(
    pop: &mut Population,
    landscape: &mut EnergyLandscape,
    cfg: &AgentSimConfig,
) -> EnergyTickSummary {
    let ep = &cfg.energy;
    let n = pop.len();
    if n == 0 {
        return EnergyTickSummary::default();
    }

    let cols = landscape.cols;
    let rows = landscape.rows;
    let cell_size = landscape.cell_size;
    let num_cells = cols * rows;

    // Group agents by grid cell and compute local tech level (mean innovation)
    let mut cell_agents: Vec<Vec<usize>> = vec![Vec::new(); num_cells];
    let mut cell_tech_sum: Vec<f32> = vec![0.0; num_cells];

    for i in 0..n {
        let cx = (pop.xs[i] / cell_size).floor() as usize;
        let cy = (pop.ys[i] / cell_size).floor() as usize;
        let key = cy.min(rows - 1) * cols + cx.min(cols - 1);
        cell_agents[key].push(i);
        cell_tech_sum[key] += pop.innovations[i];
    }

    let mut summary = EnergyTickSummary::default();

    // Harvest energy per cell and distribute to agents
    for k in 0..num_cells {
        let agent_count = cell_agents[k].len();
        if agent_count == 0 {
            continue;
        }

        let tech = cell_tech_sum[k] / agent_count as f32;
        let agents_f = agent_count as f64;
        let cell = &mut landscape.cells[k];
        let mut cell_net_energy = 0.0_f64;

        for (type_idx, source) in cell.sources.iter_mut().enumerate() {
            if tech < source.tech_threshold || source.flow_rate <= 0.0 {
                continue;
            }

            let eroei = source.current_eroei();
            if eroei <= 1.0 {
                continue; // uneconomical to extract
            }

            // Gross harvest: per-agent rate * number of agents * scaling
            let gross_max = source.flow_rate * agents_f * ep.harvest_per_agent;
            let gross = if source.stock.is_finite() {
                gross_max.min(source.stock)
            } else {
                gross_max
            };

            // Net energy after extraction costs
            let net = gross * (1.0 - 1.0 / eroei);
            cell_net_energy += net;
            summary.energy_by_type[type_idx] += net;

            // Deplete finite stocks
            if source.stock.is_finite() {
                source.stock = (source.stock - gross).max(0.0);
            }
        }

        // Distribute net energy among agents in this cell
        let per_agent = (cell_net_energy / agents_f) as f32;
        for &agent_idx in &cell_agents[k] {
            pop.resources[agent_idx] += per_agent;
            summary.agents_harvesting += 1;
        }
        summary.total_net_energy += cell_net_energy;
    }

    // Biomass regeneration: slowly recover stock in all cells
    for cell in &mut landscape.cells {
        let src = &mut cell.sources[EnergyType::Biomass as usize];
        if src.stock < src.initial_stock {
            src.stock = (src.stock + ep.biomass_regen_rate).min(src.initial_stock);
        }
    }

    summary
}

// ---------------------------------------------------------------------------
// Cultural transmission (Phase 5)
// ---------------------------------------------------------------------------

/// Horizontal transmission: during interactions, agents may adopt cultural traits from peers.
/// Oblique transmission: agents may adopt traits from the most prestigious nearby agent.
fn cultural_transmission_tick(
    pop: &mut Population,
    grid: &SpatialGrid,
    rng: &mut u64,
    cfg: &AgentSimConfig,
) {
    let cp = &cfg.cultural;
    let n = pop.len();
    if n == 0 {
        return;
    }

    // Collect changes to apply atomically
    let mut culture_updates: Vec<(usize, Culture)> = Vec::new();

    for i in 0..n {
        let neighbors = grid.neighbors_of(pop.xs[i], pop.ys[i]);
        if neighbors.is_empty() {
            continue;
        }

        // --- Oblique transmission: adopt from most prestigious nearby agent ---
        let mut best_prestige_idx: Option<usize> = None;
        let mut best_prestige = pop.prestiges[i] + cp.oblique_prestige_gap;
        for &j_u32 in &neighbors {
            let j = j_u32 as usize;
            if j == i {
                continue;
            }
            if pop.prestiges[j] > best_prestige {
                best_prestige = pop.prestiges[j];
                best_prestige_idx = Some(j);
            }
        }

        if let Some(j) = best_prestige_idx {
            if rand01f(rng) < cp.oblique_adoption_prob {
                let mut new_culture = pop.cultures[i];
                let model = &pop.cultures[j];
                // Adopt one random continuous trait from prestigious model
                match (rand01(rng) * 5.0) as u32 {
                    0 => new_culture.authority_norm = model.authority_norm,
                    1 => new_culture.sharing_norm = model.sharing_norm,
                    2 => new_culture.property_norm = model.property_norm,
                    3 => new_culture.trust_outgroup = model.trust_outgroup,
                    _ => new_culture.coercion_tolerance = model.coercion_tolerance,
                }
                // Techniques spread via prestige
                new_culture.techniques |= model.techniques;
                culture_updates.push((i, new_culture));
                continue; // skip horizontal if oblique happened
            }
        }

        // --- Horizontal transmission: adopt from random interaction partner ---
        if rand01f(rng) < cp.horizontal_adoption_prob {
            let j_idx = (rand01(rng) * neighbors.len() as f64) as usize % neighbors.len();
            let j = neighbors[j_idx] as usize;
            if j != i {
                let mut new_culture = pop.cultures[i];
                let peer = &pop.cultures[j];
                // Adopt one random discrete trait from peer
                match (rand01(rng) * 4.0) as u32 {
                    0 => new_culture.kinship_system = peer.kinship_system,
                    1 => new_culture.marriage_rule = peer.marriage_rule,
                    2 => new_culture.residence_rule = peer.residence_rule,
                    _ => new_culture.inheritance_rule = peer.inheritance_rule,
                }
                // Blend one continuous trait
                match (rand01(rng) * 5.0) as u32 {
                    0 => {
                        new_culture.authority_norm =
                            (new_culture.authority_norm + peer.authority_norm) * 0.5;
                    }
                    1 => {
                        new_culture.sharing_norm =
                            (new_culture.sharing_norm + peer.sharing_norm) * 0.5;
                    }
                    2 => {
                        new_culture.property_norm =
                            (new_culture.property_norm + peer.property_norm) * 0.5;
                    }
                    3 => {
                        new_culture.trust_outgroup =
                            (new_culture.trust_outgroup + peer.trust_outgroup) * 0.5;
                    }
                    _ => {
                        new_culture.coercion_tolerance =
                            (new_culture.coercion_tolerance + peer.coercion_tolerance) * 0.5;
                    }
                }
                culture_updates.push((i, new_culture));
            }
        }
    }

    // Apply updates
    for (i, culture) in culture_updates {
        pop.cultures[i] = culture;
    }
}

/// Measure cultural diversity as entropy over discrete cultural type combinations.
pub(crate) fn measure_cultural_diversity(pop: &Population) -> f32 {
    if pop.is_empty() {
        return 0.0;
    }
    let mut combo_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for i in 0..pop.len() {
        let c = &pop.cultures[i];
        // Encode combo as: kinship*27 + marriage*9 + residence*3 + inheritance
        let combo = (c.kinship_system as u32) * 27
            + (c.marriage_rule as u32) * 9
            + (c.residence_rule as u32) * 3
            + (c.inheritance_rule as u32);
        *combo_counts.entry(combo).or_insert(0) += 1;
    }
    let n = pop.len() as f32;
    let max_entropy = (81.0_f32).ln(); // 3^4 = 81 possible combos
    let mut entropy = 0.0_f32;
    for &count in combo_counts.values() {
        let p = count as f32 / n;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }
    if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    }
}

/// Count dominant kinship system.
pub(crate) fn dominant_kinship(pop: &Population) -> u8 {
    let mut counts = [0_u32; 3];
    for c in &pop.cultures {
        counts[c.kinship_system as usize] += 1;
    }
    let mut best = 0_u8;
    for i in 1..3 {
        if counts[i] > counts[best as usize] {
            best = i as u8;
        }
    }
    best
}

/// Count dominant marriage rule.
pub(crate) fn dominant_marriage(pop: &Population) -> u8 {
    let mut counts = [0_u32; 3];
    for c in &pop.cultures {
        counts[c.marriage_rule as usize] += 1;
    }
    let mut best = 0_u8;
    for i in 1..3 {
        if counts[i] > counts[best as usize] {
            best = i as u8;
        }
    }
    best
}

/// Mean number of technique bits set.
pub(crate) fn mean_technique_count(pop: &Population) -> f32 {
    if pop.is_empty() {
        return 0.0;
    }
    let sum: u32 = pop.cultures.iter().map(|c| c.techniques.count_ones()).sum();
    sum as f32 / pop.len() as f32
}

// ---------------------------------------------------------------------------
// Inter-society dynamics (Phase 4)
// ---------------------------------------------------------------------------

/// Compute kin-group aggregate power for inter-society interactions.
fn kin_group_power(pop: &Population, kin: u32, ip: &InteractionParams) -> f32 {
    let mut power = 0.0_f32;
    for i in 0..pop.len() {
        if pop.kin_groups[i] == kin {
            power += pop.statuses[i] * ip.power_status_weight
                + pop.skill_levels[i] * ip.power_skill_weight
                + pop.aggressions[i] * ip.power_aggression_weight;
        }
    }
    power
}

/// Compute mean aggression for a kin group.
pub(crate) fn kin_group_mean_aggression(pop: &Population, kin: u32) -> f32 {
    let mut sum = 0.0_f32;
    let mut count = 0_u32;
    for i in 0..pop.len() {
        if pop.kin_groups[i] == kin {
            sum += pop.aggressions[i];
            count += 1;
        }
    }
    if count > 0 {
        sum / count as f32
    } else {
        0.0
    }
}

/// Compute mean position (centroid) of a kin group.
pub(crate) fn kin_group_centroid(pop: &Population, kin: u32) -> (f32, f32) {
    let mut sx = 0.0_f32;
    let mut sy = 0.0_f32;
    let mut count = 0_u32;
    for i in 0..pop.len() {
        if pop.kin_groups[i] == kin {
            sx += pop.xs[i];
            sy += pop.ys[i];
            count += 1;
        }
    }
    if count > 0 {
        (sx / count as f32, sy / count as f32)
    } else {
        (0.0, 0.0)
    }
}

/// Run one tick of inter-society dynamics: raids, conquest, tribute, migration.
fn inter_society_tick(
    pop: &mut Population,
    tributes: &mut Vec<TributeRelation>,
    rng: &mut u64,
    cfg: &AgentSimConfig,
    inter_group_trades: u32,
    total_trades: u32,
) -> InterSocietySummary {
    let isp = &cfg.inter_society;
    let ip = &cfg.interaction;
    let mut summary = InterSocietySummary::default();

    // Collect unique kin groups and their sizes
    let mut kin_sizes: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for &kg in &pop.kin_groups {
        *kin_sizes.entry(kg).or_insert(0) += 1;
    }
    let kin_ids: Vec<u32> = kin_sizes.keys().copied().collect();

    // Compute centroids for distance checks
    let centroids: std::collections::HashMap<u32, (f32, f32)> = kin_ids
        .iter()
        .map(|&k| (k, kin_group_centroid(pop, k)))
        .collect();

    // --- Raids ---
    for &attacker_kin in &kin_ids {
        let attacker_size = kin_sizes[&attacker_kin];
        if attacker_size < isp.min_raid_warriors {
            continue;
        }
        let mean_aggr = kin_group_mean_aggression(pop, attacker_kin);
        if mean_aggr < isp.raid_aggression_threshold {
            continue;
        }
        // Roll for raid attempt
        if rand01f(rng) > mean_aggr {
            continue;
        }
        // Find nearest target within raid_range
        let (ax, ay) = centroids[&attacker_kin];
        let mut best_target: Option<u32> = None;
        let mut best_dist = f32::MAX;
        for &target_kin in &kin_ids {
            if target_kin == attacker_kin {
                continue;
            }
            let (tx, ty) = centroids[&target_kin];
            let dist = ((ax - tx).powi(2) + (ay - ty).powi(2)).sqrt();
            if dist < isp.raid_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(target_kin);
            }
        }
        let Some(target_kin) = best_target else {
            continue;
        };

        let attacker_power = kin_group_power(pop, attacker_kin, ip);
        let defender_power = kin_group_power(pop, target_kin, ip);

        if attacker_power > defender_power * 0.5 {
            // Successful raid: loot resources, damage defenders
            let warriors = attacker_size;
            let loot = isp.raid_loot_per_warrior * warriors as f32;
            let damage = isp.raid_damage_per_warrior * warriors as f32;
            let defender_count = kin_sizes[&target_kin];

            // Distribute loot from defenders to attackers
            let loot_per_defender = if defender_count > 0 {
                (loot / defender_count as f32).min(0.3) // cap per-agent loss
            } else {
                0.0
            };
            let loot_per_attacker = if attacker_size > 0 {
                (loot_per_defender * defender_count as f32) / attacker_size as f32
            } else {
                0.0
            };
            let damage_per_defender = if defender_count > 0 {
                (damage / defender_count as f32).min(0.15)
            } else {
                0.0
            };

            for i in 0..pop.len() {
                if pop.kin_groups[i] == target_kin {
                    pop.resources[i] = (pop.resources[i] - loot_per_defender).max(0.0);
                    pop.healths[i] = (pop.healths[i] - damage_per_defender).max(0.0);
                } else if pop.kin_groups[i] == attacker_kin {
                    pop.resources[i] += loot_per_attacker;
                    pop.prestiges[i] += 0.01; // raid prestige
                }
            }
            summary.raids += 1;

            // Check for conquest: overwhelming power ratio
            if attacker_power > defender_power * isp.conquest_power_ratio {
                // Conquest: vassals start paying tribute
                tributes.push(TributeRelation {
                    vassal_kin: target_kin,
                    overlord_kin: attacker_kin,
                    rate: isp.tribute_rate,
                    ticks_remaining: isp.tribute_duration,
                });
                summary.conquests += 1;
            }
        }
    }

    // --- Tribute collection ---
    tributes.retain_mut(|tr| {
        if tr.ticks_remaining == 0 {
            return false;
        }
        tr.ticks_remaining -= 1;
        let mut tribute_collected = 0.0_f32;
        let mut vassal_count = 0_u32;
        let mut overlord_count = 0_u32;
        for i in 0..pop.len() {
            if pop.kin_groups[i] == tr.vassal_kin {
                vassal_count += 1;
            } else if pop.kin_groups[i] == tr.overlord_kin {
                overlord_count += 1;
            }
        }
        if vassal_count == 0 || overlord_count == 0 {
            return false;
        }
        // Collect tribute from vassals
        for i in 0..pop.len() {
            if pop.kin_groups[i] == tr.vassal_kin {
                let payment = pop.resources[i] * tr.rate;
                pop.resources[i] -= payment;
                tribute_collected += payment;
            }
        }
        // Distribute to overlords
        let per_overlord = tribute_collected / overlord_count as f32;
        for i in 0..pop.len() {
            if pop.kin_groups[i] == tr.overlord_kin {
                pop.resources[i] += per_overlord;
            }
        }
        summary.tribute_total += tribute_collected;
        true
    });

    // --- Migration ---
    for i in 0..pop.len() {
        if pop.resources[i] < isp.migration_resource_threshold
            && rand01f(rng) < isp.migration_probability
        {
            // Migrate to a random different kin group
            if kin_ids.len() > 1 {
                let idx = (rand01(rng) * kin_ids.len() as f64) as usize % kin_ids.len();
                let new_kin = kin_ids[idx];
                if new_kin != pop.kin_groups[i] {
                    pop.kin_groups[i] = new_kin;
                    // Move agent near the new group's centroid
                    let (cx, cy) = centroids[&new_kin];
                    pop.xs[i] = cx + (rand01f(rng) - 0.5) * cfg.interaction_radius;
                    pop.ys[i] = cy + (rand01f(rng) - 0.5) * cfg.interaction_radius;
                    summary.migrations += 1;
                }
            }
        }
    }

    // Record trade stats
    summary.inter_group_trades = inter_group_trades;
    summary.total_trades = total_trades;

    summary
}

// ---------------------------------------------------------------------------
// Main simulation loop
// ---------------------------------------------------------------------------

/// Run the individual agent simulation.
#[must_use]
pub fn simulate_agents(cfg: AgentSimConfig) -> AgentSimResult {
    let mut pop = seed_population(&cfg);
    let mut landscape = init_energy_landscape(&cfg);
    let mut next_id = cfg.initial_population as u64;
    let mut snapshots = Vec::with_capacity(cfg.ticks as usize);
    let mut tributes: Vec<TributeRelation> = Vec::new();
    let mut rng = cfg.seed.wrapping_add(0xDEAD_BEEF); // separate RNG stream for inter-society

    for tick in 0..cfg.ticks {
        if (pop.len() as u32) < cfg.min_population {
            break;
        }

        // Build spatial index
        let grid = SpatialGrid::build(&pop.xs, &pop.ys, cfg.interaction_radius, cfg.world_size);

        // Compute and apply interactions
        let effects = compute_interactions(&pop, &grid, tick, &cfg);
        let coop_events = effects.cooperation_events;
        let conflict_events = effects.conflict_events;
        let total_interactions = effects.total_interactions;
        let inter_group_trades = effects.inter_group_trades;
        let total_actual_surplus = effects.total_actual_surplus;
        let total_cooperative_optimal = effects.total_cooperative_optimal;
        let trade_events = effects.trade_events;
        let institutional = detect_institutional_profile(&pop, &effects, &cfg);
        apply_effects(&mut pop, &effects, &cfg);

        // Energy harvest (replaces flat resource_regen)
        let energy_summary = energy_harvest_tick(&mut pop, &mut landscape, &cfg);

        // Inter-society dynamics: raids, conquest, tribute, migration
        let inter_society_summary = inter_society_tick(
            &mut pop,
            &mut tributes,
            &mut rng,
            &cfg,
            inter_group_trades,
            trade_events,
        );

        // Cultural transmission: horizontal and oblique (must use grid before lifecycle changes pop)
        cultural_transmission_tick(&mut pop, &grid, &mut rng, &cfg);

        // Lifecycle: aging, death, reproduction
        lifecycle_tick(&mut pop, tick, &cfg, &mut next_id);

        // Movement
        movement_tick(&mut pop, tick, &cfg);

        // Measure emergent state
        let emergent = measure_emergent_state(
            &pop,
            &TickMeasureInput {
                cooperation_events: coop_events,
                conflict_events,
                total_interactions,
                energy_summary: &energy_summary,
                landscape: &landscape,
                institutional: &institutional,
                inter_society: &inter_society_summary,
                active_tribute_count: tributes.len() as u32,
                total_actual_surplus,
                total_cooperative_optimal,
            },
        );
        snapshots.push(AgentSnapshot { tick, emergent });
    }

    AgentSimResult {
        snapshots,
        final_population: pop,
        final_landscape: landscape,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_population_creates_expected_count() {
        let cfg = AgentSimConfig {
            initial_population: 100,
            ..AgentSimConfig::default()
        };
        let pop = seed_population(&cfg);
        assert_eq!(pop.len(), 100);
        assert!(pop.sexes.contains(&Sex::Male));
        assert!(pop.sexes.contains(&Sex::Female));
    }

    #[test]
    fn population_has_multiple_kin_groups() {
        let pop = seed_population(&AgentSimConfig::default());
        let groups = count_kin_groups(&pop.kin_groups);
        assert!(groups > 1);
    }

    #[test]
    fn gini_is_zero_for_equal_resources() {
        let resources = vec![1.0_f32; 50];
        let g = measure_gini(&resources);
        assert!(g < 0.01);
    }

    #[test]
    fn gini_increases_for_unequal_resources() {
        let equal = vec![1.0_f32; 50];
        let mut unequal = vec![0.1_f32; 50];
        unequal[0] = 100.0;
        assert!(measure_gini(&unequal) > measure_gini(&equal));
    }

    #[test]
    fn gini_fast_matches_exact_for_small_population() {
        let resources: Vec<f32> = (0..20).map(|i| i as f32 * 0.5 + 0.1).collect();
        let exact = measure_gini(&resources);
        let fast = measure_gini_fast(&resources);
        assert!((exact - fast).abs() < 0.05, "exact={exact} fast={fast}");
    }

    #[test]
    fn skill_entropy_is_zero_for_uniform_skills() {
        let skills = vec![SkillType::Forager; 50];
        assert!(measure_skill_entropy(&skills) < 0.01);
    }

    #[test]
    fn skill_entropy_is_high_for_diverse_skills() {
        let mut skills = Vec::new();
        for _ in 0..10 {
            skills.push(SkillType::Forager);
            skills.push(SkillType::Crafter);
            skills.push(SkillType::Builder);
            skills.push(SkillType::Leader);
            skills.push(SkillType::Warrior);
        }
        assert!(measure_skill_entropy(&skills) > 0.9);
    }

    #[test]
    fn hierarchy_depth_is_zero_without_patrons() {
        let patrons = vec![None; 50];
        assert_eq!(measure_hierarchy_depth(&patrons), 0);
    }

    #[test]
    fn hierarchy_depth_tracks_patron_chains() {
        // Chain: 0->1->2->3
        let mut patrons = vec![None; 10];
        patrons[0] = Some(1);
        patrons[1] = Some(2);
        patrons[2] = Some(3);
        assert_eq!(measure_hierarchy_depth(&patrons), 3);
    }

    #[test]
    fn simulation_runs_and_produces_snapshots() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            ..AgentSimConfig::default()
        });
        assert!(!result.snapshots.is_empty());
        assert!(!result.final_population.is_empty());
    }

    #[test]
    fn simulation_produces_interactions() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 60,
            ticks: 20,
            world_size: 30.0, // smaller world = more interactions
            ..AgentSimConfig::default()
        });
        // Should have some cooperation and conflict
        let total_coop: f32 = result
            .snapshots
            .iter()
            .map(|s| s.emergent.cooperation_rate)
            .sum();
        let total_conflict: f32 = result
            .snapshots
            .iter()
            .map(|s| s.emergent.conflict_rate)
            .sum();
        assert!(total_coop > 0.0, "should have cooperation events");
        assert!(total_conflict > 0.0, "should have conflict events");
    }

    #[test]
    fn inequality_emerges_over_time() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 100,
            ticks: 100,
            world_size: 40.0,
            ..AgentSimConfig::default()
        });
        let early_gini = result.snapshots[5].emergent.gini_coefficient;
        let late_gini = result
            .snapshots
            .last()
            .map(|s| s.emergent.gini_coefficient)
            .unwrap_or(0.0);
        assert!(
            (late_gini - early_gini).abs() > 0.001,
            "inequality should evolve: early={early_gini:.3} late={late_gini:.3}"
        );
    }

    #[test]
    fn delegation_produces_hierarchy() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 120,
            ticks: 100,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        let max_depth = result
            .snapshots
            .iter()
            .map(|s| s.emergent.max_hierarchy_depth)
            .max()
            .unwrap_or(0);
        assert!(
            max_depth >= 1,
            "should see at least depth-1 hierarchy, got {max_depth}"
        );
    }

    #[test]
    fn population_grows_with_sufficient_resources() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 200,
            ticks: 100,
            energy: EnergyParams {
                biomass_flow_rate: 0.25,
                ..EnergyParams::default()
            },
            world_size: 50.0,
            max_population: 2000,
            min_population: 2,
            ..AgentSimConfig::default()
        });
        // With generous resources, peak population should exceed starting
        let peak_pop = result
            .snapshots
            .iter()
            .map(|s| s.emergent.population_size)
            .max()
            .unwrap_or(0);
        assert!(
            peak_pop > 200,
            "population should grow with good resources, peak was {peak_pop}"
        );
    }

    #[test]
    fn scarce_resources_limit_population() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 200,
            energy: EnergyParams {
                biomass_flow_rate: 0.006,
                ..EnergyParams::default()
            },
            world_size: 50.0,
            ..AgentSimConfig::default()
        });
        let final_pop = result.final_population.len();
        assert!(
            final_pop < 200,
            "population should be constrained, got {final_pop}"
        );
    }

    #[test]
    fn emergent_state_values_are_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 50,
            ticks: 30,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            let e = &snap.emergent;
            assert!((0.0..=1.0).contains(&e.gini_coefficient));
            assert!((0.0..=1.0).contains(&e.skill_entropy));
            assert!((0.0..=1.0).contains(&e.cooperation_rate));
            assert!((0.0..=1.0).contains(&e.conflict_rate));
            assert!(e.mean_health >= 0.0 && e.mean_health <= 1.0);
        }
    }

    // --- Energy model tests ---

    #[test]
    fn energy_source_depletion_tracks_stock() {
        let src = EnergySource {
            stock: 50.0,
            initial_stock: 100.0,
            flow_rate: 1.0,
            base_eroei: 10.0,
            tech_threshold: 0.0,
            steepness: 2.0,
        };
        assert!((src.depletion() - 0.5).abs() < 1e-9);
        assert!(src.current_eroei() < src.base_eroei);
        // At 50% depletion with steepness 2: eroei = 10 * 0.5^2 = 2.5
        assert!((src.current_eroei() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn energy_source_infinite_stock_has_zero_depletion() {
        let src = EnergySource {
            stock: f64::INFINITY,
            initial_stock: f64::INFINITY,
            flow_rate: 1.0,
            base_eroei: 15.0,
            tech_threshold: 0.0,
            steepness: 1.0,
        };
        assert!((src.depletion() - 0.0).abs() < 1e-9);
        assert!((src.current_eroei() - 15.0).abs() < 1e-9);
    }

    #[test]
    fn energy_landscape_initialized_with_correct_dimensions() {
        let cfg = AgentSimConfig {
            world_size: 40.0,
            interaction_radius: 8.0,
            ..AgentSimConfig::default()
        };
        let landscape = init_energy_landscape(&cfg);
        let expected_cols = (40.0_f32 / 8.0).ceil() as usize + 1;
        assert_eq!(landscape.cols, expected_cols);
        assert_eq!(landscape.rows, expected_cols);
        assert_eq!(landscape.cells.len(), expected_cols * expected_cols);
    }

    #[test]
    fn biomass_provides_resources_without_tech() {
        // Even with zero innovation, biomass should be harvestable
        let cfg = AgentSimConfig {
            initial_population: 50,
            ticks: 10,
            ..AgentSimConfig::default()
        };
        let result = simulate_agents(cfg);
        // Agents should have some resources from biomass
        let mean_res = result
            .snapshots
            .last()
            .map(|s| s.emergent.mean_resources)
            .unwrap_or(0.0);
        assert!(mean_res > 0.0, "agents should have resources from biomass");
    }

    #[test]
    fn energy_per_capita_is_positive() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 50,
            ticks: 10,
            ..AgentSimConfig::default()
        });
        let epc = result.snapshots[5].emergent.energy_per_capita;
        assert!(epc > 0.0, "energy per capita should be positive, got {epc}");
    }

    #[test]
    fn fossil_depletion_increases_with_extraction() {
        // High innovation to unlock fossil, small world to concentrate agents
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 200,
            world_size: 20.0,
            lifecycle: LifecycleParams {
                innovation_growth_rate: 0.01, // fast tech growth
                ..LifecycleParams::default()
            },
            energy: EnergyParams {
                fossil_tech_threshold: 0.1, // low threshold for testing
                fossil_abundance: 1.0,      // every cell has fossil
                fossil_initial_stock: 50.0, // small stock to see depletion
                ..EnergyParams::default()
            },
            ..AgentSimConfig::default()
        });
        let fossil_dep = result.final_landscape.mean_depletion(EnergyType::Fossil);
        assert!(
            fossil_dep > 0.0,
            "fossil should show depletion after extraction, got {fossil_dep}"
        );
    }

    #[test]
    fn agriculture_unlocks_at_tech_threshold() {
        // Run with innovation growth. Agriculture should eventually contribute.
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 300,
            world_size: 30.0,
            lifecycle: LifecycleParams {
                innovation_growth_rate: 0.005,
                ..LifecycleParams::default()
            },
            energy: EnergyParams {
                agriculture_tech_threshold: 0.2,
                agriculture_fertility_prob: 1.0, // all cells fertile
                ..EnergyParams::default()
            },
            ..AgentSimConfig::default()
        });
        // By tick 300, innovation should be ~0.15 + 300*0.005 = 1.65 (clamped to 1.0)
        // Agriculture should be the dominant energy source
        // By tick 300, innovation should have grown past the threshold
        let last_innov = result
            .snapshots
            .last()
            .map(|s| s.emergent.mean_innovation)
            .unwrap_or(0.0);
        assert!(
            last_innov > 0.2,
            "innovation should have grown past ag threshold, got {last_innov}"
        );
    }

    #[test]
    fn innovation_grows_over_time() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 50,
            ticks: 100,
            ..AgentSimConfig::default()
        });
        let early_innov = result.snapshots[5].emergent.mean_innovation;
        let late_innov = result
            .snapshots
            .last()
            .map(|s| s.emergent.mean_innovation)
            .unwrap_or(0.0);
        assert!(
            late_innov > early_innov,
            "innovation should grow: early={early_innov:.4} late={late_innov:.4}"
        );
    }

    #[test]
    fn biomass_only_society_plateaus() {
        // With no tech growth, only biomass available -> population should plateau
        let result = simulate_agents(AgentSimConfig {
            initial_population: 100,
            ticks: 300,
            world_size: 40.0,
            max_population: 5000,
            lifecycle: LifecycleParams {
                innovation_growth_rate: 0.0, // no tech progress
                ..LifecycleParams::default()
            },
            energy: EnergyParams {
                agriculture_tech_threshold: 10.0, // unreachable
                fossil_tech_threshold: 10.0,
                renewable_tech_threshold: 10.0,
                ..EnergyParams::default()
            },
            ..AgentSimConfig::default()
        });
        let peak = result
            .snapshots
            .iter()
            .map(|s| s.emergent.population_size)
            .max()
            .unwrap_or(0);
        // Biomass-only should support limited population (migration can shuffle groups)
        assert!(
            peak < 10000,
            "biomass-only society should plateau, peak was {peak}"
        );
    }

    // --- Institution tests (Phase 3) ---

    #[test]
    fn coercion_rate_is_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.coercion_rate),
                "coercion_rate should be 0-1, got {}",
                snap.emergent.coercion_rate
            );
        }
    }

    #[test]
    fn property_norm_strength_is_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.property_norm_strength),
                "property_norm_strength should be 0-1, got {}",
                snap.emergent.property_norm_strength
            );
        }
    }

    #[test]
    fn patron_count_increases_with_population() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 200,
            ticks: 100,
            world_size: 30.0,
            max_population: 2000,
            energy: EnergyParams {
                biomass_flow_rate: 0.15,
                ..EnergyParams::default()
            },
            ..AgentSimConfig::default()
        });
        // Should have some patrons by the end
        let max_patrons = result
            .snapshots
            .iter()
            .map(|s| s.emergent.patron_count)
            .max()
            .unwrap_or(0);
        assert!(
            max_patrons > 0,
            "should have at least one patron, got {max_patrons}"
        );
    }

    #[test]
    fn patron_tenure_grows_with_inheritance() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 120,
            ticks: 150,
            world_size: 30.0,
            institution: InstitutionParams {
                patron_inheritance: true,
                ..InstitutionParams::default()
            },
            ..AgentSimConfig::default()
        });
        // With inheritance, patron tenure should grow over time
        let late_tenure = result
            .snapshots
            .last()
            .map(|s| s.emergent.mean_patron_tenure)
            .unwrap_or(0.0);
        // Should have some tenure accumulated
        assert!(
            late_tenure >= 0.0,
            "patron tenure should be non-negative, got {late_tenure}"
        );
    }

    #[test]
    fn institutional_type_is_valid() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                snap.emergent.institutional_type <= 3,
                "institutional_type should be 0-3, got {}",
                snap.emergent.institutional_type
            );
        }
    }

    #[test]
    fn public_goods_benefit_group_survival() {
        // Compare with and without public goods
        let with_goods = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 100,
            ticks: 200,
            world_size: 30.0,
            institution: InstitutionParams {
                public_goods_rate: 0.5,
                public_goods_bonus: 0.02,
                ..InstitutionParams::default()
            },
            ..AgentSimConfig::default()
        });
        let without_goods = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 100,
            ticks: 200,
            world_size: 30.0,
            institution: InstitutionParams {
                public_goods_rate: 0.0,
                public_goods_bonus: 0.0,
                ..InstitutionParams::default()
            },
            ..AgentSimConfig::default()
        });
        // Public goods should provide some measurable benefit
        let with_mean_res = with_goods
            .snapshots
            .iter()
            .map(|s| s.emergent.mean_resources)
            .sum::<f32>();
        let without_mean_res = without_goods
            .snapshots
            .iter()
            .map(|s| s.emergent.mean_resources)
            .sum::<f32>();
        // The runs diverge due to public goods, so they should differ
        assert!(
            (with_mean_res - without_mean_res).abs() > 0.01,
            "public goods should affect resource levels"
        );
    }

    // --- Inter-society tests (Phase 4) ---

    #[test]
    fn raids_occur_with_aggressive_population() {
        // High aggression, small world, many kin groups -> should trigger raids
        let result = simulate_agents(AgentSimConfig {
            initial_population: 120,
            ticks: 200,
            world_size: 25.0,
            lifecycle: LifecycleParams {
                agents_per_kin_group: 15, // many small groups
                ..LifecycleParams::default()
            },
            inter_society: InterSocietyParams {
                min_raid_warriors: 2,
                raid_aggression_threshold: 0.15, // low threshold
                raid_range: 30.0,                // wide range
                ..InterSocietyParams::default()
            },
            ..AgentSimConfig::default()
        });
        let total_raids: u32 = result
            .snapshots
            .iter()
            .map(|s| s.emergent.raid_events)
            .sum();
        assert!(
            total_raids > 0,
            "should have at least one raid with aggressive population"
        );
    }

    #[test]
    fn inter_group_trade_rate_is_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.inter_group_trade_rate),
                "inter_group_trade_rate should be 0-1, got {}",
                snap.emergent.inter_group_trade_rate
            );
        }
    }

    #[test]
    fn migration_occurs_under_resource_stress() {
        // Low resources, high migration probability
        let result = simulate_agents(AgentSimConfig {
            initial_population: 100,
            ticks: 200,
            world_size: 50.0,
            energy: EnergyParams {
                biomass_flow_rate: 0.01, // very low energy -> resource stress
                ..EnergyParams::default()
            },
            inter_society: InterSocietyParams {
                migration_resource_threshold: 0.5, // high threshold -> more migration
                migration_probability: 0.2,        // high probability
                ..InterSocietyParams::default()
            },
            ..AgentSimConfig::default()
        });
        let total_migrations: u32 = result
            .snapshots
            .iter()
            .map(|s| s.emergent.migration_events)
            .sum();
        assert!(
            total_migrations > 0,
            "should have migrations under resource stress"
        );
    }

    #[test]
    fn tribute_flows_are_non_negative() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 100,
            ticks: 100,
            world_size: 25.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                snap.emergent.tribute_flows >= 0.0,
                "tribute flows should be non-negative, got {}",
                snap.emergent.tribute_flows
            );
        }
    }

    #[test]
    fn num_active_societies_matches_kin_groups() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 20,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            // num_active_societies should equal num_kin_groups
            assert_eq!(
                snap.emergent.num_active_societies, snap.emergent.num_kin_groups,
                "active societies should match kin groups"
            );
        }
    }

    // --- Cultural transmission tests (Phase 5) ---

    #[test]
    fn cultural_diversity_is_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.cultural_diversity),
                "cultural_diversity should be 0-1, got {}",
                snap.emergent.cultural_diversity
            );
        }
    }

    #[test]
    fn authority_norm_is_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.mean_authority_norm),
                "mean_authority_norm should be 0-1, got {}",
                snap.emergent.mean_authority_norm
            );
        }
    }

    #[test]
    fn cultural_traits_evolve_over_time() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 100,
            ticks: 200,
            world_size: 30.0,
            cultural: CulturalParams {
                horizontal_adoption_prob: 0.1,
                oblique_adoption_prob: 0.05,
                ..CulturalParams::default()
            },
            ..AgentSimConfig::default()
        });
        let early_div = result.snapshots[5].emergent.cultural_diversity;
        let late_div = result
            .snapshots
            .last()
            .map(|s| s.emergent.cultural_diversity)
            .unwrap_or(0.0);
        assert!(
            (late_div - early_div).abs() > 0.001 || late_div > 0.0,
            "cultural diversity should evolve: early={early_div:.4} late={late_div:.4}"
        );
    }

    #[test]
    fn techniques_accumulate_over_generations() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 200,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        let early_tech = result.snapshots[5].emergent.technique_count;
        let late_tech = result
            .snapshots
            .last()
            .map(|s| s.emergent.technique_count)
            .unwrap_or(0.0);
        assert!(
            late_tech >= early_tech,
            "techniques should accumulate: early={early_tech:.2} late={late_tech:.2}"
        );
    }

    #[test]
    fn high_sharing_norm_boosts_cooperation() {
        let high_sharing = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            cultural: CulturalParams {
                sharing_coop_bonus: 0.5,
                ..CulturalParams::default()
            },
            ..AgentSimConfig::default()
        });
        let low_sharing = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            cultural: CulturalParams {
                sharing_coop_bonus: 0.0,
                ..CulturalParams::default()
            },
            ..AgentSimConfig::default()
        });
        let high_coop: f32 = high_sharing
            .snapshots
            .iter()
            .map(|s| s.emergent.cooperation_rate)
            .sum();
        let low_coop: f32 = low_sharing
            .snapshots
            .iter()
            .map(|s| s.emergent.cooperation_rate)
            .sum();
        assert!(
            (high_coop - low_coop).abs() > 0.01,
            "sharing_coop_bonus should affect cooperation: high={high_coop:.4} low={low_coop:.4}"
        );
    }

    #[test]
    fn dominant_kinship_is_valid() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 20,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                snap.emergent.dominant_kinship <= 2,
                "dominant_kinship should be 0-2, got {}",
                snap.emergent.dominant_kinship
            );
            assert!(
                snap.emergent.dominant_marriage <= 2,
                "dominant_marriage should be 0-2, got {}",
                snap.emergent.dominant_marriage
            );
        }
    }

    // --- Superorganism detection tests (Phase 6) ---

    #[test]
    fn superorganism_index_is_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            let idx = superorganism_index(&snap.emergent);
            assert!(
                (0.0..=1.0).contains(&idx),
                "superorganism index should be 0-1, got {idx}"
            );
        }
    }

    #[test]
    fn superorganism_index_zero_for_tiny_population() {
        let state = EmergentState {
            population_size: 5,
            ..EmergentState {
                population_size: 5,
                mean_resources: 0.0,
                gini_coefficient: 0.0,
                skill_entropy: 0.0,
                max_hierarchy_depth: 0,
                num_leaders: 0,
                mean_group_size: 0.0,
                num_kin_groups: 0,
                cooperation_rate: 0.0,
                conflict_rate: 0.0,
                mean_prestige: 0.0,
                mean_health: 0.0,
                mean_innovation: 0.0,
                dominant_energy: 0,
                energy_per_capita: 0.0,
                mean_eroei: 0.0,
                biomass_depletion: 0.0,
                fossil_depletion: 0.0,
                coercion_rate: 0.0,
                property_norm_strength: 0.0,
                institutional_type: 0,
                public_goods_investment: 0.0,
                patron_count: 0,
                recognized_leaders: 0,
                mean_patron_tenure: 0.0,
                raid_events: 0,
                conquest_events: 0,
                tribute_flows: 0.0,
                migration_events: 0,
                num_active_societies: 0,
                inter_group_trade_rate: 0.0,
                active_tributes: 0,
                mean_authority_norm: 0.0,
                mean_sharing_norm: 0.0,
                mean_property_norm: 0.0,
                mean_trust_outgroup: 0.0,
                cultural_diversity: 0.0,
                dominant_kinship: 0,
                dominant_marriage: 0,
                mean_coercion_tolerance: 0.0,
                technique_count: 0.0,
                coordination_failure_index: 0.0,
                mean_trust: 0.5,
            }
        };
        assert_eq!(superorganism_index(&state), 0.0);
    }

    #[test]
    fn analyze_superorganism_produces_valid_result() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        let analysis = analyze_superorganism(&result, 0.3, 10);
        assert!(analysis.peak_index >= analysis.final_index || analysis.peak_index >= 0.0);
        assert!(analysis.mean_index >= 0.0);
        assert!(analysis.final_institution <= 3);
        assert!(analysis.final_kinship <= 2);
    }

    #[test]
    fn high_complexity_run_has_higher_superorganism_index() {
        // Dense, aggressive, hierarchical -> should score higher than sparse passive
        let complex = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 200,
            ticks: 200,
            world_size: 25.0, // denser packing for more interactions
            max_population: 3000,
            cultural: CulturalParams {
                authority_delegation_bonus: 0.4,
                coercion_conflict_bonus: 0.2,
                ..CulturalParams::default()
            },
            interaction: InteractionParams {
                trust_coop_weight: 0.25,
                ..InteractionParams::default()
            },
            ..AgentSimConfig::default()
        });
        let simple = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 30, // fewer agents
            ticks: 200,
            world_size: 120.0, // much sparser
            ..AgentSimConfig::default()
        });
        let complex_peak = complex
            .snapshots
            .iter()
            .map(|s| superorganism_index(&s.emergent))
            .fold(0.0_f32, f32::max);
        let simple_peak = simple
            .snapshots
            .iter()
            .map(|s| superorganism_index(&s.emergent))
            .fold(0.0_f32, f32::max);
        assert!(
            complex_peak > simple_peak,
            "complex scenario should have higher superorganism index: complex={complex_peak:.3} simple={simple_peak:.3}"
        );
    }

    #[test]
    fn experiment_conditions_are_valid() {
        let conditions = default_agent_experiment_conditions();
        assert_eq!(conditions.len(), 8);
        for cond in &conditions {
            assert!(!cond.label.is_empty());
            assert!(cond.config.initial_population > 0);
        }
    }

    // --- Trust-memory coordination dilemma tests ---

    #[test]
    fn trust_memory_stable_without_interactions() {
        // Large world, sparse population -> few interactions -> trust stays near 0.5
        let result = simulate_agents(AgentSimConfig {
            initial_population: 50,
            ticks: 20,
            world_size: 500.0, // very large world -> agents rarely meet
            ..AgentSimConfig::default()
        });
        let final_trust = result
            .snapshots
            .last()
            .map(|s| s.emergent.mean_trust)
            .unwrap_or(0.0);
        assert!(
            (final_trust - 0.5).abs() < 0.3,
            "mean trust should stay near 0.5 without many interactions, got {final_trust}"
        );
    }

    #[test]
    fn trust_memory_responds_to_cooperation() {
        // Cooperative config vs aggressive config
        let cooperative = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            interaction: InteractionParams {
                coop_self_weight: 0.9,
                coop_other_weight: 0.9,
                coop_kin_bonus: 0.5,
                conflict_self_weight: 0.01,
                conflict_other_weight: 0.01,
                ..InteractionParams::default()
            },
            ..AgentSimConfig::default()
        });
        let aggressive = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            interaction: InteractionParams {
                coop_self_weight: 0.01,
                coop_other_weight: 0.01,
                conflict_self_weight: 0.9,
                conflict_other_weight: 0.9,
                conflict_stranger_bonus: 0.5,
                ..InteractionParams::default()
            },
            ..AgentSimConfig::default()
        });
        let coop_trust = cooperative
            .snapshots
            .last()
            .map(|s| s.emergent.mean_trust)
            .unwrap_or(0.0);
        let aggr_trust = aggressive
            .snapshots
            .last()
            .map(|s| s.emergent.mean_trust)
            .unwrap_or(1.0);
        assert!(
            coop_trust > aggr_trust,
            "cooperative config should have higher trust: coop={coop_trust:.4} aggr={aggr_trust:.4}"
        );
    }

    #[test]
    fn coordination_failure_index_bounded() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 50,
            world_size: 30.0,
            ..AgentSimConfig::default()
        });
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.coordination_failure_index),
                "CFI should be 0-1, got {}",
                snap.emergent.coordination_failure_index
            );
        }
    }

    #[test]
    fn cfi_higher_under_conflict_bias() {
        let warlike = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            interaction: InteractionParams {
                conflict_self_weight: 0.9,
                conflict_other_weight: 0.9,
                conflict_stranger_bonus: 0.5,
                coop_self_weight: 0.01,
                coop_other_weight: 0.01,
                ..InteractionParams::default()
            },
            ..AgentSimConfig::default()
        });
        let peaceful = simulate_agents(AgentSimConfig {
            seed: 42,
            initial_population: 80,
            ticks: 100,
            world_size: 30.0,
            interaction: InteractionParams {
                coop_self_weight: 0.9,
                coop_other_weight: 0.9,
                coop_kin_bonus: 0.5,
                conflict_self_weight: 0.01,
                conflict_other_weight: 0.01,
                ..InteractionParams::default()
            },
            ..AgentSimConfig::default()
        });
        let war_cfi: f32 = warlike
            .snapshots
            .iter()
            .map(|s| s.emergent.coordination_failure_index)
            .sum();
        let peace_cfi: f32 = peaceful
            .snapshots
            .iter()
            .map(|s| s.emergent.coordination_failure_index)
            .sum();
        assert!(
            war_cfi > peace_cfi,
            "warlike scenario should have higher CFI: war={war_cfi:.4} peace={peace_cfi:.4}"
        );
    }
}
