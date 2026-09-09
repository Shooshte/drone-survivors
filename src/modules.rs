//! Equipped module state; battery accounting stays in EnergyPlugin.
use bevy::prelude::*;

pub(crate) const SLOT_KEYS: [KeyCode; 4] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModuleKind {
    Overdrive,
    Shield,
    Mobility,
    Rocket,
}
impl ModuleKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Overdrive => "OVERDRIVE",
            Self::Shield => "SHIELD",
            Self::Mobility => "MOBILITY",
            Self::Rocket => "ROCKETS",
        }
    }
}

#[derive(Resource)]
pub(crate) struct ModuleConfig {
    pub drains: [f64; 4],
    pub overdrive_multiplier: f64,
    pub shield_blocks: u32,
    pub shield_recharge: f64,
    pub mobility_multiplier: f32,
    pub rocket_interval: f64,
    pub rocket_damage: u32,
    pub rocket_radius: f32,
    pub rocket_speed: f32,
    pub rocket_lifetime: f32,
    pub rocket_range: f32,
}
impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            drains: [10., 8., 8., 10.],
            overdrive_multiplier: 2.,
            shield_blocks: 1,
            shield_recharge: 5.,
            mobility_multiplier: 1.25,
            rocket_interval: 2.,
            rocket_damage: 20,
            rocket_radius: 70.,
            rocket_speed: 500.,
            rocket_lifetime: 1.5,
            rocket_range: 400.,
        }
    }
}
impl ModuleConfig {
    pub fn drain(&self, kind: ModuleKind) -> f64 {
        self.drains[kind as usize]
    }
}

/// Validated at construction; positions carry no special module semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Loadout([Option<ModuleKind>; 4]);
impl Loadout {
    pub fn new(slots: [Option<ModuleKind>; 4]) -> Result<Self, &'static str> {
        for (index, kind) in slots.iter().enumerate() {
            if kind.is_some() && slots[..index].contains(kind) {
                return Err("A module type can occupy only one slot");
            }
        }
        Ok(Self(slots))
    }
    pub fn slots(&self) -> &[Option<ModuleKind>; 4] {
        &self.0
    }
}
impl Default for Loadout {
    fn default() -> Self {
        Self::new([
            Some(ModuleKind::Overdrive),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Mobility),
            Some(ModuleKind::Rocket),
        ])
        .expect("unique starter modules")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Shield {
    pub blocks: u32,
    pub remaining: f64,
}

#[derive(Resource, Clone, Debug)]
pub(crate) struct Modules {
    pub loadout: Loadout,
    pub enabled: [bool; 4],
    pub rejected_for: [f64; 4],
    pub shield: Shield,
}
impl Default for Modules {
    fn default() -> Self {
        Self::new(Loadout::default(), &ModuleConfig::default())
    }
}
impl Modules {
    pub fn new(loadout: Loadout, config: &ModuleConfig) -> Self {
        Self {
            loadout,
            enabled: [false; 4],
            rejected_for: [0.; 4],
            shield: Shield {
                blocks: config.shield_blocks,
                remaining: 0.,
            },
        }
    }
    pub fn active(&self, kind: ModuleKind) -> bool {
        self.loadout
            .0
            .iter()
            .zip(self.enabled)
            .any(|(slot, on)| *slot == Some(kind) && on)
    }
    pub fn drain(&self, config: &ModuleConfig) -> f64 {
        self.loadout
            .0
            .iter()
            .zip(self.enabled)
            .filter_map(|(kind, on)| kind.filter(|_| on))
            .map(|kind| config.drain(kind))
            .sum::<f64>()
            .max(0.)
    }
    pub fn toggle(&mut self, keys: &ButtonInput<KeyCode>, energy: f64, activation: f64, dt: f64) {
        for (index, key) in SLOT_KEYS.into_iter().enumerate() {
            self.rejected_for[index] = (self.rejected_for[index] - dt).max(0.);
            if self.loadout.0[index].is_none() {
                self.enabled[index] = false;
                continue;
            }
            if keys.just_pressed(key) {
                if self.enabled[index] {
                    self.enabled[index] = false;
                } else if energy >= activation {
                    self.enabled[index] = true;
                    self.rejected_for[index] = 0.;
                } else {
                    self.rejected_for[index] = 1.5;
                }
            }
        }
    }
    pub fn recharge_shield(&mut self, seconds: f64, config: &ModuleConfig) {
        if self.active(ModuleKind::Shield) && self.shield.blocks == 0 {
            self.shield.remaining = (self.shield.remaining - seconds).max(0.);
            if self.shield.remaining <= 1e-7 {
                self.shield.remaining = 0.;
                self.shield.blocks = config.shield_blocks;
            }
        }
    }
    pub fn block(&mut self, config: &ModuleConfig) -> bool {
        if !self.active(ModuleKind::Shield) || self.shield.blocks == 0 {
            return false;
        }
        self.shield.blocks -= 1;
        if self.shield.blocks == 0 {
            self.shield.remaining = config.shield_recharge;
        }
        true
    }
}

pub(crate) mod scene;

#[cfg(test)]
mod tests;
