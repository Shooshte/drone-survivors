//! Deterministic campaign purchases, independent of UI and encounter state.
use crate::economy::Amounts;
pub(crate) mod scene;
pub(crate) mod validation;
pub(crate) const PURCHASE_KEYS: [bevy::prelude::KeyCode; 9] = [
    bevy::prelude::KeyCode::Digit1,
    bevy::prelude::KeyCode::Digit2,
    bevy::prelude::KeyCode::Digit3,
    bevy::prelude::KeyCode::Digit4,
    bevy::prelude::KeyCode::Digit5,
    bevy::prelude::KeyCode::Digit6,
    bevy::prelude::KeyCode::Digit7,
    bevy::prelude::KeyCode::Digit8,
    bevy::prelude::KeyCode::Digit9,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NodeId {
    Damage,
    FireRate,
    Range,
    Hull,
    Armor,
    Protection,
    Battery,
    Reserve,
    Charging,
}
impl NodeId {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Damage => "Projectile damage",
            Self::FireRate => "Fire rate",
            Self::Range => "Targeting range",
            Self::Hull => "Hull capacity",
            Self::Armor => "Contact armor",
            Self::Protection => "Post-hit protection",
            Self::Battery => "Battery capacity",
            Self::Reserve => "Reserve efficiency",
            Self::Charging => "Charging speed",
        }
    }
    pub(crate) fn prerequisite(self) -> Option<Self> {
        match self {
            Self::Damage | Self::Hull | Self::Battery => None,
            _ => Some(Self::ALL[self as usize - 1]),
        }
    }
    pub(crate) fn percent_per_rank(self) -> u32 {
        match self {
            Self::Hull | Self::Battery | Self::Charging => 20,
            Self::Reserve => 5,
            _ => 10,
        }
    }
    pub(crate) const ALL: [Self; 9] = [
        Self::Damage,
        Self::FireRate,
        Self::Range,
        Self::Hull,
        Self::Armor,
        Self::Protection,
        Self::Battery,
        Self::Reserve,
        Self::Charging,
    ];
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PassiveTree {
    ranks: [u8; 9],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PurchaseError {
    Locked(NodeId),
    Maxed,
    InsufficientFunds,
}
impl PassiveTree {
    pub(crate) fn from_ranks(ranks: [u8; 9]) -> Result<Self, &'static str> {
        let tree = Self { ranks };
        for node in NodeId::ALL {
            if tree.rank(node) > 5 {
                return Err("passive rank exceeds five");
            }
            if tree.rank(node) > 0
                && node
                    .prerequisite()
                    .is_some_and(|previous| tree.rank(previous) == 0)
            {
                return Err("passive prerequisite is missing");
            }
        }
        Ok(tree)
    }

    /// Called only on fresh copies of pristine tuning, before run-local modifiers.
    pub(crate) fn apply(
        &self,
        combat: &mut crate::combat::CombatConfig,
        energy: &mut crate::energy::EnergyConfig,
    ) {
        let increase = |base: u32, node| {
            ((u64::from(base) * u64::from(100 + self.bonus_percent(node))) / 100)
                .min(u64::from(u32::MAX)) as u32
        };
        let factor = |node| 1. + f64::from(self.bonus_percent(node)) / 100.;
        combat.shot_damage = increase(combat.shot_damage, NodeId::Damage);
        combat.player_health = increase(combat.player_health, NodeId::Hull);
        combat.fire_interval /= factor(NodeId::FireRate);
        combat.target_range *= factor(NodeId::Range) as f32;
        combat.projectile_lifetime *= factor(NodeId::Range) as f32;
        combat.contact_damage = (u64::from(combat.contact_damage)
            * u64::from(100 - self.bonus_percent(NodeId::Armor)))
        .div_ceil(100) as u32;
        combat.invulnerability *= factor(NodeId::Protection);
        energy.capacity *= factor(NodeId::Battery);
        energy.recharge *= factor(NodeId::Charging);
        energy.reserve_cost *= 1. - f64::from(self.bonus_percent(NodeId::Reserve)) / 100.;
    }
    pub(crate) fn rank(&self, node: NodeId) -> u8 {
        self.ranks[node as usize]
    }
    pub(crate) fn next_cost(&self, node: NodeId) -> Option<Amounts> {
        let rank = self.rank(node);
        (rank < 5).then(|| Amounts {
            salvage: 10 * u64::from(rank + 1),
            components: [0, 0, 1, 1, 2][rank as usize],
        })
    }
    pub(crate) fn availability(&self, node: NodeId, wallet: &Amounts) -> Result<(), PurchaseError> {
        let cost = self.next_cost(node).ok_or(PurchaseError::Maxed)?;
        if let Some(previous) = node.prerequisite()
            && self.rank(previous) == 0
        {
            return Err(PurchaseError::Locked(previous));
        }
        if wallet.salvage < cost.salvage || wallet.components < cost.components {
            return Err(PurchaseError::InsufficientFunds);
        }
        Ok(())
    }
    pub(crate) fn purchase(
        &mut self,
        node: NodeId,
        wallet: &mut Amounts,
    ) -> Result<u8, PurchaseError> {
        self.availability(node, wallet)?;
        wallet
            .try_spend(self.next_cost(node).ok_or(PurchaseError::Maxed)?)
            .map_err(|_| PurchaseError::InsufficientFunds)?;
        self.ranks[node as usize] += 1;
        Ok(self.rank(node))
    }
    pub(crate) fn bonus_percent(&self, node: NodeId) -> u32 {
        u32::from(self.rank(node)) * node.percent_per_rank()
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod lifecycle_tests;

#[cfg(test)]
mod input_tests;
