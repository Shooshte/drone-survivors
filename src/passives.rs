//! Deterministic campaign purchases, independent of UI and encounter state.
use crate::economy::Amounts;

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
