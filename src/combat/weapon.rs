use super::{
    CombatConfig, CombatOutcome, CombatOutcomes, Encounter, Enemy, Projectile, Weapon,
    collision::segment_box, rockets::Rocket,
};
use crate::{
    arena::{Arena, Drone, world_half_extents},
    game::GamePhase,
    modules::{ModuleConfig, ModuleKind, Modules},
};
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<CombatConfig>,
    world: Option<Res<crate::world::WorldGeometry>>,
    phase: Res<GamePhase>,
    modules: Res<Modules>,
    module_config: Res<ModuleConfig>,
    mut weapon: ResMut<Weapon>,
    drone: Single<&Transform, With<Drone>>,
    enemies: Query<(Entity, &Enemy, &Transform)>,
) {
    if *phase != GamePhase::Playing {
        return;
    }
    let now = time.elapsed_secs_f64();
    let multiplier = if modules.active(ModuleKind::Overdrive) {
        module_config.overdrive_multiplier
    } else {
        1.
    };
    let interval = config.fire_interval / multiplier;
    if let Some(previous) = weapon.interval
        && previous != interval
        && weapon.ready_at > now
    {
        // Preserve progress so toggling cannot reset a cooldown or bank a shot.
        weapon.ready_at = now + (weapon.ready_at - now) / previous * interval;
    }
    weapon.interval = Some(interval);
    let target = enemies
        .iter()
        .filter(|(_, enemy, _)| enemy.health > 0)
        .map(|(id, _, transform)| {
            (
                id,
                transform.translation,
                transform.translation.distance_squared(drone.translation),
            )
        })
        .filter(|(_, position, distance)| {
            *distance <= config.target_range.powi(2)
                && world
                    .as_ref()
                    .is_none_or(|w| w.line_clear(drone.translation, *position))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.to_bits().cmp(&b.0.to_bits())));
    let Some((_, position, _)) = target else {
        // A target-free interval never banks shots for a later burst.
        weapon.ready_at = weapon.ready_at.max(now);
        return;
    };
    if now + 1e-7 < weapon.ready_at {
        return;
    }
    let direction = (position - drone.translation)
        .try_normalize()
        .unwrap_or(Vec3::NEG_Z);
    commands.spawn((
        Projectile {
            velocity: direction * config.projectile_speed,
            remaining: config.projectile_lifetime,
        },
        Transform::from_translation(drone.translation),
    ));
    // Keep normal cadence across fractional frames, discard missed shots on hitches.
    weapon.ready_at = if now - weapon.ready_at >= interval {
        now + interval
    } else {
        weapon.ready_at + interval
    };
}

#[allow(clippy::too_many_arguments)]
pub(super) fn advance_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    arena: Res<Arena>,
    config: Res<CombatConfig>,
    world: Option<Res<crate::world::WorldGeometry>>,
    module_config: Res<ModuleConfig>,
    mut projectiles: Query<
        (Entity, &mut Projectile, &mut Transform, Option<&Rocket>),
        Without<Enemy>,
    >,
    mut enemies: Query<(Entity, &mut Enemy, &Transform), Without<Projectile>>,
    mut run: ResMut<Encounter>,
    mut outcomes: ResMut<CombatOutcomes>,
) {
    let dt = time.delta_secs();
    for (id, mut shot, mut transform, rocket) in &mut projectiles {
        let start = transform.translation;
        if shot.remaining <= 0. || (start - arena.center()).abs().cmpgt(arena.half_size).any() {
            commands.entity(id).despawn();
            continue;
        }
        let travel_time = dt.min(shot.remaining);
        let mut fraction = if dt > 0. { travel_time / dt } else { 1. };
        let mut end = start + shot.velocity * travel_time;
        if (end - arena.center()).abs().cmpgt(arena.half_size).any() {
            let exit_fraction =
                1. - segment_box(end, start, arena.center(), arena.half_size).unwrap_or(1.);
            fraction *= exit_fraction;
            end = start.lerp(end, exit_fraction);
        }
        let hit = enemies
            .iter()
            .filter(|(_, enemy, _)| enemy.health > 0)
            .filter_map(|(entity, enemy, target)| {
                let impact = if enemy.path.is_empty() {
                    let half =
                        world_half_extents(target.rotation, Vec3::splat(config.enemy_half_size))
                            + Vec3::splat(config.projectile_radius);
                    let target_end = enemy.previous.lerp(target.translation, fraction);
                    segment_box(start - enemy.previous, end - target_end, Vec3::ZERO, half)
                        .map(|impact| impact * fraction)
                } else {
                    enemy
                        .path
                        .iter()
                        .filter_map(|segment| {
                            if segment.from > fraction {
                                return None;
                            }
                            let to = segment.to.min(fraction);
                            let enemy_end = segment.start.lerp(
                                segment.end,
                                (to - segment.from) / (segment.to - segment.from),
                            );
                            let shot_start = start + shot.velocity * (dt * segment.from);
                            let shot_end = start + shot.velocity * (dt * to);
                            segment_box(
                                shot_start - segment.start,
                                shot_end - enemy_end,
                                Vec3::ZERO,
                                segment.half + Vec3::splat(config.projectile_radius),
                            )
                            .map(|impact| segment.from + impact * (to - segment.from))
                        })
                        .min_by(f32::total_cmp)
                };
                impact.map(|impact| (entity, impact))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.to_bits().cmp(&b.0.to_bits())));
        let terrain = world
            .as_ref()
            .and_then(|w| w.first_hit(start, end, config.projectile_radius))
            .map(|impact| impact * fraction);
        let hit = match (hit, terrain) {
            (Some((_, enemy_at)), Some(wall_at)) if wall_at <= enemy_at => Some((None, wall_at)),
            (Some((target, at)), _) => Some((Some(target), at)),
            (None, Some(at)) => Some((None, at)),
            (None, None) => None,
        };
        if let Some((target, impact)) = hit {
            if rocket.is_some() {
                let impact_position = start + shot.velocity * (dt * impact);
                outcomes.0.push(CombatOutcome::RocketExplosion {
                    position: impact_position,
                    radius: module_config.rocket_radius,
                });
                let radius_squared = module_config.rocket_radius.powi(2);
                for (entity, mut enemy, enemy_transform) in &mut enemies {
                    if enemy.health == 0 {
                        continue;
                    }
                    let position = enemy_position_at(&enemy, enemy_transform, impact);
                    let visibility_origin =
                        impact_position - shot.velocity.normalize_or_zero() * 0.01;
                    if (Some(entity) != target
                        && position.distance_squared(impact_position) > radius_squared)
                        || world
                            .as_ref()
                            .is_some_and(|w| !w.line_clear(visibility_origin, position))
                    {
                        continue;
                    }
                    enemy.health = enemy.health.saturating_sub(module_config.rocket_damage);
                    let killed = enemy.health == 0;
                    outcomes.0.push(CombatOutcome::Hit {
                        entity,
                        position,
                        killed,
                    });
                    if killed {
                        run.kills = run.kills.saturating_add(1);
                        commands.entity(entity).despawn();
                    }
                }
            } else if let Some(target) = target {
                let (_, mut enemy, enemy_transform) = enemies.get_mut(target).unwrap();
                enemy.health = enemy.health.saturating_sub(config.shot_damage);
                outcomes.0.push(CombatOutcome::Hit {
                    entity: target,
                    position: enemy_transform.translation,
                    killed: enemy.health == 0,
                });
                if enemy.health == 0 {
                    run.kills = run.kills.saturating_add(1);
                    commands.entity(target).despawn();
                }
            }
            commands.entity(id).despawn();
        } else {
            transform.translation = end;
            shot.remaining -= dt;
            if shot.remaining <= 0. || fraction < 1. {
                commands.entity(id).despawn();
            }
        }
    }
}

fn enemy_position_at(enemy: &Enemy, transform: &Transform, fraction: f32) -> Vec3 {
    if enemy.path.is_empty() {
        return enemy.previous.lerp(transform.translation, fraction);
    }
    let segment = enemy
        .path
        .iter()
        .find(|segment| fraction >= segment.from && fraction <= segment.to)
        .unwrap_or_else(|| enemy.path.last().unwrap());
    let span = segment.to - segment.from;
    let local = if span > 0. {
        ((fraction - segment.from) / span).clamp(0., 1.)
    } else {
        1.
    };
    segment.start.lerp(segment.end, local)
}
