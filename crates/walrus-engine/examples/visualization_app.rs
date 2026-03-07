use std::fs::{self, File};
use std::io::Write;

use walrus_engine::{
    classify_trajectory, run_emergence_simulation, scenario_dense_coupled_growth,
    scenario_ecological_stress, scenario_fragmented_low_coupling,
    scenario_local_emergence_baseline, summarize_emergence, LocalSocietyState, TrajectoryClass,
    TransitionConfig,
};

#[derive(Clone, Copy)]
struct ScenarioSpec {
    name: &'static str,
    builder: fn() -> Vec<LocalSocietyState>,
    cfg: TransitionConfig,
}

fn safe_name(name: &str) -> String {
    name.replace('/', "_")
}

fn class_label(class: TrajectoryClass) -> &'static str {
    match class {
        TrajectoryClass::StabilizingComplexity => "stabilizing",
        TrajectoryClass::OvershootAndCorrection => "overshoot-correction",
        TrajectoryClass::FragileTransition => "fragile-transition",
        TrajectoryClass::StagnantLowComplexity => "stagnant-low-complexity",
    }
}

fn json_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn build_data_js() -> String {
    let base_cfg = TransitionConfig::default();
    let fast_transition_cfg = TransitionConfig {
        sedentarism_population_threshold: 90,
        sedentarism_surplus_threshold: 0.18,
        agriculture_population_threshold: 500,
        agriculture_surplus_threshold: 0.35,
        ..TransitionConfig::default()
    };
    let fragile_cfg = TransitionConfig {
        regression_ecological_pressure_threshold: 0.72,
        regression_surplus_threshold: 0.28,
        ..TransitionConfig::default()
    };

    let specs = [
        ScenarioSpec {
            name: "baseline/default",
            builder: scenario_local_emergence_baseline,
            cfg: base_cfg,
        },
        ScenarioSpec {
            name: "baseline/fast-transition",
            builder: scenario_local_emergence_baseline,
            cfg: fast_transition_cfg,
        },
        ScenarioSpec {
            name: "eco-stress/default",
            builder: scenario_ecological_stress,
            cfg: base_cfg,
        },
        ScenarioSpec {
            name: "eco-stress/fragile",
            builder: scenario_ecological_stress,
            cfg: fragile_cfg,
        },
        ScenarioSpec {
            name: "dense-coupled/default",
            builder: scenario_dense_coupled_growth,
            cfg: base_cfg,
        },
        ScenarioSpec {
            name: "fragmented-low-coupling/default",
            builder: scenario_fragmented_low_coupling,
            cfg: base_cfg,
        },
    ];

    let mut scenarios = String::new();

    for (idx, spec) in specs.iter().enumerate() {
        let snapshots = run_emergence_simulation((spec.builder)(), 300, spec.cfg);
        let summary = summarize_emergence(&snapshots);
        let class = classify_trajectory(summary);

        if idx > 0 {
            scenarios.push(',');
        }

        let points = snapshots
            .iter()
            .map(|s| {
                format!(
                    "{{\"tick\":{},\"so\":{:.6},\"cx\":{:.6},\"h\":{},\"s\":{},\"a\":{}}}",
                    s.tick,
                    s.global.superorganism_index,
                    s.mean_local_complexity,
                    s.hunter_gatherer_count,
                    s.sedentary_count,
                    s.agriculture_count,
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        let final_modes = snapshots[snapshots.len() - 1];

        scenarios.push_str(&format!(
            "{{\"name\":\"{}\",\"id\":\"{}\",\"class\":\"{}\",\"summary\":{{\"startSO\":{:.6},\"peakSO\":{:.6},\"endSO\":{:.6},\"startCX\":{:.6},\"peakCX\":{:.6},\"endCX\":{:.6}}},\"finalModes\":{{\"h\":{},\"s\":{},\"a\":{}}},\"points\":[{}]}}",
            json_escape(spec.name),
            safe_name(spec.name),
            class_label(class),
            summary.start_superorganism,
            summary.peak_superorganism,
            summary.end_superorganism,
            summary.start_mean_complexity,
            summary.peak_mean_complexity,
            summary.end_mean_complexity,
            final_modes.hunter_gatherer_count,
            final_modes.sedentary_count,
            final_modes.agriculture_count,
            points,
        ));
    }

    format!("const APP_DATA = [{}];", scenarios)
}

fn build_html() -> String {
    let data_js = build_data_js();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Walrus Simulator Viewer</title>
  <style>
    :root {{
      --bg: #f6f2e9;
      --panel: #fffdfa;
      --ink: #212018;
      --muted: #5f5a4d;
      --accent: #0f6c5a;
      --warn: #9d4a35;
      --line1: #1b7f79;
      --line2: #d15f2e;
    }}
    body {{ margin: 0; font-family: ui-serif, Georgia, serif; background: radial-gradient(circle at top left, #fffaf0 0%, var(--bg) 60%); color: var(--ink); }}
    .wrap {{ max-width: 1040px; margin: 0 auto; padding: 24px; }}
    .card {{ background: var(--panel); border: 1px solid #d9d2c4; border-radius: 14px; padding: 16px; margin-bottom: 16px; }}
    h1 {{ margin: 0 0 8px; font-size: 30px; }}
    p {{ margin: 0 0 10px; color: var(--muted); line-height: 1.45; }}
    select {{ width: 100%; padding: 10px; border: 1px solid #c6bea8; border-radius: 10px; background: #fff; }}
    .grid {{ display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }}
    .metric {{ background: #fffcf5; border: 1px solid #e0d8c7; border-radius: 10px; padding: 10px; }}
    .metric .k {{ color: var(--muted); font-size: 12px; }}
    .metric .v {{ font-size: 24px; font-weight: 700; }}
    .badge {{ display: inline-block; padding: 4px 10px; border-radius: 999px; font-size: 12px; border: 1px solid #c4baa3; background: #f3efe4; }}
    canvas {{ width: 100%; height: 280px; border: 1px solid #ddd4c0; border-radius: 10px; background: #fff; }}
    .legend {{ display: flex; gap: 12px; font-size: 13px; color: var(--muted); margin-top: 6px; }}
    .dot {{ width: 10px; height: 10px; border-radius: 50%; display: inline-block; margin-right: 6px; }}
    .line1 {{ background: var(--line1); }}
    .line2 {{ background: var(--line2); }}
    @media (max-width: 900px) {{ .grid {{ grid-template-columns: 1fr; }} }}
  </style>
</head>
<body>
  <div class="wrap">
    <div class="card">
      <h1>Walrus Simulator Viewer</h1>
      <p>This dashboard translates simulation output into plain language. Choose a scenario to see how collective behavior evolves over time.</p>
      <select id="scenario"></select>
      <p style="margin-top:10px;"><span class="badge" id="classBadge">-</span></p>
    </div>

    <div class="card">
      <div class="grid">
        <div class="metric"><div class="k">Start Superorganism</div><div class="v" id="startSO">-</div></div>
        <div class="metric"><div class="k">Peak Superorganism</div><div class="v" id="peakSO">-</div></div>
        <div class="metric"><div class="k">End Superorganism</div><div class="v" id="endSO">-</div></div>
        <div class="metric"><div class="k">Start Complexity</div><div class="v" id="startCX">-</div></div>
        <div class="metric"><div class="k">Peak Complexity</div><div class="v" id="peakCX">-</div></div>
        <div class="metric"><div class="k">End Complexity</div><div class="v" id="endCX">-</div></div>
      </div>
      <p id="modes" style="margin-top:12px;"></p>
    </div>

    <div class="card">
      <canvas id="chart" width="980" height="280"></canvas>
      <div class="legend">
        <span><span class="dot line1"></span>Superorganism Index</span>
        <span><span class="dot line2"></span>Mean Local Complexity</span>
      </div>
    </div>
  </div>

  <script>{}</script>
  <script>
    const selector = document.getElementById('scenario');
    const chart = document.getElementById('chart');
    const ctx = chart.getContext('2d');

    function fmt(v) {{ return Number(v).toFixed(3); }}

    function drawSeries(points) {{
      const w = chart.width;
      const h = chart.height;
      const pad = 24;
      ctx.clearRect(0, 0, w, h);

      ctx.strokeStyle = '#ddd4c0';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(pad, h - pad);
      ctx.lineTo(w - pad, h - pad);
      ctx.moveTo(pad, pad);
      ctx.lineTo(pad, h - pad);
      ctx.stroke();

      const maxTick = points[points.length - 1].tick || 1;
      const x = (t) => pad + (t / maxTick) * (w - pad * 2);
      const y = (v) => h - pad - (v * (h - pad * 2));

      function line(color, key) {{
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.beginPath();
        points.forEach((p, i) => {{
          const px = x(p.tick);
          const py = y(p[key]);
          if (i === 0) ctx.moveTo(px, py);
          else ctx.lineTo(px, py);
        }});
        ctx.stroke();
      }}

      line('#1b7f79', 'so');
      line('#d15f2e', 'cx');
    }}

    function setScenario(index) {{
      const scenario = APP_DATA[index];
      document.getElementById('classBadge').textContent = scenario.class;
      document.getElementById('startSO').textContent = fmt(scenario.summary.startSO);
      document.getElementById('peakSO').textContent = fmt(scenario.summary.peakSO);
      document.getElementById('endSO').textContent = fmt(scenario.summary.endSO);
      document.getElementById('startCX').textContent = fmt(scenario.summary.startCX);
      document.getElementById('peakCX').textContent = fmt(scenario.summary.peakCX);
      document.getElementById('endCX').textContent = fmt(scenario.summary.endCX);
      document.getElementById('modes').textContent = 'Final social composition (H/S/A): ' + scenario.finalModes.h + '/' + scenario.finalModes.s + '/' + scenario.finalModes.a;
      drawSeries(scenario.points);
    }}

    APP_DATA.forEach((s, i) => {{
      const opt = document.createElement('option');
      opt.value = String(i);
      opt.textContent = s.name;
      selector.appendChild(opt);
    }});

    selector.addEventListener('change', (e) => setScenario(Number(e.target.value)));
    setScenario(0);
  </script>
</body>
</html>
"#,
        data_js
    )
}

fn main() -> std::io::Result<()> {
    let app_dir = "outputs/latest/app";
    fs::create_dir_all(app_dir)?;
    let mut file = File::create(format!("{app_dir}/index.html"))?;
    file.write_all(build_html().as_bytes())?;
    println!("Wrote standalone viewer: {app_dir}/index.html");
    Ok(())
}
