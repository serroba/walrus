use walrus_engine::agents::{simulate_agents, AgentSimConfig};

fn main() {
    let configs = [
        ("abundant-dense", AgentSimConfig {
            initial_population: 200,
            ticks: 500,
            resource_regen: 0.15,
            world_size: 40.0,
            max_population: 2000,
            ..AgentSimConfig::default()
        }),
        ("scarce-sparse", AgentSimConfig {
            initial_population: 100,
            ticks: 500,
            resource_regen: 0.03,
            world_size: 80.0,
            max_population: 2000,
            seed: 7,
            ..AgentSimConfig::default()
        }),
        ("dense-small-world", AgentSimConfig {
            initial_population: 300,
            ticks: 500,
            resource_regen: 0.10,
            world_size: 25.0,
            interaction_radius: 5.0,
            max_population: 5000,
            seed: 99,
            ..AgentSimConfig::default()
        }),
    ];

    println!("tick,scenario,pop,mean_resources,gini,skill_entropy,hierarchy_depth,leaders,mean_group_size,kin_groups,coop_rate,conflict_rate,prestige,health");

    for (name, cfg) in &configs {
        let result = simulate_agents(cfg.clone());
        for snap in &result.snapshots {
            let e = &snap.emergent;
            println!(
                "{},{},{},{:.4},{:.4},{:.4},{},{},{:.2},{},{:.4},{:.4},{:.4},{:.4}",
                snap.tick,
                name,
                e.population_size,
                e.mean_resources,
                e.gini_coefficient,
                e.skill_entropy,
                e.max_hierarchy_depth,
                e.num_leaders,
                e.mean_group_size,
                e.num_kin_groups,
                e.cooperation_rate,
                e.conflict_rate,
                e.mean_prestige,
                e.mean_health,
            );
        }
        let final_pop = result.final_population.len();
        let ticks_run = result.snapshots.len();
        eprintln!("{name}: {ticks_run} ticks, final pop = {final_pop}");
    }
}
