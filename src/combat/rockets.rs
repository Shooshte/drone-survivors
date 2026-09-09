use super::{Enemy, Projectile};
use crate::{
    arena::Drone,
    game::GamePhase,
    modules::{ModuleConfig, Modules},
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct Rocket;

#[derive(Resource, Default)]
pub(super) struct RocketLauncher {
    pub(super) ready_at: f64,
    interval: Option<f64>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<ModuleConfig>,
    phase: Res<GamePhase>,
    modules: Res<Modules>,
    mut launcher: ResMut<RocketLauncher>,
    drone: Single<&Transform, With<Drone>>,
    enemies: Query<(Entity, &Enemy, &Transform), Without<Projectile>>,
) {
    if *phase != GamePhase::Playing {
        return;
    }
    let now = time.elapsed_secs_f64();
    let interval = config.rocket_interval;
    if let Some(previous) = launcher.interval
        && previous != interval
        && launcher.ready_at > now
    {
        launcher.ready_at = now + (launcher.ready_at - now) / previous * interval;
    }
    launcher.interval = Some(interval);

    if !modules.active(crate::modules::ModuleKind::Rocket) {
        // Elapsed gameplay consumes the cooldown without accumulating shots.
        launcher.ready_at = launcher.ready_at.max(now);
        return;
    }
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
        .filter(|(_, _, distance)| *distance <= config.rocket_range.powi(2))
        .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.to_bits().cmp(&b.0.to_bits())));
    let Some((_, position, _)) = target else {
        launcher.ready_at = launcher.ready_at.max(now);
        return;
    };
    if now + 1e-7 < launcher.ready_at {
        return;
    }
    let direction = (position - drone.translation)
        .try_normalize()
        .unwrap_or(Vec3::NEG_Z);
    commands.spawn((
        Projectile {
            velocity: direction * config.rocket_speed,
            remaining: config.rocket_lifetime,
        },
        Transform::from_translation(drone.translation),
        Rocket,
    ));
    launcher.ready_at = if now - launcher.ready_at >= interval {
        now + interval
    } else {
        launcher.ready_at + interval
    };
}
