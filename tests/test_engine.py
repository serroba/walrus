from walrus_sim.engine import SimulationConfig, SimulationEngine
from walrus_sim.state import AgentState, WorldState


def _build_engine(seed: int = 42) -> SimulationEngine:
    return SimulationEngine(
        config=SimulationConfig(seed=seed, regen_rate=0.02, extraction_rate=1.0),
        agents=[AgentState(wealth=1.0, need=1.0, status_drive=0.5) for _ in range(5)],
        state=WorldState(tick=0, renewable_stock=100.0, nonrenewable_stock=100.0),
    )


def test_run_is_deterministic_for_same_seed() -> None:
    lhs = _build_engine(seed=7)
    rhs = _build_engine(seed=7)

    lhs_values = [s.aggregate_output for s in lhs.run(10)]
    rhs_values = [s.aggregate_output for s in rhs.run(10)]

    assert lhs_values == rhs_values


def test_nonrenewable_stock_never_goes_negative() -> None:
    engine = _build_engine()
    for _ in range(500):
        state = engine.step()
    assert state.nonrenewable_stock >= 0.0


def test_negative_ticks_raises() -> None:
    engine = _build_engine()
    try:
        engine.run(-1)
    except ValueError as err:
        assert "non-negative" in str(err)
    else:
        raise AssertionError("expected ValueError")
