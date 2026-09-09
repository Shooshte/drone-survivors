use super::{CombatConfig, Enemy};
use crate::arena::{Arena, Drone, DroneFlight, FlightConfig, FlightInput, world_half_extents};
use bevy::prelude::*;

pub(super) type FlightSegment = crate::world::MotionSegment;

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
fn pilot(
    position: Vec3,
    flight: &DroneFlight,
    target: Vec3,
    config: &FlightConfig,
    separation: Vec3,
) -> FlightInput {
    let offset = target - position;
    let horizontal = offset.with_y(0.);
    let distance = horizontal.length();
    let desired_speed = (distance * 1.5)
        .min((2. * 75. * distance).sqrt())
        .min(config.max_horizontal_speed);
    let desired = horizontal.normalize_or_zero() * desired_speed;
    let horizontal_acceleration = (desired - flight.velocity.with_y(0.)) * 2.5
        + flight.velocity.with_y(0.) * config.horizontal_drag
        + separation.with_y(0.);
    // Deliberate climb/descent pace leaves the scout room to launch under the
    // elevated starting pursuer while still allowing altitude pursuit.
    let vertical_limit = config.max_horizontal_speed.min(60.);
    let desired_vertical = (offset.y * 1.5).clamp(-vertical_limit, vertical_limit);
    let vertical_acceleration = (desired_vertical - flight.velocity.y) * 3.
        + flight.velocity.y * config.vertical_drag
        + separation.y;
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

#[allow(clippy::too_many_arguments)]
pub(super) fn chase(
    time: Res<Time>,
    arena: Res<Arena>,
    config: Res<CombatConfig>,
    world_flight: Res<FlightConfig>,
    world: Option<Res<crate::world::WorldGeometry>>,
    drone: Single<&Transform, With<Drone>>,
    mut enemies: Query<(Entity, &mut Enemy, &mut Transform, &mut DroneFlight), Without<Drone>>,
) {
    let seconds = time.delta_secs();
    let steps = (seconds / (1. / 120.)).ceil().max(1.) as u32;
    let dt = seconds / steps as f32;
    let local_half = Vec3::splat(config.enemy_half_size);
    let profile = &FlightConfig {
        gravity: world_flight.gravity,
        ..config.enemy_flight
    };
    let mut positions: Vec<_> = enemies
        .iter()
        .filter(|(_, e, _, _)| e.health > 0)
        .map(|(id, _, t, _)| (id, t.translation))
        .collect();
    positions.sort_by_key(|(id, _)| id.to_bits());
    for (id, mut enemy, mut transform, mut flight) in &mut enemies {
        let separation = separation(
            id,
            transform.translation,
            &positions,
            config.separation_radius,
        ) * config.separation_acceleration;
        enemy.previous = transform.translation;
        enemy.path.clear();
        let target = world
            .as_deref()
            .map_or(Some(drone.translation), |world| {
                crate::world::navigation::next_point(
                    world,
                    transform.translation,
                    drone.translation,
                    world_half_extents(transform.rotation, local_half),
                )
            })
            .unwrap_or(transform.translation);
        for index in 0..steps {
            let input = pilot(transform.translation, &flight, target, profile, separation);
            for mut segment in flight.step_in_world(
                &mut transform,
                &input,
                profile,
                &arena,
                local_half,
                dt,
                world.as_deref(),
            ) {
                segment.from = (index as f32 + segment.from) / steps as f32;
                segment.to = (index as f32 + segment.to) / steps as f32;
                enemy.path.push(segment);
            }
        }
    }
}

// A single neighbor snapshot per update avoids order-dependent position writes.
// The bounded acceleration enters the same rotor inputs as pursuit.
fn separation(id: Entity, position: Vec3, neighbors: &[(Entity, Vec3)], radius: f32) -> Vec3 {
    let mut force = Vec3::ZERO;
    for &(other, point) in neighbors {
        if other == id {
            continue;
        }
        let offset = position - point;
        let distance = offset.length();
        if distance >= radius {
            continue;
        }
        let direction = if distance > 0.001 {
            offset / distance
        } else {
            // Antisymmetric even at exactly coincident starts; never NaN.
            if id.to_bits() < other.to_bits() {
                Vec3::Z
            } else {
                Vec3::NEG_Z
            }
        };
        force += direction * (1. - distance / radius);
    }
    force.clamp_length_max(1.)
}

#[cfg(test)]
#[path = "terrain_tests.rs"]
mod terrain_tests;
