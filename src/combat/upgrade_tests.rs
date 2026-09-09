use super::*;

#[test]
fn basic_rounds_keep_launch_damage_when_configuration_changes() {
    let (mut app, _) = empty_app();
    let target = enemy(&mut app, START + Vec3::X * 180., 40);
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    step(&mut app, 0., &[]);
    assert_eq!(count::<Projectile>(&mut app), 1);
    app.world_mut().resource_mut::<CombatConfig>().shot_damage = 20;
    step(&mut app, 0.3, &[]);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().health, 30);
}

fn upgrade_app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(0.1),
        ))
        .add_plugins((
            bevy::time::TimePlugin,
            ArenaPlugin,
            CombatPlugin,
            crate::upgrades::runtime::UpgradePlugin,
        ));
    app.update();
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(60));
    app.world_mut().resource_mut::<WaveConfig>().bursts.clear();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    (app, drone)
}
fn tick(app: &mut App, dt: f32, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for &key in keys {
        input.press(key);
    }
    *app.world_mut()
        .resource_mut::<bevy::time::TimeUpdateStrategy>() =
        bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(dt));
    app.update();
}
fn offer(app: &mut App, xp: u32) {
    app.world_mut()
        .resource_mut::<crate::upgrades::UpgradeRun>()
        .award(xp);
    tick(app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    tick(app, 0., &[]); // release gate
}
fn pick(app: &mut App, kind: crate::upgrades::UpgradeKind) {
    offer(app, 50);
    app.world_mut()
        .resource_mut::<crate::upgrades::UpgradeRun>()
        .offer = vec![kind];
    tick(app, 0., &[KeyCode::Digit1]);
    tick(app, 0., &[]);
}
#[test]
fn xp_kills_and_pickup_are_credited_once_and_terminal_beats_choice() {
    let (mut app, drone) = upgrade_app();
    app.world_mut().resource_mut::<Encounter>().kills = 1;
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 10);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = Vec3::new(0., 90., -180.);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 40);
    app.world_mut().resource_mut::<Encounter>().kills = 3;
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert_eq!(
        app.world().resource::<crate::upgrades::UpgradeRun>().level,
        2
    );
}
#[test]
fn long_choice_pause_preserves_gameplay_and_skip_resumes_without_toggle() {
    let (mut app, drone) = upgrade_app();
    let enemy_id = enemy(&mut app, START + Vec3::X * 250., 30);
    let projectile = shot(&mut app, START + Vec3::Y * 150., Vec3::X, 5.);
    let warning = app
        .world_mut()
        .spawn((
            SpawnWarning { ready_at: 10. },
            Transform::from_xyz(-380., 90., 0.),
        ))
        .id();
    app.world_mut()
        .resource_mut::<PlayerHealth>()
        .invulnerable_until = 2.;
    app.world_mut().resource_mut::<Weapon>().ready_at = 1.;
    app.world_mut()
        .resource_mut::<crate::modules::Modules>()
        .enabled = [true; 4];
    offer(&mut app, 50);
    let now = app.world().resource::<Time>().elapsed_secs_f64();
    let before = position(&app, drone);
    let target = position(&app, enemy_id);
    let battery = app.world().resource::<crate::energy::Energy>().current;
    tick(&mut app, 40., &[KeyCode::Space, KeyCode::Digit4]);
    assert_eq!(app.world().resource::<Time>().elapsed_secs_f64(), now);
    assert_eq!(position(&app, drone), before);
    assert_eq!(position(&app, enemy_id), target);
    assert_eq!(
        app.world().get::<Projectile>(projectile).unwrap().remaining,
        5.
    );
    assert!(app.world().get_entity(warning).is_ok());
    assert_eq!(
        app.world().resource::<crate::energy::Energy>().current,
        battery
    );
    tick(&mut app, 10., &[KeyCode::Backspace]);
    assert_eq!(
        app.world()
            .resource::<crate::upgrades::UpgradeRun>()
            .pending,
        0
    );
    assert!(
        app.world()
            .resource::<crate::upgrades::UpgradeRun>()
            .selected
            .is_empty()
    );
    tick(&mut app, 0.1, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!(app.world().resource::<Time>().elapsed_secs_f64() < now + 0.11);
    assert!(
        app.world().resource::<PlayerHealth>().invulnerable_until
            > app.world().resource::<Time>().elapsed_secs_f64()
    );
    assert_eq!(
        app.world().resource::<crate::modules::Modules>().enabled,
        [true; 4]
    );
}
#[test]
fn applying_combined_choices_updates_hull_damage_and_restart_restores_baseline() {
    use crate::upgrades::{UpgradeKind::*, UpgradeRun};
    let (mut app, _) = upgrade_app();
    pick(&mut app, HeavyArmor);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 150);
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .acceleration_multiplier,
        0.75
    );
    offer(&mut app, 75);
    app.world_mut().resource_mut::<UpgradeRun>().offer = vec![HeavyRounds];
    tick(&mut app, 0., &[KeyCode::Digit1]);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 20);
    assert!(
        (app.world()
            .resource::<crate::arena::FlightConfig>()
            .acceleration_multiplier
            - 0.675)
            .abs()
            < 1e-6
    );
    assert_eq!(
        app.world().resource::<crate::modules::Modules>().enabled,
        [false; 4]
    );
    tick(&mut app, 0., &[KeyCode::KeyR, KeyCode::Digit1]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 10);
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .acceleration_multiplier,
        1.
    );
    assert_eq!(app.world().resource::<UpgradeRun>().level, 1);
    assert!(
        !app.world()
            .resource::<crate::upgrades::runtime::ExplorationPickup>()
            .collected
    );
}

#[test]
fn armor_and_rounds_reduce_horizontal_vertical_and_passive_braking_acceleration() {
    use crate::arena::{DroneFlight, FlightConfig};
    use crate::upgrades::UpgradeKind::*;
    for kind in [HeavyArmor, HeavyRounds] {
        let multiplier = if kind == HeavyArmor { 0.75 } else { 0.9 };
        for keys in [
            vec![KeyCode::KeyW, KeyCode::Space],
            vec![KeyCode::ShiftLeft],
            vec![],
        ] {
            let mut samples = Vec::new();
            for upgraded in [false, true] {
                let (mut app, drone) = upgrade_app();
                if upgraded {
                    pick(&mut app, kind);
                }
                app.world_mut()
                    .get_mut::<Transform>(drone)
                    .unwrap()
                    .translation = Vec3::new(0., 150., 0.);
                app.world_mut()
                    .get_mut::<DroneFlight>(drone)
                    .unwrap()
                    .velocity = Vec3::new(20., 20., 0.);
                let before = app.world().get::<DroneFlight>(drone).unwrap().velocity;
                tick(&mut app, 0.001, &keys);
                let flight = app.world().get::<DroneFlight>(drone).unwrap();
                samples.push(flight.velocity - before);
                assert_eq!(
                    app.world().resource::<FlightConfig>().yaw_rate,
                    FlightConfig::default().yaw_rate
                );
            }
            for axis in 0..3 {
                if samples[0][axis].abs() > 1e-5 {
                    assert!(
                        (samples[1][axis] / samples[0][axis] - multiplier).abs() < 0.003,
                        "{kind:?} {keys:?} {samples:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn queued_selection_requires_release_and_restart_wins_inside_modal() {
    use crate::upgrades::UpgradeRun;
    let (mut app, _) = upgrade_app();
    offer(&mut app, 140);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 1);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 1);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::KeyR, KeyCode::Digit1]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 0);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
}
#[test]
fn mouse_skip_uses_same_choice_and_release_rules() {
    use crate::upgrades::{ChoiceAction, UpgradeRun};
    let (mut app, _) = upgrade_app();
    offer(&mut app, 50);
    app.world_mut()
        .spawn((ChoiceAction::Skip, Interaction::Pressed));
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 0);
    tick(&mut app, 0.1, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
}

#[test]
fn module_and_capacity_choices_preserve_power_and_recharge_progress() {
    use crate::modules::{ModuleConfig, ModuleKind, Modules};
    use crate::upgrades::UpgradeKind::*;
    let (mut app, _) = upgrade_app();
    {
        let mut modules = app.world_mut().resource_mut::<Modules>();
        modules.enabled[ModuleKind::Shield as usize] = true;
        modules.shield.blocks = 0;
        modules.shield.remaining = 3.;
    }
    pick(&mut app, RapidShield);
    let modules = app.world().resource::<Modules>();
    assert!(modules.active(ModuleKind::Shield));
    assert_eq!(modules.shield.blocks, 0);
    assert_eq!(modules.shield.remaining, 1.5);
    assert_eq!(
        app.world()
            .resource::<ModuleConfig>()
            .drain(ModuleKind::Shield),
        12.
    );
    let (mut app, _) = upgrade_app();
    pick(&mut app, Interceptor);
    assert_eq!(app.world().resource::<crate::energy::Energy>().current, 75.);
    assert_eq!(
        app.world()
            .resource::<crate::energy::EnergyConfig>()
            .capacity,
        75.
    );
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .max_horizontal_speed,
        546.
    );
}
#[test]
fn rocket_snapshot_radius_and_cooldown_survive_a_wide_area_choice() {
    use crate::upgrades::UpgradeKind::*;
    let (mut app, _) = upgrade_app();
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    enemy(&mut app, START + Vec3::X * 300., 500);
    tick(&mut app, 0., &[KeyCode::Digit4]);
    let payload = *app
        .world_mut()
        .query_filtered::<&ShotPayload, With<rockets::Rocket>>()
        .single(app.world())
        .unwrap();
    let deadline = app.world().resource::<rockets::RocketLauncher>().ready_at;
    pick(&mut app, WideAreaRockets);
    assert_eq!(
        app.world()
            .resource::<crate::modules::ModuleConfig>()
            .rocket_radius,
        105.
    );
    let in_flight = app
        .world_mut()
        .query_filtered::<&ShotPayload, With<rockets::Rocket>>()
        .single(app.world())
        .unwrap();
    assert_eq!(in_flight.radius, payload.radius);
    assert_eq!(in_flight.radius, 70.);
    let now = app.world().resource::<Time>().elapsed_secs_f64();
    assert!(
        (app.world().resource::<rockets::RocketLauncher>().ready_at - now - (deadline - now) * 1.5)
            .abs()
            < 1e-6
    );
}
#[test]
fn actual_rocket_multikills_award_each_victim_once() {
    let (mut app, _) = upgrade_app();
    app.world_mut()
        .resource_mut::<CombatConfig>()
        .enemy_flight
        .max_horizontal_speed = 0.;
    enemy(&mut app, START + Vec3::X * 180., 20);
    enemy(&mut app, START + Vec3::new(180., 40., 0.), 20);
    tick(&mut app, 0., &[KeyCode::Digit4]);
    tick(&mut app, 0.4, &[]);
    assert_eq!(app.world().resource::<Encounter>().kills, 2);
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 20);
    tick(&mut app, 0.4, &[]);
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 20);
}
#[test]
fn fatal_contact_on_threshold_frame_suppresses_choice_and_restart_restores_pickup() {
    let (mut app, drone) = upgrade_app();
    app.world_mut().resource_mut::<PlayerHealth>().current = 10;
    app.world_mut()
        .resource_mut::<crate::upgrades::UpgradeRun>()
        .award(50);
    enemy(&mut app, START, 100);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    tick(&mut app, 0., &[KeyCode::KeyR]);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = Vec3::new(30., 90., -180.);
    tick(&mut app, 0., &[]);
    assert!(
        app.world()
            .resource::<crate::upgrades::runtime::ExplorationPickup>()
            .collected
    );
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert!(
        !app.world()
            .resource::<crate::upgrades::runtime::ExplorationPickup>()
            .collected
    );
}
