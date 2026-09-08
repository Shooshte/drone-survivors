use super::{CombatConfig, Enemy};
use crate::arena::{Arena, Drone, DroneFlight, FlightConfig, FlightInput, world_half_extents};
use bevy::prelude::*;

pub(super) struct FlightSegment {
    pub(super) start: Vec3,
    pub(super) end: Vec3,
    pub(super) from: f32,
    pub(super) to: f32,
    pub(super) half: Vec3,
}

pub(super) fn spawn_enemy(
    commands: &mut Commands,
    config: &CombatConfig,
    position: Vec3,
    target: Vec3,
) {
    let offset = target - position;
    let flight = DroneFlight {
        heading: (-offset.x).atan2(-offset.z),
        ..default()
    };
    commands.spawn((
        Enemy {
            health: config.enemy_health,
            previous: position,
            path: Vec::new(),
        },
        Transform::from_translation(position).with_rotation(flight.rotation()),
        flight,
    ));
}

/// Arrival velocity and velocity-error feedback are pilot inputs. The shared
/// rotor-force integrator alone changes position, attitude and momentum.
fn pilot(position: Vec3, flight: &DroneFlight, target: Vec3, config: &FlightConfig) -> FlightInput {
    let offset = target - position;
    let horizontal = offset.with_y(0.);
    let distance = horizontal.length();
    let desired_speed = (distance * 1.5)
        .min((2. * 75. * distance).sqrt())
        .min(config.max_horizontal_speed);
    let desired = horizontal.normalize_or_zero() * desired_speed;
    let horizontal_acceleration = (desired - flight.velocity.with_y(0.)) * 2.5
        + flight.velocity.with_y(0.) * config.horizontal_drag;
    // Deliberate climb/descent pace leaves the scout room to launch under the
    // elevated starting pursuer while still allowing altitude pursuit.
    let vertical_limit = config.max_horizontal_speed.min(60.);
    let desired_vertical = (offset.y * 1.5).clamp(-vertical_limit, vertical_limit);
    let vertical_acceleration =
        (desired_vertical - flight.velocity.y) * 3. + flight.velocity.y * config.vertical_drag;
    let lift = (config.gravity + vertical_acceleration).clamp(
        config.gravity * config.reduced_thrust,
        config.gravity * config.boost_thrust * 0.9,
    );
    let angle = horizontal_acceleration
        .length()
        .atan2(lift)
        .min(config.max_tilt);
    let local =
        Quat::from_rotation_y(-flight.heading) * horizontal_acceleration.normalize_or_zero();
    let tilt = Vec2::new(local.x, -local.z) * (angle / config.max_tilt);
    let desired_heading = (-offset.x).atan2(-offset.z);
    let heading_error = (desired_heading - flight.heading + std::f32::consts::PI)
        .rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let yaw = if distance > 1. {
        (heading_error / (config.yaw_rate * 0.25)).clamp(-1., 1.)
    } else {
        0.
    };
    // This pilot compensates for its own current banking through rotor input.
    // Keyboard flight keeps the player's explicitly selected thrust unchanged.
    let thrust = (lift / (config.gravity * flight.tilt.length().cos()))
        .clamp(config.reduced_thrust, config.boost_thrust);
    FlightInput { tilt, yaw, thrust }
}

pub(super) fn chase(
    time: Res<Time>,
    arena: Res<Arena>,
    config: Res<CombatConfig>,
    world_flight: Res<FlightConfig>,
    drone: Single<&Transform, With<Drone>>,
    mut enemies: Query<(&mut Enemy, &mut Transform, &mut DroneFlight), Without<Drone>>,
) {
    let seconds = time.delta_secs();
    let steps = (seconds / (1. / 120.)).ceil().max(1.) as u32;
    let dt = seconds / steps as f32;
    let local_half = Vec3::splat(config.enemy_half_size);
    let profile = &FlightConfig {
        gravity: world_flight.gravity,
        ..config.enemy_flight
    };
    for (mut enemy, mut transform, mut flight) in &mut enemies {
        enemy.previous = transform.translation;
        enemy.path.clear();
        for index in 0..steps {
            let start = transform.translation;
            let half_before = world_half_extents(transform.rotation, local_half);
            let input = pilot(start, &flight, drone.translation, profile);
            // Bound orientation between samples, and the slight curvature of
            // the integrated position relative to this substep's straight chord.
            let angular_pad = local_half.length()
                * (profile.yaw_rate + profile.tilt_rate.max(profile.leveling_rate))
                * dt;
            let curve_pad = (profile.gravity * (profile.boost_thrust + 1.)
                + flight.velocity.length() * profile.vertical_drag.max(profile.horizontal_drag))
                * dt
                * dt
                / 8.;
            flight.step(&mut transform, &input, profile, &arena, local_half, dt);
            enemy.path.push(FlightSegment {
                start,
                end: transform.translation,
                from: index as f32 / steps as f32,
                to: (index + 1) as f32 / steps as f32,
                half: half_before.max(world_half_extents(transform.rotation, local_half))
                    + Vec3::splat(angular_pad + curve_pad),
            });
        }
    }
}
