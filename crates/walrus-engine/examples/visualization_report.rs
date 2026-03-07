use std::fs::{self, File};
use std::io::{BufWriter, Write};

use walrus_engine::calibration::{
    baseline_parameters, calibration_confidence, default_parameter_bounds, ingest_owid_or_maddison,
    run_calibration, stylized_targets, CalibrationConfig,
};
use walrus_engine::ensemble::{run_ensemble, validation_report, EnsembleConfig};

fn confidence_label(level: walrus_engine::calibration::CalibrationConfidence) -> &'static str {
    match level {
        walrus_engine::calibration::CalibrationConfidence::Exploratory => "exploratory",
        walrus_engine::calibration::CalibrationConfidence::CalibratedStylized => {
            "calibrated-stylized"
        }
        walrus_engine::calibration::CalibrationConfidence::CalibratedCurveFit => {
            "calibrated-curve-fit"
        }
    }
}

fn main() -> std::io::Result<()> {
    let output_dir = "outputs/latest";
    fs::create_dir_all(output_dir)?;

    let data_path = "data/benchmarks/owid_maddison_anchor.csv";
    let benchmarks = ingest_owid_or_maddison(data_path)
        .unwrap_or_else(|e| panic!("failed to load benchmark data at {data_path}: {e:?}"));
    let targets = stylized_targets(&benchmarks);

    let exploratory_params = baseline_parameters();
    let exploratory_report = validation_report(
        exploratory_params,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );

    let artifact = run_calibration(
        &benchmarks,
        CalibrationConfig {
            seed: 13,
            iterations: 220,
            ticks: 180,
            ..CalibrationConfig::default()
        },
        default_parameter_bounds(),
    );
    let calibrated_report = validation_report(
        artifact.parameters,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );
    let calibrated_ensemble = run_ensemble(
        artifact.parameters,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );

    let report_path = format!("{output_dir}/report.md");
    let report_file = File::create(&report_path)?;
    let mut report = BufWriter::new(report_file);

    writeln!(report, "# Walrus Validation Report")?;
    writeln!(report)?;
    writeln!(
        report,
        "Data anchor: `{}` (OWID+Maddison compatible schema)",
        data_path
    )?;
    writeln!(report)?;
    writeln!(report, "## Calibration Status")?;
    writeln!(report)?;
    writeln!(
        report,
        "- baseline objective: {:.4}",
        artifact.baseline_objective
    )?;
    writeln!(report, "- best objective: {:.4}", artifact.best_objective)?;
    writeln!(
        report,
        "- status: {}",
        confidence_label(calibration_confidence(artifact.best_objective))
    )?;
    writeln!(report)?;

    writeln!(report, "## Scenario Confidence")?;
    writeln!(report)?;
    writeln!(
        report,
        "| Scenario | Confidence | Robustness | Pop fit | Urban fit | GDP fit | Energy fit |"
    )?;
    writeln!(report, "|---|---|---:|---:|---:|---:|---:|")?;
    writeln!(
        report,
        "| exploratory baseline | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |",
        confidence_label(exploratory_report.confidence),
        exploratory_report.robustness_score,
        exploratory_report.fit_population,
        exploratory_report.fit_urbanization,
        exploratory_report.fit_gdp_per_capita,
        exploratory_report.fit_energy,
    )?;
    writeln!(
        report,
        "| calibrated stylized | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |",
        confidence_label(calibrated_report.confidence),
        calibrated_report.robustness_score,
        calibrated_report.fit_population,
        calibrated_report.fit_urbanization,
        calibrated_report.fit_gdp_per_capita,
        calibrated_report.fit_energy,
    )?;
    writeln!(report)?;

    writeln!(report, "## Ensemble Uncertainty")?;
    writeln!(report)?;
    writeln!(report, "- runs: {}", calibrated_ensemble.runs)?;
    writeln!(
        report,
        "- robustness score: {:.3}",
        calibrated_ensemble.robustness_score
    )?;
    if let (Some(first), Some(last)) = (
        calibrated_ensemble.trajectories.first(),
        calibrated_ensemble.trajectories.last(),
    ) {
        writeln!(
            report,
            "- SO p50 change: {:.3} -> {:.3}",
            first.superorganism_p50, last.superorganism_p50
        )?;
        writeln!(
            report,
            "- Complexity p50 change: {:.3} -> {:.3}",
            first.complexity_p50, last.complexity_p50
        )?;
    }
    writeln!(report)?;

    writeln!(report, "## Notes")?;
    writeln!(report)?;
    writeln!(
        report,
        "- Objective prioritizes stylized directions and turning windows, not exact curve fit."
    )?;
    writeln!(
        report,
        "- Confidence labels indicate model maturity and uncertainty posture."
    )?;
    writeln!(
        report,
        "- Outputs are descriptive only and avoid normative claims."
    )?;

    report.flush()?;

    println!("Wrote report: {report_path}");
    println!("Use `make viz-app` for the interactive uncertainty viewer.");

    Ok(())
}
