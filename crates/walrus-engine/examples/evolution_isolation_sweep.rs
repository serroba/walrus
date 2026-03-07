use walrus_engine::evolution::{simulate_evolution, ContinentalLayout, EvolutionConfig};

fn summarize(layout: ContinentalLayout, isolation_factor: f64) {
    let result = simulate_evolution(EvolutionConfig {
        layout,
        isolation_factor,
        generations: 260,
        initial_societies: 20,
        ..EvolutionConfig::default()
    });

    let collapse_total = result
        .snapshots
        .iter()
        .map(|s| u64::from(s.collapse_events))
        .sum::<u64>();
    let peak_emergence = result
        .snapshots
        .iter()
        .map(|s| s.emergent_civilizations)
        .max()
        .unwrap_or(0);
    let final_snap =
        result
            .snapshots
            .last()
            .copied()
            .unwrap_or(walrus_engine::evolution::EvolutionSnapshot {
                generation: 0,
                population_total: 0,
                mean_complexity: 0.0,
                mean_energy_access: 0.0,
                collapse_events: 0,
                emergent_civilizations: 0,
                convergence_index: 0.0,
                adaptation_divergence: 0.0,
                superorganism_index: 0.0,
            });

    println!(
        "{:?},{:.2},{},{},{:.3},{:.3},{:.3},{:.3}",
        layout,
        isolation_factor,
        collapse_total,
        peak_emergence,
        final_snap.mean_complexity,
        final_snap.convergence_index,
        final_snap.adaptation_divergence,
        final_snap.superorganism_index,
    );
}

fn main() {
    println!(
        "layout,isolation_factor,total_collapses,peak_emergent_polities,final_complexity,final_convergence,final_divergence,final_superorganism"
    );

    summarize(ContinentalLayout::Connected, 0.05);
    summarize(ContinentalLayout::Regional, 0.35);
    summarize(ContinentalLayout::Regional, 0.70);
    summarize(ContinentalLayout::Islands, 0.75);
}
