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
