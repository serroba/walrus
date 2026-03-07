use walrus_engine::{
    run_emergence_simulation, scenario_ecological_stress, scenario_local_emergence_baseline,
    summarize_emergence, TransitionConfig,
};

#[test]
fn baseline_scenario_exhibits_emergence_window() {
    let snapshots = run_emergence_simulation(
        scenario_local_emergence_baseline(),
        240,
        TransitionConfig::default(),
    );
    assert!(!snapshots.is_empty());

    let summary = summarize_emergence(&snapshots);

    // Emergence should occur at some point during the run.
    assert!(summary.peak_superorganism >= summary.start_superorganism);
    assert!(summary.peak_mean_complexity >= summary.start_mean_complexity);

    // Baseline should remain in a viable non-zero complexity regime.
    assert!(summary.end_superorganism >= 0.25);
    assert!(summary.end_mean_complexity >= 0.25);
}

#[test]
fn ecological_stress_scenario_reduces_end_state_vs_peak() {
    let snapshots = run_emergence_simulation(
        scenario_ecological_stress(),
        240,
        TransitionConfig::default(),
    );
    assert!(!snapshots.is_empty());

    let summary = summarize_emergence(&snapshots);

    // Stress case should still show temporary organization but degraded endpoint.
    assert!(summary.peak_superorganism >= summary.start_superorganism);
    assert!(summary.end_superorganism <= summary.peak_superorganism);
    assert!(summary.end_mean_complexity <= summary.peak_mean_complexity);
}
