//! One attempt-local attachment; damage and powered pulses resolve exactly once.
use super::{CombatConfig, CombatOutcomes, PlayerHealth};
use crate::{
    energy::PowerFrame,
    game::GamePhase,
    modules::{ModuleConfig, ModuleKind},
};
use bevy::prelude::*;

pub(super) const FUSE: f64 = 3.;
pub(super) const DAMAGE: u32 = 25;

#[derive(Resource, Default)]
pub(crate) struct BombState {
    pub(super) remaining: Option<f64>,
    pub(crate) pulse_cooldown: f64,
    pub(super) pulse_flash: f64,
    pub(super) notice: &'static str,
    pub(super) notice_for: f64,
}
impl BombState {
    pub(super) fn attach(&mut self) {
        if self.remaining.is_none() {
            self.remaining = Some(FUSE);
            self.notice = "BOMB ATTACHED";
            self.notice_for = FUSE;
        }
    }
}
// Age before contacts so even a hitch cannot consume a newly attached fuse.
pub(super) fn advance(time: Res<Time>, mut bomb: ResMut<BombState>) {
    let dt = time.delta_secs_f64();
    bomb.remaining = bomb.remaining.map(|remaining| (remaining - dt).max(0.));
    bomb.pulse_cooldown = (bomb.pulse_cooldown - dt).max(0.);
    bomb.pulse_flash = (bomb.pulse_flash - dt).max(0.);
    bomb.notice_for = (bomb.notice_for - dt).max(0.);
}
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve(
    time: Res<Time>,
    config: Res<CombatConfig>,
    modules: Res<ModuleConfig>,
    mut bomb: ResMut<BombState>,
    mut power: ResMut<PowerFrame>,
    mut health: ResMut<PlayerHealth>,
    mut phase: ResMut<GamePhase>,
    mut outcomes: ResMut<CombatOutcomes>,
    drone: Single<&Transform, With<crate::arena::Drone>>,
    world: Option<Res<crate::world::WorldGeometry>>,
    mut enemies: Query<
        (&super::Enemy, &Transform, &mut crate::arena::DroneFlight),
        Without<crate::arena::Drone>,
    >,
) {
    if *phase != GamePhase::Playing {
        return;
    }
    if power.modules.active(ModuleKind::Repulsor) && bomb.pulse_cooldown <= 1e-7 {
        bomb.pulse_cooldown = modules.repulsor_interval;
        bomb.pulse_flash = 0.3;
        for (enemy, transform, mut flight) in &mut enemies {
            let offset = transform.translation - drone.translation;
            if enemy.health > 0
                && offset.length_squared() <= modules.repulsor_radius.powi(2)
                && world
                    .as_deref()
                    .is_none_or(|w| w.line_clear(drone.translation, transform.translation))
            {
                // Keep displacement in the normal swept flight integrator.
                // Coincident centers receive a deterministic upward impulse.
                let outward = offset.try_normalize().unwrap_or(Vec3::Y);
                // Reverse closing motion without stacking speed on later pulses.
                let radial = flight.velocity.dot(outward);
                flight.velocity += outward * (modules.repulsor_impulse - radial).max(0.);
            }
        }
        if bomb.remaining.take().is_some() {
            bomb.notice = "BOMB DISLODGED";
            bomb.notice_for = 2.;
        }
    }
    if bomb.remaining.is_some_and(|remaining| remaining <= 1e-7) {
        bomb.remaining = None;
        let before = health.current;
        super::lifecycle::apply_player_damage(
            DAMAGE,
            time.elapsed_secs_f64(),
            &config,
            &mut health,
            &mut phase,
            &mut outcomes,
            &mut power,
            &modules,
        );
        bomb.notice = if health.current < before {
            "BOMB DETONATED: 25 HULL"
        } else {
            "BOMB BLOCKED"
        };
        bomb.notice_for = 2.;
    }
}
pub(super) fn reset(mut bomb: ResMut<BombState>) {
    *bomb = default();
}
pub(super) fn cleanup(
    phase: Res<GamePhase>,
    boundary: Option<Res<crate::game::MissionBoundary>>,
    mut bomb: ResMut<BombState>,
) {
    if matches!(*phase, GamePhase::Dead | GamePhase::Survived)
        || boundary.is_some_and(|b| b.cleanup || b.reset)
    {
        *bomb = default();
    }
}
#[cfg(test)]
#[path = "bomb_tests.rs"]
mod tests;
