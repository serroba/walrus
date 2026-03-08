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

use walrus_engine::evolution::{
    simulate_evolution_with_observer, EvolutionConfig, GenerationFrame, MapEvent, SocietyActor,
};
use walrus_engine::SubsistenceMode;

// ---------------------------------------------------------------------------
// World map bitmap
// ---------------------------------------------------------------------------
// Each char maps to a continent: '0'=Africa, '1'=Eurasia, '2'=Americas,
// '3'=Oceania, ' '=ocean. ~78 cols x 22 rows, simplified Mercator-ish.

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
        '0' => Some(0), // Africa
        '1' => Some(1), // Eurasia
        '2' => Some(2), // Americas
        '3' => Some(3), // Oceania
        _ => None,
    }
}

/// Approximate centroid (col, row) for each continent in the map bitmap.
const CONTINENT_CENTROIDS: [(u16, u16); 4] = [
    (34, 13), // Africa
    (38, 6),  // Eurasia
    (7, 7),   // Americas
    (61, 10), // Oceania
];

const CONTINENT_NAMES: [&str; 4] = ["Africa", "Eurasia", "Americas", "Oceania"];

// ---------------------------------------------------------------------------
// Flash types — per-continent, per-event-type
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct ContinentFlash {
    disaster: u8,  // yellow
    pandemic: u8,  // magenta
    collapse: u8,  // red
    climate: u8,   // orange
    migration: u8, // cyan pulse on destination
}

impl ContinentFlash {
    fn decay(&mut self) {
        self.disaster = self.disaster.saturating_sub(1);
        self.pandemic = self.pandemic.saturating_sub(1);
        self.collapse = self.collapse.saturating_sub(1);
        self.climate = self.climate.saturating_sub(1);
        self.migration = self.migration.saturating_sub(1);
    }

    fn has_any(&self) -> bool {
        self.disaster > 0
            || self.pandemic > 0
            || self.collapse > 0
            || self.climate > 0
            || self.migration > 0
    }
}

// ---------------------------------------------------------------------------
// Color logic
// ---------------------------------------------------------------------------

fn continent_cell_style(
    ci: usize,
    frame: &GenerationFrame,
    flashes: &[ContinentFlash; 4],
) -> Style {
    let pop = per_continent_population(&frame.societies, ci);
    let cap = frame
        .carrying_capacities
        .get(ci)
        .copied()
        .unwrap_or(1.0);
    let depletion = frame.continent_states.get(ci).map_or(0.0, |s| s.depletion);
    let complexity = per_continent_mean(&frame.societies, ci, |s| s.complexity);

    // Base green from population density
    let pop_ratio = (pop as f64 / (cap * 200.0).max(1.0)).clamp(0.0, 1.0);
    let green_base = (80.0 + 160.0 * pop_ratio) as u8;

    // Depletion fades green toward grey
    let depl_factor = 1.0 - depletion.clamp(0.0, 1.0) * 0.7;
    let green = (f64::from(green_base) * depl_factor) as u8;

    // Complexity adds blue tint
    let blue = (complexity.clamp(0.0, 1.0) * 120.0) as u8;

    let mut r = 20_u8;
    let mut g = green;
    let mut b = blue;

    // Flash compositing — each event type adds its color signature
    let fl = &flashes[ci];
    if fl.disaster > 0 {
        let i = fl.disaster.min(6);
        r = r.saturating_add(i * 40); // yellow-orange
        g = g.saturating_add(i * 30);
    }
    if fl.pandemic > 0 {
        let i = fl.pandemic.min(6);
        r = r.saturating_add(i * 30); // magenta
        b = b.saturating_add(i * 35);
    }
    if fl.collapse > 0 {
        let i = fl.collapse.min(8);
        r = r.saturating_add(i * 30); // deep red
        g = g.saturating_sub(i * 8);
        b = b.saturating_sub(i * 8);
    }
    if fl.climate > 0 {
        let i = fl.climate.min(6);
        r = r.saturating_add(i * 35); // orange
        g = g.saturating_add(i * 15);
    }
    if fl.migration > 0 {
        let i = fl.migration.min(4);
        g = g.saturating_add(i * 8); // cyan pulse
        b = b.saturating_add(i * 20);
    }

    Style::default().fg(Color::Rgb(r, g, b))
}

fn per_continent_population(societies: &[SocietyActor], continent: usize) -> u64 {
    societies
        .iter()
        .filter(|s| s.continent == continent)
        .map(|s| u64::from(s.population))
        .sum()
}

fn per_continent_mean(
    societies: &[SocietyActor],
    continent: usize,
    f: fn(&SocietyActor) -> f64,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0_usize;
    for s in societies {
        if s.continent == continent {
            sum += f(s);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

fn per_continent_count(societies: &[SocietyActor], continent: usize) -> usize {
    societies
        .iter()
        .filter(|s| s.continent == continent)
        .count()
}

fn dominant_mode(societies: &[SocietyActor], continent: usize) -> &'static str {
    let mut hg = 0_u32;
    let mut sed = 0_u32;
    let mut ag = 0_u32;
    for s in societies {
        if s.continent == continent {
            match s.mode {
                SubsistenceMode::HunterGatherer => hg += s.population,
                SubsistenceMode::Sedentary => sed += s.population,
                SubsistenceMode::Agriculture => ag += s.population,
            }
        }
    }
    if ag >= sed && ag >= hg {
        "Agriculture"
    } else if sed >= hg {
        "Sedentary"
    } else {
        "Hunter-Gatherer"
    }
}

// ---------------------------------------------------------------------------
// Migration / corridor arrow rendering
// ---------------------------------------------------------------------------

/// Bresenham-ish line: returns cells along the path between two points.
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

/// Choose arrow/line character based on direction.
fn path_char(from: (u16, u16), to: (u16, u16), is_tip: bool) -> char {
    let dx = to.0 as i32 - from.0 as i32;
    let dy = to.1 as i32 - from.1 as i32;
    if is_tip {
        if dx.abs() > dy.abs() {
            if dx > 0 {
                '\u{25B6}'
            } else {
                '\u{25C0}'
            } // ▶ ◀
        } else if dy > 0 {
            '\u{25BC}' // ▼
        } else {
            '\u{25B2}' // ▲
        }
    } else if dx.abs() > dy.abs() * 2 {
        '\u{2500}' // ─
    } else if dy.abs() > dx.abs() * 2 {
        '\u{2502}' // │
    } else if (dx > 0) == (dy > 0) {
        '\u{2572}' // ╲
    } else {
        '\u{2571}' // ╱
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

fn format_mode(mode: SubsistenceMode) -> &'static str {
    match mode {
        SubsistenceMode::HunterGatherer => "HG",
        SubsistenceMode::Sedentary => "Sed",
        SubsistenceMode::Agriculture => "Agr",
    }
}

// ---------------------------------------------------------------------------
// Sparkline helpers
// ---------------------------------------------------------------------------

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
    frame: Option<GenerationFrame>,
    flashes: [ContinentFlash; 4],
    active_migrations: Vec<(usize, usize, u8)>, // (from, to, ttl frames)
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

    fn update(&mut self, frame: GenerationFrame) {
        let gen = frame.snapshot.generation;

        // Process events into flashes and log entries
        for ev in &frame.events {
            match *ev {
                MapEvent::NaturalDisaster {
                    continent,
                    severity,
                } => {
                    if continent < 4 {
                        self.flashes[continent].disaster = 7;
                    }
                    self.log(
                        gen,
                        format!(
                            "DISASTER {}: sev {severity:.2}",
                            CONTINENT_NAMES.get(continent).unwrap_or(&"?")
                        ),
                        Color::Yellow,
                    );
                }
                MapEvent::Pandemic {
                    continent,
                    severity,
                } => {
                    if continent < 4 {
                        self.flashes[continent].pandemic = 7;
                    }
                    self.log(
                        gen,
                        format!(
                            "PANDEMIC {}: sev {severity:.2}",
                            CONTINENT_NAMES.get(continent).unwrap_or(&"?")
                        ),
                        Color::Magenta,
                    );
                }
                MapEvent::ClimateShock {
                    continent,
                    severity,
                } => {
                    if continent < 4 {
                        self.flashes[continent].climate = 6;
                    }
                    self.log(
                        gen,
                        format!(
                            "CLIMATE {}: sev {severity:.2}",
                            CONTINENT_NAMES.get(continent).unwrap_or(&"?")
                        ),
                        Color::Rgb(255, 140, 0),
                    );
                }
                MapEvent::Collapse { continent, .. } => {
                    if continent < 4 {
                        self.flashes[continent].collapse = 9;
                    }
                    self.log(
                        gen,
                        format!(
                            "COLLAPSE {}",
                            CONTINENT_NAMES.get(continent).unwrap_or(&"?")
                        ),
                        Color::Red,
                    );
                }
                MapEvent::Migration { from, to } => {
                    if to < 4 {
                        self.flashes[to].migration = 4;
                    }
                    self.active_migrations.push((from, to, 6));
                    self.log(
                        gen,
                        format!(
                            "MIGRATE {} -> {}",
                            CONTINENT_NAMES.get(from).unwrap_or(&"?"),
                            CONTINENT_NAMES.get(to).unwrap_or(&"?")
                        ),
                        Color::Cyan,
                    );
                }
                MapEvent::ModeTransition {
                    continent,
                    from,
                    to,
                    ..
                } => {
                    self.log(
                        gen,
                        format!(
                            "MODE {} {}->{} ",
                            CONTINENT_NAMES.get(continent).unwrap_or(&"?"),
                            format_mode(from),
                            format_mode(to),
                        ),
                        Color::White,
                    );
                }
            }
        }

        push_bounded(&mut self.pop_history, frame.snapshot.population_total, 60);
        push_bounded(
            &mut self.complexity_history,
            (frame.snapshot.mean_complexity * 1000.0) as u64,
            60,
        );
        push_bounded(
            &mut self.so_history,
            (frame.snapshot.superorganism_index * 1000.0) as u64,
            60,
        );

        self.frame = Some(frame);
    }

    fn decay(&mut self) {
        for fl in &mut self.flashes {
            fl.decay();
        }
        // Decay migration arrow ttl
        for m in &mut self.active_migrations {
            m.2 = m.2.saturating_sub(1);
        }
        self.active_migrations.retain(|m| m.2 > 0);
    }

    fn log(&mut self, generation: u32, text: String, color: Color) {
        self.event_log.push(EventLogEntry {
            generation,
            text,
            color,
        });
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
            f.render_widget(
                Paragraph::new("Waiting for simulation data..."),
                f.area(),
            );
            return;
        }
    };

    // Layout: top row (map + sidebar), bottom row (sparklines + event log)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(MAP_HEIGHT as u16 + 2),
            Constraint::Length(8),
        ])
        .split(f.area());

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(MAP_WIDTH as u16 + 2),
            Constraint::Length(30),
        ])
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
    frame: &GenerationFrame,
    flashes: &[ContinentFlash; 4],
    active_migrations: &[(usize, usize, u8)],
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " World Map | gen {} | pop {} | soc {} ",
            frame.snapshot.generation,
            frame.snapshot.population_total,
            frame.societies.len(),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render continent cells
    for (row_idx, row_str) in MAP_ROWS.iter().enumerate() {
        if row_idx as u16 >= inner.height {
            break;
        }
        let y = inner.y + row_idx as u16;
        for (col_idx, ch) in row_str.chars().enumerate() {
            if col_idx as u16 >= inner.width {
                break;
            }
            let x = inner.x + col_idx as u16;
            if let Some(ci) = continent_index(ch) {
                let style = continent_cell_style(ci, frame, flashes);
                if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(x, y)) {
                    buf_cell.set_char('\u{2588}'); // █
                    buf_cell.set_style(style);
                }
            }
        }
    }

    // Draw corridor arrows (static corridors from the simulation)
    for &(from, to, strength) in &frame.corridor_strengths {
        if strength < 0.05 || from >= 4 || to >= 4 {
            continue;
        }
        let c_from = CONTINENT_CENTROIDS[from];
        let c_to = CONTINENT_CENTROIDS[to];
        let cells = line_cells(c_from, c_to);
        let len = cells.len();
        if len < 2 {
            continue;
        }

        // Draw only a few cells along the path to avoid cluttering
        let step = (len / 4).max(1);
        let intensity = (strength * 200.0).clamp(30.0, 180.0) as u8;
        for (i, &(cx, cy)) in cells.iter().enumerate() {
            if i == 0 || (i % step != 0 && i != len - 1) {
                continue;
            }
            let ax = inner.x + cx;
            let ay = inner.y + cy;
            if ax < inner.x + inner.width && ay < inner.y + inner.height {
                let ch = path_char(c_from, c_to, i == len - 1);
                if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(ax, ay)) {
                    buf_cell.set_char(ch);
                    buf_cell.set_style(
                        Style::default().fg(Color::Rgb(0, intensity / 2, intensity)),
                    );
                }
            }
        }
    }

    // Draw active migration arrows (bright, animated)
    for &(from, to, ttl) in active_migrations {
        if from >= 4 || to >= 4 {
            continue;
        }
        let c_from = CONTINENT_CENTROIDS[from];
        let c_to = CONTINENT_CENTROIDS[to];
        let cells = line_cells(c_from, c_to);
        let len = cells.len();
        if len < 2 {
            continue;
        }

        // Animate: show progressively more of the path as ttl decreases
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
                    buf_cell.set_style(
                        Style::default().fg(Color::Rgb(brightness, 255, 255)).bold(),
                    );
                }
            }
        }
    }

    // Continent labels at centroids
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

fn render_sidebar(f: &mut Frame, area: Rect, frame: &GenerationFrame, state: &TuiState) {
    let block = Block::default().borders(Borders::ALL).title(" Stats ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (ci, name) in CONTINENT_NAMES.iter().enumerate() {
        let pop = per_continent_population(&frame.societies, ci);
        let soc = per_continent_count(&frame.societies, ci);
        let cx = per_continent_mean(&frame.societies, ci, |s| s.complexity);
        let depl = frame.continent_states.get(ci).map_or(0.0, |s| s.depletion);
        let mode = dominant_mode(&frame.societies, ci);

        let name_color = match ci {
            0 => Color::Yellow,
            1 => Color::Green,
            2 => Color::Cyan,
            3 => Color::Magenta,
            _ => Color::White,
        };

        // Compact: name + key stats on fewer lines
        lines.push(Line::from(vec![
            Span::styled(name.to_string(), Style::default().fg(name_color).bold()),
            Span::raw(format!(" p:{pop} s:{soc}")),
        ]));
        lines.push(Line::from(format!(
            " cx:{cx:.2} dep:{depl:.2} {mode}"
        )));
    }

    lines.push(Line::from(""));

    // Global
    lines.push(Line::from(vec![
        Span::styled("Global", Style::default().fg(Color::White).bold()),
    ]));
    lines.push(Line::from(format!(
        " SO:{:.3} CX:{:.3}",
        frame.snapshot.superorganism_index, frame.snapshot.mean_complexity,
    )));
    lines.push(Line::from(format!(
        " conv:{:.3} civ:{}",
        frame.snapshot.convergence_index, frame.snapshot.emergent_civilizations,
    )));

    // Active event indicators
    let mut indicators: Vec<Span> = Vec::new();
    if frame.snapshot.collapse_events > 0 {
        indicators.push(Span::styled(
            format!(" C:{}", frame.snapshot.collapse_events),
            Style::default().fg(Color::Red).bold(),
        ));
    }
    if frame.snapshot.natural_disaster_events > 0 {
        indicators.push(Span::styled(
            format!(" D:{}", frame.snapshot.natural_disaster_events),
            Style::default().fg(Color::Yellow),
        ));
    }
    if frame.snapshot.pandemic_events > 0 {
        indicators.push(Span::styled(
            format!(" P:{}", frame.snapshot.pandemic_events),
            Style::default().fg(Color::Magenta),
        ));
    }
    if !indicators.is_empty() {
        lines.push(Line::from(indicators));
    }

    lines.push(Line::from(""));
    let status = if state.paused { "PAUSED" } else { "RUNNING" };
    lines.push(Line::from(vec![Span::styled(
        format!("[{status}] {:.0}ms", state.speed_ms),
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "q:quit p:pause +/-:spd",
        Style::default().fg(Color::DarkGray),
    )]));

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

fn render_timeline(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(area);

    // Top row: population + complexity side by side
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let pop_spark = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Population "),
        )
        .data(&state.pop_history)
        .style(Style::default().fg(Color::Green));
    f.render_widget(pop_spark, top_row[0]);

    let cx_spark = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Complexity "),
        )
        .data(&state.complexity_history)
        .style(Style::default().fg(Color::Blue));
    f.render_widget(cx_spark, top_row[1]);

    // Bottom row: superorganism index
    let so_spark = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Superorganism Index "),
        )
        .data(&state.so_history)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(so_spark, chunks[1]);
}

fn render_event_log(f: &mut Frame, area: Rect, state: &TuiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Event Log ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible = inner.height as usize;
    let start = state.event_log.len().saturating_sub(visible);
    let lines: Vec<Line> = state.event_log[start..]
        .iter()
        .map(|entry| {
            Line::from(vec![
                Span::styled(
                    format!("{:>3} ", entry.generation),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&entry.text, Style::default().fg(entry.color)),
            ])
        })
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    let (tx, rx) = mpsc::channel::<GenerationFrame>();

    let sim_handle = thread::spawn(move || {
        let config = EvolutionConfig {
            generations: 600,
            initial_societies: 24,
            ..EvolutionConfig::default()
        };
        let _ = simulate_evolution_with_observer(config, |frame| {
            let _ = tx.send(frame.clone());
        });
    });

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = TuiState::new();

    loop {
        // Drain channel to latest frame
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

        if sim_handle.is_finished() && state.frame.is_some() {
            // Simulation complete — keep rendering for inspection
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
