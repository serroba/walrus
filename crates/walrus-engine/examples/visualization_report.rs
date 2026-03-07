use std::fs::{self, File};
use std::io::{BufWriter, Write};

use walrus_engine::{
    classify_trajectory, run_emergence_simulation, scenario_dense_coupled_growth,
    scenario_ecological_stress, scenario_fragmented_low_coupling,
    scenario_local_emergence_baseline, summarize_emergence, EmergenceSnapshot, TrajectoryClass,
    TransitionConfig,
};

#[derive(Clone, Copy)]
struct ScenarioSpec {
    name: &'static str,
    builder: fn() -> Vec<walrus_engine::LocalSocietyState>,
    cfg: TransitionConfig,
}

fn trajectory_label(class: TrajectoryClass) -> &'static str {
    match class {
        TrajectoryClass::StabilizingComplexity => "Stabilizing complexity",
        TrajectoryClass::OvershootAndCorrection => "Overshoot and correction",
        TrajectoryClass::FragileTransition => "Fragile transition",
        TrajectoryClass::StagnantLowComplexity => "Stagnant low complexity",
    }
}

fn trajectory_explainer(class: TrajectoryClass) -> &'static str {
    match class {
        TrajectoryClass::StabilizingComplexity => {
            "Complex coordination emerges and remains relatively stable over time."
        }
        TrajectoryClass::OvershootAndCorrection => {
            "The system organizes quickly, peaks, then gives up complexity under pressure."
        }
        TrajectoryClass::FragileTransition => {
            "Some emergence occurs, but institutions remain unstable and prone to reversal."
        }
        TrajectoryClass::StagnantLowComplexity => {
            "Coordination stays local and fragmented; large-scale complexity does not consolidate."
        }
    }
}

fn write_csv(path: &str, snapshots: &[EmergenceSnapshot]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "tick,superorganism_index,mean_local_complexity,hunter_gatherer_count,sedentary_count,agriculture_count"
    )?;

    for snapshot in snapshots {
        writeln!(
            writer,
            "{},{:.6},{:.6},{},{},{}",
            snapshot.tick,
            snapshot.global.superorganism_index,
            snapshot.mean_local_complexity,
            snapshot.hunter_gatherer_count,
            snapshot.sedentary_count,
            snapshot.agriculture_count
        )?;
    }

    writer.flush()?;
    Ok(())
}

fn safe_name(name: &str) -> String {
    name.replace('/', "_")
}

fn main() -> std::io::Result<()> {
    let output_dir = "outputs/latest";
    fs::create_dir_all(output_dir)?;

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

    let scenarios = vec![
        ScenarioSpec {
            name: "baseline/default",
            builder: scenario_local_emergence_baseline,
            cfg: base_cfg,
        },
        ScenarioSpec {
            name: "baseline/fast-transition",
            builder: scenario_local_emergence_baseline,
            cfg: fast_transition_cfg,
        },
        ScenarioSpec {
            name: "eco-stress/default",
            builder: scenario_ecological_stress,
            cfg: base_cfg,
        },
        ScenarioSpec {
            name: "eco-stress/fragile",
            builder: scenario_ecological_stress,
            cfg: fragile_cfg,
        },
        ScenarioSpec {
            name: "dense-coupled/default",
            builder: scenario_dense_coupled_growth,
            cfg: base_cfg,
        },
        ScenarioSpec {
            name: "fragmented-low-coupling/default",
            builder: scenario_fragmented_low_coupling,
            cfg: base_cfg,
        },
    ];

    let report_path = format!("{output_dir}/report.md");
    let report_file = File::create(&report_path)?;
    let mut report = BufWriter::new(report_file);

    writeln!(report, "# Walrus Simulation Report")?;
    writeln!(report)?;
    writeln!(
        report,
        "This report is designed for non-technical readers: each scenario gives a plain-language behavior label plus key metrics."
    )?;
    writeln!(report)?;
    writeln!(
        report,
        "| Scenario | Behavior | Start SO | Peak SO | End SO | Start CX | Peak CX | End CX | Final Modes (H/S/A) |"
    )?;
    writeln!(report, "|---|---|---:|---:|---:|---:|---:|---:|---:|")?;

    let mut scenario_runs: Vec<(String, TrajectoryClass)> = Vec::new();

    for spec in &scenarios {
        let snapshots = run_emergence_simulation((spec.builder)(), 300, spec.cfg);
        let summary = summarize_emergence(&snapshots);
        let class = classify_trajectory(summary);
        let final_snapshot = snapshots[snapshots.len() - 1];

        let csv_name = format!("{output_dir}/timeline_{}.csv", safe_name(spec.name));
        write_csv(&csv_name, &snapshots)?;

        writeln!(
            report,
            "| {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {}/{}/{} |",
            spec.name,
            trajectory_label(class),
            summary.start_superorganism,
            summary.peak_superorganism,
            summary.end_superorganism,
            summary.start_mean_complexity,
            summary.peak_mean_complexity,
            summary.end_mean_complexity,
            final_snapshot.hunter_gatherer_count,
            final_snapshot.sedentary_count,
            final_snapshot.agriculture_count,
        )?;
        scenario_runs.push((spec.name.to_string(), class));
    }

    for (name, class) in scenario_runs {
        writeln!(report)?;
        writeln!(report, "## {}", name)?;
        writeln!(report)?;
        writeln!(report, "Behavior: **{}**", trajectory_label(class))?;
        writeln!(report)?;
        writeln!(report, "{}", trajectory_explainer(class))?;
        writeln!(report)?;
        writeln!(
            report,
            "Data: `outputs/latest/timeline_{}.csv`",
            safe_name(&name)
        )?;
        writeln!(report)?;
    }

    report.flush()?;

    println!("Wrote report: {report_path}");
    println!("Wrote scenario timelines: {output_dir}/timeline_*.csv");

    Ok(())
}
