use std::fs::{self, File};
use std::io::Write;

use walrus_engine::calibration::{
    baseline_parameters, calibration_confidence, default_parameter_bounds, ingest_owid_or_maddison,
    run_calibration, stylized_targets, CalibrationConfidence, CalibrationConfig,
};
use walrus_engine::ensemble::{
    run_ensemble, validation_report, EnsembleConfig, EnsembleSummary, ValidationReport,
};
use walrus_engine::{
    run_agent_based_simulation, seed_agent_based_society_with_topology, InteractionTopology,
    SubsistenceMode, TransitionConfig,
};

#[derive(Clone, Debug)]
struct ScenarioView {
    name: String,
    confidence: CalibrationConfidence,
    robustness: f64,
    fit_population: f64,
    fit_urbanization: f64,
    fit_gdp: f64,
    fit_energy: f64,
    driver: String,
    trajectories: Vec<String>,
    events: Vec<String>,
}

fn json_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn confidence_label(level: CalibrationConfidence) -> &'static str {
    match level {
        CalibrationConfidence::Exploratory => "exploratory",
        CalibrationConfidence::CalibratedStylized => "calibrated-stylized",
        CalibrationConfidence::CalibratedCurveFit => "calibrated-curve-fit",
    }
}

fn build_view(
    name: &str,
    confidence: CalibrationConfidence,
    report: &ValidationReport,
    ensemble: &EnsembleSummary,
) -> ScenarioView {
    let mut driver = "mixed loops".to_string();
    if let (Some(first), Some(last)) = (ensemble.trajectories.first(), ensemble.trajectories.last())
    {
        let so_gain = last.superorganism_p50 - first.superorganism_p50;
        let cx_gain = last.complexity_p50 - first.complexity_p50;
        if so_gain > 0.15 && cx_gain > 0.15 {
            driver = "coordination and trade loops dominate".to_string();
        } else if so_gain < 0.0 || cx_gain < 0.0 {
            driver = "stress and conflict balancing loops dominate".to_string();
        } else if so_gain > 0.10 && cx_gain < 0.05 {
            driver = "throughput rises faster than social complexity".to_string();
        }
    }

    let trajectories = ensemble
        .trajectories
        .iter()
        .map(|point| {
            format!(
                "{{\"year\":{},\"so_p10\":{:.6},\"so_p50\":{:.6},\"so_p90\":{:.6},\"cx_p10\":{:.6},\"cx_p50\":{:.6},\"cx_p90\":{:.6}}}",
                point.year,
                point.superorganism_p10,
                point.superorganism_p50,
                point.superorganism_p90,
                point.complexity_p10,
                point.complexity_p50,
                point.complexity_p90,
            )
        })
        .collect::<Vec<String>>();

    let society = seed_agent_based_society_with_topology(
        324,
        SubsistenceMode::HunterGatherer,
        0.22,
        0.1,
        InteractionTopology::SmallWorld,
        2,
        73,
    );
    let snaps = run_agent_based_simulation(society.clone(), 160, TransitionConfig::default());
    let mut events = Vec::new();
    for idx in 1..snaps.len() {
        if snaps[idx].mode != snaps[idx - 1].mode {
            events.push(format!(
                "{{\"year\":{},\"label\":\"mode -> {:?}\"}}",
                1000 + (idx as i32),
                snaps[idx].mode
            ));
        }
    }
    if events.is_empty() {
        events.push("{\"year\":1000,\"label\":\"no mode transitions in sample run\"}".to_string());
    }

    ScenarioView {
        name: name.to_string(),
        confidence,
        robustness: report.robustness_score,
        fit_population: report.fit_population,
        fit_urbanization: report.fit_urbanization,
        fit_gdp: report.fit_gdp_per_capita,
        fit_energy: report.fit_energy,
        driver,
        trajectories,
        events,
    }
}

fn build_data_js() -> String {
    let data_path = "data/benchmarks/owid_maddison_anchor.csv";
    let benchmarks = ingest_owid_or_maddison(data_path).unwrap_or_else(|e| {
        panic!("failed to load benchmark data at {data_path}: {e:?}");
    });
    let targets = stylized_targets(&benchmarks);

    let exploratory_params = baseline_parameters();
    let exploratory_ensemble = run_ensemble(
        exploratory_params,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );
    let exploratory_report = validation_report(
        exploratory_params,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );

    let artifact = run_calibration(
        &benchmarks,
        CalibrationConfig {
            seed: 13,
            iterations: 220,
            ticks: 180,
            ..CalibrationConfig::default()
        },
        default_parameter_bounds(),
    );
    let calibrated_ensemble = run_ensemble(
        artifact.parameters,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );
    let calibrated_report = validation_report(
        artifact.parameters,
        &targets,
        &benchmarks,
        EnsembleConfig {
            ticks: 180,
            start_year: 1000,
            ..EnsembleConfig::default()
        },
    );

    let scenarios = [
        build_view(
            "exploratory baseline",
            exploratory_report.confidence,
            &exploratory_report,
            &exploratory_ensemble,
        ),
        build_view(
            "calibrated stylized",
            calibration_confidence(artifact.best_objective),
            &calibrated_report,
            &calibrated_ensemble,
        ),
    ];

    let payload = scenarios
        .iter()
        .map(|scenario| {
            format!(
                "{{\"name\":\"{}\",\"confidence\":\"{}\",\"robustness\":{:.6},\"fit\":{{\"population\":{:.6},\"urbanization\":{:.6},\"gdp\":{:.6},\"energy\":{:.6}}},\"driver\":\"{}\",\"trajectory\":[{}],\"events\":[{}]}}",
                json_escape(&scenario.name),
                confidence_label(scenario.confidence),
                scenario.robustness,
                scenario.fit_population,
                scenario.fit_urbanization,
                scenario.fit_gdp,
                scenario.fit_energy,
                json_escape(&scenario.driver),
                scenario.trajectories.join(","),
                scenario.events.join(","),
            )
        })
        .collect::<Vec<String>>()
        .join(",");

    format!("const APP_DATA = [{}];", payload)
}

fn build_html() -> String {
    let data_js = build_data_js();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Walrus Validation Viewer</title>
  <style>
    :root {{
      --bg: #f4f1e9;
      --card: #fffdf7;
      --ink: #26231c;
      --muted: #5a5447;
      --line-so: #106c76;
      --line-cx: #c2572d;
      --band-so: rgba(16, 108, 118, 0.18);
      --band-cx: rgba(194, 87, 45, 0.18);
    }}
    body {{ margin: 0; background: linear-gradient(135deg, #faf8f1 0%, var(--bg) 100%); color: var(--ink); font-family: ui-serif, Georgia, serif; }}
    .wrap {{ max-width: 1120px; margin: 0 auto; padding: 24px; }}
    .card {{ background: var(--card); border: 1px solid #dbd2c0; border-radius: 14px; padding: 16px; margin-bottom: 14px; }}
    h1 {{ margin: 0 0 8px; font-size: 30px; }}
    p {{ color: var(--muted); margin: 0 0 10px; line-height: 1.45; }}
    select {{ width: 100%; padding: 10px; border: 1px solid #cdc3ad; border-radius: 9px; background: #fff; }}
    .grid {{ display: grid; gap: 10px; grid-template-columns: repeat(4, minmax(0,1fr)); }}
    .metric {{ background: #fefcf5; border: 1px solid #e3dac8; border-radius: 10px; padding: 10px; }}
    .k {{ font-size: 12px; color: var(--muted); }}
    .v {{ font-size: 22px; font-weight: 700; }}
    .chip {{ display: inline-block; border: 1px solid #cbbfa7; border-radius: 999px; padding: 4px 10px; font-size: 12px; background: #f4efe3; }}
    canvas {{ width: 100%; height: 340px; border: 1px solid #ddd4c1; border-radius: 10px; background: #fff; }}
    .rows {{ display: grid; gap: 12px; grid-template-columns: 1fr 1fr; }}
    ul {{ margin: 0; padding-left: 16px; color: var(--muted); }}
    @media (max-width: 920px) {{ .grid {{ grid-template-columns: 1fr 1fr; }} .rows {{ grid-template-columns: 1fr; }} }}
  </style>
</head>
<body>
  <div class="wrap">
    <div class="card">
      <h1>Walrus Calibration + Ensemble Viewer</h1>
      <p>Shows uncertainty bands, calibration maturity, and annotated mode-shift events for non-technical exploration.</p>
      <select id="scenario"></select>
      <p style="margin-top:8px;"><span id="confidence" class="chip">-</span></p>
    </div>

    <div class="card">
      <div class="grid">
        <div class="metric"><div class="k">Robustness</div><div class="v" id="robustness">-</div></div>
        <div class="metric"><div class="k">Fit Population</div><div class="v" id="fitPop">-</div></div>
        <div class="metric"><div class="k">Fit Urbanization</div><div class="v" id="fitUrban">-</div></div>
        <div class="metric"><div class="k">Fit GDP/Energy</div><div class="v" id="fitMacro">-</div></div>
      </div>
    </div>

    <div class="card">
      <canvas id="chart" width="1040" height="340"></canvas>
    </div>

    <div class="rows">
      <div class="card">
        <h3 style="margin-top:0;">What Drove Change</h3>
        <p id="driver"></p>
      </div>
      <div class="card">
        <h3 style="margin-top:0;">Event Annotations</h3>
        <ul id="events"></ul>
      </div>
    </div>
  </div>

  <script>{}</script>
  <script>
    const selector = document.getElementById('scenario');
    const chart = document.getElementById('chart');
    const ctx = chart.getContext('2d');

    function fmt(v) {{ return Number(v).toFixed(3); }}

    function draw(s) {{
      const pts = s.trajectory;
      const w = chart.width;
      const h = chart.height;
      const pad = 34;
      ctx.clearRect(0, 0, w, h);

      const years = pts.map(p => p.year);
      const minY = Math.min(...years);
      const maxY = Math.max(...years);
      const x = year => pad + ((year - minY) / Math.max(1, (maxY - minY))) * (w - pad * 2);
      const y = v => h - pad - v * (h - pad * 2);

      ctx.strokeStyle = '#ded5c3';
      ctx.beginPath();
      ctx.moveTo(pad, h - pad);
      ctx.lineTo(w - pad, h - pad);
      ctx.moveTo(pad, pad);
      ctx.lineTo(pad, h - pad);
      ctx.stroke();

      function band(lowKey, highKey, color) {{
        ctx.fillStyle = color;
        ctx.beginPath();
        pts.forEach((p, i) => {{
          const px = x(p.year); const py = y(p[highKey]);
          if (i === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py);
        }});
        for (let i = pts.length - 1; i >= 0; i--) {{
          const p = pts[i];
          ctx.lineTo(x(p.year), y(p[lowKey]));
        }}
        ctx.closePath();
        ctx.fill();
      }}

      function line(key, color) {{
        ctx.strokeStyle = color;
        ctx.lineWidth = 2.1;
        ctx.beginPath();
        pts.forEach((p, i) => {{
          const px = x(p.year); const py = y(p[key]);
          if (i === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py);
        }});
        ctx.stroke();
      }}

      band('so_p10', 'so_p90', 'rgba(16, 108, 118, 0.18)');
      band('cx_p10', 'cx_p90', 'rgba(194, 87, 45, 0.18)');
      line('so_p50', '#106c76');
      line('cx_p50', '#c2572d');

      ctx.fillStyle = '#5a5447';
      ctx.font = '12px Georgia';
      s.events.slice(0, 5).forEach((event, idx) => {{
        const px = x(event.year);
        const py = 22 + idx * 14;
        ctx.fillRect(px - 1, pad, 2, h - pad * 2);
        ctx.fillText(event.label, Math.min(px + 4, w - 170), py);
      }});
    }}

    function render(i) {{
      const s = APP_DATA[i];
      document.getElementById('confidence').textContent = s.confidence;
      document.getElementById('robustness').textContent = fmt(s.robustness);
      document.getElementById('fitPop').textContent = fmt(s.fit.population);
      document.getElementById('fitUrban').textContent = fmt(s.fit.urbanization);
      document.getElementById('fitMacro').textContent = fmt((s.fit.gdp + s.fit.energy) / 2);
      document.getElementById('driver').textContent = s.driver;

      const list = document.getElementById('events');
      list.innerHTML = '';
      s.events.forEach(ev => {{
        const li = document.createElement('li');
        li.textContent = ev.year + ': ' + ev.label;
        list.appendChild(li);
      }});

      draw(s);
    }}

    APP_DATA.forEach((s, i) => {{
      const opt = document.createElement('option');
      opt.value = String(i);
      opt.textContent = s.name;
      selector.appendChild(opt);
    }});

    selector.addEventListener('change', e => render(Number(e.target.value)));
    render(0);
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
