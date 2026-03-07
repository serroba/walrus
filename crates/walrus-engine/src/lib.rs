//! Deterministic, explicit stock-flow simulator core.

/// Minimal agent state used by the MVP model.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentState {
    /// Monetary/resource proxy held by the agent.
    pub wealth: f64,
    /// Baseline needs pressure.
    pub need: f64,
    /// Relative weight for status-seeking behavior.
    pub status_drive: f64,
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

#[cfg(test)]
mod tests {
    use super::{AgentState, SimulationConfig, SimulationEngine, WorldState};

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

        let lhs_values: Vec<f64> = lhs.run(10).into_iter().map(|s| s.aggregate_output).collect();
        let rhs_values: Vec<f64> = rhs.run(10).into_iter().map(|s| s.aggregate_output).collect();

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
}
