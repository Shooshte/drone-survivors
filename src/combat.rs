use crate::game::{GamePhase, GameplaySet, is_playing};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

mod collision;
mod scene;
pub(crate) use scene::CombatScenePlugin;
mod enemies;
mod lifecycle;
mod weapon;

#[cfg(test)]
mod tests;

const ENEMY_STARTS: [Vec3; 3] = [
    Vec3::new(-240., 90., 0.),
    Vec3::new(240., 90., 0.),
    Vec3::new(0., 210., -180.),
];

#[derive(Resource)]
struct CombatConfig {
    player_health: u32,
    enemy_health: u32,
    contact_damage: u32,
    shot_damage: u32,
    chase_speed: f32,
    projectile_speed: f32,
    fire_interval: f64,
    target_range: f32,
    projectile_lifetime: f32,
    enemy_half_size: f32,
    projectile_radius: f32,
    invulnerability: f64,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            player_health: 100,
            enemy_health: 40,
            contact_damage: 25,
            shot_damage: 10,
            chase_speed: 150.,
            projectile_speed: 650.,
            fire_interval: 0.5,
            target_range: 400.,
            projectile_lifetime: 1.,
            enemy_half_size: 14.,
            projectile_radius: 3.,
            invulnerability: 0.75,
        }
    }
}

#[derive(Component)]
struct Enemy {
    health: u32,
    previous: Vec3,
}

#[derive(Component)]
struct Projectile {
    velocity: Vec3,
    remaining: f32,
}

#[derive(Resource)]
struct PlayerHealth {
    current: u32,
    invulnerable_until: f64,
}

#[derive(Resource, Default)]
struct Weapon {
    ready_at: f64,
}

pub(crate) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatConfig>()
            .init_resource::<GamePhase>()
            .init_resource::<Weapon>()
            .insert_resource(PlayerHealth {
                current: 100,
                invulnerable_until: 0.,
            })
            .add_systems(Startup, lifecycle::setup)
            .add_systems(
                Update,
                lifecycle::restart
                    .in_set(GameplaySet::Reset)
                    .run_if(input_just_pressed(KeyCode::KeyR)),
            )
            .add_systems(
                Update,
                (
                    enemies::chase,
                    weapon::advance_projectiles,
                    lifecycle::contact_damage,
                    weapon::fire,
                )
                    .chain()
                    .in_set(GameplaySet::Combat)
                    .run_if(is_playing)
                    .run_if(not(input_just_pressed(KeyCode::KeyR))),
            );
    }
}
