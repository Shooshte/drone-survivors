use super::{
    CombatConfig, Enemy,
    waves::{Encounter, SpawnWarning, WaveConfig, safe, spawn_half},
};
use crate::{
    arena::{Arena, Drone, world_half_extents},
    economy::runtime::EnemyKind,
    game::GamePhase,
};
use bevy::prelude::*;

const LAUNCH_INTERVAL: f64 = 6.;
const WARNING_SECONDS: f64 = 1.2;
const RING_RADIUS: f32 = 110.;
const RING_CANDIDATES: usize = 16;

#[derive(Component)]
pub(super) struct Mothership {
    pub(super) ready_at: Option<f64>,
    pub(super) next_kind: EnemyKind,
    candidate: usize,
}

impl Default for Mothership {
    fn default() -> Self {
        Self {
            ready_at: None,
            next_kind: EnemyKind::Chaser,
            candidate: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SpawnParent(pub Entity);

pub(super) fn flight_target(position: Vec3, player: Vec3) -> Vec3 {
    if position.distance(player) <= 350. {
        position
    } else {
        player
    }
}

fn following(kind: EnemyKind) -> EnemyKind {
    match kind {
        EnemyKind::Chaser => EnemyKind::Fast,
        EnemyKind::Fast => EnemyKind::Rammer,
        EnemyKind::Rammer => EnemyKind::Slower,
        EnemyKind::Slower => EnemyKind::Jammer,
        EnemyKind::Jammer => EnemyKind::Bomber,
        EnemyKind::Bomber | EnemyKind::Mothership => EnemyKind::Chaser,
    }
}

fn ring_candidate(parent: Vec3, index: usize) -> Vec3 {
    let angle = std::f32::consts::TAU * (index % RING_CANDIDATES) as f32 / RING_CANDIDATES as f32;
    parent + Vec3::new(angle.cos() * RING_RADIUS, 0., angle.sin() * RING_RADIUS)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    mut commands: Commands,
    config: Res<WaveConfig>,
    combat: Res<CombatConfig>,
    arena: Res<Arena>,
    world: Option<Res<crate::world::WorldGeometry>>,
    player: Single<&Transform, With<Drone>>,
    phase: Res<GamePhase>,
    mut run: ResMut<Encounter>,
    mut enemies: Query<(Entity, &Enemy, &Transform, Option<&mut Mothership>)>,
    warnings: Query<(Entity, &SpawnWarning, &Transform, Option<&SpawnParent>)>,
    parent_enemies: Query<&Enemy>,
) {
    if *phase != GamePhase::Playing {
        return;
    }

    let mut occupied = Vec::new();
    let mut parents = Vec::new();
    let half = spawn_half(&combat);
    let mut live = 0;
    for (id, enemy, transform, mothership) in &mut enemies {
        if enemy.health == 0 {
            continue;
        }
        live += 1;
        occupied.push((
            id,
            transform.translation,
            world_half_extents(transform.rotation, Vec3::splat(combat.enemy_half_size)),
        ));
        if mothership.is_some() {
            parents.push(id);
        }
    }
    let mut pending = 0;
    for (id, warning, transform, parent) in &warnings {
        if warning.cancelled {
            continue;
        }
        if parent.is_some_and(|parent| {
            !parent_enemies
                .get(parent.0)
                .is_ok_and(|enemy| enemy.health > 0)
        }) {
            continue;
        }
        pending += 1;
        occupied.push((id, transform.translation, half));
    }
    parents.sort_by_key(|id| id.to_bits());

    for parent in parents {
        let Ok((_, enemy, transform, Some(mut mothership))) = enemies.get_mut(parent) else {
            continue;
        };
        if enemy.health == 0 {
            continue;
        }
        let Some(ready_at) = mothership.ready_at else {
            mothership.ready_at = Some(run.elapsed + LAUNCH_INTERVAL);
            continue;
        };
        if run.elapsed + 1e-7 < ready_at {
            continue;
        }

        mothership.ready_at = Some(run.elapsed + LAUNCH_INTERVAL);
        let kind = mothership.next_kind;
        mothership.next_kind = following(kind);
        run.spawns.requested += 1;
        if live + pending >= config.cap {
            run.spawns.rejected_cap += 1;
            continue;
        }

        let mut selected = None;
        for _ in 0..RING_CANDIDATES {
            let position = ring_candidate(transform.translation, mothership.candidate);
            mothership.candidate = (mothership.candidate + 1) % RING_CANDIDATES;
            if safe(
                position,
                half,
                &arena,
                &player,
                config.clearance,
                &occupied,
                None,
                world.as_deref(),
            ) {
                selected = Some(position);
                break;
            }
        }
        let Some(position) = selected else {
            run.spawns.rejected_space += 1;
            continue;
        };
        let warning = commands
            .spawn((
                SpawnWarning {
                    ready_at: run.elapsed + WARNING_SECONDS,
                    kind,
                    ..default()
                },
                SpawnParent(parent),
                Transform::from_translation(position),
            ))
            .id();
        occupied.push((warning, position, half));
        pending += 1;
        run.spawns.admitted += 1;
    }
}

#[cfg(test)]
#[path = "mothership_tests.rs"]
mod tests;
