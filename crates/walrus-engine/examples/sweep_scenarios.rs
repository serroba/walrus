use walrus_engine::{
    classify_trajectory, run_named_scenario, scenario_dense_coupled_growth,
    scenario_ecological_stress, scenario_fragmented_low_coupling,
    scenario_local_emergence_baseline, LocalSocietyState, TrajectoryClass, TransitionConfig,
};

fn class_label(class: TrajectoryClass) -> &'static str {
    match class {
        TrajectoryClass::StabilizingComplexity => "stabilizing",
        TrajectoryClass::OvershootAndCorrection => "overshoot-correction",
        TrajectoryClass::FragileTransition => "fragile-transition",
        TrajectoryClass::StagnantLowComplexity => "stagnant-low-complexity",
    }
}

fn print_header() {
    println!(
        "scenario,class,start_SO,peak_SO,end_SO,start_CX,peak_CX,end_CX,peak_complex_societies,final_modes(H/S/A)",
    );
}

fn print_row(name: &str, result: &walrus_engine::NamedSummary, class: TrajectoryClass) {
    println!(
        "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{}/{}/{}",
        name,
        class_label(class),
        result.summary.start_superorganism,
        result.summary.peak_superorganism,
        result.summary.end_superorganism,
        result.summary.start_mean_complexity,
        result.summary.peak_mean_complexity,
        result.summary.end_mean_complexity,
        result.summary.peak_complex_societies,
        result.final_snapshot.hunter_gatherer_count,
        result.final_snapshot.sedentary_count,
        result.final_snapshot.agriculture_count,
    );
}

fn main() {
    let ticks = 300;

    let base_cfg = TransitionConfig::default();
    let fast_transition_cfg = TransitionConfig {
        sedentarism_population_threshold: 90,
        sedentarism_surplus_threshold: 0.18,
        agriculture_population_threshold: 500,
        agriculture_surplus_threshold: 0.35,
        ..TransitionConfig::default()
    };
    let fragile_cfg = TransitionConfig {
        regression_ecological_pressure_threshold: 0.72,
        regression_surplus_threshold: 0.28,
        ..TransitionConfig::default()
    };

    let scenarios: [(&str, Vec<LocalSocietyState>, TransitionConfig); 6] = [
        (
            "baseline/default",
            scenario_local_emergence_baseline(),
            base_cfg,
        ),
        (
            "baseline/fast-transition",
            scenario_local_emergence_baseline(),
            fast_transition_cfg,
        ),
        ("eco-stress/default", scenario_ecological_stress(), base_cfg),
        (
            "eco-stress/fragile",
            scenario_ecological_stress(),
            fragile_cfg,
        ),
        (
            "dense-coupled/default",
            scenario_dense_coupled_growth(),
            base_cfg,
        ),
        (
            "fragmented-low-coupling/default",
            scenario_fragmented_low_coupling(),
            base_cfg,
        ),
    ];

    print_header();
    for (name, societies, cfg) in scenarios {
        let result = run_named_scenario(name, societies, ticks, cfg);
        let class = classify_trajectory(result.summary);
        print_row(name, &result, class);
    }
}
