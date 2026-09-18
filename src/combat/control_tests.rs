use super::*;
use crate::{
    arena::{ArenaPlugin, Drone, DroneFlight},
    combat::{CombatConfig, CombatPlugin, Enemy, WaveConfig},
    energy::Energy,
    game::GamePhase,
    modules::{ModuleConfig, ModuleKind, Modules},
};
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, CombatPlugin));
    app.update();
    app.world_mut()
        .resource_mut::<WaveConfig>()
        .disable_authored_waves();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    app
}
fn spawn(app: &mut App, kind: EnemyKind) -> Entity {
    let position = Vec3::new(-240., 90., 0.);
    app.world_mut()
        .spawn((
            Enemy {
                kind,
                health: 40,
                previous: position,
                path: vec![],
            },
            Transform::from_translation(position),
            DroneFlight::default(),
            ControlAttack::default(),
        ))
        .id()
}
fn step(app: &mut App, dt: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(dt));
    app.update();
}
fn run(app: &mut App, seconds: f32, hz: u32) {
    for _ in 0..(seconds * hz as f32).round() as u32 {
        step(app, 1. / hz as f32);
    }
}
fn wall(app: &mut App) {
    app.insert_resource(crate::world::WorldGeometry {
        solids: vec![crate::world::Solid {
            center: Vec3::new(-120., 90., 0.),
            half: Vec3::new(10., 150., 500.),
        }],
        hazard: None,
    });
}

#[test]
fn control_beam_warns_slows_without_stacking_then_recovers_across_frame_rates() {
    for hz in [30, 60, 144] {
        let mut app = app();
        let id = spawn(&mut app, EnemyKind::Slower);
        spawn(&mut app, EnemyKind::Slower);
        run(&mut app, 1., hz);
        assert_eq!(
            app.world().get::<ControlAttack>(id).unwrap().phase,
            ControlPhase::Windup
        );
        assert_eq!(
            app.world()
                .resource::<ControlEffects>()
                .movement_multiplier(),
            1.
        );
        run(&mut app, 0.4, hz);
        assert_eq!(
            app.world()
                .resource::<ControlEffects>()
                .movement_multiplier(),
            0.6
        );
        run(&mut app, 2., hz);
        assert_eq!(
            app.world().get::<ControlAttack>(id).unwrap().phase,
            ControlPhase::Recovery
        );
        assert_eq!(
            app.world()
                .resource::<ControlEffects>()
                .movement_multiplier(),
            1.
        );
    }
}

#[test]
fn control_breaking_sight_or_range_cancels_warning_and_active_beam() {
    for active in [false, true] {
        for cover in [false, true] {
            let mut app = app();
            let id = spawn(&mut app, EnemyKind::Slower);
            run(&mut app, if active { 1.5 } else { 0.5 }, 60);
            if cover {
                wall(&mut app);
            } else {
                app.world_mut()
                    .query_filtered::<&mut Transform, With<Drone>>()
                    .single_mut(app.world_mut())
                    .unwrap()
                    .translation
                    .z = 500.;
            }
            step(&mut app, 1. / 60.);
            assert_eq!(
                app.world().get::<ControlAttack>(id).unwrap().phase,
                ControlPhase::Recovery
            );
            assert!(!app.world().resource::<ControlEffects>().slowed);
        }
    }
}

#[test]
fn control_jammer_keeps_warned_slot_and_forces_manual_restart_without_active_drain() {
    let mut app = app();
    let id = spawn(&mut app, EnemyKind::Jammer);
    app.world_mut().resource_mut::<Modules>().enabled[1] = true;
    step(&mut app, 1. / 60.);
    assert_eq!(app.world().get::<ControlAttack>(id).unwrap().slot, Some(1));
    // Turning the target off does not redirect the attack to a different slot.
    app.world_mut().resource_mut::<Modules>().enabled = [true, false, false, false];
    run(&mut app, 1.3, 60);
    let modules = app.world().resource::<Modules>();
    assert!(modules.disabled_for[1] > 2.8);
    assert_eq!(modules.disabled_for[0], 0.);
    app.world_mut().resource_mut::<Modules>().enabled[0] = false;
    let before = app.world().resource::<Energy>().current;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Digit2);
    run(&mut app, 0.5, 60);
    assert_eq!(app.world().resource::<Energy>().current, before);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut().despawn(id);
    run(&mut app, 3., 60);
    assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
    assert!(!app.world().resource::<Modules>().active(ModuleKind::Shield));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Digit2);
    step(&mut app, 1. / 60.);
    assert!(app.world().resource::<Modules>().active(ModuleKind::Shield));
}

#[test]
fn control_overlapping_jammers_cannot_chain_or_extend_a_lock() {
    let mut app = app();
    spawn(&mut app, EnemyKind::Jammer);
    spawn(&mut app, EnemyKind::Jammer);
    app.world_mut().resource_mut::<Modules>().enabled = [true; 4];
    run(&mut app, 1.4, 60);
    let modules = app.world().resource::<Modules>();
    assert_eq!(modules.disabled_for.iter().filter(|&&t| t > 0.).count(), 1);
    assert!(modules.disabled_for[0] < 3.);
    run(&mut app, 3., 60);
    assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
    assert!(app.world().resource::<Modules>().jam_grace > 0.);
    run(&mut app, 1.5, 60);
    assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
}

#[test]
fn control_pause_freezes_attacks_and_lock_reset_clears_all() {
    let mut app = app();
    let id = spawn(&mut app, EnemyKind::Slower);
    run(&mut app, 1.5, 60);
    app.world_mut().resource_mut::<Modules>().jam(2);
    let elapsed = app.world().get::<ControlAttack>(id).unwrap().elapsed;
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
    run(&mut app, 5., 60);
    assert_eq!(
        app.world().get::<ControlAttack>(id).unwrap().elapsed,
        elapsed
    );
    assert_eq!(app.world().resource::<Modules>().disabled_for[2], 3.);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    step(&mut app, 1. / 60.);
    assert!(app.world().get_entity(id).is_err());
    assert!(!app.world().resource::<ControlEffects>().slowed);
    assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
    assert_eq!(app.world().resource::<Modules>().jam_grace, 0.);
}

#[test]
fn control_dead_sources_and_empty_loadouts_do_not_apply_effects() {
    let mut app = app();
    let id = spawn(&mut app, EnemyKind::Slower);
    run(&mut app, 1.5, 60);
    app.world_mut().get_mut::<Enemy>(id).unwrap().health = 0;
    step(&mut app, 1. / 60.);
    assert!(!app.world().resource::<ControlEffects>().slowed);
    app.world_mut().despawn(id);
    spawn(&mut app, EnemyKind::Jammer);
    *app.world_mut().resource_mut::<Modules>() = Modules::new(
        crate::modules::Loadout::new([None; 4]).unwrap(),
        &ModuleConfig::default(),
    );
    run(&mut app, 2., 60);
    assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
}

#[test]
fn control_hitch_cannot_skip_a_new_warning() {
    let mut app = app();
    let id = spawn(&mut app, EnemyKind::Jammer);
    step(&mut app, 5.);
    assert_eq!(
        app.world().get::<ControlAttack>(id).unwrap().phase,
        ControlPhase::Windup
    );
    assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
}

#[test]
fn control_slow_clamps_existing_speed_and_jammed_mobility_cannot_boost_it() {
    let mut app = app();
    spawn(&mut app, EnemyKind::Slower);
    run(&mut app, 1.5, 60);
    app.world_mut().resource_mut::<Modules>().enabled[2] = true;
    assert!(app.world_mut().resource_mut::<Modules>().jam(2));
    {
        let mut flight = app
            .world_mut()
            .query_filtered::<&mut DroneFlight, With<Drone>>()
            .single_mut(app.world_mut())
            .unwrap();
        flight.velocity = Vec3::new(420., 0., 0.);
    }
    let baseline = *app.world().resource::<crate::arena::FlightConfig>();
    step(&mut app, 1. / 60.);
    let speed = app
        .world_mut()
        .query_filtered::<&DroneFlight, With<Drone>>()
        .single(app.world())
        .unwrap()
        .velocity
        .with_y(0.)
        .length();
    assert!(
        speed <= baseline.max_horizontal_speed * 0.6 + 0.01,
        "{speed}"
    );
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .max_horizontal_speed,
        baseline.max_horizontal_speed
    );
}

#[test]
fn control_wall_prevents_jam_and_killing_windup_source_prevents_delivery() {
    for cover in [false, true] {
        let mut app = app();
        let id = spawn(&mut app, EnemyKind::Jammer);
        run(&mut app, 0.5, 60);
        if cover {
            wall(&mut app);
        } else {
            app.world_mut().despawn(id);
        }
        run(&mut app, 1., 60);
        assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
    }
}
