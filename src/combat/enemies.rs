use super::{CombatConfig, ENEMY_STARTS, Enemy, collision::segment_box};
use crate::arena::{Arena, DRONE_HALF_EXTENTS, Drone};
use bevy::prelude::*;

pub(super) fn spawn_enemies(commands: &mut Commands, config: &CombatConfig) {
    for position in ENEMY_STARTS {
        commands.spawn((
            Enemy {
                health: config.enemy_health,
                previous: position,
            },
            Transform::from_translation(position),
        ));
    }
}

pub(super) fn chase(
    time: Res<Time>,
    arena: Res<Arena>,
    config: Res<CombatConfig>,
    drone: Single<&Transform, With<Drone>>,
    mut enemies: Query<(&mut Enemy, &mut Transform), Without<Drone>>,
) {
    let contact_half = DRONE_HALF_EXTENTS + Vec3::splat(config.enemy_half_size);
    for (mut enemy, mut transform) in &mut enemies {
        enemy.previous = transform.translation;
        let offset = drone.translation - transform.translation;
        let fraction = segment_box(
            transform.translation,
            drone.translation,
            drone.translation,
            contact_half,
        )
        .unwrap_or(1.);
        let distance = (config.chase_speed * time.delta_secs()).min(offset.length() * fraction);
        let next = transform.translation + offset.normalize_or_zero() * distance;
        let limit = arena.half_size - Vec3::splat(config.enemy_half_size);
        transform.translation = next.clamp(arena.center() - limit, arena.center() + limit);
    }
}
