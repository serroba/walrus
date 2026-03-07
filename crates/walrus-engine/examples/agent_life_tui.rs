use std::thread;
use std::time::Duration;

use walrus_engine::{
    emergence_order_parameters, local_complexity, macro_from_agents, seed_agent_based_society,
    step_agent_based_society, TransitionConfig,
};

fn glyph(agent: &walrus_engine::MicroAgent) -> char {
    if agent.aggression > 0.7 && agent.trust < 0.35 {
        'X'
    } else if agent.cooperation > 0.65 && agent.trust > 0.55 {
        'o'
    } else if agent.resources > 1.1 {
        '+'
    } else {
        '.'
    }
}

fn print_frame(
    frame: u64,
    society: &walrus_engine::AgentBasedSociety,
    stats: walrus_engine::AgentInteractionStats,
) {
    let macro_state = macro_from_agents(society);
    let complexity = local_complexity(macro_state);
    let emergence = emergence_order_parameters(
        macro_state.population,
        macro_state.mode,
        macro_state.surplus_per_capita,
        macro_state.network_coupling,
        macro_state.ecological_pressure,
    );

    let n = society.agents.len();
    let side = (n as f64).sqrt().floor() as usize;

    print!("\x1B[2J\x1B[H");
    println!("Walrus Agent Life TUI  |  tick={frame}");
    println!(
        "mode={:?}  pop={}  SO={:.3}  CX={:.3}  trust={:.3}  inequality={:.3}",
        macro_state.mode,
        macro_state.population,
        emergence.superorganism_index,
        complexity.complexity_index,
        stats.mean_trust,
        stats.inequality,
    );
    println!(
        "cooperate={} conflict={} trade={}  surplus={:.3}  eco={:.3} coupling={:.3}",
        stats.cooperations,
        stats.conflicts,
        stats.trades,
        macro_state.surplus_per_capita,
        macro_state.ecological_pressure,
        macro_state.network_coupling,
    );
    println!("legend: o cooperative  X conflict-prone  + high-resource  . neutral");
    println!();

    for r in 0..side {
        let mut line = String::with_capacity(side);
        for c in 0..side {
            let idx = r * side + c;
            line.push(glyph(&society.agents[idx]));
        }
        println!("{line}");
    }
}

fn main() {
    let mut society =
        seed_agent_based_society(400, walrus_engine::SubsistenceMode::Sedentary, 0.45, 0.18);
    let cfg = TransitionConfig::default();

    for tick in 0..220 {
        let stats = step_agent_based_society(&mut society);
        let macro_state = macro_from_agents(&society);
        society.mode = walrus_engine::next_subsistence_mode(
            society.mode,
            macro_state.population,
            macro_state.surplus_per_capita,
            macro_state.ecological_pressure,
            cfg,
        );

        print_frame(tick, &society, stats);
        thread::sleep(Duration::from_millis(60));
    }
}
