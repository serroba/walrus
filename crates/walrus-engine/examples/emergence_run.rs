use walrus_engine::{
    run_emergence_simulation, scenario_local_emergence_baseline, TransitionConfig,
};

fn main() {
    let societies = scenario_local_emergence_baseline();
    let snapshots = run_emergence_simulation(societies, 200, TransitionConfig::default());

    if snapshots.is_empty() {
        println!("No snapshots produced.");
        return;
    }

    println!("tick,superorganism,mean_complexity,hunter,sedentary,agriculture");
    for snapshot in snapshots.iter().step_by(20) {
        println!(
            "{},{:.3},{:.3},{},{},{}",
            snapshot.tick,
            snapshot.global.superorganism_index,
            snapshot.mean_local_complexity,
            snapshot.hunter_gatherer_count,
            snapshot.sedentary_count,
            snapshot.agriculture_count
        );
    }

    let last = snapshots[snapshots.len() - 1];
    println!(
        "final: tick={} superorganism={:.3} complexity={:.3} modes=({}, {}, {})",
        last.tick,
        last.global.superorganism_index,
        last.mean_local_complexity,
        last.hunter_gatherer_count,
        last.sedentary_count,
        last.agriculture_count
    );
}
