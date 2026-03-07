"""Core immutable-ish model state types for the simulation."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(slots=True)
class AgentState:
    """Minimal agent state for MVP experimentation."""

    wealth: float
    need: float
    status_drive: float


@dataclass(slots=True)
class WorldState:
    """Global stock-flow state for each tick."""

    tick: int
    renewable_stock: float
    nonrenewable_stock: float
    aggregate_output: float = 0.0
