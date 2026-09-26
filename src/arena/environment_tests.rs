use super::*;
use crate::world::{Solid, WorldGeometry, environment::Environment};

fn coast(
    hz: u32,
    environment: &Environment,
    world: Option<&WorldGeometry>,
) -> (Transform, DroneFlight) {
    let config = FlightConfig {
        horizontal_drag: 0.,
        ..default()
    };
    let input = FlightInput::read(&ButtonInput::default(), &config);
    let mut flight = DroneFlight {
        velocity: Vec3::X * 200.,
        ..default()
    };
    let mut transform = Transform::from_xyz(-620., 90., -190.);
    for _ in 0..2 * hz {
        let path = flight.advance_in_environment(
            &mut transform,
            &input,
            &config,
            &Arena::default(),
            1. / hz as f32,
            world,
            Some(environment),
        );
        assert_eq!(path.last().unwrap().end, transform.translation);
        assert!(
            path.iter()
                .all(|s| s.from <= s.to && s.from >= 0. && s.to <= 1.)
        );
    }
    (transform, flight)
}

#[test]
fn environment_field_crossing_is_consistent_at_30_60_120_hz_and_has_no_exit_impulse() {
    let mut environment = Environment::default();
    environment.reset(true);
    let results = [30, 60, 120].map(|hz| coast(hz, &environment, None));
    for (transform, flight) in results {
        // 420 units at 280/s, then 100 units at 200/s. Boundary sampling <= one substep.
        assert!((transform.translation.x + 100.).abs() < 2.);
        assert!((transform.translation.y - 90.).abs() < 0.001);
        assert!((flight.velocity.x - 200.).abs() < 0.001);
        assert!(transform.translation.distance(results[0].0.translation) < 1.);
    }
}

#[test]
fn environment_boost_respects_swept_terrain_and_actual_player_path() {
    let mut environment = Environment::default();
    environment.reset(true);
    let world = WorldGeometry {
        navigation: None,
        solids: vec![Solid {
            center: Vec3::new(-300., 150., -190.),
            half: Vec3::new(5., 150., 125.),
        }],
        hazard: None,
    };
    for hz in [30, 60, 120] {
        let (transform, flight) = coast(hz, &environment, Some(&world));
        assert!(transform.translation.x <= -340.);
        assert!(transform.translation.x > -341.);
        assert_eq!(flight.velocity.x, 0.);
    }
}

#[test]
fn mission01_boosted_rotor_flight_measures_authored_route() {
    use crate::arena::{DroneFlight, FlightConfig, FlightInput, VerticalControl};
    use crate::world::mission01::{arena, fields, geometry, route, start};
    let world = geometry();
    let environment = Environment {
        fields: fields(),
        ..default()
    };
    let arena = arena();
    let config = FlightConfig::default();
    let mut transform = Transform::from_translation(start());
    let mut flight = DroneFlight::default();
    let mut elapsed = 0.;
    for goal in route().into_iter().skip(1) {
        let deadline = elapsed + 60.;
        while transform.translation.distance(goal) > 120. && elapsed < deadline {
            let offset = (goal - transform.translation).with_y(0.);
            let desired =
                offset.normalize_or_zero() * config.max_horizontal_speed.min(offset.length() * 2.);
            let acceleration = (desired - flight.velocity.with_y(0.)) * 3.
                + flight.velocity.with_y(0.) * config.horizontal_drag;
            let angle = (acceleration.length()
                / (config.gravity * config.horizontal_acceleration_multiplier))
                .clamp(0., 1.)
                .asin()
                .min(config.max_tilt);
            let input = FlightInput {
                tilt: Vec2::new(acceleration.x, -acceleration.z).normalize_or_zero()
                    * (angle / config.max_tilt),
                yaw: 0.,
                yaw_override: true,
                thrust: 1.,
                vertical: VerticalControl::AltitudeHold,
            };
            flight.advance_in_environment(
                &mut transform,
                &input,
                &config,
                &arena,
                1. / 120.,
                Some(&world),
                Some(&environment),
            );
            assert!(world.solids.iter().all(|s| !s.overlaps(
                transform.translation,
                world_half_extents(transform.rotation, DRONE_HALF_EXTENTS)
            )));
            elapsed += 1. / 120.;
        }
        assert!(
            transform.translation.distance(goal) <= 120.,
            "flight stalled at {:?} toward {goal:?}",
            transform.translation
        );
    }
    let length: f32 = route().windows(2).map(|p| p[0].distance(p[1])).sum();
    eprintln!(
        "Mission 01 route: {length:.0} world units, authored-field rotor flight {elapsed:.2}s, no equipment, boosts enabled, altitude hold, 120Hz"
    );
    assert!(
        (165. ..195.).contains(&elapsed),
        "ordinary route took {elapsed}s"
    );
}

#[test]
fn mission01_authored_arrows_change_real_flight_with_against_and_perpendicular() {
    let world = crate::world::mission01::geometry();
    let arena = crate::world::mission01::arena();
    let environment = Environment {
        fields: crate::world::mission01::fields(),
        ..default()
    };
    let config = FlightConfig {
        horizontal_drag: 0.,
        ..default()
    };
    let input = FlightInput::read(&ButtonInput::default(), &config);
    for (index, field) in environment.fields.iter().enumerate() {
        let arrow = field.direction.normalize();
        for (direction, expected) in [
            (arrow, 280.),
            (-arrow, 120.),
            (Vec3::new(-arrow.z, 0., arrow.x), 200.),
        ] {
            let start = field.bounds.center.with_y(90.);
            let mut transform = Transform::from_translation(start);
            let mut flight = DroneFlight {
                velocity: direction * 200.,
                ..default()
            };
            for _ in 0..120 {
                flight.advance_in_environment(
                    &mut transform,
                    &input,
                    &config,
                    &arena,
                    1. / 120.,
                    Some(&world),
                    Some(&environment),
                );
            }
            let distance = (transform.translation - start).dot(direction);
            assert!(
                (distance - expected).abs() < 0.25,
                "field {index}: {direction:?} moved {distance}, expected {expected}"
            );
            assert!((flight.velocity.length() - 200.).abs() < 0.01);
            assert!((transform.translation.y - 90.).abs() < 0.001);
        }
    }
}
