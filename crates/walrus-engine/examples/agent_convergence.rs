use std::env;

use walrus_engine::agents::{
    default_agent_experiment_conditions, run_agent_convergence_experiment,
};

fn kinship_name(code: u8) -> &'static str {
    match code {
        0 => "patrilineal",
        1 => "matrilineal",
        _ => "bilateral",
    }
}

fn marriage_name(code: u8) -> &'static str {
    match code {
        0 => "monogamy",
        1 => "polygyny",
        _ => "polyandry",
    }
}

fn institution_name(code: u8) -> &'static str {
    match code {
        0 => "band",
        1 => "tribe",
        2 => "chiefdom",
        3 => "state",
        _ => "unknown",
    }
}

fn main() {
    let seeds: u32 = env::var("SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let ticks: u32 = env::var("TICKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let threshold: f32 = env::var("THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.35);
    let sustained: u32 = env::var("SUSTAINED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let conditions = default_agent_experiment_conditions();

    eprintln!("Agent Convergence Experiment (Phase 6: Superorganism Question)");
    eprintln!(
        "  {} conditions x {} seeds = {} total runs",
        conditions.len(),
        seeds,
        conditions.len() as u32 * seeds
    );
    eprintln!(
        "  ticks={}, threshold={:.2}, sustained={}",
        ticks, threshold, sustained
    );
    eprintln!();

    let result = run_agent_convergence_experiment(&conditions, seeds, ticks, threshold, sustained);

    // Per-condition summary table
    println!("condition,runs,arrival_rate,mean_peak_so,mean_final_so,median_time,mean_collapses,mean_final_pop,patrilineal,matrilineal,bilateral,monogamy,polygyny,polyandry,band,tribe,chiefdom,state");
    for s in &result.condition_summaries {
        let median_str = s
            .median_time_to_sustained
            .map_or("-".to_string(), |t| t.to_string());
        println!(
            "{},{},{:.1}%,{:.3},{:.3},{},{:.1},{:.0},{},{},{},{},{},{},{},{},{},{}",
            s.label,
            s.runs,
            s.arrival_rate * 100.0,
            s.mean_peak_index,
            s.mean_final_index,
            median_str,
            s.mean_collapses,
            s.mean_final_population,
            s.kinship_distribution[0],
            s.kinship_distribution[1],
            s.kinship_distribution[2],
            s.marriage_distribution[0],
            s.marriage_distribution[1],
            s.marriage_distribution[2],
            s.institution_distribution[0],
            s.institution_distribution[1],
            s.institution_distribution[2],
            s.institution_distribution[3],
        );
    }

    // Overall summary
    eprintln!(
        "\nOverall arrival rate: {:.1}% ({}/{} runs reached sustained superorganism)",
        result.overall_arrival_rate * 100.0,
        result
            .all_analyses
            .iter()
            .filter(|(_, a)| a.reached_sustained)
            .count(),
        result.all_analyses.len(),
    );

    // Distribution of peak superorganism index
    let mut peaks: Vec<f32> = result
        .all_analyses
        .iter()
        .map(|(_, a)| a.peak_index)
        .collect();
    peaks.sort_by(f32::total_cmp);
    if !peaks.is_empty() {
        eprintln!("\nPeak superorganism index distribution:");
        eprintln!(
            "  min={:.3}, p25={:.3}, median={:.3}, p75={:.3}, max={:.3}",
            peaks[0],
            peaks[peaks.len() / 4],
            peaks[peaks.len() / 2],
            peaks[3 * peaks.len() / 4],
            peaks[peaks.len() - 1],
        );
    }

    // Time-to-sustained distribution
    let mut times: Vec<u32> = result
        .all_analyses
        .iter()
        .filter_map(|(_, a)| a.time_to_sustained)
        .collect();
    times.sort_unstable();
    if !times.is_empty() {
        eprintln!("\nTime-to-sustained distribution (ticks):");
        eprintln!(
            "  min={}, p25={}, median={}, p75={}, max={}",
            times[0],
            times[times.len() / 4],
            times[times.len() / 2],
            times[3 * times.len() / 4],
            times[times.len() - 1],
        );
    }

    // Questions analysis
    eprintln!("\n--- Key Questions ---");

    // Q1: Does hierarchy predict collapse?
    let hierarchical_runs: Vec<&(String, _)> = result
        .all_analyses
        .iter()
        .filter(|(_, a)| a.final_institution >= 2)
        .collect();
    let flat_runs: Vec<&(String, _)> = result
        .all_analyses
        .iter()
        .filter(|(_, a)| a.final_institution < 2)
        .collect();
    if !hierarchical_runs.is_empty() && !flat_runs.is_empty() {
        let hier_collapses: f32 = hierarchical_runs
            .iter()
            .map(|(_, a)| a.collapses as f32)
            .sum::<f32>()
            / hierarchical_runs.len() as f32;
        let flat_collapses: f32 = flat_runs
            .iter()
            .map(|(_, a)| a.collapses as f32)
            .sum::<f32>()
            / flat_runs.len() as f32;
        eprintln!(
            "Hierarchy & collapse: hierarchical={:.1} collapses/run, flat={:.1} collapses/run",
            hier_collapses, flat_collapses
        );
    }

    // Q2: Kinship system distribution by energy condition
    for cond_label in ["rich_energy", "scarce_energy", "baseline"] {
        let cond_runs: Vec<&(String, _)> = result
            .all_analyses
            .iter()
            .filter(|(l, _)| l == cond_label)
            .collect();
        if !cond_runs.is_empty() {
            let mut kin_counts = [0_u32; 3];
            for (_, a) in &cond_runs {
                if (a.final_kinship as usize) < 3 {
                    kin_counts[a.final_kinship as usize] += 1;
                }
            }
            eprintln!(
                "{}: kinship=[{}={}, {}={}, {}={}]",
                cond_label,
                kinship_name(0),
                kin_counts[0],
                kinship_name(1),
                kin_counts[1],
                kinship_name(2),
                kin_counts[2],
            );
        }
    }

    // Q3: Does fossil access drive superorganism?
    let rich_arrived = result
        .condition_summaries
        .iter()
        .find(|s| s.label == "rich_energy")
        .map(|s| s.arrival_rate)
        .unwrap_or(0.0);
    let scarce_arrived = result
        .condition_summaries
        .iter()
        .find(|s| s.label == "scarce_energy")
        .map(|s| s.arrival_rate)
        .unwrap_or(0.0);
    eprintln!(
        "Energy & superorganism: rich_energy arrival={:.0}%, scarce_energy arrival={:.0}%",
        rich_arrived * 100.0,
        scarce_arrived * 100.0,
    );

    // Per-run detail CSV (to stderr for easy separation)
    eprintln!("\n--- Per-run details (CSV) ---");
    eprintln!("condition,seed,peak_so,final_so,reached,time_to,collapses,final_pop,institution,kinship,marriage,cultural_div");
    for (i, (label, a)) in result.all_analyses.iter().enumerate() {
        eprintln!(
            "{},{},{:.3},{:.3},{},{},{},{},{},{},{},{:.3}",
            label,
            i,
            a.peak_index,
            a.final_index,
            a.reached_sustained,
            a.time_to_sustained
                .map_or("-".to_string(), |t| t.to_string()),
            a.collapses,
            a.final_population,
            institution_name(a.final_institution),
            kinship_name(a.final_kinship),
            marriage_name(a.final_marriage),
            a.final_cultural_diversity,
        );
    }
}
