use crate::calibration::{
    calibration_confidence, objective, score, simulate_series, CalibrationConfidence,
    CalibrationParameters, CalibrationWeights, CanonicalBenchmarks, StylizedTargets,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EnsembleConfig {
    pub seeds: usize,
    pub perturbations_per_seed: usize,
    pub ticks: usize,
    pub start_year: i32,
    pub perturbation_scale: f64,
    pub turning_window_years: i32,
    pub weights: CalibrationWeights,
}

impl Default for EnsembleConfig {
    fn default() -> Self {
        Self {
            seeds: 8,
            perturbations_per_seed: 4,
            ticks: 200,
            start_year: 1800,
            perturbation_scale: 0.12,
            turning_window_years: 20,
            weights: CalibrationWeights::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnsembleTrajectoryPoint {
    pub tick: usize,
    pub year: i32,
    pub superorganism_p10: f64,
    pub superorganism_p50: f64,
    pub superorganism_p90: f64,
    pub complexity_p10: f64,
    pub complexity_p50: f64,
    pub complexity_p90: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnsembleSummary {
    pub runs: usize,
    pub confidence: CalibrationConfidence,
    pub robustness_score: f64,
    pub trajectories: Vec<EnsembleTrajectoryPoint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationReport {
    pub confidence: CalibrationConfidence,
    pub robustness_score: f64,
    pub fit_population: f64,
    pub fit_urbanization: f64,
    pub fit_gdp_per_capita: f64,
    pub fit_energy: f64,
}

#[must_use]
pub fn run_ensemble(
    base_params: CalibrationParameters,
    targets: &StylizedTargets,
    benchmarks: &CanonicalBenchmarks,
    cfg: EnsembleConfig,
) -> EnsembleSummary {
    let mut rng = 17_u64;
    let run_count = cfg.seeds.saturating_mul(cfg.perturbations_per_seed).max(1);
    let mut all_superorganism: Vec<Vec<f64>> = Vec::with_capacity(run_count);
    let mut all_complexity: Vec<Vec<f64>> = Vec::with_capacity(run_count);
    let mut stylized_success = 0_u32;

    for seed_idx in 0..cfg.seeds.max(1) {
        for _ in 0..cfg.perturbations_per_seed.max(1) {
            let params = perturb_params(base_params, cfg.perturbation_scale, &mut rng);
            let seed = ((seed_idx as u64) + 1)
                .saturating_mul(997)
                .saturating_add(lcg_next(&mut rng));
            let model = simulate_series(params, seed, cfg.ticks, cfg.start_year);
            let cmp = objective(&model, targets, cfg.turning_window_years, cfg.weights);
            let fit = score(&cmp);
            if fit <= 2.50 {
                stylized_success = stylized_success.saturating_add(1);
            }

            all_superorganism.push(model.energy.clone());
            all_complexity.push(model.urbanization.clone());
        }
    }

    let mut trajectories = Vec::with_capacity(cfg.ticks);
    for tick in 0..cfg.ticks {
        let mut so_slice = all_superorganism
            .iter()
            .map(|series| *series.get(tick).unwrap_or(&0.0))
            .collect::<Vec<f64>>();
        let mut cx_slice = all_complexity
            .iter()
            .map(|series| *series.get(tick).unwrap_or(&0.0))
            .collect::<Vec<f64>>();
        so_slice.sort_by(f64::total_cmp);
        cx_slice.sort_by(f64::total_cmp);

        trajectories.push(EnsembleTrajectoryPoint {
            tick,
            year: cfg.start_year + (tick as i32),
            superorganism_p10: percentile(&so_slice, 0.10),
            superorganism_p50: percentile(&so_slice, 0.50),
            superorganism_p90: percentile(&so_slice, 0.90),
            complexity_p10: percentile(&cx_slice, 0.10),
            complexity_p50: percentile(&cx_slice, 0.50),
            complexity_p90: percentile(&cx_slice, 0.90),
        });
    }

    let robustness_score = (stylized_success as f64) / (run_count as f64);
    let confidence = if robustness_score > 0.8 {
        calibration_confidence(0.5)
    } else if robustness_score > 0.4 {
        calibration_confidence(2.0)
    } else {
        calibration_confidence(3.0)
    };

    // keep anchors in API for future global calibration reports
    let _benchmark_len = benchmarks.population.years.len();

    EnsembleSummary {
        runs: run_count,
        confidence,
        robustness_score,
        trajectories,
    }
}

#[must_use]
pub fn validation_report(
    base_params: CalibrationParameters,
    targets: &StylizedTargets,
    benchmarks: &CanonicalBenchmarks,
    cfg: EnsembleConfig,
) -> ValidationReport {
    let summary = run_ensemble(base_params, targets, benchmarks, cfg.clone());
    let model = simulate_series(base_params, 123, cfg.ticks, cfg.start_year);
    let cmp = objective(&model, targets, cfg.turning_window_years, cfg.weights);

    ValidationReport {
        confidence: summary.confidence,
        robustness_score: summary.robustness_score,
        fit_population: cmp.population.weighted_error,
        fit_urbanization: cmp.urbanization.weighted_error,
        fit_gdp_per_capita: cmp.gdp_per_capita.weighted_error,
        fit_energy: cmp.energy.weighted_error,
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let clamped = p.clamp(0.0, 1.0);
    let idx = ((sorted.len() - 1) as f64 * clamped).round() as usize;
    sorted[idx]
}

fn perturb_params(
    base: CalibrationParameters,
    scale: f64,
    state: &mut u64,
) -> CalibrationParameters {
    let span = scale.abs();
    let jitter = |v: f64, state: &mut u64| -> f64 {
        let d = (rand01(state) * 2.0 - 1.0) * span;
        (v * (1.0 + d)).max(0.01)
    };

    CalibrationParameters {
        cooperation_weight: jitter(base.cooperation_weight, state),
        conflict_weight: jitter(base.conflict_weight, state),
        trade_weight: jitter(base.trade_weight, state),
        migration_weight: jitter(base.migration_weight, state),
        ecological_feedback: jitter(base.ecological_feedback, state),
        sedentarism_population_threshold: jitter(base.sedentarism_population_threshold, state),
        agriculture_population_threshold: jitter(base.agriculture_population_threshold, state),
        regression_ecological_pressure_threshold: jitter(
            base.regression_ecological_pressure_threshold,
            state,
        )
        .clamp(0.55, 0.98),
    }
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

fn rand01(state: &mut u64) -> f64 {
    (lcg_next(state) as f64) / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::calibration::{
        baseline_parameters, ingest_owid_csv, stylized_targets, CalibrationWeights,
    };

    use super::{run_ensemble, validation_report, EnsembleConfig};

    fn fixture(path: &str) {
        fs::write(
            path,
            "year,population,urbanization,gdp_per_capita,primary_energy_consumption\n1800,1,0.05,0.2,0.1\n1850,2,0.06,0.25,0.13\n1900,3,0.08,0.30,0.20\n1950,4,0.12,0.45,0.35\n2000,5,0.20,0.60,0.50\n",
        )
        .unwrap_or_else(|e| panic!("fixture write should succeed: {e}"));
    }

    #[test]
    fn ensemble_percentiles_are_monotonic() {
        let path = "/tmp/walrus_ensemble_fixture.csv";
        fixture(path);
        let data = ingest_owid_csv(path).unwrap_or_else(|e| panic!("fixture should parse: {e:?}"));
        let targets = stylized_targets(&data);
        let summary = run_ensemble(
            baseline_parameters(),
            &targets,
            &data,
            EnsembleConfig {
                seeds: 3,
                perturbations_per_seed: 3,
                ticks: 40,
                start_year: 1800,
                perturbation_scale: 0.1,
                turning_window_years: 25,
                weights: CalibrationWeights::default(),
            },
        );

        assert!(!summary.trajectories.is_empty());
        for point in &summary.trajectories {
            assert!(point.superorganism_p10 <= point.superorganism_p50);
            assert!(point.superorganism_p50 <= point.superorganism_p90);
            assert!(point.complexity_p10 <= point.complexity_p50);
            assert!(point.complexity_p50 <= point.complexity_p90);
        }
    }

    #[test]
    fn robustness_score_is_in_valid_range() {
        let path = "/tmp/walrus_ensemble_fixture_2.csv";
        fixture(path);
        let data = ingest_owid_csv(path).unwrap_or_else(|e| panic!("fixture should parse: {e:?}"));
        let targets = stylized_targets(&data);
        let report = validation_report(
            baseline_parameters(),
            &targets,
            &data,
            EnsembleConfig {
                seeds: 2,
                perturbations_per_seed: 2,
                ticks: 30,
                ..EnsembleConfig::default()
            },
        );

        assert!((0.0..=1.0).contains(&report.robustness_score));
        assert!(report.fit_population.is_finite());
    }
}
