//! Repair consumes the paid interval after damage and terminal outcomes resolve.
use super::{CombatConfig, PlayerHealth};
use crate::{
    energy::PowerFrame,
    game::{GamePhase, MissionBoundary},
    modules::{ModuleConfig, ModuleKind},
};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct RepairState {
    pub(crate) credit: f64,
}

pub(super) fn repair(
    phase: Res<GamePhase>,
    config: Res<CombatConfig>,
    tuning: Res<ModuleConfig>,
    mut power: ResMut<PowerFrame>,
    mut health: ResMut<PlayerHealth>,
    mut repair: ResMut<RepairState>,
) {
    let seconds = std::mem::take(&mut power.repair_seconds);
    if *phase != GamePhase::Playing || health.current == 0 {
        return;
    }
    if health.current >= config.player_health {
        repair.credit = 0.;
        return;
    }
    // Depletion switches enabled off, but its paid fraction still heals. A lock
    // delivered in this frame instead blocks the effect until manually re-enabled.
    let unlocked = power
        .modules
        .loadout
        .slots()
        .iter()
        .enumerate()
        .any(|(slot, kind)| {
            *kind == Some(ModuleKind::Repair) && power.modules.disabled_for[slot] <= 0.
        });
    if !unlocked || seconds <= 0. {
        return;
    }
    repair.credit += seconds * tuning.repair_rate.max(0.);
    // Nanosecond Time rounding must not lose a whole hull at common frame rates.
    let restored = (repair.credit + 1e-6).floor() as u32;
    repair.credit = (repair.credit - f64::from(restored)).max(0.);
    health.current = health
        .current
        .saturating_add(restored)
        .min(config.player_health);
    if health.current == config.player_health {
        repair.credit = 0.;
    }
}

pub(super) fn reset(mut repair: ResMut<RepairState>) {
    *repair = default();
}

pub(super) fn cleanup(
    phase: Res<GamePhase>,
    config: Res<CombatConfig>,
    health: Res<PlayerHealth>,
    boundary: Option<Res<MissionBoundary>>,
    mut repair: ResMut<RepairState>,
    mut power: ResMut<PowerFrame>,
) {
    // Collection and progression can also restore hull after powered repair.
    // Discard credit at the end of that update, before future damage can spend it.
    if health.current >= config.player_health
        || matches!(*phase, GamePhase::Dead | GamePhase::Survived)
        || boundary.is_some_and(|b| b.cleanup || b.reset)
    {
        *repair = default();
        power.repair_seconds = 0.;
    }
}
