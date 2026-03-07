//! Deterministic, explicit stock-flow simulator core.

pub mod calibration;
pub mod ensemble;
pub mod evolution;

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

/// Condensed long-horizon summary for regression-style feedback checks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmergenceSummary {
    pub start_superorganism: f64,
    pub end_superorganism: f64,
    pub peak_superorganism: f64,
    pub start_mean_complexity: f64,
    pub end_mean_complexity: f64,
    pub peak_mean_complexity: f64,
    pub peak_complex_societies: usize,
}

/// Named simulation result for parameter-sweep workflows.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedSummary {
    pub name: String,
    pub summary: EmergenceSummary,
    pub final_snapshot: EmergenceSnapshot,
}

/// High-level trajectory classes for non-technical interpretation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrajectoryClass {
    StabilizingComplexity,
    OvershootAndCorrection,
    FragileTransition,
    StagnantLowComplexity,
}

/// Individual-level actor for micro interaction simulations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MicroAgent {
    pub resources: f64,
    pub trust: f64,
    pub status: f64,
    pub aggression: f64,
    pub cooperation: f64,
    pub recent_conflict: f64,
    pub recent_coop: f64,
}

/// Topology for selecting interaction partners in the agent graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionTopology {
    Ring,
    SmallWorld,
    Random,
}

/// Coefficients controlling probabilistic interaction behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InteractionParameters {
    pub cooperation_weight: f64,
    pub conflict_weight: f64,
    pub trade_weight: f64,
    pub migration_weight: f64,
    pub ecological_feedback: f64,
}

impl Default for InteractionParameters {
    fn default() -> Self {
        Self {
            cooperation_weight: 1.0,
            conflict_weight: 1.0,
            trade_weight: 1.0,
            migration_weight: 0.4,
            ecological_feedback: 1.0,
        }
    }
}

/// Aggregate statistics from one micro-interaction tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentInteractionStats {
    pub cooperations: u32,
    pub conflicts: u32,
    pub trades: u32,
    pub migrations: u32,
    pub births: u32,
    pub deaths: u32,
    pub replacements: u32,
    pub mean_trust: f64,
    pub inequality: f64,
    pub cooperation_rate: f64,
    pub conflict_rate: f64,
    pub trade_rate: f64,
    pub migration_rate: f64,
}

/// Micro-founded society state for explicit agent-based simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentBasedSociety {
    pub mode: SubsistenceMode,
    pub agents: Vec<MicroAgent>,
    pub topology: InteractionTopology,
    pub interaction_radius: usize,
    pub network_coupling: f64,
    pub ecological_pressure: f64,
    pub min_population: usize,
    pub max_population: usize,
    pub parameters: InteractionParameters,
    pub rng_state: u64,
}

/// One tick record for the agent-based simulation loop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AgentBasedSnapshot {
    pub tick: u64,
    pub mode: SubsistenceMode,
    pub macro_state: LocalSocietyState,
    pub interactions: AgentInteractionStats,
    pub complexity: LocalComplexity,
    pub emergence: EmergenceOrderParameters,
}

/// Explicit micro->macro contract used for projection and mode transitions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MicroMacroProjection {
    pub mean_resources: f64,
    pub mean_trust: f64,
    pub inequality: f64,
    pub cooperation_rate: f64,
    pub conflict_rate: f64,
    pub trade_rate: f64,
    pub migration_rate: f64,
    pub hunter_share: f64,
    pub sedentary_share: f64,
    pub agriculture_share: f64,
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

/// Produces aggregate metrics used for system-level regression checks.
#[must_use]
pub fn summarize_emergence(snapshots: &[EmergenceSnapshot]) -> EmergenceSummary {
    if snapshots.is_empty() {
        return EmergenceSummary {
            start_superorganism: 0.0,
            end_superorganism: 0.0,
            peak_superorganism: 0.0,
            start_mean_complexity: 0.0,
            end_mean_complexity: 0.0,
            peak_mean_complexity: 0.0,
            peak_complex_societies: 0,
        };
    }

    let first = snapshots[0];
    let last = snapshots[snapshots.len() - 1];
    let peak_superorganism = snapshots
        .iter()
        .map(|s| s.global.superorganism_index)
        .fold(0.0, f64::max);
    let peak_mean_complexity = snapshots
        .iter()
        .map(|s| s.mean_local_complexity)
        .fold(0.0, f64::max);
    let peak_complex_societies = snapshots
        .iter()
        .map(|s| s.sedentary_count + s.agriculture_count)
        .max()
        .unwrap_or(0);

    EmergenceSummary {
        start_superorganism: first.global.superorganism_index,
        end_superorganism: last.global.superorganism_index,
        peak_superorganism,
        start_mean_complexity: first.mean_local_complexity,
        end_mean_complexity: last.mean_local_complexity,
        peak_mean_complexity,
        peak_complex_societies,
    }
}

/// Runs one named scenario and returns summary plus final state.
#[must_use]
pub fn run_named_scenario(
    name: &str,
    societies: Vec<LocalSocietyState>,
    ticks: u64,
    cfg: TransitionConfig,
) -> NamedSummary {
    let snapshots = run_emergence_simulation(societies, ticks, cfg);
    let summary = summarize_emergence(&snapshots);
    let final_snapshot = snapshots.last().copied().unwrap_or(EmergenceSnapshot {
        tick: 0,
        global: EmergenceOrderParameters {
            throughput_pressure: 0.0,
            coordination_centralization: 0.0,
            policy_lock_in: 0.0,
            autonomy_loss: 0.0,
            superorganism_index: 0.0,
        },
        mean_local_complexity: 0.0,
        hunter_gatherer_count: 0,
        sedentary_count: 0,
        agriculture_count: 0,
    });

    NamedSummary {
        name: name.to_string(),
        summary,
        final_snapshot,
    }
}

/// Classifies long-horizon behavior into a human-readable regime.
#[must_use]
pub fn classify_trajectory(summary: EmergenceSummary) -> TrajectoryClass {
    let peak_gain = summary.peak_superorganism - summary.start_superorganism;
    let correction = summary.peak_superorganism - summary.end_superorganism;
    let stagnant = summary.peak_superorganism < 0.45
        && summary.end_superorganism < 0.30
        && summary.end_mean_complexity < 0.30;

    if summary.end_superorganism >= 0.50
        && summary.end_mean_complexity >= 0.45
        && correction <= 0.10
    {
        TrajectoryClass::StabilizingComplexity
    } else if stagnant {
        TrajectoryClass::StagnantLowComplexity
    } else if peak_gain >= 0.06 && correction >= 0.12 {
        TrajectoryClass::OvershootAndCorrection
    } else {
        TrajectoryClass::FragileTransition
    }
}

/// Builds deterministic initial agents for a subsistence mode.
#[must_use]
pub fn seed_micro_agents(count: usize, mode: SubsistenceMode) -> Vec<MicroAgent> {
    let (resource_base, trust_base, aggression_base, cooperation_base) = match mode {
        SubsistenceMode::HunterGatherer => (0.22, 0.62, 0.25, 0.68),
        SubsistenceMode::Sedentary => (0.35, 0.52, 0.35, 0.55),
        SubsistenceMode::Agriculture => (0.50, 0.44, 0.45, 0.46),
    };

    (0..count)
        .map(|i| {
            let i_f = i as f64;
            let wave = (i_f * 0.37).sin() * 0.08;
            let skew = ((i % 7) as f64) / 30.0;
            MicroAgent {
                resources: (resource_base + wave + skew).clamp(0.01, 2.0),
                trust: (trust_base + wave * 0.4).clamp(0.0, 1.0),
                status: (0.45 + skew * 0.6).clamp(0.0, 1.0),
                aggression: (aggression_base + ((i % 5) as f64) * 0.04).clamp(0.0, 1.0),
                cooperation: (cooperation_base - ((i % 6) as f64) * 0.03).clamp(0.0, 1.0),
                recent_conflict: 0.0,
                recent_coop: 0.0,
            }
        })
        .collect()
}

/// Creates an agent-based society with deterministic seed population.
#[must_use]
pub fn seed_agent_based_society(
    count: usize,
    mode: SubsistenceMode,
    network_coupling: f64,
    ecological_pressure: f64,
) -> AgentBasedSociety {
    AgentBasedSociety {
        mode,
        agents: seed_micro_agents(count, mode),
        topology: InteractionTopology::Ring,
        interaction_radius: 1,
        network_coupling: clamp01(network_coupling),
        ecological_pressure: clamp01(ecological_pressure),
        min_population: count.saturating_div(2).max(8),
        max_population: count.saturating_mul(4).max(count.saturating_add(1)),
        parameters: InteractionParameters::default(),
        rng_state: 0x9e37_79b9_7f4a_7c15_u64 ^ (count as u64),
    }
}

/// Creates an agent-based society with configurable topology and deterministic RNG seed.
#[must_use]
pub fn seed_agent_based_society_with_topology(
    count: usize,
    mode: SubsistenceMode,
    network_coupling: f64,
    ecological_pressure: f64,
    topology: InteractionTopology,
    interaction_radius: usize,
    seed: u64,
) -> AgentBasedSociety {
    let mut seeded = seed_agent_based_society(count, mode, network_coupling, ecological_pressure);
    seeded.topology = topology;
    seeded.interaction_radius = interaction_radius.max(1);
    seeded.rng_state = seed.max(1);
    seeded
}

/// Converts micro state into local macro proxies used by the emergence model.
#[must_use]
pub fn macro_from_agents(society: &AgentBasedSociety) -> LocalSocietyState {
    let pop = society.agents.len().max(1) as u32;
    let projection = micro_macro_projection(society);
    let weighted_surplus = projection.mean_resources
        * (1.0 + 0.12 * projection.cooperation_rate - 0.08 * projection.conflict_rate)
        * (1.0 - 0.22 * projection.inequality)
        * (1.0 - 0.15 * society.ecological_pressure);

    LocalSocietyState {
        population: pop,
        mode: society.mode,
        surplus_per_capita: weighted_surplus.clamp(0.0, 2.0),
        network_coupling: society.network_coupling,
        ecological_pressure: society.ecological_pressure,
    }
}

/// Advances the agent-level system by one interaction tick.
#[must_use]
pub fn step_agent_based_society(society: &mut AgentBasedSociety) -> AgentInteractionStats {
    let n = society.agents.len();
    if n < 2 || n > (u32::MAX as usize) {
        return AgentInteractionStats {
            cooperations: 0,
            conflicts: 0,
            trades: 0,
            migrations: 0,
            births: 0,
            deaths: 0,
            replacements: 0,
            mean_trust: society.agents.first().map_or(0.0, |a| a.trust),
            inequality: 0.0,
            cooperation_rate: 0.0,
            conflict_rate: 0.0,
            trade_rate: 0.0,
            migration_rate: 0.0,
        };
    }

    let mut cooperations = 0_u32;
    let mut conflicts = 0_u32;
    let mut trades = 0_u32;
    let mut migrations = 0_u32;

    for idx in 0..n {
        let j = partner_for(society, idx);
        if j == idx {
            continue;
        }

        let (left, right) = if idx < j {
            let (head, tail) = society.agents.split_at_mut(j);
            (&mut head[idx], &mut tail[0])
        } else {
            let (head, tail) = society.agents.split_at_mut(idx);
            (&mut tail[0], &mut head[j])
        };

        let stress = clamp01(society.ecological_pressure * society.parameters.ecological_feedback);
        let coop_bias = 0.40 * left.cooperation
            + 0.30 * right.cooperation
            + 0.20 * left.trust
            + 0.10 * right.trust
            + 0.12 * left.recent_coop
            + 0.08 * right.recent_coop
            - 0.18 * stress;
        let conflict_bias = 0.42 * left.aggression
            + 0.36 * right.aggression
            + 0.22 * (left.status - right.status).abs()
            + 0.16 * left.recent_conflict
            + 0.12 * right.recent_conflict
            + 0.20 * stress;
        let trade_bias = 0.50 * (1.0 - (left.resources - right.resources).abs() / 3.0).max(0.0)
            + 0.20 * (left.trust + right.trust)
            + 0.15 * (1.0 - stress);
        let migration_bias = 0.32 * stress
            + 0.28 * (0.4 - 0.5 * (left.resources + right.resources)).max(0.0)
            + 0.16 * (1.0 - 0.5 * (left.trust + right.trust));

        let coop_p = clamp01(
            society.parameters.cooperation_weight * coop_bias
                / (1.6 + society.parameters.cooperation_weight),
        );
        let conflict_p = clamp01(
            society.parameters.conflict_weight * conflict_bias
                / (1.9 + society.parameters.conflict_weight),
        );
        let trade_p = clamp01(
            society.parameters.trade_weight * trade_bias / (1.8 + society.parameters.trade_weight),
        );
        let migration_p = clamp01(
            society.parameters.migration_weight * migration_bias
                / (2.2 + society.parameters.migration_weight),
        );

        let pick = rand01(&mut society.rng_state);
        let total = coop_p + conflict_p + trade_p + migration_p;
        let norm = if total > 0.0 { total } else { 1.0 };
        let coop_cut = coop_p / norm;
        let conflict_cut = coop_cut + (conflict_p / norm);
        let trade_cut = conflict_cut + (trade_p / norm);

        if pick < coop_cut {
            cooperations = cooperations.saturating_add(1);
            let gain = (0.012 + 0.010 * society.network_coupling).clamp(0.0, 0.05);
            left.resources = (left.resources + gain).clamp(0.0, 3.0);
            right.resources = (right.resources + gain).clamp(0.0, 3.0);
            left.trust = (left.trust + 0.018).clamp(0.0, 1.0);
            right.trust = (right.trust + 0.018).clamp(0.0, 1.0);
            left.recent_coop = (left.recent_coop * 0.80 + 0.20).clamp(0.0, 1.0);
            right.recent_coop = (right.recent_coop * 0.80 + 0.20).clamp(0.0, 1.0);
            left.recent_conflict = (left.recent_conflict * 0.85).clamp(0.0, 1.0);
            right.recent_conflict = (right.recent_conflict * 0.85).clamp(0.0, 1.0);
        } else if pick < conflict_cut {
            conflicts = conflicts.saturating_add(1);
            let transfer = (0.016 + 0.018 * conflict_p).clamp(0.0, 0.08);
            if left.status >= right.status {
                left.resources = (left.resources + transfer).clamp(0.0, 3.0);
                right.resources = (right.resources - transfer).clamp(0.0, 3.0);
                left.status = (left.status + 0.008).clamp(0.0, 1.0);
                right.status = (right.status - 0.006).clamp(0.0, 1.0);
            } else {
                right.resources = (right.resources + transfer).clamp(0.0, 3.0);
                left.resources = (left.resources - transfer).clamp(0.0, 3.0);
                right.status = (right.status + 0.008).clamp(0.0, 1.0);
                left.status = (left.status - 0.006).clamp(0.0, 1.0);
            }
            left.trust = (left.trust - 0.022).clamp(0.0, 1.0);
            right.trust = (right.trust - 0.022).clamp(0.0, 1.0);
            left.recent_conflict = (left.recent_conflict * 0.78 + 0.22).clamp(0.0, 1.0);
            right.recent_conflict = (right.recent_conflict * 0.78 + 0.22).clamp(0.0, 1.0);
            left.recent_coop = (left.recent_coop * 0.82).clamp(0.0, 1.0);
            right.recent_coop = (right.recent_coop * 0.82).clamp(0.0, 1.0);
        } else if pick < trade_cut {
            trades = trades.saturating_add(1);
            let mean = 0.5 * (left.resources + right.resources);
            left.resources = (0.70 * left.resources + 0.30 * mean).clamp(0.0, 3.0);
            right.resources = (0.70 * right.resources + 0.30 * mean).clamp(0.0, 3.0);
            left.trust = (left.trust + 0.006).clamp(0.0, 1.0);
            right.trust = (right.trust + 0.006).clamp(0.0, 1.0);
            left.recent_coop = (left.recent_coop * 0.92 + 0.06).clamp(0.0, 1.0);
            right.recent_coop = (right.recent_coop * 0.92 + 0.06).clamp(0.0, 1.0);
            left.recent_conflict = (left.recent_conflict * 0.92).clamp(0.0, 1.0);
            right.recent_conflict = (right.recent_conflict * 0.92).clamp(0.0, 1.0);
        } else if rand01(&mut society.rng_state) < migration_p {
            migrations = migrations.saturating_add(1);
            let trust_shift = (0.012 - 0.018 * stress).clamp(-0.03, 0.02);
            left.trust = (left.trust + trust_shift).clamp(0.0, 1.0);
            right.trust = (right.trust + trust_shift).clamp(0.0, 1.0);
            left.resources = (left.resources * (0.99 - 0.02 * stress)).clamp(0.0, 3.0);
            right.resources = (right.resources * (0.99 - 0.02 * stress)).clamp(0.0, 3.0);
            left.recent_conflict = (left.recent_conflict * 0.90 + 0.03 * stress).clamp(0.0, 1.0);
            right.recent_conflict = (right.recent_conflict * 0.90 + 0.03 * stress).clamp(0.0, 1.0);
        }
    }

    let maintenance = match society.mode {
        SubsistenceMode::HunterGatherer => 0.006,
        SubsistenceMode::Sedentary => 0.010,
        SubsistenceMode::Agriculture => 0.014,
    };
    let ecological_cost = 0.020 * society.ecological_pressure;
    for agent in &mut society.agents {
        agent.resources = (agent.resources - maintenance - ecological_cost).clamp(0.0, 3.0);
        agent.recent_coop = (agent.recent_coop * 0.97).clamp(0.0, 1.0);
        agent.recent_conflict = (agent.recent_conflict * 0.97).clamp(0.0, 1.0);
    }

    let mut births = 0_u32;
    let mut deaths = 0_u32;
    let mut replacements = 0_u32;
    let mut survivors: Vec<MicroAgent> = Vec::with_capacity(society.agents.len());
    let birth_chance = match society.mode {
        SubsistenceMode::HunterGatherer => 0.008,
        SubsistenceMode::Sedentary => 0.010,
        SubsistenceMode::Agriculture => 0.012,
    };

    for agent in society.agents.drain(..) {
        let death_risk = (0.003
            + 0.045 * society.ecological_pressure
            + (0.25 - agent.resources).max(0.0) * 0.05)
            .clamp(0.0, 0.20);
        if rand01(&mut society.rng_state) < death_risk && survivors.len() > society.min_population {
            deaths = deaths.saturating_add(1);
            continue;
        }

        let fertile = agent.resources > 0.35 && agent.trust > 0.25;
        if fertile
            && survivors.len() < society.max_population
            && rand01(&mut society.rng_state) < birth_chance
        {
            births = births.saturating_add(1);
            let child = MicroAgent {
                resources: (0.40 * agent.resources + 0.07).clamp(0.05, 1.4),
                trust: (0.85 * agent.trust + 0.10 * rand01(&mut society.rng_state)).clamp(0.0, 1.0),
                status: (0.70 * agent.status + 0.20 * rand01(&mut society.rng_state))
                    .clamp(0.0, 1.0),
                aggression: (0.75 * agent.aggression + 0.20 * rand01(&mut society.rng_state))
                    .clamp(0.0, 1.0),
                cooperation: (0.75 * agent.cooperation + 0.20 * rand01(&mut society.rng_state))
                    .clamp(0.0, 1.0),
                recent_conflict: 0.0,
                recent_coop: 0.0,
            };
            survivors.push(child);
        }
        survivors.push(agent);
    }

    if survivors.len() < society.min_population {
        let target = society
            .min_population
            .min(society.max_population.max(society.min_population));
        while survivors.len() < target {
            replacements = replacements.saturating_add(1);
            let seed_wave = rand01(&mut society.rng_state);
            survivors.push(MicroAgent {
                resources: (0.24 + 0.20 * seed_wave).clamp(0.05, 1.0),
                trust: (0.30 + 0.40 * rand01(&mut society.rng_state)).clamp(0.0, 1.0),
                status: (0.20 + 0.50 * rand01(&mut society.rng_state)).clamp(0.0, 1.0),
                aggression: (0.20 + 0.50 * rand01(&mut society.rng_state)).clamp(0.0, 1.0),
                cooperation: (0.25 + 0.55 * rand01(&mut society.rng_state)).clamp(0.0, 1.0),
                recent_conflict: 0.0,
                recent_coop: 0.0,
            });
        }
    }
    society.agents = survivors;

    let n_after = society.agents.len().max(1) as f64;
    let mean_trust = society.agents.iter().map(|a| a.trust).sum::<f64>() / n_after;
    let inequality = gini_of_resources(&society.agents);
    let total_events = (cooperations + conflicts + trades + migrations).max(1) as f64;

    AgentInteractionStats {
        cooperations,
        conflicts,
        trades,
        migrations,
        births,
        deaths,
        replacements,
        mean_trust,
        inequality,
        cooperation_rate: (cooperations as f64) / total_events,
        conflict_rate: (conflicts as f64) / total_events,
        trade_rate: (trades as f64) / total_events,
        migration_rate: (migrations as f64) / total_events,
    }
}

/// Runs a micro agent-based simulation and projects each tick into emergence metrics.
#[must_use]
pub fn run_agent_based_simulation(
    mut society: AgentBasedSociety,
    ticks: u64,
    cfg: TransitionConfig,
) -> Vec<AgentBasedSnapshot> {
    let mut out = Vec::with_capacity(ticks as usize);

    for tick in 0..ticks {
        let interactions = step_agent_based_society(&mut society);
        let macro_state = macro_from_agents(&society);
        let projection = micro_macro_projection(&society);
        let complexity = local_complexity(macro_state);
        let emergence = emergence_from_projection(macro_state, projection);

        out.push(AgentBasedSnapshot {
            tick,
            mode: society.mode,
            macro_state,
            interactions,
            complexity,
            emergence,
        });

        let next_mode = next_subsistence_mode(
            society.mode,
            macro_state.population,
            macro_state.surplus_per_capita,
            macro_state.ecological_pressure,
            cfg,
        );
        society.mode = next_mode;

        let trust_drag = 0.03 * (1.0 - interactions.mean_trust);
        let restoration = match society.mode {
            SubsistenceMode::HunterGatherer => 0.018,
            SubsistenceMode::Sedentary => 0.010,
            SubsistenceMode::Agriculture => 0.006,
        };
        society.ecological_pressure =
            (society.ecological_pressure + 0.020 * emergence.throughput_pressure + trust_drag
                - restoration)
                .clamp(0.0, 1.0);

        society.network_coupling = (society.network_coupling
            + 0.015 * emergence.coordination_centralization
            - 0.010 * society.ecological_pressure)
            .clamp(0.0, 1.0);
    }

    out
}

/// Computes explicit micro-level aggregates and composition shares.
#[must_use]
pub fn micro_macro_projection(society: &AgentBasedSociety) -> MicroMacroProjection {
    if society.agents.is_empty() {
        return MicroMacroProjection {
            mean_resources: 0.0,
            mean_trust: 0.0,
            inequality: 0.0,
            cooperation_rate: 0.0,
            conflict_rate: 0.0,
            trade_rate: 0.0,
            migration_rate: 0.0,
            hunter_share: 0.0,
            sedentary_share: 0.0,
            agriculture_share: 0.0,
        };
    }

    let n = society.agents.len() as f64;
    let mean_resources = society.agents.iter().map(|a| a.resources).sum::<f64>() / n;
    let mean_trust = society.agents.iter().map(|a| a.trust).sum::<f64>() / n;
    let inequality = gini_of_resources(&society.agents);

    let hunter_like = society
        .agents
        .iter()
        .filter(|a| a.resources < 0.55 && a.status < 0.55)
        .count() as f64;
    let agri_like = society
        .agents
        .iter()
        .filter(|a| a.resources > 0.95 && a.status > 0.52)
        .count() as f64;
    let sedentary_like = (n - hunter_like - agri_like).max(0.0);

    let conflict_pressure = society
        .agents
        .iter()
        .map(|a| a.recent_conflict * (0.5 + a.aggression))
        .sum::<f64>()
        / n;
    let coop_pressure = society
        .agents
        .iter()
        .map(|a| a.recent_coop * (0.5 + a.cooperation))
        .sum::<f64>()
        / n;
    let trade_pressure = (1.0 - inequality).clamp(0.0, 1.0) * (0.4 + 0.6 * mean_trust);
    let migration_pressure =
        (society.ecological_pressure * (1.0 - mean_resources / 2.0)).clamp(0.0, 1.0);
    let total = (coop_pressure + conflict_pressure + trade_pressure + migration_pressure).max(1e-9);

    MicroMacroProjection {
        mean_resources,
        mean_trust,
        inequality,
        cooperation_rate: coop_pressure / total,
        conflict_rate: conflict_pressure / total,
        trade_rate: trade_pressure / total,
        migration_rate: migration_pressure / total,
        hunter_share: hunter_like / n,
        sedentary_share: sedentary_like / n,
        agriculture_share: agri_like / n,
    }
}

/// Adjusts emergence order parameters with micro-level event composition.
#[must_use]
pub fn emergence_from_projection(
    macro_state: LocalSocietyState,
    projection: MicroMacroProjection,
) -> EmergenceOrderParameters {
    let base = emergence_order_parameters(
        macro_state.population,
        macro_state.mode,
        macro_state.surplus_per_capita,
        macro_state.network_coupling,
        macro_state.ecological_pressure,
    );
    let throughput = clamp01(
        base.throughput_pressure
            + 0.15 * projection.trade_rate
            + 0.12 * projection.cooperation_rate
            - 0.18 * projection.migration_rate,
    );
    let centralization = clamp01(
        base.coordination_centralization
            + 0.16 * projection.conflict_rate
            + 0.10 * projection.inequality
            - 0.08 * projection.hunter_share,
    );
    let lock_in = clamp01(
        base.policy_lock_in + 0.14 * projection.agriculture_share + 0.08 * projection.inequality
            - 0.06 * projection.migration_rate,
    );
    let autonomy_loss = clamp01(
        base.autonomy_loss
            + 0.12 * projection.conflict_rate
            + 0.10 * projection.inequality
            + 0.06 * projection.trade_rate
            - 0.08 * projection.cooperation_rate,
    );
    let superorganism_index =
        clamp01(0.30 * throughput + 0.28 * centralization + 0.24 * lock_in + 0.18 * autonomy_loss);

    EmergenceOrderParameters {
        throughput_pressure: throughput,
        coordination_centralization: centralization,
        policy_lock_in: lock_in,
        autonomy_loss,
        superorganism_index,
    }
}

fn partner_for(society: &mut AgentBasedSociety, idx: usize) -> usize {
    let n = society.agents.len();
    if n < 2 {
        return idx;
    }
    match society.topology {
        InteractionTopology::Ring => {
            let span = (society.interaction_radius.max(1)) % n.max(2);
            let dir = if rand01(&mut society.rng_state) < 0.5 {
                n.saturating_sub(span)
            } else {
                span
            };
            (idx + dir) % n
        }
        InteractionTopology::SmallWorld => {
            let span = (society.interaction_radius.max(1)) % n.max(2);
            let dir = if rand01(&mut society.rng_state) < 0.5 {
                n.saturating_sub(span)
            } else {
                span
            };
            let local = (idx + dir) % n;
            if rand01(&mut society.rng_state) < 0.18 {
                let mut pick = (rand01(&mut society.rng_state) * (n as f64)).floor() as usize;
                if pick == idx {
                    pick = (pick + 1) % n;
                }
                pick
            } else {
                local
            }
        }
        InteractionTopology::Random => {
            let mut pick = (rand01(&mut society.rng_state) * (n as f64)).floor() as usize;
            if pick == idx {
                pick = (pick + 1) % n;
            }
            pick
        }
    }
}

fn rand01(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*state as f64) / (u64::MAX as f64)
}

fn gini_of_resources(agents: &[MicroAgent]) -> f64 {
    if agents.is_empty() {
        return 0.0;
    }

    let mut values: Vec<f64> = agents.iter().map(|a| a.resources.max(0.0)).collect();
    values.sort_by(f64::total_cmp);
    let n = values.len() as f64;
    let sum = values.iter().sum::<f64>();
    if sum <= 0.0 {
        return 0.0;
    }

    let weighted = values
        .iter()
        .enumerate()
        .map(|(i, v)| ((i as f64) + 1.0) * v)
        .sum::<f64>();

    (2.0 * weighted) / (n * sum) - (n + 1.0) / n
}

/// Baseline multi-society starting point for long-horizon emergence runs.
#[must_use]
pub fn scenario_local_emergence_baseline() -> Vec<LocalSocietyState> {
    vec![
        LocalSocietyState {
            population: 90,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.18,
            network_coupling: 0.15,
            ecological_pressure: 0.08,
        },
        LocalSocietyState {
            population: 130,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.22,
            network_coupling: 0.20,
            ecological_pressure: 0.10,
        },
        LocalSocietyState {
            population: 240,
            mode: SubsistenceMode::Sedentary,
            surplus_per_capita: 0.35,
            network_coupling: 0.35,
            ecological_pressure: 0.18,
        },
        LocalSocietyState {
            population: 820,
            mode: SubsistenceMode::Agriculture,
            surplus_per_capita: 0.52,
            network_coupling: 0.62,
            ecological_pressure: 0.30,
        },
    ]
}

/// Higher stress setup for testing balancing loops and regression pressure.
#[must_use]
pub fn scenario_ecological_stress() -> Vec<LocalSocietyState> {
    vec![
        LocalSocietyState {
            population: 300,
            mode: SubsistenceMode::Sedentary,
            surplus_per_capita: 0.20,
            network_coupling: 0.30,
            ecological_pressure: 0.75,
        },
        LocalSocietyState {
            population: 1_500,
            mode: SubsistenceMode::Agriculture,
            surplus_per_capita: 0.25,
            network_coupling: 0.70,
            ecological_pressure: 0.82,
        },
    ]
}

/// High-growth, tightly coupled initial condition.
#[must_use]
pub fn scenario_dense_coupled_growth() -> Vec<LocalSocietyState> {
    vec![
        LocalSocietyState {
            population: 220,
            mode: SubsistenceMode::Sedentary,
            surplus_per_capita: 0.45,
            network_coupling: 0.65,
            ecological_pressure: 0.12,
        },
        LocalSocietyState {
            population: 650,
            mode: SubsistenceMode::Sedentary,
            surplus_per_capita: 0.55,
            network_coupling: 0.78,
            ecological_pressure: 0.15,
        },
        LocalSocietyState {
            population: 1_100,
            mode: SubsistenceMode::Agriculture,
            surplus_per_capita: 0.62,
            network_coupling: 0.82,
            ecological_pressure: 0.2,
        },
    ]
}

/// Low-coupling fragmented initial condition.
#[must_use]
pub fn scenario_fragmented_low_coupling() -> Vec<LocalSocietyState> {
    vec![
        LocalSocietyState {
            population: 60,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.10,
            network_coupling: 0.05,
            ecological_pressure: 0.08,
        },
        LocalSocietyState {
            population: 70,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.12,
            network_coupling: 0.07,
            ecological_pressure: 0.10,
        },
        LocalSocietyState {
            population: 80,
            mode: SubsistenceMode::HunterGatherer,
            surplus_per_capita: 0.11,
            network_coupling: 0.06,
            ecological_pressure: 0.09,
        },
    ]
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        aggregate_from_local_societies, classify_trajectory, emergence_order_parameters,
        emergent_dynamics, gini_of_resources, group_behavior_profile, local_complexity,
        macro_from_agents, micro_macro_projection, next_subsistence_mode,
        run_agent_based_simulation, run_emergence_simulation, scenario_dense_coupled_growth,
        scenario_ecological_stress, scenario_fragmented_low_coupling,
        scenario_local_emergence_baseline, seed_agent_based_society,
        seed_agent_based_society_with_topology, seed_micro_agents, step_agent_based_society,
        step_local_society, summarize_emergence, AgentState, EmergenceOrderParameters,
        InteractionTopology, LocalSocietyState, MicroAgent, SimulationConfig, SimulationEngine,
        SubsistenceMode, TrajectoryClass, TransitionConfig, WorldState,
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
        let summary = summarize_emergence(&snapshots);

        assert!(summary.peak_mean_complexity >= summary.start_mean_complexity);
        assert!(summary.peak_superorganism >= summary.start_superorganism);
        assert!(summary.end_superorganism >= 0.2);
        assert!(summary.peak_complex_societies >= first.sedentary_count);
    }

    #[test]
    fn scenario_builders_produce_non_empty_societies() {
        let baseline = scenario_local_emergence_baseline();
        let stress = scenario_ecological_stress();
        let dense = scenario_dense_coupled_growth();
        let fragmented = scenario_fragmented_low_coupling();
        assert!(!baseline.is_empty());
        assert!(!stress.is_empty());
        assert!(!dense.is_empty());
        assert!(!fragmented.is_empty());
    }

    #[test]
    fn classify_stagnant_low_complexity_trajectory() {
        let class = classify_trajectory(super::EmergenceSummary {
            start_superorganism: 0.28,
            end_superorganism: 0.22,
            peak_superorganism: 0.35,
            start_mean_complexity: 0.30,
            end_mean_complexity: 0.21,
            peak_mean_complexity: 0.34,
            peak_complex_societies: 0,
        });
        assert_eq!(class, TrajectoryClass::StagnantLowComplexity);
    }

    #[test]
    fn classify_overshoot_and_correction_trajectory() {
        let class = classify_trajectory(super::EmergenceSummary {
            start_superorganism: 0.50,
            end_superorganism: 0.34,
            peak_superorganism: 0.72,
            start_mean_complexity: 0.55,
            end_mean_complexity: 0.33,
            peak_mean_complexity: 0.80,
            peak_complex_societies: 3,
        });
        assert_eq!(class, TrajectoryClass::OvershootAndCorrection);
    }

    #[test]
    fn seed_micro_agents_builds_expected_count() {
        let agents = seed_micro_agents(64, SubsistenceMode::Sedentary);
        assert_eq!(agents.len(), 64);
        assert!(agents.iter().all(|a| (0.0..=1.0).contains(&a.trust)));
    }

    #[test]
    fn step_agent_based_society_generates_interaction_events() {
        let mut society = seed_agent_based_society(36, SubsistenceMode::Sedentary, 0.4, 0.2);
        let stats = step_agent_based_society(&mut society);
        let total = stats.cooperations + stats.conflicts + stats.trades + stats.migrations;
        assert!(total > 0);
        assert!((0.0..=1.0).contains(&stats.mean_trust));
        assert!((0.0..=1.0).contains(&stats.inequality));
        assert!((0.0..=1.0).contains(&stats.cooperation_rate));
        assert!((0.0..=1.0).contains(&stats.conflict_rate));
        assert!((0.0..=1.0).contains(&stats.trade_rate));
        assert!((0.0..=1.0).contains(&stats.migration_rate));
    }

    #[test]
    fn macro_from_agents_reflects_population_and_mode() {
        let society = seed_agent_based_society(81, SubsistenceMode::HunterGatherer, 0.2, 0.1);
        let macro_state = macro_from_agents(&society);
        assert_eq!(macro_state.population, 81);
        assert_eq!(macro_state.mode, SubsistenceMode::HunterGatherer);
    }

    #[test]
    fn run_agent_based_simulation_produces_snapshots() {
        let society = seed_agent_based_society(49, SubsistenceMode::Sedentary, 0.4, 0.2);
        let snaps = run_agent_based_simulation(society, 50, TransitionConfig::default());
        assert_eq!(snaps.len(), 50);
        assert!(snaps[49].emergence.superorganism_index >= 0.0);
    }

    #[test]
    fn topology_with_fixed_seed_is_deterministic() {
        let mut lhs = seed_agent_based_society_with_topology(
            64,
            SubsistenceMode::Sedentary,
            0.45,
            0.2,
            InteractionTopology::SmallWorld,
            2,
            1337,
        );
        let mut rhs = seed_agent_based_society_with_topology(
            64,
            SubsistenceMode::Sedentary,
            0.45,
            0.2,
            InteractionTopology::SmallWorld,
            2,
            1337,
        );

        for _ in 0..20 {
            let a = step_agent_based_society(&mut lhs);
            let b = step_agent_based_society(&mut rhs);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn stress_recovery_respects_transition_hysteresis() {
        let cfg = TransitionConfig::default();
        let mut society = seed_agent_based_society_with_topology(
            220,
            SubsistenceMode::Sedentary,
            0.35,
            0.92,
            InteractionTopology::Ring,
            1,
            44,
        );
        for _ in 0..15 {
            let _ = step_agent_based_society(&mut society);
            let macro_state = macro_from_agents(&society);
            society.mode = next_subsistence_mode(
                society.mode,
                macro_state.population,
                macro_state.surplus_per_capita,
                macro_state.ecological_pressure,
                cfg,
            );
        }
        assert_eq!(society.mode, SubsistenceMode::HunterGatherer);

        society.ecological_pressure = 0.05;
        for agent in &mut society.agents {
            agent.resources = 1.2;
        }
        for _ in 0..25 {
            let _ = step_agent_based_society(&mut society);
            let macro_state = macro_from_agents(&society);
            society.mode = next_subsistence_mode(
                society.mode,
                macro_state.population,
                macro_state.surplus_per_capita,
                macro_state.ecological_pressure,
                cfg,
            );
        }
        assert!(matches!(
            society.mode,
            SubsistenceMode::Sedentary | SubsistenceMode::Agriculture
        ));
    }

    #[test]
    fn demographic_turnover_changes_population_within_bounds() {
        let mut society = seed_agent_based_society_with_topology(
            42,
            SubsistenceMode::HunterGatherer,
            0.25,
            0.7,
            InteractionTopology::Random,
            3,
            87,
        );
        society.min_population = 30;
        society.max_population = 80;
        let mut turnover = 0_u32;
        for _ in 0..25 {
            let stats = step_agent_based_society(&mut society);
            turnover = turnover
                .saturating_add(stats.births)
                .saturating_add(stats.deaths)
                .saturating_add(stats.replacements);
        }
        let after = society.agents.len();
        assert!((30..=80).contains(&after));
        assert!(turnover > 0);
    }

    #[test]
    fn projection_tracks_mode_composition() {
        let society = seed_agent_based_society_with_topology(
            50,
            SubsistenceMode::Sedentary,
            0.4,
            0.2,
            InteractionTopology::Ring,
            1,
            9,
        );
        let p = micro_macro_projection(&society);
        let sum = p.hunter_share + p.sedentary_share + p.agriculture_share;
        assert!((sum - 1.0).abs() < 1e-6);
        assert!((0.0..=1.0).contains(&p.conflict_rate));
    }

    #[test]
    fn gini_is_zero_for_equal_distribution() {
        let agents = vec![
            MicroAgent {
                resources: 1.0,
                trust: 0.5,
                status: 0.5,
                aggression: 0.2,
                cooperation: 0.6,
                recent_conflict: 0.0,
                recent_coop: 0.0,
            };
            8
        ];
        let g = gini_of_resources(&agents);
        assert!(g <= 0.0001);
    }

    #[test]
    fn gini_increases_with_unequal_distribution() {
        let equal = vec![
            MicroAgent {
                resources: 1.0,
                trust: 0.5,
                status: 0.5,
                aggression: 0.2,
                cooperation: 0.6,
                recent_conflict: 0.0,
                recent_coop: 0.0,
            };
            6
        ];
        let unequal = vec![
            MicroAgent {
                resources: 0.1,
                trust: 0.5,
                status: 0.5,
                aggression: 0.2,
                cooperation: 0.6,
                recent_conflict: 0.0,
                recent_coop: 0.0,
            },
            MicroAgent {
                resources: 0.1,
                trust: 0.5,
                status: 0.5,
                aggression: 0.2,
                cooperation: 0.6,
                recent_conflict: 0.0,
                recent_coop: 0.0,
            },
            MicroAgent {
                resources: 0.1,
                trust: 0.5,
                status: 0.5,
                aggression: 0.2,
                cooperation: 0.6,
                recent_conflict: 0.0,
                recent_coop: 0.0,
            },
            MicroAgent {
                resources: 2.8,
                trust: 0.5,
                status: 0.5,
                aggression: 0.2,
                cooperation: 0.6,
                recent_conflict: 0.0,
                recent_coop: 0.0,
            },
        ];

        assert!(gini_of_resources(&unequal) > gini_of_resources(&equal));
    }
}
