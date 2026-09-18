//! Per-type enemy behavior; the roster is installed only by the catalog arena.
use crate::economy::runtime::EnemyKind;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct SpawnRoster(pub Vec<EnemyKind>);

impl SpawnRoster {
    pub(super) fn kind(&self, index: usize) -> EnemyKind {
        if self.0.is_empty() {
            EnemyKind::Chaser
        } else {
            self.0[index % self.0.len()]
        }
    }
}

#[derive(Clone)]
pub(super) struct VariantConfig {
    pub fast_speed: f32,
    pub rammer_health: u32,
    pub charge_speed: f32,
    pub windup_seconds: f32,
    pub charge_seconds: f32,
    pub retreat_seconds: f32,
    pub engage_range: f32,
    pub retreat_distance: f32,
    pub rearm_distance: f32,
}

impl Default for VariantConfig {
    fn default() -> Self {
        Self {
            fast_speed: 350.,
            rammer_health: 80,
            charge_speed: 420.,
            windup_seconds: 1.,
            charge_seconds: 3.0,
            retreat_seconds: 1.2,
            engage_range: 280.,
            retreat_distance: 240.,
            rearm_distance: 180.,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RamPhase {
    #[default]
    Approach,
    Windup,
    Charge,
    Retreat,
}

#[derive(Component, Default)]
pub(super) struct Rammer {
    pub phase: RamPhase,
    pub impacts: u8,
    pub elapsed: f32,
    pub target: Vec3,
    anchor: Vec3,
    retreat_goal: Option<Vec3>,
}

impl Rammer {
    fn warn(&mut self, position: Vec3, player: Vec3, config: &VariantConfig) {
        self.phase = RamPhase::Windup;
        self.elapsed = 0.;
        self.anchor = position;
        let direction = (player - position).try_normalize().unwrap_or(Vec3::X);
        self.target = player + direction * config.retreat_distance;
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn plan(
        &mut self,
        position: Vec3,
        player: Vec3,
        dt: f32,
        config: &VariantConfig,
        half: Vec3,
        arena: &crate::arena::Arena,
        world: Option<&crate::world::WorldGeometry>,
    ) -> Vec3 {
        match self.phase {
            RamPhase::Approach => {
                if position.distance(player) <= config.engage_range
                    && world.is_none_or(|w| w.line_clear(position, player))
                {
                    self.warn(position, player, config);
                    position
                } else {
                    player
                }
            }
            RamPhase::Windup => {
                // Test the elapsed time before adding dt: a hitch never spends
                // an entire newly shown warning and charge in the same update.
                if self.elapsed + 1e-6 >= config.windup_seconds {
                    self.phase = RamPhase::Charge;
                    self.elapsed = 0.;
                    self.target
                } else {
                    self.elapsed += dt;
                    self.anchor
                }
            }
            RamPhase::Charge => {
                if self.elapsed >= config.charge_seconds {
                    self.impact(false);
                    position
                } else {
                    self.elapsed += dt;
                    self.target
                }
            }
            RamPhase::Retreat => {
                self.elapsed += dt;
                if self.elapsed + 1e-6 >= config.retreat_seconds
                    && position.distance(player) >= config.rearm_distance
                {
                    if position.distance(player) <= config.engage_range
                        && world.is_none_or(|w| w.line_clear(position, player))
                    {
                        self.warn(position, player, config);
                        position
                    } else {
                        self.phase = RamPhase::Approach;
                        player
                    }
                } else {
                    // Keep a stable flight target, but keep escaping if the
                    // player follows us to it before separation allows rearming.
                    if self
                        .retreat_goal
                        .is_some_and(|goal| position.distance(goal) < 30.)
                    {
                        self.retreat_goal = None;
                    }
                    *self.retreat_goal.get_or_insert_with(|| {
                        retreat_target(
                            position,
                            player,
                            config.retreat_distance,
                            half,
                            arena,
                            world,
                        )
                    })
                }
            }
        }
    }

    /// An invulnerable collision still ends the charge, without spending a hit.
    pub(super) fn impact(&mut self, accepted: bool) -> bool {
        if self.phase != RamPhase::Charge {
            return false;
        }
        if accepted {
            self.impacts = self.impacts.saturating_add(1).min(2);
        }
        self.phase = RamPhase::Retreat;
        self.elapsed = 0.;
        self.retreat_goal = None;
        self.impacts == 2
    }
}

fn retreat_target(
    position: Vec3,
    player: Vec3,
    distance: f32,
    half: Vec3,
    arena: &crate::arena::Arena,
    world: Option<&crate::world::WorldGeometry>,
) -> Vec3 {
    let away = (position - player)
        .with_y(0.)
        .try_normalize()
        .unwrap_or(Vec3::X);
    let min = arena.center() - arena.half_size + half;
    let max = arena.center() + arena.half_size - half;
    (0..8)
        .map(|i| {
            let direction = Quat::from_rotation_y(i as f32 * std::f32::consts::FRAC_PI_4) * away;
            (position + direction * distance).clamp(min, max)
        })
        .filter(|point| {
            world.is_none_or(|w| {
                !w.solids.iter().any(|s| s.overlaps(*point, half))
                    && crate::world::navigation::next_point(w, position, *point, half).is_some()
            })
        })
        .max_by(|a, b| {
            a.distance_squared(player)
                .total_cmp(&b.distance_squared(player))
        })
        .unwrap_or(position)
}

#[cfg(test)]
#[path = "variant_tests.rs"]
mod tests;
