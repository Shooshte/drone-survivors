pub(crate) mod runtime;
pub(crate) mod scene;
use bevy::prelude::*;

use crate::modules::{Loadout, ModuleKind};

#[derive(Resource)]
pub(crate) struct ExperienceConfig {
    pub kill_xp: u32,
    pub early_kill_xp: u32,
    pub reduction_starts_at: f64,
}

impl Default for ExperienceConfig {
    fn default() -> Self {
        Self {
            kill_xp: 4,
            early_kill_xp: 4,
            reduction_starts_at: 105.,
        }
    }
}

impl ExperienceConfig {
    pub(crate) fn kill_xp_at(&self, elapsed: f64) -> u32 {
        if elapsed < self.reduction_starts_at {
            self.early_kill_xp
        } else {
            self.kill_xp
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpgradeKind {
    Interceptor,
    AgileFrame,
    HeavyArmor,
    HeavyRounds,
    RapidShield,
    WideAreaRockets,
}

impl UpgradeKind {
    pub(crate) const ALL: [Self; 6] = [
        Self::Interceptor,
        Self::AgileFrame,
        Self::HeavyArmor,
        Self::HeavyRounds,
        Self::RapidShield,
        Self::WideAreaRockets,
    ];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Interceptor => "Interceptor",
            Self::AgileFrame => "Agile frame",
            Self::HeavyArmor => "Heavy armor",
            Self::HeavyRounds => "Heavy rounds",
            Self::RapidShield => "Rapid shield",
            Self::WideAreaRockets => "Wide-area rockets",
        }
    }

    pub(crate) const fn benefit(self) -> &'static str {
        match self {
            Self::Interceptor => "+30% horizontal acceleration and maximum horizontal speed",
            Self::AgileFrame => "+40% turn, tilt, and automatic leveling response",
            Self::HeavyArmor => "+50 maximum hull",
            Self::HeavyRounds => "Double basic projectile damage: 10 -> 20",
            Self::RapidShield => "Powered shield recharge falls from 5s to 2.5s",
            Self::WideAreaRockets => "Explosion radius grows from 70 to 105 world units",
        }
    }

    pub(crate) const fn drawback(self) -> &'static str {
        match self {
            Self::Interceptor => "Battery capacity falls from 100 to 75",
            Self::AgileFrame => "Maximum hull falls from 100 to 80",
            Self::HeavyArmor => "-25% acceleration in every movement direction",
            Self::HeavyRounds => "-10% acceleration in every movement direction",
            Self::RapidShield => "Shield drain rises from 8 to 12 energy/s",
            Self::WideAreaRockets => "Rocket launch interval increases from 2s to 3s",
        }
    }

    fn eligible(self, loadout: &Loadout) -> bool {
        let prerequisite = match self {
            Self::RapidShield => Some(ModuleKind::Shield),
            Self::WideAreaRockets => Some(ModuleKind::Rocket),
            _ => None,
        };
        prerequisite.is_none_or(|kind| loadout.slots().contains(&Some(kind)))
    }
}

// Individual costs; cumulative requirements are 50 / 200 / 500 / 1,000 XP.
// XP alone unlocks opportunities; active time never gates progression.
const OPPORTUNITY_COSTS: [u32; 4] = [50, 150, 300, 500];
pub(crate) const OPPORTUNITY_LIMIT: u32 = OPPORTUNITY_COSTS.len() as u32;

#[derive(Resource, Debug)]
pub(crate) struct UpgradeRun {
    pub level: u32,
    pub xp: u32,
    pub pending: u32,
    pub resolved: u32,
    pub total_xp: u32,
    pub selected: Vec<UpgradeKind>,
    pub offer: Vec<UpgradeKind>,
    pub exhausted: bool,
    rng: u64,
}

impl Default for UpgradeRun {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            pending: 0,
            resolved: 0,
            total_xp: 0,
            selected: Vec::new(),
            offer: Vec::new(),
            exhausted: false,
            rng: 0x4d59_5df4_d0f3_3173,
        }
    }
}

impl UpgradeRun {
    pub(crate) fn threshold(&self) -> u32 {
        OPPORTUNITY_COSTS
            .get(self.level.saturating_sub(1) as usize)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn remaining(&self) -> u32 {
        if self.exhausted {
            0
        } else {
            OPPORTUNITY_LIMIT.saturating_sub(self.resolved)
        }
    }

    pub(crate) fn award(&mut self, amount: u32) {
        self.total_xp = self.total_xp.saturating_add(amount);
        let mut available = u64::from(self.xp) + u64::from(amount);
        // At most four iterations, even for a maximal award. After completion
        // retain XP as a statistic without earning levels or promising rewards.
        while !self.exhausted && self.level <= OPPORTUNITY_LIMIT {
            let cost = u64::from(self.threshold());
            if available < cost {
                break;
            }
            available -= cost;
            self.level += 1;
            self.pending += 1;
        }
        self.xp = available.min(u64::from(u32::MAX)) as u32;
    }

    pub(crate) fn prepare_offer(&mut self, loadout: &Loadout) {
        if !self.offer.is_empty() || self.exhausted {
            return;
        }

        let mut eligible: Vec<_> = UpgradeKind::ALL
            .into_iter()
            .filter(|kind| !self.selected.contains(kind) && kind.eligible(loadout))
            .collect();
        if eligible.is_empty() {
            self.offer.clear();
            self.pending = 0;
            self.exhausted = true;
            return;
        }

        // Completion must be visible immediately after the last selection,
        // without making the player earn another level to discover it.
        if self.pending == 0 {
            return;
        }
        let count = eligible.len().min(3);
        for index in 0..count {
            let sampled = index + self.random_index(eligible.len() - index);
            eligible.swap(index, sampled);
        }
        self.offer.extend_from_slice(&eligible[..count]);
    }

    pub(crate) fn resolve(&mut self, index: Option<usize>) -> bool {
        if self.offer.is_empty() || self.exhausted || self.pending == 0 {
            return false;
        }
        let selected = match index {
            Some(index) => match self.offer.get(index).copied() {
                Some(kind) => Some(kind),
                None => return false,
            },
            None => None,
        };
        if let Some(kind) = selected {
            self.selected.push(kind);
        }
        self.pending -= 1;
        self.resolved += 1;
        self.offer.clear();
        if self.resolved == OPPORTUNITY_LIMIT {
            self.pending = 0;
            self.exhausted = true;
        }
        true
    }

    fn random_index(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        ((self.rng.wrapping_mul(0x2545_f491_4f6c_dd1d)) % upper as u64) as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UpgradeModifiers {
    pub horizontal_acceleration: f32,
    pub speed: f32,
    pub handling: f32,
    pub acceleration: f32,
    pub hull_delta: i32,
    pub capacity: f64,
    pub shot_damage: u32,
    pub shield_recharge: f64,
    pub shield_drain: f64,
    pub rocket_radius: f32,
    pub rocket_interval: f64,
}

impl UpgradeModifiers {
    pub(crate) fn from_selected(selected: &[UpgradeKind]) -> Self {
        let mut modifiers = Self {
            horizontal_acceleration: 1.,
            speed: 1.,
            handling: 1.,
            acceleration: 1.,
            hull_delta: 0,
            capacity: 1.,
            shot_damage: 1,
            shield_recharge: 1.,
            shield_drain: 1.,
            rocket_radius: 1.,
            rocket_interval: 1.,
        };
        for kind in selected {
            match kind {
                UpgradeKind::Interceptor => {
                    modifiers.horizontal_acceleration *= 1.3;
                    modifiers.speed *= 1.3;
                    modifiers.capacity *= 0.75;
                }
                UpgradeKind::AgileFrame => {
                    modifiers.handling *= 1.4;
                    modifiers.hull_delta -= 20;
                }
                UpgradeKind::HeavyArmor => {
                    modifiers.acceleration *= 0.75;
                    modifiers.hull_delta += 50;
                }
                UpgradeKind::HeavyRounds => {
                    modifiers.acceleration *= 0.9;
                    modifiers.shot_damage *= 2;
                }
                UpgradeKind::RapidShield => {
                    modifiers.shield_recharge *= 0.5;
                    modifiers.shield_drain *= 1.5;
                }
                UpgradeKind::WideAreaRockets => {
                    modifiers.rocket_radius *= 1.5;
                    modifiers.rocket_interval *= 1.5;
                }
            }
        }
        modifiers
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChoiceAction {
    Pick(usize),
    Skip,
}

#[cfg(test)]
mod tests;
