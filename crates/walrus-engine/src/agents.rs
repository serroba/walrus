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
    pub norms: Vec<u64>,
    pub innovations: Vec<f32>,

    // Relationships
    pub kin_groups: Vec<u32>,
    pub partners: Vec<Option<u32>>, // index into population, not id
    pub patrons: Vec<Option<u32>>,  // index into population, not id

    // Spatial (grid position)
    pub xs: Vec<f32>,
    pub ys: Vec<f32>,
}

impl Population {
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    fn push_agent(&mut self, agent: AgentInit) {
        self.ids.push(agent.id);
        self.sexes.push(agent.sex);
        self.ages.push(agent.age);
        self.fertilities.push(agent.fertility);
        self.healths.push(agent.health);
        self.skill_types.push(agent.skill_type);
        self.skill_levels.push(agent.skill_level);
        self.statuses.push(agent.status);
        self.prestiges.push(agent.prestige);
        self.aggressions.push(agent.aggression);
        self.cooperations.push(agent.cooperation);
        self.resources.push(agent.resources);
        self.surpluses.push(agent.surplus);
        self.norms.push(agent.norms);
        self.innovations.push(agent.innovation);
        self.kin_groups.push(agent.kin_group);
        self.partners.push(None);
        self.patrons.push(None);
        self.xs.push(agent.x);
        self.ys.push(agent.y);
    }

    /// Remove agent at index by swap-removing (O(1)).
    fn swap_remove(&mut self, idx: usize) {
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
        self.norms.swap_remove(idx);
        self.innovations.swap_remove(idx);
        self.kin_groups.swap_remove(idx);
        self.partners.swap_remove(idx);
        self.patrons.swap_remove(idx);
        self.xs.swap_remove(idx);
        self.ys.swap_remove(idx);
    }

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
            norms: Vec::new(),
            innovations: Vec::new(),
            kin_groups: Vec::new(),
            partners: Vec::new(),
            patrons: Vec::new(),
            xs: Vec::new(),
            ys: Vec::new(),
        }
    }
}

struct AgentInit {
    id: u64,
    sex: Sex,
    age: u16,
    fertility: f32,
    health: f32,
    skill_type: SkillType,
    skill_level: f32,
    status: f32,
    prestige: f32,
    aggression: f32,
    cooperation: f32,
    resources: f32,
    surplus: f32,
    norms: u64,
    innovation: f32,
    kin_group: u32,
    x: f32,
    y: f32,
}

// ---------------------------------------------------------------------------
// Spatial grid for neighbor lookups
// ---------------------------------------------------------------------------

/// Spatial hash grid for O(1) neighbor queries.
struct SpatialGrid {
    cell_size: f32,
    cols: usize,
    rows: usize,
    cells: Vec<Vec<u32>>, // cell -> list of agent indices
}

impl SpatialGrid {
    fn build(xs: &[f32], ys: &[f32], cell_size: f32, world_size: f32) -> Self {
        let dim = (world_size / cell_size).ceil() as usize;
        let cols = dim.max(1);
        let rows = dim.max(1);
        let mut cells = vec![Vec::new(); cols * rows];
        for (i, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
            let cx = ((*x / cell_size).floor() as usize).min(cols - 1);
            let cy = ((*y / cell_size).floor() as usize).min(rows - 1);
            cells[cy * cols + cx].push(i as u32);
        }
        Self {
            cell_size,
            cols,
            rows,
            cells,
        }
    }

    fn neighbors_of(&self, x: f32, y: f32) -> Vec<u32> {
        let cx = ((x / self.cell_size).floor() as i32).clamp(0, self.cols as i32 - 1);
        let cy = ((y / self.cell_size).floor() as i32).clamp(0, self.rows as i32 - 1);
        let mut result = Vec::new();
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < self.cols as i32 && ny >= 0 && ny < self.rows as i32 {
                    let cell_idx = ny as usize * self.cols + nx as usize;
                    result.extend_from_slice(&self.cells[cell_idx]);
                }
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for individual agent simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentSimConfig {
    pub seed: u64,
    pub initial_population: u32,
    pub ticks: u32,
    pub world_size: f32,
    pub interaction_radius: f32,
    /// Base resource regeneration per tick across the world.
    pub resource_regen: f32,
    /// Maximum age before guaranteed death.
    pub max_age: u16,
    /// Minimum population below which simulation stops.
    pub min_population: u32,
    /// Maximum population above which birth rate is suppressed.
    pub max_population: u32,
}

impl Default for AgentSimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            initial_population: 150,
            ticks: 500,
            world_size: 100.0,
            interaction_radius: 8.0,
            resource_regen: 0.05,
            max_age: 80,
            min_population: 10,
            max_population: 10_000,
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
}

// ---------------------------------------------------------------------------
// Measurement functions
// ---------------------------------------------------------------------------

fn measure_gini(resources: &[f32]) -> f32 {
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

fn measure_gini_fast(resources: &[f32]) -> f32 {
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

fn measure_skill_entropy(skill_types: &[SkillType]) -> f32 {
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

fn measure_hierarchy_depth(patrons: &[Option<u32>]) -> u32 {
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

fn count_kin_groups(kin_groups: &[u32]) -> u32 {
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

fn mean_group_size(kin_groups: &[u32]) -> f32 {
    let n_groups = count_kin_groups(kin_groups);
    if n_groups == 0 {
        return 0.0;
    }
    kin_groups.len() as f32 / n_groups as f32
}

fn measure_emergent_state(
    pop: &Population,
    cooperation_events: u32,
    conflict_events: u32,
    total_interactions: u32,
) -> EmergentState {
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
        cooperation_rate: if total_interactions > 0 {
            cooperation_events as f32 / total_interactions as f32
        } else {
            0.0
        },
        conflict_rate: if total_interactions > 0 {
            conflict_events as f32 / total_interactions as f32
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
    let kin_group_count = (n / 8).max(2); // ~8 agents per initial kin group

    for i in 0..n {
        let sex = if rand01(&mut rng) < 0.5 {
            Sex::Male
        } else {
            Sex::Female
        };
        let age = (rand01(&mut rng) * 25.0) as u16 + 5; // start younger for demographic balance
        let fertility = match sex {
            Sex::Female => (0.8 - (age as f32 - 20.0).abs() * 0.02).clamp(0.0, 1.0),
            Sex::Male => (0.9 - (age as f32 - 25.0).abs() * 0.01).clamp(0.0, 1.0),
        };
        let skill_type = match (rand01(&mut rng) * 5.0) as u32 {
            0 => SkillType::Forager,
            1 => SkillType::Crafter,
            2 => SkillType::Builder,
            3 => SkillType::Leader,
            _ => SkillType::Warrior,
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
            norms: (rand01(&mut rng) * f64::from(u32::MAX)) as u64,
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
    // Delegation choices: agent_idx -> chosen_patron_idx
    delegation_choices: Vec<(u32, u32)>,
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
    best_patron: Option<u32>,
}

fn compute_interactions(
    pop: &Population,
    grid: &SpatialGrid,
    tick: u32,
    seed: u64,
) -> InteractionEffects {
    let n = pop.len();
    // Compute per-agent effects in parallel, then merge.
    let per_agent: Vec<AgentInteractionResult> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut rng = (pop.ids[i])
                .wrapping_mul(6364136223846793005)
                .wrapping_add(tick as u64)
                .wrapping_add(seed)
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
            let mut best_patron: Option<u32> = None;
            let mut best_patron_score = 0.0_f32;

            let my_coop = pop.cooperations[i];
            let my_aggr = pop.aggressions[i];
            let _my_res = pop.resources[i];
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
                // Grid already limits to nearby, but check exact radius
                // Use cell_size^2 as max range (slightly larger than interaction_radius)
                let max_dist_sq = grid.cell_size * grid.cell_size * 4.0;
                if dist_sq > max_dist_sq {
                    continue;
                }

                interaction_count += 1;
                let same_kin = my_kin == pop.kin_groups[j];
                let other_coop = pop.cooperations[j];
                let other_aggr = pop.aggressions[j];

                // Interaction decision: cooperate, trade, or conflict
                let coop_tendency =
                    my_coop * 0.5 + other_coop * 0.3 + if same_kin { 0.2 } else { 0.0 };
                let conflict_tendency =
                    my_aggr * 0.4 + other_aggr * 0.3 + if !same_kin { 0.15 } else { 0.0 };
                let trade_tendency = if my_skill != pop.skill_types[j] {
                    0.4
                } else {
                    0.15
                };

                let total = coop_tendency + conflict_tendency + trade_tendency;
                let roll = rand01f(&mut rng) * total;

                if roll < coop_tendency {
                    // Cooperation: mutual effort produces surplus for both
                    let coop_bonus = 0.01 * (pop.cooperations[j] + my_coop) * 0.5;
                    res_delta += coop_bonus;
                    prestige_delta += 0.005;
                    coop_count += 1;
                } else if roll < coop_tendency + conflict_tendency {
                    // Conflict: winner takes resources, loser loses health
                    let my_power = my_status * 0.4 + pop.skill_levels[i] * 0.3 + my_aggr * 0.3;
                    let other_power =
                        pop.statuses[j] * 0.4 + pop.skill_levels[j] * 0.3 + other_aggr * 0.3;
                    if my_power > other_power + rand01f(&mut rng) * 0.2 {
                        res_delta += 0.05;
                        status_delta += 0.01;
                    } else {
                        res_delta -= 0.03;
                        health_delta -= 0.005;
                    }
                    conflict_count += 1;
                } else {
                    // Trade: complementary skills produce surplus
                    let skill_bonus = if my_skill != pop.skill_types[j] {
                        0.03 * (pop.skill_levels[i] + pop.skill_levels[j])
                    } else {
                        0.005
                    };
                    res_delta += skill_bonus;
                    trade_count += 1;
                }

                // Delegation: consider this neighbor as patron
                if pop.statuses[j] > my_status + 0.1
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
                health_delta: health_delta.max(-0.01), // cap health loss per tick
                coop_count,
                conflict_count,
                trade_count,
                interaction_count,
                best_patron,
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
        delegation_choices: Vec::new(),
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
        if let Some(patron) = result.best_patron {
            effects.delegation_choices.push((i as u32, patron));
        }
    }

    effects
}

fn apply_effects(pop: &mut Population, effects: &InteractionEffects, cfg: &AgentSimConfig) {
    let n = pop.len();
    for i in 0..n {
        pop.resources[i] =
            (pop.resources[i] + effects.resource_deltas[i] + cfg.resource_regen).max(0.0);
        pop.statuses[i] = (pop.statuses[i] + effects.status_deltas[i]).clamp(0.0, 2.0);
        pop.prestiges[i] = (pop.prestiges[i] + effects.prestige_deltas[i]).clamp(0.0, 5.0);
        pop.healths[i] = (pop.healths[i] + effects.health_deltas[i]).clamp(0.0, 1.0);
        // Well-fed agents recover health (stronger recovery = population sustains)
        if pop.resources[i] > 0.2 {
            let recovery = 0.02 * (pop.resources[i] - 0.2).min(1.0);
            pop.healths[i] = (pop.healths[i] + recovery).min(1.0);
        }
        pop.surpluses[i] = (pop.resources[i] - 0.5).max(0.0); // surplus = above subsistence

        // Skill improvement through practice
        pop.skill_levels[i] = (pop.skill_levels[i] + 0.002).min(1.0);
    }

    // Apply delegation choices
    for &(agent, patron) in &effects.delegation_choices {
        if (patron as usize) < n {
            pop.patrons[agent as usize] = Some(patron);
            // Patron extracts small tax (proto-hierarchy cost)
            let tax = pop.resources[agent as usize] * 0.01;
            pop.resources[agent as usize] -= tax;
            pop.resources[patron as usize] += tax;
            pop.prestiges[patron as usize] += 0.002;
        }
    }
}

// ---------------------------------------------------------------------------
// Lifecycle: aging, death, courtship, birth
// ---------------------------------------------------------------------------

fn lifecycle_tick(pop: &mut Population, tick: u32, cfg: &AgentSimConfig, next_id: &mut u64) {
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
        pop.healths[i] -= 0.001 + 0.008 * age_factor;
        pop.healths[i] = pop.healths[i].clamp(0.0, 1.0);

        // Fertility peaks mid-life, declines at extremes
        let age_f = pop.ages[i] as f32;
        pop.fertilities[i] = match pop.sexes[i] {
            Sex::Female => (0.8 - (age_f - 25.0).abs() * 0.02).clamp(0.0, 1.0),
            Sex::Male => (0.9 - (age_f - 30.0).abs() * 0.012).clamp(0.0, 1.0),
        };
    }

    // Death: old age, low health, or starvation
    let mut deaths = Vec::new();
    for i in (0..pop.len()).rev() {
        let die = pop.ages[i] >= cfg.max_age
            || pop.healths[i] < 0.01
            || (pop.resources[i] < 0.01 && rand01f(&mut rng) < 0.1);
        if die {
            deaths.push(i);
        }
    }
    // Remove from highest index first (swap_remove is safe this way)
    for &idx in &deaths {
        pop.swap_remove(idx);
    }

    // Fix patron/partner references after swap_remove
    // (swap_remove moves last element to removed position)
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

    let mut births: Vec<AgentInit> = Vec::new();
    let pop_len = pop.len();

    for i in 0..pop_len {
        if pop.sexes[i] != Sex::Female {
            continue;
        }
        if pop.fertilities[i] < 0.2 || pop.ages[i] < 8 || pop.ages[i] > 50 {
            continue;
        }
        if pop.resources[i] < 0.4 {
            continue; // needs resources to reproduce
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
            // Sexual selection: find nearby high-status male
            find_mate(pop, i, &mut rng, cfg)
        };

        if let Some(m) = mate {
            let birth_prob = 0.25 * pop.fertilities[i] * pop.healths[i];
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
            let skill = if rand01(&mut rng) < 0.7 {
                pop.skill_types[i] // mother's skill more likely
            } else if rand01(&mut rng) < 0.5 {
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

            let child_norms = if rand01(&mut rng) < 0.5 {
                pop.norms[i]
            } else {
                pop.norms[m]
            };

            births.push(AgentInit {
                id: *next_id,
                sex: child_sex,
                age: 0,
                fertility: 0.0, // too young
                health: 0.9,
                skill_type: skill,
                skill_level: 0.05,
                status: 0.2,
                prestige: 0.0,
                aggression: ((pop.aggressions[i] + pop.aggressions[m]) * 0.5
                    + (rand01f(&mut rng) - 0.5) * 0.1)
                    .clamp(0.0, 1.0),
                cooperation: ((pop.cooperations[i] + pop.cooperations[m]) * 0.5
                    + (rand01f(&mut rng) - 0.5) * 0.1)
                    .clamp(0.0, 1.0),
                resources: 0.3,
                surplus: 0.0,
                norms: child_norms
                    ^ if rand01(&mut rng) < 0.05 {
                        1 << ((rand01(&mut rng) * 16.0) as u64)
                    } else {
                        0
                    },
                innovation: ((pop.innovations[i] + pop.innovations[m]) * 0.5
                    + (rand01f(&mut rng) - 0.5) * 0.05)
                    .clamp(0.0, 1.0),
                kin_group: pop.kin_groups[i], // inherit mother's kin group
                x: pop.xs[i] + (rand01f(&mut rng) - 0.5) * 2.0,
                y: pop.ys[i] + (rand01f(&mut rng) - 0.5) * 2.0,
            });
            *next_id += 1;

            // Reproduction cost
            pop.resources[i] -= 0.2;
            pop.healths[i] -= 0.05;
        }
    }

    for birth in births {
        pop.push_agent(birth);
    }
}

fn find_mate(
    pop: &Population,
    female_idx: usize,
    rng: &mut u64,
    cfg: &AgentSimConfig,
) -> Option<usize> {
    let fx = pop.xs[female_idx];
    let fy = pop.ys[female_idx];
    let r_sq = cfg.interaction_radius * cfg.interaction_radius * 4.0;

    let mut best: Option<usize> = None;
    let mut best_score = f32::NEG_INFINITY;

    for j in 0..pop.len() {
        if pop.sexes[j] != Sex::Male || pop.ages[j] < 8 {
            continue;
        }
        let dx = fx - pop.xs[j];
        let dy = fy - pop.ys[j];
        if dx * dx + dy * dy > r_sq {
            continue;
        }
        // Sexual selection: prefer high status, resources, prestige
        let score = pop.statuses[j] * 0.3
            + pop.resources[j] * 0.3
            + pop.prestiges[j] * 0.3
            + rand01f(rng) * 0.1;
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

fn movement_tick(pop: &mut Population, tick: u32, seed: u64) {
    let n = pop.len();
    if n == 0 {
        return;
    }

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

    // Move each agent slightly toward kin centroid + random drift
    for i in 0..n {
        let mut rng = pop.ids[i]
            .wrapping_mul(2862933555777941757)
            .wrapping_add(tick as u64)
            .wrapping_add(seed)
            .wrapping_add(0xDEAD)
            .max(1);

        let kg = pop.kin_groups[i] as usize;
        if kg < kin_cx.len() && kin_count[kg] > 1 {
            let cx = kin_cx[kg] as f32;
            let cy = kin_cy[kg] as f32;
            let dx = (cx - pop.xs[i]) * 0.02; // gentle pull toward kin
            let dy = (cy - pop.ys[i]) * 0.02;
            pop.xs[i] += dx + (rand01f(&mut rng) - 0.5) * 0.5;
            pop.ys[i] += dy + (rand01f(&mut rng) - 0.5) * 0.5;
        } else {
            pop.xs[i] += (rand01f(&mut rng) - 0.5) * 0.8;
            pop.ys[i] += (rand01f(&mut rng) - 0.5) * 0.8;
        }

        // Clamp to world bounds
        pop.xs[i] = pop.xs[i].clamp(0.0, 99.9);
        pop.ys[i] = pop.ys[i].clamp(0.0, 99.9);
    }
}

// ---------------------------------------------------------------------------
// Main simulation loop
// ---------------------------------------------------------------------------

/// Run the individual agent simulation.
#[must_use]
pub fn simulate_agents(cfg: AgentSimConfig) -> AgentSimResult {
    let mut pop = seed_population(&cfg);
    let mut next_id = cfg.initial_population as u64;
    let mut snapshots = Vec::with_capacity(cfg.ticks as usize);

    for tick in 0..cfg.ticks {
        if (pop.len() as u32) < cfg.min_population {
            break;
        }

        // Build spatial index
        let grid = SpatialGrid::build(&pop.xs, &pop.ys, cfg.interaction_radius, cfg.world_size);

        // Compute and apply interactions
        let effects = compute_interactions(&pop, &grid, tick, cfg.seed);
        let coop_events = effects.cooperation_events;
        let conflict_events = effects.conflict_events;
        let total_interactions = effects.total_interactions;
        apply_effects(&mut pop, &effects, &cfg);

        // Lifecycle: aging, death, reproduction
        lifecycle_tick(&mut pop, tick, &cfg, &mut next_id);

        // Movement
        movement_tick(&mut pop, tick, cfg.seed);

        // Measure emergent state
        let emergent =
            measure_emergent_state(&pop, coop_events, conflict_events, total_interactions);
        snapshots.push(AgentSnapshot { tick, emergent });
    }

    AgentSimResult {
        snapshots,
        final_population: pop,
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
        // Gini should change over time (not necessarily always increase
        // but should not stay exactly the same)
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
        // Over 100 ticks with 120 agents, some hierarchy should emerge
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
            initial_population: 150,
            ticks: 300,
            resource_regen: 0.15, // generous resources
            world_size: 30.0,
            max_population: 1000,
            min_population: 2,
            ..AgentSimConfig::default()
        });
        // With generous resources, population should sustain and not collapse
        let final_pop = result.final_population.len();
        assert!(
            final_pop > 50,
            "population should sustain with good resources, final was {final_pop}"
        );
        // And the simulation should run all ticks (not hit min_population early exit)
        assert_eq!(
            result.snapshots.len(),
            300,
            "should run all ticks without collapse"
        );
    }

    #[test]
    fn scarce_resources_limit_population() {
        let result = simulate_agents(AgentSimConfig {
            initial_population: 80,
            ticks: 200,
            resource_regen: 0.005, // very scarce
            world_size: 50.0,
            ..AgentSimConfig::default()
        });
        let final_pop = result.final_population.len();
        // Should not have grown much (or may have shrunk)
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
}
