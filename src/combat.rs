use crate::game::{GamePhase, GameplaySet, is_playing};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

mod collision;
mod scene;
pub(crate) use scene::{CombatHudRoot, CombatScenePlugin, CombatSceneSetup};
mod enemies;
mod feedback;
use feedback::{CombatOutcome, CombatOutcomes};
mod lifecycle;
mod rockets;
pub(crate) use rockets::RocketLauncher;
pub(crate) mod validation;
mod waves;
mod weapon;
pub(crate) use waves::Encounter;
use waves::{SpawnWarning, WaveConfig};

#[cfg(test)]
mod tests;

#[derive(Resource, Clone)]
pub(crate) struct CombatConfig {
    pub(crate) player_health: u32,
    enemy_health: u32,
    contact_damage: u32,
    pub(crate) shot_damage: u32,
    enemy_flight: crate::arena::FlightConfig,
    projectile_speed: f32,
    fire_interval: f64,
    target_range: f32,
    projectile_lifetime: f32,
    enemy_half_size: f32,
    projectile_radius: f32,
    invulnerability: f64,
    separation_radius: f32,
    separation_acceleration: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            player_health: 100,
            enemy_health: 20,
            contact_damage: 10,
            shot_damage: 10,
            enemy_flight: crate::arena::FlightConfig {
                max_horizontal_speed: 260.,
                max_tilt: 20_f32.to_radians(),
                tilt_rate: 100_f32.to_radians(),
                leveling_rate: 150_f32.to_radians(),
                yaw_rate: 120_f32.to_radians(),
                reduced_thrust: 0.5,
                boost_thrust: 2.,
                ..default()
            },
            projectile_speed: 650.,
            fire_interval: 0.5,
            target_range: 400.,
            projectile_lifetime: 1.,
            enemy_half_size: 14.,
            projectile_radius: 3.,
            invulnerability: 0.75,
            separation_radius: 65.,
            separation_acceleration: 160.,
        }
    }
}

#[derive(Component)]
struct Enemy {
    health: u32,
    previous: Vec3,
    /// Bounded physics trajectory for moving-target projectile sweeps.
    path: Vec<enemies::FlightSegment>,
}

#[derive(Component)]
struct Projectile {
    velocity: Vec3,
    remaining: f32,
}

#[derive(Resource)]
pub(crate) struct PlayerHealth {
    pub(crate) current: u32,
    invulnerable_until: f64,
}

#[derive(Component, Clone, Copy)]
struct ShotPayload {
    damage: u32,
    radius: f32,
}

#[derive(Resource, Default)]
struct Weapon {
    ready_at: f64,
    interval: Option<f64>,
}

pub(crate) struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::energy::EnergyPlugin)
            .init_resource::<CombatConfig>()
            .init_resource::<GamePhase>()
            .init_resource::<Weapon>()
            .init_resource::<rockets::RocketLauncher>()
            .init_resource::<WaveConfig>()
            .init_resource::<Encounter>()
            .init_resource::<CombatOutcomes>()
            .insert_resource(PlayerHealth {
                current: 100,
                invulnerable_until: 0.,
            })
            .add_systems(Startup, lifecycle::setup)
            .add_systems(Update, feedback::clear.in_set(GameplaySet::Reset))
            .add_systems(
                Update,
                lifecycle::restart
                    .in_set(GameplaySet::Reset)
                    .run_if(input_just_pressed(KeyCode::KeyR)),
            )
            .add_systems(
                Update,
                (
                    waves::advance_clock,
                    enemies::chase,
                    weapon::advance_projectiles,
                    crate::energy::prepare,
                    lifecycle::contact_damage,
                    waves::finish,
                    waves::update,
                    crate::energy::update,
                    weapon::fire,
                    rockets::fire,
                )
                    .chain()
                    .in_set(GameplaySet::Combat)
                    .run_if(is_playing)
                    .run_if(not(input_just_pressed(KeyCode::KeyR))),
            )
            .add_systems(
                Update,
                waves::update
                    .in_set(GameplaySet::Combat)
                    .run_if(not(is_playing)),
            );
    }
}
