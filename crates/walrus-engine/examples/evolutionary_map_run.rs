use walrus_engine::evolution::{simulate_evolution, EvolutionConfig};

fn main() {
    let result = simulate_evolution(EvolutionConfig {
        seed: 2026,
        generations: 260,
        initial_societies: 20,
        nk_n: 14,
        nk_k: 3,
    });

    println!("generation,population_total,mean_complexity,mean_energy_access,collapse_events,emergent_civilizations");
    for snapshot in &result.snapshots {
        if snapshot.generation % 20 == 0 || snapshot.generation + 1 == result.snapshots.len() as u32
        {
            println!(
                "{},{},{:.3},{:.3},{},{}",
                snapshot.generation,
                snapshot.population_total,
                snapshot.mean_complexity,
                snapshot.mean_energy_access,
                snapshot.collapse_events,
                snapshot.emergent_civilizations,
            );
        }
    }

    println!("\ncontinent,surviving_societies,total_population,mean_complexity,mean_depletion");
    for outcome in &result.continent_outcomes {
        println!(
            "{},{},{},{:.3},{:.3}",
            outcome.name,
            outcome.surviving_societies,
            outcome.total_population,
            outcome.mean_complexity,
            outcome.mean_depletion,
        );
    }
}
