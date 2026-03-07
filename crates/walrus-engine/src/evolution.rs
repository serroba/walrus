use crate::SubsistenceMode;

/// Dunbar reference sizes used to classify social scale transitions.
pub const DUNBAR_NUMBERS: [u32; 6] = [5, 15, 50, 150, 500, 1_500];

/// Group scale bucket aligned with Dunbar-style social layers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupScale {
    Intimate,
    Sympathy,
    Band,
    Village,
    Polity,
    Civilizational,
}

/// Returns a Dunbar-inspired social layer for a given population size.
#[must_use]
pub fn dunbar_group_scale(population: u32) -> GroupScale {
    match population {
        0..=5 => GroupScale::Intimate,
        6..=15 => GroupScale::Sympathy,
        16..=50 => GroupScale::Band,
        51..=150 => GroupScale::Village,
        151..=500 => GroupScale::Polity,
        _ => GroupScale::Civilizational,
    }
}

/// Continent-level parameters inspired by Diamond-style geographic constraints.
#[derive(Clone, Debug, PartialEq)]
pub struct Continent {
    pub name: String,
    pub domesticable_biomass: f64,
    pub diffusion_access: f64,
    pub energy_endowment: f64,
    pub carrying_capacity: f64,
    pub regen_rate: f64,
    pub shock_risk: f64,
}

/// Dynamic continent resource stock and depletion state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContinentState {
    pub stock: f64,
    pub depletion: f64,
}

/// Directed migration/trade corridor between continents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corridor {
    pub from: usize,
    pub to: usize,
    pub strength: f64,
}

/// Geography layer for multi-society simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldMap {
    pub continents: Vec<Continent>,
    pub states: Vec<ContinentState>,
    pub corridors: Vec<Corridor>,
}

impl WorldMap {
    #[must_use]
    pub fn default_world() -> Self {
        let continents = vec![
            Continent {
                name: "Africa".to_string(),
                domesticable_biomass: 0.50,
                diffusion_access: 0.45,
                energy_endowment: 0.55,
                carrying_capacity: 1.0,
                regen_rate: 0.012,
                shock_risk: 0.08,
            },
            Continent {
                name: "Eurasia".to_string(),
                domesticable_biomass: 0.86,
                diffusion_access: 0.88,
                energy_endowment: 0.92,
                carrying_capacity: 1.4,
                regen_rate: 0.014,
                shock_risk: 0.07,
            },
            Continent {
                name: "Americas".to_string(),
                domesticable_biomass: 0.62,
                diffusion_access: 0.50,
                energy_endowment: 0.68,
                carrying_capacity: 1.1,
                regen_rate: 0.013,
                shock_risk: 0.09,
            },
            Continent {
                name: "Oceania".to_string(),
                domesticable_biomass: 0.42,
                diffusion_access: 0.22,
                energy_endowment: 0.40,
                carrying_capacity: 0.6,
                regen_rate: 0.011,
                shock_risk: 0.11,
            },
        ];
        let states = continents
            .iter()
            .map(|c| ContinentState {
                stock: c.carrying_capacity,
                depletion: 0.0,
            })
            .collect::<Vec<ContinentState>>();
        let corridors = vec![
            Corridor {
                from: 0,
                to: 1,
                strength: 0.40,
            },
            Corridor {
                from: 1,
                to: 0,
                strength: 0.40,
            },
            Corridor {
                from: 1,
                to: 2,
                strength: 0.28,
            },
            Corridor {
                from: 2,
                to: 1,
                strength: 0.28,
            },
            Corridor {
                from: 2,
                to: 3,
                strength: 0.12,
            },
        ];

        Self {
            continents,
            states,
            corridors,
        }
    }
}

/// NK fitness landscape for adaptive institutional/genetic search.
#[derive(Clone, Debug, PartialEq)]
pub struct NkLandscape {
    pub n: usize,
    pub k: usize,
    tables: Vec<Vec<f64>>,
}

impl NkLandscape {
    #[must_use]
    pub fn deterministic(n: usize, k: usize, seed: u64) -> Self {
        let n_safe = n.clamp(2, 20);
        let k_safe = k.min(n_safe - 1);
        let table_len = 1_usize << (k_safe + 1);
        let mut rng = seed.max(1);
        let mut tables = Vec::with_capacity(n_safe);

        for _ in 0..n_safe {
            let mut row = Vec::with_capacity(table_len);
            for _ in 0..table_len {
                row.push(rand01(&mut rng));
            }
            tables.push(row);
        }

        Self {
            n: n_safe,
            k: k_safe,
            tables,
        }
    }

    #[must_use]
    pub fn fitness(&self, genome_bits: u64) -> f64 {
        let mut total = 0.0;
        for locus in 0..self.n {
            let mut index = bit(genome_bits, locus);
            for j in 1..=self.k {
                let neighbor = (locus + j) % self.n;
                index |= bit(genome_bits, neighbor) << j;
            }
            total += self.tables[locus][index as usize];
        }
        (total / (self.n as f64)).clamp(0.0, 1.0)
    }
}

/// Evolvable traits represented as a compact binary genome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Genome {
    pub bits: u64,
    pub mutation_rate: f64,
}

/// Society-level actor carrying population and institutional traits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SocietyActor {
    pub id: u64,
    pub continent: usize,
    pub mode: SubsistenceMode,
    pub population: u32,
    pub complexity: f64,
    pub surplus: f64,
    pub trust: f64,
    pub resilience: f64,
    pub genome: Genome,
}

/// Actor-model messages delivered to societies every generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActorMessage {
    ClimateShock { severity: f64 },
    ResourcePulse { abundance: f64 },
    MigrationLink { strength: f64 },
}

/// Configures multi-generation evolutionary simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvolutionConfig {
    pub seed: u64,
    pub generations: u32,
    pub initial_societies: u32,
    pub nk_n: usize,
    pub nk_k: usize,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            seed: 2026,
            generations: 320,
            initial_societies: 16,
            nk_n: 14,
            nk_k: 3,
        }
    }
}

/// Aggregated run-level signals for emergence and collapse analysis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EvolutionSnapshot {
    pub generation: u32,
    pub population_total: u64,
    pub mean_complexity: f64,
    pub mean_energy_access: f64,
    pub collapse_events: u32,
    pub emergent_civilizations: u32,
}

/// Final outcome summary per continent.
#[derive(Clone, Debug, PartialEq)]
pub struct ContinentOutcome {
    pub name: String,
    pub surviving_societies: usize,
    pub total_population: u64,
    pub mean_complexity: f64,
    pub mean_depletion: f64,
}

/// End-to-end result including generation snapshots and final map state.
#[derive(Clone, Debug, PartialEq)]
pub struct EvolutionResult {
    pub snapshots: Vec<EvolutionSnapshot>,
    pub continent_outcomes: Vec<ContinentOutcome>,
    pub final_societies: Vec<SocietyActor>,
}

#[must_use]
pub fn simulate_evolution(config: EvolutionConfig) -> EvolutionResult {
    let mut rng = config.seed.max(1);
    let mut map = WorldMap::default_world();
    let landscape = NkLandscape::deterministic(config.nk_n, config.nk_k, config.seed ^ 0xa5a5);
    let mut societies = seed_societies(config.initial_societies, &map, &mut rng, config.nk_n);
    let mut snapshots = Vec::with_capacity(config.generations as usize);

    for generation in 0..config.generations {
        let mut collapse_events = 0_u32;
        let continent_counts = per_continent_counts(&societies, map.continents.len());

        for society in &mut societies {
            let c_idx = society.continent;
            let continent = &map.continents[c_idx];
            let mut state = map.states[c_idx];
            let messages = actor_messages_for(c_idx, &map, &mut rng);
            apply_actor_messages(society, &messages);

            let nk_fit = landscape.fitness(society.genome.bits);
            let energy_access =
                (continent.energy_endowment * state.stock * (1.0 - state.depletion)
                    + 0.35 * continent.domesticable_biomass
                    + 0.22 * continent.diffusion_access)
                    .clamp(0.0, 2.0);

            let scale = dunbar_group_scale(society.population);
            let scale_bonus = match scale {
                GroupScale::Intimate => 0.02,
                GroupScale::Sympathy => 0.04,
                GroupScale::Band => 0.08,
                GroupScale::Village => 0.12,
                GroupScale::Polity => 0.18,
                GroupScale::Civilizational => 0.24,
            };

            let innovation =
                (0.48 * nk_fit + 0.27 * continent.diffusion_access + scale_bonus).clamp(0.0, 1.4);
            let complexity_gain =
                (0.20 * energy_access + 0.24 * innovation + 0.08 * society.trust).clamp(0.0, 1.0);
            let maintenance = (0.06
                + 0.16 * society.complexity
                + 0.10 * society.complexity.powi(2)
                + 0.08 * state.depletion)
                .clamp(0.0, 2.0);

            society.surplus = (society.surplus + complexity_gain - maintenance).clamp(-1.0, 2.5);
            society.complexity =
                (society.complexity + 0.14 * complexity_gain - 0.10 * maintenance).clamp(0.0, 1.8);

            let stress_shock = if rand01(&mut rng) < continent.shock_risk {
                rand01(&mut rng) * 0.35
            } else {
                0.0
            };
            society.resilience =
                (society.resilience + 0.04 * innovation - 0.05 * stress_shock).clamp(0.05, 1.3);
            society.trust =
                (society.trust + 0.03 * innovation - 0.04 * stress_shock).clamp(0.0, 1.0);

            let growth = (0.012 * society.surplus + 0.010 * society.resilience
                - 0.008 * stress_shock)
                .clamp(-0.08, 0.12);
            let next_population = ((society.population as f64) * (1.0 + growth)).round() as i64;
            society.population = next_population.max(4) as u32;

            let local_count = continent_counts[c_idx].max(1) as f64;
            let extraction = ((society.population as f64) / 140_000.0)
                * (1.0 + 0.8 * society.complexity)
                + 0.012 * society.surplus.max(0.0);
            let regen =
                continent.regen_rate * continent.carrying_capacity * (1.0 - 0.35 * state.depletion);
            state.stock =
                (state.stock + regen - extraction).clamp(0.0, continent.carrying_capacity * 1.1);
            let depletion_load = extraction / local_count;
            state.depletion = (state.depletion + 0.08 * depletion_load
                - 0.45 * continent.regen_rate)
                .clamp(0.0, 1.0);

            let collapse_trigger = (state.stock < 0.12 * continent.carrying_capacity
                || state.depletion > 0.88)
                && society.complexity > 0.55;
            if collapse_trigger {
                collapse_events = collapse_events.saturating_add(1);
                society.population = ((society.population as f64) * 0.68).round().max(4.0) as u32;
                society.complexity = (society.complexity * 0.72).clamp(0.0, 1.8);
                society.surplus = (society.surplus - 0.22).clamp(-1.0, 2.5);
                society.mode = SubsistenceMode::HunterGatherer;
            } else {
                society.mode = mode_from_population(society.population, society.surplus);
            }

            society.genome = mutate_genome(society.genome, config.nk_n, &mut rng);
            map.states[c_idx] = state;
        }

        let mut offspring = Vec::new();
        let mut id_counter = societies.len() as u64 + (generation as u64) * 10;
        for society in &societies {
            if society.surplus > 0.38 && society.population > 90 && rand01(&mut rng) < 0.08 {
                let target = migrate_target(society.continent, &map.corridors, &mut rng)
                    .unwrap_or(society.continent);
                let mut child = *society;
                child.id = id_counter;
                id_counter = id_counter.saturating_add(1);
                child.continent = target;
                child.population = ((society.population as f64) * 0.16).round().max(5.0) as u32;
                child.complexity = (society.complexity * 0.80).clamp(0.0, 1.8);
                child.surplus = (society.surplus * 0.70).clamp(-1.0, 2.5);
                child.genome = mutate_genome(society.genome, config.nk_n, &mut rng);
                offspring.push(child);
            }
        }
        societies.extend(offspring);

        societies.retain(|s| s.population > 3);

        let population_total = societies
            .iter()
            .map(|s| u64::from(s.population))
            .sum::<u64>();
        let mean_complexity = if societies.is_empty() {
            0.0
        } else {
            societies.iter().map(|s| s.complexity).sum::<f64>() / (societies.len() as f64)
        };
        let mean_energy_access = if societies.is_empty() {
            0.0
        } else {
            societies
                .iter()
                .map(|s| {
                    let c = &map.continents[s.continent];
                    let st = map.states[s.continent];
                    (c.energy_endowment * st.stock * (1.0 - st.depletion)
                        + 0.25 * c.domesticable_biomass
                        + 0.15 * c.diffusion_access)
                        .clamp(0.0, 1.5)
                })
                .sum::<f64>()
                / (societies.len() as f64)
        };
        let emergent_civilizations = societies
            .iter()
            .filter(|s| s.population >= 150 && s.complexity > 0.65)
            .count() as u32;

        snapshots.push(EvolutionSnapshot {
            generation,
            population_total,
            mean_complexity,
            mean_energy_access,
            collapse_events,
            emergent_civilizations,
        });
    }

    let mut continent_outcomes = Vec::new();
    for (idx, continent) in map.continents.iter().enumerate() {
        let local = societies
            .iter()
            .filter(|s| s.continent == idx)
            .copied()
            .collect::<Vec<SocietyActor>>();
        let total_population = local.iter().map(|s| u64::from(s.population)).sum::<u64>();
        let mean_complexity = if local.is_empty() {
            0.0
        } else {
            local.iter().map(|s| s.complexity).sum::<f64>() / (local.len() as f64)
        };

        continent_outcomes.push(ContinentOutcome {
            name: continent.name.clone(),
            surviving_societies: local.len(),
            total_population,
            mean_complexity,
            mean_depletion: map.states[idx].depletion,
        });
    }

    EvolutionResult {
        snapshots,
        continent_outcomes,
        final_societies: societies,
    }
}

fn actor_messages_for(continent_idx: usize, map: &WorldMap, rng: &mut u64) -> Vec<ActorMessage> {
    let mut messages = Vec::new();
    let continent = &map.continents[continent_idx];
    let state = map.states[continent_idx];

    if rand01(rng) < continent.shock_risk {
        messages.push(ActorMessage::ClimateShock {
            severity: (0.15 + 0.55 * rand01(rng)).clamp(0.0, 1.0),
        });
    }
    messages.push(ActorMessage::ResourcePulse {
        abundance: (state.stock / continent.carrying_capacity.max(1e-9)).clamp(0.0, 1.5),
    });

    let link_strength = map
        .corridors
        .iter()
        .filter(|c| c.from == continent_idx)
        .map(|c| c.strength)
        .sum::<f64>();
    messages.push(ActorMessage::MigrationLink {
        strength: link_strength.clamp(0.0, 1.5),
    });
    messages
}

fn apply_actor_messages(actor: &mut SocietyActor, messages: &[ActorMessage]) {
    for message in messages {
        match *message {
            ActorMessage::ClimateShock { severity } => {
                actor.resilience = (actor.resilience - 0.08 * severity).clamp(0.05, 1.3);
                actor.trust = (actor.trust - 0.06 * severity).clamp(0.0, 1.0);
                actor.surplus = (actor.surplus - 0.10 * severity).clamp(-1.0, 2.5);
            }
            ActorMessage::ResourcePulse { abundance } => {
                actor.surplus = (actor.surplus + 0.05 * abundance).clamp(-1.0, 2.5);
            }
            ActorMessage::MigrationLink { strength } => {
                actor.trust = (actor.trust + 0.02 * strength).clamp(0.0, 1.0);
                actor.resilience = (actor.resilience + 0.02 * strength).clamp(0.05, 1.3);
            }
        }
    }
}

fn seed_societies(count: u32, map: &WorldMap, rng: &mut u64, nk_n: usize) -> Vec<SocietyActor> {
    let mut out = Vec::with_capacity(count as usize);
    for id in 0..count {
        let continent = ((rand01(rng) * (map.continents.len() as f64)).floor() as usize)
            .min(map.continents.len().saturating_sub(1));
        let pop = (12.0 + rand01(rng) * 90.0).round() as u32;
        out.push(SocietyActor {
            id: u64::from(id),
            continent,
            mode: SubsistenceMode::HunterGatherer,
            population: pop,
            complexity: (0.08 + rand01(rng) * 0.2).clamp(0.0, 1.8),
            surplus: (0.02 + rand01(rng) * 0.14).clamp(-1.0, 2.5),
            trust: (0.40 + rand01(rng) * 0.4).clamp(0.0, 1.0),
            resilience: (0.35 + rand01(rng) * 0.4).clamp(0.05, 1.3),
            genome: Genome {
                bits: random_genome(nk_n, rng),
                mutation_rate: 0.015,
            },
        });
    }
    out
}

fn random_genome(n: usize, rng: &mut u64) -> u64 {
    let mut bits = 0_u64;
    for locus in 0..n.min(63) {
        if rand01(rng) < 0.5 {
            bits |= 1_u64 << locus;
        }
    }
    bits
}

fn mutate_genome(genome: Genome, n: usize, rng: &mut u64) -> Genome {
    let mut bits = genome.bits;
    for locus in 0..n.min(63) {
        if rand01(rng) < genome.mutation_rate {
            bits ^= 1_u64 << locus;
        }
    }
    Genome { bits, ..genome }
}

fn mode_from_population(population: u32, surplus: f64) -> SubsistenceMode {
    if population >= 500 && surplus > 0.32 {
        SubsistenceMode::Agriculture
    } else if population >= 120 && surplus > 0.18 {
        SubsistenceMode::Sedentary
    } else {
        SubsistenceMode::HunterGatherer
    }
}

fn migrate_target(source: usize, corridors: &[Corridor], rng: &mut u64) -> Option<usize> {
    let options = corridors
        .iter()
        .filter(|c| c.from == source)
        .copied()
        .collect::<Vec<Corridor>>();
    if options.is_empty() {
        return None;
    }
    let total = options.iter().map(|o| o.strength.max(0.0)).sum::<f64>();
    if total <= 0.0 {
        return Some(options[0].to);
    }

    let draw = rand01(rng) * total;
    let mut acc = 0.0;
    for option in options {
        acc += option.strength.max(0.0);
        if draw <= acc {
            return Some(option.to);
        }
    }
    None
}

fn per_continent_counts(societies: &[SocietyActor], continent_count: usize) -> Vec<usize> {
    let mut counts = vec![0_usize; continent_count];
    for society in societies {
        if society.continent < counts.len() {
            counts[society.continent] = counts[society.continent].saturating_add(1);
        }
    }
    counts
}

fn bit(bits: u64, idx: usize) -> u32 {
    ((bits >> idx) & 1) as u32
}

fn rand01(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*state as f64) / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_actor_messages, dunbar_group_scale, simulate_evolution, ActorMessage,
        EvolutionConfig, Genome, GroupScale, NkLandscape, SocietyActor, WorldMap,
    };
    use crate::SubsistenceMode;

    #[test]
    fn dunbar_scale_thresholds_match_expected_buckets() {
        assert_eq!(dunbar_group_scale(5), GroupScale::Intimate);
        assert_eq!(dunbar_group_scale(15), GroupScale::Sympathy);
        assert_eq!(dunbar_group_scale(50), GroupScale::Band);
        assert_eq!(dunbar_group_scale(150), GroupScale::Village);
        assert_eq!(dunbar_group_scale(500), GroupScale::Polity);
        assert_eq!(dunbar_group_scale(1_501), GroupScale::Civilizational);
    }

    #[test]
    fn nk_fitness_is_bounded() {
        let nk = NkLandscape::deterministic(12, 3, 7);
        let fit = nk.fitness(0b1010_1010_1100);
        assert!((0.0..=1.0).contains(&fit));
    }

    #[test]
    fn richer_continent_has_higher_energy_endowment_in_map() {
        let map = WorldMap::default_world();
        let eurasia = &map.continents[1];
        let oceania = &map.continents[3];
        assert!(eurasia.energy_endowment > oceania.energy_endowment);
        assert!(eurasia.domesticable_biomass > oceania.domesticable_biomass);
    }

    #[test]
    fn evolution_run_shows_both_emergence_and_collapse_events() {
        let result = simulate_evolution(EvolutionConfig {
            seed: 99,
            generations: 220,
            initial_societies: 18,
            nk_n: 12,
            nk_k: 3,
        });

        assert!(!result.snapshots.is_empty());
        let peak_complexity = result
            .snapshots
            .iter()
            .map(|s| s.mean_complexity)
            .fold(f64::NEG_INFINITY, f64::max);
        let collapse_sum = result
            .snapshots
            .iter()
            .map(|s| u64::from(s.collapse_events))
            .sum::<u64>();

        assert!(peak_complexity > 0.20);
        assert!(collapse_sum > 0);
    }

    #[test]
    fn actor_messages_modify_society_state() {
        let mut actor = SocietyActor {
            id: 1,
            continent: 0,
            mode: SubsistenceMode::HunterGatherer,
            population: 120,
            complexity: 0.3,
            surplus: 0.2,
            trust: 0.5,
            resilience: 0.6,
            genome: Genome {
                bits: 0b101010,
                mutation_rate: 0.01,
            },
        };
        let before = actor;
        let messages = [
            ActorMessage::ClimateShock { severity: 0.6 },
            ActorMessage::ResourcePulse { abundance: 1.0 },
            ActorMessage::MigrationLink { strength: 0.5 },
        ];
        apply_actor_messages(&mut actor, &messages);
        assert!(actor.trust < before.trust);
        assert!(actor.surplus < before.surplus + 0.06);
        assert!(actor.resilience > 0.0);
    }
}
