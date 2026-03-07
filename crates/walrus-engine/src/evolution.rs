use rayon::prelude::*;

use crate::{LocalSocietyState, SubsistenceMode};

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

/// Behavioral parameters that shift as social scale increases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DunbarLayerBehavior {
    pub expectation_load: f64,
    pub trust_decay: f64,
    pub communication_cost: f64,
    pub coordination_gain: f64,
}

/// Configurable Dunbar model with thresholds and behavior profiles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DunbarBehaviorModel {
    pub thresholds: [u32; 6],
    pub expectation_load: [f64; 6],
    pub trust_decay: [f64; 6],
    pub communication_cost: [f64; 6],
    pub coordination_gain: [f64; 6],
}

impl Default for DunbarBehaviorModel {
    fn default() -> Self {
        Self {
            thresholds: DUNBAR_NUMBERS,
            expectation_load: [0.06, 0.09, 0.13, 0.20, 0.31, 0.42],
            trust_decay: [0.010, 0.014, 0.018, 0.024, 0.030, 0.038],
            communication_cost: [0.01, 0.02, 0.04, 0.07, 0.12, 0.18],
            coordination_gain: [0.02, 0.04, 0.08, 0.13, 0.19, 0.24],
        }
    }
}

/// Returns a Dunbar-inspired social layer for a given population size.
#[must_use]
pub fn dunbar_group_scale(population: u32) -> GroupScale {
    dunbar_group_scale_with_thresholds(population, DUNBAR_NUMBERS)
}

/// Returns social layer for custom threshold sets.
#[must_use]
pub fn dunbar_group_scale_with_thresholds(population: u32, thresholds: [u32; 6]) -> GroupScale {
    if population <= thresholds[0] {
        GroupScale::Intimate
    } else if population <= thresholds[1] {
        GroupScale::Sympathy
    } else if population <= thresholds[2] {
        GroupScale::Band
    } else if population <= thresholds[3] {
        GroupScale::Village
    } else if population <= thresholds[4] {
        GroupScale::Polity
    } else {
        GroupScale::Civilizational
    }
}

#[must_use]
pub fn dunbar_behavior(population: u32, model: DunbarBehaviorModel) -> DunbarLayerBehavior {
    let idx = match dunbar_group_scale_with_thresholds(population, model.thresholds) {
        GroupScale::Intimate => 0,
        GroupScale::Sympathy => 1,
        GroupScale::Band => 2,
        GroupScale::Village => 3,
        GroupScale::Polity => 4,
        GroupScale::Civilizational => 5,
    };
    DunbarLayerBehavior {
        expectation_load: model.expectation_load[idx],
        trust_decay: model.trust_decay[idx],
        communication_cost: model.communication_cost[idx],
        coordination_gain: model.coordination_gain[idx],
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

/// Abstract continental layout used to test isolation and diffusion regimes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContinentalLayout {
    Connected,
    Regional,
    Islands,
}

impl WorldMap {
    #[must_use]
    pub fn default_world() -> Self {
        Self::from_layout(ContinentalLayout::Regional, 0.35)
    }

    #[must_use]
    pub fn from_layout(layout: ContinentalLayout, isolation_factor: f64) -> Self {
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
        let base_corridors = match layout {
            ContinentalLayout::Connected => {
                let mut links = Vec::new();
                for from in 0..continents.len() {
                    for to in 0..continents.len() {
                        if from != to {
                            links.push(Corridor {
                                from,
                                to,
                                strength: 0.36,
                            });
                        }
                    }
                }
                links
            }
            ContinentalLayout::Regional => vec![
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
            ],
            ContinentalLayout::Islands => vec![
                Corridor {
                    from: 0,
                    to: 1,
                    strength: 0.10,
                },
                Corridor {
                    from: 1,
                    to: 0,
                    strength: 0.10,
                },
                Corridor {
                    from: 2,
                    to: 3,
                    strength: 0.08,
                },
                Corridor {
                    from: 3,
                    to: 2,
                    strength: 0.08,
                },
            ],
        };
        let isolation = isolation_factor.clamp(0.0, 1.0);
        let corridor_scale = (1.0 - isolation).clamp(0.0, 1.0);
        let corridors = base_corridors
            .into_iter()
            .map(|corridor| Corridor {
                strength: corridor.strength * corridor_scale,
                ..corridor
            })
            .collect::<Vec<Corridor>>();

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
    pub layout: ContinentalLayout,
    /// 0 = open diffusion, 1 = fully isolated corridors.
    pub isolation_factor: f64,
    pub dunbar_model: DunbarBehaviorModel,
    /// Min and max initial population per society.
    pub population_range: (u32, u32),
    /// Min and max initial complexity per society.
    pub initial_complexity_range: (f64, f64),
    /// Scales continent carrying_capacity and energy_endowment.
    pub resource_multiplier: f64,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            seed: 2026,
            generations: 320,
            initial_societies: 16,
            nk_n: 14,
            nk_k: 3,
            layout: ContinentalLayout::Regional,
            isolation_factor: 0.35,
            dunbar_model: DunbarBehaviorModel::default(),
            population_range: (12, 102),
            initial_complexity_range: (0.08, 0.28),
            resource_multiplier: 1.0,
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
    pub convergence_index: f64,
    pub adaptation_divergence: f64,
    /// Weighted superorganism signal from emergence_order_parameters.
    pub superorganism_index: f64,
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
    let mut map = WorldMap::from_layout(config.layout, config.isolation_factor);
    if (config.resource_multiplier - 1.0).abs() > 1e-9 {
        let m = config.resource_multiplier.clamp(0.1, 10.0);
        for continent in &mut map.continents {
            continent.carrying_capacity *= m;
            continent.energy_endowment = (continent.energy_endowment * m).clamp(0.0, 2.0);
        }
        for (i, state) in map.states.iter_mut().enumerate() {
            state.stock = map.continents[i].carrying_capacity;
        }
    }
    let landscape = NkLandscape::deterministic(config.nk_n, config.nk_k, config.seed ^ 0xa5a5);
    let mut societies = seed_societies(
        config.initial_societies,
        &map,
        &mut rng,
        config.nk_n,
        config.population_range,
        config.initial_complexity_range,
    );
    let mut snapshots = Vec::with_capacity(config.generations as usize);

    for generation in 0..config.generations {
        let continent_counts = per_continent_counts(&societies, map.continents.len());
        // Snapshot continent states so all societies act on the same world view.
        let state_snapshot: Vec<ContinentState> = map.states.clone();

        // Pre-compute per-continent actor messages deterministically.
        let continent_messages: Vec<Vec<ActorMessage>> = (0..map.continents.len())
            .map(|ci| {
                let mut msg_rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(ci as u64)
                    .wrapping_add(generation as u64)
                    .max(1);
                actor_messages_for(ci, &map, &mut msg_rng)
            })
            .collect();
        // Advance global rng to stay deterministic.
        rand01(&mut rng);

        // --- Pass 1 (parallel): compute society updates and resource extraction ---
        struct SocietyUpdate {
            extraction: f64,
            continent: usize,
            collapsed: bool,
        }

        let updates: Vec<SocietyUpdate> = societies
            .par_iter_mut()
            .map(|society| {
                let c_idx = society.continent;
                let continent = &map.continents[c_idx];
                let state = state_snapshot[c_idx];

                // Per-society deterministic RNG seeded from id + generation.
                let mut srng = society
                    .id
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(generation as u64)
                    .wrapping_add(config.seed)
                    .max(1);

                apply_actor_messages(society, &continent_messages[c_idx]);

                let nk_fit = landscape.fitness(society.genome.bits);
                let energy_access =
                    (continent.energy_endowment * state.stock * (1.0 - state.depletion)
                        + 0.35 * continent.domesticable_biomass
                        + 0.22 * continent.diffusion_access)
                        .clamp(0.0, 2.0);

                let layer = dunbar_behavior(society.population, config.dunbar_model);

                let innovation =
                    (0.48 * nk_fit + 0.27 * continent.diffusion_access + layer.coordination_gain)
                        .clamp(0.0, 1.4);
                let complexity_gain =
                    (0.20 * energy_access + 0.24 * innovation + 0.08 * society.trust)
                        .clamp(0.0, 1.0);
                let maintenance = (0.06
                    + 0.16 * society.complexity
                    + 0.10 * society.complexity.powi(2)
                    + layer.communication_cost
                    + 0.5 * layer.expectation_load
                    + 0.08 * state.depletion)
                    .clamp(0.0, 2.0);

                society.surplus =
                    (society.surplus + complexity_gain - maintenance).clamp(-1.0, 2.5);
                society.complexity = (society.complexity + 0.14 * complexity_gain
                    - 0.10 * maintenance)
                    .clamp(0.0, 1.8);

                let stress_shock = if rand01(&mut srng) < continent.shock_risk {
                    rand01(&mut srng) * 0.35
                } else {
                    0.0
                };
                society.resilience =
                    (society.resilience + 0.04 * innovation - 0.05 * stress_shock).clamp(0.05, 1.3);
                society.trust = (society.trust + 0.03 * innovation
                    - 0.04 * stress_shock
                    - layer.trust_decay
                    - 0.02 * layer.expectation_load)
                    .clamp(0.0, 1.0);

                let growth = (0.012 * society.surplus + 0.010 * society.resilience
                    - 0.008 * stress_shock)
                    .clamp(-0.08, 0.12);
                let next_population = ((society.population as f64) * (1.0 + growth)).round() as i64;
                society.population = next_population.max(4) as u32;

                let local_count = continent_counts[c_idx].max(1) as f64;
                let extraction = ((society.population as f64) / 140_000.0)
                    * (1.0 + 0.8 * society.complexity)
                    + 0.012 * society.surplus.max(0.0);

                let collapse_trigger = (state.stock < 0.12 * continent.carrying_capacity
                    || state.depletion > 0.88)
                    && society.complexity > 0.55;
                if collapse_trigger {
                    society.population =
                        ((society.population as f64) * 0.68).round().max(4.0) as u32;
                    society.complexity = (society.complexity * 0.72).clamp(0.0, 1.8);
                    society.surplus = (society.surplus - 0.22).clamp(-1.0, 2.5);
                    society.mode = SubsistenceMode::HunterGatherer;
                } else {
                    society.mode = mode_from_population(society.population, society.surplus);
                }

                society.genome = mutate_genome(society.genome, config.nk_n, &mut srng);

                SocietyUpdate {
                    extraction: extraction / local_count,
                    continent: c_idx,
                    collapsed: collapse_trigger,
                }
            })
            .collect();

        // --- Pass 2: aggregate resource changes per continent ---
        let collapse_events = updates.iter().filter(|u| u.collapsed).count() as u32;
        let mut extraction_per_continent = vec![0.0_f64; map.continents.len()];
        for update in &updates {
            extraction_per_continent[update.continent] += update.extraction;
        }
        for (ci, continent) in map.continents.iter().enumerate() {
            let state = &mut map.states[ci];
            let regen =
                continent.regen_rate * continent.carrying_capacity * (1.0 - 0.35 * state.depletion);
            let total_extraction = extraction_per_continent[ci];
            state.stock = (state.stock + regen - total_extraction)
                .clamp(0.0, continent.carrying_capacity * 1.1);
            state.depletion = (state.depletion + 0.08 * total_extraction
                - 0.45 * continent.regen_rate)
                .clamp(0.0, 1.0);
        }

        // Sexual selection: reproduction probability scales with mate fitness.
        // Uses per-society RNG for deterministic parallel reproduction decisions.
        let offspring: Vec<SocietyActor> = societies
            .par_iter()
            .filter_map(|society| {
                if society.population < 30 {
                    return None;
                }
                let mut srng = society
                    .id
                    .wrapping_mul(2862933555777941757)
                    .wrapping_add(generation as u64)
                    .wrapping_add(config.seed)
                    .wrapping_add(0xBEEF)
                    .max(1);
                let nk_fit = landscape.fitness(society.genome.bits);
                let mate_score = mate_fitness(society, nk_fit);
                let reproduction_prob = 0.04 * (1.0 + 3.0 * mate_score);
                if rand01(&mut srng) >= reproduction_prob {
                    return None;
                }
                let target = migrate_target(society.continent, &map.corridors, &mut srng)
                    .unwrap_or(society.continent);
                let child_id = society
                    .id
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(generation as u64)
                    .wrapping_add(1);
                let mut child = *society;
                child.id = child_id;
                child.continent = target;
                child.population = ((society.population as f64) * 0.16).round().max(5.0) as u32;
                child.complexity = (society.complexity * 0.80).clamp(0.0, 1.8);
                child.surplus = (society.surplus * 0.70).clamp(-1.0, 2.5);
                child.genome = mutate_genome(society.genome, config.nk_n, &mut srng);
                Some(child)
            })
            .collect();
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
        let continent_complexity_means =
            continent_means(&societies, map.continents.len(), |s| s.complexity);
        let continent_resilience_means =
            continent_means(&societies, map.continents.len(), |s| s.resilience);
        let convergence_index = convergence_index(&continent_complexity_means);
        let adaptation_divergence = standard_deviation(&continent_resilience_means);

        // Compute superorganism index by projecting societies to LocalSocietyState
        // and aggregating via the core emergence_order_parameters machinery.
        let local_states: Vec<LocalSocietyState> = societies
            .iter()
            .map(|s| {
                let st = map.states[s.continent];
                let coupling = map
                    .corridors
                    .iter()
                    .filter(|cor| cor.from == s.continent)
                    .map(|cor| cor.strength)
                    .sum::<f64>()
                    .clamp(0.0, 1.0);
                let eco_pressure = st.depletion;
                LocalSocietyState {
                    population: s.population,
                    mode: s.mode,
                    surplus_per_capita: s.surplus.max(0.0),
                    network_coupling: coupling,
                    ecological_pressure: eco_pressure.clamp(0.0, 1.0),
                }
            })
            .collect();
        let global_emergence = crate::aggregate_from_local_societies(&local_states);

        snapshots.push(EvolutionSnapshot {
            generation,
            population_total,
            mean_complexity,
            mean_energy_access,
            collapse_events,
            emergent_civilizations,
            convergence_index,
            adaptation_divergence,
            superorganism_index: global_emergence.superorganism_index,
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

fn standard_deviation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / (values.len() as f64);
    let var = values
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<f64>()
        / (values.len() as f64);
    var.sqrt()
}

fn convergence_index(values: &[f64]) -> f64 {
    let spread = standard_deviation(values);
    (1.0 / (1.0 + 4.0 * spread)).clamp(0.0, 1.0)
}

fn continent_means(
    societies: &[SocietyActor],
    continent_count: usize,
    getter: fn(&SocietyActor) -> f64,
) -> Vec<f64> {
    let mut sums = vec![0.0; continent_count];
    let mut counts = vec![0_usize; continent_count];
    for society in societies {
        if society.continent < continent_count {
            sums[society.continent] += getter(society);
            counts[society.continent] = counts[society.continent].saturating_add(1);
        }
    }
    let mut means = Vec::new();
    for idx in 0..continent_count {
        if counts[idx] > 0 {
            means.push(sums[idx] / (counts[idx] as f64));
        }
    }
    means
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

fn seed_societies(
    count: u32,
    map: &WorldMap,
    rng: &mut u64,
    nk_n: usize,
    population_range: (u32, u32),
    complexity_range: (f64, f64),
) -> Vec<SocietyActor> {
    let mut out = Vec::with_capacity(count as usize);
    let pop_min = population_range.0 as f64;
    let pop_span = (population_range.1 as f64 - pop_min).max(1.0);
    let cx_min = complexity_range.0;
    let cx_span = (complexity_range.1 - cx_min).max(0.01);
    for id in 0..count {
        let continent = ((rand01(rng) * (map.continents.len() as f64)).floor() as usize)
            .min(map.continents.len().saturating_sub(1));
        let pop = (pop_min + rand01(rng) * pop_span).round() as u32;
        out.push(SocietyActor {
            id: u64::from(id),
            continent,
            mode: SubsistenceMode::HunterGatherer,
            population: pop,
            complexity: (cx_min + rand01(rng) * cx_span).clamp(0.0, 1.8),
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

/// Composite mate-fitness score for sexual selection.
/// Combines NK landscape fitness with observable society traits.
fn mate_fitness(society: &SocietyActor, nk_fitness: f64) -> f64 {
    (0.35 * nk_fitness
        + 0.25 * (society.surplus.max(0.0) / 2.5).clamp(0.0, 1.0)
        + 0.20 * society.trust
        + 0.20 * (society.complexity / 1.8).clamp(0.0, 1.0))
    .clamp(0.0, 1.0)
}

fn bit(bits: u64, idx: usize) -> u32 {
    ((bits >> idx) & 1) as u32
}

fn rand01(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*state as f64) / (u64::MAX as f64)
}

// ---------------------------------------------------------------------------
// Convergence experiment: systematic multi-run hypothesis testing
// ---------------------------------------------------------------------------

/// Describes one set of starting conditions for a convergence experiment.
#[derive(Clone, Debug, PartialEq)]
pub struct StartingConditions {
    pub label: String,
    pub layout: ContinentalLayout,
    pub isolation_factor: f64,
    pub initial_societies: u32,
    pub population_range: (u32, u32),
    pub initial_complexity_range: (f64, f64),
    pub resource_multiplier: f64,
}

/// Configuration for a convergence experiment across many starting conditions.
#[derive(Clone, Debug, PartialEq)]
pub struct ConvergenceExperimentConfig {
    pub conditions: Vec<StartingConditions>,
    pub seeds_per_condition: u32,
    pub generations: u32,
    /// Superorganism index must exceed this to count as "arrived".
    pub superorganism_threshold: f64,
    /// Must sustain above threshold for this many consecutive generations.
    pub sustained_generations: u32,
    pub nk_n: usize,
    pub nk_k: usize,
    pub dunbar_model: DunbarBehaviorModel,
}

/// Per-run outcome for one seed under one condition.
#[derive(Clone, Debug, PartialEq)]
pub struct RunOutcome {
    pub condition_index: usize,
    pub seed: u64,
    pub reached_superorganism: bool,
    pub time_to_superorganism: Option<u32>,
    pub peak_superorganism_index: f64,
    pub final_superorganism_index: f64,
    pub final_population: u64,
    pub total_collapses: u32,
    pub final_mean_complexity: f64,
}

/// Aggregate statistics for one starting condition.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionSummary {
    pub label: String,
    pub runs: u32,
    pub arrival_rate: f64,
    pub median_time_to_superorganism: Option<u32>,
    pub mean_peak_superorganism: f64,
    pub mean_final_superorganism: f64,
    pub mean_final_complexity: f64,
    pub mean_collapses: f64,
}

/// Full experiment result with per-run outcomes and aggregate summaries.
#[derive(Clone, Debug, PartialEq)]
pub struct ConvergenceResult {
    pub outcomes: Vec<RunOutcome>,
    pub condition_summaries: Vec<ConditionSummary>,
    pub overall_arrival_rate: f64,
}

/// Runs a convergence experiment: many seeds x many starting conditions.
/// Returns per-run outcomes and aggregate statistics to test whether
/// superorganism emergence is an attractor under varied initial conditions.
#[must_use]
pub fn run_convergence_experiment(cfg: &ConvergenceExperimentConfig) -> ConvergenceResult {
    // Build work items: (condition_index, seed, EvolutionConfig).
    let work_items: Vec<(usize, u64, EvolutionConfig)> = cfg
        .conditions
        .iter()
        .enumerate()
        .flat_map(|(ci, cond)| {
            (0..cfg.seeds_per_condition).map(move |seed_idx| {
                let seed = (seed_idx as u64)
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(ci as u64)
                    .wrapping_add(1)
                    .max(1);
                (
                    ci,
                    seed,
                    EvolutionConfig {
                        seed,
                        generations: cfg.generations,
                        initial_societies: cond.initial_societies,
                        nk_n: cfg.nk_n,
                        nk_k: cfg.nk_k,
                        layout: cond.layout,
                        isolation_factor: cond.isolation_factor,
                        dunbar_model: cfg.dunbar_model,
                        population_range: cond.population_range,
                        initial_complexity_range: cond.initial_complexity_range,
                        resource_multiplier: cond.resource_multiplier,
                    },
                )
            })
        })
        .collect();

    // Run all simulations in parallel across available cores.
    let threshold = cfg.superorganism_threshold;
    let sustained_gens = cfg.sustained_generations;
    let outcomes: Vec<RunOutcome> = work_items
        .par_iter()
        .map(|(ci, seed, evo_cfg)| {
            let result = simulate_evolution(*evo_cfg);

            let peak_so = result
                .snapshots
                .iter()
                .map(|s| s.superorganism_index)
                .fold(0.0_f64, f64::max);
            let final_snap = result.snapshots.last().copied();
            let final_so = final_snap.map_or(0.0, |s| s.superorganism_index);
            let final_pop = final_snap.map_or(0, |s| s.population_total);
            let final_cx = final_snap.map_or(0.0, |s| s.mean_complexity);
            let total_collapses: u32 = result.snapshots.iter().map(|s| s.collapse_events).sum();

            let mut consecutive = 0_u32;
            let mut first_sustained: Option<u32> = None;
            for snap in &result.snapshots {
                if snap.superorganism_index >= threshold {
                    consecutive = consecutive.saturating_add(1);
                    if consecutive >= sustained_gens && first_sustained.is_none() {
                        first_sustained = Some(snap.generation.saturating_sub(sustained_gens - 1));
                    }
                } else {
                    consecutive = 0;
                }
            }

            RunOutcome {
                condition_index: *ci,
                seed: *seed,
                reached_superorganism: first_sustained.is_some(),
                time_to_superorganism: first_sustained,
                peak_superorganism_index: peak_so,
                final_superorganism_index: final_so,
                final_population: final_pop,
                total_collapses,
                final_mean_complexity: final_cx,
            }
        })
        .collect();

    let mut condition_summaries = Vec::with_capacity(cfg.conditions.len());
    for (ci, cond) in cfg.conditions.iter().enumerate() {
        let runs: Vec<&RunOutcome> = outcomes
            .iter()
            .filter(|o| o.condition_index == ci)
            .collect();
        let n = runs.len() as f64;
        let arrived = runs.iter().filter(|o| o.reached_superorganism).count();
        let arrival_rate = if n > 0.0 { (arrived as f64) / n } else { 0.0 };

        let mut times: Vec<u32> = runs
            .iter()
            .filter_map(|o| o.time_to_superorganism)
            .collect();
        times.sort_unstable();
        let median_time = if times.is_empty() {
            None
        } else {
            Some(times[times.len() / 2])
        };

        let mean_peak = runs.iter().map(|o| o.peak_superorganism_index).sum::<f64>() / n.max(1.0);
        let mean_final = runs
            .iter()
            .map(|o| o.final_superorganism_index)
            .sum::<f64>()
            / n.max(1.0);
        let mean_cx = runs.iter().map(|o| o.final_mean_complexity).sum::<f64>() / n.max(1.0);
        let mean_col = runs
            .iter()
            .map(|o| f64::from(o.total_collapses))
            .sum::<f64>()
            / n.max(1.0);

        condition_summaries.push(ConditionSummary {
            label: cond.label.clone(),
            runs: runs.len() as u32,
            arrival_rate,
            median_time_to_superorganism: median_time,
            mean_peak_superorganism: mean_peak,
            mean_final_superorganism: mean_final,
            mean_final_complexity: mean_cx,
            mean_collapses: mean_col,
        });
    }

    let total_runs = outcomes.len();
    let total_arrived = outcomes.iter().filter(|o| o.reached_superorganism).count();
    let overall_arrival_rate = if total_runs > 0 {
        (total_arrived as f64) / (total_runs as f64)
    } else {
        0.0
    };

    ConvergenceResult {
        outcomes,
        condition_summaries,
        overall_arrival_rate,
    }
}

/// Returns a default set of starting conditions spanning the hypothesis space.
#[must_use]
pub fn default_experiment_conditions() -> Vec<StartingConditions> {
    vec![
        StartingConditions {
            label: "abundant-connected".into(),
            layout: ContinentalLayout::Connected,
            isolation_factor: 0.1,
            initial_societies: 20,
            population_range: (30, 150),
            initial_complexity_range: (0.10, 0.30),
            resource_multiplier: 1.5,
        },
        StartingConditions {
            label: "abundant-isolated".into(),
            layout: ContinentalLayout::Islands,
            isolation_factor: 0.8,
            initial_societies: 20,
            population_range: (30, 150),
            initial_complexity_range: (0.10, 0.30),
            resource_multiplier: 1.5,
        },
        StartingConditions {
            label: "scarce-connected".into(),
            layout: ContinentalLayout::Connected,
            isolation_factor: 0.1,
            initial_societies: 20,
            population_range: (8, 60),
            initial_complexity_range: (0.05, 0.15),
            resource_multiplier: 0.6,
        },
        StartingConditions {
            label: "scarce-isolated".into(),
            layout: ContinentalLayout::Islands,
            isolation_factor: 0.8,
            initial_societies: 20,
            population_range: (8, 60),
            initial_complexity_range: (0.05, 0.15),
            resource_multiplier: 0.6,
        },
        StartingConditions {
            label: "baseline-regional".into(),
            layout: ContinentalLayout::Regional,
            isolation_factor: 0.35,
            initial_societies: 16,
            population_range: (12, 102),
            initial_complexity_range: (0.08, 0.28),
            resource_multiplier: 1.0,
        },
        StartingConditions {
            label: "large-groups-regional".into(),
            layout: ContinentalLayout::Regional,
            isolation_factor: 0.35,
            initial_societies: 8,
            population_range: (80, 500),
            initial_complexity_range: (0.15, 0.40),
            resource_multiplier: 1.2,
        },
        StartingConditions {
            label: "many-small-connected".into(),
            layout: ContinentalLayout::Connected,
            isolation_factor: 0.15,
            initial_societies: 40,
            population_range: (5, 30),
            initial_complexity_range: (0.03, 0.12),
            resource_multiplier: 1.0,
        },
        StartingConditions {
            label: "rich-few-isolated".into(),
            layout: ContinentalLayout::Islands,
            isolation_factor: 0.9,
            initial_societies: 6,
            population_range: (100, 400),
            initial_complexity_range: (0.20, 0.50),
            resource_multiplier: 2.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        apply_actor_messages, default_experiment_conditions, dunbar_behavior, dunbar_group_scale,
        dunbar_group_scale_with_thresholds, run_convergence_experiment, simulate_evolution,
        ActorMessage, ContinentalLayout, ConvergenceExperimentConfig, DunbarBehaviorModel,
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
    fn custom_dunbar_thresholds_shift_scale_boundaries() {
        let custom = [8, 20, 70, 210, 700, 2_000];
        assert_eq!(
            dunbar_group_scale_with_thresholds(150, custom),
            GroupScale::Village
        );
        assert_eq!(
            dunbar_group_scale_with_thresholds(650, custom),
            GroupScale::Polity
        );
    }

    #[test]
    fn dunbar_behavior_increases_coordination_and_costs_with_scale() {
        let model = DunbarBehaviorModel::default();
        let small = dunbar_behavior(20, model);
        let large = dunbar_behavior(1_800, model);
        assert!(large.coordination_gain > small.coordination_gain);
        assert!(large.communication_cost > small.communication_cost);
        assert!(large.trust_decay > small.trust_decay);
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
    fn isolation_factor_reduces_total_corridor_strength() {
        let open = WorldMap::from_layout(ContinentalLayout::Regional, 0.0);
        let isolated = WorldMap::from_layout(ContinentalLayout::Regional, 0.9);
        let open_strength = open.corridors.iter().map(|c| c.strength).sum::<f64>();
        let isolated_strength = isolated.corridors.iter().map(|c| c.strength).sum::<f64>();
        assert!(isolated_strength < open_strength);
    }

    #[test]
    fn evolution_run_shows_both_emergence_and_collapse_events() {
        let result = simulate_evolution(EvolutionConfig {
            seed: 99,
            generations: 300,
            initial_societies: 20,
            nk_n: 12,
            nk_k: 3,
            layout: ContinentalLayout::Connected,
            isolation_factor: 0.1,
            resource_multiplier: 0.6,
            population_range: (8, 60),
            initial_complexity_range: (0.05, 0.15),
            ..EvolutionConfig::default()
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
        assert!(result.snapshots[0].convergence_index >= 0.0);
        // Superorganism index should be computed and bounded.
        for snap in &result.snapshots {
            assert!((0.0..=1.0).contains(&snap.superorganism_index));
        }
    }

    #[test]
    fn sexual_selection_biases_reproduction_toward_fit_societies() {
        // Run two configs: one with very high resources (should produce higher
        // superorganism signal) vs very low.
        let rich = simulate_evolution(EvolutionConfig {
            seed: 42,
            generations: 200,
            resource_multiplier: 2.0,
            ..EvolutionConfig::default()
        });
        let poor = simulate_evolution(EvolutionConfig {
            seed: 42,
            generations: 200,
            resource_multiplier: 0.4,
            ..EvolutionConfig::default()
        });
        let rich_peak = rich
            .snapshots
            .iter()
            .map(|s| s.superorganism_index)
            .fold(0.0_f64, f64::max);
        let poor_peak = poor
            .snapshots
            .iter()
            .map(|s| s.superorganism_index)
            .fold(0.0_f64, f64::max);
        // Rich environments should produce higher superorganism signals.
        assert!(rich_peak > poor_peak);
    }

    #[test]
    fn convergence_experiment_produces_valid_summaries() {
        let result = run_convergence_experiment(&ConvergenceExperimentConfig {
            conditions: default_experiment_conditions(),
            seeds_per_condition: 3,
            generations: 100,
            superorganism_threshold: 0.35,
            sustained_generations: 5,
            nk_n: 10,
            nk_k: 2,
            dunbar_model: DunbarBehaviorModel::default(),
        });

        assert_eq!(
            result.condition_summaries.len(),
            default_experiment_conditions().len()
        );
        assert!(!result.outcomes.is_empty());
        assert!((0.0..=1.0).contains(&result.overall_arrival_rate));

        for summary in &result.condition_summaries {
            assert!((0.0..=1.0).contains(&summary.arrival_rate));
            assert!(summary.mean_peak_superorganism >= 0.0);
            assert!(summary.mean_final_complexity >= 0.0);
        }
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
