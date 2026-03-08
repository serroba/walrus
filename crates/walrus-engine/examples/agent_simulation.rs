use std::env;

use walrus_engine::agents::{
    simulate_agents, AgentSimConfig, CulturalParams, EnergyParams, InstitutionParams,
    InterSocietyParams, InteractionParams, LifecycleParams, MateSelectionParams, MovementParams,
};
use walrus_engine::event_sim::{simulate_event_driven, EventParams, EventSimConfig};

fn env_f32(key: &str, default: f32) -> f32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u16(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u32(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn build_config() -> AgentSimConfig {
    let d = AgentSimConfig::default();
    let di = d.interaction;
    let dl = d.lifecycle;
    let dm = d.movement;
    let dms = d.mate_selection;
    let de = d.energy;

    AgentSimConfig {
        seed: env_u64("SEED", d.seed),
        initial_population: env_u32("INITIAL_POP", d.initial_population),
        ticks: env_u32("TICKS", d.ticks),
        world_size: env_f32("WORLD_SIZE", d.world_size),
        interaction_radius: env_f32("INTERACTION_RADIUS", d.interaction_radius),
        energy: EnergyParams {
            biomass_base_eroei: env_f64("BIOMASS_BASE_EROEI", de.biomass_base_eroei),
            biomass_initial_stock: env_f64("BIOMASS_INITIAL_STOCK", de.biomass_initial_stock),
            biomass_flow_rate: env_f64("BIOMASS_FLOW_RATE", de.biomass_flow_rate),
            biomass_steepness: env_f64("BIOMASS_STEEPNESS", de.biomass_steepness),
            biomass_tech_threshold: env_f32("BIOMASS_TECH_THRESHOLD", de.biomass_tech_threshold),
            biomass_regen_rate: env_f64("BIOMASS_REGEN_RATE", de.biomass_regen_rate),
            agriculture_base_eroei: env_f64("AG_BASE_EROEI", de.agriculture_base_eroei),
            agriculture_initial_stock: env_f64("AG_INITIAL_STOCK", de.agriculture_initial_stock),
            agriculture_flow_rate: env_f64("AG_FLOW_RATE", de.agriculture_flow_rate),
            agriculture_steepness: env_f64("AG_STEEPNESS", de.agriculture_steepness),
            agriculture_tech_threshold: env_f32("AG_TECH_THRESHOLD", de.agriculture_tech_threshold),
            agriculture_fertility_prob: env_f64("AG_FERTILITY_PROB", de.agriculture_fertility_prob),
            fossil_base_eroei: env_f64("FOSSIL_BASE_EROEI", de.fossil_base_eroei),
            fossil_initial_stock: env_f64("FOSSIL_INITIAL_STOCK", de.fossil_initial_stock),
            fossil_flow_rate: env_f64("FOSSIL_FLOW_RATE", de.fossil_flow_rate),
            fossil_steepness: env_f64("FOSSIL_STEEPNESS", de.fossil_steepness),
            fossil_tech_threshold: env_f32("FOSSIL_TECH_THRESHOLD", de.fossil_tech_threshold),
            fossil_abundance: env_f64("FOSSIL_ABUNDANCE", de.fossil_abundance),
            renewable_base_eroei: env_f64("RENEW_BASE_EROEI", de.renewable_base_eroei),
            renewable_flow_rate: env_f64("RENEW_FLOW_RATE", de.renewable_flow_rate),
            renewable_tech_threshold: env_f32("RENEW_TECH_THRESHOLD", de.renewable_tech_threshold),
            harvest_per_agent: env_f64("HARVEST_PER_AGENT", de.harvest_per_agent),
        },
        max_age: env_u16("MAX_AGE", d.max_age),
        min_population: env_u32("MIN_POP", d.min_population),
        max_population: env_u32("MAX_POP", d.max_population),
        interaction: InteractionParams {
            coop_self_weight: env_f32("COOP_SELF_WEIGHT", di.coop_self_weight),
            coop_other_weight: env_f32("COOP_OTHER_WEIGHT", di.coop_other_weight),
            coop_kin_bonus: env_f32("COOP_KIN_BONUS", di.coop_kin_bonus),
            conflict_self_weight: env_f32("CONFLICT_SELF_WEIGHT", di.conflict_self_weight),
            conflict_other_weight: env_f32("CONFLICT_OTHER_WEIGHT", di.conflict_other_weight),
            conflict_stranger_bonus: env_f32("CONFLICT_STRANGER_BONUS", di.conflict_stranger_bonus),
            trade_complementary: env_f32("TRADE_COMPLEMENTARY", di.trade_complementary),
            trade_same_skill: env_f32("TRADE_SAME_SKILL", di.trade_same_skill),
            coop_resource_bonus: env_f32("COOP_RESOURCE_BONUS", di.coop_resource_bonus),
            coop_prestige_gain: env_f32("COOP_PRESTIGE_GAIN", di.coop_prestige_gain),
            conflict_win_resources: env_f32("CONFLICT_WIN_RESOURCES", di.conflict_win_resources),
            conflict_win_status: env_f32("CONFLICT_WIN_STATUS", di.conflict_win_status),
            conflict_lose_resources: env_f32("CONFLICT_LOSE_RESOURCES", di.conflict_lose_resources),
            conflict_lose_health: env_f32("CONFLICT_LOSE_HEALTH", di.conflict_lose_health),
            conflict_noise: env_f32("CONFLICT_NOISE", di.conflict_noise),
            trade_complementary_bonus: env_f32(
                "TRADE_COMPLEMENTARY_BONUS",
                di.trade_complementary_bonus,
            ),
            trade_same_bonus: env_f32("TRADE_SAME_BONUS", di.trade_same_bonus),
            max_health_loss_per_tick: env_f32(
                "MAX_HEALTH_LOSS_PER_TICK",
                di.max_health_loss_per_tick,
            ),
            delegation_status_gap: env_f32("DELEGATION_STATUS_GAP", di.delegation_status_gap),
            delegation_tax_rate: env_f32("DELEGATION_TAX_RATE", di.delegation_tax_rate),
            delegation_prestige_gain: env_f32(
                "DELEGATION_PRESTIGE_GAIN",
                di.delegation_prestige_gain,
            ),
            power_status_weight: env_f32("POWER_STATUS_WEIGHT", di.power_status_weight),
            power_skill_weight: env_f32("POWER_SKILL_WEIGHT", di.power_skill_weight),
            power_aggression_weight: env_f32("POWER_AGGRESSION_WEIGHT", di.power_aggression_weight),
            max_status: env_f32("MAX_STATUS", di.max_status),
            max_prestige: env_f32("MAX_PRESTIGE", di.max_prestige),
            subsistence_level: env_f32("SUBSISTENCE_LEVEL", di.subsistence_level),
            skill_practice_rate: env_f32("SKILL_PRACTICE_RATE", di.skill_practice_rate),
            trust_coop_weight: env_f32("TRUST_COOP_WEIGHT", di.trust_coop_weight),
            trust_memory_decay: env_f32("TRUST_MEMORY_DECAY", di.trust_memory_decay),
        },
        lifecycle: LifecycleParams {
            health_decay_base: env_f32("HEALTH_DECAY_BASE", dl.health_decay_base),
            health_decay_age_factor: env_f32("HEALTH_DECAY_AGE_FACTOR", dl.health_decay_age_factor),
            health_recovery_threshold: env_f32(
                "HEALTH_RECOVERY_THRESHOLD",
                dl.health_recovery_threshold,
            ),
            health_recovery_rate: env_f32("HEALTH_RECOVERY_RATE", dl.health_recovery_rate),
            death_health_threshold: env_f32("DEATH_HEALTH_THRESHOLD", dl.death_health_threshold),
            starvation_resource_threshold: env_f32(
                "STARVATION_RESOURCE_THRESHOLD",
                dl.starvation_resource_threshold,
            ),
            starvation_death_prob: env_f32("STARVATION_DEATH_PROB", dl.starvation_death_prob),
            female_peak_fertility: env_f32("FEMALE_PEAK_FERTILITY", dl.female_peak_fertility),
            female_fertility_peak_age: env_f32(
                "FEMALE_FERTILITY_PEAK_AGE",
                dl.female_fertility_peak_age,
            ),
            female_fertility_decline: env_f32(
                "FEMALE_FERTILITY_DECLINE",
                dl.female_fertility_decline,
            ),
            male_peak_fertility: env_f32("MALE_PEAK_FERTILITY", dl.male_peak_fertility),
            male_fertility_peak_age: env_f32("MALE_FERTILITY_PEAK_AGE", dl.male_fertility_peak_age),
            male_fertility_decline: env_f32("MALE_FERTILITY_DECLINE", dl.male_fertility_decline),
            min_fertility: env_f32("MIN_FERTILITY", dl.min_fertility),
            min_reproduction_age: env_u16("MIN_REPRO_AGE", dl.min_reproduction_age),
            max_reproduction_age: env_u16("MAX_REPRO_AGE", dl.max_reproduction_age),
            reproduction_resource_threshold: env_f32(
                "REPRO_RESOURCE_THRESHOLD",
                dl.reproduction_resource_threshold,
            ),
            birth_rate: env_f32("BIRTH_RATE", dl.birth_rate),
            birth_resource_cost: env_f32("BIRTH_RESOURCE_COST", dl.birth_resource_cost),
            birth_health_cost: env_f32("BIRTH_HEALTH_COST", dl.birth_health_cost),
            skill_maternal_inherit_prob: env_f64(
                "SKILL_MATERNAL_INHERIT_PROB",
                dl.skill_maternal_inherit_prob,
            ),
            skill_mutation_prob: env_f64("SKILL_MUTATION_PROB", dl.skill_mutation_prob),
            trait_mutation_magnitude: env_f32(
                "TRAIT_MUTATION_MAGNITUDE",
                dl.trait_mutation_magnitude,
            ),
            norm_mutation_prob: env_f64("NORM_MUTATION_PROB", dl.norm_mutation_prob),
            newborn_health: env_f32("NEWBORN_HEALTH", dl.newborn_health),
            newborn_skill_level: env_f32("NEWBORN_SKILL_LEVEL", dl.newborn_skill_level),
            newborn_status: env_f32("NEWBORN_STATUS", dl.newborn_status),
            newborn_resources: env_f32("NEWBORN_RESOURCES", dl.newborn_resources),
            birth_spawn_radius: env_f32("BIRTH_SPAWN_RADIUS", dl.birth_spawn_radius),
            agents_per_kin_group: env_u32("AGENTS_PER_KIN_GROUP", dl.agents_per_kin_group),
            innovation_growth_rate: env_f32("INNOVATION_GROWTH_RATE", dl.innovation_growth_rate),
        },
        movement: MovementParams {
            kin_pull_strength: env_f32("KIN_PULL_STRENGTH", dm.kin_pull_strength),
            drift_with_kin: env_f32("DRIFT_WITH_KIN", dm.drift_with_kin),
            drift_alone: env_f32("DRIFT_ALONE", dm.drift_alone),
        },
        mate_selection: MateSelectionParams {
            status_weight: env_f32("MATE_STATUS_WEIGHT", dms.status_weight),
            resource_weight: env_f32("MATE_RESOURCE_WEIGHT", dms.resource_weight),
            prestige_weight: env_f32("MATE_PRESTIGE_WEIGHT", dms.prestige_weight),
            noise_weight: env_f32("MATE_NOISE_WEIGHT", dms.noise_weight),
        },
        institution: InstitutionParams {
            public_goods_rate: env_f32("PUBLIC_GOODS_RATE", d.institution.public_goods_rate),
            public_goods_bonus: env_f32("PUBLIC_GOODS_BONUS", d.institution.public_goods_bonus),
            defense_bonus: env_f32("DEFENSE_BONUS", d.institution.defense_bonus),
            leadership_threshold: env_f32(
                "LEADERSHIP_THRESHOLD",
                d.institution.leadership_threshold,
            ),
            patron_inheritance: env::var("PATRON_INHERITANCE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.institution.patron_inheritance),
        },
        inter_society: InterSocietyParams {
            min_raid_warriors: env_u32("MIN_RAID_WARRIORS", d.inter_society.min_raid_warriors),
            raid_aggression_threshold: env_f32(
                "RAID_AGGRESSION_THRESHOLD",
                d.inter_society.raid_aggression_threshold,
            ),
            raid_loot_per_warrior: env_f32(
                "RAID_LOOT_PER_WARRIOR",
                d.inter_society.raid_loot_per_warrior,
            ),
            raid_damage_per_warrior: env_f32(
                "RAID_DAMAGE_PER_WARRIOR",
                d.inter_society.raid_damage_per_warrior,
            ),
            conquest_power_ratio: env_f32(
                "CONQUEST_POWER_RATIO",
                d.inter_society.conquest_power_ratio,
            ),
            tribute_rate: env_f32("TRIBUTE_RATE", d.inter_society.tribute_rate),
            tribute_duration: env_u32("TRIBUTE_DURATION", d.inter_society.tribute_duration),
            migration_resource_threshold: env_f32(
                "MIGRATION_RESOURCE_THRESHOLD",
                d.inter_society.migration_resource_threshold,
            ),
            migration_probability: env_f32(
                "MIGRATION_PROBABILITY",
                d.inter_society.migration_probability,
            ),
            raid_range: env_f32("RAID_RANGE", d.inter_society.raid_range),
        },
        cultural: CulturalParams {
            vertical_mutation_prob: env_f64(
                "CULTURAL_VERTICAL_MUTATION",
                d.cultural.vertical_mutation_prob,
            ),
            cultural_mutation_magnitude: env_f32(
                "CULTURAL_MUTATION_MAG",
                d.cultural.cultural_mutation_magnitude,
            ),
            horizontal_adoption_prob: env_f32(
                "CULTURAL_HORIZONTAL_PROB",
                d.cultural.horizontal_adoption_prob,
            ),
            oblique_adoption_prob: env_f32(
                "CULTURAL_OBLIQUE_PROB",
                d.cultural.oblique_adoption_prob,
            ),
            oblique_prestige_gap: env_f32("CULTURAL_OBLIQUE_GAP", d.cultural.oblique_prestige_gap),
            authority_delegation_bonus: env_f32(
                "AUTHORITY_DELEGATION_BONUS",
                d.cultural.authority_delegation_bonus,
            ),
            trust_trade_bonus: env_f32("TRUST_TRADE_BONUS", d.cultural.trust_trade_bonus),
            sharing_coop_bonus: env_f32("SHARING_COOP_BONUS", d.cultural.sharing_coop_bonus),
            coercion_conflict_bonus: env_f32(
                "COERCION_CONFLICT_BONUS",
                d.cultural.coercion_conflict_bonus,
            ),
        },
    }
}

fn kinship_name(code: u8) -> &'static str {
    match code {
        0 => "patrilineal",
        1 => "matrilineal",
        _ => "bilateral",
    }
}

fn marriage_name(code: u8) -> &'static str {
    match code {
        0 => "monogamy",
        1 => "polygyny",
        _ => "polyandry",
    }
}

fn energy_type_name(code: u8) -> &'static str {
    match code {
        0 => "biomass",
        1 => "agriculture",
        2 => "fossil",
        3 => "renewable",
        _ => "unknown",
    }
}

fn institution_type_name(code: u8) -> &'static str {
    match code {
        0 => "band",
        1 => "tribe",
        2 => "chiefdom",
        3 => "state",
        _ => "unknown",
    }
}

fn print_csv_header() {
    println!("time,pop,mean_resources,gini,skill_entropy,hierarchy_depth,leaders,mean_group_size,kin_groups,coop_rate,conflict_rate,prestige,health,innovation,dominant_energy,energy_per_capita,mean_eroei,biomass_depletion,fossil_depletion,coercion_rate,property_norms,institution,public_goods,patrons,recognized_leaders,patron_tenure,raids,conquests,tribute_flows,migrations,societies,inter_group_trade_rate,active_tributes,authority_norm,sharing_norm,property_norm_cultural,trust_outgroup,cultural_diversity,kinship,marriage,coercion_tolerance,techniques,coordination_failure_index,mean_trust");
}

fn print_emergent_row(time: f64, e: &walrus_engine::agents::EmergentState) {
    println!(
        "{:.2},{},{:.4},{:.4},{:.4},{},{},{:.2},{},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.4},{:.2},{:.4},{:.4},{:.4},{:.4},{},{:.4},{},{},{:.1},{},{},{:.4},{},{},{:.4},{},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{:.4},{:.2},{:.4},{:.4}",
        time,
        e.population_size,
        e.mean_resources,
        e.gini_coefficient,
        e.skill_entropy,
        e.max_hierarchy_depth,
        e.num_leaders,
        e.mean_group_size,
        e.num_kin_groups,
        e.cooperation_rate,
        e.conflict_rate,
        e.mean_prestige,
        e.mean_health,
        e.mean_innovation,
        energy_type_name(e.dominant_energy),
        e.energy_per_capita,
        e.mean_eroei,
        e.biomass_depletion,
        e.fossil_depletion,
        e.coercion_rate,
        e.property_norm_strength,
        institution_type_name(e.institutional_type),
        e.public_goods_investment,
        e.patron_count,
        e.recognized_leaders,
        e.mean_patron_tenure,
        e.raid_events,
        e.conquest_events,
        e.tribute_flows,
        e.migration_events,
        e.num_active_societies,
        e.inter_group_trade_rate,
        e.active_tributes,
        e.mean_authority_norm,
        e.mean_sharing_norm,
        e.mean_property_norm,
        e.mean_trust_outgroup,
        e.cultural_diversity,
        kinship_name(e.dominant_kinship),
        marriage_name(e.dominant_marriage),
        e.mean_coercion_tolerance,
        e.technique_count,
        e.coordination_failure_index,
        e.mean_trust,
    );
}

fn main() {
    let cfg = build_config();
    let event_driven = env::var("EVENT_DRIVEN")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    if event_driven {
        let end_time = env_f64("END_TIME", cfg.ticks as f64);
        let event_cfg = EventSimConfig {
            agent: cfg,
            event: EventParams {
                forage_base_rate: env_f64("FORAGE_RATE", 1.0),
                interact_base_rate: env_f64("INTERACT_RATE", 1.5),
                move_base_rate: env_f64("MOVE_RATE", 1.0),
                transmit_base_rate: env_f64("TRANSMIT_RATE", 0.3),
                reproduce_base_rate: env_f64("REPRODUCE_RATE", 0.5),
                age_base_rate: env_f64("AGE_RATE", 1.0),
                learn_base_rate: env_f64("LEARN_RATE", 1.0),
                raid_base_rate: env_f64("RAID_RATE", 0.2),
                migrate_base_rate: env_f64("MIGRATE_RATE", 0.3),
                tribute_interval: env_f64("TRIBUTE_INTERVAL", 1.0),
                spatial_rebuild_interval: env_f64("SPATIAL_REBUILD_INTERVAL", 1.0),
                measure_interval: env_f64("MEASURE_INTERVAL", 1.0),
                landscape_update_interval: env_f64("LANDSCAPE_UPDATE_INTERVAL", 1.0),
            },
            end_time,
        };

        eprintln!("Event-Driven Agent Simulation");
        eprintln!(
            "  pop={} end_time={:.1} world={} radius={}",
            event_cfg.agent.initial_population,
            event_cfg.end_time,
            event_cfg.agent.world_size,
            event_cfg.agent.interaction_radius
        );

        let result = simulate_event_driven(event_cfg);

        print_csv_header();
        for snap in &result.snapshots {
            print_emergent_row(snap.time, &snap.emergent);
        }

        let snaps = result.snapshots.len();
        let final_pop = result.final_population.len();
        eprintln!(
            "{snaps} snapshots, {events} events processed, final pop = {final_pop}",
            events = result.events_processed,
        );
    } else {
        eprintln!("Agent Simulation (Phase 5: Cultural Transmission)");
        eprintln!(
            "  pop={} ticks={} world={} radius={}",
            cfg.initial_population, cfg.ticks, cfg.world_size, cfg.interaction_radius
        );
        eprintln!(
            "  energy: biomass_eroei={} ag_eroei={} fossil_eroei={} renew_eroei={}",
            cfg.energy.biomass_base_eroei,
            cfg.energy.agriculture_base_eroei,
            cfg.energy.fossil_base_eroei,
            cfg.energy.renewable_base_eroei
        );
        eprintln!(
            "  tech thresholds: biomass={} ag={} fossil={} renew={}",
            cfg.energy.biomass_tech_threshold,
            cfg.energy.agriculture_tech_threshold,
            cfg.energy.fossil_tech_threshold,
            cfg.energy.renewable_tech_threshold
        );
        eprintln!(
            "  innovation_growth_rate={}",
            cfg.lifecycle.innovation_growth_rate
        );

        let result = simulate_agents(cfg);

        print_csv_header();
        for snap in &result.snapshots {
            print_emergent_row(f64::from(snap.tick), &snap.emergent);
        }

        let ticks_run = result.snapshots.len();
        let final_pop = result.final_population.len();
        let final_innov = result
            .snapshots
            .last()
            .map(|s| s.emergent.mean_innovation)
            .unwrap_or(0.0);
        eprintln!(
            "{ticks_run} ticks completed, final pop = {final_pop}, final innovation = {final_innov:.4}"
        );
        eprintln!(
            "  biomass depletion: {:.4}, fossil depletion: {:.4}",
            result
                .final_landscape
                .mean_depletion(walrus_engine::agents::EnergyType::Biomass),
            result
                .final_landscape
                .mean_depletion(walrus_engine::agents::EnergyType::Fossil),
        );
    }
}
