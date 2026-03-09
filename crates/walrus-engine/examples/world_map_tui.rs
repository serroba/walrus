use std::collections::VecDeque;
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
use walrus_engine::event_sim::{
    simulate_event_driven_with_observer, EventMapFrame, EventSimConfig,
};
use walrus_engine::evolution::{
    simulate_evolution_with_observer, DunbarBehaviorModel, EvolutionConfig, GenerationFrame,
    MapEvent,
};

// ---------------------------------------------------------------------------
// TOML configuration
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlConfig {
    simulation: TomlSimulation,
    evolution: TomlEvolution,
    events: TomlEvents,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlSimulation {
    mode: Option<String>,
    seed: Option<u64>,
    format: Option<String>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlEvolution {
    generations: Option<u32>,
    initial_societies: Option<u32>,
    isolation_factor: Option<f64>,
    resource_multiplier: Option<f64>,
    natural_disaster_base_rate: Option<f64>,
    pandemic_base_rate: Option<f64>,
    nk_n: Option<usize>,
    nk_k: Option<usize>,
    population_range: Option<[u32; 2]>,
    initial_complexity_range: Option<[f64; 2]>,
    dunbar: Option<TomlDunbar>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlDunbar {
    thresholds: Option<[u32; 6]>,
    expectation_load: Option<[f64; 6]>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlEvents {
    end_time: Option<f64>,
    measure_interval: Option<f64>,
    agent: Option<TomlAgent>,
    energy: Option<TomlEnergy>,
    interaction: Option<TomlInteraction>,
    lifecycle: Option<TomlLifecycle>,
    inter_society: Option<TomlInterSociety>,
    cultural: Option<TomlCultural>,
    institution: Option<TomlInstitution>,
    rates: Option<TomlRates>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlAgent {
    initial_population: Option<u32>,
    world_size: Option<f32>,
    ticks: Option<u32>,
    interaction_radius: Option<f32>,
    max_age: Option<u16>,
    min_population: Option<u32>,
    max_population: Option<u32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlEnergy {
    biomass_base_eroei: Option<f64>,
    biomass_regen_rate: Option<f64>,
    agriculture_base_eroei: Option<f64>,
    agriculture_tech_threshold: Option<f32>,
    agriculture_fertility_prob: Option<f64>,
    fossil_base_eroei: Option<f64>,
    fossil_tech_threshold: Option<f32>,
    renewable_base_eroei: Option<f64>,
    renewable_tech_threshold: Option<f32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlInteraction {
    coop_self_weight: Option<f32>,
    coop_other_weight: Option<f32>,
    coop_kin_bonus: Option<f32>,
    conflict_self_weight: Option<f32>,
    conflict_stranger_bonus: Option<f32>,
    subsistence_level: Option<f32>,
    trust_coop_weight: Option<f32>,
    trust_memory_decay: Option<f32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlLifecycle {
    birth_rate: Option<f32>,
    death_health_threshold: Option<f32>,
    starvation_resource_threshold: Option<f32>,
    starvation_death_prob: Option<f32>,
    reproduction_resource_threshold: Option<f32>,
    innovation_growth_rate: Option<f32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlInterSociety {
    min_raid_warriors: Option<u32>,
    raid_aggression_threshold: Option<f32>,
    raid_range: Option<f32>,
    conquest_power_ratio: Option<f32>,
    tribute_rate: Option<f32>,
    migration_resource_threshold: Option<f32>,
    migration_probability: Option<f32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlCultural {
    vertical_mutation_prob: Option<f64>,
    horizontal_adoption_prob: Option<f32>,
    oblique_adoption_prob: Option<f32>,
    oblique_prestige_gap: Option<f32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlInstitution {
    public_goods_rate: Option<f32>,
    defense_bonus: Option<f32>,
    leadership_threshold: Option<f32>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct TomlRates {
    forage_base_rate: Option<f64>,
    interact_base_rate: Option<f64>,
    move_base_rate: Option<f64>,
    reproduce_base_rate: Option<f64>,
    raid_base_rate: Option<f64>,
    migrate_base_rate: Option<f64>,
}

/// Apply Option<T> to a mutable field: if Some, overwrite.
macro_rules! apply {
    ($target:expr, $source:expr) => {
        if let Some(v) = $source {
            $target = v;
        }
    };
}

impl TomlConfig {
    fn load(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Warning: failed to parse {path}: {e}");
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    fn to_evolution_config(&self, seed: u64) -> EvolutionConfig {
        let e = &self.evolution;
        let mut cfg = EvolutionConfig {
            seed,
            ..EvolutionConfig::default()
        };
        apply!(cfg.generations, e.generations);
        apply!(cfg.initial_societies, e.initial_societies);
        apply!(cfg.isolation_factor, e.isolation_factor);
        apply!(cfg.resource_multiplier, e.resource_multiplier);
        apply!(cfg.natural_disaster_base_rate, e.natural_disaster_base_rate);
        apply!(cfg.pandemic_base_rate, e.pandemic_base_rate);
        apply!(cfg.nk_n, e.nk_n);
        apply!(cfg.nk_k, e.nk_k);
        if let Some(pr) = e.population_range {
            cfg.population_range = (pr[0], pr[1]);
        }
        if let Some(cr) = e.initial_complexity_range {
            cfg.initial_complexity_range = (cr[0], cr[1]);
        }
        if let Some(ref d) = e.dunbar {
            let mut dm = DunbarBehaviorModel::default();
            apply!(dm.thresholds, d.thresholds);
            apply!(dm.expectation_load, d.expectation_load);
            cfg.dunbar_model = dm;
        }
        cfg
    }

    fn to_event_config(&self, seed: u64) -> EventSimConfig {
        let ev = &self.events;
        let mut agent_cfg = AgentSimConfig {
            seed,
            ..AgentSimConfig::default()
        };

        if let Some(ref a) = ev.agent {
            apply!(agent_cfg.initial_population, a.initial_population);
            apply!(agent_cfg.world_size, a.world_size);
            apply!(agent_cfg.ticks, a.ticks);
            apply!(agent_cfg.interaction_radius, a.interaction_radius);
            apply!(agent_cfg.max_age, a.max_age);
            apply!(agent_cfg.min_population, a.min_population);
            apply!(agent_cfg.max_population, a.max_population);
        }

        if let Some(ref en) = ev.energy {
            let e = &mut agent_cfg.energy;
            apply!(e.biomass_base_eroei, en.biomass_base_eroei);
            apply!(e.biomass_regen_rate, en.biomass_regen_rate);
            apply!(e.agriculture_base_eroei, en.agriculture_base_eroei);
            apply!(e.agriculture_tech_threshold, en.agriculture_tech_threshold);
            apply!(e.agriculture_fertility_prob, en.agriculture_fertility_prob);
            apply!(e.fossil_base_eroei, en.fossil_base_eroei);
            apply!(e.fossil_tech_threshold, en.fossil_tech_threshold);
            apply!(e.renewable_base_eroei, en.renewable_base_eroei);
            apply!(e.renewable_tech_threshold, en.renewable_tech_threshold);
        }

        if let Some(ref ix) = ev.interaction {
            let i = &mut agent_cfg.interaction;
            apply!(i.coop_self_weight, ix.coop_self_weight);
            apply!(i.coop_other_weight, ix.coop_other_weight);
            apply!(i.coop_kin_bonus, ix.coop_kin_bonus);
            apply!(i.conflict_self_weight, ix.conflict_self_weight);
            apply!(i.conflict_stranger_bonus, ix.conflict_stranger_bonus);
            apply!(i.subsistence_level, ix.subsistence_level);
            apply!(i.trust_coop_weight, ix.trust_coop_weight);
            apply!(i.trust_memory_decay, ix.trust_memory_decay);
        }

        if let Some(ref lc) = ev.lifecycle {
            let l = &mut agent_cfg.lifecycle;
            apply!(l.birth_rate, lc.birth_rate);
            apply!(l.death_health_threshold, lc.death_health_threshold);
            apply!(
                l.starvation_resource_threshold,
                lc.starvation_resource_threshold
            );
            apply!(l.starvation_death_prob, lc.starvation_death_prob);
            apply!(
                l.reproduction_resource_threshold,
                lc.reproduction_resource_threshold
            );
            apply!(l.innovation_growth_rate, lc.innovation_growth_rate);
        }

        if let Some(ref is) = ev.inter_society {
            let s = &mut agent_cfg.inter_society;
            apply!(s.min_raid_warriors, is.min_raid_warriors);
            apply!(s.raid_aggression_threshold, is.raid_aggression_threshold);
            apply!(s.raid_range, is.raid_range);
            apply!(s.conquest_power_ratio, is.conquest_power_ratio);
            apply!(s.tribute_rate, is.tribute_rate);
            apply!(
                s.migration_resource_threshold,
                is.migration_resource_threshold
            );
            apply!(s.migration_probability, is.migration_probability);
        }

        if let Some(ref cu) = ev.cultural {
            let c = &mut agent_cfg.cultural;
            apply!(c.vertical_mutation_prob, cu.vertical_mutation_prob);
            apply!(c.horizontal_adoption_prob, cu.horizontal_adoption_prob);
            apply!(c.oblique_adoption_prob, cu.oblique_adoption_prob);
            apply!(c.oblique_prestige_gap, cu.oblique_prestige_gap);
        }

        if let Some(ref inst) = ev.institution {
            let n = &mut agent_cfg.institution;
            apply!(n.public_goods_rate, inst.public_goods_rate);
            apply!(n.defense_bonus, inst.defense_bonus);
            apply!(n.leadership_threshold, inst.leadership_threshold);
        }

        let mut event_params = walrus_engine::event_sim::EventParams::default();
        apply!(event_params.measure_interval, ev.measure_interval);

        if let Some(ref r) = ev.rates {
            apply!(event_params.forage_base_rate, r.forage_base_rate);
            apply!(event_params.interact_base_rate, r.interact_base_rate);
            apply!(event_params.move_base_rate, r.move_base_rate);
            apply!(event_params.reproduce_base_rate, r.reproduce_base_rate);
            apply!(event_params.raid_base_rate, r.raid_base_rate);
            apply!(event_params.migrate_base_rate, r.migrate_base_rate);
        }

        EventSimConfig {
            agent: agent_cfg,
            event: event_params,
            end_time: ev.end_time.unwrap_or(500.0),
        }
    }
}

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

#[derive(Clone, serde::Serialize)]
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

#[derive(Clone, serde::Serialize)]
struct UnifiedFrame {
    generation: u32,
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

#[derive(Clone, serde::Serialize)]
enum TuiEvent {
    Disaster {
        continent: usize,
        severity: f64,
    },
    Pandemic {
        continent: usize,
        severity: f64,
    },
    Climate {
        continent: usize,
        severity: f64,
    },
    Collapse {
        continent: usize,
    },
    Migration {
        from: usize,
        to: usize,
    },
    ModeTransition {
        continent: usize,
        from: &'static str,
        to: &'static str,
    },
    Raid {
        count: u32,
    },
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
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
        ];

        for (ci, slot) in continents.iter_mut().enumerate() {
            let socs: Vec<_> = f.societies.iter().filter(|s| s.continent == ci).collect();
            let pop: u64 = socs.iter().map(|s| u64::from(s.population)).sum();
            let cx = if socs.is_empty() {
                0.0
            } else {
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
            let mode = if ag >= sed && ag >= hg {
                "Agr"
            } else if sed >= hg {
                "Sed"
            } else {
                "HG"
            };

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

        let events: Vec<TuiEvent> = f
            .events
            .iter()
            .map(|e| match *e {
                MapEvent::NaturalDisaster {
                    continent,
                    severity,
                } => TuiEvent::Disaster {
                    continent,
                    severity,
                },
                MapEvent::Pandemic {
                    continent,
                    severity,
                } => TuiEvent::Pandemic {
                    continent,
                    severity,
                },
                MapEvent::ClimateShock {
                    continent,
                    severity,
                } => TuiEvent::Climate {
                    continent,
                    severity,
                },
                MapEvent::Collapse { continent, .. } => TuiEvent::Collapse { continent },
                MapEvent::Migration { from, to } => TuiEvent::Migration { from, to },
                MapEvent::ModeTransition {
                    continent,
                    from,
                    to,
                    ..
                } => TuiEvent::ModeTransition {
                    continent,
                    from: mode_str(from),
                    to: mode_str(to),
                },
            })
            .collect();

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
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
            ContinentStats {
                population: 0,
                society_count: 0,
                mean_complexity: 0.0,
                depletion: 0.0,
                carrying_capacity: 1.0,
                dominant_mode: "HG",
                mean_resources: 0.0,
                cooperation_count: 0,
                conflict_count: 0,
            },
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
            events.push(TuiEvent::Raid {
                count: f.raid_events,
            });
        }
        // NOTE: event-driven sim lacks per-continent origin/destination data for
        // migrations, so we don't emit directional Migration events (which would
        // misrepresent where migrations actually occur). Migration count is still
        // visible in the summary/JSONL output via the frame's migration_events field.

        // Map institutional type to complexity proxy
        let inst_type = f.emergent.institutional_type;
        let complexity_proxy = match inst_type {
            0 => 0.1,  // band
            1 => 0.35, // tribe
            2 => 0.65, // chiefdom
            _ => 0.9,  // state
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

fn continent_cell_style(ci: usize, frame: &UnifiedFrame, flashes: &[ContinentFlash; 4]) -> Style {
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
            if dx > 0 {
                '\u{25B6}'
            } else {
                '\u{25C0}'
            }
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

const MAX_HISTORY: usize = 1000;

struct TuiState {
    frame: Option<UnifiedFrame>,
    flashes: [ContinentFlash; 4],
    active_migrations: Vec<(usize, usize, u8)>,
    event_log: VecDeque<EventLogEntry>,
    pop_history: Vec<u64>,
    complexity_history: Vec<u64>,
    so_history: Vec<u64>,
    paused: bool,
    speed_ms: u64,
    // Phase F: history + playback
    history: VecDeque<UnifiedFrame>,
    playback_cursor: Option<usize>, // None = live, Some(idx) = viewing history
    focused_continent: Option<usize>, // None = all, Some(0..4) = focused
}

const MAX_LOG_ENTRIES: usize = 50;

impl TuiState {
    fn new() -> Self {
        Self {
            frame: None,
            flashes: [ContinentFlash::default(); 4],
            active_migrations: Vec::new(),
            event_log: VecDeque::new(),
            pop_history: Vec::new(),
            complexity_history: Vec::new(),
            so_history: Vec::new(),
            paused: false,
            speed_ms: 80,
            history: VecDeque::new(),
            playback_cursor: None,
            focused_continent: None,
        }
    }

    fn update(&mut self, frame: UnifiedFrame) {
        let gen = frame.generation;

        for ev in &frame.events {
            match *ev {
                TuiEvent::Disaster {
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
                TuiEvent::Pandemic {
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
                TuiEvent::Climate {
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
                TuiEvent::Collapse { continent } => {
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
                TuiEvent::Migration { from, to } => {
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
                TuiEvent::ModeTransition {
                    continent,
                    from,
                    to,
                } => {
                    self.log(
                        gen,
                        format!(
                            "MODE {} {from}->{to}",
                            CONTINENT_NAMES.get(continent).unwrap_or(&"?")
                        ),
                        Color::White,
                    );
                }
                TuiEvent::Raid { count } => {
                    for fl in &mut self.flashes {
                        fl.raid = 5;
                    }
                    self.log(gen, format!("RAIDS: {count}"), Color::Red);
                }
            }
        }

        push_bounded(&mut self.pop_history, frame.total_population, 60);
        push_bounded(
            &mut self.complexity_history,
            (frame.mean_complexity * 1000.0) as u64,
            60,
        );
        push_bounded(
            &mut self.so_history,
            (frame.superorganism_index * 1000.0) as u64,
            60,
        );

        // Store in history ring buffer (O(1) eviction with VecDeque)
        self.history.push_back(frame.clone());
        if self.history.len() > MAX_HISTORY {
            self.history.pop_front();
            // Adjust playback cursor if in playback mode
            if let Some(ref mut cursor) = self.playback_cursor {
                *cursor = cursor.saturating_sub(1);
            }
        }

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
        self.event_log.push_back(EventLogEntry {
            generation,
            text,
            color,
        });
        if self.event_log.len() > MAX_LOG_ENTRIES {
            self.event_log.pop_front();
        }
    }

    /// Returns the frame that should be displayed (live or playback).
    fn display_frame(&self) -> Option<&UnifiedFrame> {
        if let Some(cursor) = self.playback_cursor {
            self.history.get(cursor)
        } else {
            self.frame.as_ref()
        }
    }

    fn scrub_back(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let cursor = match self.playback_cursor {
            Some(c) => c.saturating_sub(1),
            None => self.history.len().saturating_sub(2),
        };
        self.playback_cursor = Some(cursor);
    }

    fn scrub_forward(&mut self) {
        if let Some(cursor) = self.playback_cursor {
            let next = cursor + 1;
            if next >= self.history.len() {
                // Back to live
                self.playback_cursor = None;
            } else {
                self.playback_cursor = Some(next);
            }
        }
    }

    fn cycle_focus(&mut self) {
        self.focused_continent = match self.focused_continent {
            None => Some(0),
            Some(3) => None,
            Some(n) => Some(n + 1),
        };
    }

    fn save_current_frame(&self) -> Option<String> {
        let frame = self.display_frame()?;
        let json = serde_json::to_string_pretty(frame).ok()?;
        let filename = format!("frame_gen{}.json", frame.generation);
        std::fs::write(&filename, &json).ok()?;
        Some(filename)
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(f: &mut Frame, state: &TuiState) {
    let frame = match state.display_frame() {
        Some(fr) => fr,
        None => {
            f.render_widget(Paragraph::new("Waiting for simulation data..."), f.area());
            return;
        }
    };

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
    frame: &UnifiedFrame,
    flashes: &[ContinentFlash; 4],
    active_migrations: &[(usize, usize, u8)],
) {
    let block = Block::default().borders(Borders::ALL).title(format!(
        " World Map [{mode}] | t={gen} | pop {pop} | soc {soc} ",
        mode = frame.mode_label,
        gen = frame.generation,
        pop = frame.total_population,
        soc = frame.total_societies,
    ));
    let inner = block.inner(area);
    f.render_widget(block, area);

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
                    buf_cell.set_char('\u{2588}');
                    buf_cell.set_style(style);
                }
            }
        }
    }

    // Static corridor arrows
    for &(from, to, strength) in &frame.corridors {
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
                    buf_cell.set_style(Style::default().fg(Color::Rgb(
                        0,
                        intensity / 2,
                        intensity,
                    )));
                }
            }
        }
    }

    // Active migration arrows
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
                    buf_cell
                        .set_style(Style::default().fg(Color::Rgb(brightness, 255, 255)).bold());
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

    let focused = state.focused_continent;

    for (ci, name) in CONTINENT_NAMES.iter().enumerate() {
        let c = &frame.continents[ci];
        let name_color = match ci {
            0 => Color::Yellow,
            1 => Color::Green,
            2 => Color::Cyan,
            3 => Color::Magenta,
            _ => Color::White,
        };

        let is_focused = focused == Some(ci);
        let marker = if is_focused { "> " } else { "  " };

        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(name.to_string(), Style::default().fg(name_color).bold()),
            Span::raw(format!(" p:{} s:{}", c.population, c.society_count)),
        ]));

        if !c.dominant_mode.is_empty() {
            lines.push(Line::from(format!(
                "   cx:{:.2} dep:{:.2} {}",
                c.mean_complexity, c.depletion, c.dominant_mode
            )));
        } else {
            lines.push(Line::from(format!(
                "   res:{:.1} c:{}/f:{}",
                c.mean_resources, c.cooperation_count, c.conflict_count
            )));
        }

        // Show extra detail when focused
        if is_focused {
            lines.push(Line::from(format!(
                "   cap:{:.1} pop/cap:{:.2}",
                c.carrying_capacity,
                c.population as f64 / (c.carrying_capacity * 200.0).max(1.0),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Global",
        Style::default().fg(Color::White).bold(),
    )]));
    lines.push(Line::from(format!(
        " SO:{:.3} CX:{:.3}",
        frame.superorganism_index, frame.mean_complexity,
    )));
    if frame.convergence_index > 0.0 {
        lines.push(Line::from(format!(
            " conv:{:.3} civ:{}",
            frame.convergence_index, frame.emergent_civilizations,
        )));
    }

    let mut indicators: Vec<Span> = Vec::new();
    if frame.collapse_events > 0 {
        indicators.push(Span::styled(
            format!(" C:{}", frame.collapse_events),
            Style::default().fg(Color::Red).bold(),
        ));
    }
    if frame.disaster_events > 0 {
        indicators.push(Span::styled(
            format!(" D:{}", frame.disaster_events),
            Style::default().fg(Color::Yellow),
        ));
    }
    if frame.pandemic_events > 0 {
        indicators.push(Span::styled(
            format!(" P:{}", frame.pandemic_events),
            Style::default().fg(Color::Magenta),
        ));
    }
    if !indicators.is_empty() {
        lines.push(Line::from(indicators));
    }

    lines.push(Line::from(""));

    // Status line with playback info
    let status = if state.playback_cursor.is_some() {
        format!(
            "PLAYBACK {}/{}",
            state.playback_cursor.map_or(0, |c| c + 1),
            state.history.len()
        )
    } else if state.paused {
        "PAUSED".to_string()
    } else {
        "LIVE".to_string()
    };
    lines.push(Line::from(vec![Span::styled(
        format!("[{status}] {:.0}ms", state.speed_ms),
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "q:quit p:pause +/-:spd",
        Style::default().fg(Color::DarkGray),
    )]));
    lines.push(Line::from(vec![Span::styled(
        "Tab:focus </>:scrub s:save",
        Style::default().fg(Color::DarkGray),
    )]));

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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Superorganism "),
            )
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
    let lines: Vec<Line> = state
        .event_log
        .iter()
        .skip(start)
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
    f.render_widget(Paragraph::new(lines), inner);
}

// ---------------------------------------------------------------------------
// Simulation modes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum SimMode {
    Evolution,
    EventDriven,
}

enum OutputFormat {
    Tui,
    Jsonl,
    Summary,
}

struct CliArgs {
    sim_mode: SimMode,
    output_format: OutputFormat,
    compare: bool,
    toml: TomlConfig,
    // CLI overrides (applied on top of TOML)
    population: Option<u32>,
    generations: Option<u32>,
    seed: Option<u64>,
}

fn parse_u64(args: &[String], i: usize) -> Option<u64> {
    args.get(i + 1).and_then(|s| s.parse().ok())
}

fn parse_u32(args: &[String], i: usize) -> Option<u32> {
    args.get(i + 1).and_then(|s| s.parse().ok())
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut sim_mode: Option<SimMode> = None;
    let mut output_format: Option<OutputFormat> = None;
    let mut compare = false;
    let mut population: Option<u32> = None;
    let mut generations: Option<u32> = None;
    let mut seed: Option<u64> = None;
    let mut config_path = "walrus.toml".to_string();

    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--events" | "--event-driven" => sim_mode = Some(SimMode::EventDriven),
            "--evolution" => sim_mode = Some(SimMode::Evolution),
            "--compare" => compare = true,
            "--config" | "-c" => {
                if let Some(p) = args.get(i + 1) {
                    config_path = p.clone();
                }
            }
            "--population" | "--pop" | "-n" => population = parse_u32(&args, i),
            "--generations" | "--gen" | "-g" => generations = parse_u32(&args, i),
            "--seed" => seed = parse_u64(&args, i),
            "--format" => {
                if let Some(fmt) = args.get(i + 1) {
                    output_format = Some(match fmt.as_str() {
                        "jsonl" | "json" => OutputFormat::Jsonl,
                        "summary" | "text" => OutputFormat::Summary,
                        _ => OutputFormat::Tui,
                    });
                }
            }
            _ => {}
        }
    }

    if std::env::var("EVENT_DRIVEN").unwrap_or_default() == "true" {
        sim_mode = Some(SimMode::EventDriven);
    }

    // Load TOML config (silent if file doesn't exist)
    let toml = TomlConfig::load(&config_path);

    // TOML provides defaults, CLI overrides
    let resolved_mode = sim_mode.unwrap_or(match toml.simulation.mode.as_deref() {
        Some("events" | "event-driven") => SimMode::EventDriven,
        _ => SimMode::Evolution,
    });

    let resolved_format = output_format.unwrap_or(match toml.simulation.format.as_deref() {
        Some("jsonl" | "json") => OutputFormat::Jsonl,
        Some("summary" | "text") => OutputFormat::Summary,
        _ => OutputFormat::Tui,
    });

    CliArgs {
        sim_mode: resolved_mode,
        output_format: resolved_format,
        compare,
        toml,
        population,
        generations,
        seed,
    }
}

fn run_evolution_with_config(tx: mpsc::Sender<UnifiedFrame>, config: EvolutionConfig) {
    let _ = simulate_evolution_with_observer(config, |frame| {
        let unified: UnifiedFrame = frame.clone().into();
        let _ = tx.send(unified);
    });
}

fn make_evolution_config(args: &CliArgs) -> EvolutionConfig {
    let seed = args
        .seed
        .unwrap_or(args.toml.simulation.seed.unwrap_or(2026));
    let mut cfg = args.toml.to_evolution_config(seed);
    // CLI overrides
    if let Some(pop) = args.population {
        cfg.initial_societies = pop;
    }
    if let Some(gen) = args.generations {
        cfg.generations = gen;
    }
    cfg
}

fn make_event_config(args: &CliArgs) -> EventSimConfig {
    let seed = args
        .seed
        .unwrap_or(args.toml.simulation.seed.unwrap_or(2026));
    let mut cfg = args.toml.to_event_config(seed);
    // CLI overrides
    if let Some(pop) = args.population {
        cfg.agent.initial_population = pop;
        // Auto-scale world_size for large populations
        if pop > 10_000 {
            cfg.agent.world_size = (pop as f32).sqrt() * 2.0;
        }
    }
    if let Some(gen) = args.generations {
        cfg.end_time = f64::from(gen);
    }
    cfg
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn spawn_sim(args: &CliArgs, tx: mpsc::SyncSender<UnifiedFrame>) -> thread::JoinHandle<()> {
    let evo_config = make_evolution_config(args);
    let event_config = make_event_config(args);
    let mode = args.sim_mode;
    thread::spawn(move || match mode {
        SimMode::Evolution => {
            let _ = simulate_evolution_with_observer(evo_config, |frame| {
                let _ = tx.send(frame.clone().into());
            });
        }
        SimMode::EventDriven => {
            let _ = simulate_event_driven_with_observer(event_config, |frame| {
                let _ = tx.send(frame.clone().into());
            });
        }
    })
}

fn run_jsonl(args: &CliArgs) {
    use std::io::Write;
    let (tx, rx) = mpsc::sync_channel::<UnifiedFrame>(8);
    let stdout = io::stdout();

    let _sim_handle = spawn_sim(args, tx);

    let mut out = io::BufWriter::new(stdout.lock());
    while let Ok(frame) = rx.recv() {
        if let Ok(json) = serde_json::to_string(&frame) {
            let _ = writeln!(out, "{json}");
            let _ = out.flush();
        }
    }
}

fn run_summary(args: &CliArgs) {
    use std::io::Write;
    let (tx, rx) = mpsc::sync_channel::<UnifiedFrame>(8);
    let stdout = io::stdout();

    let _sim_handle = spawn_sim(args, tx);

    let mut out = io::BufWriter::new(stdout.lock());
    while let Ok(frame) = rx.recv() {
        let continent_summary: String = frame
            .continents
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                format!(
                    "{}:p={}/cx={:.2}",
                    CONTINENT_NAMES[ci], c.population, c.mean_complexity,
                )
            })
            .collect::<Vec<_>>()
            .join(" ");

        let event_counts = if !frame.events.is_empty() {
            format!(" events={}", frame.events.len())
        } else {
            String::new()
        };

        let _ = writeln!(
            out,
            "t={:<5} pop={:<8} soc={:<3} SO={:.3} CX={:.3} {} {}{}",
            frame.generation,
            frame.total_population,
            frame.total_societies,
            frame.superorganism_index,
            frame.mean_complexity,
            frame.mode_label,
            continent_summary,
            event_counts,
        );
        let _ = out.flush();
    }
}

fn run_tui(args: &CliArgs) -> io::Result<()> {
    let (tx, rx) = mpsc::sync_channel::<UnifiedFrame>(8);

    let _sim_handle = spawn_sim(args, tx);

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = TuiState::new();

    loop {
        // Only consume new frames when not paused
        if !state.paused {
            while let Ok(frame) = rx.try_recv() {
                state.update(frame);
            }
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
                        KeyCode::Left => state.scrub_back(),
                        KeyCode::Right => state.scrub_forward(),
                        KeyCode::Tab => state.cycle_focus(),
                        KeyCode::Char('s') => {
                            if let Some(filename) = state.save_current_frame() {
                                state.log(
                                    state.display_frame().map_or(0, |f| f.generation),
                                    format!("Saved: {filename}"),
                                    Color::White,
                                );
                            }
                        }
                        KeyCode::Home => {
                            // Jump to live
                            state.playback_cursor = None;
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

// ---------------------------------------------------------------------------
// Scenario comparison mode
// ---------------------------------------------------------------------------

struct CompareState {
    a: TuiState,
    b: TuiState,
    label_a: String,
    label_b: String,
}

impl CompareState {
    fn new(label_a: String, label_b: String) -> Self {
        Self {
            a: TuiState::new(),
            b: TuiState::new(),
            label_a,
            label_b,
        }
    }
}

fn render_compare(f: &mut Frame, state: &CompareState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(MAP_HEIGHT as u16 + 2),
            Constraint::Length(8),
        ])
        .split(f.area());

    // Two maps side by side
    let maps = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[0]);

    render_compare_map(f, maps[0], &state.a, &state.label_a);
    render_compare_map(f, maps[1], &state.b, &state.label_b);

    // Bottom: delta panel + shared sparklines
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    render_compare_sparklines(f, bottom[0], &state.a, &state.b);
    render_compare_delta(f, bottom[1], &state.a, &state.b);
}

fn render_compare_map(f: &mut Frame, area: Rect, state: &TuiState, label: &str) {
    let frame = match &state.frame {
        Some(fr) => fr,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" {label}: waiting... "));
            f.render_widget(block, area);
            return;
        }
    };

    let block = Block::default().borders(Borders::ALL).title(format!(
        " {label} | t={gen} | pop {pop} ",
        gen = frame.generation,
        pop = frame.total_population,
    ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Scale the map to fit the half-width panel
    let scale_x = inner.width as f64 / MAP_WIDTH as f64;
    let scale_y = inner.height as f64 / MAP_HEIGHT as f64;

    for (row_idx, row_str) in MAP_ROWS.iter().enumerate() {
        let y = inner.y + (row_idx as f64 * scale_y) as u16;
        if y >= inner.y + inner.height {
            break;
        }
        for (col_idx, ch) in row_str.chars().enumerate() {
            let x = inner.x + (col_idx as f64 * scale_x) as u16;
            if x >= inner.x + inner.width {
                break;
            }
            if let Some(ci) = continent_index(ch) {
                let style = continent_cell_style(ci, frame, &state.flashes);
                if let Some(buf_cell) = f.buffer_mut().cell_mut(Position::new(x, y)) {
                    buf_cell.set_char('\u{2588}');
                    buf_cell.set_style(style);
                }
            }
        }
    }
}

fn render_compare_sparklines(f: &mut Frame, area: Rect, a: &TuiState, b: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(area);

    // Population A
    f.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Pop A (green) "),
            )
            .data(&a.pop_history)
            .style(Style::default().fg(Color::Green)),
        chunks[0],
    );

    // Population B
    f.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Pop B (cyan) "),
            )
            .data(&b.pop_history)
            .style(Style::default().fg(Color::Cyan)),
        chunks[1],
    );

    // Complexity A
    f.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" CX A (blue) "),
            )
            .data(&a.complexity_history)
            .style(Style::default().fg(Color::Blue)),
        chunks[2],
    );

    // Complexity B
    f.render_widget(
        Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" CX B (yellow) "),
            )
            .data(&b.complexity_history)
            .style(Style::default().fg(Color::Yellow)),
        chunks[3],
    );
}

fn render_compare_delta(f: &mut Frame, area: Rect, a: &TuiState, b: &TuiState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delta (A - B) ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (fa, fb) = match (&a.frame, &b.frame) {
        (Some(fa), Some(fb)) => (fa, fb),
        _ => {
            f.render_widget(Paragraph::new("Waiting for both sims..."), inner);
            return;
        }
    };

    let dpop = fa.total_population as i64 - fb.total_population as i64;
    let dsoc = fa.total_societies as i64 - fb.total_societies as i64;
    let dcx = fa.mean_complexity - fb.mean_complexity;
    let dso = fa.superorganism_index - fb.superorganism_index;

    let delta_color = |v: f64| -> Color {
        if v > 0.01 {
            Color::Green
        } else if v < -0.01 {
            Color::Red
        } else {
            Color::DarkGray
        }
    };

    let lines = vec![
        Line::from(vec![
            Span::raw("Pop: "),
            Span::styled(
                format!("{:+}", dpop),
                Style::default().fg(if dpop > 0 {
                    Color::Green
                } else if dpop < 0 {
                    Color::Red
                } else {
                    Color::DarkGray
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw("Soc: "),
            Span::styled(
                format!("{:+}", dsoc),
                Style::default().fg(if dsoc > 0 {
                    Color::Green
                } else if dsoc < 0 {
                    Color::Red
                } else {
                    Color::DarkGray
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw("CX:  "),
            Span::styled(
                format!("{:+.3}", dcx),
                Style::default().fg(delta_color(dcx)),
            ),
        ]),
        Line::from(vec![
            Span::raw("SO:  "),
            Span::styled(
                format!("{:+.3}", dso),
                Style::default().fg(delta_color(dso)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "q:quit",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn run_compare() -> io::Result<()> {
    // Scenario A: baseline (default parameters)
    let (tx_a, rx_a) = mpsc::channel::<UnifiedFrame>();
    let config_a = EvolutionConfig {
        generations: 600,
        initial_societies: 24,
        ..EvolutionConfig::default()
    };
    let _sim_a = thread::spawn(move || run_evolution_with_config(tx_a, config_a));

    // Scenario B: high isolation, higher disaster rate
    let (tx_b, rx_b) = mpsc::channel::<UnifiedFrame>();
    let config_b = EvolutionConfig {
        generations: 600,
        initial_societies: 24,
        seed: 9999,
        isolation_factor: 0.85,
        natural_disaster_base_rate: 0.15,
        resource_multiplier: 0.7,
        ..EvolutionConfig::default()
    };
    let _sim_b = thread::spawn(move || run_evolution_with_config(tx_b, config_b));

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut state = CompareState::new("Baseline".to_string(), "Isolated+Harsh".to_string());

    let speed_ms = 80u64;

    loop {
        while let Ok(frame) = rx_a.try_recv() {
            state.a.update(frame);
        }
        while let Ok(frame) = rx_b.try_recv() {
            state.b.update(frame);
        }

        terminal.draw(|f| render_compare(f, &state))?;
        state.a.decay();
        state.b.decay();

        if event::poll(Duration::from_millis(speed_ms))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char('q') = key.code {
                        break;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args = parse_args();
    if args.compare {
        return run_compare();
    }
    match args.output_format {
        OutputFormat::Jsonl => {
            run_jsonl(&args);
            Ok(())
        }
        OutputFormat::Summary => {
            run_summary(&args);
            Ok(())
        }
        OutputFormat::Tui => run_tui(&args),
    }
}
