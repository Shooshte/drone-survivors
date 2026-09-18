//! Range/line-of-sight control attacks. Only catalog rosters spawn these sources.
use super::Enemy;
use crate::{
    arena::Drone, economy::runtime::EnemyKind, energy::PowerFrame, game::GamePhase,
    modules::Modules,
};
use bevy::prelude::*;

pub(super) const RANGE: f32 = 320.;
const APPROACH_DISTANCE: f32 = 240.;
const WINDUP: f32 = 1.2;
const RECOVERY: f32 = 3.;
const BEAM_SECONDS: f32 = 2.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ControlPhase {
    #[default]
    Approach,
    Windup,
    Active,
    Recovery,
}

#[derive(Component, Default)]
pub(super) struct ControlAttack {
    pub phase: ControlPhase,
    pub elapsed: f32,
    pub slot: Option<usize>,
    anchor: Vec3,
}
impl ControlAttack {
    fn enter(&mut self, phase: ControlPhase) {
        self.phase = phase;
        self.elapsed = 0.;
    }
    pub(super) fn flight_target(
        &self,
        position: Vec3,
        player: Vec3,
        world: Option<&crate::world::WorldGeometry>,
    ) -> Vec3 {
        if matches!(self.phase, ControlPhase::Windup | ControlPhase::Active) {
            self.anchor
        } else if position.distance(player) <= APPROACH_DISTANCE
            && world.is_none_or(|w| w.line_clear(position, player))
        {
            position
        } else {
            player
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct ControlEffects {
    pub(super) slowed: bool,
}
impl ControlEffects {
    pub(crate) fn movement_multiplier(&self) -> f32 {
        if self.slowed { 0.6 } else { 1. }
    }
}

pub(super) fn reset(mut effects: ResMut<ControlEffects>) {
    *effects = default();
}

fn in_sight(source: Vec3, player: Vec3, world: Option<&crate::world::WorldGeometry>) -> bool {
    source.distance(player) <= RANGE && world.is_none_or(|w| w.line_clear(source, player))
}

/// Only already-committed beams can affect this frame's movement. A newly ready
/// warning must survive this frame's projectile and hazard resolution first.
pub(super) fn prepare(
    time: Res<Time>,
    drone: Single<&Transform, With<Drone>>,
    world: Option<Res<crate::world::WorldGeometry>>,
    mut modules: ResMut<Modules>,
    mut effects: ResMut<ControlEffects>,
    enemies: Query<(&Enemy, &Transform, &ControlAttack)>,
) {
    modules.advance_jam(time.delta_secs_f64());
    effects.slowed = enemies.iter().any(|(enemy, transform, attack)| {
        enemy.health > 0
            && enemy.kind == EnemyKind::Slower
            && attack.phase == ControlPhase::Active
            && attack.elapsed < BEAM_SECONDS
            && in_sight(transform.translation, drone.translation, world.as_deref())
    });
}

/// Commit attacks after damage/outcome resolution, into the staged power frame
/// so energy::update cannot overwrite a delivered lock. Energy accounted before
/// delivery belongs to the elapsed, unlocked interval; future frames draw none.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve(
    time: Res<Time>,
    drone: Single<&Transform, With<Drone>>,
    world: Option<Res<crate::world::WorldGeometry>>,
    phase: Res<GamePhase>,
    mut power: ResMut<PowerFrame>,
    mut effects: ResMut<ControlEffects>,
    mut enemies: Query<(Entity, &Enemy, &Transform, &mut ControlAttack)>,
) {
    let dt = time.delta_secs();
    effects.slowed = false;
    if *phase != GamePhase::Playing {
        return;
    }
    let modules = &mut power.modules;
    // Resolve simultaneous pulses deterministically, without extending a lock.
    let mut ids: Vec<_> = enemies.iter().map(|(id, _, _, _)| id).collect();
    ids.sort_by_key(|id| id.to_bits());
    for id in ids {
        let Ok((_, enemy, transform, mut attack)) = enemies.get_mut(id) else {
            continue;
        };
        if enemy.health == 0 {
            continue;
        }
        let jammer = enemy.kind == EnemyKind::Jammer;
        let in_sight = in_sight(transform.translation, drone.translation, world.as_deref());
        match attack.phase {
            ControlPhase::Approach => {
                let slot = if jammer {
                    (0..4)
                        .filter(|&i| modules.loadout.slots()[i].is_some())
                        .min_by_key(|&i| (!modules.enabled[i], i))
                } else {
                    None
                };
                if in_sight
                    && transform.translation.distance(drone.translation) <= APPROACH_DISTANCE
                    && (!jammer || (slot.is_some() && modules.can_be_jammed()))
                {
                    attack.slot = slot;
                    attack.anchor = transform.translation;
                    attack.enter(ControlPhase::Windup);
                }
            }
            ControlPhase::Windup => {
                if !in_sight {
                    attack.enter(ControlPhase::Recovery);
                } else if attack.elapsed + 1e-6 >= WINDUP {
                    if !jammer || attack.slot.is_some_and(|slot| modules.jam(slot)) {
                        attack.enter(ControlPhase::Active);
                    } else {
                        attack.enter(ControlPhase::Recovery);
                    }
                } else {
                    attack.elapsed += dt;
                }
            }
            ControlPhase::Active => {
                attack.elapsed += dt;
                let duration = if jammer { 0.4 } else { BEAM_SECONDS };
                if (!jammer && !in_sight) || attack.elapsed + 1e-6 >= duration {
                    attack.enter(ControlPhase::Recovery);
                }
            }
            ControlPhase::Recovery => {
                attack.elapsed += dt;
                if attack.elapsed + 1e-6 >= RECOVERY {
                    attack.enter(ControlPhase::Approach);
                }
            }
        }
        if !jammer && attack.phase == ControlPhase::Active {
            effects.slowed = true;
        }
    }
}

#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
