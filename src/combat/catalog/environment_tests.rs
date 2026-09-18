use super::*;

#[test]
fn environment_scenario_is_selectable_and_repairs_only_one_charge() {
    let mut app = app();
    for _ in 0..12 {
        step(&mut app, &[KeyCode::ArrowRight]);
    }
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|t| t.0.contains("Environment practice"))
    );
    step(&mut app, &[KeyCode::Enter]);
    app.world_mut().resource_mut::<PlayerHealth>().current = 40;
    let mut transform = app
        .world_mut()
        .query_filtered::<&mut Transform, With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap();
    transform.translation = Vec3::new(-420., 90., 0.);
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    for _ in 0..60 {
        step(&mut app, &[]);
    }
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    step(&mut app, &[KeyCode::KeyR]);
    app.world_mut().resource_mut::<PlayerHealth>().current = 90;
    app.world_mut()
        .query_filtered::<&mut Transform, With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap()
        .translation = Vec3::new(-420., 90., 0.);
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
}

#[test]
fn environment_full_hull_preserves_charge_and_other_scenarios_have_no_repair() {
    let mut app = app();
    for _ in 0..12 {
        step(&mut app, &[KeyCode::ArrowRight]);
    }
    step(&mut app, &[KeyCode::Enter]);
    app.world_mut()
        .query_filtered::<&mut Transform, With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap()
        .translation = Vec3::new(-420., 90., 0.);
    step(&mut app, &[]);
    app.world_mut().resource_mut::<PlayerHealth>().current = 80;
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    step(&mut app, &[KeyCode::Tab]);
    step(&mut app, &[KeyCode::ArrowRight]);
    step(&mut app, &[KeyCode::ArrowRight]);
    step(&mut app, &[KeyCode::Enter]);
    app.world_mut().resource_mut::<PlayerHealth>().current = 80;
    app.world_mut()
        .query_filtered::<&mut Transform, With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap()
        .translation = Vec3::new(-420., 90., 0.);
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 80);
}

#[test]
fn environment_changes_travel_without_storing_a_velocity_modifier() {
    fn travel(position: Vec3, velocity: Vec3, environment: bool) -> (Vec3, Vec3) {
        let mut app = app();
        if environment {
            for _ in 0..12 {
                step(&mut app, &[KeyCode::ArrowRight]);
            }
        }
        step(&mut app, &[KeyCode::Enter]);
        let (mut t, mut f) = app
            .world_mut()
            .query_filtered::<(&mut Transform, &mut DroneFlight), With<Drone>>()
            .single_mut(app.world_mut())
            .unwrap();
        t.translation = position;
        f.velocity = velocity;
        step(&mut app, &[]);
        let (t, f) = app
            .world_mut()
            .query_filtered::<(&Transform, &DroneFlight), With<Drone>>()
            .single(app.world())
            .unwrap();
        (t.translation - position, f.velocity)
    }
    let at = Vec3::new(-420., 90., -190.);
    for direction in [Vec3::X, Vec3::NEG_X, Vec3::Z] {
        let plain = travel(at, direction * 100., false);
        let field = travel(at, direction * 100., true);
        let expected = if direction == Vec3::X {
            1.4
        } else if direction == Vec3::NEG_X {
            0.6
        } else {
            1.
        };
        assert!((field.0.length() / plain.0.length() - expected).abs() < 0.001);
        assert!(field.1.distance(plain.1) < 0.001);
    }
    let outside = Vec3::new(-180., 90., -190.);
    assert_eq!(
        travel(outside, Vec3::X * 100., true),
        travel(outside, Vec3::X * 100., false)
    );
}

fn environment_app() -> App {
    let mut app = app();
    for _ in 0..12 {
        step(&mut app, &[KeyCode::ArrowRight]);
    }
    step(&mut app, &[KeyCode::Enter]);
    app
}
fn place_at(app: &mut App, position: Vec3) {
    let (mut t, mut f) = app
        .world_mut()
        .query_filtered::<(&mut Transform, &mut DroneFlight), With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap();
    t.translation = position;
    *f = default();
}

#[test]
fn environment_pauses_with_upgrade_and_resets_on_return_restart_and_launch() {
    use crate::world::environment::Environment;
    let mut app = environment_app();
    step(&mut app, &[]);
    app.world_mut().resource_mut::<Environment>().elapsed = 6.5;
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    step(&mut app, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    let clock = app.world().resource::<Environment>().elapsed;
    app.world_mut().resource_mut::<PlayerHealth>().current = 40;
    place_at(&mut app, Vec3::new(-420., 90., 0.));
    for _ in 0..60 {
        step(&mut app, &[]);
    }
    assert_eq!(app.world().resource::<Environment>().elapsed, clock);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 40);
    assert!(app.world().resource::<Environment>().repair_ready);
    step(&mut app, &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<Environment>().elapsed, 0.);
    assert!(app.world().resource::<Environment>().repair_ready);
    step(&mut app, &[KeyCode::Tab]);
    assert!(!app.world().resource::<Environment>().enabled());
    step(&mut app, &[KeyCode::Enter]);
    assert_eq!(app.world().resource::<Environment>().elapsed, 0.);
    assert!(app.world().resource::<Environment>().repair_ready);
}

#[test]
fn environment_repair_sweeps_fast_crossings_and_clamps_to_upgraded_hull() {
    let mut app = environment_app();
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            0.25,
        )));
    app.world_mut()
        .resource_mut::<crate::arena::FlightConfig>()
        .max_horizontal_speed = 2000.;
    app.world_mut().resource_mut::<CombatConfig>().player_health = 150;
    app.world_mut().resource_mut::<PlayerHealth>().current = 130;
    place_at(&mut app, Vec3::new(-420., 90., -120.));
    app.world_mut()
        .query_filtered::<&mut DroneFlight, With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap()
        .velocity = Vec3::Z * 1000.;
    step(&mut app, &[]);
    let at = app
        .world_mut()
        .query_filtered::<&Transform, With<Drone>>()
        .single(app.world())
        .unwrap()
        .translation;
    assert!(at.z > 70., "{at:?}");
    assert_eq!(app.world().resource::<PlayerHealth>().current, 150);
}

#[test]
fn environment_repair_requires_sight_height_and_living_player() {
    let mut app = environment_app();
    app.world_mut().resource_mut::<PlayerHealth>().current = 40;
    place_at(&mut app, Vec3::new(-420., 200., 0.));
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 40);
    app.world_mut()
        .resource_mut::<crate::world::WorldGeometry>()
        .solids
        .push(crate::world::Solid {
            center: Vec3::new(-390., 150., 0.),
            half: Vec3::new(2., 150., 100.),
        });
    place_at(&mut app, Vec3::new(-350., 90., 0.));
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 40);
    app.world_mut()
        .resource_mut::<crate::world::WorldGeometry>()
        .solids
        .clear();
    place_at(&mut app, Vec3::new(-420., 90., 0.));
    app.world_mut().resource_mut::<PlayerHealth>().current = 25;
    app.world_mut().resource_mut::<bombs::BombState>().remaining = Some(0.001);
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 0);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert!(
        app.world()
            .resource::<crate::world::environment::Environment>()
            .repair_ready
    );
}

#[test]
fn environment_composes_with_powered_mobility_and_active_beam_without_mutating_tuning() {
    fn movement(field: bool, mobility: bool, beam: bool) -> (f32, Vec3) {
        let mut app = if field {
            environment_app()
        } else {
            let mut app = app();
            step(&mut app, &[KeyCode::Enter]);
            app
        };
        app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
        if mobility {
            let mut modules = app.world_mut().resource_mut::<Modules>();
            modules.loadout =
                crate::modules::Loadout::new([Some(ModuleKind::Mobility), None, None, None])
                    .unwrap();
            modules.enabled[0] = true;
        }
        let at = Vec3::new(-420., 90., -190.);
        place_at(&mut app, at);
        app.world_mut()
            .query_filtered::<&mut DroneFlight, With<Drone>>()
            .single_mut(app.world_mut())
            .unwrap()
            .velocity = Vec3::X * 400.;
        if beam {
            let mut attack = control::ControlAttack::default();
            attack.phase = control::ControlPhase::Active;
            let enemy_at = at + Vec3::Z * 200.;
            app.world_mut().spawn((
                Enemy {
                    kind: crate::economy::runtime::EnemyKind::Slower,
                    health: 20,
                    previous: enemy_at,
                    path: Vec::new(),
                },
                attack,
                Transform::from_translation(enemy_at),
                DroneFlight::default(),
            ));
        }
        let speed = app
            .world()
            .resource::<crate::arena::FlightConfig>()
            .max_horizontal_speed;
        step(&mut app, &[]);
        assert_eq!(
            app.world()
                .resource::<crate::arena::FlightConfig>()
                .max_horizontal_speed,
            speed
        );
        assert_eq!(
            app.world()
                .resource::<control::ControlEffects>()
                .movement_multiplier(),
            if beam { 0.6 } else { 1. }
        );
        assert_eq!(
            app.world()
                .resource::<Modules>()
                .active(ModuleKind::Mobility),
            mobility
        );
        let (t, f) = app
            .world_mut()
            .query_filtered::<(&Transform, &DroneFlight), With<Drone>>()
            .single(app.world())
            .unwrap();
        (t.translation.x - at.x, f.velocity)
    }
    for mobility in [false, true] {
        for beam in [false, true] {
            let plain = movement(false, mobility, beam);
            let field = movement(true, mobility, beam);
            assert!((field.0 / plain.0 - 1.4).abs() < 0.001);
            assert!(field.1.distance(plain.1) < 0.001);
        }
    }
}

#[test]
fn environment_is_not_installed_by_normal_combat() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, ArenaPlugin, CombatPlugin));
    assert!(
        !app.world()
            .contains_resource::<crate::world::environment::Environment>()
    );
}

#[test]
fn environment_full_hull_repair_discards_paid_module_credit_before_next_damage() {
    use crate::combat::support::RepairState;
    let mut app = environment_app();
    app.world_mut().resource_mut::<Modules>().loadout =
        crate::modules::Loadout::new([Some(ModuleKind::Repair), None, None, None]).unwrap();
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    app.world_mut().resource_mut::<PlayerHealth>().current = 70;
    place_at(&mut app, Vec3::new(-420., 90., -400.));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        0.1,
    )));
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 70);
    assert!((app.world().resource::<RepairState>().credit - 0.6).abs() < 1e-6);

    // Site collection runs after powered module repair in this same update.
    place_at(&mut app, crate::world::environment::REPAIR_CENTER);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        0.05,
    )));
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert!(
        !app.world()
            .resource::<crate::world::environment::Environment>()
            .repair_ready
    );
    assert_eq!(app.world().resource::<RepairState>().credit, 0.);

    app.world_mut().resource_mut::<PlayerHealth>().current = 80;
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 80);
    assert!((app.world().resource::<RepairState>().credit - 0.3).abs() < 1e-6);
}
