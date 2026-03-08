use std::collections::BTreeMap;
use std::fs;

use crate::{
    emergence_from_projection, macro_from_agents, micro_macro_projection, next_subsistence_mode,
    seed_agent_based_society_with_topology, step_agent_based_society, InteractionParameters,
    InteractionTopology, SubsistenceMode, TransitionConfig,
};

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkSeries {
    pub name: String,
    pub years: Vec<i32>,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalBenchmarks {
    pub population: BenchmarkSeries,
    pub urbanization: BenchmarkSeries,
    pub gdp_per_capita: BenchmarkSeries,
    pub energy: BenchmarkSeries,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StylizedSeriesTarget {
    pub name: String,
    pub direction: TrendDirection,
    pub turning_points: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StylizedTargets {
    pub population: StylizedSeriesTarget,
    pub urbanization: StylizedSeriesTarget,
    pub gdp_per_capita: StylizedSeriesTarget,
    pub energy: StylizedSeriesTarget,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationParameters {
    pub cooperation_weight: f64,
    pub conflict_weight: f64,
    pub trade_weight: f64,
    pub migration_weight: f64,
    pub ecological_feedback: f64,
    pub sedentarism_population_threshold: f64,
    pub agriculture_population_threshold: f64,
    pub regression_ecological_pressure_threshold: f64,
}

impl CalibrationParameters {
    #[must_use]
    pub fn to_interaction_parameters(self) -> InteractionParameters {
        InteractionParameters {
            cooperation_weight: self.cooperation_weight,
            conflict_weight: self.conflict_weight,
            trade_weight: self.trade_weight,
            migration_weight: self.migration_weight,
            ecological_feedback: self.ecological_feedback,
        }
    }

    #[must_use]
    pub fn to_transition_config(self) -> TransitionConfig {
        TransitionConfig {
            sedentarism_population_threshold: self.sedentarism_population_threshold.round() as u32,
            sedentarism_surplus_threshold: 0.22,
            agriculture_population_threshold: self.agriculture_population_threshold.round() as u32,
            agriculture_surplus_threshold: 0.42,
            regression_ecological_pressure_threshold: self.regression_ecological_pressure_threshold,
            regression_surplus_threshold: 0.20,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBounds {
    pub cooperation_weight: (f64, f64),
    pub conflict_weight: (f64, f64),
    pub trade_weight: (f64, f64),
    pub migration_weight: (f64, f64),
    pub ecological_feedback: (f64, f64),
    pub sedentarism_population_threshold: (f64, f64),
    pub agriculture_population_threshold: (f64, f64),
    pub regression_ecological_pressure_threshold: (f64, f64),
}

impl Default for ParameterBounds {
    fn default() -> Self {
        Self {
            cooperation_weight: (0.4, 1.8),
            conflict_weight: (0.4, 1.8),
            trade_weight: (0.3, 1.8),
            migration_weight: (0.1, 1.2),
            ecological_feedback: (0.4, 1.8),
            sedentarism_population_threshold: (70.0, 220.0),
            agriculture_population_threshold: (450.0, 1_600.0),
            regression_ecological_pressure_threshold: (0.65, 0.95),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationConfig {
    pub seed: u64,
    pub iterations: usize,
    pub ticks: usize,
    pub turning_window_years: i32,
    pub weights: CalibrationWeights,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            seed: 41,
            iterations: 120,
            ticks: 240,
            turning_window_years: 20,
            weights: CalibrationWeights::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationWeights {
    pub direction: f64,
    pub turning_points: f64,
}

impl Default for CalibrationWeights {
    fn default() -> Self {
        Self {
            direction: 0.7,
            turning_points: 0.3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationArtifact {
    pub seed: u64,
    pub iterations: usize,
    pub best_objective: f64,
    pub baseline_objective: f64,
    pub parameters: CalibrationParameters,
    pub comparison: BenchmarkComparison,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkComparison {
    pub population: SeriesFitDiagnostic,
    pub urbanization: SeriesFitDiagnostic,
    pub gdp_per_capita: SeriesFitDiagnostic,
    pub energy: SeriesFitDiagnostic,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeriesFitDiagnostic {
    pub name: String,
    pub expected_direction: TrendDirection,
    pub model_direction: TrendDirection,
    pub expected_turning_points: Vec<i32>,
    pub model_turning_points: Vec<i32>,
    pub weighted_error: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationSeries {
    pub years: Vec<i32>,
    pub population: Vec<f64>,
    pub urbanization: Vec<f64>,
    pub gdp_per_capita: Vec<f64>,
    pub energy: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalibrationConfidence {
    Exploratory,
    CalibratedStylized,
    CalibratedCurveFit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataIngestError {
    Io(String),
    MissingColumn(String),
    Parse(String),
    UnsupportedFormat(String),
}

impl From<std::io::Error> for DataIngestError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

#[must_use]
pub fn default_parameter_bounds() -> ParameterBounds {
    ParameterBounds::default()
}

#[must_use]
pub fn baseline_parameters() -> CalibrationParameters {
    CalibrationParameters {
        cooperation_weight: 1.0,
        conflict_weight: 1.0,
        trade_weight: 1.0,
        migration_weight: 0.4,
        ecological_feedback: 1.0,
        sedentarism_population_threshold: 120.0,
        agriculture_population_threshold: 800.0,
        regression_ecological_pressure_threshold: 0.85,
    }
}

pub fn ingest_owid_csv(path: &str) -> Result<CanonicalBenchmarks, DataIngestError> {
    ingest_csv(
        path,
        "year",
        "population",
        "urbanization",
        "gdp_per_capita",
        "primary_energy_consumption",
    )
}

pub fn ingest_maddison_csv(path: &str) -> Result<CanonicalBenchmarks, DataIngestError> {
    ingest_csv(
        path,
        "year",
        "population",
        "urbanization",
        "gdppc",
        "energy_proxy",
    )
}

/// Ingests HANDY-like model exports for macro trend anchoring.
///
/// Expected columns:
/// - `year`
/// - `population`
/// - `resources`
/// - `output_per_capita`
/// - `inequality`
pub fn ingest_handy_csv(path: &str) -> Result<CanonicalBenchmarks, DataIngestError> {
    let base = ingest_csv(
        path,
        "year",
        "population",
        "inequality",
        "output_per_capita",
        "resources",
    )?;

    let transformed_urban = base
        .urbanization
        .values
        .iter()
        .map(|v| (1.0 - *v).clamp(0.0, 1.0))
        .collect::<Vec<f64>>();

    Ok(CanonicalBenchmarks {
        urbanization: BenchmarkSeries {
            name: "coordination_proxy".to_string(),
            years: base.urbanization.years.clone(),
            values: transformed_urban,
        },
        ..base
    })
}

pub fn ingest_owid_or_maddison(path: &str) -> Result<CanonicalBenchmarks, DataIngestError> {
    if path.ends_with(".csv") {
        ingest_owid_csv(path)
            .or_else(|_| ingest_maddison_csv(path))
            .or_else(|_| ingest_handy_csv(path))
    } else if path.ends_with(".parquet") {
        Err(DataIngestError::UnsupportedFormat(
            "parquet ingestion is not yet compiled in this minimal core; export csv first"
                .to_string(),
        ))
    } else {
        Err(DataIngestError::UnsupportedFormat(
            "only .csv and .parquet paths are supported".to_string(),
        ))
    }
}

fn ingest_csv(
    path: &str,
    year_col: &str,
    population_col: &str,
    urbanization_col: &str,
    gdp_col: &str,
    energy_col: &str,
) -> Result<CanonicalBenchmarks, DataIngestError> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| DataIngestError::Parse("missing header".to_string()))?;
    let fields: Vec<&str> = header.split(',').collect();

    let year_idx = column_index(&fields, year_col)?;
    let pop_idx = column_index(&fields, population_col)?;
    let urb_idx = column_index(&fields, urbanization_col)?;
    let gdp_idx = column_index(&fields, gdp_col)?;
    let energy_idx = column_index(&fields, energy_col)?;

    let mut years = Vec::new();
    let mut population = Vec::new();
    let mut urbanization = Vec::new();
    let mut gdp_per_capita = Vec::new();
    let mut energy = Vec::new();

    for line in lines.filter(|line| !line.trim().is_empty()) {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() <= energy_idx {
            continue;
        }

        let year = parse_i32(cols[year_idx])?;
        let pop = parse_f64(cols[pop_idx])?;
        let urb = parse_f64(cols[urb_idx])?;
        let gdp = parse_f64(cols[gdp_idx])?;
        let en = parse_f64(cols[energy_idx])?;

        years.push(year);
        population.push(pop);
        urbanization.push(urb);
        gdp_per_capita.push(gdp);
        energy.push(en);
    }

    if years.is_empty() {
        return Err(DataIngestError::Parse(
            "no valid rows in benchmark file".to_string(),
        ));
    }

    Ok(CanonicalBenchmarks {
        population: BenchmarkSeries {
            name: "population".to_string(),
            years: years.clone(),
            values: normalize(&population),
        },
        urbanization: BenchmarkSeries {
            name: "urbanization".to_string(),
            years: years.clone(),
            values: normalize(&urbanization),
        },
        gdp_per_capita: BenchmarkSeries {
            name: "gdp_per_capita".to_string(),
            years: years.clone(),
            values: normalize(&gdp_per_capita),
        },
        energy: BenchmarkSeries {
            name: "energy".to_string(),
            years,
            values: normalize(&energy),
        },
    })
}

fn column_index(fields: &[&str], wanted: &str) -> Result<usize, DataIngestError> {
    fields
        .iter()
        .position(|field| field.trim() == wanted)
        .ok_or_else(|| DataIngestError::MissingColumn(wanted.to_string()))
}

fn parse_i32(input: &str) -> Result<i32, DataIngestError> {
    input
        .trim()
        .parse::<i32>()
        .map_err(|_| DataIngestError::Parse(format!("invalid integer: {input}")))
}

fn parse_f64(input: &str) -> Result<f64, DataIngestError> {
    input
        .trim()
        .parse::<f64>()
        .map_err(|_| DataIngestError::Parse(format!("invalid number: {input}")))
}

fn normalize(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let min = values
        .iter()
        .copied()
        .fold(f64::INFINITY, |acc, value| acc.min(value));
    let max = values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |acc, value| acc.max(value));
    let span = (max - min).max(1e-9);
    values.iter().map(|value| (value - min) / span).collect()
}

#[must_use]
pub fn stylized_targets(benchmarks: &CanonicalBenchmarks) -> StylizedTargets {
    StylizedTargets {
        population: target_for(&benchmarks.population),
        urbanization: target_for(&benchmarks.urbanization),
        gdp_per_capita: target_for(&benchmarks.gdp_per_capita),
        energy: target_for(&benchmarks.energy),
    }
}

fn target_for(series: &BenchmarkSeries) -> StylizedSeriesTarget {
    StylizedSeriesTarget {
        name: series.name.clone(),
        direction: trend_direction(&series.values),
        turning_points: find_turning_points(&series.years, &series.values),
    }
}

fn trend_direction(values: &[f64]) -> TrendDirection {
    if values.len() < 2 {
        return TrendDirection::Stable;
    }
    let delta = values[values.len() - 1] - values[0];
    if delta > 0.05 {
        TrendDirection::Increasing
    } else if delta < -0.05 {
        TrendDirection::Decreasing
    } else {
        TrendDirection::Stable
    }
}

fn find_turning_points(years: &[i32], values: &[f64]) -> Vec<i32> {
    if values.len() < 5 || years.len() != values.len() {
        return Vec::new();
    }

    let mut points = Vec::new();
    for idx in 2..(values.len() - 2) {
        let left = values[idx] - values[idx - 1];
        let right = values[idx + 1] - values[idx];
        let abrupt = left.abs() > 0.01 || right.abs() > 0.01;
        if abrupt && left.signum() != right.signum() {
            points.push(years[idx]);
        }
    }
    points
}

#[must_use]
pub fn simulate_series(
    params: CalibrationParameters,
    seed: u64,
    ticks: usize,
    start_year: i32,
) -> SimulationSeries {
    let mut society = seed_agent_based_society_with_topology(
        256,
        SubsistenceMode::HunterGatherer,
        0.18,
        0.08,
        InteractionTopology::SmallWorld,
        2,
        seed,
    );
    society.parameters = params.to_interaction_parameters();

    let cfg = params.to_transition_config();
    let mut years = Vec::with_capacity(ticks);
    let mut population = Vec::with_capacity(ticks);
    let mut urbanization = Vec::with_capacity(ticks);
    let mut gdp_per_capita = Vec::with_capacity(ticks);
    let mut energy = Vec::with_capacity(ticks);

    for tick in 0..ticks {
        let interactions = step_agent_based_society(&mut society);
        let macro_state = macro_from_agents(&society);
        let projection = micro_macro_projection(&society);
        let emergence = emergence_from_projection(macro_state, projection);

        years.push(start_year + (tick as i32));
        population.push((macro_state.population as f64) / 1_000.0);
        urbanization
            .push((projection.sedentary_share + projection.agriculture_share).clamp(0.0, 1.0));
        gdp_per_capita.push(
            (macro_state.surplus_per_capita
                * (1.0 + 0.4 * projection.trade_rate - 0.3 * projection.conflict_rate))
                .clamp(0.0, 4.0),
        );
        energy
            .push((emergence.throughput_pressure + 0.5 * interactions.trade_rate).clamp(0.0, 2.0));

        society.mode = next_subsistence_mode(
            society.mode,
            macro_state.population,
            macro_state.surplus_per_capita,
            macro_state.ecological_pressure,
            cfg,
        );
        society.ecological_pressure = (society.ecological_pressure
            + 0.012 * emergence.throughput_pressure
            + 0.010 * interactions.conflict_rate
            - 0.008 * interactions.cooperation_rate)
            .clamp(0.0, 1.0);
        society.network_coupling = (society.network_coupling
            + 0.012 * emergence.coordination_centralization
            - 0.006 * interactions.migration_rate)
            .clamp(0.0, 1.0);
    }

    SimulationSeries {
        years,
        population: normalize(&population),
        urbanization: normalize(&urbanization),
        gdp_per_capita: normalize(&gdp_per_capita),
        energy: normalize(&energy),
    }
}

#[must_use]
pub fn objective(
    model: &SimulationSeries,
    targets: &StylizedTargets,
    turning_window_years: i32,
    weights: CalibrationWeights,
) -> BenchmarkComparison {
    BenchmarkComparison {
        population: series_objective(
            "population",
            &model.years,
            &model.population,
            &targets.population,
            turning_window_years,
            weights,
        ),
        urbanization: series_objective(
            "urbanization",
            &model.years,
            &model.urbanization,
            &targets.urbanization,
            turning_window_years,
            weights,
        ),
        gdp_per_capita: series_objective(
            "gdp_per_capita",
            &model.years,
            &model.gdp_per_capita,
            &targets.gdp_per_capita,
            turning_window_years,
            weights,
        ),
        energy: series_objective(
            "energy",
            &model.years,
            &model.energy,
            &targets.energy,
            turning_window_years,
            weights,
        ),
    }
}

fn series_objective(
    name: &str,
    years: &[i32],
    values: &[f64],
    target: &StylizedSeriesTarget,
    turning_window_years: i32,
    weights: CalibrationWeights,
) -> SeriesFitDiagnostic {
    let model_direction = trend_direction(values);
    let model_turning = find_turning_points(years, values);
    let direction_error = if model_direction == target.direction {
        0.0
    } else {
        1.0
    };
    let turn_error =
        turning_point_miss(&model_turning, &target.turning_points, turning_window_years);
    let weighted_error = weights.direction * direction_error + weights.turning_points * turn_error;

    SeriesFitDiagnostic {
        name: name.to_string(),
        expected_direction: target.direction,
        model_direction,
        expected_turning_points: target.turning_points.clone(),
        model_turning_points: model_turning,
        weighted_error,
    }
}

fn turning_point_miss(model: &[i32], target: &[i32], window: i32) -> f64 {
    if target.is_empty() {
        return if model.is_empty() { 0.0 } else { 0.5 };
    }

    let mut misses = 0_u32;
    for wanted in target {
        let found = model
            .iter()
            .any(|actual| (actual - wanted).abs() <= window.max(1));
        if !found {
            misses = misses.saturating_add(1);
        }
    }
    (misses as f64) / (target.len() as f64)
}

#[must_use]
pub fn score(comparison: &BenchmarkComparison) -> f64 {
    comparison.population.weighted_error
        + comparison.urbanization.weighted_error
        + comparison.gdp_per_capita.weighted_error
        + comparison.energy.weighted_error
}

#[must_use]
pub fn calibration_confidence(best_score: f64) -> CalibrationConfidence {
    if best_score <= 0.60 {
        CalibrationConfidence::CalibratedCurveFit
    } else if best_score <= 2.50 {
        CalibrationConfidence::CalibratedStylized
    } else {
        CalibrationConfidence::Exploratory
    }
}

/// Number of dimensions in the calibration parameter space.
const PARAM_DIM: usize = 8;

fn params_to_vec(p: CalibrationParameters) -> [f64; PARAM_DIM] {
    [
        p.cooperation_weight,
        p.conflict_weight,
        p.trade_weight,
        p.migration_weight,
        p.ecological_feedback,
        p.sedentarism_population_threshold,
        p.agriculture_population_threshold,
        p.regression_ecological_pressure_threshold,
    ]
}

fn vec_to_params(v: &[f64; PARAM_DIM]) -> CalibrationParameters {
    CalibrationParameters {
        cooperation_weight: v[0],
        conflict_weight: v[1],
        trade_weight: v[2],
        migration_weight: v[3],
        ecological_feedback: v[4],
        sedentarism_population_threshold: v[5],
        agriculture_population_threshold: v[6],
        regression_ecological_pressure_threshold: v[7],
    }
}

fn bounds_to_lo_hi(b: &ParameterBounds) -> ([f64; PARAM_DIM], [f64; PARAM_DIM]) {
    let pairs = [
        b.cooperation_weight,
        b.conflict_weight,
        b.trade_weight,
        b.migration_weight,
        b.ecological_feedback,
        b.sedentarism_population_threshold,
        b.agriculture_population_threshold,
        b.regression_ecological_pressure_threshold,
    ];
    let mut lo = [0.0; PARAM_DIM];
    let mut hi = [0.0; PARAM_DIM];
    for (i, (a, z)) in pairs.iter().enumerate() {
        lo[i] = a.min(*z);
        hi[i] = a.max(*z);
    }
    (lo, hi)
}

/// Clamp each element of `v` to the bounds `[lo, hi]`.
fn clamp_to_bounds(v: &mut [f64; PARAM_DIM], lo: &[f64; PARAM_DIM], hi: &[f64; PARAM_DIM]) {
    for i in 0..PARAM_DIM {
        v[i] = v[i].clamp(lo[i], hi[i]);
    }
}

/// Evaluate the objective function for a parameter vector.
fn eval_objective(
    v: &[f64; PARAM_DIM],
    seed: u64,
    ticks: usize,
    start_year: i32,
    targets: &StylizedTargets,
    turning_window_years: i32,
    weights: CalibrationWeights,
) -> (f64, BenchmarkComparison) {
    let params = vec_to_params(v);
    let model = simulate_series(params, seed, ticks, start_year);
    let cmp = objective(&model, targets, turning_window_years, weights);
    (score(&cmp), cmp)
}

/// Calibrate model parameters using the Nelder-Mead downhill simplex algorithm.
///
/// Nelder-Mead is a derivative-free optimizer well-suited for noisy, simulation-based
/// objectives in moderate dimensions. It maintains a simplex of n+1 vertices in
/// n-dimensional space and iteratively improves by reflecting, expanding, contracting,
/// or shrinking the simplex toward lower-objective regions.
///
/// This replaces the previous pure random search, providing substantially faster
/// convergence in the 8-dimensional parameter space.
#[must_use]
pub fn run_calibration(
    benchmarks: &CanonicalBenchmarks,
    config: CalibrationConfig,
    bounds: ParameterBounds,
) -> CalibrationArtifact {
    let targets = stylized_targets(benchmarks);
    let baseline = baseline_parameters();
    let sy = start_year(benchmarks);
    let baseline_model = simulate_series(baseline, config.seed, config.ticks, sy);
    let baseline_cmp = objective(
        &baseline_model,
        &targets,
        config.turning_window_years,
        config.weights,
    );
    let baseline_score = score(&baseline_cmp);

    let (lo, hi) = bounds_to_lo_hi(&bounds);
    let mut rng = config.seed.max(1);

    // --- Build initial simplex: baseline vertex + n random vertices ---
    let n = PARAM_DIM;
    let simplex_size = n + 1;
    let mut simplex: Vec<[f64; PARAM_DIM]> = Vec::with_capacity(simplex_size);
    let mut scores: Vec<f64> = Vec::with_capacity(simplex_size);
    let mut comparisons: Vec<BenchmarkComparison> = Vec::with_capacity(simplex_size);

    // Vertex 0 = baseline
    let baseline_vec = params_to_vec(baseline);
    simplex.push(baseline_vec);
    scores.push(baseline_score);
    comparisons.push(baseline_cmp.clone());

    // Remaining vertices: Latin hypercube–inspired seeding for better initial coverage.
    // Divide each dimension into n strata, shuffle, then jitter within each stratum.
    let mut strata: Vec<Vec<usize>> = Vec::with_capacity(n);
    for _ in 0..n {
        let mut perm: Vec<usize> = (0..n).collect();
        // Fisher-Yates shuffle
        for k in (1..n).rev() {
            let j = (lcg_next(&mut rng) as usize) % (k + 1);
            perm.swap(k, j);
        }
        strata.push(perm);
    }

    for vertex_idx in 0..n {
        let mut v = [0.0; PARAM_DIM];
        for (dim, stratum_col) in strata.iter().enumerate().take(n) {
            let stratum = stratum_col[vertex_idx];
            let base_frac = (stratum as f64) / (n as f64);
            let jitter = rand01(&mut rng) / (n as f64);
            let frac = (base_frac + jitter).clamp(0.0, 1.0);
            v[dim] = lo[dim] + (hi[dim] - lo[dim]) * frac;
        }
        clamp_to_bounds(&mut v, &lo, &hi);
        let seed_i = lcg_next(&mut rng);
        let (s, cmp) = eval_objective(
            &v,
            seed_i,
            config.ticks,
            sy,
            &targets,
            config.turning_window_years,
            config.weights,
        );
        simplex.push(v);
        scores.push(s);
        comparisons.push(cmp);
    }

    // --- Nelder-Mead iteration ---
    // Standard coefficients (Nelder & Mead, 1965)
    let alpha = 1.0; // reflection
    let gamma = 2.0; // expansion
    let rho = 0.5; // contraction
    let sigma = 0.5; // shrink

    // We use `config.iterations` as the evaluation budget.
    // Each NM step costs 1–2 evaluations (reflect ± expand/contract), or n+1 on shrink.
    let mut evals = simplex_size;
    let max_evals = config.iterations.max(simplex_size + 1);

    while evals < max_evals {
        // Sort simplex by score (ascending = best first)
        let mut order: Vec<usize> = (0..simplex_size).collect();
        order.sort_by(|&a, &b| scores[a].total_cmp(&scores[b]));
        let sorted_simplex: Vec<[f64; PARAM_DIM]> = order.iter().map(|&i| simplex[i]).collect();
        let sorted_scores: Vec<f64> = order.iter().map(|&i| scores[i]).collect();
        let sorted_cmps: Vec<BenchmarkComparison> =
            order.iter().map(|&i| comparisons[i].clone()).collect();
        simplex = sorted_simplex;
        scores = sorted_scores;
        comparisons = sorted_cmps;

        let best_score = scores[0];
        let worst_idx = simplex_size - 1;
        let second_worst_idx = simplex_size - 2;

        // Centroid of all vertices except the worst
        let mut centroid = [0.0; PARAM_DIM];
        for vertex in simplex.iter().take(worst_idx) {
            for (d, val) in vertex.iter().enumerate().take(n) {
                centroid[d] += val;
            }
        }
        for c in centroid.iter_mut().take(n) {
            *c /= worst_idx as f64;
        }

        // 1) Reflection
        let mut reflected = [0.0; PARAM_DIM];
        for d in 0..n {
            reflected[d] = centroid[d] + alpha * (centroid[d] - simplex[worst_idx][d]);
        }
        clamp_to_bounds(&mut reflected, &lo, &hi);
        let seed_r = lcg_next(&mut rng);
        let (score_r, cmp_r) = eval_objective(
            &reflected,
            seed_r,
            config.ticks,
            sy,
            &targets,
            config.turning_window_years,
            config.weights,
        );
        evals += 1;

        if score_r < scores[second_worst_idx] && score_r >= best_score {
            // Reflected point is better than second-worst but not best: accept reflection
            simplex[worst_idx] = reflected;
            scores[worst_idx] = score_r;
            comparisons[worst_idx] = cmp_r;
            continue;
        }

        if score_r < best_score {
            // 2) Expansion — try going further in the reflection direction
            let mut expanded = [0.0; PARAM_DIM];
            for d in 0..n {
                expanded[d] = centroid[d] + gamma * (reflected[d] - centroid[d]);
            }
            clamp_to_bounds(&mut expanded, &lo, &hi);
            let seed_e = lcg_next(&mut rng);
            let (score_e, cmp_e) = eval_objective(
                &expanded,
                seed_e,
                config.ticks,
                sy,
                &targets,
                config.turning_window_years,
                config.weights,
            );
            evals += 1;

            if score_e < score_r {
                simplex[worst_idx] = expanded;
                scores[worst_idx] = score_e;
                comparisons[worst_idx] = cmp_e;
            } else {
                simplex[worst_idx] = reflected;
                scores[worst_idx] = score_r;
                comparisons[worst_idx] = cmp_r;
            }
            continue;
        }

        // 3) Contraction — reflected point is worse than second-worst
        let mut contracted = [0.0; PARAM_DIM];
        if score_r < scores[worst_idx] {
            // Outside contraction
            for d in 0..n {
                contracted[d] = centroid[d] + rho * (reflected[d] - centroid[d]);
            }
        } else {
            // Inside contraction
            for d in 0..n {
                contracted[d] = centroid[d] + rho * (simplex[worst_idx][d] - centroid[d]);
            }
        }
        clamp_to_bounds(&mut contracted, &lo, &hi);
        let seed_c = lcg_next(&mut rng);
        let (score_c, cmp_c) = eval_objective(
            &contracted,
            seed_c,
            config.ticks,
            sy,
            &targets,
            config.turning_window_years,
            config.weights,
        );
        evals += 1;

        let accept_contraction =
            (score_r < scores[worst_idx] && score_c <= score_r) || score_c < scores[worst_idx];

        if accept_contraction {
            simplex[worst_idx] = contracted;
            scores[worst_idx] = score_c;
            comparisons[worst_idx] = cmp_c;
            continue;
        }

        // 4) Shrink — contract all vertices toward the best vertex
        let best_vertex = simplex[0];
        for i in 1..simplex_size {
            for d in 0..n {
                simplex[i][d] = best_vertex[d] + sigma * (simplex[i][d] - best_vertex[d]);
            }
            clamp_to_bounds(&mut simplex[i], &lo, &hi);
            let seed_s = lcg_next(&mut rng);
            let (s, cmp) = eval_objective(
                &simplex[i],
                seed_s,
                config.ticks,
                sy,
                &targets,
                config.turning_window_years,
                config.weights,
            );
            scores[i] = s;
            comparisons[i] = cmp;
            evals += 1;
            if evals >= max_evals {
                break;
            }
        }
    }

    // Find the best vertex in the final simplex
    let best_idx = scores
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(i, _)| i)
        .unwrap_or(0);

    let best_params = vec_to_params(&simplex[best_idx]);
    let best_score = scores[best_idx];
    let best_cmp = comparisons[best_idx].clone();

    CalibrationArtifact {
        seed: config.seed,
        iterations: config.iterations,
        best_objective: best_score,
        baseline_objective: baseline_score,
        parameters: best_params,
        comparison: best_cmp,
    }
}

fn start_year(benchmarks: &CanonicalBenchmarks) -> i32 {
    *benchmarks.population.years.first().unwrap_or(&0)
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

fn rand01(state: &mut u64) -> f64 {
    (lcg_next(state) as f64) / (u64::MAX as f64)
}

#[must_use]
pub fn comparison_table(artifact: &CalibrationArtifact) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    out.insert(
        "population_error".to_string(),
        artifact.comparison.population.weighted_error,
    );
    out.insert(
        "urbanization_error".to_string(),
        artifact.comparison.urbanization.weighted_error,
    );
    out.insert(
        "gdp_per_capita_error".to_string(),
        artifact.comparison.gdp_per_capita.weighted_error,
    );
    out.insert(
        "energy_error".to_string(),
        artifact.comparison.energy.weighted_error,
    );
    out.insert("objective".to_string(), artifact.best_objective);
    out.insert(
        "baseline_objective".to_string(),
        artifact.baseline_objective,
    );
    out
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        bounds_to_lo_hi, calibration_confidence, clamp_to_bounds, comparison_table,
        default_parameter_bounds, ingest_handy_csv, ingest_maddison_csv, ingest_owid_csv,
        params_to_vec, run_calibration, score, stylized_targets, vec_to_params,
        CalibrationConfidence, CalibrationConfig, PARAM_DIM,
    };

    fn write_fixture(path: &str, header: &str) {
        let body = [
            "1900,10,0.1,1.0,0.8",
            "1910,11,0.11,1.1,0.9",
            "1920,12,0.14,1.3,1.1",
            "1930,11,0.17,1.2,1.0",
            "1940,13,0.20,1.4,1.2",
            "1950,15,0.24,1.8,1.5",
        ]
        .join("\n");
        fs::write(path, format!("{header}\n{body}\n"))
            .unwrap_or_else(|e| panic!("fixture write should succeed: {e}"));
    }

    #[test]
    fn owid_adapter_parses_expected_schema() {
        let path = "/tmp/walrus_owid_fixture.csv";
        write_fixture(
            path,
            "year,population,urbanization,gdp_per_capita,primary_energy_consumption",
        );

        let data =
            ingest_owid_csv(path).unwrap_or_else(|e| panic!("owid schema should parse: {e:?}"));
        assert_eq!(data.population.years.len(), 6);
        assert_eq!(data.population.values.len(), 6);
        assert!(data.energy.values[5] > data.energy.values[0]);
    }

    #[test]
    fn maddison_adapter_parses_expected_schema() {
        let path = "/tmp/walrus_maddison_fixture.csv";
        write_fixture(path, "year,population,urbanization,gdppc,energy_proxy");

        let data = ingest_maddison_csv(path)
            .unwrap_or_else(|e| panic!("maddison schema should parse: {e:?}"));
        assert_eq!(data.gdp_per_capita.years.len(), 6);
        assert_eq!(data.urbanization.values.len(), 6);
    }

    #[test]
    fn handy_adapter_parses_expected_schema() {
        let path = "/tmp/walrus_handy_fixture.csv";
        write_fixture(
            path,
            "year,population,resources,output_per_capita,inequality",
        );

        let data =
            ingest_handy_csv(path).unwrap_or_else(|e| panic!("handy schema should parse: {e:?}"));
        assert_eq!(data.population.years.len(), 6);
        assert_eq!(data.gdp_per_capita.values.len(), 6);
        assert_eq!(data.energy.values.len(), 6);
    }

    #[test]
    fn calibration_smoke_improves_over_baseline() {
        let path = "/tmp/walrus_owid_fixture_calib.csv";
        write_fixture(
            path,
            "year,population,urbanization,gdp_per_capita,primary_energy_consumption",
        );
        let data = ingest_owid_csv(path).unwrap_or_else(|e| panic!("fixture must parse: {e:?}"));
        let artifact = run_calibration(
            &data,
            CalibrationConfig {
                seed: 7,
                iterations: 64,
                ticks: 80,
                ..CalibrationConfig::default()
            },
            default_parameter_bounds(),
        );

        assert!(artifact.best_objective <= artifact.baseline_objective);
        let table = comparison_table(&artifact);
        assert!(table["objective"] <= table["baseline_objective"]);
    }

    #[test]
    fn stylized_targets_and_confidence_are_computed() {
        let path = "/tmp/walrus_owid_fixture_targets.csv";
        write_fixture(
            path,
            "year,population,urbanization,gdp_per_capita,primary_energy_consumption",
        );
        let data = ingest_owid_csv(path).unwrap_or_else(|e| panic!("fixture must parse: {e:?}"));
        let targets = stylized_targets(&data);
        assert!(!targets.population.turning_points.is_empty());

        let conf_a = calibration_confidence(3.0);
        let conf_b = calibration_confidence(1.4);
        let conf_c = calibration_confidence(0.1);
        assert_eq!(conf_a, CalibrationConfidence::Exploratory);
        assert_eq!(conf_b, CalibrationConfidence::CalibratedStylized);
        assert_eq!(conf_c, CalibrationConfidence::CalibratedCurveFit);

        // sanity that score() consumes diagnostics and stays finite.
        let artifact = run_calibration(
            &data,
            CalibrationConfig {
                seed: 11,
                iterations: 8,
                ticks: 40,
                ..CalibrationConfig::default()
            },
            default_parameter_bounds(),
        );
        assert!(score(&artifact.comparison).is_finite());
    }

    #[test]
    fn params_vec_roundtrip() {
        let original = super::baseline_parameters();
        let v = params_to_vec(original);
        let recovered = vec_to_params(&v);
        assert_eq!(original, recovered);
    }

    #[test]
    fn bounds_conversion_preserves_order() {
        let bounds = default_parameter_bounds();
        let (lo, hi) = bounds_to_lo_hi(&bounds);
        for i in 0..PARAM_DIM {
            assert!(lo[i] <= hi[i], "lo[{i}] > hi[{i}]");
        }
    }

    #[test]
    fn clamp_to_bounds_clamps_correctly() {
        let lo = [0.0; PARAM_DIM];
        let hi = [1.0; PARAM_DIM];
        let mut v = [-0.5; PARAM_DIM];
        clamp_to_bounds(&mut v, &lo, &hi);
        for val in v {
            assert!((val - 0.0).abs() < 1e-9);
        }
        let mut v2 = [2.0; PARAM_DIM];
        clamp_to_bounds(&mut v2, &lo, &hi);
        for val in v2 {
            assert!((val - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn nelder_mead_produces_finite_results() {
        let path = "/tmp/walrus_nm_finite_fixture.csv";
        write_fixture(
            path,
            "year,population,urbanization,gdp_per_capita,primary_energy_consumption",
        );
        let data = ingest_owid_csv(path).unwrap_or_else(|e| panic!("fixture must parse: {e:?}"));
        let artifact = run_calibration(
            &data,
            CalibrationConfig {
                seed: 42,
                iterations: 30,
                ticks: 40,
                ..CalibrationConfig::default()
            },
            default_parameter_bounds(),
        );
        assert!(artifact.best_objective.is_finite());
        assert!(artifact.baseline_objective.is_finite());
        // Parameters should be within bounds
        let bounds = default_parameter_bounds();
        let (lo, hi) = bounds_to_lo_hi(&bounds);
        let v = params_to_vec(artifact.parameters);
        for i in 0..PARAM_DIM {
            assert!(
                v[i] >= lo[i] && v[i] <= hi[i],
                "param[{i}] = {} out of bounds [{}, {}]",
                v[i],
                lo[i],
                hi[i]
            );
        }
    }
}
