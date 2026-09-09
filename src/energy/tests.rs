use super::*;
use crate::arena::ArenaPlugin;
use std::time::Duration;

pub(super) fn app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, EnergyPlugin))
        .add_systems(Update, update.in_set(GameplaySet::Combat));
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    (app, drone)
}
pub(super) fn step(app: &mut App, dt: f64, keys: &[KeyCode]) {
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
pub(super) fn at(app: &mut App, drone: Entity, point: Vec3) {
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = point;
}
fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-5, "{actual} != {expected}");
}

#[test]
fn starts_full_off_and_drains_without_targets_at_consistent_rates() {
    for dt in [1. / 30., 1. / 120., 1.] {
        let (mut app, _) = app();
        near(app.world().resource::<Energy>().current, 100.);
        assert!(!app.world().resource::<Energy>().overdrive);
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
    assert!(!energy.overdrive);
    at(&mut app, drone, Vec3::new(-280., 90., 0.));
    step(&mut app, 1., &[]);
    near(app.world().resource::<Energy>().current, 25.);
    assert!(!app.world().resource::<Energy>().overdrive);
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(app.world().resource::<Energy>().overdrive);
}
#[test]
fn both_nodes_charge_clamp_and_combine_drain_before_clamping() {
    for x in [-280., 280.] {
        let (mut app, drone) = app();
        at(&mut app, drone, Vec3::new(x, 90., 0.));
        app.world_mut().resource_mut::<Energy>().current = 0.;
        step(&mut app, 4., &[]);
        near(app.world().resource::<Energy>().current, 100.);
        app.world_mut().resource_mut::<Energy>().current = 10.;
        step(&mut app, 1., &[KeyCode::Digit1]);
        near(app.world().resource::<Energy>().current, 25.);
        assert!(app.world().resource::<Energy>().overdrive);
        step(&mut app, 10., &[]);
        near(app.world().resource::<Energy>().current, 100.);
        assert!(app.world().resource::<Energy>().overdrive);
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
    assert!(!app.world().resource::<Energy>().overdrive);
    assert!(app.world().resource::<Energy>().rejected_for > 0.);
    app.world_mut().resource_mut::<Energy>().current = 10.;
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(app.world().resource::<Energy>().overdrive);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(0.1));
    app.update();
    assert!(app.world().resource::<Energy>().overdrive);
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!(!app.world().resource::<Energy>().overdrive);
    step(&mut app, 3., &[]);
    near(app.world().resource::<Energy>().rejected_for, 0.);
}
#[test]
fn frozen_outcomes_and_repeated_restart_win_over_toggles() {
    let (mut app, drone) = app();
    for phase in [GamePhase::Dead, GamePhase::Survived] {
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        {
            let mut e = app.world_mut().resource_mut::<Energy>();
            e.current = 42.;
            e.overdrive = true;
            e.rejected_for = 1.;
        }
        step(&mut app, 5., &[KeyCode::Digit1]);
        let e = app.world().resource::<Energy>();
        near(e.current, 42.);
        assert!(e.overdrive);
        near(e.rejected_for, 1.);
        step(&mut app, 1., &[KeyCode::KeyR, KeyCode::Digit1]);
        let e = app.world().resource::<Energy>();
        near(e.current, 100.);
        assert!(!e.overdrive);
        assert!(e.charging.is_none());
        near(e.rejected_for, 0.);
    }
    assert_eq!(
        app.world_mut()
            .query::<&ChargingNode>()
            .iter(app.world())
            .count(),
        2
    );
}
