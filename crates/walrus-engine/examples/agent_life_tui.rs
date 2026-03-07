use std::thread;
use std::time::Duration;

use walrus_engine::{
    emergence_from_projection, local_complexity, macro_from_agents, micro_macro_projection,
    seed_agent_based_society_with_topology, step_agent_based_society, InteractionTopology,
    SubsistenceMode, TransitionConfig,
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
    mode_changed: bool,
    coop_hist: &[f64],
    conflict_hist: &[f64],
    trade_hist: &[f64],
) {
    let macro_state = macro_from_agents(society);
    let projection = micro_macro_projection(society);
    let complexity = local_complexity(macro_state);
    let emergence = emergence_from_projection(macro_state, projection);

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
    println!(
        "migration={} births={} deaths={} replacements={}",
        stats.migrations, stats.births, stats.deaths, stats.replacements
    );
    if mode_changed {
        println!("event: MODE TRANSITION -> {:?}", society.mode);
    }
    println!(
        "activity: {}  coop:{} conflict:{} trade:{}",
        interaction_label(stats),
        sparkline(coop_hist),
        sparkline(conflict_hist),
        sparkline(trade_hist),
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

fn interaction_label(stats: walrus_engine::AgentInteractionStats) -> &'static str {
    if stats.conflict_rate > stats.cooperation_rate && stats.conflict_rate > stats.trade_rate {
        "\x1b[31mconflict-heavy\x1b[0m"
    } else if stats.cooperation_rate > stats.trade_rate {
        "\x1b[32mcooperation-heavy\x1b[0m"
    } else {
        "\x1b[36mtrade-heavy\x1b[0m"
    }
}

fn sparkline(values: &[f64]) -> String {
    let mut out = String::with_capacity(values.len());
    for value in values {
        let ch = if *value < 0.2 {
            '.'
        } else if *value < 0.4 {
            '-'
        } else if *value < 0.6 {
            '='
        } else if *value < 0.8 {
            '+'
        } else {
            '#'
        };
        out.push(ch);
    }
    out
}

fn main() {
    let mut society = seed_agent_based_society_with_topology(
        400,
        SubsistenceMode::Sedentary,
        0.45,
        0.18,
        InteractionTopology::SmallWorld,
        2,
        2026,
    );
    let cfg = TransitionConfig::default();
    let mut coop_hist = Vec::new();
    let mut conflict_hist = Vec::new();
    let mut trade_hist = Vec::new();

    for tick in 0..220 {
        let prev_mode = society.mode;
        let stats = step_agent_based_society(&mut society);
        let macro_state = macro_from_agents(&society);
        society.mode = walrus_engine::next_subsistence_mode(
            society.mode,
            macro_state.population,
            macro_state.surplus_per_capita,
            macro_state.ecological_pressure,
            cfg,
        );
        let mode_changed = prev_mode != society.mode;
        coop_hist.push(stats.cooperation_rate);
        conflict_hist.push(stats.conflict_rate);
        trade_hist.push(stats.trade_rate);
        if coop_hist.len() > 48 {
            coop_hist.remove(0);
            conflict_hist.remove(0);
            trade_hist.remove(0);
        }

        print_frame(
            tick,
            &society,
            stats,
            mode_changed,
            &coop_hist,
            &conflict_hist,
            &trade_hist,
        );
        thread::sleep(Duration::from_millis(60));
    }
}
