//! Deterministic, explicit stock-flow simulator core.

/// Historical subsistence regimes used for scenario transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubsistenceMode {
    HunterGatherer,
    Sedentary,
    Agriculture,
}

/// Aggregated social behavior profile induced by group size and regime.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BehaviorProfile {
    pub coordination_cost: f64,
    pub hierarchy_pressure: f64,
    pub coercion_propensity: f64,
    pub cohesion: f64,
}

/// Emergent social dynamics derived from regime and scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergentDynamics {
    pub storage_dependence: f64,
    pub labor_specialization: f64,
    pub property_lock_in: f64,
    pub institutional_centralization: f64,
}

/// Macro-level order parameters used to detect superorganism emergence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergenceOrderParameters {
    pub throughput_pressure: f64,
    pub coordination_centralization: f64,
    pub policy_lock_in: f64,
    pub autonomy_loss: f64,
    pub superorganism_index: f64,
}

/// Local society state used for multi-society emergence simulations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalSocietyState {
    pub population: u32,
    pub mode: SubsistenceMode,
    pub surplus_per_capita: f64,
    pub network_coupling: f64,
    pub ecological_pressure: f64,
}

/// Complexity signature for a local society.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalComplexity {
    pub hierarchy: f64,
    pub specialization: f64,
    pub lock_in: f64,
    pub centralization: f64,
    pub complexity_index: f64,
}

/// Thresholds for regime transitions with simple hysteresis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionConfig {
    pub sedentarism_population_threshold: u32,
    pub sedentarism_surplus_threshold: f64,
    pub agriculture_population_threshold: u32,
    pub agriculture_surplus_threshold: f64,
    pub regression_ecological_pressure_threshold: f64,
    pub regression_surplus_threshold: f64,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            sedentarism_population_threshold: 120,
            sedentarism_surplus_threshold: 0.25,
            agriculture_population_threshold: 800,
            agriculture_surplus_threshold: 0.45,
            regression_ecological_pressure_threshold: 0.85,
            regression_surplus_threshold: 0.20,
        }
    }
}

/// Snapshot of emergence state at one simulation tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergenceSnapshot {
    pub tick: u64,
    pub global: EmergenceOrderParameters,
    pub mean_local_complexity: f64,
    pub hunter_gatherer_count: usize,
    pub sedentary_count: usize,
    pub agriculture_count: usize,
}

/// Minimal agent state used by the MVP model.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    /// Monetary/resource proxy held by the agent.
    pub wealth: f64,
    /// Baseline needs pressure.
    pub need: f64,
    /// Relative weight for status-seeking behavior.
    pub status_drive: f64,
    /// Current local social group size.
    pub group_size: u32,
    /// Current subsistence regime.
    pub subsistence_mode: SubsistenceMode,
}

/// Global world state updated every simulation tick.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldState {
    pub tick: u64,
    pub renewable_stock: f64,
    pub nonrenewable_stock: f64,
    pub aggregate_output: f64,
}

/// Runtime configuration for deterministic simulation runs.
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationConfig {
    pub seed: u64,
    pub regen_rate: f64,
    pub extraction_rate: f64,
}

/// Deterministic simulation engine with explicit resource constraints.
#[derive(Clone, Debug)]
pub struct SimulationEngine {
    cfg: SimulationConfig,
    state: WorldState,
    #[allow(dead_code)]
    agents: Vec<AgentState>,
    rng_state: u64,
}

impl SimulationEngine {
    #[must_use]
    pub fn new(cfg: SimulationConfig, agents: Vec<AgentState>, state: WorldState) -> Self {
        Self {
            rng_state: cfg.seed,
            cfg,
            state,
            agents,
        }
    }

    #[must_use]
    pub fn state(&self) -> &WorldState {
        &self.state
    }

    /// Advances the world by one tick.
    pub fn step(&mut self) -> &WorldState {
        let extracted = self
            .state
            .nonrenewable_stock
            .min(self.cfg.extraction_rate.max(0.0));
        let renewable_gain = self.cfg.regen_rate.max(0.0) * self.state.renewable_stock;

        // Tiny bounded perturbation keeps runs stochastic while deterministic by seed.
        let shock = self.next_shock();
        let output = (extracted + renewable_gain + shock).max(0.0);

        self.state.nonrenewable_stock -= extracted;
        self.state.renewable_stock =
            (self.state.renewable_stock + renewable_gain - output * 0.1).max(0.0);
        self.state.aggregate_output = output;
        self.state.tick = self.state.tick.saturating_add(1);
        &self.state
    }

    /// Runs for a number of ticks, returning pre-step snapshots.
    pub fn run(&mut self, ticks: u64) -> Vec<WorldState> {
        let mut history = Vec::with_capacity(ticks as usize);
        for _ in 0..ticks {
            history.push(self.state.clone());
            self.step();
        }
        history
    }

    fn next_shock(&mut self) -> f64 {
        // Simple deterministic LCG; replace with stream-splittable RNG in later phases.
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);

        let unit = (self.rng_state as f64) / (u64::MAX as f64);
        (unit * 0.02) - 0.01
    }
}

/// Computes regime-conditioned social behavior for a group of size `group_size`.
#[must_use]
pub fn group_behavior_profile(group_size: u32, mode: SubsistenceMode) -> BehaviorProfile {
    let n = f64::from(group_size.max(1));
    let log_n = (1.0 + n).ln();

    let (alpha, beta, gamma, delta, eta, n0) = match mode {
        SubsistenceMode::HunterGatherer => (0.8, 0.5, 0.2, 1.2, 0.004, 80.0),
        SubsistenceMode::Sedentary => (1.0, 0.8, 0.45, 1.0, 0.006, 120.0),
        SubsistenceMode::Agriculture => (1.2, 1.1, 0.8, 0.85, 0.008, 150.0),
    };

    let coordination_cost = alpha * log_n;
    let hierarchy_pressure = beta * log_n;
    let coercion_propensity = gamma * ((n - n0).max(0.0) / n);
    let cohesion = delta / (1.0 + eta * n);

    BehaviorProfile {
        coordination_cost,
        hierarchy_pressure,
        coercion_propensity,
        cohesion,
    }
}

/// Computes expected emergent dynamics from group size, regime, and available surplus.
///
/// `surplus_per_capita` is a normalized proxy in `[0, +inf)` where larger values indicate
/// greater storage and deferred consumption capacity.
#[must_use]
pub fn emergent_dynamics(
    group_size: u32,
    mode: SubsistenceMode,
    surplus_per_capita: f64,
) -> EmergentDynamics {
    let n = f64::from(group_size.max(1));
    let log_scale = n.ln_1p();
    let surplus = surplus_per_capita.max(0.0);

    let (storage_base, specialization_base, property_base, centralization_base) = match mode {
        SubsistenceMode::HunterGatherer => (0.10, 0.15, 0.10, 0.12),
        SubsistenceMode::Sedentary => (0.40, 0.45, 0.40, 0.45),
        SubsistenceMode::Agriculture => (0.65, 0.70, 0.75, 0.70),
    };

    let storage_dependence = clamp01(storage_base + 0.08 * log_scale + 0.20 * surplus);
    let labor_specialization = clamp01(specialization_base + 0.10 * log_scale + 0.20 * surplus);
    let property_lock_in = clamp01(property_base + 0.06 * log_scale + 0.15 * surplus);
    let institutional_centralization =
        clamp01(centralization_base + 0.12 * log_scale + 0.10 * surplus);

    EmergentDynamics {
        storage_dependence,
        labor_specialization,
        property_lock_in,
        institutional_centralization,
    }
}

/// Computes macro emergence signals from micro/meso state proxies.
///
/// `network_coupling` and `ecological_pressure` are normalized to `[0, 1]`.
#[must_use]
pub fn emergence_order_parameters(
    group_size: u32,
    mode: SubsistenceMode,
    surplus_per_capita: f64,
    network_coupling: f64,
    ecological_pressure: f64,
) -> EmergenceOrderParameters {
    let behavior = group_behavior_profile(group_size, mode);
    let dynamics = emergent_dynamics(group_size, mode, surplus_per_capita);
    let coupling = clamp01(network_coupling);
    let eco_pressure = clamp01(ecological_pressure);

    // R1: surplus/storage -> centralization -> throughput pressure.
    let throughput_pressure = clamp01(
        0.35 * dynamics.storage_dependence + 0.35 * dynamics.property_lock_in + 0.30 * coupling,
    );

    // R2: scale/coordination cost -> hierarchy delegation -> centralization.
    let coordination_centralization = clamp01(
        0.25 * behavior.coordination_cost / 10.0
            + 0.40 * behavior.hierarchy_pressure / 10.0
            + 0.35 * dynamics.institutional_centralization,
    );

    // R3: centralization + property lock-in -> policy lock-in.
    let policy_lock_in = clamp01(
        0.50 * dynamics.property_lock_in + 0.35 * coordination_centralization + 0.15 * coupling,
    );

    // Autonomy loss rises with lock-in and coercion; cohesion partially offsets it.
    let autonomy_loss = clamp01(
        0.45 * policy_lock_in
            + 0.35 * behavior.coercion_propensity
            + 0.20 * coordination_centralization
            - 0.20 * behavior.cohesion,
    );

    // Balancing loop: ecological pressure constrains aggregate superorganism coherence.
    let raw_index = 0.30 * throughput_pressure
        + 0.30 * coordination_centralization
        + 0.20 * policy_lock_in
        + 0.20 * autonomy_loss;
    let superorganism_index = clamp01(raw_index * (1.0 - 0.35 * eco_pressure));

    EmergenceOrderParameters {
        throughput_pressure,
        coordination_centralization,
        policy_lock_in,
        autonomy_loss,
        superorganism_index,
    }
}

/// Computes local complexity emergence for one society.
#[must_use]
pub fn local_complexity(state: LocalSocietyState) -> LocalComplexity {
    let behavior = group_behavior_profile(state.population, state.mode);
    let dynamics = emergent_dynamics(state.population, state.mode, state.surplus_per_capita);

    let hierarchy =
        clamp01(0.70 * behavior.hierarchy_pressure / 10.0 + 0.30 * behavior.coercion_propensity);
    let specialization = dynamics.labor_specialization;
    let lock_in = dynamics.property_lock_in;
    let centralization = dynamics.institutional_centralization;

    let complexity_index = clamp01(
        0.30 * hierarchy + 0.25 * specialization + 0.20 * lock_in + 0.25 * centralization
            - 0.20 * clamp01(state.ecological_pressure),
    );

    LocalComplexity {
        hierarchy,
        specialization,
        lock_in,
        centralization,
        complexity_index,
    }
}

/// Aggregates many local societies into a global superorganism signal.
#[must_use]
pub fn aggregate_from_local_societies(societies: &[LocalSocietyState]) -> EmergenceOrderParameters {
    if societies.is_empty() {
        return EmergenceOrderParameters {
            throughput_pressure: 0.0,
            coordination_centralization: 0.0,
            policy_lock_in: 0.0,
            autonomy_loss: 0.0,
            superorganism_index: 0.0,
        };
    }

    let total_pop: f64 = societies.iter().map(|s| f64::from(s.population)).sum();
    let denom = total_pop.max(1.0);

    let mut throughput_pressure = 0.0;
    let mut coordination_centralization = 0.0;
    let mut policy_lock_in = 0.0;
    let mut autonomy_loss = 0.0;
    let mut superorganism_index = 0.0;

    for state in societies {
        let weight = f64::from(state.population) / denom;
        let local = emergence_order_parameters(
            state.population,
            state.mode,
            state.surplus_per_capita,
            state.network_coupling,
            state.ecological_pressure,
        );

        throughput_pressure += weight * local.throughput_pressure;
        coordination_centralization += weight * local.coordination_centralization;
        policy_lock_in += weight * local.policy_lock_in;
        autonomy_loss += weight * local.autonomy_loss;
        superorganism_index += weight * local.superorganism_index;
    }

    EmergenceOrderParameters {
        throughput_pressure,
        coordination_centralization,
        policy_lock_in,
        autonomy_loss,
        superorganism_index,
    }
}

/// Determines next subsistence mode based on scale, surplus, and ecological stress.
#[must_use]
pub fn next_subsistence_mode(
    current_mode: SubsistenceMode,
    population: u32,
    surplus_per_capita: f64,
    ecological_pressure: f64,
    cfg: TransitionConfig,
) -> SubsistenceMode {
    let pop = population;
    let surplus = surplus_per_capita.max(0.0);
    let eco = clamp01(ecological_pressure);

    match current_mode {
        SubsistenceMode::HunterGatherer => {
            if pop >= cfg.sedentarism_population_threshold
                && surplus >= cfg.sedentarism_surplus_threshold
            {
                SubsistenceMode::Sedentary
            } else {
                SubsistenceMode::HunterGatherer
            }
        }
        SubsistenceMode::Sedentary => {
            if pop >= cfg.agriculture_population_threshold
                && surplus >= cfg.agriculture_surplus_threshold
            {
                SubsistenceMode::Agriculture
            } else if eco >= cfg.regression_ecological_pressure_threshold
                && surplus < cfg.regression_surplus_threshold * 0.6
            {
                SubsistenceMode::HunterGatherer
            } else {
                SubsistenceMode::Sedentary
            }
        }
        SubsistenceMode::Agriculture => {
            if eco >= cfg.regression_ecological_pressure_threshold
                && surplus < cfg.regression_surplus_threshold
            {
                SubsistenceMode::Sedentary
            } else {
                SubsistenceMode::Agriculture
            }
        }
    }
}

/// One deterministic time step for a local society with global feedback signals.
#[must_use]
pub fn step_local_society(
    state: LocalSocietyState,
    global: EmergenceOrderParameters,
    cfg: TransitionConfig,
) -> LocalSocietyState {
    let mode_growth = match state.mode {
        SubsistenceMode::HunterGatherer => 0.003,
        SubsistenceMode::Sedentary => 0.008,
        SubsistenceMode::Agriculture => 0.012,
    };
    let growth_feedback = 0.004 * global.throughput_pressure * clamp01(state.network_coupling);
    let ecological_penalty = 0.02 * clamp01(state.ecological_pressure);
    let growth_rate = mode_growth + growth_feedback - ecological_penalty;
    let grown_pop = f64::from(state.population) * (1.0 + growth_rate);
    let next_population = grown_pop.max(1.0).round() as u32;

    let productivity_bonus = match state.mode {
        SubsistenceMode::HunterGatherer => 0.02,
        SubsistenceMode::Sedentary => 0.05,
        SubsistenceMode::Agriculture => 0.08,
    };
    let next_surplus = (state.surplus_per_capita
        + productivity_bonus
        + 0.10 * global.throughput_pressure * clamp01(state.network_coupling)
        - 0.16 * clamp01(state.ecological_pressure))
    .clamp(0.0, 2.0);

    let mode_coupling_bonus = match state.mode {
        SubsistenceMode::HunterGatherer => -0.01,
        SubsistenceMode::Sedentary => 0.0,
        SubsistenceMode::Agriculture => 0.01,
    };
    let next_coupling =
        (state.network_coupling + mode_coupling_bonus + 0.06 * global.coordination_centralization
            - 0.03 * clamp01(state.ecological_pressure))
        .clamp(0.0, 1.0);

    let restoration_factor = match state.mode {
        SubsistenceMode::HunterGatherer => 0.06,
        SubsistenceMode::Sedentary => 0.03,
        SubsistenceMode::Agriculture => 0.01,
    };
    let next_ecological_pressure =
        (state.ecological_pressure + 0.05 * global.throughput_pressure + 0.04 * next_surplus
            - restoration_factor)
            .clamp(0.0, 1.0);

    let next_mode = next_subsistence_mode(
        state.mode,
        next_population,
        next_surplus,
        next_ecological_pressure,
        cfg,
    );

    LocalSocietyState {
        population: next_population,
        mode: next_mode,
        surplus_per_capita: next_surplus,
        network_coupling: next_coupling,
        ecological_pressure: next_ecological_pressure,
    }
}

/// Runs a local-to-global emergence simulation for `ticks` iterations.
#[must_use]
pub fn run_emergence_simulation(
    mut societies: Vec<LocalSocietyState>,
    ticks: u64,
    cfg: TransitionConfig,
) -> Vec<EmergenceSnapshot> {
    let mut snapshots = Vec::with_capacity(ticks as usize);

    for tick in 0..ticks {
        let global = aggregate_from_local_societies(&societies);
        let mean_local_complexity = if societies.is_empty() {
            0.0
        } else {
            let sum: f64 = societies
                .iter()
                .map(|s| local_complexity(*s).complexity_index)
                .sum();
            sum / (societies.len() as f64)
        };

        let hunter_gatherer_count = societies
            .iter()
            .filter(|s| s.mode == SubsistenceMode::HunterGatherer)
            .count();
        let sedentary_count = societies
            .iter()
            .filter(|s| s.mode == SubsistenceMode::Sedentary)
            .count();
        let agriculture_count = societies
            .iter()
            .filter(|s| s.mode == SubsistenceMode::Agriculture)
            .count();

        snapshots.push(EmergenceSnapshot {
            tick,
            global,
            mean_local_complexity,
            hunter_gatherer_count,
            sedentary_count,
            agriculture_count,
        });

        for society in &mut societies {
            *society = step_local_society(*society, global, cfg);
        }
    }

    snapshots
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        aggregate_from_local_societies, emergence_order_parameters, emergent_dynamics,
        group_behavior_profile, local_complexity, next_subsistence_mode, run_emergence_simulation,
        step_local_society, AgentState, EmergenceOrderParameters, LocalSocietyState,
        SimulationConfig, SimulationEngine, SubsistenceMode, TransitionConfig, WorldState,
    };

    fn build_engine(seed: u64) -> SimulationEngine {
        SimulationEngine::new(
            SimulationConfig {
                seed,
                regen_rate: 0.02,
                extraction_rate: 1.0,
            },
            vec![
                AgentState {
                    wealth: 1.0,
                    need: 1.0,
                    status_drive: 0.5,
                    group_size: 50,
                    subsistence_mode: SubsistenceMode::HunterGatherer,
                };
                5
            ],
            WorldState {
                tick: 0,
                renewable_stock: 100.0,
                nonrenewable_stock: 100.0,
                aggregate_output: 0.0,
            },
        )
    }

    #[test]
    fn deterministic_for_same_seed() {
        let mut lhs = build_engine(7);
        let mut rhs = build_engine(7);

        let lhs_values: Vec<f64> = lhs
            .run(10)
            .into_iter()
            .map(|s| s.aggregate_output)
            .collect();
        let rhs_values: Vec<f64> = rhs
            .run(10)
            .into_iter()
            .map(|s| s.aggregate_output)
            .collect();

        assert_eq!(lhs_values, rhs_values);
    }

    #[test]
    fn nonrenewable_never_negative() {
        let mut engine = build_engine(42);
        for _ in 0..500 {
            let _ = engine.step();
        }
        assert!(engine.state().nonrenewable_stock >= 0.0);
    }

    #[test]
    fn run_returns_requested_length() {
        let mut engine = build_engine(1);
        let history = engine.run(12);
        assert_eq!(history.len(), 12);
    }

    #[test]
    fn hierarchy_pressure_increases_with_group_size() {
        let small = group_behavior_profile(30, SubsistenceMode::Sedentary);
        let large = group_behavior_profile(3_000, SubsistenceMode::Sedentary);
        assert!(large.hierarchy_pressure > small.hierarchy_pressure);
    }

    #[test]
    fn agriculture_has_higher_hierarchy_than_hunter_gatherer_at_same_size() {
        let n = 500;
        let hg = group_behavior_profile(n, SubsistenceMode::HunterGatherer);
        let ag = group_behavior_profile(n, SubsistenceMode::Agriculture);
        assert!(ag.hierarchy_pressure > hg.hierarchy_pressure);
        assert!(ag.coercion_propensity > hg.coercion_propensity);
    }

    #[test]
    fn sedentarism_increases_storage_and_property_lock_in() {
        let n = 150;
        let surplus = 0.5;

        let hunter = emergent_dynamics(n, SubsistenceMode::HunterGatherer, surplus);
        let sedentary = emergent_dynamics(n, SubsistenceMode::Sedentary, surplus);

        assert!(sedentary.storage_dependence > hunter.storage_dependence);
        assert!(sedentary.property_lock_in > hunter.property_lock_in);
        assert!(sedentary.institutional_centralization > hunter.institutional_centralization);
    }

    #[test]
    fn agriculture_pushes_further_than_sedentary() {
        let n = 5;
        let surplus = 0.1;

        let sedentary = emergent_dynamics(n, SubsistenceMode::Sedentary, surplus);
        let agricultural = emergent_dynamics(n, SubsistenceMode::Agriculture, surplus);

        assert!(agricultural.labor_specialization > sedentary.labor_specialization);
        assert!(agricultural.property_lock_in > sedentary.property_lock_in);
        assert!(agricultural.institutional_centralization > sedentary.institutional_centralization);
    }

    #[test]
    fn superorganism_signal_increases_with_group_size_and_mode() {
        let small_hg =
            emergence_order_parameters(30, SubsistenceMode::HunterGatherer, 0.1, 0.2, 0.1);
        let large_ag =
            emergence_order_parameters(3_000, SubsistenceMode::Agriculture, 0.5, 0.8, 0.1);
        assert!(large_ag.superorganism_index > small_hg.superorganism_index);
    }

    #[test]
    fn ecological_pressure_is_a_balancing_loop() {
        let low_pressure =
            emergence_order_parameters(1_000, SubsistenceMode::Agriculture, 0.5, 0.8, 0.1);
        let high_pressure =
            emergence_order_parameters(1_000, SubsistenceMode::Agriculture, 0.5, 0.8, 0.9);
        assert!(high_pressure.superorganism_index < low_pressure.superorganism_index);
    }

    #[test]
    fn local_complexity_rises_from_hunter_to_sedentary() {
        let n = 200;
        let hg = local_complexity(LocalSocietyState {
            population: n,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.3,
            network_coupling: 0.2,
            ecological_pressure: 0.2,
        });
        let sed = local_complexity(LocalSocietyState {
            population: n,
            mode: SubsistenceMode::Sedentary,
            surplus_per_capita: 0.3,
            network_coupling: 0.2,
            ecological_pressure: 0.2,
        });
        assert!(sed.complexity_index > hg.complexity_index);
    }

    #[test]
    fn global_signal_aggregates_local_societies() {
        let small_local = LocalSocietyState {
            population: 100,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.1,
            network_coupling: 0.1,
            ecological_pressure: 0.1,
        };
        let large_complex = LocalSocietyState {
            population: 10_000,
            mode: SubsistenceMode::Agriculture,
            surplus_per_capita: 0.6,
            network_coupling: 0.9,
            ecological_pressure: 0.2,
        };

        let mixed = aggregate_from_local_societies(&[small_local, large_complex]);
        let only_small = aggregate_from_local_societies(&[small_local]);
        assert!(mixed.superorganism_index > only_small.superorganism_index);
    }

    #[test]
    fn transitions_to_sedentary_when_thresholds_are_met() {
        let cfg = TransitionConfig::default();
        let next = next_subsistence_mode(SubsistenceMode::HunterGatherer, 200, 0.4, 0.2, cfg);
        assert_eq!(next, SubsistenceMode::Sedentary);
    }

    #[test]
    fn agricultural_regresses_under_high_ecological_stress() {
        let cfg = TransitionConfig::default();
        let next = next_subsistence_mode(SubsistenceMode::Agriculture, 2_000, 0.1, 0.95, cfg);
        assert_eq!(next, SubsistenceMode::Sedentary);
    }

    #[test]
    fn step_local_society_is_deterministic() {
        let cfg = TransitionConfig::default();
        let input = LocalSocietyState {
            population: 300,
            mode: SubsistenceMode::Sedentary,
            surplus_per_capita: 0.3,
            network_coupling: 0.4,
            ecological_pressure: 0.2,
        };
        let global = EmergenceOrderParameters {
            throughput_pressure: 0.5,
            coordination_centralization: 0.5,
            policy_lock_in: 0.4,
            autonomy_loss: 0.3,
            superorganism_index: 0.45,
        };

        let a = step_local_society(input, global, cfg);
        let b = step_local_society(input, global, cfg);
        assert_eq!(a, b);
    }

    #[test]
    fn run_emergence_simulation_shows_local_to_global_shift() {
        let cfg = TransitionConfig::default();
        let initial = vec![
            LocalSocietyState {
                population: 150,
                mode: SubsistenceMode::HunterGatherer,
                surplus_per_capita: 0.3,
                network_coupling: 0.3,
                ecological_pressure: 0.1,
            },
            LocalSocietyState {
                population: 180,
                mode: SubsistenceMode::HunterGatherer,
                surplus_per_capita: 0.35,
                network_coupling: 0.35,
                ecological_pressure: 0.15,
            },
            LocalSocietyState {
                population: 220,
                mode: SubsistenceMode::Sedentary,
                surplus_per_capita: 0.4,
                network_coupling: 0.4,
                ecological_pressure: 0.2,
            },
        ];

        let snapshots = run_emergence_simulation(initial, 60, cfg);
        assert!(!snapshots.is_empty());
        let first = snapshots[0];
        let last = snapshots[snapshots.len() - 1];
        let peak_complexity = snapshots
            .iter()
            .map(|s| s.mean_local_complexity)
            .fold(0.0, f64::max);
        let peak_superorganism = snapshots
            .iter()
            .map(|s| s.global.superorganism_index)
            .fold(0.0, f64::max);
        let peak_complex_societies = snapshots
            .iter()
            .map(|s| s.sedentary_count + s.agriculture_count)
            .max()
            .unwrap_or(0);

        assert!(peak_complexity >= first.mean_local_complexity);
        assert!(peak_superorganism >= first.global.superorganism_index);
        assert!(last.global.superorganism_index >= 0.2);
        assert!(peak_complex_societies >= first.sedentary_count);
    }
}
