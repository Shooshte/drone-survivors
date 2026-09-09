use super::{Arena, DRONE_HALF_EXTENTS};
use bevy::prelude::*;

/// Actor flight tuning, in world units, seconds, and radians.
#[derive(Resource, Clone, Copy)]
pub(crate) struct FlightConfig {
    pub(crate) max_horizontal_speed: f32,
    pub(crate) horizontal_acceleration_multiplier: f32,
    /// Scales all net translational forces, including drag, while preserving hover.
    pub(crate) acceleration_multiplier: f32,
    pub(crate) max_tilt: f32,
    pub(crate) tilt_rate: f32,
    pub(crate) leveling_rate: f32,
    pub(crate) yaw_rate: f32,
    pub(crate) gravity: f32,
    pub(crate) neutral_thrust: f32,
    pub(crate) boost_thrust: f32,
    pub(crate) reduced_thrust: f32,
    pub(crate) horizontal_drag: f32,
    pub(crate) vertical_drag: f32,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            max_horizontal_speed: 420.,
            horizontal_acceleration_multiplier: 1.,
            acceleration_multiplier: 1.,
            max_tilt: 30_f32.to_radians(),
            tilt_rate: 240_f32.to_radians(),
            leveling_rate: 300_f32.to_radians(),
            yaw_rate: 240_f32.to_radians(),
            gravity: 360.,
            neutral_thrust: 1.,
            boost_thrust: 4. / 3.,
            reduced_thrust: 5. / 6.,
            horizontal_drag: 0.25,
            vertical_drag: 0.5,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct DroneFlight {
    pub(crate) velocity: Vec3,
    pub(crate) heading: f32,
    /// Heading-relative tilt vector: X banks right, Y pitches forward.
    pub(crate) tilt: Vec2,
}

pub(crate) struct FlightInput {
    pub(crate) tilt: Vec2,
    pub(crate) yaw: f32,
    pub(crate) thrust: f32,
}

impl FlightInput {
    pub(super) fn read(keys: &ButtonInput<KeyCode>, config: &FlightConfig) -> Self {
        let axis = |positive: &[KeyCode], negative: &[KeyCode]| {
            f32::from(keys.any_pressed(positive.iter().copied()))
                - f32::from(keys.any_pressed(negative.iter().copied()))
        };
        let tilt = Vec2::new(
            axis(&[KeyCode::KeyE], &[KeyCode::KeyQ]),
            axis(
                &[KeyCode::KeyW, KeyCode::ArrowUp],
                &[KeyCode::KeyS, KeyCode::ArrowDown],
            ),
        )
        .normalize_or_zero();
        let yaw = axis(
            &[KeyCode::KeyA, KeyCode::ArrowLeft],
            &[KeyCode::KeyD, KeyCode::ArrowRight],
        );
        let thrust = match axis(
            &[KeyCode::Space],
            &[KeyCode::ShiftLeft, KeyCode::ShiftRight],
        ) {
            x if x > 0. => config.boost_thrust,
            x if x < 0. => config.reduced_thrust,
            _ => config.neutral_thrust,
        };
        Self { tilt, yaw, thrust }
    }
}

impl DroneFlight {
    pub(super) fn advance(
        &mut self,
        transform: &mut Transform,
        input: &FlightInput,
        config: &FlightConfig,
        arena: &Arena,
        seconds: f32,
    ) {
        // Equal bounded substeps consume the whole render frame before combat.
        let steps = (seconds / (1. / 120.)).ceil().max(1.) as u32;
        let dt = seconds / steps as f32;
        for _ in 0..steps {
            self.step(transform, input, config, arena, DRONE_HALF_EXTENTS, dt);
        }
    }

    /// One bounded physics step, shared by keyboard and AI control sources.
    pub(crate) fn step(
        &mut self,
        transform: &mut Transform,
        input: &FlightInput,
        config: &FlightConfig,
        arena: &Arena,
        local_half: Vec3,
        dt: f32,
    ) {
        self.update_attitude(input, config, dt);
        transform.rotation = self.rotation();
        let acceleration = self.acceleration(transform.rotation, input, config);
        self.integrate(transform, acceleration, config, dt);
        self.contain(transform, arena, local_half);
    }

    fn update_attitude(&mut self, input: &FlightInput, config: &FlightConfig, dt: f32) {
        self.heading = (self.heading + input.yaw.clamp(-1., 1.) * config.yaw_rate * dt)
            .rem_euclid(std::f32::consts::TAU);
        let target = input.tilt.clamp_length_max(1.) * config.max_tilt;
        let mut delta = Vec2::ZERO;
        let mut total_rate: f32 = 0.;
        for axis in 0..2 {
            let rate = if input.tilt[axis] == 0. {
                config.leveling_rate
            } else {
                config.tilt_rate
            };
            let difference = target[axis] - self.tilt[axis];
            delta[axis] = difference.clamp(-rate * dt, rate * dt);
            if difference != 0. {
                total_rate = total_rate.max(rate);
            }
        }
        // Each axis retains its own target and response; cap the combined
        // change so diagonal input never multiplies the angular response rate.
        self.tilt += delta.clamp_length_max(total_rate * dt);
        self.tilt = self.tilt.clamp_length_max(config.max_tilt);
    }

    pub(crate) fn rotation(&self) -> Quat {
        // One axis-angle tilt preserves the shared angle limit exactly, unlike
        // composing independent Euler pitch/roll rotations.
        Quat::from_rotation_y(self.heading)
            * Quat::from_scaled_axis(Vec3::new(-self.tilt.y, 0., -self.tilt.x))
    }

    fn acceleration(&self, rotation: Quat, input: &FlightInput, config: &FlightConfig) -> Vec3 {
        let mut acceleration =
            rotation * Vec3::Y * (config.gravity * input.thrust) - Vec3::Y * config.gravity;
        acceleration.x *= config.horizontal_acceleration_multiplier;
        acceleration.z *= config.horizontal_acceleration_multiplier;
        let horizontal = self.velocity.with_y(0.);
        let speed = horizontal.length();
        if speed > config.max_horizontal_speed * 0.9 {
            let radial = horizontal / speed;
            let outward = acceleration.dot(radial).max(0.);
            let fraction = ((speed / config.max_horizontal_speed - 0.9) / 0.1).clamp(0., 1.);
            let taper = fraction * fraction * (3. - 2. * fraction);
            // Remove only outward horizontal force; retain steering, braking,
            // and the vertical lift lost to tilt.
            acceleration -= radial * outward * taper;
        }
        acceleration
    }

    fn integrate(
        &mut self,
        transform: &mut Transform,
        acceleration: Vec3,
        config: &FlightConfig,
        dt: f32,
    ) {
        for axis in 0..3 {
            let drag = if axis == 1 {
                config.vertical_drag
            } else {
                config.horizontal_drag
            } * config.acceleration_multiplier;
            let velocity = self.velocity[axis];
            let force = acceleration[axis] * config.acceleration_multiplier;
            if drag > 0. {
                // Exact linear-drag integration for constant substep force.
                let decay = (-drag * dt).exp();
                let response = -(-drag * dt).exp_m1() / drag;
                transform.translation[axis] += velocity * response + force * (dt - response) / drag;
                self.velocity[axis] = velocity * decay + force * response;
            } else {
                transform.translation[axis] += velocity * dt + 0.5 * force * dt * dt;
                self.velocity[axis] += force * dt;
            }
        }
        let horizontal = self
            .velocity
            .with_y(0.)
            .clamp_length_max(config.max_horizontal_speed);
        self.velocity.x = horizontal.x;
        self.velocity.z = horizontal.z;
    }

    fn contain(&mut self, transform: &mut Transform, arena: &Arena, local_half: Vec3) {
        let limit = arena.half_size - world_half_extents(transform.rotation, local_half);
        let min = arena.center() - limit;
        let max = arena.center() + limit;
        for axis in 0..3 {
            if transform.translation[axis] <= min[axis] {
                transform.translation[axis] = min[axis];
                self.velocity[axis] = self.velocity[axis].max(0.);
            } else if transform.translation[axis] >= max[axis] {
                transform.translation[axis] = max[axis];
                self.velocity[axis] = self.velocity[axis].min(0.);
            }
        }
    }
}

/// Conservative world AABB shared by arena contact, combat, and altitude guide.
pub(crate) fn drone_world_half_extents(rotation: Quat) -> Vec3 {
    world_half_extents(rotation, DRONE_HALF_EXTENTS)
}

pub(crate) fn world_half_extents(rotation: Quat, local_half: Vec3) -> Vec3 {
    let basis = Mat3::from_quat(rotation);
    basis.x_axis.abs() * local_half.x
        + basis.y_axis.abs() * local_half.y
        + basis.z_axis.abs() * local_half.z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_taper_only_reduces_outward_horizontal_force() {
        let config = FlightConfig::default();
        let input = FlightInput {
            tilt: Vec2::Y,
            yaw: 0.,
            thrust: config.boost_thrust,
        };
        for (speed, expected) in [
            (0., 240.),
            (378., 240.),
            (399., 120.),
            (420., 0.),
            (430., 0.),
        ] {
            let flight = DroneFlight {
                velocity: Vec3::NEG_Z * speed,
                tilt: Vec2::Y * config.max_tilt,
                ..default()
            };
            let acceleration = flight.acceleration(flight.rotation(), &input, &config);
            assert!((acceleration.z + expected).abs() < 0.001);
            assert!((acceleration.y - (480. * 30_f32.to_radians().cos() - 360.)).abs() < 0.001);
        }
        for tilt in [Vec2::NEG_Y, Vec2::X] {
            let flight = DroneFlight {
                velocity: Vec3::NEG_Z * 420.,
                tilt: tilt * config.max_tilt,
                ..default()
            };
            let acceleration = flight.acceleration(flight.rotation(), &input, &config);
            let expected = flight.rotation() * Vec3::Y * 480. - Vec3::Y * 360.;
            assert!(acceleration.distance(expected) < 0.001);
        }
    }
}
