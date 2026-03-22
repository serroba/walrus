//! Event-driven simulation engine.
//!
//! Replaces the tick-based `simulate_agents` with a discrete-event loop where
//! each agent acts on its own stochastic timeline.  Effects are applied
//! immediately to the current world state, producing emergent non-deterministic
//! ordering across runs.

use crate::agents::*;
use crate::event_queue::*;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Parameters controlling event scheduling rates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventParams {
    /// Base rate for foraging events (per sim-time unit per agent).
    pub forage_base_rate: f64,
    /// Base rate for interaction events.
    pub interact_base_rate: f64,
    /// Base rate for movement events.
    pub move_base_rate: f64,
    /// Base rate for cultural transmission events.
    pub transmit_base_rate: f64,
    /// Base rate for reproduction attempts.
    pub reproduce_base_rate: f64,
    /// Base rate for aging events.
    pub age_base_rate: f64,
    /// Base rate for learning / innovation events.
    pub learn_base_rate: f64,
    /// Base rate for raid attempts (per kin group).
    pub raid_base_rate: f64,
    /// Base rate for migration evaluation (per kin group).
    pub migrate_base_rate: f64,
    /// Interval between tribute collections (tribute now runs on the landscape
    /// update interval via `WorldAction::UpdateLandscape`; kept for config compat).
    pub tribute_interval: f64,
    /// Interval between spatial index rebuilds.
    pub spatial_rebuild_interval: f64,
    /// Interval between state measurement snapshots.
    pub measure_interval: f64,
    /// Interval between landscape updates (biomass regen).
    pub landscape_update_interval: f64,
}

impl Default for EventParams {
    fn default() -> Self {
        Self {
            forage_base_rate: 1.0,
            interact_base_rate: 1.5,
            move_base_rate: 1.0,
            transmit_base_rate: 0.3,
            reproduce_base_rate: 0.5,
            age_base_rate: 1.0,
            learn_base_rate: 1.0,
            raid_base_rate: 0.2,
            migrate_base_rate: 0.3,
            tribute_interval: 1.0,
            spatial_rebuild_interval: 1.0,
            measure_interval: 1.0,
            landscape_update_interval: 1.0,
        }
    }
}

/// Full configuration for the event-driven simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct EventSimConfig {
    /// Agent simulation parameters (reuses existing config).
    pub agent: AgentSimConfig,
    /// Event scheduling parameters.
    pub event: EventParams,
    /// Simulation end time (in sim-time units).
    pub end_time: f64,
}

impl Default for EventSimConfig {
    fn default() -> Self {
        Self {
            agent: AgentSimConfig::default(),
            event: EventParams::default(),
            end_time: 500.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Simulation world state
// ---------------------------------------------------------------------------

/// Mutable world state for the event-driven simulation.
pub struct SimWorld {
    pub pop: Population,
    pub landscape: EnergyLandscape,
    pub(crate) grid: SpatialGrid,
    pub tributes: Vec<TributeRelation>,
    pub next_id: u64,
    pub rng: u64,
    pub now: SimTime,
    /// Accumulated event counters for measurement windows.
    pub counters: EventCounters,
    /// Set of alive agent IDs for O(1) death checks.
    alive: std::collections::HashSet<u64>,
    /// O(1) agent-id to population-index lookup.
    id_to_index: std::collections::HashMap<u64, usize>,
}

/// Accumulated event counts between measurement snapshots.
#[derive(Clone, Debug, Default)]
pub struct EventCounters {
    pub cooperation_events: u32,
    pub conflict_events: u32,
    pub trade_events: u32,
    pub total_interactions: u32,
    pub voluntary_transfers: u32,
    pub involuntary_transfers: u32,
    pub intra_kin_conflicts: u32,
    pub intra_kin_interactions: u32,
    pub inter_group_trades: u32,
    pub raid_events: u32,
    pub conquest_events: u32,
    pub tribute_total: f32,
    pub migration_events: u32,
    pub total_actual_surplus: f32,
    pub total_cooperative_optimal: f32,
}

impl EventCounters {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Per-snapshot record with continuous time.
#[derive(Clone, Debug, PartialEq)]
pub struct EventSnapshot {
    pub time: f64,
    pub emergent: EmergentState,
}

/// Result of an event-driven simulation run.
#[derive(Clone, Debug)]
pub struct EventSimResult {
    pub snapshots: Vec<EventSnapshot>,
    pub final_population: Population,
    pub final_landscape: EnergyLandscape,
    pub events_processed: u64,
}

// ---------------------------------------------------------------------------
// Agent index lookup
// ---------------------------------------------------------------------------

/// Find the array index of an agent by its stable ID.
/// Returns `None` if the agent is dead (not in population).
fn find_agent(world: &SimWorld, id: u64) -> Option<usize> {
    world.id_to_index.get(&id).copied()
}

// ---------------------------------------------------------------------------
// Event handlers
// ---------------------------------------------------------------------------

fn handle_forage(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let ep = &cfg.agent.energy;
    let cell_size = world.landscape.cell_size;
    let cols = world.landscape.cols;
    let rows = world.landscape.rows;

    let cx = (world.pop.xs[i] / cell_size).floor() as usize;
    let cy = (world.pop.ys[i] / cell_size).floor() as usize;
    let key = cy.min(rows - 1) * cols + cx.min(cols - 1);

    let tech = world.pop.innovations[i];
    let cell = &mut world.landscape.cells[key];

    for source in cell.sources.iter_mut() {
        if tech < source.tech_threshold || source.flow_rate <= 0.0 {
            continue;
        }
        let eroei = source.current_eroei();
        if eroei <= 1.0 {
            continue;
        }
        let gross = (source.flow_rate * ep.harvest_per_agent).min(if source.stock.is_finite() {
            source.stock
        } else {
            f64::MAX
        });
        let net = gross * (1.0 - 1.0 / eroei);
        world.pop.resources[i] += net as f32;
        if source.stock.is_finite() {
            source.stock = (source.stock - gross).max(0.0);
        }
    }
}

fn handle_interact(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let ip = &cfg.agent.interaction;
    let cp = &cfg.agent.cultural;
    let lp = &cfg.agent.lifecycle;
    let inst = &cfg.agent.institution;
    let rng = &mut world.rng;

    // Find a neighbor to interact with
    let neighbors = world.grid.neighbors_of(world.pop.xs[i], world.pop.ys[i]);
    if neighbors.is_empty() {
        return;
    }

    // Pick a random neighbor
    let j_idx = (rand_f64(rng) * neighbors.len() as f64) as usize % neighbors.len();
    let j = neighbors[j_idx] as usize;
    if j == i || j >= world.pop.len() {
        return;
    }

    // Distance check
    let dx = world.pop.xs[i] - world.pop.xs[j];
    let dy = world.pop.ys[i] - world.pop.ys[j];
    let dist_sq = dx * dx + dy * dy;
    let max_dist_sq = world.grid.cell_size * world.grid.cell_size * 4.0;
    if dist_sq > max_dist_sq {
        return;
    }

    let same_kin = world.pop.kin_groups[i] == world.pop.kin_groups[j];
    world.counters.total_interactions += 1;
    if same_kin {
        world.counters.intra_kin_interactions += 1;
    }

    let my_coop = world.pop.cooperations[i];
    let my_aggr = world.pop.aggressions[i];
    let my_skill = world.pop.skill_types[i];
    let my_status = world.pop.statuses[i];
    let my_culture = world.pop.cultures[i];
    let other_coop = world.pop.cooperations[j];
    let other_aggr = world.pop.aggressions[j];

    // Interaction decision
    let sharing_boost = my_culture.sharing_norm * cp.sharing_coop_bonus;
    let trust_boost = if !same_kin {
        my_culture.trust_outgroup * cp.trust_trade_bonus
    } else {
        0.0
    };
    let coercion_boost = my_culture.coercion_tolerance * cp.coercion_conflict_bonus;

    let coop_tendency = my_coop * ip.coop_self_weight
        + other_coop * ip.coop_other_weight
        + if same_kin { ip.coop_kin_bonus } else { 0.0 }
        + sharing_boost
        + world.pop.trust_memory[i] * ip.trust_coop_weight;
    let conflict_tendency = my_aggr * ip.conflict_self_weight
        + other_aggr * ip.conflict_other_weight
        + if !same_kin {
            ip.conflict_stranger_bonus
        } else {
            0.0
        }
        + coercion_boost
        + (1.0 - world.pop.trust_memory[i]) * ip.trust_coop_weight * 0.5;
    let trade_tendency = if my_skill != world.pop.skill_types[j] {
        ip.trade_complementary
    } else {
        ip.trade_same_skill
    } + trust_boost;

    let total = coop_tendency + conflict_tendency + trade_tendency;
    let roll = rand_f64(rng) as f32 * total;

    // Track cooperative counterfactual for every interaction (both agents benefit)
    world.counters.total_cooperative_optimal += ip.coop_resource_bonus * 2.0;

    let was_cooperation;

    if roll < coop_tendency {
        // Cooperation
        let coop_bonus = ip.coop_resource_bonus * (world.pop.cooperations[j] + my_coop) * 0.5;
        world.pop.resources[i] = (world.pop.resources[i] + coop_bonus).max(0.0);
        world.pop.resources[j] = (world.pop.resources[j] + coop_bonus).max(0.0);
        world.pop.prestiges[i] =
            (world.pop.prestiges[i] + ip.coop_prestige_gain).min(ip.max_prestige);
        world.pop.prestiges[j] =
            (world.pop.prestiges[j] + ip.coop_prestige_gain).min(ip.max_prestige);
        world.counters.cooperation_events += 1;
        world.counters.voluntary_transfers += 1;
        world.counters.total_actual_surplus += coop_bonus * 2.0; // both agents benefit
        was_cooperation = true;
    } else if roll < coop_tendency + conflict_tendency {
        // Conflict
        let my_power = my_status * ip.power_status_weight
            + world.pop.skill_levels[i] * ip.power_skill_weight
            + my_aggr * ip.power_aggression_weight;
        let other_power = world.pop.statuses[j] * ip.power_status_weight
            + world.pop.skill_levels[j] * ip.power_skill_weight
            + other_aggr * ip.power_aggression_weight;
        if my_power > other_power + rand_f64(rng) as f32 * ip.conflict_noise {
            // I win
            world.pop.resources[i] += ip.conflict_win_resources;
            world.pop.statuses[i] =
                (world.pop.statuses[i] + ip.conflict_win_status).min(ip.max_status);
            world.pop.resources[j] = (world.pop.resources[j] - ip.conflict_lose_resources).max(0.0);
            world.pop.healths[j] = (world.pop.healths[j] - ip.conflict_lose_health).max(0.0);
            world.counters.total_actual_surplus +=
                ip.conflict_win_resources - ip.conflict_lose_resources;
        } else {
            // They win
            world.pop.resources[j] += ip.conflict_win_resources;
            world.pop.statuses[j] =
                (world.pop.statuses[j] + ip.conflict_win_status).min(ip.max_status);
            world.pop.resources[i] = (world.pop.resources[i] - ip.conflict_lose_resources).max(0.0);
            world.pop.healths[i] = (world.pop.healths[i] - ip.conflict_lose_health).max(0.0);
            world.counters.total_actual_surplus +=
                ip.conflict_win_resources - ip.conflict_lose_resources;
        }
        world.counters.conflict_events += 1;
        world.counters.involuntary_transfers += 1;
        if same_kin {
            world.counters.intra_kin_conflicts += 1;
        }
        was_cooperation = false;
    } else {
        // Trade
        let skill_bonus = if my_skill != world.pop.skill_types[j] {
            ip.trade_complementary_bonus * (world.pop.skill_levels[i] + world.pop.skill_levels[j])
        } else {
            ip.trade_same_bonus
        };
        world.pop.resources[i] += skill_bonus;
        world.pop.resources[j] += skill_bonus;
        world.counters.trade_events += 1;
        world.counters.voluntary_transfers += 1;
        world.counters.total_actual_surplus += skill_bonus * 2.0; // both agents benefit
        if !same_kin {
            world.counters.inter_group_trades += 1;
        }
        was_cooperation = false;
    }

    // Update trust_memory for both agents based on whether this interaction was cooperation
    {
        let alpha = ip.trust_memory_decay.clamp(0.0, 1.0);
        let coop_signal = if was_cooperation { 1.0_f32 } else { 0.0 };
        world.pop.trust_memory[i] =
            ((1.0 - alpha) * world.pop.trust_memory[i] + alpha * coop_signal).clamp(0.0, 1.0);
        world.pop.trust_memory[j] =
            ((1.0 - alpha) * world.pop.trust_memory[j] + alpha * coop_signal).clamp(0.0, 1.0);
    }

    // Delegation: consider this neighbor as patron
    let effective_gap =
        ip.delegation_status_gap - my_culture.authority_norm * cp.authority_delegation_bonus;
    if world.pop.statuses[j] > my_status + effective_gap
        && world.pop.skill_types[j] == SkillType::Leader
    {
        let old_patron = world.pop.patrons[i];
        if old_patron != Some(j as u32) {
            world.pop.patron_ticks[i] = 0;
        }
        world.pop.patrons[i] = Some(j as u32);
        let tax = world.pop.resources[i] * ip.delegation_tax_rate;
        world.pop.resources[i] -= tax;
        let public_share = tax * inst.public_goods_rate;
        world.pop.resources[j] += tax - public_share;
        world.pop.prestiges[j] =
            (world.pop.prestiges[j] + ip.delegation_prestige_gain).min(ip.max_prestige);
    }

    // Increment patron tenure
    if world.pop.patrons[i].is_some() {
        world.pop.patron_ticks[i] += 1;
    }

    // Skill improvement through practice
    world.pop.skill_levels[i] = (world.pop.skill_levels[i] + ip.skill_practice_rate).min(1.0);

    // Health recovery for well-fed agents
    if world.pop.resources[i] > lp.health_recovery_threshold {
        let recovery = lp.health_recovery_rate
            * (world.pop.resources[i] - lp.health_recovery_threshold).min(1.0);
        world.pop.healths[i] = (world.pop.healths[i] + recovery).min(1.0);
    }

    // Surplus
    world.pop.surpluses[i] = (world.pop.resources[i] - ip.subsistence_level).max(0.0);
}

fn handle_move(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let mp = &cfg.agent.movement;
    let rng = &mut world.rng;
    let n = world.pop.len();
    if n == 0 {
        return;
    }

    // Compute kin centroid on the fly (just for this agent's group)
    let my_kin = world.pop.kin_groups[i];
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    let mut count = 0_u32;
    for k in 0..n {
        if world.pop.kin_groups[k] == my_kin {
            cx += f64::from(world.pop.xs[k]);
            cy += f64::from(world.pop.ys[k]);
            count += 1;
        }
    }

    let world_max = cfg.agent.world_size - 0.1;

    if count > 1 {
        let centroid_x = (cx / f64::from(count)) as f32;
        let centroid_y = (cy / f64::from(count)) as f32;
        let dx = (centroid_x - world.pop.xs[i]) * mp.kin_pull_strength;
        let dy = (centroid_y - world.pop.ys[i]) * mp.kin_pull_strength;
        world.pop.xs[i] += dx + (rand_f64(rng) as f32 - 0.5) * mp.drift_with_kin;
        world.pop.ys[i] += dy + (rand_f64(rng) as f32 - 0.5) * mp.drift_with_kin;
    } else {
        world.pop.xs[i] += (rand_f64(rng) as f32 - 0.5) * mp.drift_alone;
        world.pop.ys[i] += (rand_f64(rng) as f32 - 0.5) * mp.drift_alone;
    }

    world.pop.xs[i] = world.pop.xs[i].clamp(0.0, world_max);
    world.pop.ys[i] = world.pop.ys[i].clamp(0.0, world_max);
}

fn handle_reproduce(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let lp = &cfg.agent.lifecycle;
    let rng = &mut world.rng;

    // Only females reproduce
    if world.pop.sexes[i] != Sex::Female {
        return;
    }
    if world.pop.fertilities[i] < lp.min_fertility
        || world.pop.ages[i] < lp.min_reproduction_age
        || world.pop.ages[i] > lp.max_reproduction_age
    {
        return;
    }
    if world.pop.resources[i] < lp.reproduction_resource_threshold {
        return;
    }
    if (world.pop.len() as u32) >= cfg.agent.max_population {
        return;
    }

    // Find a mate
    let mate = world.pop.partners[i]
        .and_then(|p| {
            if (p as usize) < world.pop.len() && world.pop.sexes[p as usize] == Sex::Male {
                Some(p as usize)
            } else {
                None
            }
        })
        .or_else(|| find_mate_event(&world.pop, i, rng, &cfg.agent));

    let Some(m) = mate else {
        return;
    };

    let birth_prob = lp.birth_rate * world.pop.fertilities[i] * world.pop.healths[i];
    if rand_f64(rng) as f32 >= birth_prob {
        return;
    }

    // Pair them
    world.pop.partners[i] = Some(m as u32);
    world.pop.partners[m] = Some(i as u32);

    let child_sex = if rand_f64(rng) < 0.5 {
        Sex::Male
    } else {
        Sex::Female
    };

    // Inherit skill
    let skill = if rand_f64(rng) < lp.skill_maternal_inherit_prob {
        world.pop.skill_types[i]
    } else if rand_f64(rng) < lp.skill_mutation_prob {
        world.pop.skill_types[m]
    } else {
        match (rand_f64(rng) * 5.0) as u32 {
            0 => SkillType::Forager,
            1 => SkillType::Crafter,
            2 => SkillType::Builder,
            3 => SkillType::Leader,
            _ => SkillType::Warrior,
        }
    };

    // Vertical cultural transmission
    let cp = &cfg.agent.cultural;
    let mother_culture = &world.pop.cultures[i];
    let father_culture = &world.pop.cultures[m];

    let child_culture = create_child_culture(mother_culture, father_culture, lp, cp, rng);

    // Patron inheritance
    let inherited_patron = if cfg.agent.institution.patron_inheritance {
        world.pop.patrons[i].and_then(|p| {
            if (p as usize) < world.pop.len() {
                Some(p)
            } else {
                None
            }
        })
    } else {
        None
    };

    let child_id = world.next_id;
    world.next_id += 1;

    let child = AgentInit {
        id: child_id,
        sex: child_sex,
        age: 0,
        fertility: 0.0,
        health: lp.newborn_health,
        skill_type: skill,
        skill_level: lp.newborn_skill_level,
        status: lp.newborn_status,
        prestige: 0.0,
        aggression: ((world.pop.aggressions[i] + world.pop.aggressions[m]) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * lp.trait_mutation_magnitude)
            .clamp(0.0, 1.0),
        cooperation: ((world.pop.cooperations[i] + world.pop.cooperations[m]) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * lp.trait_mutation_magnitude)
            .clamp(0.0, 1.0),
        resources: lp.newborn_resources,
        surplus: 0.0,
        culture: child_culture,
        innovation: ((world.pop.innovations[i] + world.pop.innovations[m]) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * lp.trait_mutation_magnitude * 0.5)
            .clamp(0.0, 1.0),
        kin_group: world.pop.kin_groups[i],
        x: world.pop.xs[i] + (rand_f64(rng) as f32 - 0.5) * lp.birth_spawn_radius,
        y: world.pop.ys[i] + (rand_f64(rng) as f32 - 0.5) * lp.birth_spawn_radius,
    };

    let child_index = world.pop.len();
    world.pop.push_agent(child);
    world.alive.insert(child_id);
    world.id_to_index.insert(child_id, child_index);

    // Set inherited patron
    if let Some(p) = inherited_patron {
        let idx = world.pop.len() - 1;
        if (p as usize) < world.pop.len() {
            world.pop.patrons[idx] = Some(p);
        }
    }

    // Reproduction cost
    world.pop.resources[i] -= lp.birth_resource_cost;
    world.pop.healths[i] -= lp.birth_health_cost;
}

fn handle_age(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let lp = &cfg.agent.lifecycle;
    let rng = &mut world.rng;

    // Age
    world.pop.ages[i] = world.pop.ages[i].saturating_add(1);

    // Health decay with age
    let age_ratio = (world.pop.ages[i] as f32 / cfg.agent.max_age as f32).clamp(0.0, 1.0);
    let age_factor = age_ratio * age_ratio;
    world.pop.healths[i] -= lp.health_decay_base + lp.health_decay_age_factor * age_factor;
    world.pop.healths[i] = world.pop.healths[i].clamp(0.0, 1.0);

    // Fertility update
    let age_f = world.pop.ages[i] as f32;
    world.pop.fertilities[i] = match world.pop.sexes[i] {
        Sex::Female => (lp.female_peak_fertility
            - (age_f - lp.female_fertility_peak_age).abs() * lp.female_fertility_decline)
            .clamp(0.0, 1.0),
        Sex::Male => (lp.male_peak_fertility
            - (age_f - lp.male_fertility_peak_age).abs() * lp.male_fertility_decline)
            .clamp(0.0, 1.0),
    };

    // Death check
    let die = world.pop.ages[i] >= cfg.agent.max_age
        || world.pop.healths[i] < lp.death_health_threshold
        || (world.pop.resources[i] < lp.starvation_resource_threshold
            && (rand_f64(rng) as f32) < lp.starvation_death_prob);

    if die {
        world.alive.remove(&agent_id);
        // swap_remove and fix references
        let n = world.pop.len();
        if i < n {
            world.pop.swap_remove(i);
            let new_n = world.pop.len();
            if i != n - 1 {
                // The last element was swapped into position i.
                // Fix references that pointed to the old last index (n-1).
                // Also update the id→index map for the swapped agent.
                if let Some(swapped_id) = world.pop.ids.get(i).copied() {
                    world.id_to_index.insert(swapped_id, i);
                }
                for k in 0..new_n {
                    if let Some(p) = world.pop.patrons[k] {
                        if p as usize == n - 1 {
                            world.pop.patrons[k] = Some(i as u32);
                        } else if p as usize >= new_n {
                            world.pop.patrons[k] = None;
                        }
                    }
                    if let Some(p) = world.pop.partners[k] {
                        if p as usize == n - 1 {
                            world.pop.partners[k] = Some(i as u32);
                        } else if p as usize >= new_n {
                            world.pop.partners[k] = None;
                        }
                    }
                }
            } else {
                // Dying agent was last element; swap_remove just popped it.
                // Clear any references that pointed to the removed index.
                for k in 0..new_n {
                    if let Some(p) = world.pop.patrons[k] {
                        if p as usize >= new_n {
                            world.pop.patrons[k] = None;
                        }
                    }
                    if let Some(p) = world.pop.partners[k] {
                        if p as usize >= new_n {
                            world.pop.partners[k] = None;
                        }
                    }
                }
            }
            // Remove the dead agent from the id→index map.
            world.id_to_index.remove(&agent_id);
        }
    }
}

fn handle_learn(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let lp = &cfg.agent.lifecycle;
    world.pop.innovations[i] = (world.pop.innovations[i] + lp.innovation_growth_rate).min(1.0);
}

fn handle_transmit(world: &mut SimWorld, agent_id: u64, cfg: &EventSimConfig) {
    let Some(i) = find_agent(world, agent_id) else {
        return;
    };

    let cp = &cfg.agent.cultural;
    let rng = &mut world.rng;

    let neighbors = world.grid.neighbors_of(world.pop.xs[i], world.pop.ys[i]);
    if neighbors.is_empty() {
        return;
    }

    // Oblique transmission: adopt from most prestigious nearby agent
    let mut best_prestige_idx: Option<usize> = None;
    let mut best_prestige = world.pop.prestiges[i] + cp.oblique_prestige_gap;
    for &j_u32 in &neighbors {
        let j = j_u32 as usize;
        if j == i || j >= world.pop.len() {
            continue;
        }
        if world.pop.prestiges[j] > best_prestige {
            best_prestige = world.pop.prestiges[j];
            best_prestige_idx = Some(j);
        }
    }

    if let Some(j) = best_prestige_idx {
        if rand_f64(rng) as f32 <= cp.oblique_adoption_prob {
            let model = world.pop.cultures[j];
            let mut new_culture = world.pop.cultures[i];
            match (rand_f64(rng) * 5.0) as u32 {
                0 => new_culture.authority_norm = model.authority_norm,
                1 => new_culture.sharing_norm = model.sharing_norm,
                2 => new_culture.property_norm = model.property_norm,
                3 => new_culture.trust_outgroup = model.trust_outgroup,
                _ => new_culture.coercion_tolerance = model.coercion_tolerance,
            }
            new_culture.techniques |= model.techniques;
            world.pop.cultures[i] = new_culture;
            return;
        }
    }

    // Horizontal transmission: adopt from random neighbor
    if rand_f64(rng) as f32 <= cp.horizontal_adoption_prob {
        let j_idx = (rand_f64(rng) * neighbors.len() as f64) as usize % neighbors.len();
        let j = neighbors[j_idx] as usize;
        if j != i && j < world.pop.len() {
            let peer = world.pop.cultures[j];
            let mut new_culture = world.pop.cultures[i];
            match (rand_f64(rng) * 4.0) as u32 {
                0 => new_culture.kinship_system = peer.kinship_system,
                1 => new_culture.marriage_rule = peer.marriage_rule,
                2 => new_culture.residence_rule = peer.residence_rule,
                _ => new_culture.inheritance_rule = peer.inheritance_rule,
            }
            match (rand_f64(rng) * 5.0) as u32 {
                0 => {
                    new_culture.authority_norm =
                        (new_culture.authority_norm + peer.authority_norm) * 0.5;
                }
                1 => {
                    new_culture.sharing_norm = (new_culture.sharing_norm + peer.sharing_norm) * 0.5;
                }
                2 => {
                    new_culture.property_norm =
                        (new_culture.property_norm + peer.property_norm) * 0.5;
                }
                3 => {
                    new_culture.trust_outgroup =
                        (new_culture.trust_outgroup + peer.trust_outgroup) * 0.5;
                }
                _ => {
                    new_culture.coercion_tolerance =
                        (new_culture.coercion_tolerance + peer.coercion_tolerance) * 0.5;
                }
            }
            world.pop.cultures[i] = new_culture;
        }
    }
}

// ---------------------------------------------------------------------------
// Group-level event handlers
// ---------------------------------------------------------------------------

fn handle_raid(world: &mut SimWorld, attacker_kin: u32, target_group: u32, cfg: &EventSimConfig) {
    let isp = &cfg.agent.inter_society;
    let ip = &cfg.agent.interaction;

    // Count members
    let mut attacker_count = 0_u32;
    let mut defender_count = 0_u32;
    let mut attacker_power = 0.0_f32;
    let mut defender_power = 0.0_f32;

    for k in 0..world.pop.len() {
        if world.pop.kin_groups[k] == attacker_kin {
            attacker_count += 1;
            attacker_power += world.pop.statuses[k] * ip.power_status_weight
                + world.pop.skill_levels[k] * ip.power_skill_weight
                + world.pop.aggressions[k] * ip.power_aggression_weight;
        } else if world.pop.kin_groups[k] == target_group {
            defender_count += 1;
            defender_power += world.pop.statuses[k] * ip.power_status_weight
                + world.pop.skill_levels[k] * ip.power_skill_weight
                + world.pop.aggressions[k] * ip.power_aggression_weight;
        }
    }

    if attacker_count < isp.min_raid_warriors || defender_count == 0 {
        return;
    }

    if attacker_power <= defender_power * 0.5 {
        return;
    }

    // Successful raid
    let loot = isp.raid_loot_per_warrior * attacker_count as f32;
    let damage = isp.raid_damage_per_warrior * attacker_count as f32;
    let loot_per_defender = (loot / defender_count as f32).min(0.3);
    let loot_per_attacker = (loot_per_defender * defender_count as f32) / attacker_count as f32;
    let damage_per_defender = (damage / defender_count as f32).min(0.15);

    for k in 0..world.pop.len() {
        if world.pop.kin_groups[k] == target_group {
            world.pop.resources[k] = (world.pop.resources[k] - loot_per_defender).max(0.0);
            world.pop.healths[k] = (world.pop.healths[k] - damage_per_defender).max(0.0);
        } else if world.pop.kin_groups[k] == attacker_kin {
            world.pop.resources[k] += loot_per_attacker;
            world.pop.prestiges[k] += 0.01;
        }
    }
    world.counters.raid_events += 1;

    // Check for conquest
    if attacker_power > defender_power * isp.conquest_power_ratio {
        world.tributes.push(TributeRelation {
            vassal_kin: target_group,
            overlord_kin: attacker_kin,
            rate: isp.tribute_rate,
            ticks_remaining: isp.tribute_duration,
        });
        world.counters.conquest_events += 1;
    }
}

fn handle_collect_tribute(world: &mut SimWorld, _cfg: &EventSimConfig) {
    world.tributes.retain_mut(|tr| {
        if tr.ticks_remaining == 0 {
            return false;
        }
        tr.ticks_remaining -= 1;
        let mut tribute_collected = 0.0_f32;
        let mut overlord_count = 0_u32;
        for k in 0..world.pop.len() {
            if world.pop.kin_groups[k] == tr.vassal_kin {
                let payment = world.pop.resources[k] * tr.rate;
                world.pop.resources[k] -= payment;
                tribute_collected += payment;
            } else if world.pop.kin_groups[k] == tr.overlord_kin {
                overlord_count += 1;
            }
        }
        if overlord_count > 0 && tribute_collected > 0.0 {
            let per_overlord = tribute_collected / overlord_count as f32;
            for k in 0..world.pop.len() {
                if world.pop.kin_groups[k] == tr.overlord_kin {
                    world.pop.resources[k] += per_overlord;
                }
            }
        }
        world.counters.tribute_total += tribute_collected;
        true
    });
}

fn handle_migrate(world: &mut SimWorld, kin_group: u32, cfg: &EventSimConfig) {
    let isp = &cfg.agent.inter_society;
    let rng = &mut world.rng;

    // Collect unique kin groups
    let mut kin_ids: Vec<u32> = Vec::new();
    for &kg in &world.pop.kin_groups {
        if !kin_ids.contains(&kg) {
            kin_ids.push(kg);
        }
    }
    if kin_ids.len() <= 1 {
        return;
    }

    // Compute centroids
    let mut centroids: Vec<(u32, f32, f32)> = Vec::new();
    for &kid in &kin_ids {
        let mut sx = 0.0_f32;
        let mut sy = 0.0_f32;
        let mut count = 0_u32;
        for k in 0..world.pop.len() {
            if world.pop.kin_groups[k] == kid {
                sx += world.pop.xs[k];
                sy += world.pop.ys[k];
                count += 1;
            }
        }
        if count > 0 {
            centroids.push((kid, sx / count as f32, sy / count as f32));
        }
    }

    // Check each member of this kin group for migration
    for i in 0..world.pop.len() {
        if world.pop.kin_groups[i] != kin_group {
            continue;
        }
        if world.pop.resources[i] < isp.migration_resource_threshold
            && (rand_f64(rng) as f32) < isp.migration_probability
        {
            let idx = (rand_f64(rng) * kin_ids.len() as f64) as usize % kin_ids.len();
            let new_kin = kin_ids[idx];
            if new_kin != world.pop.kin_groups[i] {
                world.pop.kin_groups[i] = new_kin;
                if let Some(&(_, cx, cy)) = centroids.iter().find(|(k, _, _)| *k == new_kin) {
                    world.pop.xs[i] =
                        cx + (rand_f64(rng) as f32 - 0.5) * cfg.agent.interaction_radius;
                    world.pop.ys[i] =
                        cy + (rand_f64(rng) as f32 - 0.5) * cfg.agent.interaction_radius;
                }
                world.counters.migration_events += 1;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// World-level event handlers
// ---------------------------------------------------------------------------

fn handle_rebuild_spatial_index(world: &mut SimWorld, cfg: &EventSimConfig) {
    world.grid = SpatialGrid::build(
        &world.pop.xs,
        &world.pop.ys,
        cfg.agent.interaction_radius,
        cfg.agent.world_size,
    );
}

fn handle_update_landscape(world: &mut SimWorld, cfg: &EventSimConfig) {
    let ep = &cfg.agent.energy;
    for cell in &mut world.landscape.cells {
        let src = &mut cell.sources[EnergyType::Biomass as usize];
        if src.stock < src.initial_stock {
            src.stock = (src.stock + ep.biomass_regen_rate).min(src.initial_stock);
        }
    }
}

fn handle_measure_state(
    world: &mut SimWorld,
    cfg: &EventSimConfig,
    snapshots: &mut Vec<EventSnapshot>,
) {
    // Build a temporary InteractionEffects-like for institutional detection
    let effects_proxy = InteractionEffectsProxy {
        cooperation_events: world.counters.cooperation_events,
        conflict_events: world.counters.conflict_events,
        total_interactions: world.counters.total_interactions,
        voluntary_transfers: world.counters.voluntary_transfers,
        involuntary_transfers: world.counters.involuntary_transfers,
        intra_kin_conflicts: world.counters.intra_kin_conflicts,
        intra_kin_interactions: world.counters.intra_kin_interactions,
        inter_group_trades: world.counters.inter_group_trades,
    };

    let emergent = measure_emergent_state_from_counters(
        &world.pop,
        &effects_proxy,
        &world.landscape,
        &world.tributes,
        &world.counters,
        cfg,
    );

    snapshots.push(EventSnapshot {
        time: world.now,
        emergent,
    });

    // Reset counters for next measurement window
    world.counters.reset();
}

/// Proxy for the interaction-derived stats needed by measurement.
struct InteractionEffectsProxy {
    cooperation_events: u32,
    conflict_events: u32,
    total_interactions: u32,
    voluntary_transfers: u32,
    involuntary_transfers: u32,
    intra_kin_conflicts: u32,
    intra_kin_interactions: u32,
    #[allow(dead_code)]
    inter_group_trades: u32,
}

fn measure_emergent_state_from_counters(
    pop: &Population,
    proxy: &InteractionEffectsProxy,
    landscape: &EnergyLandscape,
    tributes: &[TributeRelation],
    counters: &EventCounters,
    cfg: &EventSimConfig,
) -> EmergentState {
    let n = pop.len() as u32;
    let gini = if pop.len() > 500 {
        measure_gini_fast(&pop.resources)
    } else {
        measure_gini(&pop.resources)
    };

    // Institutional detection (simplified — uses proxy counts)
    let hierarchy = measure_hierarchy_depth(&pop.patrons);
    let pop_size = n;
    let institutional_type = if hierarchy >= 3 && pop_size > 500 {
        3 // State
    } else if hierarchy >= 2 && pop_size > 150 {
        2 // Chiefdom
    } else if hierarchy >= 1 || pop_size > 50 {
        1 // Tribe
    } else {
        0 // Band
    };

    // Coercion rate
    let delegation_count = pop.patrons.iter().filter(|p| p.is_some()).count() as u32;
    let total_involuntary = proxy.involuntary_transfers + delegation_count;
    let total_transfers = proxy.voluntary_transfers + total_involuntary;
    let coercion_rate = if total_transfers > 0 {
        total_involuntary as f32 / total_transfers as f32
    } else {
        0.0
    };

    // Property norm
    let property_norm_strength = if proxy.intra_kin_interactions > 0 {
        1.0 - (proxy.intra_kin_conflicts as f32 / proxy.intra_kin_interactions as f32)
    } else {
        1.0
    };

    // Patron count and recognized leaders
    let mut patron_set: Vec<u32> = Vec::new();
    let mut patron_tenure_sum = 0_u64;
    let mut patron_tenure_count = 0_u32;
    for (i, p) in pop.patrons.iter().enumerate() {
        if let Some(patron) = p {
            if !patron_set.contains(patron) {
                patron_set.push(*patron);
            }
            patron_tenure_sum += u64::from(pop.patron_ticks[i]);
            patron_tenure_count += 1;
        }
    }
    let patron_count = patron_set.len() as u32;
    let mean_patron_tenure = if patron_tenure_count > 0 {
        patron_tenure_sum as f32 / patron_tenure_count as f32
    } else {
        0.0
    };

    // Recognized leaders
    let n_kin = count_kin_groups(&pop.kin_groups);
    let mut recognized_leaders = 0_u32;
    for kg in 0..n_kin {
        let mut kin_members = 0_u32;
        let mut patron_votes: Vec<(u32, u32)> = Vec::new();
        for k in 0..pop.len() {
            if pop.kin_groups[k] != kg {
                continue;
            }
            kin_members += 1;
            if let Some(p) = pop.patrons[k] {
                if let Some(entry) = patron_votes.iter_mut().find(|(pid, _)| *pid == p) {
                    entry.1 += 1;
                } else {
                    patron_votes.push((p, 1));
                }
            }
        }
        if kin_members > 0 {
            for &(_, count) in &patron_votes {
                if count as f32 / kin_members as f32 >= cfg.agent.institution.leadership_threshold {
                    recognized_leaders += 1;
                }
            }
        }
    }

    // Public goods investment estimate
    let public_goods_investment = if !pop.is_empty() {
        let total_tax: f32 = pop
            .patrons
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.map(|_| pop.resources[i] * cfg.agent.interaction.delegation_tax_rate)
            })
            .sum();
        total_tax * cfg.agent.institution.public_goods_rate
    } else {
        0.0
    };

    // Energy stats
    let mut total_net_energy = 0.0_f64;
    let mut energy_by_type = [0.0_f64; 4];
    // Approximate from landscape state
    for cell in &landscape.cells {
        for (type_idx, source) in cell.sources.iter().enumerate() {
            if source.flow_rate > 0.0 {
                let eroei = source.current_eroei();
                if eroei > 1.0 {
                    let net = source.flow_rate * (1.0 - 1.0 / eroei);
                    energy_by_type[type_idx] += net;
                    total_net_energy += net;
                }
            }
        }
    }

    let trade_total = counters.trade_events;

    EmergentState {
        population_size: n,
        mean_resources: if n > 0 {
            pop.resources.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        gini_coefficient: gini,
        skill_entropy: measure_skill_entropy(&pop.skill_types),
        max_hierarchy_depth: hierarchy,
        num_leaders: pop
            .skill_types
            .iter()
            .filter(|s| **s == SkillType::Leader)
            .count() as u32,
        mean_group_size: mean_group_size(&pop.kin_groups),
        num_kin_groups: count_kin_groups(&pop.kin_groups),
        cooperation_rate: if proxy.total_interactions > 0 {
            proxy.cooperation_events as f32 / proxy.total_interactions as f32
        } else {
            0.0
        },
        conflict_rate: if proxy.total_interactions > 0 {
            proxy.conflict_events as f32 / proxy.total_interactions as f32
        } else {
            0.0
        },
        mean_prestige: if n > 0 {
            pop.prestiges.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_health: if n > 0 {
            pop.healths.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_innovation: if n > 0 {
            pop.innovations.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
        dominant_energy: {
            let mut best = 0_u8;
            let mut best_val = energy_by_type[0];
            for (i, &val) in energy_by_type.iter().enumerate().skip(1) {
                if val > best_val {
                    best = i as u8;
                    best_val = val;
                }
            }
            best
        },
        energy_per_capita: if n > 0 {
            (total_net_energy / f64::from(n)) as f32
        } else {
            0.0
        },
        mean_eroei: {
            let mut sum = 0.0_f64;
            let mut count = 0_u32;
            for cell in &landscape.cells {
                for src in &cell.sources {
                    if src.flow_rate > 0.0 && src.stock > 0.0 {
                        sum += src.current_eroei();
                        count += 1;
                    }
                }
            }
            if count > 0 {
                (sum / f64::from(count)) as f32
            } else {
                0.0
            }
        },
        biomass_depletion: landscape.mean_depletion(EnergyType::Biomass) as f32,
        fossil_depletion: landscape.mean_depletion(EnergyType::Fossil) as f32,
        coercion_rate,
        property_norm_strength,
        institutional_type: institutional_type as u8,
        public_goods_investment,
        patron_count,
        recognized_leaders,
        mean_patron_tenure,
        raid_events: counters.raid_events,
        conquest_events: counters.conquest_events,
        tribute_flows: counters.tribute_total,
        migration_events: counters.migration_events,
        num_active_societies: count_kin_groups(&pop.kin_groups),
        inter_group_trade_rate: if trade_total > 0 {
            counters.inter_group_trades as f32 / trade_total as f32
        } else {
            0.0
        },
        active_tributes: tributes.len() as u32,
        mean_authority_norm: if n > 0 {
            pop.cultures.iter().map(|c| c.authority_norm).sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_sharing_norm: if n > 0 {
            pop.cultures.iter().map(|c| c.sharing_norm).sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_property_norm: if n > 0 {
            pop.cultures.iter().map(|c| c.property_norm).sum::<f32>() / n as f32
        } else {
            0.0
        },
        mean_trust_outgroup: if n > 0 {
            pop.cultures.iter().map(|c| c.trust_outgroup).sum::<f32>() / n as f32
        } else {
            0.0
        },
        cultural_diversity: measure_cultural_diversity(pop),
        dominant_kinship: dominant_kinship(pop),
        dominant_marriage: dominant_marriage(pop),
        mean_coercion_tolerance: if n > 0 {
            pop.cultures
                .iter()
                .map(|c| c.coercion_tolerance)
                .sum::<f32>()
                / n as f32
        } else {
            0.0
        },
        technique_count: mean_technique_count(pop),
        coordination_failure_index: if counters.total_cooperative_optimal > 0.0 {
            (1.0 - (counters.total_actual_surplus / counters.total_cooperative_optimal).min(1.0))
                .clamp(0.0, 1.0)
        } else {
            0.0
        },
        mean_trust: if n > 0 {
            pop.trust_memory.iter().sum::<f32>() / n as f32
        } else {
            0.0
        },
    }
}

// ---------------------------------------------------------------------------
// Helper: child culture creation
// ---------------------------------------------------------------------------

fn create_child_culture(
    mother: &Culture,
    father: &Culture,
    lp: &LifecycleParams,
    cp: &CulturalParams,
    rng: &mut u64,
) -> Culture {
    Culture {
        kinship_system: if rand_f64(rng) < cp.vertical_mutation_prob {
            match (rand_f64(rng) * 3.0) as u32 {
                0 => KinshipSystem::Patrilineal,
                1 => KinshipSystem::Matrilineal,
                _ => KinshipSystem::Bilateral,
            }
        } else if rand_f64(rng) < 0.5 {
            mother.kinship_system
        } else {
            father.kinship_system
        },
        marriage_rule: if rand_f64(rng) < cp.vertical_mutation_prob {
            match (rand_f64(rng) * 3.0) as u32 {
                0 => MarriageRule::Monogamy,
                1 => MarriageRule::Polygyny,
                _ => MarriageRule::Polyandry,
            }
        } else if rand_f64(rng) < 0.5 {
            mother.marriage_rule
        } else {
            father.marriage_rule
        },
        residence_rule: if rand_f64(rng) < cp.vertical_mutation_prob {
            match (rand_f64(rng) * 3.0) as u32 {
                0 => ResidenceRule::Patrilocal,
                1 => ResidenceRule::Matrilocal,
                _ => ResidenceRule::Neolocal,
            }
        } else if rand_f64(rng) < 0.5 {
            mother.residence_rule
        } else {
            father.residence_rule
        },
        inheritance_rule: if rand_f64(rng) < cp.vertical_mutation_prob {
            match (rand_f64(rng) * 3.0) as u32 {
                0 => InheritanceRule::Primogeniture,
                1 => InheritanceRule::Partible,
                _ => InheritanceRule::Matrilineal,
            }
        } else if rand_f64(rng) < 0.5 {
            mother.inheritance_rule
        } else {
            father.inheritance_rule
        },
        authority_norm: ((mother.authority_norm + father.authority_norm) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * cp.cultural_mutation_magnitude)
            .clamp(0.0, 1.0),
        coercion_tolerance: ((mother.coercion_tolerance + father.coercion_tolerance) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * cp.cultural_mutation_magnitude)
            .clamp(0.0, 1.0),
        sharing_norm: ((mother.sharing_norm + father.sharing_norm) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * cp.cultural_mutation_magnitude)
            .clamp(0.0, 1.0),
        property_norm: ((mother.property_norm + father.property_norm) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * cp.cultural_mutation_magnitude)
            .clamp(0.0, 1.0),
        techniques: mother.techniques
            | father.techniques
            | if rand_f64(rng) < lp.norm_mutation_prob {
                1 << ((rand_f64(rng) * 16.0) as u64)
            } else {
                0
            },
        trust_outgroup: ((mother.trust_outgroup + father.trust_outgroup) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * cp.cultural_mutation_magnitude)
            .clamp(0.0, 1.0),
        risk_tolerance: ((mother.risk_tolerance + father.risk_tolerance) * 0.5
            + (rand_f64(rng) as f32 - 0.5) * cp.cultural_mutation_magnitude)
            .clamp(0.0, 1.0),
    }
}

/// Find a mate for a female agent (event-driven version using Population directly).
fn find_mate_event(
    pop: &Population,
    female_idx: usize,
    rng: &mut u64,
    cfg: &AgentSimConfig,
) -> Option<usize> {
    let ms = &cfg.mate_selection;
    let lp = &cfg.lifecycle;
    let fx = pop.xs[female_idx];
    let fy = pop.ys[female_idx];
    let r_sq = cfg.interaction_radius * cfg.interaction_radius * 4.0;

    let mut best: Option<usize> = None;
    let mut best_score = f32::NEG_INFINITY;

    for j in 0..pop.len() {
        if pop.sexes[j] != Sex::Male || pop.ages[j] < lp.min_reproduction_age {
            continue;
        }
        let dx = fx - pop.xs[j];
        let dy = fy - pop.ys[j];
        if dx * dx + dy * dy > r_sq {
            continue;
        }
        let score = pop.statuses[j] * ms.status_weight
            + pop.resources[j] * ms.resource_weight
            + pop.prestiges[j] * ms.prestige_weight
            + rand_f64(rng) as f32 * ms.noise_weight;
        if score > best_score {
            best_score = score;
            best = Some(j);
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Initial event scheduling
// ---------------------------------------------------------------------------

fn schedule_initial_agent_events(
    queue: &mut EventQueue,
    pop: &Population,
    rng: &mut u64,
    ep: &EventParams,
) {
    for idx in 0..pop.len() {
        let agent_id = pop.ids[idx];
        // Schedule all event types for each agent
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Forage,
            ep.forage_base_rate,
            rng,
        ));
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Interact,
            ep.interact_base_rate,
            rng,
        ));
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Move,
            ep.move_base_rate,
            rng,
        ));
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Reproduce,
            ep.reproduce_base_rate,
            rng,
        ));
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Age,
            ep.age_base_rate,
            rng,
        ));
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Learn,
            ep.learn_base_rate,
            rng,
        ));
        queue.push(schedule_agent(
            0.0,
            agent_id,
            AgentAction::Transmit,
            ep.transmit_base_rate,
            rng,
        ));
    }
}

fn schedule_initial_group_events(
    queue: &mut EventQueue,
    pop: &Population,
    rng: &mut u64,
    ep: &EventParams,
) {
    // Collect unique kin groups
    let mut kin_ids: Vec<u32> = Vec::new();
    for &kg in &pop.kin_groups {
        if !kin_ids.contains(&kg) {
            kin_ids.push(kg);
        }
    }
    for &kin in &kin_ids {
        queue.push(schedule_group(
            0.0,
            kin,
            GroupAction::Migrate,
            ep.migrate_base_rate,
            rng,
        ));
    }
}

fn schedule_initial_world_events(queue: &mut EventQueue, ep: &EventParams) {
    queue.push(schedule_world(
        0.0,
        WorldAction::RebuildSpatialIndex,
        ep.spatial_rebuild_interval,
    ));
    queue.push(schedule_world(
        0.0,
        WorldAction::MeasureState,
        ep.measure_interval,
    ));
    queue.push(schedule_world(
        0.0,
        WorldAction::UpdateLandscape,
        ep.landscape_update_interval,
    ));
}

fn schedule_new_agent_events(
    queue: &mut EventQueue,
    agent_id: u64,
    now: SimTime,
    rng: &mut u64,
    ep: &EventParams,
) {
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Forage,
        ep.forage_base_rate,
        rng,
    ));
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Interact,
        ep.interact_base_rate,
        rng,
    ));
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Move,
        ep.move_base_rate,
        rng,
    ));
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Reproduce,
        ep.reproduce_base_rate,
        rng,
    ));
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Age,
        ep.age_base_rate,
        rng,
    ));
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Learn,
        ep.learn_base_rate,
        rng,
    ));
    queue.push(schedule_agent(
        now,
        agent_id,
        AgentAction::Transmit,
        ep.transmit_base_rate,
        rng,
    ));
}

// ---------------------------------------------------------------------------
// Main event-driven simulation
// ---------------------------------------------------------------------------

/// Run the event-driven simulation.
#[must_use]
pub fn simulate_event_driven(cfg: EventSimConfig) -> EventSimResult {
    let pop = seed_population(&cfg.agent);
    let landscape = init_energy_landscape(&cfg.agent);
    let grid = SpatialGrid::build(
        &pop.xs,
        &pop.ys,
        cfg.agent.interaction_radius,
        cfg.agent.world_size,
    );
    let mut rng = cfg.agent.seed.wrapping_add(0xE0E0).max(1);

    let mut alive: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut id_to_index: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for (idx, &id) in pop.ids.iter().enumerate() {
        alive.insert(id);
        id_to_index.insert(id, idx);
    }

    let mut world = SimWorld {
        pop,
        landscape,
        grid,
        tributes: Vec::new(),
        next_id: cfg.agent.initial_population as u64,
        rng: cfg.agent.seed.wrapping_add(0xCAFE).max(1),
        now: 0.0,
        counters: EventCounters::default(),
        alive,
        id_to_index,
    };

    let mut queue = EventQueue::with_capacity(world.pop.len() * 8);
    let mut snapshots: Vec<EventSnapshot> = Vec::new();

    // Schedule initial events
    schedule_initial_agent_events(&mut queue, &world.pop, &mut rng, &cfg.event);
    schedule_initial_group_events(&mut queue, &world.pop, &mut rng, &cfg.event);
    schedule_initial_world_events(&mut queue, &cfg.event);

    let mut events_processed: u64 = 0;
    let end_time = cfg.end_time;
    let ep = &cfg.event;

    while let Some(event) = queue.pop() {
        if event.time > end_time {
            queue.push(event);
            break;
        }

        world.now = event.time;

        // Check min population
        if (world.pop.len() as u32) < cfg.agent.min_population {
            break;
        }

        match event.kind {
            EventKind::Agent { id, ref action } => {
                // Skip dead agents
                if !world.alive.contains(&id) {
                    events_processed += 1;
                    continue;
                }

                let next_id_before = world.next_id;

                match action {
                    AgentAction::Forage => handle_forage(&mut world, id, &cfg),
                    AgentAction::Interact => handle_interact(&mut world, id, &cfg),
                    AgentAction::Move => handle_move(&mut world, id, &cfg),
                    AgentAction::Reproduce => handle_reproduce(&mut world, id, &cfg),
                    AgentAction::Age => handle_age(&mut world, id, &cfg),
                    AgentAction::Learn => handle_learn(&mut world, id, &cfg),
                    AgentAction::Transmit => handle_transmit(&mut world, id, &cfg),
                }

                // Reschedule this event type if agent is still alive
                if world.alive.contains(&id) {
                    let rate = match action {
                        AgentAction::Forage => ep.forage_base_rate,
                        AgentAction::Interact => ep.interact_base_rate,
                        AgentAction::Move => ep.move_base_rate,
                        AgentAction::Reproduce => ep.reproduce_base_rate,
                        AgentAction::Age => ep.age_base_rate,
                        AgentAction::Learn => ep.learn_base_rate,
                        AgentAction::Transmit => ep.transmit_base_rate,
                    };
                    queue.push(schedule_agent(
                        world.now,
                        id,
                        action.clone(),
                        rate,
                        &mut world.rng,
                    ));
                }

                // Schedule events for any newborns
                if world.next_id > next_id_before {
                    for new_id in next_id_before..world.next_id {
                        schedule_new_agent_events(
                            &mut queue,
                            new_id,
                            world.now,
                            &mut world.rng,
                            ep,
                        );
                    }
                }
            }
            EventKind::Group {
                kin_group,
                ref action,
            } => {
                match action {
                    GroupAction::Raid { target_group } => {
                        handle_raid(&mut world, kin_group, *target_group, &cfg);
                    }
                    GroupAction::Migrate => {
                        handle_migrate(&mut world, kin_group, &cfg);
                        // Reschedule migration check for this group
                        queue.push(schedule_group(
                            world.now,
                            kin_group,
                            GroupAction::Migrate,
                            ep.migrate_base_rate,
                            &mut world.rng,
                        ));

                        // Also try to schedule raids from this group
                        let mean_aggr = kin_group_mean_aggression(&world.pop, kin_group);
                        if mean_aggr > cfg.agent.inter_society.raid_aggression_threshold {
                            // Find nearest target
                            let (ax, ay) = kin_group_centroid(&world.pop, kin_group);
                            let mut kin_ids: Vec<u32> = Vec::new();
                            for &kg in &world.pop.kin_groups {
                                if !kin_ids.contains(&kg) {
                                    kin_ids.push(kg);
                                }
                            }
                            let mut best_target: Option<u32> = None;
                            let mut best_dist = f32::MAX;
                            for &target in &kin_ids {
                                if target == kin_group {
                                    continue;
                                }
                                let (tx, ty) = kin_group_centroid(&world.pop, target);
                                let dist = ((ax - tx).powi(2) + (ay - ty).powi(2)).sqrt();
                                if dist < cfg.agent.inter_society.raid_range && dist < best_dist {
                                    best_dist = dist;
                                    best_target = Some(target);
                                }
                            }
                            if let Some(target) = best_target {
                                if rand_f64(&mut world.rng) < f64::from(mean_aggr) {
                                    queue.push(schedule_group(
                                        world.now,
                                        kin_group,
                                        GroupAction::Raid {
                                            target_group: target,
                                        },
                                        ep.raid_base_rate,
                                        &mut world.rng,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            EventKind::World { ref action } => {
                match action {
                    WorldAction::RebuildSpatialIndex => {
                        handle_rebuild_spatial_index(&mut world, &cfg);
                        queue.push(schedule_world(
                            world.now,
                            WorldAction::RebuildSpatialIndex,
                            ep.spatial_rebuild_interval,
                        ));
                    }
                    WorldAction::MeasureState => {
                        handle_measure_state(&mut world, &cfg, &mut snapshots);
                        queue.push(schedule_world(
                            world.now,
                            WorldAction::MeasureState,
                            ep.measure_interval,
                        ));
                    }
                    WorldAction::UpdateLandscape => {
                        handle_update_landscape(&mut world, &cfg);
                        // Also handle tribute collection as a periodic world event
                        handle_collect_tribute(&mut world, &cfg);
                        queue.push(schedule_world(
                            world.now,
                            WorldAction::UpdateLandscape,
                            ep.landscape_update_interval,
                        ));
                    }
                }
            }
        }
        events_processed += 1;
    }

    EventSimResult {
        snapshots,
        final_population: world.pop,
        final_landscape: world.landscape,
        events_processed,
    }
}

// ---------------------------------------------------------------------------
// Observer support for streaming to TUI / external consumers
// ---------------------------------------------------------------------------

/// Snapshot emitted to observers during event-driven simulation.
/// Includes per-continent aggregation derived from agent positions.
#[derive(Clone, Debug)]
pub struct EventMapFrame {
    pub time: f64,
    pub emergent: EmergentState,
    pub continent_populations: [u32; 4],
    pub continent_mean_resources: [f32; 4],
    pub continent_mean_health: [f32; 4],
    pub continent_mean_innovation: [f32; 4],
    pub continent_cooperation_counts: [u32; 4],
    pub continent_conflict_counts: [u32; 4],
    pub total_population: u32,
    pub events_processed: u64,
    /// Accumulated event counters for this measurement window.
    pub raid_events: u32,
    pub migration_events: u32,
    pub conquest_events: u32,
    pub tribute_total: f32,
}

/// Map an (x, y) position in the world to a continent index (0-3).
///
/// Layout (inspired by world geography, using asymmetric thresholds):
/// ```text
///   x < 0.5              x >= 0.5
///   ┌──────────────┬─────────────────────┐
///   │              │     Eurasia (1)      │  y < 0.35
///   │              ├──────────┬──────────┤
///   │  Americas    │ Eurasia  │ Oceania  │  0.35 <= y < 0.65
///   │    (2)       │   (1)    │   (3)    │  (Oceania: x > 0.7)
///   │              ├──────────┴──────────┤
///   │              │     Africa (0)      │  y >= 0.65
///   └──────────────┴─────────────────────┘
/// ```
/// Thresholds: mid=0.5*world_size, bottom=0.65*world_size, right=0.7*world_size.
pub fn continent_from_position(x: f32, y: f32, world_size: f32) -> usize {
    let mid = world_size * 0.5;
    let right_third = world_size * 0.7;
    let bottom_third = world_size * 0.65;

    if x < mid {
        2 // Americas (left side)
    } else if y < bottom_third {
        if x > right_third && y > world_size * 0.35 {
            3 // Oceania (right portion of upper-middle)
        } else {
            1 // Eurasia (right side, upper)
        }
    } else {
        0 // Africa (right side, lower)
    }
}

/// Aggregate per-continent stats from population positions.
fn aggregate_continent_stats(
    pop: &Population,
    world_size: f32,
) -> ([u32; 4], [f32; 4], [f32; 4], [f32; 4]) {
    let mut counts = [0_u32; 4];
    let mut res_sums = [0.0_f32; 4];
    let mut health_sums = [0.0_f32; 4];
    let mut innov_sums = [0.0_f32; 4];

    for i in 0..pop.len() {
        let ci = continent_from_position(pop.xs[i], pop.ys[i], world_size);
        counts[ci] += 1;
        res_sums[ci] += pop.resources[i];
        health_sums[ci] += pop.healths[i];
        innov_sums[ci] += pop.innovations[i];
    }

    let mut mean_res = [0.0_f32; 4];
    let mut mean_health = [0.0_f32; 4];
    let mut mean_innov = [0.0_f32; 4];
    for ci in 0..4 {
        if counts[ci] > 0 {
            let n = counts[ci] as f32;
            mean_res[ci] = res_sums[ci] / n;
            mean_health[ci] = health_sums[ci] / n;
            mean_innov[ci] = innov_sums[ci] / n;
        }
    }

    (counts, mean_res, mean_health, mean_innov)
}

/// Count cooperation and conflict events per continent by examining agent positions.
fn aggregate_continent_interactions(
    pop: &Population,
    counters: &EventCounters,
    world_size: f32,
) -> ([u32; 4], [u32; 4]) {
    // We don't have per-agent interaction tracking, so distribute global counts
    // proportional to population per continent.
    let mut counts = [0_u32; 4];
    for i in 0..pop.len() {
        let ci = continent_from_position(pop.xs[i], pop.ys[i], world_size);
        counts[ci] += 1;
    }
    let total = pop.len().max(1) as f32;

    let mut coop = [0_u32; 4];
    let mut conf = [0_u32; 4];
    for ci in 0..4 {
        let frac = counts[ci] as f32 / total;
        coop[ci] = (counters.cooperation_events as f32 * frac) as u32;
        conf[ci] = (counters.conflict_events as f32 * frac) as u32;
    }
    (coop, conf)
}

/// Run the event-driven simulation with an observer callback on each measurement.
#[must_use]
pub fn simulate_event_driven_with_observer<F>(
    cfg: EventSimConfig,
    mut observer: F,
) -> EventSimResult
where
    F: FnMut(&EventMapFrame),
{
    let pop = seed_population(&cfg.agent);
    let landscape = init_energy_landscape(&cfg.agent);
    let grid = SpatialGrid::build(
        &pop.xs,
        &pop.ys,
        cfg.agent.interaction_radius,
        cfg.agent.world_size,
    );
    let mut rng = cfg.agent.seed.wrapping_add(0xE0E0).max(1);

    let mut alive: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut id_to_index: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for (idx, &id) in pop.ids.iter().enumerate() {
        alive.insert(id);
        id_to_index.insert(id, idx);
    }

    let mut world = SimWorld {
        pop,
        landscape,
        grid,
        tributes: Vec::new(),
        next_id: cfg.agent.initial_population as u64,
        rng: cfg.agent.seed.wrapping_add(0xCAFE).max(1),
        now: 0.0,
        counters: EventCounters::default(),
        alive,
        id_to_index,
    };

    let mut queue = EventQueue::with_capacity(world.pop.len() * 8);
    let mut snapshots: Vec<EventSnapshot> = Vec::new();

    schedule_initial_agent_events(&mut queue, &world.pop, &mut rng, &cfg.event);
    schedule_initial_group_events(&mut queue, &world.pop, &mut rng, &cfg.event);
    schedule_initial_world_events(&mut queue, &cfg.event);

    let mut events_processed: u64 = 0;
    let end_time = cfg.end_time;
    let ep = &cfg.event;
    let world_size = cfg.agent.world_size;

    while let Some(event) = queue.pop() {
        if event.time > end_time {
            queue.push(event);
            break;
        }

        world.now = event.time;

        if (world.pop.len() as u32) < cfg.agent.min_population {
            break;
        }

        match event.kind {
            EventKind::Agent { id, ref action } => {
                if !world.alive.contains(&id) {
                    events_processed += 1;
                    continue;
                }

                let next_id_before = world.next_id;

                match action {
                    AgentAction::Forage => handle_forage(&mut world, id, &cfg),
                    AgentAction::Interact => handle_interact(&mut world, id, &cfg),
                    AgentAction::Move => handle_move(&mut world, id, &cfg),
                    AgentAction::Reproduce => handle_reproduce(&mut world, id, &cfg),
                    AgentAction::Age => handle_age(&mut world, id, &cfg),
                    AgentAction::Learn => handle_learn(&mut world, id, &cfg),
                    AgentAction::Transmit => handle_transmit(&mut world, id, &cfg),
                }

                if world.alive.contains(&id) {
                    let rate = match action {
                        AgentAction::Forage => ep.forage_base_rate,
                        AgentAction::Interact => ep.interact_base_rate,
                        AgentAction::Move => ep.move_base_rate,
                        AgentAction::Reproduce => ep.reproduce_base_rate,
                        AgentAction::Age => ep.age_base_rate,
                        AgentAction::Learn => ep.learn_base_rate,
                        AgentAction::Transmit => ep.transmit_base_rate,
                    };
                    queue.push(schedule_agent(
                        world.now,
                        id,
                        action.clone(),
                        rate,
                        &mut world.rng,
                    ));
                }

                if world.next_id > next_id_before {
                    for new_id in next_id_before..world.next_id {
                        schedule_new_agent_events(
                            &mut queue,
                            new_id,
                            world.now,
                            &mut world.rng,
                            ep,
                        );
                    }
                }
            }
            EventKind::Group {
                kin_group,
                ref action,
            } => match action {
                GroupAction::Raid { target_group } => {
                    handle_raid(&mut world, kin_group, *target_group, &cfg);
                }
                GroupAction::Migrate => {
                    handle_migrate(&mut world, kin_group, &cfg);
                    queue.push(schedule_group(
                        world.now,
                        kin_group,
                        GroupAction::Migrate,
                        ep.migrate_base_rate,
                        &mut world.rng,
                    ));

                    let mean_aggr = kin_group_mean_aggression(&world.pop, kin_group);
                    if mean_aggr > cfg.agent.inter_society.raid_aggression_threshold {
                        let (ax, ay) = kin_group_centroid(&world.pop, kin_group);
                        let mut kin_ids: Vec<u32> = Vec::new();
                        for &kg in &world.pop.kin_groups {
                            if !kin_ids.contains(&kg) {
                                kin_ids.push(kg);
                            }
                        }
                        let mut best_target: Option<u32> = None;
                        let mut best_dist = f32::MAX;
                        for &target in &kin_ids {
                            if target == kin_group {
                                continue;
                            }
                            let (tx, ty) = kin_group_centroid(&world.pop, target);
                            let dist = ((ax - tx).powi(2) + (ay - ty).powi(2)).sqrt();
                            if dist < cfg.agent.inter_society.raid_range && dist < best_dist {
                                best_dist = dist;
                                best_target = Some(target);
                            }
                        }
                        if let Some(target) = best_target {
                            if rand_f64(&mut world.rng) < f64::from(mean_aggr) {
                                queue.push(schedule_group(
                                    world.now,
                                    kin_group,
                                    GroupAction::Raid {
                                        target_group: target,
                                    },
                                    ep.raid_base_rate,
                                    &mut world.rng,
                                ));
                            }
                        }
                    }
                }
            },
            EventKind::World { ref action } => {
                match action {
                    WorldAction::RebuildSpatialIndex => {
                        handle_rebuild_spatial_index(&mut world, &cfg);
                        queue.push(schedule_world(
                            world.now,
                            WorldAction::RebuildSpatialIndex,
                            ep.spatial_rebuild_interval,
                        ));
                    }
                    WorldAction::MeasureState => {
                        // Capture counters before measurement resets them
                        let raid_events = world.counters.raid_events;
                        let migration_events = world.counters.migration_events;
                        let conquest_events = world.counters.conquest_events;
                        let tribute_total = world.counters.tribute_total;

                        handle_measure_state(&mut world, &cfg, &mut snapshots);

                        // Build map frame from current population state
                        let (cpop, cres, chealth, cinnov) =
                            aggregate_continent_stats(&world.pop, world_size);
                        let (ccoop, cconf) = aggregate_continent_interactions(
                            &world.pop,
                            &EventCounters {
                                cooperation_events: snapshots.last().map_or(0, |s| {
                                    (s.emergent.cooperation_rate
                                        * s.emergent.population_size as f32)
                                        as u32
                                }),
                                conflict_events: snapshots.last().map_or(0, |s| {
                                    (s.emergent.conflict_rate * s.emergent.population_size as f32)
                                        as u32
                                }),
                                ..EventCounters::default()
                            },
                            world_size,
                        );

                        if let Some(snap) = snapshots.last() {
                            let map_frame = EventMapFrame {
                                time: snap.time,
                                emergent: snap.emergent,
                                continent_populations: cpop,
                                continent_mean_resources: cres,
                                continent_mean_health: chealth,
                                continent_mean_innovation: cinnov,
                                continent_cooperation_counts: ccoop,
                                continent_conflict_counts: cconf,
                                total_population: snap.emergent.population_size,
                                events_processed,
                                raid_events,
                                migration_events,
                                conquest_events,
                                tribute_total,
                            };
                            observer(&map_frame);
                        }

                        queue.push(schedule_world(
                            world.now,
                            WorldAction::MeasureState,
                            ep.measure_interval,
                        ));
                    }
                    WorldAction::UpdateLandscape => {
                        handle_update_landscape(&mut world, &cfg);
                        handle_collect_tribute(&mut world, &cfg);
                        queue.push(schedule_world(
                            world.now,
                            WorldAction::UpdateLandscape,
                            ep.landscape_update_interval,
                        ));
                    }
                }
            }
        }
        events_processed += 1;
    }

    EventSimResult {
        snapshots,
        final_population: world.pop,
        final_landscape: world.landscape,
        events_processed,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_event_config() -> EventSimConfig {
        EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 60,
                world_size: 30.0,
                ..AgentSimConfig::default()
            },
            event: EventParams::default(),
            end_time: 50.0,
        }
    }

    // --- Layer 1: Event loop mechanics ---

    #[test]
    fn simulation_runs_and_produces_snapshots() {
        let result = simulate_event_driven(default_event_config());
        assert!(!result.snapshots.is_empty(), "should produce snapshots");
        assert!(result.events_processed > 0, "should process events");
    }

    #[test]
    fn snapshots_have_increasing_time() {
        let result = simulate_event_driven(default_event_config());
        for window in result.snapshots.windows(2) {
            assert!(
                window[1].time >= window[0].time,
                "snapshot times should increase: {} >= {}",
                window[1].time,
                window[0].time
            );
        }
    }

    #[test]
    fn population_stays_positive() {
        let result = simulate_event_driven(default_event_config());
        for snap in &result.snapshots {
            assert!(
                snap.emergent.population_size > 0,
                "population should stay positive"
            );
        }
    }

    // --- Layer 2: Property-based invariants ---

    #[test]
    fn emergent_state_values_are_bounded() {
        let result = simulate_event_driven(default_event_config());
        for snap in &result.snapshots {
            let e = &snap.emergent;
            assert!((0.0..=1.0).contains(&e.gini_coefficient));
            assert!((0.0..=1.0).contains(&e.skill_entropy));
            assert!((0.0..=1.0).contains(&e.cooperation_rate));
            assert!((0.0..=1.0).contains(&e.conflict_rate));
            assert!(e.mean_health >= 0.0 && e.mean_health <= 1.0);
        }
    }

    #[test]
    fn resources_never_go_negative() {
        let result = simulate_event_driven(default_event_config());
        for &res in &result.final_population.resources {
            assert!(res >= 0.0, "resources should never be negative, got {res}");
        }
    }

    #[test]
    fn health_stays_bounded() {
        let result = simulate_event_driven(default_event_config());
        for &h in &result.final_population.healths {
            assert!(
                (0.0..=1.0).contains(&h),
                "health should be in [0,1], got {h}"
            );
        }
    }

    #[test]
    fn spatial_positions_stay_within_world_bounds() {
        let cfg = default_event_config();
        let result = simulate_event_driven(cfg.clone());
        let world_size = cfg.agent.world_size;
        for (&x, &y) in result
            .final_population
            .xs
            .iter()
            .zip(result.final_population.ys.iter())
        {
            assert!(
                x >= 0.0 && x <= world_size,
                "x={x} out of bounds [0, {world_size}]"
            );
            assert!(
                y >= 0.0 && y <= world_size,
                "y={y} out of bounds [0, {world_size}]"
            );
        }
    }

    #[test]
    fn all_patron_references_point_to_valid_indices() {
        let result = simulate_event_driven(default_event_config());
        let pop = &result.final_population;
        let n = pop.len();
        for (k, patron) in pop.patrons.iter().enumerate() {
            if let Some(p) = patron {
                assert!(
                    (*p as usize) < n,
                    "agent {k} has patron index {p} but population size is {n}"
                );
            }
        }
        for (k, partner) in pop.partners.iter().enumerate() {
            if let Some(p) = partner {
                assert!(
                    (*p as usize) < n,
                    "agent {k} has partner index {p} but population size is {n}"
                );
            }
        }
    }

    // --- Layer 3: Behavioral tests ---

    #[test]
    fn interactions_produce_events() {
        let result = simulate_event_driven(default_event_config());
        let total_coop: f32 = result
            .snapshots
            .iter()
            .map(|s| s.emergent.cooperation_rate)
            .sum();
        let total_conflict: f32 = result
            .snapshots
            .iter()
            .map(|s| s.emergent.conflict_rate)
            .sum();
        // At least some interactions should have occurred
        assert!(
            total_coop > 0.0 || total_conflict > 0.0,
            "should have interaction events"
        );
    }

    #[test]
    fn innovation_grows_over_time() {
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 60,
                world_size: 30.0,
                ..AgentSimConfig::default()
            },
            end_time: 100.0,
            ..EventSimConfig::default()
        };
        let result = simulate_event_driven(cfg);
        if result.snapshots.len() >= 10 {
            let early = result.snapshots[2].emergent.mean_innovation;
            let Some(last_snap) = result.snapshots.last() else {
                panic!("expected snapshots");
            };
            let late = last_snap.emergent.mean_innovation;
            assert!(
                late > early,
                "innovation should grow: early={early:.4} late={late:.4}"
            );
        }
    }

    #[test]
    fn cultural_diversity_is_bounded() {
        let result = simulate_event_driven(default_event_config());
        for snap in &result.snapshots {
            assert!(
                (0.0..=1.0).contains(&snap.emergent.cultural_diversity),
                "cultural_diversity should be 0-1, got {}",
                snap.emergent.cultural_diversity
            );
        }
    }

    #[test]
    fn different_seeds_produce_different_results() {
        let cfg1 = EventSimConfig {
            agent: AgentSimConfig {
                seed: 42,
                initial_population: 40,
                world_size: 25.0,
                ..AgentSimConfig::default()
            },
            end_time: 30.0,
            ..EventSimConfig::default()
        };
        let cfg2 = EventSimConfig {
            agent: AgentSimConfig {
                seed: 999,
                initial_population: 40,
                world_size: 25.0,
                ..AgentSimConfig::default()
            },
            end_time: 30.0,
            ..EventSimConfig::default()
        };
        let r1 = simulate_event_driven(cfg1);
        let r2 = simulate_event_driven(cfg2);

        // Final populations should differ
        assert_ne!(
            r1.final_population.len(),
            r2.final_population.len(),
            "different seeds should produce different population sizes (or this is very unlikely)"
        );
    }

    #[test]
    fn empty_simulation_completes() {
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 0,
                min_population: 0,
                ..AgentSimConfig::default()
            },
            end_time: 10.0,
            ..EventSimConfig::default()
        };
        let result = simulate_event_driven(cfg);
        assert_eq!(result.final_population.len(), 0);
    }

    #[test]
    fn tribute_flows_are_non_negative() {
        let result = simulate_event_driven(default_event_config());
        for snap in &result.snapshots {
            assert!(
                snap.emergent.tribute_flows >= 0.0,
                "tribute flows should be non-negative"
            );
        }
    }

    // --- continent_from_position tests ---

    #[test]
    fn continent_from_position_americas_left_half() {
        let ws = 100.0_f32;
        // Anything with x < 0.5 * world_size is Americas (index 2)
        assert_eq!(continent_from_position(10.0, 10.0, ws), 2);
        assert_eq!(continent_from_position(10.0, 80.0, ws), 2);
        assert_eq!(continent_from_position(49.0, 50.0, ws), 2);
        assert_eq!(continent_from_position(0.0, 0.0, ws), 2);
    }

    #[test]
    fn continent_from_position_eurasia_upper_right() {
        let ws = 100.0_f32;
        // x >= 50, y < 35 -> Eurasia (1)
        assert_eq!(continent_from_position(60.0, 10.0, ws), 1);
        assert_eq!(continent_from_position(80.0, 20.0, ws), 1);
        assert_eq!(continent_from_position(99.0, 0.0, ws), 1);
        // x >= 50, 35 <= y < 65, x <= 70 -> Eurasia (1)
        assert_eq!(continent_from_position(55.0, 40.0, ws), 1);
        assert_eq!(continent_from_position(65.0, 50.0, ws), 1);
    }

    #[test]
    fn continent_from_position_oceania_right_middle() {
        let ws = 100.0_f32;
        // x > 70, 35 <= y < 65 -> Oceania (3)
        assert_eq!(continent_from_position(75.0, 40.0, ws), 3);
        assert_eq!(continent_from_position(90.0, 50.0, ws), 3);
        assert_eq!(continent_from_position(80.0, 60.0, ws), 3);
    }

    #[test]
    fn continent_from_position_africa_lower_right() {
        let ws = 100.0_f32;
        // x >= 50, y >= 65 -> Africa (0)
        assert_eq!(continent_from_position(60.0, 70.0, ws), 0);
        assert_eq!(continent_from_position(80.0, 90.0, ws), 0);
        assert_eq!(continent_from_position(99.0, 99.0, ws), 0);
    }

    // --- aggregate_continent_stats tests ---

    #[test]
    fn aggregate_continent_stats_distributes_agents() {
        let cfg = AgentSimConfig {
            initial_population: 40,
            world_size: 100.0,
            ..AgentSimConfig::default()
        };
        let pop = seed_population(&cfg);
        let (counts, mean_res, mean_health, mean_innov) =
            aggregate_continent_stats(&pop, cfg.world_size);

        let total: u32 = counts.iter().sum();
        assert_eq!(total, pop.len() as u32);

        // Mean values should be non-negative
        for ci in 0..4 {
            assert!(mean_res[ci] >= 0.0);
            assert!(mean_health[ci] >= 0.0);
            assert!(mean_innov[ci] >= 0.0);
        }
    }

    // --- aggregate_continent_interactions tests ---

    #[test]
    fn aggregate_continent_interactions_proportional() {
        let cfg = AgentSimConfig {
            initial_population: 80,
            world_size: 100.0,
            ..AgentSimConfig::default()
        };
        let pop = seed_population(&cfg);
        let counters = EventCounters {
            cooperation_events: 100,
            conflict_events: 50,
            ..EventCounters::default()
        };
        let (coop, conf) = aggregate_continent_interactions(&pop, &counters, cfg.world_size);

        let total_coop: u32 = coop.iter().sum();
        let total_conf: u32 = conf.iter().sum();

        // Due to integer rounding, totals may be slightly less than input counts
        assert!(total_coop <= 100);
        assert!(total_conf <= 50);
        // But at least some should be distributed
        assert!(total_coop > 0);
        assert!(total_conf > 0);
    }

    // --- simulate_event_driven_with_observer tests ---

    #[test]
    fn event_observer_is_called_with_valid_frames() {
        let mut frames: Vec<EventMapFrame> = Vec::new();
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 40,
                world_size: 30.0,
                seed: 123,
                ..AgentSimConfig::default()
            },
            event: EventParams::default(),
            end_time: 20.0,
        };
        let result = simulate_event_driven_with_observer(cfg, |frame: &EventMapFrame| {
            frames.push(frame.clone());
        });

        assert!(
            !frames.is_empty(),
            "observer should be called at least once"
        );
        assert!(result.events_processed > 0);

        for frame in &frames {
            assert!(frame.time >= 0.0);
            assert!(frame.total_population > 0);
            assert!(frame.events_processed > 0);

            // Continent populations should sum to total
            let sum: u32 = frame.continent_populations.iter().sum();
            assert_eq!(sum, frame.total_population);

            // Mean values should be non-negative
            for ci in 0..4 {
                assert!(frame.continent_mean_resources[ci] >= 0.0);
                assert!(frame.continent_mean_health[ci] >= 0.0);
                assert!(frame.continent_mean_innovation[ci] >= 0.0);
            }

            // Emergent state should have valid bounds
            assert!((0.0..=1.0).contains(&frame.emergent.gini_coefficient));
            assert!((0.0..=1.0).contains(&frame.emergent.cooperation_rate));
        }
    }

    #[test]
    fn event_observer_frame_times_increase() {
        let mut times: Vec<f64> = Vec::new();
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 40,
                world_size: 25.0,
                seed: 456,
                ..AgentSimConfig::default()
            },
            event: EventParams::default(),
            end_time: 20.0,
        };
        let _ = simulate_event_driven_with_observer(cfg, |frame: &EventMapFrame| {
            times.push(frame.time);
        });

        assert!(times.len() >= 2, "should have multiple measurement frames");
        for window in times.windows(2) {
            assert!(
                window[1] >= window[0],
                "frame times should increase: {} >= {}",
                window[1],
                window[0]
            );
        }
    }

    #[test]
    fn event_observer_result_matches_direct_run() {
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 40,
                world_size: 25.0,
                seed: 789,
                ..AgentSimConfig::default()
            },
            event: EventParams::default(),
            end_time: 15.0,
        };

        let direct = simulate_event_driven(cfg.clone());
        let mut observer_count = 0_u32;
        let observer_result = simulate_event_driven_with_observer(cfg, |_frame: &EventMapFrame| {
            observer_count += 1;
        });

        // Same seed should produce the same number of snapshots
        assert_eq!(direct.snapshots.len(), observer_result.snapshots.len());
        assert_eq!(direct.events_processed, observer_result.events_processed);

        // Observer should be called once per measurement snapshot
        assert_eq!(observer_count, direct.snapshots.len() as u32);
    }

    #[test]
    fn event_observer_captures_inter_society_counters() {
        let mut has_nonzero_raid = false;
        let mut has_nonzero_migration = false;
        let cfg = EventSimConfig {
            agent: AgentSimConfig {
                initial_population: 80,
                world_size: 30.0,
                seed: 321,
                ..AgentSimConfig::default()
            },
            event: EventParams {
                raid_base_rate: 0.5,
                migrate_base_rate: 0.8,
                ..EventParams::default()
            },
            end_time: 50.0,
        };
        let _ = simulate_event_driven_with_observer(cfg, |frame: &EventMapFrame| {
            if frame.raid_events > 0 {
                has_nonzero_raid = true;
            }
            if frame.migration_events > 0 {
                has_nonzero_migration = true;
            }
        });

        // With elevated rates and enough time, at least one should fire
        assert!(
            has_nonzero_raid || has_nonzero_migration,
            "expected at least some raid or migration events in 50 time units"
        );
    }
}
