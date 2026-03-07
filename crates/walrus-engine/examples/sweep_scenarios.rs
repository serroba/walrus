use walrus_engine::{
    run_named_scenario, scenario_ecological_stress, scenario_local_emergence_baseline,
    LocalSocietyState, SubsistenceMode, TransitionConfig,
};

fn dense_coupled_growth() -> Vec<LocalSocietyState> {
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

fn fragmented_low_coupling() -> Vec<LocalSocietyState> {
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

fn print_header() {
    println!(
        "scenario,start_SO,peak_SO,end_SO,start_CX,peak_CX,end_CX,peak_complex_societies,final_modes(H/S/A)",
    );
}

fn print_row(name: &str, result: &walrus_engine::NamedSummary) {
    println!(
        "{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{}/{}/{}",
        name,
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
        ("dense-coupled/default", dense_coupled_growth(), base_cfg),
        (
            "fragmented-low-coupling/default",
            fragmented_low_coupling(),
            base_cfg,
        ),
    ];

    print_header();
    for (name, societies, cfg) in scenarios {
        let result = run_named_scenario(name, societies, ticks, cfg);
        print_row(name, &result);
    }
}
