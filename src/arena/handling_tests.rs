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
