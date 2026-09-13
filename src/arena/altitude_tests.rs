//! Altitude behavior through the production keyboard adapter and collision path.
use super::*;
use crate::world::{Solid, WorldGeometry};

struct Pilot {
    flight: DroneFlight,
    transform: Transform,
    config: FlightConfig,
    arena: Arena,
    world: Option<WorldGeometry>,
}

impl Pilot {
    fn new(y: f32, mobility: f32, acceleration: f32) -> Self {
        Self {
            flight: default(),
            transform: Transform::from_xyz(0., y, 0.),
            config: FlightConfig {
                max_horizontal_speed: 420. * mobility,
                horizontal_acceleration_multiplier: 2.5 * mobility,
                acceleration_multiplier: acceleration,
                ..default()
            },
            arena: Arena {
                half_size: Vec3::splat(100_000.),
            },
            world: None,
        }
    }

    fn tick(&mut self, keys: &[KeyCode], dt: f32) {
        let mut input = ButtonInput::default();
        for key in keys {
            input.press(*key);
        }
        let path = self.flight.advance(
            &mut self.transform,
            &FlightInput::read(&input, &self.config),
            &self.config,
            &self.arena,
            dt,
            self.world.as_ref(),
        );
        assert_eq!(path.last().unwrap().end, self.transform.translation);
        let half = drone_world_half_extents(self.transform.rotation);
        assert!(
            (self.transform.translation - self.arena.center())
                .abs()
                .cmple(self.arena.half_size - half + Vec3::splat(0.002))
                .all()
        );
        if let Some(world) = &self.world {
            for solid in &world.solids {
                assert!(!solid.overlaps(self.transform.translation, half));
            }
        }
    }

    fn fly(&mut self, keys: &[KeyCode], seconds: f32, hz: u32) {
        for _ in 0..(seconds * hz as f32).round() as u32 {
            self.tick(keys, 1. / hz as f32);
        }
    }

    fn holds(&mut self, keys: &[KeyCode], seconds: f32, hz: u32) {
        let height = self.transform.translation.y;
        for _ in 0..(seconds * hz as f32).round() as u32 {
            self.tick(keys, 1. / hz as f32);
            assert!(
                (self.transform.translation.y - height).abs() < 0.002,
                "{hz} Hz {keys:?}: {} != {height}",
                self.transform.translation.y
            );
            assert_eq!(self.flight.velocity.y, 0.);
        }
    }
}

const BUILDS: [(f32, f32); 4] = [(1., 1.), (1.25, 1.), (1., 0.675), (1.25, 0.675)];

#[test]
fn altitude_hold_survives_sustained_combined_maneuvers_all_builds_and_rates() {
    for hz in [30, 60, 120, 144] {
        for (mobility, acceleration) in BUILDS {
            let mut pilot = Pilot::new(137., mobility, acceleration);
            for keys in [
                vec![],
                vec![KeyCode::KeyW],
                vec![KeyCode::KeyS],
                vec![KeyCode::KeyQ],
                vec![KeyCode::KeyE],
                vec![KeyCode::KeyW, KeyCode::KeyQ, KeyCode::KeyD],
                vec![KeyCode::KeyS, KeyCode::KeyE, KeyCode::KeyA],
                vec![KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyA, KeyCode::KeyD],
                vec![],
            ] {
                pilot.holds(&keys, 3., hz);
            }
            pilot.tick(&[KeyCode::KeyW, KeyCode::KeyE], 0.25);
            assert_eq!(pilot.transform.translation.y, 137.);
        }
    }
}

#[test]
fn altitude_release_captures_each_new_height_and_opposing_inputs_cancel() {
    for hz in [30, 60, 120, 144] {
        for (mobility, acceleration) in BUILDS {
            let mut pilot = Pilot::new(137., mobility, acceleration);
            for key in [
                KeyCode::Space,
                KeyCode::ShiftLeft,
                KeyCode::Space,
                KeyCode::ShiftRight,
            ] {
                let before = pilot.transform.translation.y;
                let sign = if key == KeyCode::Space { 1. } else { -1. };
                pilot.tick(&[key], 1. / hz as f32);
                assert!(pilot.flight.velocity.y * sign > 0., "immediate override");
                pilot.fly(&[key, KeyCode::KeyW, KeyCode::KeyE], 0.5, hz);
                assert!((pilot.transform.translation.y - before) * sign > 2.);
                pilot.holds(&[KeyCode::KeyW, KeyCode::KeyQ], 2., hz);
                pilot.fly(&[key], 0.25, hz);
                pilot.holds(
                    &[
                        KeyCode::Space,
                        KeyCode::ShiftLeft,
                        KeyCode::ShiftRight,
                        KeyCode::KeyW,
                        KeyCode::KeyE,
                    ],
                    1.,
                    hz,
                );
                pilot.holds(&[], 1., hz);
            }
        }
    }
}

#[test]
fn altitude_manual_commands_keep_level_flight_response_while_tilted() {
    for hz in [30, 60, 120, 144] {
        for (mobility, acceleration) in BUILDS {
            for key in [KeyCode::Space, KeyCode::ShiftLeft, KeyCode::ShiftRight] {
                let mut level = Pilot::new(137., mobility, acceleration);
                let mut tilted = Pilot::new(137., mobility, acceleration);
                for _ in 0..hz {
                    level.tick(&[key], 1. / hz as f32);
                    tilted.tick(
                        &[key, KeyCode::KeyW, KeyCode::KeyQ, KeyCode::KeyD],
                        1. / hz as f32,
                    );
                    assert!(
                        (level.transform.translation.y - tilted.transform.translation.y).abs()
                            < 0.002
                    );
                    assert!((level.flight.velocity.y - tilted.flight.velocity.y).abs() < 0.002);
                }
                let thrust = if key == KeyCode::Space {
                    level.config.boost_thrust
                } else {
                    level.config.reduced_thrust
                };
                let drag = level.config.vertical_drag * acceleration;
                let expected =
                    level.config.gravity * (thrust - 1.) * acceleration * (1. - (-drag).exp())
                        / drag;
                assert!((level.flight.velocity.y - expected).abs() < 0.002);
            }
        }
    }
}

#[test]
fn altitude_hold_accepts_ground_ceiling_and_rotation_clearance_then_departs() {
    for hz in [30, 60, 120, 144] {
        for (into, away, start) in [
            (KeyCode::ShiftLeft, KeyCode::Space, 20.),
            (KeyCode::Space, KeyCode::ShiftRight, 280.),
        ] {
            let mut pilot = Pilot::new(start, 1., 1.);
            pilot.arena = Arena::default();
            pilot.fly(&[into], 3., hz);
            pilot.holds(&[], 1., hz);
            let contact = pilot.transform.translation.y;
            pilot.fly(&[KeyCode::KeyW], 0.2, hz);
            assert!(
                (pilot.transform.translation.y - contact).abs() > 5.,
                "rotation needs clearance"
            );
            pilot.holds(&[], 2., hz);
            let corrected = pilot.transform.translation.y;
            pilot.fly(&[away], 0.5, hz);
            assert!((pilot.transform.translation.y - corrected).abs() > 2.);
            pilot.holds(&[], 1., hz);
        }
    }
}

#[test]
fn altitude_hold_accepts_obstacle_top_and_underside_contact_without_stale_target() {
    for hz in [30, 60, 120, 144] {
        for (into, away, start) in [
            (KeyCode::ShiftLeft, KeyCode::Space, 195.),
            (KeyCode::Space, KeyCode::ShiftRight, 105.),
        ] {
            let mut pilot = Pilot::new(start, 1., 1.);
            pilot.arena = Arena::default();
            pilot.world = Some(WorldGeometry {
                solids: vec![Solid {
                    center: Vec3::new(0., 150., 0.),
                    half: Vec3::new(600., 20., 1000.),
                }],
                hazard: None,
            });
            pilot.fly(&[into], 2., hz);
            pilot.holds(&[], 1., hz);
            let contact = pilot.transform.translation.y;
            pilot.fly(&[KeyCode::KeyW], 0.2, hz);
            assert!((pilot.transform.translation.y - contact).abs() > 5.);
            pilot.holds(&[], 1., hz);
            let corrected = pilot.transform.translation.y;
            pilot.fly(&[away], 0.5, hz);
            assert!((pilot.transform.translation.y - corrected).abs() > 2.);
            pilot.holds(&[], 1., hz);
        }
    }
}

#[test]
fn altitude_assistance_preserves_raw_horizontal_steering_and_braking() {
    for hz in [30, 60, 120, 144] {
        for (mobility, acceleration) in BUILDS {
            let mut assisted = Pilot::new(10_000., mobility, acceleration);
            let mut raw = Pilot::new(10_000., mobility, acceleration);
            for keys in [
                vec![KeyCode::KeyW, KeyCode::Space],
                vec![KeyCode::KeyE, KeyCode::KeyA],
                vec![KeyCode::KeyS, KeyCode::ShiftRight],
                vec![KeyCode::KeyQ, KeyCode::KeyW],
                vec![],
            ] {
                let mut buttons = ButtonInput::default();
                for &key in &keys {
                    buttons.press(key);
                }
                let mut input = FlightInput::read(&buttons, &raw.config);
                input.vertical = VerticalControl::RotorThrust;
                for _ in 0..hz {
                    assisted.tick(&keys, 1. / hz as f32);
                    raw.flight.advance(
                        &mut raw.transform,
                        &input,
                        &raw.config,
                        &raw.arena,
                        1. / hz as f32,
                        None,
                    );
                    assert_eq!(assisted.flight.heading, raw.flight.heading);
                    assert_eq!(assisted.flight.tilt, raw.flight.tilt);
                    assert_eq!(assisted.flight.velocity.xz(), raw.flight.velocity.xz());
                    assert_eq!(
                        assisted.transform.translation.xz(),
                        raw.transform.translation.xz()
                    );
                }
            }
        }
    }
}

#[test]
fn altitude_hold_slides_along_obstacle_side_and_can_bank_away() {
    for hz in [30, 60, 120, 144] {
        let mut pilot = Pilot::new(137., 1.25, 0.675);
        pilot.world = Some(WorldGeometry {
            solids: vec![Solid {
                center: Vec3::new(100., 150., 0.),
                half: Vec3::new(1., 150., 1000.),
            }],
            hazard: None,
        });
        pilot.flight.velocity = Vec3::new(400., 0., -100.);
        pilot.holds(&[], 1., hz);
        assert!(pilot.transform.translation.x < 65.);
        assert!(pilot.transform.translation.z < -80.);
        let contact = pilot.transform.translation.x;
        // Opposing yaw keys fix heading so bank force points directly away.
        pilot.holds(&[KeyCode::KeyQ, KeyCode::KeyA, KeyCode::KeyD], 0.5, hz);
        assert!(pilot.transform.translation.x < contact - 10.);
    }
}
