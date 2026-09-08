use super::{CombatConfig, Enemy, PlayerHealth, Projectile, Weapon, enemies::spawn_enemies};
use crate::{
    arena::{Drone, drone_world_half_extents, world_half_extents},
    game::GamePhase,
};
use bevy::prelude::*;

type CombatEntities = Or<(With<Enemy>, With<Projectile>)>;

pub(super) fn setup(mut commands: Commands, config: Res<CombatConfig>) {
    commands.insert_resource(PlayerHealth {
        current: config.player_health,
        invulnerable_until: 0.,
    });
    spawn_enemies(&mut commands, &config);
}

pub(super) fn restart(
    mut commands: Commands,
    config: Res<CombatConfig>,
    mut health: ResMut<PlayerHealth>,
    mut weapon: ResMut<Weapon>,
    mut phase: ResMut<GamePhase>,
    transient: Query<Entity, CombatEntities>,
) {
    for entity in &transient {
        commands.entity(entity).despawn();
    }
    health.current = config.player_health;
    health.invulnerable_until = 0.;
    *weapon = Weapon::default();
    *phase = GamePhase::Playing;
    spawn_enemies(&mut commands, &config);
}

pub(super) fn contact_damage(
    time: Res<Time>,
    config: Res<CombatConfig>,
    drone: Single<&Transform, With<Drone>>,
    enemies: Query<(&Enemy, &Transform)>,
    mut health: ResMut<PlayerHealth>,
    mut phase: ResMut<GamePhase>,
) {
    let now = time.elapsed_secs_f64();
    if now + 1e-7 < health.invulnerable_until {
        return;
    }
    if enemies.iter().any(|(enemy, target)| {
        let half = drone_world_half_extents(drone.rotation)
            + world_half_extents(target.rotation, Vec3::splat(config.enemy_half_size))
            + Vec3::splat(0.001);
        enemy.health > 0
            && (target.translation - drone.translation)
                .abs()
                .cmple(half)
                .all()
    }) {
        health.current = health.current.saturating_sub(config.contact_damage);
        health.invulnerable_until = now + config.invulnerability;
        if health.current == 0 {
            *phase = GamePhase::Dead;
        }
    }
}
