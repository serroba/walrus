use std::fs::{self, File};
use std::io::{BufWriter, Write};

use walrus_engine::{
    run_emergence_simulation, scenario_ecological_stress, scenario_fragmented_low_coupling,
    scenario_local_emergence_baseline, TransitionConfig,
};

fn main() -> std::io::Result<()> {
    let output_dir = "outputs/latest";
    fs::create_dir_all(output_dir)?;

    let ticks = 300;
    let base_cfg = TransitionConfig::default();
    let fragile_cfg = TransitionConfig {
        regression_ecological_pressure_threshold: 0.72,
        regression_surplus_threshold: 0.28,
        ..TransitionConfig::default()
    };

    let scenarios: Vec<(
        &str,
        Vec<walrus_engine::LocalSocietyState>,
        TransitionConfig,
    )> = vec![
        (
            "timeline_baseline_default.csv",
            scenario_local_emergence_baseline(),
            base_cfg,
        ),
        (
            "timeline_eco-stress_fragile.csv",
            scenario_ecological_stress(),
            fragile_cfg,
        ),
        (
            "timeline_fragmented-low-coupling_default.csv",
            scenario_fragmented_low_coupling(),
            base_cfg,
        ),
    ];

    for (filename, societies, cfg) in scenarios {
        let snapshots = run_emergence_simulation(societies, ticks, cfg);
        let path = format!("{output_dir}/{filename}");
        let file = File::create(&path)?;
        let mut w = BufWriter::new(file);
        writeln!(w, "tick,so,cx,h,s,a")?;
        for snap in &snapshots {
            writeln!(
                w,
                "{},{:.6},{:.6},{},{},{}",
                snap.tick,
                snap.global.superorganism_index,
                snap.mean_local_complexity,
                snap.hunter_gatherer_count,
                snap.sedentary_count,
                snap.agriculture_count,
            )?;
        }
        w.flush()?;
        println!("Wrote {path}");
    }

    Ok(())
}
