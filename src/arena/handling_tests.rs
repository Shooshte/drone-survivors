//! Response measurements use the production keyboard adapter and flight integrator.
use super::*;

struct Pilot {
    flight: DroneFlight,
    transform: Transform,
    config: FlightConfig,
}
impl Pilot {
    fn new() -> Self {
        Self {
            flight: DroneFlight::default(),
            transform: Transform::from_xyz(0., 10_000., 0.),
            config: FlightConfig::default(),
        }
    }
    fn tick(&mut self, keys: &[KeyCode], dt: f32) {
        let mut input = ButtonInput::default();
        for key in keys {
            input.press(*key);
        }
        self.flight.advance(
            &mut self.transform,
            &FlightInput::read(&input, &self.config),
            &self.config,
            &Arena {
                half_size: Vec3::splat(100_000.),
            },
            dt,
            None,
        );
    }
    fn fly(&mut self, keys: &[KeyCode], seconds: f32, hz: u32) {
        for _ in 0..(seconds * hz as f32).round() as u32 {
            self.tick(keys, 1. / hz as f32);
        }
    }
    fn nose(&self) -> Vec3 {
        Quat::from_rotation_y(self.flight.heading) * Vec3::NEG_Z
    }
}

#[test]
fn handling_bank_turns_nose_and_forward_trajectory_in_both_directions() {
    for hz in [30, 60, 120] {
        let mut sides = vec![];
        for (key, sign) in [(KeyCode::KeyQ, -1.), (KeyCode::KeyE, 1.)] {
            let mut p = Pilot::new();
            p.flight.velocity = Vec3::NEG_Z * 200.;
            p.fly(&[KeyCode::KeyW, key], 0.75, hz);
            assert!(p.nose().x * sign > 0.5, "{hz} Hz: nose {:?}", p.nose());
            assert!(
                p.flight.velocity.x * sign > 80.,
                "{hz} Hz: {:?}",
                p.flight.velocity
            );
            assert!(p.transform.translation.x * sign > 20.);
            assert!(p.flight.velocity.z < -100., "retain forward momentum");
            sides.push((p.transform.translation, p.flight.velocity));
        }
        assert!((sides[0].0.x + sides[1].0.x).abs() < 0.1);
        assert!((sides[0].1.z - sides[1].1.z).abs() < 0.1);
    }
}

#[test]
fn handling_counter_yaw_takes_priority_and_release_levels_the_bank() {
    let mut p = Pilot::new();
    p.flight.velocity = Vec3::NEG_Z * 200.;
    p.fly(&[KeyCode::KeyE], 0.4, 120);
    assert!(p.nose().x > 0.4, "bank turns right before counter-steering");
    let nose = p.nose();
    let velocity = p.flight.velocity;
    p.tick(&[KeyCode::KeyE, KeyCode::KeyA], 1. / 120.);
    assert!(p.nose().x < nose.x, "direct yaw must turn left immediately");
    assert!(
        p.flight.velocity.distance(velocity) < 6.,
        "no velocity snap"
    );
    p.fly(&[], 0.2, 120);
    assert_eq!(p.flight.tilt, Vec2::ZERO);
    let heading = p.flight.heading;
    let speed = p.flight.velocity.with_y(0.).length();
    p.fly(&[], 0.5, 120);
    assert_eq!(p.flight.heading, heading);
    assert!(p.flight.velocity.with_y(0.).length() > speed * 0.85);
}

#[test]
fn handling_launch_and_opposite_input_response_across_builds_and_rates() {
    let mut results = vec![];
    for (name, horizontal, acceleration, stop_limit) in [
        ("base", 1., 1., 1.),
        ("mobility", 1.25, 1., 1.),
        ("armor+rounds", 1., 0.75 * 0.9, 1.5),
        ("mobility+armor+rounds", 1.25, 0.75 * 0.9, 1.5),
    ] {
        let mut stops = vec![];
        for hz in [30, 60, 120] {
            let mut p = Pilot::new();
            p.config.horizontal_acceleration_multiplier *= horizontal;
            p.config.max_horizontal_speed *= horizontal;
            p.config.acceleration_multiplier *= acceleration;
            p.fly(&[KeyCode::KeyW], 0.5, hz);
            let launch = -p.flight.velocity.z;
            if name == "base" {
                assert!(launch >= 140., "launch: {launch}");
            }
            p.flight.velocity = Vec3::NEG_Z * 300.;
            let start = p.transform.translation;
            let dt = 1. / hz as f32;
            let mut stop = None;
            let mut reversed = None;
            for frame in 1..=2 * hz {
                let before = p.flight.velocity;
                p.tick(&[KeyCode::KeyS], dt);
                assert!(
                    p.flight.velocity.distance(before) < 25.,
                    "smooth force response"
                );
                if frame == hz / 10 {
                    assert!(
                        p.flight.velocity.z > -300.,
                        "respond by 0.1 s: {:?}",
                        p.flight.velocity
                    );
                }
                if reversed.is_none() && p.flight.tilt.y <= -p.config.max_tilt + 0.001 {
                    reversed = Some(frame as f32 * dt);
                }
                if p.flight.velocity.z >= 0. {
                    stop = Some(frame as f32 * dt);
                    break;
                }
            }
            let stop = stop.expect("must reverse within two seconds");
            assert!(stop <= stop_limit, "{name} {hz} Hz stop {stop}");
            assert!(
                reversed.unwrap() <= 0.15,
                "{name} {hz} Hz tilt reversal {reversed:?}"
            );
            let distance = start.z - p.transform.translation.z;
            println!(
                "handling {name} {hz} Hz: launch={launch:.2}, tilt_reversal={:.3}s, stop={stop:.3}s, distance={distance:.2}",
                reversed.unwrap()
            );
            stops.push((stop, distance));
        }
        for stop in &stops {
            assert!((stop.1 - stops[0].1).abs() < 1., "{name}: {stops:?}");
        }
        results.push(stops[0]);
    }
    assert!(results[1].0 < results[0].0, "mobility improves stopping");
    assert!(
        results[2].0 > results[0].0 && results[2].1 > results[0].1,
        "heavy drawbacks remain meaningful"
    );
}

#[test]
fn handling_reversed_bank_changes_heading_promptly_without_snapping_momentum() {
    for hz in [30, 60, 120] {
        let mut p = Pilot::new();
        p.flight.velocity = Vec3::NEG_Z * 200.;
        p.fly(&[KeyCode::KeyE], 0.3, hz);
        p.fly(&[KeyCode::KeyQ], 0.15, hz);
        let nose = p.nose();
        assert!(p.flight.tilt.x < -0.5);
        p.tick(&[KeyCode::KeyQ], 1. / hz as f32);
        assert!(p.nose().x < nose.x, "new bank turns left");
        assert!(p.flight.velocity.z < -100., "momentum retained");
    }
}

#[test]
fn handling_fast_build_motion_envelope_contains_the_curved_midpoint() {
    // Mobility + Interceptor is the fastest real build. Hold a settled pitch
    // so this checks acceleration curvature independently of the angular pad.
    for heading in 0..360 {
        let mut p = Pilot::new();
        p.config.horizontal_acceleration_multiplier *= 1.25 * 1.3;
        p.flight.tilt = Vec2::Y * p.config.max_tilt;
        p.flight.heading = (heading as f32).to_radians();
        p.transform.rotation = p.flight.rotation();
        let start = p.transform;
        let input = FlightInput {
            tilt: Vec2::Y,
            yaw: 0.,
            thrust: p.config.boost_thrust,
        };
        let arena = Arena {
            half_size: Vec3::splat(100_000.),
        };
        let world = crate::world::WorldGeometry {
            solids: vec![],
            hazard: None,
        };
        let dt = 1. / 120.;
        let mut midpoint = p.flight;
        let mut middle_transform = start;
        midpoint.step(
            &mut middle_transform,
            &input,
            &p.config,
            &arena,
            DRONE_HALF_EXTENTS,
            dt / 2.,
        );
        let path = p.flight.step_in_world(
            &mut p.transform,
            &input,
            &p.config,
            &arena,
            DRONE_HALF_EXTENTS,
            dt,
            Some(&world),
        );
        let segment = path[0];
        let chord_midpoint = segment.start.lerp(segment.end, 0.5);
        let occupied = (middle_transform.translation - chord_midpoint).abs()
            + world_half_extents(middle_transform.rotation, DRONE_HALF_EXTENTS);
        assert!(
            occupied.cmple(segment.half).all(),
            "curved midpoint {occupied:?} escapes {:?}",
            segment.half
        );
    }
}

#[test]
fn handling_curves_and_counter_inputs_remain_consistent_across_frame_rates() {
    let mut results = Vec::new();
    for hz in [30, 60, 120, 144] {
        let mut p = Pilot::new();
        p.fly(&[KeyCode::KeyW], 0.5, hz);
        p.fly(&[KeyCode::KeyW, KeyCode::KeyE], 0.5, hz);
        p.fly(&[KeyCode::KeyS, KeyCode::KeyQ], 0.5, hz);
        p.fly(&[], 0.5, hz);
        results.push((p.transform.translation, p.flight));
    }
    for (position, flight) in &results {
        assert!(
            position.distance(results[0].0) < 1.,
            "positions: {results:?}"
        );
        assert!(
            flight.velocity.distance(results[0].1.velocity) < 1.,
            "velocities: {results:?}"
        );
        assert!(
            Quat::from_rotation_y(flight.heading)
                .angle_between(Quat::from_rotation_y(results[0].1.heading))
                < 0.01
        );
    }
}
