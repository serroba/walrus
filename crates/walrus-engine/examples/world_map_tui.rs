use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};

use walrus_engine::agents::AgentSimConfig;
use walrus_engine::evolution::{
    simulate_evolution_with_observer, EvolutionConfig, GenerationFrame, MapEvent,
};
use walrus_engine::event_sim::{
    simulate_event_driven_with_observer, EventMapFrame, EventSimConfig,
};

// ---------------------------------------------------------------------------
// World map bitmap
// ---------------------------------------------------------------------------

const MAP_ROWS: &[&str] = &[
    "                                                                              ",
    "                                                                              ",
    "              1111                                                            ",
    "    222      111111111111111111111                                             ",
    "   22222    1111111111111111111111111                                          ",
    "   222222   11111111111111111111111111111                                      ",
    "   2222222   1111111111111111111111111111111                                   ",
    "    2222222   11111 111111111111111111111111                                   ",
    "    22222222   1111  1111111111111111111111          33                        ",
    "     2222222    111   00001111111111111111          3333                       ",
    "      222222          000001111111111111            33333                      ",
    "       22222          0000000 11111111               3333                      ",
    "        2222           00000000  111                  33                       ",
    "         222           000000000                                               ",
    "          22            00000000                                               ",
    "           2             0000000                                               ",
    "                          000000                                               ",
    "                           0000                                                ",
    "                            000                                                ",
    "                             0                                                 ",
    "                                                                              ",
    "                                                                              ",
];

const MAP_HEIGHT: usize = MAP_ROWS.len();
const MAP_WIDTH: usize = 78;

fn continent_index(ch: char) -> Option<usize> {
    match ch {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        _ => None,
    }
}

const CONTINENT_CENTROIDS: [(u16, u16); 4] = [
    (34, 13), // Africa
    (38, 6),  // Eurasia
    (7, 7),   // Americas
    (61, 10), // Oceania
];

const CONTINENT_NAMES: [&str; 4] = ["Africa", "Eurasia", "Americas", "Oceania"];

// ---------------------------------------------------------------------------
// Unified frame — abstracts over evolution vs event-driven simulation
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ContinentStats {
    population: u64,
    society_count: usize,
    mean_complexity: f64,
    depletion: f64,
    carrying_capacity: f64,
    dominant_mode: &'static str,
    mean_resources: f32,
    cooperation_count: u32,
    conflict_count: u32,
}

#[derive(Clone)]
struct UnifiedFrame {
    generation: u32,
    #[allow(dead_code)]
    time: f64,
    total_population: u64,
    total_societies: usize,
    superorganism_index: f64,
    mean_complexity: f64,
    convergence_index: f64,
    emergent_civilizations: u32,
    collapse_events: u32,
    disaster_events: u32,
    pandemic_events: u32,
    continents: [ContinentStats; 4],
    corridors: Vec<(usize, usize, f64)>,
    events: Vec<TuiEvent>,
    mode_label: &'static str,
}

#[derive(Clone)]
enum TuiEvent {
    Disaster { continent: usize, severity: f64 },
    Pandemic { continent: usize, severity: f64 },
    Climate { continent: usize, severity: f64 },
    Collapse { continent: usize },
    Migration { from: usize, to: usize },
    ModeTransition { continent: usize, from: &'static str, to: &'static str },
    Raid { count: u32 },
}

fn mode_str(mode: walrus_engine::SubsistenceMode) -> &'static str {
    match mode {
        walrus_engine::SubsistenceMode::HunterGatherer => "HG",
        walrus_engine::SubsistenceMode::Sedentary => "Sed",
        walrus_engine::SubsistenceMode::Agriculture => "Agr",
    }
}

impl From<GenerationFrame> for UnifiedFrame {
    fn from(f: GenerationFrame) -> Self {
        let mut continents = [
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
        ];

        for (ci, slot) in continents.iter_mut().enumerate() {
            let socs: Vec<_> = f.societies.iter().filter(|s| s.continent == ci).collect();
            let pop: u64 = socs.iter().map(|s| u64::from(s.population)).sum();
            let cx = if socs.is_empty() { 0.0 } else {
                socs.iter().map(|s| s.complexity).sum::<f64>() / socs.len() as f64
            };
            let depl = f.continent_states.get(ci).map_or(0.0, |s| s.depletion);
            let cap = f.carrying_capacities.get(ci).copied().unwrap_or(1.0);

            // Dominant mode by population weight
            let (mut hg, mut sed, mut ag) = (0u64, 0u64, 0u64);
            for s in &socs {
                match s.mode {
                    walrus_engine::SubsistenceMode::HunterGatherer => hg += u64::from(s.population),
                    walrus_engine::SubsistenceMode::Sedentary => sed += u64::from(s.population),
                    walrus_engine::SubsistenceMode::Agriculture => ag += u64::from(s.population),
                }
            }
            let mode = if ag >= sed && ag >= hg { "Agr" }
                else if sed >= hg { "Sed" }
                else { "HG" };

            *slot = ContinentStats {
                population: pop,
                society_count: socs.len(),
                mean_complexity: cx,
                depletion: depl,
                carrying_capacity: cap,
                dominant_mode: mode,
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            };
        }

        let events: Vec<TuiEvent> = f.events.iter().map(|e| match *e {
            MapEvent::NaturalDisaster { continent, severity } => TuiEvent::Disaster { continent, severity },
            MapEvent::Pandemic { continent, severity } => TuiEvent::Pandemic { continent, severity },
            MapEvent::ClimateShock { continent, severity } => TuiEvent::Climate { continent, severity },
            MapEvent::Collapse { continent, .. } => TuiEvent::Collapse { continent },
            MapEvent::Migration { from, to } => TuiEvent::Migration { from, to },
            MapEvent::ModeTransition { continent, from, to, .. } => TuiEvent::ModeTransition {
                continent, from: mode_str(from), to: mode_str(to),
            },
        }).collect();

        UnifiedFrame {
            generation: f.snapshot.generation,
            time: f64::from(f.snapshot.generation),
            total_population: f.snapshot.population_total,
            total_societies: f.societies.len(),
            superorganism_index: f.snapshot.superorganism_index,
            mean_complexity: f.snapshot.mean_complexity,
            convergence_index: f.snapshot.convergence_index,
            emergent_civilizations: f.snapshot.emergent_civilizations,
            collapse_events: f.snapshot.collapse_events,
            disaster_events: f.snapshot.natural_disaster_events,
            pandemic_events: f.snapshot.pandemic_events,
            continents,
            corridors: f.corridor_strengths,
            events,
            mode_label: "evolution",
        }
    }
}

impl From<EventMapFrame> for UnifiedFrame {
    fn from(f: EventMapFrame) -> Self {
        let mut continents = [
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
            ContinentStats { population: 0, society_count: 0, mean_complexity: 0.0, depletion: 0.0, carrying_capacity: 1.0, dominant_mode: "HG", mean_resources: 0.0, cooperation_count: 0, conflict_count: 0 },
        ];

        for (ci, slot) in continents.iter_mut().enumerate() {
            *slot = ContinentStats {
                population: u64::from(f.continent_populations[ci]),
                society_count: 0, // event sim doesn't have society-level grouping
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "",
                mean_resources: f.continent_mean_resources[ci],
                cooperation_count: f.continent_cooperation_counts[ci],
                conflict_count: f.continent_conflict_counts[ci],
            };
        }

        let mut events = Vec::new();
        if f.raid_events > 0 {
            events.push(TuiEvent::Raid { count: f.raid_events });
        }
        if f.migration_events > 0 {
            // Distribute migrations across corridors heuristically
            for _ in 0..f.migration_events.min(4) {
                events.push(TuiEvent::Migration { from: 1, to: 0 });
            }
        }

        // Map institutional type to complexity proxy
        let inst_type = f.emergent.institutional_type;
        let complexity_proxy = match inst_type {
            0 => 0.1, // band
            1 => 0.35, // tribe
            2 => 0.65, // chiefdom
            _ => 0.9, // state
        };
        for slot in &mut continents {
            slot.mean_complexity = complexity_proxy;
        }

        UnifiedFrame {
            generation: f.time as u32,
            time: f.time,
            total_population: u64::from(f.total_population),
            total_societies: f.emergent.num_active_societies as usize,
            superorganism_index: 0.0, // computed differently in event sim
            mean_complexity: f64::from(f.emergent.mean_innovation),
            convergence_index: 0.0,
            emergent_civilizations: 0,
            collapse_events: 0,
            disaster_events: 0,
            pandemic_events: 0,
            continents,
            corridors: Vec::new(), // agent sim has no explicit corridors
            events,
            mode_label: "event-driven",
        }
    }
}

// ---------------------------------------------------------------------------
// Flash types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct ContinentFlash {
    disaster: u8,
    pandemic: u8,
    collapse: u8,
    climate: u8,
    migration: u8,
    raid: u8,
}

impl ContinentFlash {
    fn decay(&mut self) {
        self.disaster = self.disaster.saturating_sub(1);
        self.pandemic = self.pandemic.saturating_sub(1);
        self.collapse = self.collapse.saturating_sub(1);
        self.climate = self.climate.saturating_sub(1);
        self.migration = self.migration.saturating_sub(1);
        self.raid = self.raid.saturating_sub(1);
    }

    fn has_any(&self) -> bool {
        self.disaster > 0
            || self.pandemic > 0
            || self.collapse > 0
            || self.climate > 0
            || self.migration > 0
            || self.raid > 0
    }
}

// ---------------------------------------------------------------------------
// Color logic
// ---------------------------------------------------------------------------

fn continent_cell_style(
    ci: usize,
    frame: &UnifiedFrame,
    flashes: &[ContinentFlash; 4],
) -> Style {
    let c = &frame.continents[ci];
    let pop_ratio = (c.population as f64 / (c.carrying_capacity * 200.0).max(1.0)).clamp(0.0, 1.0);
    let green_base = (80.0 + 160.0 * pop_ratio) as u8;
    let depl_factor = 1.0 - c.depletion.clamp(0.0, 1.0) * 0.7;
    let green = (f64::from(green_base) * depl_factor) as u8;
    let blue = (c.mean_complexity.clamp(0.0, 1.0) * 120.0) as u8;

    let mut r = 20_u8;
    let mut g = green;
    let mut b = blue;

    let fl = &flashes[ci];
    if fl.disaster > 0 {
        let i = fl.disaster.min(6);
        r = r.saturating_add(i * 40);
        g = g.saturating_add(i * 30);
    }
    if fl.pandemic > 0 {
        let i = fl.pandemic.min(6);
        r = r.saturating_add(i * 30);
        b = b.saturating_add(i * 35);
    }
    if fl.collapse > 0 {
        let i = fl.collapse.min(8);
        r = r.saturating_add(i * 30);
        g = g.saturating_sub(i * 8);
        b = b.saturating_sub(i * 8);
    }
    if fl.climate > 0 {
        let i = fl.climate.min(6);
        r = r.saturating_add(i * 35);
        g = g.saturating_add(i * 15);
    }
    if fl.migration > 0 {
        let i = fl.migration.min(4);
        g = g.saturating_add(i * 8);
        b = b.saturating_add(i * 20);
    }
    if fl.raid > 0 {
        let i = fl.raid.min(6);
        r = r.saturating_add(i * 35);
        g = g.saturating_sub(i * 5);
    }

    Style::default().fg(Color::Rgb(r, g, b))
}

// ---------------------------------------------------------------------------
// Arrow rendering
// ---------------------------------------------------------------------------

fn line_cells(from: (u16, u16), to: (u16, u16)) -> Vec<(u16, u16)> {
    let mut cells = Vec::new();
    let dx = (to.0 as i32 - from.0 as i32).abs();
    let dy = (to.1 as i32 - from.1 as i32).abs();
    let sx: i32 = if from.0 < to.0 { 1 } else { -1 };
    let sy: i32 = if from.1 < to.1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = from.0 as i32;
    let mut y = from.1 as i32;
    let tx = to.0 as i32;
    let ty = to.1 as i32;
    for _ in 0..200 {
        cells.push((x as u16, y as u16));
        if x == tx && y == ty {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
    cells
}

fn path_char(from: (u16, u16), to: (u16, u16), is_tip: bool) -> char {
    let dx = to.0 as i32 - from.0 as i32;
    let dy = to.1 as i32 - from.1 as i32;
    if is_tip {
        if dx.abs() > dy.abs() {
            if dx > 0 { '\u{25B6}' } else { '\u{25C0}' }
        } else if dy > 0 {
            '\u{25BC}'
        } else {
            '\u{25B2}'
        }
    } else if dx.abs() > dy.abs() * 2 {
        '\u{2500}'
    } else if dy.abs() > dx.abs() * 2 {
        '\u{2502}'
    } else if (dx > 0) == (dy > 0) {
        '\u{2572}'
    } else {
        '\u{2571}'
    }
}

// ---------------------------------------------------------------------------
// Event log
// ---------------------------------------------------------------------------

struct EventLogEntry {
    generation: u32,
    text: String,
    color: Color,
}

fn push_bounded(buf: &mut Vec<u64>, val: u64, max_len: usize) {
    buf.push(val);
    if buf.len() > max_len {
        buf.remove(0);
    }
}

// ---------------------------------------------------------------------------
// TUI state
// ---------------------------------------------------------------------------

struct TuiState {
    frame: Option<UnifiedFrame>,
    flashes: [ContinentFlash; 4],
    active_migrations: Vec<(usize, usize, u8)>,
    event_log: Vec<EventLogEntry>,
    pop_history: Vec<u64>,
    complexity_history: Vec<u64>,
    so_history: Vec<u64>,
    paused: bool,
    speed_ms: u64,
}

const MAX_LOG_ENTRIES: usize = 50;

impl TuiState {
    fn new() -> Self {
        Self {
            frame: None,
            flashes: [ContinentFlash::default(); 4],
            active_migrations: Vec::new(),
            event_log: Vec::new(),
            pop_history: Vec::new(),
            complexity_history: Vec::new(),
            so_history: Vec::new(),
            paused: false,
            speed_ms: 80,
        }
    }

    fn update(&mut self, frame: UnifiedFrame) {
        let gen = frame.generation;

        for ev in &frame.events {
            match *ev {
                TuiEvent::Disaster { continent, severity } => {
                    if continent < 4 { self.flashes[continent].disaster = 7; }
                    self.log(gen, format!("DISASTER {}: sev {severity:.2}",
                        CONTINENT_NAMES.get(continent).unwrap_or(&"?")), Color::Yellow);
                }
                TuiEvent::Pandemic { continent, severity } => {
                    if continent < 4 { self.flashes[continent].pandemic = 7; }
                    self.log(gen, format!("PANDEMIC {}: sev {severity:.2}",
                        CONTINENT_NAMES.get(continent).unwrap_or(&"?")), Color::Magenta);
                }
                TuiEvent::Climate { continent, severity } => {
                    if continent < 4 { self.flashes[continent].climate = 6; }
                    self.log(gen, format!("CLIMATE {}: sev {severity:.2}",
                        CONTINENT_NAMES.get(continent).unwrap_or(&"?")), Color::Rgb(255, 140, 0));
                }
                TuiEvent::Collapse { continent } => {
                    if continent < 4 { self.flashes[continent].collapse = 9; }
                    self.log(gen, format!("COLLAPSE {}",
                        CONTINENT_NAMES.get(continent).unwrap_or(&"?")), Color::Red);
                }
                TuiEvent::Migration { from, to } => {
                    if to < 4 { self.flashes[to].migration = 4; }
                    self.active_migrations.push((from, to, 6));
                    self.log(gen, format!("MIGRATE {} -> {}",
                        CONTINENT_NAMES.get(from).unwrap_or(&"?"),
                        CONTINENT_NAMES.get(to).unwrap_or(&"?")), Color::Cyan);
                }
                TuiEvent::ModeTransition { continent, from, to } => {
                    self.log(gen, format!("MODE {} {from}->{to}",
                        CONTINENT_NAMES.get(continent).unwrap_or(&"?")), Color::White);
                }
                TuiEvent::Raid { count } => {
                    // Flash all continents with some raid activity
                    for fl in &mut self.flashes {
                        fl.raid = 5;
                    }
                    self.log(gen, format!("RAIDS: {count}"), Color::Red);
                }
            }
        }

        push_bounded(&mut self.pop_history, frame.total_population, 60);
        push_bounded(&mut self.complexity_history,
            (frame.mean_complexity * 1000.0) as u64, 60);
        push_bounded(&mut self.so_history,
            (frame.superorganism_index * 1000.0) as u64, 60);

        self.frame = Some(frame);
    }

    fn decay(&mut self) {
        for fl in &mut self.flashes {
            fl.decay();
        }
        for m in &mut self.active_migrations {
            m.2 = m.2.saturating_sub(1);
        }
        self.active_migrations.retain(|m| m.2 > 0);
    }

    fn log(&mut self, generation: u32, text: String, color: Color) {
        self.event_log.push(EventLogEntry { generation, text, color });
        if self.event_log.len() > MAX_LOG_ENTRIES {
            self.event_log.remove(0);
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(f: &mut Frame, state: &TuiState) {
    let frame = match &state.frame {
        Some(fr) => fr,
        None => {
            f.render_widget(Paragraph::new("Waiting for simulation data..."), f.area());
            return;
        }
    };

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(MAP_HEIGHT as u16 + 2), Constraint::Length(8)])
        .split(f.area());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(MAP_WIDTH as u16 + 2), Constraint::Length(30)])
        .split(outer[0]);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(outer[1]);

    render_map(f, top[0], frame, &state.flashes, &state.active_migrations);
    render_sidebar(f, top[1], frame, state);
    render_timeline(f, bottom[0], state);
    render_event_log(f, bottom[1], state);
}

fn render_map(
    f: &mut Frame,
    area: Rect,
    frame: &UnifiedFrame,
    flashes: &[ContinentFlash; 4],
    active_migrations: &[(usize, usize, u8)],
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " World Map [{mode}] | t={gen} | pop {pop} | soc {soc} ",
            mode = frame.mode_label,
            gen = frame.generation,
            pop = frame.total_population,
            soc = frame.total_societies,
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    for (row_idx, row_str) in MAP_ROWS.iter().enumerate() {
        if row_idx as u16 >= inner.height { break; }
        let y = inner.y + row_idx as u16;
        for (col_idx, ch) in row_str.chars().enumerate() {
            if col_idx as u16 >= inner.width { break; }
            let x = inner.x + col_idx as u16;
            if let Some(ci) = continent_index(ch) {
                let style = continent_cell_style(ci, frame, flashes);
                if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(x, y)) {
                    buf_cell.set_char('\u{2588}');
                    buf_cell.set_style(style);
                }
            }
        }
    }

    // Static corridor arrows
    for &(from, to, strength) in &frame.corridors {
        if strength < 0.05 || from >= 4 || to >= 4 { continue; }
        let c_from = CONTINENT_CENTROIDS[from];
        let c_to = CONTINENT_CENTROIDS[to];
        let cells = line_cells(c_from, c_to);
        let len = cells.len();
        if len < 2 { continue; }
        let step = (len / 4).max(1);
        let intensity = (strength * 200.0).clamp(30.0, 180.0) as u8;
        for (i, &(cx, cy)) in cells.iter().enumerate() {
            if i == 0 || (i % step != 0 && i != len - 1) { continue; }
            let ax = inner.x + cx;
            let ay = inner.y + cy;
            if ax < inner.x + inner.width && ay < inner.y + inner.height {
                let ch = path_char(c_from, c_to, i == len - 1);
                if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(ax, ay)) {
                    buf_cell.set_char(ch);
                    buf_cell.set_style(Style::default().fg(Color::Rgb(0, intensity / 2, intensity)));
                }
            }
        }
    }

    // Active migration arrows
    for &(from, to, ttl) in active_migrations {
        if from >= 4 || to >= 4 { continue; }
        let c_from = CONTINENT_CENTROIDS[from];
        let c_to = CONTINENT_CENTROIDS[to];
        let cells = line_cells(c_from, c_to);
        let len = cells.len();
        if len < 2 { continue; }
        let progress = 1.0 - (f64::from(ttl) / 6.0);
        let visible = ((len as f64) * progress).ceil() as usize;
        let brightness = (ttl as u16 * 40).min(255) as u8;
        for (i, &(cx, cy)) in cells.iter().enumerate().take(visible) {
            let ax = inner.x + cx;
            let ay = inner.y + cy;
            if ax < inner.x + inner.width && ay < inner.y + inner.height {
                let is_tip = i == visible.saturating_sub(1);
                let ch = path_char(c_from, c_to, is_tip);
                if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(ax, ay)) {
                    buf_cell.set_char(ch);
                    buf_cell.set_style(Style::default().fg(Color::Rgb(brightness, 255, 255)).bold());
                }
            }
        }
    }

    // Continent labels
    for (ci, name) in CONTINENT_NAMES.iter().enumerate() {
        let (cx, cy) = CONTINENT_CENTROIDS[ci];
        let label_x = inner.x + cx.saturating_sub(name.len() as u16 / 2);
        let label_y = inner.y + cy + 1;
        if label_y < inner.y + inner.height {
            let has_flash = flashes[ci].has_any();
            let style = if has_flash {
                Style::default().fg(Color::White).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            for (i, ch) in name.chars().enumerate() {
                let lx = label_x + i as u16;
                if lx < inner.x + inner.width {
                    if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(lx, label_y)) {
                        buf_cell.set_char(ch);
                        buf_cell.set_style(style);
                    }
                }
            }
        }
    }
}

fn render_sidebar(f: &mut Frame, area: Rect, frame: &UnifiedFrame, state: &TuiState) {
    let block = Block::default().borders(Borders::ALL).title(" Stats ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (ci, name) in CONTINENT_NAMES.iter().enumerate() {
        let c = &frame.continents[ci];
        let name_color = match ci {
            0 => Color::Yellow,
            1 => Color::Green,
            2 => Color::Cyan,
            3 => Color::Magenta,
            _ => Color::White,
        };

        lines.push(Line::from(vec![
            Span::styled(name.to_string(), Style::default().fg(name_color).bold()),
            Span::raw(format!(" p:{} s:{}", c.population, c.society_count)),
        ]));

        if !c.dominant_mode.is_empty() {
            lines.push(Line::from(format!(
                " cx:{:.2} dep:{:.2} {}", c.mean_complexity, c.depletion, c.dominant_mode
            )));
        } else {
            // Event-driven mode: show resources + interactions
            lines.push(Line::from(format!(
                " res:{:.1} c:{}/f:{}", c.mean_resources, c.cooperation_count, c.conflict_count
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Global", Style::default().fg(Color::White).bold()),
    ]));
    lines.push(Line::from(format!(
        " SO:{:.3} CX:{:.3}", frame.superorganism_index, frame.mean_complexity,
    )));
    if frame.convergence_index > 0.0 {
        lines.push(Line::from(format!(
            " conv:{:.3} civ:{}", frame.convergence_index, frame.emergent_civilizations,
        )));
    }

    let mut indicators: Vec<Span> = Vec::new();
    if frame.collapse_events > 0 {
        indicators.push(Span::styled(
            format!(" C:{}", frame.collapse_events), Style::default().fg(Color::Red).bold()));
    }
    if frame.disaster_events > 0 {
        indicators.push(Span::styled(
            format!(" D:{}", frame.disaster_events), Style::default().fg(Color::Yellow)));
    }
    if frame.pandemic_events > 0 {
        indicators.push(Span::styled(
            format!(" P:{}", frame.pandemic_events), Style::default().fg(Color::Magenta)));
    }
    if !indicators.is_empty() {
        lines.push(Line::from(indicators));
    }

    lines.push(Line::from(""));
    let status = if state.paused { "PAUSED" } else { "RUNNING" };
    lines.push(Line::from(vec![Span::styled(
        format!("[{status}] {:.0}ms", state.speed_ms), Style::default().fg(Color::DarkGray))]));
    lines.push(Line::from(vec![Span::styled(
        "q:quit p:pause +/-:spd", Style::default().fg(Color::DarkGray))]));

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_timeline(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(4)])
        .split(area);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    f.render_widget(
        Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(" Population "))
            .data(&state.pop_history)
            .style(Style::default().fg(Color::Green)),
        top_row[0],
    );
    f.render_widget(
        Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(" Complexity "))
            .data(&state.complexity_history)
            .style(Style::default().fg(Color::Blue)),
        top_row[1],
    );
    f.render_widget(
        Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title(" Superorganism "))
            .data(&state.so_history)
            .style(Style::default().fg(Color::Cyan)),
        chunks[1],
    );
}

fn render_event_log(f: &mut Frame, area: Rect, state: &TuiState) {
    let block = Block::default().borders(Borders::ALL).title(" Event Log ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible = inner.height as usize;
    let start = state.event_log.len().saturating_sub(visible);
    let lines: Vec<Line> = state.event_log[start..]
        .iter()
        .map(|entry| {
            Line::from(vec![
                Span::styled(format!("{:>3} ", entry.generation), Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.text, Style::default().fg(entry.color)),
            ])
        })
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------------------
// Simulation modes
// ---------------------------------------------------------------------------

enum SimMode {
    Evolution,
    EventDriven,
}

fn parse_mode() -> SimMode {
    for arg in std::env::args() {
        if arg == "--events" || arg == "--event-driven" {
            return SimMode::EventDriven;
        }
    }
    // Also check env var
    if std::env::var("EVENT_DRIVEN").unwrap_or_default() == "true" {
        return SimMode::EventDriven;
    }
    SimMode::Evolution
}

fn run_evolution(tx: mpsc::Sender<UnifiedFrame>) {
    let config = EvolutionConfig {
        generations: 600,
        initial_societies: 24,
        ..EvolutionConfig::default()
    };
    let _ = simulate_evolution_with_observer(config, |frame| {
        let unified: UnifiedFrame = frame.clone().into();
        let _ = tx.send(unified);
    });
}

fn run_event_driven(tx: mpsc::Sender<UnifiedFrame>) {
    let config = EventSimConfig {
        agent: AgentSimConfig {
            initial_population: 200,
            world_size: 80.0,
            ..AgentSimConfig::default()
        },
        event: walrus_engine::event_sim::EventParams {
            measure_interval: 2.0,
            ..Default::default()
        },
        end_time: 500.0,
    };
    let _ = simulate_event_driven_with_observer(config, |frame| {
        let unified: UnifiedFrame = frame.clone().into();
        let _ = tx.send(unified);
    });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let mode = parse_mode();
    let (tx, rx) = mpsc::channel::<UnifiedFrame>();

    let _sim_handle = thread::spawn(move || match mode {
        SimMode::Evolution => run_evolution(tx),
        SimMode::EventDriven => run_event_driven(tx),
    });

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = TuiState::new();

    loop {
        while let Ok(frame) = rx.try_recv() {
            state.update(frame);
        }

        terminal.draw(|f| render(f, &state))?;
        state.decay();

        if event::poll(Duration::from_millis(state.speed_ms))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => state.paused = !state.paused,
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            state.speed_ms = state.speed_ms.saturating_sub(20).max(10);
                        }
                        KeyCode::Char('-') => {
                            state.speed_ms = (state.speed_ms + 20).min(500);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
