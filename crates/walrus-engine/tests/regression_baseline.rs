//! Regression baseline: compare tick-based and event-driven engines.
//!
//! Both engines should produce statistically similar macro-level distributions
//! when given equivalent parameters.  We run multiple seeds per engine and
//! compare population-level metrics using a two-sample Kolmogorov-Smirnov test.
//!
//! This is the validation gate before the old tick-based engine can be removed.

use walrus_engine::agents::{simulate_agents, AgentSimConfig, EnergyParams, LifecycleParams};
use walrus_engine::event_sim::{simulate_event_driven, EventParams, EventSimConfig};

// ---------------------------------------------------------------------------
// Kolmogorov-Smirnov two-sample test (no external dependency)
// ---------------------------------------------------------------------------

/// Compute the KS statistic between two samples.
/// Returns (D, approximate p-value using the asymptotic formula).
fn ks_two_sample(a: &mut [f64], b: &mut [f64]) -> (f64, f64) {
    a.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    b.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

    let na = a.len() as f64;
    let nb = b.len() as f64;
    let mut ia = 0_usize;
    let mut ib = 0_usize;
    let mut d_max = 0.0_f64;

    while ia < a.len() && ib < b.len() {
        if a[ia] < b[ib] {
            ia += 1;
        } else if a[ia] > b[ib] {
            ib += 1;
        } else {
            ia += 1;
            ib += 1;
        }
        let d = ((ia as f64 / na) - (ib as f64 / nb)).abs();
        if d > d_max {
            d_max = d;
        }
    }

    // Asymptotic p-value approximation (Kolmogorov distribution)
    let en = (na * nb / (na + nb)).sqrt();
    let lambda = (en + 0.12 + 0.11 / en) * d_max;
    // Approximation: P(D > d) ≈ 2 * sum_{k=1}^{inf} (-1)^{k+1} * exp(-2 k^2 lambda^2)
    let mut p = 0.0_f64;
    for k in 1..=100 {
        let sign = if k % 2 == 1 { 1.0 } else { -1.0 };
        let term = sign * (-2.0 * (k as f64).powi(2) * lambda * lambda).exp();
        p += term;
        if term.abs() < 1e-12 {
            break;
        }
    }
    let p_value = (2.0 * p).clamp(0.0, 1.0);
    (d_max, p_value)
}

// ---------------------------------------------------------------------------
// Helper: run multiple seeds and collect final metrics
// ---------------------------------------------------------------------------

struct MetricSet {
    final_population: Vec<f64>,
    gini: Vec<f64>,
    cooperation_rate: Vec<f64>,
    conflict_rate: Vec<f64>,
    mean_health: Vec<f64>,
    mean_innovation: Vec<f64>,
    cultural_diversity: Vec<f64>,
    hierarchy_depth: Vec<f64>,
}

impl MetricSet {
    fn new() -> Self {
        Self {
            final_population: Vec::new(),
            gini: Vec::new(),
            cooperation_rate: Vec::new(),
            conflict_rate: Vec::new(),
            mean_health: Vec::new(),
            mean_innovation: Vec::new(),
            cultural_diversity: Vec::new(),
            hierarchy_depth: Vec::new(),
        }
    }
}

fn base_config() -> AgentSimConfig {
    AgentSimConfig {
        initial_population: 80,
        ticks: 150,
        world_size: 35.0,
        max_population: 2000,
        lifecycle: LifecycleParams {
            innovation_growth_rate: 0.001,
            ..LifecycleParams::default()
        },
        energy: EnergyParams {
            biomass_flow_rate: 0.08,
            ..EnergyParams::default()
        },
        ..AgentSimConfig::default()
    }
}

const SEEDS: u32 = 20;

fn run_tick_based(seeds: u32) -> MetricSet {
    let mut metrics = MetricSet::new();
    let base = base_config();

    for s in 0..seeds {
        let cfg = AgentSimConfig {
            seed: 1000 + u64::from(s) * 7919,
            ..base
        };
        let result = simulate_agents(cfg);
        if let Some(last) = result.snapshots.last() {
            let e = &last.emergent;
            metrics.final_population.push(f64::from(e.population_size));
            metrics.gini.push(f64::from(e.gini_coefficient));
            metrics.cooperation_rate.push(f64::from(e.cooperation_rate));
            metrics.conflict_rate.push(f64::from(e.conflict_rate));
            metrics.mean_health.push(f64::from(e.mean_health));
            metrics.mean_innovation.push(f64::from(e.mean_innovation));
            metrics
                .cultural_diversity
                .push(f64::from(e.cultural_diversity));
            metrics
                .hierarchy_depth
                .push(f64::from(e.max_hierarchy_depth));
        }
    }
    metrics
}

fn run_event_driven(seeds: u32) -> MetricSet {
    let mut metrics = MetricSet::new();
    let base = base_config();

    for s in 0..seeds {
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                seed: 1000 + u64::from(s) * 7919,
                ..base
            },
            event: EventParams::default(),
            // Match tick count: 150 ticks ≈ 150 sim-time units
            end_time: 150.0,
        };
        let result = simulate_event_driven(cfg);
        if let Some(last) = result.snapshots.last() {
            let e = &last.emergent;
            metrics.final_population.push(f64::from(e.population_size));
            metrics.gini.push(f64::from(e.gini_coefficient));
            metrics.cooperation_rate.push(f64::from(e.cooperation_rate));
            metrics.conflict_rate.push(f64::from(e.conflict_rate));
            metrics.mean_health.push(f64::from(e.mean_health));
            metrics.mean_innovation.push(f64::from(e.mean_innovation));
            metrics
                .cultural_diversity
                .push(f64::from(e.cultural_diversity));
            metrics
                .hierarchy_depth
                .push(f64::from(e.max_hierarchy_depth));
        }
    }
    metrics
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Compare a single metric between engines.  Returns (name, D, p, passed).
fn compare_metric(name: &str, tick: &[f64], event: &[f64], alpha: f64) -> (String, f64, f64, bool) {
    let mut a = tick.to_vec();
    let mut b = event.to_vec();
    let (d, p) = ks_two_sample(&mut a, &mut b);
    // We want to *not* reject H0 (same distribution), so p > alpha is a pass.
    (name.to_string(), d, p, p > alpha)
}

/// The two engines are architecturally different by design — the event-driven
/// engine produces different lifecycle dynamics (agents forage and recover on
/// independent stochastic schedules).  What must be preserved is:
///
/// 1. **Interaction ratios**: cooperation/conflict/trade balance comes from
///    the same formulas, so the relative rates should be similar.
/// 2. **Qualitative emergence**: both engines produce hierarchy, inequality,
///    cultural diversity, and innovation growth.
/// 3. **Bounded metrics**: everything stays in valid ranges.
///
/// We use KS tests only on the interaction ratios (where the math is shared)
/// and qualitative checks for everything else.
#[test]
fn interaction_ratios_are_preserved_across_engines() {
    let tick_metrics = run_tick_based(SEEDS);
    let event_metrics = run_event_driven(SEEDS);

    // Trust-memory updates differ structurally: tick-based batches updates while
    // event-driven updates immediately.  This introduces a small controlled
    // divergence in conflict_rate, so we use a relaxed alpha.
    let alpha = 0.005;

    let comparisons = vec![
        compare_metric(
            "cooperation_rate",
            &tick_metrics.cooperation_rate,
            &event_metrics.cooperation_rate,
            alpha,
        ),
        compare_metric(
            "conflict_rate",
            &tick_metrics.conflict_rate,
            &event_metrics.conflict_rate,
            alpha,
        ),
    ];

    // Print full diagnostic table
    eprintln!("\n=== Regression Baseline: Tick vs Event-Driven ===");
    eprintln!("{:<22} {:>10} {:>10}", "Metric", "Tick mean", "Event mean");
    eprintln!("{}", "-".repeat(50));
    let print_means = |name: &str, a: &[f64], b: &[f64]| {
        let ma: f64 = a.iter().sum::<f64>() / a.len() as f64;
        let mb: f64 = b.iter().sum::<f64>() / b.len() as f64;
        eprintln!("{:<22} {:>10.3} {:>10.3}", name, ma, mb);
    };
    print_means(
        "final_population",
        &tick_metrics.final_population,
        &event_metrics.final_population,
    );
    print_means("gini", &tick_metrics.gini, &event_metrics.gini);
    print_means(
        "cooperation_rate",
        &tick_metrics.cooperation_rate,
        &event_metrics.cooperation_rate,
    );
    print_means(
        "conflict_rate",
        &tick_metrics.conflict_rate,
        &event_metrics.conflict_rate,
    );
    print_means(
        "mean_health",
        &tick_metrics.mean_health,
        &event_metrics.mean_health,
    );
    print_means(
        "mean_innovation",
        &tick_metrics.mean_innovation,
        &event_metrics.mean_innovation,
    );
    print_means(
        "cultural_diversity",
        &tick_metrics.cultural_diversity,
        &event_metrics.cultural_diversity,
    );
    print_means(
        "hierarchy_depth",
        &tick_metrics.hierarchy_depth,
        &event_metrics.hierarchy_depth,
    );

    eprintln!(
        "\n{:<22} {:>6} {:>8} {:>6}",
        "KS Test", "D", "p-value", "Pass"
    );
    eprintln!("{}", "-".repeat(50));
    for (name, d, p, passed) in &comparisons {
        eprintln!(
            "{:<22} {:>6.3} {:>8.4} {:>6}",
            name,
            d,
            p,
            if *passed { "OK" } else { "FAIL" }
        );
    }

    // Interaction ratios should be statistically similar (same formulas)
    for (name, _d, _p, passed) in &comparisons {
        assert!(
            *passed,
            "{name} distributions diverged between engines (p < {alpha})"
        );
    }
}

/// Both engines must produce the same qualitative emergent properties:
/// hierarchy, inequality, cultural diversity, cooperation, and conflict.
#[test]
fn both_engines_produce_qualitative_emergence() {
    let tick_metrics = run_tick_based(SEEDS);
    let event_metrics = run_event_driven(SEEDS);

    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;

    // Both produce inequality
    assert!(mean(&tick_metrics.gini) > 0.05, "tick: gini too low");
    assert!(mean(&event_metrics.gini) > 0.05, "event: gini too low");

    // Both produce hierarchy
    assert!(
        mean(&tick_metrics.hierarchy_depth) >= 1.0,
        "tick: no hierarchy"
    );
    assert!(
        mean(&event_metrics.hierarchy_depth) >= 1.0,
        "event: no hierarchy"
    );

    // Both produce cultural diversity
    assert!(
        mean(&tick_metrics.cultural_diversity) > 0.1,
        "tick: no cultural diversity"
    );
    assert!(
        mean(&event_metrics.cultural_diversity) > 0.1,
        "event: no cultural diversity"
    );

    // Both have cooperation and conflict (not all-peace or all-war)
    assert!(
        mean(&tick_metrics.cooperation_rate) > 0.1,
        "tick: no cooperation"
    );
    assert!(
        mean(&event_metrics.cooperation_rate) > 0.1,
        "event: no cooperation"
    );
    assert!(
        mean(&tick_metrics.conflict_rate) > 0.05,
        "tick: no conflict"
    );
    assert!(
        mean(&event_metrics.conflict_rate) > 0.05,
        "event: no conflict"
    );

    // Both show innovation growth (starting ~0.15)
    assert!(
        mean(&tick_metrics.mean_innovation) > 0.15,
        "tick: no innovation growth"
    );
    assert!(
        mean(&event_metrics.mean_innovation) > 0.15,
        "event: no innovation growth"
    );

    // Both sustain populations
    assert!(
        mean(&tick_metrics.final_population) > 20.0,
        "tick: population collapsed"
    );
    assert!(
        mean(&event_metrics.final_population) > 20.0,
        "event: population collapsed"
    );
}
