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
    upgrade_app_with_pool(crate::upgrades::UpgradePool::default())
}

fn upgrade_app_with_pool(pool: crate::upgrades::UpgradePool) -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(pool)
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
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 4);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = Vec3::new(0., 90., -180.);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 34);
    app.world_mut().resource_mut::<Encounter>().kills = 5;
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
            SpawnWarning {
                ready_at: 10.,
                ..default()
            },
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
    offer(&mut app, 150);
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
                if axis == 1 && keys.is_empty() {
                    // Ideal altitude hold arrests vertical drift equally for every build.
                    assert_eq!(samples[0][axis], -20.);
                    assert_eq!(samples[1][axis], -20.);
                } else if samples[0][axis].abs() > 1e-5 {
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
    offer(&mut app, 215);
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
fn queued_mouse_choice_requires_release_then_a_fresh_click() {
    use crate::upgrades::{ChoiceAction, UpgradeRun};
    let (mut app, _) = upgrade_app();
    offer(&mut app, 200);
    let button = app
        .world_mut()
        .spawn((ChoiceAction::Skip, Interaction::Pressed))
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 1);
    // A held button, including a changed interaction on the next card, is ignored.
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    tick(&mut app, 20., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 1);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Hovered;
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 1);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 1);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 2);
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 0);
}

#[test]
fn four_queued_skips_complete_without_gameplay_or_held_input_leaks_and_reset_budget() {
    use crate::upgrades::UpgradeRun;
    let (mut app, drone) = upgrade_app();
    offer(&mut app, 1_000);
    let before = position(&app, drone);
    let elapsed = app.world().resource::<Encounter>().elapsed;
    for resolved in 1..=4 {
        tick(
            &mut app,
            10.,
            &[KeyCode::Backspace, KeyCode::Digit4, KeyCode::Space],
        );
        assert_eq!(app.world().resource::<UpgradeRun>().resolved, resolved);
        assert_eq!(position(&app, drone), before);
        assert_eq!(app.world().resource::<Encounter>().elapsed, elapsed);
        assert!(!app.world().resource::<crate::modules::Modules>().enabled[3]);
        if resolved < 4 {
            tick(&mut app, 10., &[KeyCode::Backspace]);
            assert_eq!(app.world().resource::<UpgradeRun>().resolved, resolved);
            tick(&mut app, 0., &[]);
        }
    }
    assert!(app.world().resource::<UpgradeRun>().exhausted);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    app.world_mut().resource_mut::<UpgradeRun>().award(u32::MAX);
    tick(&mut app, 0., &[]);
    assert!(app.world().resource::<UpgradeRun>().offer.is_empty());
    tick(&mut app, 0., &[KeyCode::KeyR]);
    let run = app.world().resource::<UpgradeRun>();
    assert_eq!((run.resolved, run.total_xp, run.pending), (0, 0, 0));
    assert_eq!(run.remaining(), 4);
    assert!(!run.exhausted);
    offer(&mut app, 50);
    let mut fresh = UpgradeRun::default();
    fresh.award(50);
    fresh.prepare_offer(&crate::modules::Loadout::default());
    assert_eq!(app.world().resource::<UpgradeRun>().offer, fresh.offer);
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
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 8);
    tick(&mut app, 0.4, &[]);
    assert_eq!(app.world().resource::<crate::upgrades::UpgradeRun>().xp, 8);
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

#[test]
fn wide_area_choice_retimes_remaining_cooldown_before_first_resumed_frame() {
    use crate::upgrades::{UpgradeKind, UpgradeRun};
    let (mut app, _) = upgrade_app();
    enemy(&mut app, START + Vec3::X * 300., 500);
    tick(&mut app, 0., &[KeyCode::Digit4]);
    let now = app.world().resource::<Time>().elapsed_secs_f64();
    app.world_mut()
        .resource_mut::<rockets::RocketLauncher>()
        .ready_at = now + 0.08;
    offer(&mut app, 50);
    app.world_mut().resource_mut::<UpgradeRun>().offer = vec![UpgradeKind::WideAreaRockets];
    tick(&mut app, 0., &[KeyCode::Digit1]);
    let before = count::<rockets::Rocket>(&mut app);
    tick(&mut app, 0.1, &[]);
    assert_eq!(
        count::<rockets::Rocket>(&mut app),
        before,
        "0.08s remaining should become 0.12s before advancing the resumed 0.1s frame"
    );
}

#[test]
fn hazard_kills_award_xp_once_and_can_open_upgrade_choices() {
    use crate::upgrades::UpgradeRun;
    use crate::world::{
        Solid, WorldGeometry,
        hazard::{HazardPhase, HazardState},
    };
    let (mut app, _) = upgrade_app();
    let center = START + Vec3::X * 200.;
    app.insert_resource(WorldGeometry {
        solids: vec![],
        hazard: Some(Solid {
            center,
            half: Vec3::splat(40.),
        }),
    });
    app.world_mut().resource_mut::<HazardState>().phase = HazardPhase::Active;
    app.world_mut().resource_mut::<UpgradeRun>().award(46);
    let target = enemy(&mut app, center, 10);
    tick(&mut app, 0.1, &[]);
    assert!(app.world().get_entity(target).is_err());
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    assert_eq!(app.world().resource::<UpgradeRun>().level, 2);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 0);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    tick(&mut app, 0.1, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 0);
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
}

#[test]
fn upgrade_choice_freezes_hazard_then_resumes_remaining_warning() {
    use crate::world::{
        Solid, WorldGeometry,
        hazard::{HazardPhase, HazardState},
    };
    let (mut app, _) = upgrade_app();
    app.insert_resource(WorldGeometry {
        solids: vec![],
        hazard: Some(Solid {
            center: START,
            half: Vec3::splat(60.),
        }),
    });
    {
        let mut state = app.world_mut().resource_mut::<HazardState>();
        state.phase = HazardPhase::Warning;
        state.elapsed = 0.5;
    }
    offer(&mut app, 50);
    tick(&mut app, 40., &[]);
    let state = app.world().resource::<HazardState>();
    assert_eq!(state.phase, HazardPhase::Warning);
    assert_eq!(state.elapsed, 0.5);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    tick(&mut app, 10., &[KeyCode::Backspace]);
    tick(&mut app, 0.1, &[]);
    assert_eq!(
        app.world().resource::<HazardState>().phase,
        HazardPhase::Warning
    );
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    tick(&mut app, 0.5, &[]);
    assert_eq!(
        app.world().resource::<HazardState>().phase,
        HazardPhase::Active
    );
    assert_eq!(app.world().resource::<PlayerHealth>().current, 90);
}

#[test]
fn reduced_kill_xp_is_credited_once_across_phases_and_restart() {
    use crate::upgrades::UpgradeRun;
    let (mut app, _) = upgrade_app();
    app.world_mut().resource_mut::<Encounter>().elapsed = 104.9;
    app.world_mut().resource_mut::<Encounter>().kills = 1;
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 4);
    app.world_mut().resource_mut::<Encounter>().elapsed = 105.;
    app.world_mut().resource_mut::<Encounter>().kills = 2;
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 8);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 8);
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 0);
    app.world_mut().resource_mut::<Encounter>().kills = 1;
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().xp, 4);
}

#[test]
fn agile_frame_improves_coordinated_turns_and_restart_restores_them() {
    use crate::arena::DroneFlight;
    use crate::upgrades::UpgradeKind;
    let mut turns = Vec::new();
    for upgraded in [false, true] {
        let (mut app, drone) = upgrade_app();
        if upgraded {
            pick(&mut app, UpgradeKind::AgileFrame);
        }
        tick(&mut app, 0.3, &[KeyCode::KeyE]);
        let flight = app.world().get::<DroneFlight>(drone).unwrap();
        let nose = Quat::from_rotation_y(flight.heading) * Vec3::NEG_Z;
        turns.push(nose.x.asin());
        tick(&mut app, 0., &[KeyCode::KeyR]);
        tick(&mut app, 0.3, &[KeyCode::KeyE]);
        let flight = app.world().get::<DroneFlight>(drone).unwrap();
        let nose = Quat::from_rotation_y(flight.heading) * Vec3::NEG_Z;
        assert!(
            (nose.x.asin() - turns[0]).abs() < 0.001,
            "restart restores baseline"
        );
    }
    assert!(turns[1] > turns[0] * 1.35, "turn benefit: {turns:?}");
}

#[test]
fn plugin_preserves_a_preinstalled_catalog_pool() {
    let pool = crate::upgrades::UpgradePool {
        catalog: true,
        preview: vec![crate::upgrades::UpgradeKind::ReserveBattery],
    };
    let (app, _) = upgrade_app_with_pool(pool.clone());
    let installed = app.world().resource::<crate::upgrades::UpgradePool>();
    assert_eq!(installed.catalog, pool.catalog);
    assert_eq!(installed.preview, pool.preview);
}

#[test]
fn catalog_reset_queues_sanitized_preview_as_normal_one_card_choices() {
    use crate::modules::{Loadout, ModuleKind, Modules};
    use crate::upgrades::{UpgradeKind, UpgradePool, UpgradeRun};
    let pool = UpgradePool {
        catalog: true,
        preview: vec![
            UpgradeKind::RapidRepair,
            UpgradeKind::RapidRepair,
            UpgradeKind::HotOverdrive,
            UpgradeKind::WideRepulsor,
            UpgradeKind::ReserveBattery,
            UpgradeKind::LongRangeRounds,
        ],
    };
    let (mut app, _) = upgrade_app_with_pool(pool);
    app.world_mut().resource_mut::<Modules>().loadout = Loadout::new([
        Some(ModuleKind::Repair),
        Some(ModuleKind::Overdrive),
        None,
        None,
    ])
    .unwrap();

    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(
        (
            app.world().resource::<UpgradeRun>().pending,
            app.world().resource::<UpgradeRun>().total_xp
        ),
        (4, 1_000)
    );
    tick(&mut app, 0., &[]);
    assert_eq!(
        app.world().resource::<UpgradeRun>().offer,
        vec![UpgradeKind::RapidRepair]
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    assert_eq!(
        app.world().resource::<UpgradeRun>().offer,
        vec![UpgradeKind::HotOverdrive]
    );
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 1);
}

#[test]
fn catalog_reset_opens_preview_before_any_gameplay_frame_can_advance() {
    use crate::energy::Energy;
    use crate::modules::Modules;
    use crate::upgrades::{UpgradeKind, UpgradePool, UpgradeRun};
    let pool = UpgradePool {
        catalog: true,
        preview: vec![UpgradeKind::ReserveBattery],
    };
    let (mut app, drone) = upgrade_app_with_pool(pool);
    app.world_mut().resource_mut::<Encounter>().elapsed = 17.;
    app.world_mut().resource_mut::<Energy>().current = 5.;
    app.world_mut().resource_mut::<Modules>().enabled = [true; 4];
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = START + Vec3::X * 80.;
    shot(&mut app, START, Vec3::X, 5.);

    tick(&mut app, 1., &[KeyCode::KeyR, KeyCode::Digit1]);

    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    assert!(app.world().resource::<Time<Virtual>>().is_paused());
    assert_eq!(
        app.world().resource::<UpgradeRun>().offer,
        vec![UpgradeKind::ReserveBattery]
    );
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    assert_eq!(app.world().resource::<Energy>().current, 100.);
    assert_eq!(position(&app, drone), START);
    assert_eq!(count::<Projectile>(&mut app), 0);
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);

    tick(&mut app, 30., &[]);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    assert_eq!(app.world().resource::<Energy>().current, 100.);
    assert_eq!(position(&app, drone), START);
    assert_eq!(count::<Projectile>(&mut app), 0);
}

#[test]
fn efficient_coils_changes_every_drain_and_enforces_the_higher_activation_threshold() {
    use crate::energy::{Energy, EnergyConfig};
    use crate::modules::{ModuleConfig, ModuleKind, Modules};
    use crate::upgrades::UpgradeKind;
    let (mut app, _) = upgrade_app();
    pick(&mut app, UpgradeKind::EfficientCoils);

    let tuning = app.world().resource::<ModuleConfig>();
    assert_eq!(tuning.drains, [7.5, 6., 6., 7.5]);
    assert_eq!(tuning.repair_drain, 9.);
    assert_eq!(tuning.repulsor_drain, 6.);
    assert_eq!(app.world().resource::<EnergyConfig>().activation, 20.);

    app.world_mut().resource_mut::<Energy>().current = 15.;
    tick(&mut app, 0., &[KeyCode::Digit1]);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    app.world_mut().resource_mut::<Energy>().current = 20.;
    tick(&mut app, 0., &[KeyCode::Digit1]);
    assert!(
        app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(
        app.world().resource::<ModuleConfig>().drains,
        ModuleConfig::default().drains
    );
    assert_eq!(
        app.world().resource::<EnergyConfig>().activation,
        EnergyConfig::default().activation
    );
}

#[test]
fn reserve_battery_increases_capacity_without_refilling_and_reduces_horizontal_speed() {
    use crate::energy::{Energy, EnergyConfig};
    use crate::upgrades::UpgradeKind;
    let (mut app, _) = upgrade_app();
    app.world_mut().resource_mut::<Energy>().current = 40.;
    pick(&mut app, UpgradeKind::ReserveBattery);

    assert_eq!(app.world().resource::<EnergyConfig>().capacity, 150.);
    assert_eq!(app.world().resource::<Energy>().current, 40.);
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .max_horizontal_speed,
        336.
    );
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(
        app.world().resource::<EnergyConfig>().capacity,
        EnergyConfig::default().capacity
    );
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .max_horizontal_speed,
        crate::arena::FlightConfig::default().max_horizontal_speed
    );
}

#[test]
fn long_range_rounds_retimes_basic_fire_progress_on_the_first_resumed_frame() {
    use crate::upgrades::UpgradeKind;
    let (mut app, _) = upgrade_app();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 400.;
    enemy(&mut app, START + Vec3::X * 500., 500);
    tick(&mut app, 0., &[]);
    let now = app.world().resource::<Time>().elapsed_secs_f64();
    app.world_mut().resource_mut::<Weapon>().ready_at = now + 0.4;
    let before = app.world().resource::<Weapon>().ready_at - now;
    pick(&mut app, UpgradeKind::LongRangeRounds);

    assert_eq!(app.world().resource::<CombatConfig>().target_range, 600.);
    assert_eq!(app.world().resource::<CombatConfig>().fire_interval, 0.625);
    assert_eq!(app.world().resource::<Weapon>().interval, Some(0.625));
    assert!((app.world().resource::<Weapon>().ready_at - now - before * 1.25).abs() < 1e-6);
    assert_eq!(count::<Projectile>(&mut app), 0);
    tick(&mut app, 0.5, &[]);
    assert_eq!(count::<Projectile>(&mut app), 1);
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(
        app.world().resource::<CombatConfig>().target_range,
        CombatConfig::default().target_range
    );
    assert_eq!(
        app.world().resource::<CombatConfig>().fire_interval,
        CombatConfig::default().fire_interval
    );
}

#[test]
fn powered_catalog_upgrades_compose_from_baseline_and_apply_once() {
    use crate::modules::{Loadout, ModuleConfig, ModuleKind, Modules};
    use crate::upgrades::{UpgradeKind, UpgradeRun};
    let (mut app, _) = upgrade_app();
    app.world_mut().resource_mut::<Modules>().loadout = Loadout::new([
        Some(ModuleKind::Overdrive),
        Some(ModuleKind::Repair),
        Some(ModuleKind::Repulsor),
        None,
    ])
    .unwrap();

    pick(&mut app, UpgradeKind::HotOverdrive);
    offer(&mut app, 150);
    app.world_mut().resource_mut::<UpgradeRun>().offer = vec![UpgradeKind::RapidRepair];
    tick(&mut app, 0., &[KeyCode::Digit1]);
    tick(&mut app, 0., &[]);
    offer(&mut app, 300);
    app.world_mut().resource_mut::<UpgradeRun>().offer = vec![UpgradeKind::WideRepulsor];
    tick(&mut app, 0., &[KeyCode::Digit1]);
    tick(&mut app, 0., &[]);
    let tuning = app.world().resource::<ModuleConfig>();
    assert_eq!(tuning.overdrive_multiplier, 3.);
    assert_eq!(tuning.drain(ModuleKind::Overdrive), 15.);
    assert_eq!(tuning.repair_rate, 12.);
    assert_eq!(tuning.repair_drain, 18.);
    assert_eq!(tuning.repulsor_radius, 270.);
    assert_eq!(tuning.repulsor_interval, 3.);

    offer(&mut app, 500);
    app.world_mut().resource_mut::<UpgradeRun>().offer = vec![UpgradeKind::HotOverdrive];
    tick(&mut app, 0., &[KeyCode::Digit1]);
    tick(&mut app, 0., &[]);
    let tuning = app.world().resource::<ModuleConfig>();
    assert_eq!(tuning.overdrive_multiplier, 3.);
    assert_eq!(tuning.drain(ModuleKind::Overdrive), 15.);
    tick(&mut app, 0., &[KeyCode::KeyR]);
    let tuning = app.world().resource::<ModuleConfig>();
    let baseline = ModuleConfig::default();
    assert_eq!(tuning.overdrive_multiplier, baseline.overdrive_multiplier);
    assert_eq!(tuning.drains, baseline.drains);
    assert_eq!(tuning.repair_rate, baseline.repair_rate);
    assert_eq!(tuning.repair_drain, baseline.repair_drain);
    assert_eq!(tuning.repulsor_radius, baseline.repulsor_radius);
    assert_eq!(tuning.repulsor_interval, baseline.repulsor_interval);
}

#[test]
fn rapid_repair_uses_the_paid_power_interval_with_composed_drain() {
    use crate::energy::Energy;
    use crate::modules::{Loadout, ModuleConfig, ModuleKind, Modules};
    use crate::upgrades::{UpgradeKind, UpgradeRun};
    let (mut app, _) = upgrade_app();
    app.world_mut().resource_mut::<Modules>().loadout =
        Loadout::new([Some(ModuleKind::Repair), None, None, None]).unwrap();
    pick(&mut app, UpgradeKind::EfficientCoils);
    offer(&mut app, 150);
    app.world_mut().resource_mut::<UpgradeRun>().offer = vec![UpgradeKind::RapidRepair];
    tick(&mut app, 0., &[KeyCode::Digit1]);
    tick(&mut app, 0., &[]);
    app.world_mut().resource_mut::<PlayerHealth>().current = 50;
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    app.world_mut().resource_mut::<Energy>().current = 100.;

    tick(&mut app, 1., &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 62);
    assert!((app.world().resource::<Energy>().current - 86.5).abs() < 1e-6);
    assert_eq!(app.world().resource::<ModuleConfig>().repair_drain, 13.5);
}

#[test]
fn wide_repulsor_preserves_pulse_cooldown_fraction_and_restart_clears_effects() {
    use crate::arena::DroneFlight;
    use crate::combat::bombs::BombState;
    use crate::energy::EnergyConfig;
    use crate::modules::{Loadout, ModuleConfig, ModuleKind, Modules};
    use crate::upgrades::{UpgradeKind, UpgradeRun};
    let (mut app, _) = upgrade_app();
    app.world_mut().resource_mut::<Modules>().loadout =
        Loadout::new([Some(ModuleKind::Repulsor), None, None, None]).unwrap();
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    app.world_mut().resource_mut::<BombState>().pulse_cooldown = 1.;
    pick(&mut app, UpgradeKind::WideRepulsor);

    assert_eq!(app.world().resource::<ModuleConfig>().repulsor_interval, 3.);
    assert_eq!(app.world().resource::<BombState>().pulse_cooldown, 1.5);
    app.world_mut().resource_mut::<BombState>().pulse_cooldown = 0.;
    let reached = enemy(&mut app, START + Vec3::X * 240., 500);
    tick(&mut app, 0., &[]);
    assert_ne!(
        app.world().get::<DroneFlight>(reached).unwrap().velocity,
        Vec3::ZERO
    );
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<ModuleConfig>().repulsor_interval, 2.);
    assert_eq!(app.world().resource::<ModuleConfig>().repulsor_radius, 180.);
    assert_eq!(app.world().resource::<EnergyConfig>().activation, 10.);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
    assert_eq!(app.world().resource::<BombState>().pulse_cooldown, 0.);
}
