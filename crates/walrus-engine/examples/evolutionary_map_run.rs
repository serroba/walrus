use walrus_engine::evolution::{
    simulate_evolution, ContinentalLayout, DunbarBehaviorModel, EvolutionConfig,
};

fn main() {
    let result = simulate_evolution(EvolutionConfig {
        seed: 2026,
        generations: 260,
        initial_societies: 20,
        nk_n: 14,
        nk_k: 3,
        layout: ContinentalLayout::Regional,
        isolation_factor: 0.35,
        dunbar_model: DunbarBehaviorModel::default(),
        ..EvolutionConfig::default()
    });

    println!(
        "generation,population_total,mean_complexity,mean_energy_access,collapse_events,emergent_civilizations,convergence_index,adaptation_divergence,natural_disaster_events,pandemic_events"
    );
    for snapshot in &result.snapshots {
        if snapshot.generation % 20 == 0 || snapshot.generation + 1 == result.snapshots.len() as u32
        {
            println!(
                "{},{},{:.3},{:.3},{},{},{:.3},{:.3},{},{}",
                snapshot.generation,
                snapshot.population_total,
                snapshot.mean_complexity,
                snapshot.mean_energy_access,
                snapshot.collapse_events,
                snapshot.emergent_civilizations,
                snapshot.convergence_index,
                snapshot.adaptation_divergence,
                snapshot.natural_disaster_events,
                snapshot.pandemic_events,
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
