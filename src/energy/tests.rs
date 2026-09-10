use super::*;
use crate::arena::ArenaPlugin;
use crate::modules::{ModuleKind, Modules};
use std::time::Duration;

pub(crate) fn app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, EnergyPlugin))
        .add_systems(
            Update,
            (prepare, update).chain().in_set(GameplaySet::Combat),
        );
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    (app, drone)
}
pub(crate) fn step(app: &mut App, dt: f64, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for key in keys {
        input.press(*key);
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(dt));
    app.update();
}
pub(crate) fn at(app: &mut App, drone: Entity, point: Vec3) {
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = point;
}
fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-5, "{actual} != {expected}");
}

fn charger_states(app: &mut App) -> Vec<(Entity, Vec3, ChargerReserve)> {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &ChargingNode, &ChargerReserve)>();
    let mut states = query
        .iter(world)
        .map(|(entity, node, reserve)| (entity, node.center, *reserve))
        .collect::<Vec<_>>();
    states.sort_by_key(|(entity, _, _)| entity.to_bits());
    states
}

fn set_reserve(app: &mut App, entity: Entity, remaining: f64, away_seconds: f64) {
    let mut reserve = app.world_mut().get_mut::<ChargerReserve>(entity).unwrap();
    reserve.remaining = remaining;
    reserve.away_seconds = away_seconds;
}

#[test]
fn stationary_overdrive_exhausts_left_charger_and_battery() {
    let (mut app, drone) = app();
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 0., &[KeyCode::Digit1]);

    step(&mut app, 31., &[]);

    near(app.world().resource::<Energy>().current, 0.);
    assert!(app.world().resource::<Energy>().charging.is_none());
    let left = charger_states(&mut app)
        .into_iter()
        .find(|(_, center, _)| center.x < 0.)
        .unwrap()
        .2;
    near(left.remaining, 0.);
    assert!(left.occupied);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
}

#[test]
fn charger_debits_only_energy_delivered_and_keeps_reserves_independent() {
    let (mut app, drone) = app();
    let initial = charger_states(&mut app);
    let left = initial
        .iter()
        .find(|(_, center, _)| center.x < 0.)
        .unwrap()
        .0;
    let right = initial
        .iter()
        .find(|(_, center, _)| center.x > 0.)
        .unwrap()
        .0;
    at(&mut app, drone, Vec3::new(-280., 90., 0.));

    step(&mut app, 5., &[]);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        200.,
    );

    step(&mut app, 0., &[KeyCode::Digit1]);
    step(&mut app, 5., &[]);
    near(app.world().resource::<Energy>().current, 100.);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        150.,
    );
    near(
        app.world().get::<ChargerReserve>(right).unwrap().remaining,
        200.,
    );

    app.world_mut().resource_mut::<Energy>().current = 0.;
    step(&mut app, 0., &[KeyCode::Digit1]);
    step(&mut app, 4., &[]);
    near(app.world().resource::<Energy>().current, 100.);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        50.,
    );
}

#[test]
fn overlapping_chargers_share_delivery_and_handoff_when_the_first_empties() {
    let (mut app, drone) = app();
    let _duplicate = app
        .world_mut()
        .spawn(ChargingNode {
            center: Vec3::new(-280., 0., 0.),
            radius: 90.,
            height: 160.,
        })
        .id();
    let mut occupied = charger_states(&mut app)
        .into_iter()
        .filter(|(_, center, _)| center.x < 0.)
        .map(|(entity, _, _)| entity)
        .collect::<Vec<_>>();
    occupied.sort_by_key(|entity| entity.to_bits());
    let [first, second] = occupied.as_slice() else {
        panic!("expected exactly two overlapping chargers");
    };
    set_reserve(&mut app, *first, 10., 0.);
    set_reserve(&mut app, *second, 200., 0.);
    app.world_mut().resource_mut::<Energy>().current = 0.;
    at(&mut app, drone, Vec3::new(-280., 90., 0.));

    step(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<Energy>().charging, Some(*first));
    step(&mut app, 1., &[]);

    near(app.world().resource::<Energy>().current, 25.);
    near(
        app.world().get::<ChargerReserve>(*first).unwrap().remaining,
        0.,
    );
    near(
        app.world()
            .get::<ChargerReserve>(*second)
            .unwrap()
            .remaining,
        185.,
    );
    assert_eq!(app.world().resource::<Energy>().charging, Some(*second));
}

#[test]
fn occupied_fields_do_not_recover_and_reentry_restarts_the_delay() {
    let (mut app, drone) = app();
    let duplicate = app
        .world_mut()
        .spawn(ChargingNode {
            center: Vec3::new(-280., 0., 0.),
            radius: 90.,
            height: 160.,
        })
        .id();
    let left = charger_states(&mut app)
        .into_iter()
        .find(|(entity, center, _)| center.x < 0. && *entity != duplicate)
        .unwrap()
        .0;
    for entity in [left, duplicate] {
        set_reserve(&mut app, entity, 0., 20.);
    }
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 3., &[]);
    for entity in [left, duplicate] {
        let reserve = app.world().get::<ChargerReserve>(entity).unwrap();
        near(reserve.remaining, 0.);
        near(reserve.away_seconds, 0.);
        assert!(reserve.occupied);
    }

    at(&mut app, drone, Vec3::ZERO);
    step(&mut app, 8., &[]);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        0.,
    );
    step(&mut app, 0.5, &[]);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        5.,
    );
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 0., &[]);
    let reserve = app.world().get::<ChargerReserve>(left).unwrap();
    near(reserve.remaining, 5.);
    near(reserve.away_seconds, 0.);

    at(&mut app, drone, Vec3::ZERO);
    step(&mut app, 7.9, &[]);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        5.,
    );
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 0., &[]);
    near(
        app.world()
            .get::<ChargerReserve>(left)
            .unwrap()
            .away_seconds,
        0.,
    );
}

#[test]
fn recovery_is_partial_after_the_delay_and_caps_at_capacity() {
    let (mut app, drone) = app();
    let left = charger_states(&mut app)
        .into_iter()
        .find(|(_, center, _)| center.x < 0.)
        .unwrap()
        .0;
    set_reserve(&mut app, left, 50., 6.);
    at(&mut app, drone, Vec3::ZERO);

    step(&mut app, 5., &[]);
    let reserve = app.world().get::<ChargerReserve>(left).unwrap();
    near(reserve.remaining, 80.);
    near(reserve.away_seconds, 11.);

    step(&mut app, 20., &[]);
    near(
        app.world().get::<ChargerReserve>(left).unwrap().remaining,
        200.,
    );
}

#[test]
fn setup_and_restart_use_the_configured_charger_capacity() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(ChargerConfig {
            capacity: 75.,
            recovery_delay: 8.,
            recovery_rate: 10.,
        })
        .add_plugins((ArenaPlugin, EnergyPlugin))
        .add_systems(
            Update,
            (prepare, update).chain().in_set(GameplaySet::Combat),
        );
    app.update();
    let chargers = charger_states(&mut app);
    assert_eq!(chargers.len(), 2);
    for (entity, _, reserve) in &chargers {
        near(reserve.remaining, 75.);
        set_reserve(&mut app, *entity, 10., 4.);
    }

    step(&mut app, 0., &[KeyCode::KeyR]);
    for (_, _, reserve) in charger_states(&mut app) {
        near(reserve.remaining, 75.);
        near(reserve.away_seconds, 0.);
        assert!(!reserve.occupied);
    }
}

fn recovery_case(dt: f64) -> f64 {
    let (mut app, drone) = app();
    let left = charger_states(&mut app)
        .into_iter()
        .find(|(_, center, _)| center.x < 0.)
        .unwrap()
        .0;
    set_reserve(&mut app, left, 0., 0.);
    at(&mut app, drone, Vec3::ZERO);
    while app.world().resource::<Time>().elapsed_secs_f64() + dt < 28. {
        step(&mut app, dt, &[]);
    }
    let remainder = 28. - app.world().resource::<Time>().elapsed_secs_f64();
    step(&mut app, remainder, &[]);
    app.world().get::<ChargerReserve>(left).unwrap().remaining
}

#[test]
fn long_recovery_update_matches_30_60_120_hz() {
    let long = recovery_case(28.);
    near(long, 200.);
    for dt in [1. / 30., 1. / 60., 1. / 120.] {
        near(recovery_case(dt), long);
    }
}

fn boundary_case(dt: f64) -> (f64, f64, [bool; 4], f64) {
    let (mut app, drone) = app();
    let left = charger_states(&mut app)
        .into_iter()
        .find(|(_, center, _)| center.x < 0.)
        .unwrap()
        .0;
    set_reserve(&mut app, left, 50., 0.);
    app.world_mut().resource_mut::<Energy>().current = 10.;
    {
        let mut modules = app.world_mut().resource_mut::<Modules>();
        modules.enabled = [true; 4];
        modules.shield.blocks = 0;
        modules.shield.remaining = 5.;
    }
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    let steps = (10. / dt).round() as usize;
    for _ in 0..steps {
        step(&mut app, dt, &[]);
    }
    let energy = app.world().resource::<Energy>().current;
    let remaining = app.world().get::<ChargerReserve>(left).unwrap().remaining;
    let modules = app.world().resource::<Modules>();
    (energy, remaining, modules.enabled, modules.shield.remaining)
}

#[test]
fn long_updates_match_30_60_120_hz_across_power_boundaries() {
    let long = boundary_case(10.);
    for dt in [1. / 30., 1. / 60., 1. / 120.] {
        let split = boundary_case(dt);
        near(split.0, long.0);
        near(split.1, long.1);
        assert_eq!(split.2, long.2);
        near(split.3, long.3);
    }
    near(long.0, 300. / 11.);
    near(long.1, 0.);
    assert_eq!(long.2, [false; 4]);
    near(long.3, 45. / 11.);
}

#[test]
fn starts_full_off_and_drains_without_targets_at_consistent_rates() {
    for dt in [1. / 30., 1. / 120., 1.] {
        let (mut app, _) = app();
        near(app.world().resource::<Energy>().current, 100.);
        assert!(
            !app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        step(&mut app, 0., &[KeyCode::Digit1]);
        for _ in 0..(3. / dt) as usize {
            step(&mut app, dt, &[]);
        }
        near(app.world().resource::<Energy>().current, 70.);
    }
}
#[test]
fn depletes_without_negative_energy_and_never_restarts_automatically() {
    let (mut app, drone) = app();
    step(&mut app, 0., &[KeyCode::Digit1]);
    step(&mut app, 11., &[]);
    let energy = app.world().resource::<Energy>();
    near(energy.current, 0.);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 1., &[]);
    near(app.world().resource::<Energy>().current, 25.);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(
        app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
}
#[test]
fn both_nodes_charge_clamp_and_combine_drain_before_clamping() {
    for x in [-280., 280.] {
        let (mut app, drone) = app();
        // This regression predates finite reserves and exercises battery/module
        // clamping, so give its selected field an effectively unlimited fixture.
        let node = charger_states(&mut app)
            .into_iter()
            .find(|(_, center, _)| center.x == x)
            .unwrap()
            .0;
        set_reserve(&mut app, node, 1_000., 0.);
        at(&mut app, drone, Vec3::new(x, 90., 0.));
        app.world_mut().resource_mut::<Energy>().current = 0.;
        step(&mut app, 4., &[]);
        near(app.world().resource::<Energy>().current, 100.);
        app.world_mut().resource_mut::<Energy>().current = 10.;
        step(&mut app, 1., &[KeyCode::Digit1]);
        near(app.world().resource::<Energy>().current, 25.);
        assert!(
            app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        step(&mut app, 10., &[]);
        near(app.world().resource::<Energy>().current, 100.);
        assert!(
            app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
    }
}
#[test]
fn volume_uses_center_inclusive_radial_and_vertical_bounds() {
    let node = ChargingNode {
        center: Vec3::new(-280., 0., 0.),
        radius: 90.,
        height: 160.,
    };
    for point in [
        Vec3::ZERO,
        Vec3::Y * 160.,
        Vec3::X * 90.,
        Vec3::Z * -90.,
        Vec3::new(54., 80., 72.),
    ] {
        assert!(node.contains(node.center + point), "inside {point:?}");
    }
    for point in [
        Vec3::Y * -0.001,
        Vec3::Y * 160.001,
        Vec3::X * 90.001,
        Vec3::Z * 90.001,
        Vec3::new(80., 90., 80.),
    ] {
        assert!(!node.contains(node.center + point), "outside {point:?}");
    }
}
#[test]
fn no_regeneration_above_or_outside_field_and_overlaps_do_not_stack() {
    let (mut app, drone) = app();
    app.world_mut().resource_mut::<Energy>().current = 20.;
    for point in [
        Vec3::new(-280., 161., 0.),
        Vec3::new(-189., 90., 0.),
        Vec3::new(0., 90., 0.),
    ] {
        at(&mut app, drone, point);
        step(&mut app, 1., &[]);
        near(app.world().resource::<Energy>().current, 20.);
        assert!(app.world().resource::<Energy>().charging.is_none());
    }
    app.world_mut().spawn(ChargingNode {
        center: Vec3::new(-280., 0., 0.),
        radius: 90.,
        height: 160.,
    });
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 1., &[]);
    near(app.world().resource::<Energy>().current, 45.);
}
#[test]
fn activation_threshold_rejection_and_key_holds() {
    let (mut app, _) = app();
    app.world_mut().resource_mut::<Energy>().current = 9.999;
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    assert!(app.world().resource::<Modules>().rejected_for[0] > 0.);
    app.world_mut().resource_mut::<Energy>().current = 10.;
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(
        app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(0.1));
    app.update();
    assert!(
        app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    step(&mut app, 3., &[]);
    near(app.world().resource::<Modules>().rejected_for[0], 0.);
}
#[test]
fn frozen_outcomes_and_repeated_restart_win_over_toggles() {
    let (mut app, drone) = app();
    let left = charger_states(&mut app)
        .into_iter()
        .find(|(_, center, _)| center.x < 0.)
        .unwrap()
        .0;
    for phase in [GamePhase::Choosing, GamePhase::Dead, GamePhase::Survived] {
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        {
            let mut e = app.world_mut().resource_mut::<Energy>();
            e.current = 42.;
        }
        app.world_mut().resource_mut::<Modules>().enabled[0] = true;
        app.world_mut().resource_mut::<Modules>().rejected_for[0] = 1.;
        set_reserve(&mut app, left, 42., 3.);
        step(&mut app, 5., &[KeyCode::Digit1]);
        let e = app.world().resource::<Energy>();
        near(e.current, 42.);
        assert!(
            app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        near(app.world().resource::<Modules>().rejected_for[0], 1.);
        let reserve = app.world().get::<ChargerReserve>(left).unwrap();
        near(reserve.remaining, 42.);
        near(reserve.away_seconds, 3.);
        assert!(!reserve.occupied);
        step(&mut app, 1., &[KeyCode::KeyR, KeyCode::Digit1]);
        let e = app.world().resource::<Energy>();
        near(e.current, 100.);
        assert!(
            !app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        assert!(e.charging.is_none());
        near(app.world().resource::<Modules>().rejected_for[0], 0.);
        let reserve = app.world().get::<ChargerReserve>(left).unwrap();
        near(reserve.remaining, 200.);
        near(reserve.away_seconds, 0.);
        assert!(!reserve.occupied);
    }
    assert_eq!(
        app.world_mut()
            .query::<&ChargingNode>()
            .iter(app.world())
            .count(),
        2
    );
}

#[test]
fn each_of_four_slot_keys_has_independent_power_drain() {
    for (key, drain) in [
        (KeyCode::Digit1, 10.),
        (KeyCode::Digit2, 8.),
        (KeyCode::Digit3, 8.),
        (KeyCode::Digit4, 10.),
    ] {
        let (mut app, _) = app();
        step(&mut app, 1., &[key]);
        near(app.world().resource::<Energy>().current, 100. - drain);
        step(&mut app, 0., &[key]);
        step(&mut app, 1., &[]);
        near(app.world().resource::<Energy>().current, 100. - drain);
    }
}

#[test]
fn all_four_drain_adds_and_exceeds_charging() {
    let (mut app, drone) = app();
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(
        &mut app,
        1.,
        &[
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
        ],
    );
    near(app.world().resource::<Energy>().current, 89.);
}
