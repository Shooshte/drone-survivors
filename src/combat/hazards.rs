use super::{CombatConfig, CombatOutcome, CombatOutcomes, Encounter, Enemy, PlayerHealth};
use crate::{
    arena::{Drone, drone_world_half_extents, world_half_extents},
    energy::PowerFrame,
    game::GamePhase,
    modules::ModuleConfig,
    world::{
        MotionSegment, PlayerPath, Solid, WorldGeometry,
        hazard::{ActiveWindow, HAZARD_DAMAGE, HazardState},
    },
};
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(super) struct HazardHits {
    cycle: u64,
    actors: HashSet<Entity>,
}

pub(super) fn advance(time: Res<Time>, mut state: ResMut<HazardState>) {
    state.advance(time.delta_secs());
}

/// Intersect only the piece of actual motion during the active part of this frame.
fn exposed(field: &Solid, path: &[MotionSegment], active: ActiveWindow) -> bool {
    path.iter().any(|segment| {
        let from = segment.from.max(active.from);
        let to = segment.to.min(active.to);
        if from > to {
            return false;
        }
        let span = segment.to - segment.from;
        let point = |t| {
            if span > 0. {
                segment.start.lerp(segment.end, (t - segment.from) / span)
            } else {
                segment.end
            }
        };
        field
            .segment_hit(point(from), point(to), segment.half)
            .is_some()
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn damage(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<HazardState>,
    world: Option<Res<WorldGeometry>>,
    path: Option<Res<PlayerPath>>,
    drone: Single<(Entity, &Transform), With<Drone>>,
    mut enemies: Query<(Entity, &mut Enemy, &Transform), Without<Drone>>,
    mut health: ResMut<PlayerHealth>,
    mut phase: ResMut<GamePhase>,
    mut outcomes: ResMut<CombatOutcomes>,
    mut power: ResMut<PowerFrame>,
    modules: Res<ModuleConfig>,
    config: Res<CombatConfig>,
    mut run: ResMut<Encounter>,
    mut hits: ResMut<HazardHits>,
) {
    if *phase != GamePhase::Playing {
        return;
    }
    let Some(field) = world.as_ref().and_then(|w| w.hazard.as_ref()) else {
        return;
    };
    let Some(active) = state.active_window else {
        return;
    };
    if hits.cycle != state.cycle {
        hits.cycle = state.cycle;
        hits.actors.clear();
    }
    let (id, transform) = *drone;
    let player_exposed = path
        .as_ref()
        .filter(|p| !p.segments.is_empty())
        .map_or_else(
            || {
                field
                    .segment_hit(
                        transform.translation,
                        transform.translation,
                        drone_world_half_extents(transform.rotation),
                    )
                    .is_some()
            },
            |p| exposed(field, &p.segments, active),
        );
    if !hits.actors.contains(&id)
        && player_exposed
        && super::lifecycle::apply_player_damage(
            HAZARD_DAMAGE,
            time.elapsed_secs_f64(),
            &config,
            &mut health,
            &mut phase,
            &mut outcomes,
            &mut power,
            &modules,
        )
    {
        hits.actors.insert(id);
    }
    // Resolve all actors in this hazard event before subsequent terminal systems.
    for (entity, mut enemy, transform) in &mut enemies {
        if enemy.health == 0 || hits.actors.contains(&entity) {
            continue;
        }
        let touches = if enemy.path.is_empty() {
            field
                .segment_hit(
                    transform.translation,
                    transform.translation,
                    world_half_extents(transform.rotation, Vec3::splat(config.enemy_half_size)),
                )
                .is_some()
        } else {
            exposed(field, &enemy.path, active)
        };
        if !touches {
            continue;
        }
        hits.actors.insert(entity);
        enemy.health = enemy.health.saturating_sub(HAZARD_DAMAGE);
        let killed = enemy.health == 0;
        outcomes.0.push(CombatOutcome::Hit {
            entity,
            position: transform.translation,
            killed,
        });
        if killed {
            run.kills = run.kills.saturating_add(1);
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn reset(mut state: ResMut<HazardState>, mut hits: ResMut<HazardHits>) {
    *state = HazardState::default();
    *hits = HazardHits::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossing_between_samples_counts_only_during_active_part() {
        let field = Solid {
            center: Vec3::ZERO,
            half: Vec3::splat(1.),
        };
        let crossing = [MotionSegment {
            start: Vec3::NEG_X * 10.,
            end: Vec3::X * 10.,
            from: 0.,
            to: 1.,
            half: Vec3::splat(0.5),
        }];
        assert!(exposed(
            &field,
            &crossing,
            ActiveWindow { from: 0.4, to: 0.6 }
        ));
        assert!(!exposed(
            &field,
            &crossing,
            ActiveWindow { from: 0.7, to: 1. }
        ));
        assert!(!exposed(
            &field,
            &crossing,
            ActiveWindow { from: 0., to: 0.3 }
        ));
    }

    #[test]
    fn rotation_correction_does_not_expose_actor_to_field_across_wall() {
        let world = WorldGeometry {
            solids: vec![Solid {
                center: Vec3::new(0., 150., 0.),
                half: Vec3::new(1., 150., 270.),
            }],
            hazard: None,
        };
        let field = Solid {
            center: Vec3::new(2., 150., 0.),
            half: Vec3::new(0.1, 30., 30.),
        };
        let mut transform = Transform::from_xyz(-36.1, 150., 0.);
        let mut flight = crate::arena::DroneFlight::default();
        let path = flight.step_in_world(
            &mut transform,
            &crate::arena::FlightInput {
                tilt: Vec2::X,
                yaw: 1.,
                thrust: 1.,
            },
            &crate::arena::FlightConfig {
                gravity: 0.,
                ..default()
            },
            &crate::arena::Arena::default(),
            crate::arena::DRONE_HALF_EXTENTS,
            1. / 120.,
            Some(&world),
        );
        assert!(
            transform.translation.x < -36.2,
            "fixture must correct contact"
        );
        assert!(
            !exposed(&field, &path, ActiveWindow { from: 0.1, to: 0.2 }),
            "field activated after the instantaneous correction on the other side of a wall"
        );
    }
}
