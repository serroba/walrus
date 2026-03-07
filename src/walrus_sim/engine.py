"""Deterministic tick engine for early model validation."""

from __future__ import annotations

from dataclasses import dataclass
from random import Random

from walrus_sim.state import AgentState, WorldState


@dataclass(slots=True)
class SimulationConfig:
    """Runtime configuration for an engine run."""

    seed: int
    regen_rate: float
    extraction_rate: float


class SimulationEngine:
    """Minimal deterministic simulation loop with explicit resource constraints."""

    def __init__(self, config: SimulationConfig, agents: list[AgentState], state: WorldState) -> None:
        self._cfg = config
        self._rng = Random(config.seed)
        self._agents = agents
        self._state = state

    @property
    def state(self) -> WorldState:
        return self._state

    def step(self) -> WorldState:
        extracted = min(self._state.nonrenewable_stock, self._cfg.extraction_rate)
        renewable_gain = self._cfg.regen_rate * self._state.renewable_stock

        # Small bounded perturbation keeps runs stochastic but reproducible.
        shock = self._rng.uniform(-0.01, 0.01)
        output = max(0.0, extracted + renewable_gain + shock)

        self._state.nonrenewable_stock -= extracted
        self._state.renewable_stock = max(0.0, self._state.renewable_stock + renewable_gain - output * 0.1)
        self._state.aggregate_output = output
        self._state.tick += 1
        return self._state

    def run(self, ticks: int) -> list[WorldState]:
        if ticks < 0:
            raise ValueError("ticks must be non-negative")

        history: list[WorldState] = []
        for _ in range(ticks):
            history.append(
                WorldState(
                    tick=self._state.tick,
                    renewable_stock=self._state.renewable_stock,
                    nonrenewable_stock=self._state.nonrenewable_stock,
                    aggregate_output=self._state.aggregate_output,
                )
            )
            self.step()
        return history
