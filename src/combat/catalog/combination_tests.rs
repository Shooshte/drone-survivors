//! Opt-in, fixed-step evidence for the combined catalog scenario.
//!
//! Run with:
//! `cargo test --locked catalog_combination_probe -- --ignored --nocapture`
//!
//! Full rounds use only catalog controls and ordinary damage/resources. The
//! short card controls deliberately set battery, hull, or target distance and
//! say so in their output; they isolate rules and are not balance outcomes.

use super::*;
use crate::{
    arena::{Drone, DroneFlight, FlightConfig},
    economy::runtime::EnemyKind,
    energy::{ChargingNode, Energy, EnergyConfig},
    modules::{ModuleConfig, ModuleKind, Modules},
    upgrades::{UpgradeKind, UpgradeRun},
    world::{
        environment::{Environment, REPAIR_CENTER, REPAIR_RADIUS},
        hazard::HazardState,
        layout::CHARGER_X,
    },
};

const DT: f64 = 1. / 30.;
const ROUND_SECONDS: f64 = 60.;
const FRAME_LIMIT: usize = 30 * 60 + 64;
const ALL_KINDS: [EnemyKind; 7] = [
    EnemyKind::Chaser,
    EnemyKind::Fast,
    EnemyKind::Rammer,
    EnemyKind::Slower,
    EnemyKind::Jammer,
    EnemyKind::Bomber,
    EnemyKind::Mothership,
];
const ALL_MODULES: [ModuleKind; 6] = [
    ModuleKind::Overdrive,
    ModuleKind::Shield,
    ModuleKind::Mobility,
    ModuleKind::Rocket,
    ModuleKind::Repulsor,
    ModuleKind::Repair,
];

#[derive(Clone, Copy)]
struct Case {
    label: &'static str,
    modules: [Option<ModuleKind>; 4],
    cards: [Option<UpgradeKind>; 4],
    moving: bool,
}

const CASES: [Case; 14] = [
    Case {
        label: "empty-stationary",
        modules: [None; 4],
        cards: [None; 4],
        moving: false,
    },
    Case {
        label: "empty-route",
        modules: [None; 4],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "overdrive-control",
        modules: [Some(ModuleKind::Overdrive), None, None, None],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "overdrive-route",
        modules: [Some(ModuleKind::Overdrive), None, None, None],
        cards: [Some(UpgradeKind::HotOverdrive), None, None, None],
        moving: true,
    },
    Case {
        label: "rockets-control",
        modules: [Some(ModuleKind::Rocket), None, None, None],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "rockets-route",
        modules: [Some(ModuleKind::Rocket), None, None, None],
        cards: [Some(UpgradeKind::WideAreaRockets), None, None, None],
        moving: true,
    },
    Case {
        label: "repulsor-control",
        modules: [Some(ModuleKind::Repulsor), None, None, None],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "repulsor-route",
        modules: [Some(ModuleKind::Repulsor), None, None, None],
        cards: [Some(UpgradeKind::WideRepulsor), None, None, None],
        moving: true,
    },
    Case {
        label: "repair-control",
        modules: [Some(ModuleKind::Repair), None, None, None],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "repair-route",
        modules: [Some(ModuleKind::Repair), None, None, None],
        cards: [Some(UpgradeKind::RapidRepair), None, None, None],
        moving: true,
    },
    Case {
        label: "offense-control",
        modules: [
            Some(ModuleKind::Overdrive),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Mobility),
            Some(ModuleKind::Rocket),
        ],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "offense-four",
        modules: [
            Some(ModuleKind::Overdrive),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Mobility),
            Some(ModuleKind::Rocket),
        ],
        cards: [
            Some(UpgradeKind::Interceptor),
            Some(UpgradeKind::AgileFrame),
            Some(UpgradeKind::HeavyArmor),
            Some(UpgradeKind::HeavyRounds),
        ],
        moving: true,
    },
    Case {
        label: "support-control",
        modules: [
            Some(ModuleKind::Repulsor),
            Some(ModuleKind::Repair),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Mobility),
        ],
        cards: [None; 4],
        moving: true,
    },
    Case {
        label: "support-four",
        modules: [
            Some(ModuleKind::Repulsor),
            Some(ModuleKind::Repair),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Mobility),
        ],
        cards: [
            Some(UpgradeKind::RapidShield),
            Some(UpgradeKind::EfficientCoils),
            Some(UpgradeKind::ReserveBattery),
            Some(UpgradeKind::LongRangeRounds),
        ],
        moving: true,
    },
];

#[derive(Debug)]
struct ChoiceRecord {
    at: f64,
    offered: Vec<UpgradeKind>,
    selected: Option<UpgradeKind>,
    preview: bool,
    kills: u32,
    total_xp: u32,
    pickup_collected: bool,
}

#[derive(Debug)]
struct ResultRow {
    label: &'static str,
    phase: GamePhase,
    active: f64,
    hull: u32,
    kills: u32,
    power: f64,
    choices: Vec<ChoiceRecord>,
    observed: [bool; 7],
    powered: [f64; 6],
    activations: [u32; 4],
    sampled_field_seconds: [f64; 2],
    repair_crossings: u32,
    repaired: u32,
    charger_visits: u32,
    hazard_cycles: u64,
    route_waypoint: usize,
    resolved: u32,
}

#[derive(Debug)]
struct CardMetrics {
    speed: f32,
    horizontal_acceleration: f32,
    acceleration: f32,
    handling: f32,
    bank_handling: f32,
    tilt_response: f32,
    leveling_response: f32,
    hull: u32,
    shot_damage: u32,
    target_range: f32,
    fire_interval: f64,
    capacity: f64,
    activation: f64,
    drains: [f64; 4],
    overdrive_multiplier: f64,
    shield_recharge: f64,
    rocket_radius: f32,
    rocket_interval: f64,
    repair_rate: f64,
    repair_drain: f64,
    repulsor_radius: f32,
    repulsor_interval: f64,
    repulsor_drain: f64,
}

#[derive(Default)]
struct RoutePilot {
    waypoint: usize,
    dwell_until: Option<f64>,
}

impl RoutePilot {
    fn keys(&mut self, position: Vec3, flight: &DroneFlight, elapsed: f64) -> Vec<KeyCode> {
        // The west-side points traverse both directional fields and the repair
        // site. The z=410 bridge is the authored safe route around the divider
        // and timed center hazard to the east charger.
        let route = [
            Vec3::new(-420., 90., -190.),
            Vec3::new(-CHARGER_X, 90., 0.),
            Vec3::new(-CHARGER_X, 90., 410.),
            Vec3::new(240., 90., 410.),
            Vec3::new(CHARGER_X, 90., 0.),
            Vec3::new(240., 90., 410.),
            Vec3::new(-CHARGER_X, 90., 410.),
            REPAIR_CENTER,
            Vec3::new(-420., 90., 190.),
        ];
        let target = route[self.waypoint];
        if position.distance(target) < 35. && flight.velocity.length() < 50. {
            if matches!(self.waypoint, 1 | 4) {
                let until = *self.dwell_until.get_or_insert(elapsed + 1.5);
                if elapsed < until {
                    return Vec::new();
                }
            }
            self.dwell_until = None;
            self.waypoint = (self.waypoint + 1) % route.len();
        }
        super::super::validation::routes::keys(position, flight, route[self.waypoint])
    }
}

fn module_index(kind: ModuleKind) -> usize {
    ALL_MODULES
        .iter()
        .position(|candidate| *candidate == kind)
        .unwrap()
}

fn kind_index(kind: EnemyKind) -> usize {
    ALL_KINDS
        .iter()
        .position(|candidate| *candidate == kind)
        .unwrap()
}

fn tick(app: &mut App, keys: &[KeyCode]) {
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(DT));
    step(app, keys);
}

fn selector_shows_module(app: &mut App, slot: usize, module: Option<ModuleKind>) -> bool {
    let name = module.map_or("EMPTY", ModuleKind::name);
    let prefix = format!("{}   {name}", slot + 1);
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0.starts_with(&prefix))
}

fn set_modules(app: &mut App, wanted: [Option<ModuleKind>; 4]) {
    for (slot, wanted) in wanted.into_iter().enumerate() {
        for _ in 0..8 {
            if selector_shows_module(app, slot, wanted) {
                break;
            }
            click(app, catalog::CatalogAction::CycleSlot(slot));
        }
        assert!(
            selector_shows_module(app, slot, wanted),
            "slot {} never reached module {wanted:?}",
            slot + 1
        );
    }
}

fn selector_shows_card(app: &mut App, slot: usize, card: UpgradeKind) -> bool {
    let prefix = format!("{}   {}\n", slot + 1, card.name());
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0.starts_with(&prefix))
}

fn set_previews(app: &mut App, wanted: [Option<UpgradeKind>; 4]) {
    if wanted.iter().all(Option::is_none) {
        return;
    }
    click(app, catalog::CatalogAction::ToggleUpgrades);
    for (slot, wanted) in wanted.into_iter().enumerate() {
        let Some(wanted) = wanted else { continue };
        for _ in 0..14 {
            if selector_shows_card(app, slot, wanted) {
                break;
            }
            click(app, catalog::CatalogAction::CycleUpgrade(slot));
        }
        assert!(
            selector_shows_card(app, slot, wanted),
            "slot {} never offered eligible preview {wanted:?}",
            slot + 1
        );
    }
}

fn prerequisite(card: UpgradeKind) -> Option<ModuleKind> {
    match card {
        UpgradeKind::RapidShield => Some(ModuleKind::Shield),
        UpgradeKind::WideAreaRockets => Some(ModuleKind::Rocket),
        UpgradeKind::HotOverdrive => Some(ModuleKind::Overdrive),
        UpgradeKind::RapidRepair => Some(ModuleKind::Repair),
        UpgradeKind::WideRepulsor => Some(ModuleKind::Repulsor),
        UpgradeKind::EfficientCoils => Some(ModuleKind::Overdrive),
        _ => None,
    }
}

fn prepared_card_app(card: UpgradeKind, selected: bool) -> App {
    let mut app = app();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(DT));
    for _ in 0..14 {
        click(&mut app, catalog::CatalogAction::Next);
    }
    let modules = [prerequisite(card), None, None, None];
    set_modules(&mut app, modules);
    if selected {
        set_previews(&mut app, [Some(card), None, None, None]);
    }
    click(&mut app, catalog::CatalogAction::Launch);
    if selected {
        for _ in 0..8 {
            if *app.world().resource::<GamePhase>() == GamePhase::Playing {
                break;
            }
            tick(&mut app, &[]);
            if !app.world().resource::<UpgradeRun>().offer.is_empty() {
                tick(&mut app, &[KeyCode::Digit1]);
            }
        }
        assert_eq!(app.world().resource::<UpgradeRun>().selected, vec![card]);
    }
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<Modules>().loadout.slots(), &modules);
    app
}

fn card_metrics(app: &App) -> CardMetrics {
    let flight = app.world().resource::<FlightConfig>();
    let combat = app.world().resource::<CombatConfig>();
    let energy = app.world().resource::<EnergyConfig>();
    let modules = app.world().resource::<ModuleConfig>();
    CardMetrics {
        speed: flight.max_horizontal_speed,
        horizontal_acceleration: flight.horizontal_acceleration_multiplier,
        acceleration: flight.acceleration_multiplier,
        handling: flight.yaw_rate,
        bank_handling: flight.bank_yaw_rate,
        tilt_response: flight.tilt_rate,
        leveling_response: flight.leveling_rate,
        hull: combat.player_health,
        shot_damage: combat.shot_damage,
        target_range: combat.target_range,
        fire_interval: combat.fire_interval,
        capacity: energy.capacity,
        activation: energy.activation,
        drains: modules.drains,
        overdrive_multiplier: modules.overdrive_multiplier,
        shield_recharge: modules.shield_recharge,
        rocket_radius: modules.rocket_radius,
        rocket_interval: modules.rocket_interval,
        repair_rate: modules.repair_rate,
        repair_drain: modules.repair_drain,
        repulsor_radius: modules.repulsor_radius,
        repulsor_interval: modules.repulsor_interval,
        repulsor_drain: modules.repulsor_drain,
    }
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}

fn controlled_card_metrics() {
    for card in UpgradeKind::CATALOG {
        let control = card_metrics(&prepared_card_app(card, false));
        let upgraded = card_metrics(&prepared_card_app(card, true));
        println!(
            "CATALOG CARD CONTROL card={} control={control:?} upgraded={upgraded:?}",
            card.name()
        );
        match card {
            UpgradeKind::Interceptor => {
                close((upgraded.speed / control.speed) as f64, 1.3);
                close(
                    (upgraded.horizontal_acceleration / control.horizontal_acceleration) as f64,
                    1.3,
                );
                close(upgraded.capacity / control.capacity, 0.75);
            }
            UpgradeKind::AgileFrame => {
                close((upgraded.handling / control.handling) as f64, 1.4);
                close((upgraded.bank_handling / control.bank_handling) as f64, 1.4);
                close((upgraded.tilt_response / control.tilt_response) as f64, 1.4);
                close(
                    (upgraded.leveling_response / control.leveling_response) as f64,
                    1.4,
                );
                assert_eq!(upgraded.hull, control.hull - 20);
            }
            UpgradeKind::HeavyArmor => {
                assert_eq!(upgraded.hull, control.hull + 50);
                close((upgraded.acceleration / control.acceleration) as f64, 0.75);
            }
            UpgradeKind::HeavyRounds => {
                assert_eq!(upgraded.shot_damage, control.shot_damage * 2);
                close((upgraded.acceleration / control.acceleration) as f64, 0.9);
            }
            UpgradeKind::RapidShield => {
                close(upgraded.shield_recharge / control.shield_recharge, 0.5);
                close(
                    upgraded.drains[ModuleKind::Shield as usize]
                        / control.drains[ModuleKind::Shield as usize],
                    1.5,
                );
            }
            UpgradeKind::WideAreaRockets => {
                close((upgraded.rocket_radius / control.rocket_radius) as f64, 1.5);
                close(upgraded.rocket_interval / control.rocket_interval, 1.5);
            }
            UpgradeKind::EfficientCoils => {
                for index in 0..4 {
                    close(upgraded.drains[index] / control.drains[index], 0.75);
                }
                close(upgraded.repair_drain / control.repair_drain, 0.75);
                close(upgraded.repulsor_drain / control.repulsor_drain, 0.75);
                close(upgraded.activation / control.activation, 2.);
            }
            UpgradeKind::ReserveBattery => {
                close(upgraded.capacity / control.capacity, 1.5);
                close((upgraded.speed / control.speed) as f64, 0.8);
            }
            UpgradeKind::LongRangeRounds => {
                close((upgraded.target_range / control.target_range) as f64, 1.5);
                close(upgraded.fire_interval / control.fire_interval, 1.25);
            }
            UpgradeKind::HotOverdrive => {
                close(
                    upgraded.overdrive_multiplier / control.overdrive_multiplier,
                    1.5,
                );
                close(
                    upgraded.drains[ModuleKind::Overdrive as usize]
                        / control.drains[ModuleKind::Overdrive as usize],
                    1.5,
                );
            }
            UpgradeKind::RapidRepair => {
                close(upgraded.repair_rate / control.repair_rate, 2.);
                close(upgraded.repair_drain / control.repair_drain, 1.5);
            }
            UpgradeKind::WideRepulsor => {
                close(
                    (upgraded.repulsor_radius / control.repulsor_radius) as f64,
                    1.5,
                );
                close(upgraded.repulsor_interval / control.repulsor_interval, 1.5);
            }
        }
    }
}

fn spawn_target(app: &mut App, distance: f32) -> Entity {
    let player = app
        .world_mut()
        .query_filtered::<&Transform, With<Drone>>()
        .single(app.world())
        .unwrap()
        .translation;
    let position = player + Vec3::X * distance;
    app.world_mut()
        .spawn((
            Enemy {
                kind: EnemyKind::Chaser,
                health: 20,
                previous: position,
                path: Vec::new(),
            },
            Transform::from_translation(position),
            DroneFlight::default(),
        ))
        .id()
}

fn controlled_runtime_tradeoffs() {
    let mut coils_control = prepared_card_app(UpgradeKind::EfficientCoils, false);
    let mut coils = prepared_card_app(UpgradeKind::EfficientCoils, true);
    for app in [&mut coils_control, &mut coils] {
        app.world_mut().resource_mut::<Energy>().current = 15.;
        tick(app, &[KeyCode::Digit1]);
    }
    assert!(coils_control.world().resource::<Modules>().enabled[0]);
    assert!(!coils.world().resource::<Modules>().enabled[0]);
    println!("CATALOG RUNTIME coils energy=15 control=ON upgraded=REJECTED");

    let mut range_control = prepared_card_app(UpgradeKind::LongRangeRounds, false);
    let mut range = prepared_card_app(UpgradeKind::LongRangeRounds, true);
    spawn_target(&mut range_control, 500.);
    spawn_target(&mut range, 500.);
    tick(&mut range_control, &[]);
    tick(&mut range, &[]);
    assert_eq!(count::<Projectile>(&mut range_control), 0);
    assert_eq!(count::<Projectile>(&mut range), 1);
    println!(
        "CATALOG RUNTIME long-range target=500 control_shots=0 upgraded_shots=1 intervals=0.5/0.625"
    );

    let mut repair_control = prepared_card_app(UpgradeKind::RapidRepair, false);
    let mut repair = prepared_card_app(UpgradeKind::RapidRepair, true);
    for app in [&mut repair_control, &mut repair] {
        app.world_mut().resource_mut::<PlayerHealth>().current = 50;
        tick(app, &[KeyCode::Digit1]);
        for _ in 1..30 {
            tick(app, &[]);
        }
    }
    assert_eq!(
        repair_control.world().resource::<PlayerHealth>().current,
        56
    );
    assert_eq!(repair.world().resource::<PlayerHealth>().current, 62);
    close(repair_control.world().resource::<Energy>().current, 88.);
    close(repair.world().resource::<Energy>().current, 82.);
    println!(
        "CATALOG RUNTIME repair one_paid_second control=+6_hull/-12_power upgraded=+12_hull/-18_power"
    );

    let mut repulsor_control = prepared_card_app(UpgradeKind::WideRepulsor, false);
    let mut repulsor = prepared_card_app(UpgradeKind::WideRepulsor, true);
    let control_target = spawn_target(&mut repulsor_control, 220.);
    let upgraded_target = spawn_target(&mut repulsor, 220.);
    tick(&mut repulsor_control, &[KeyCode::Digit1]);
    tick(&mut repulsor, &[KeyCode::Digit1]);
    assert!(
        repulsor_control
            .world()
            .get::<DroneFlight>(control_target)
            .unwrap()
            .velocity
            .x
            < 0.
    );
    assert!(
        repulsor
            .world()
            .get::<DroneFlight>(upgraded_target)
            .unwrap()
            .velocity
            .x
            > 0.
    );
    close(
        repulsor_control
            .world()
            .resource::<bombs::BombState>()
            .pulse_cooldown,
        2.,
    );
    close(
        repulsor
            .world()
            .resource::<bombs::BombState>()
            .pulse_cooldown,
        3.,
    );
    println!(
        "CATALOG RUNTIME wide-repulsor target=220 control_push=false/2s upgraded_push=true/3s"
    );
}

fn module_keys(app: &App, case: Case, elapsed: f64) -> Vec<KeyCode> {
    let energy = app.world().resource::<Energy>().current;
    let modules = app.world().resource::<Modules>();
    let occupied = case.modules.iter().flatten().count();
    let scheduled = if occupied > 1 {
        Some((elapsed / 4.).floor() as usize % occupied)
    } else {
        Some(0)
    };
    let mut keys = Vec::new();
    for (slot, kind) in case.modules.iter().enumerate() {
        let Some(_) = kind else { continue };
        let want = scheduled == Some(slot)
            && if modules.enabled[slot] {
                energy > 6.
            } else {
                energy >= 20.
            };
        if modules.enabled[slot] != want && modules.disabled_for[slot] <= 0. {
            keys.push(crate::modules::SLOT_KEYS[slot]);
        }
    }
    keys
}

fn run_case(case: Case) -> ResultRow {
    let mut app = app();
    *app.world_mut().resource_mut::<TimeUpdateStrategy>() =
        TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(DT));
    for _ in 0..14 {
        click(&mut app, catalog::CatalogAction::Next);
    }
    set_modules(&mut app, case.modules);
    set_previews(&mut app, case.cards);
    click(&mut app, catalog::CatalogAction::Launch);

    assert!(app.world().resource::<Environment>().enabled());
    assert_eq!(app.world().resource::<WaveConfig>().duration, ROUND_SECONDS);
    assert_eq!(
        app.world().resource::<WaveConfig>().bursts,
        vec![(1., 7), (16., 7), (31., 7)]
    );

    let preview_count = case.cards.iter().flatten().count();
    let mut route = RoutePilot::default();
    let mut choices = Vec::new();
    let mut choice_released = false;
    let mut observed = [false; 7];
    let mut powered = [0.; 6];
    let mut activations = [0; 4];
    let mut enabled_before = [false; 4];
    let mut sampled_field_seconds = [0.; 2];
    let mut repair_crossings = 0;
    let mut repair_inside = false;
    let mut charger_visits = 0;
    let mut occupied_before: Vec<Entity> = Vec::new();

    for _ in 0..FRAME_LIMIT {
        let phase = *app.world().resource::<GamePhase>();
        if matches!(phase, GamePhase::Dead | GamePhase::Survived) {
            break;
        }
        let elapsed = app.world().resource::<Encounter>().elapsed;
        let mut keys = Vec::new();
        if phase == GamePhase::Choosing {
            let offer = app.world().resource::<UpgradeRun>().offer.clone();
            if offer.is_empty() {
                choice_released = false;
            } else if choice_released {
                let preview = choices.len() < preview_count;
                let selected = preview.then_some(offer[0]);
                choices.push(ChoiceRecord {
                    at: elapsed,
                    offered: offer,
                    selected,
                    preview,
                    kills: app.world().resource::<Encounter>().kills,
                    total_xp: app.world().resource::<UpgradeRun>().total_xp,
                    pickup_collected: app
                        .world()
                        .resource::<crate::upgrades::runtime::ExplorationPickup>()
                        .collected,
                });
                keys.push(if preview {
                    KeyCode::Digit1
                } else {
                    KeyCode::Backspace
                });
                choice_released = false;
            } else {
                choice_released = true;
            }
        } else {
            choice_released = false;
            if case.moving {
                let (position, flight) = app
                    .world_mut()
                    .query_filtered::<(&Transform, &DroneFlight), With<Drone>>()
                    .single(app.world())
                    .map(|(transform, flight)| (transform.translation, *flight))
                    .unwrap();
                keys.extend(route.keys(position, &flight, elapsed));
            }
            keys.extend(module_keys(&app, case, elapsed));
        }

        tick(&mut app, &keys);
        let active_delta = (app.world().resource::<Encounter>().elapsed - elapsed).max(0.);

        for enemy in app.world_mut().query::<&Enemy>().iter(app.world()) {
            observed[kind_index(enemy.kind)] = true;
        }
        let modules = app.world().resource::<Modules>();
        for (slot, kind) in case.modules.iter().enumerate() {
            if !enabled_before[slot] && modules.enabled[slot] {
                activations[slot] += 1;
            }
            enabled_before[slot] = modules.enabled[slot];
            if let Some(kind) = kind
                && modules.active(*kind)
            {
                powered[module_index(*kind)] += active_delta;
            }
        }

        let position = app
            .world_mut()
            .query_filtered::<&Transform, With<Drone>>()
            .single(app.world())
            .unwrap()
            .translation;
        let environment = app.world().resource::<Environment>();
        for (index, field) in environment.fields.iter().enumerate() {
            if field.bounds.overlaps(position, Vec3::ZERO) && field.active(environment.elapsed) {
                sampled_field_seconds[index] += active_delta;
            }
        }
        let inside = position.distance(REPAIR_CENTER) <= REPAIR_RADIUS;
        if inside && !repair_inside {
            repair_crossings += 1;
        }
        repair_inside = inside;

        let occupied: Vec<_> = app
            .world_mut()
            .query::<(Entity, &ChargingNode, &crate::energy::ChargerReserve)>()
            .iter(app.world())
            .filter_map(|(entity, _, reserve)| reserve.occupied.then_some(entity))
            .collect();
        charger_visits += occupied
            .iter()
            .filter(|entity| !occupied_before.contains(entity))
            .count() as u32;
        occupied_before = occupied;
    }

    let phase = *app.world().resource::<GamePhase>();
    let encounter = app.world().resource::<Encounter>();
    assert!(matches!(phase, GamePhase::Dead | GamePhase::Survived));
    assert!(encounter.elapsed <= ROUND_SECONDS + f64::EPSILON);
    assert!(
        observed.into_iter().all(|seen| seen),
        "{} missed a kind",
        case.label
    );
    let run = app.world().resource::<UpgradeRun>();
    let selected = &run.selected;
    let expected: Vec<_> = case.cards.iter().flatten().copied().collect();
    assert_eq!(&selected[..expected.len()], expected.as_slice());
    for kind in case.modules.iter().flatten() {
        assert!(
            powered[module_index(*kind)] > 0.,
            "{} never powered {kind:?}",
            case.label
        );
    }
    assert_eq!(choices.len(), run.resolved as usize);
    assert!(choices.len() <= crate::upgrades::OPPORTUNITY_LIMIT as usize);
    let resolved = run.resolved;
    let environment = app.world().resource::<Environment>();
    ResultRow {
        label: case.label,
        phase,
        active: encounter.elapsed,
        hull: app.world().resource::<PlayerHealth>().current,
        kills: encounter.kills,
        power: app.world().resource::<Energy>().current,
        choices,
        observed,
        powered,
        activations,
        sampled_field_seconds,
        repair_crossings,
        repaired: environment.repaired,
        charger_visits,
        hazard_cycles: app.world().resource::<HazardState>().cycle,
        route_waypoint: route.waypoint,
        resolved,
    }
}

#[test]
#[ignore = "explicit fourteen-run, 30 Hz catalog comparison"]
fn catalog_combination_probe() {
    for kind in ALL_MODULES {
        assert!(CASES.iter().any(|case| case.modules.contains(&Some(kind))));
    }
    for card in UpgradeKind::CATALOG {
        assert!(CASES.iter().any(|case| case.cards.contains(&Some(card))));
    }
    controlled_card_metrics();
    controlled_runtime_tradeoffs();

    let mut rows = Vec::new();
    for case in CASES {
        let row = run_case(case);
        let choice_summary: Vec<_> = row
            .choices
            .iter()
            .map(|choice| {
                format!(
                    "at={:.3}/preview={}/offered={:?}/selected={:?}/kills={}/xp={}/pickup={}",
                    choice.at,
                    choice.preview,
                    choice.offered,
                    choice.selected,
                    choice.kills,
                    choice.total_xp,
                    choice.pickup_collected,
                )
            })
            .collect();
        println!(
            "CATALOG COMBINATION case={} route={} outcome={:?} active={:.3} hull={} kills={} power={:.3} choices={:?} observed={:?} powered={:?} activations={:?} fields={:?} repair_crossings={} repaired={} charger_visits={} hazard_cycles={} route_waypoint={}",
            row.label,
            case.moving,
            row.phase,
            row.active,
            row.hull,
            row.kills,
            row.power,
            choice_summary,
            row.observed,
            row.powered,
            row.activations,
            row.sampled_field_seconds,
            row.repair_crossings,
            row.repaired,
            row.charger_visits,
            row.hazard_cycles,
            row.route_waypoint,
        );
        rows.push(row);
    }

    let stationary = rows
        .iter()
        .find(|row| row.label == "empty-stationary")
        .unwrap();
    assert_eq!(stationary.sampled_field_seconds, [0.; 2]);
    assert_eq!(stationary.repair_crossings, 0);
    assert_eq!(stationary.charger_visits, 0);
    let moving = &rows[1..];
    assert!(moving.iter().all(|row| row.sampled_field_seconds[0] > 0.));
    assert!(moving.iter().any(|row| row.sampled_field_seconds[1] > 0.));
    assert!(moving.iter().any(|row| row.repair_crossings > 0));
    assert!(moving.iter().any(|row| row.repaired > 0));
    assert!(moving.iter().any(|row| row.charger_visits > 0));

    // Preview XP is disclosed by the `preview` flag above. Non-preview entries
    // are gameplay-earned; the output records their kills, XP and pickup state.
    let natural_choices = rows
        .iter()
        .flat_map(|row| &row.choices)
        .filter(|choice| !choice.preview)
        .count();
    assert!(natural_choices > 0);
    assert!(rows.iter().all(|row| row.choices.len() <= 4));
    assert!(
        rows.iter()
            .all(|row| row.choices.len() == row.resolved as usize)
    );
    println!("CATALOG COMBINATION natural_choices={natural_choices}");

    for (control, upgraded) in [
        ("overdrive-control", "overdrive-route"),
        ("rockets-control", "rockets-route"),
        ("repulsor-control", "repulsor-route"),
        ("repair-control", "repair-route"),
        ("offense-control", "offense-four"),
        ("support-control", "support-four"),
    ] {
        let control = rows.iter().find(|row| row.label == control).unwrap();
        let upgraded = rows.iter().find(|row| row.label == upgraded).unwrap();
        println!(
            "CATALOG DELTA control={} upgraded={} active={:.3} hull={} kills={} power={:.3}",
            control.label,
            upgraded.label,
            upgraded.active - control.active,
            i64::from(upgraded.hull) - i64::from(control.hull),
            i64::from(upgraded.kills) - i64::from(control.kills),
            upgraded.power - control.power,
        );
    }
}
