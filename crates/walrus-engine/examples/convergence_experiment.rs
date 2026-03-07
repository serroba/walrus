use walrus_engine::evolution::{
    default_experiment_conditions, run_convergence_experiment, ConvergenceExperimentConfig,
    DunbarBehaviorModel,
};

fn main() {
    let cfg = ConvergenceExperimentConfig {
        conditions: default_experiment_conditions(),
        seeds_per_condition: 24,
        generations: 400,
        superorganism_threshold: 0.45,
        sustained_generations: 10,
        nk_n: 14,
        nk_k: 3,
        dunbar_model: DunbarBehaviorModel::default(),
    };

    println!(
        "Running convergence experiment: {} conditions x {} seeds = {} total runs",
        cfg.conditions.len(),
        cfg.seeds_per_condition,
        cfg.conditions.len() as u32 * cfg.seeds_per_condition
    );
    println!(
        "Superorganism threshold: {:.2}, sustained for {} generations\n",
        cfg.superorganism_threshold, cfg.sustained_generations
    );

    let result = run_convergence_experiment(&cfg);

    // Per-condition summary table
    println!("condition,runs,arrival_rate,median_time,mean_peak_so,mean_final_so,mean_complexity,mean_collapses");
    for summary in &result.condition_summaries {
        let median_str = summary
            .median_time_to_superorganism
            .map_or("-".to_string(), |t| t.to_string());
        println!(
            "{},{},{:.1}%,{},{:.3},{:.3},{:.3},{:.1}",
            summary.label,
            summary.runs,
            summary.arrival_rate * 100.0,
            median_str,
            summary.mean_peak_superorganism,
            summary.mean_final_superorganism,
            summary.mean_final_complexity,
            summary.mean_collapses,
        );
    }

    println!(
        "\nOverall arrival rate: {:.1}% ({}/{} runs reached sustained superorganism)",
        result.overall_arrival_rate * 100.0,
        result
            .outcomes
            .iter()
            .filter(|o| o.reached_superorganism)
            .count(),
        result.outcomes.len(),
    );

    // Histogram of time-to-superorganism for runs that arrived
    let mut times: Vec<u32> = result
        .outcomes
        .iter()
        .filter_map(|o| o.time_to_superorganism)
        .collect();
    times.sort_unstable();
    if !times.is_empty() {
        println!("\nTime-to-superorganism distribution (generations):");
        println!(
            "  min={}, p25={}, median={}, p75={}, max={}",
            times[0],
            times[times.len() / 4],
            times[times.len() / 2],
            times[3 * times.len() / 4],
            times[times.len() - 1],
        );
    }

    // Peak superorganism distribution across all runs
    let mut peaks: Vec<f64> = result
        .outcomes
        .iter()
        .map(|o| o.peak_superorganism_index)
        .collect();
    peaks.sort_by(f64::total_cmp);
    if !peaks.is_empty() {
        println!("\nPeak superorganism index distribution:");
        println!(
            "  min={:.3}, p25={:.3}, median={:.3}, p75={:.3}, max={:.3}",
            peaks[0],
            peaks[peaks.len() / 4],
            peaks[peaks.len() / 2],
            peaks[3 * peaks.len() / 4],
            peaks[peaks.len() - 1],
        );
    }
}
