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

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        emergent_dynamics, group_behavior_profile, AgentState, SimulationConfig, SimulationEngine,
        SubsistenceMode, WorldState,
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
}
