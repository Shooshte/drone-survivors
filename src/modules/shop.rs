//! Campaign module catalog, ownership, and editable launch loadout.
use std::collections::BTreeSet;

use crate::{
    economy::Amounts,
    modules::{Loadout, ModuleKind},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShopError {
    AlreadyOwned(ModuleKind),
    CatalogOnly,
    InsufficientFunds,
    NotOwned(ModuleKind),
    InvalidSlot(usize),
    InvalidLoadout(&'static str),
}

/// Prices are provisional and intentionally live with the catalog.
pub(crate) fn price(kind: ModuleKind) -> Amounts {
    match kind {
        ModuleKind::Overdrive => Amounts {
            salvage: 10,
            components: 0,
        },
        ModuleKind::Shield | ModuleKind::Mobility => Amounts {
            salvage: 15,
            components: 0,
        },
        ModuleKind::Repulsor | ModuleKind::Repair => Amounts::default(),
        ModuleKind::Rocket => Amounts {
            salvage: 25,
            components: 1,
        },
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ModuleInventory {
    owned: BTreeSet<ModuleKind>,
    loadout: Loadout,
}

impl Default for ModuleInventory {
    fn default() -> Self {
        Self {
            owned: BTreeSet::new(),
            loadout: Loadout::new([None; 4]).expect("empty loadout is valid"),
        }
    }
}

impl ModuleInventory {
    pub(crate) fn from_saved(
        owned: Vec<ModuleKind>,
        slots: [Option<ModuleKind>; 4],
    ) -> Result<Self, &'static str> {
        if owned.iter().any(|kind| !ModuleKind::ALL.contains(kind)) {
            return Err("catalog-only module");
        }
        let count = owned.len();
        let owned: BTreeSet<_> = owned.into_iter().collect();
        if owned.len() != count {
            return Err("duplicate module ownership");
        }
        let inventory = Self {
            owned,
            loadout: Loadout::new(slots)?,
        };
        inventory
            .validate()
            .map_err(|_| "equipped module is not owned")?;
        Ok(inventory)
    }

    pub(crate) fn owns(&self, kind: ModuleKind) -> bool {
        self.owned.contains(&kind)
    }

    pub(crate) fn purchase(
        &mut self,
        kind: ModuleKind,
        bank: &mut Amounts,
    ) -> Result<(), ShopError> {
        if !ModuleKind::ALL.contains(&kind) {
            return Err(ShopError::CatalogOnly);
        }
        if self.owns(kind) {
            return Err(ShopError::AlreadyOwned(kind));
        }
        bank.try_spend(price(kind))
            .map_err(|_| ShopError::InsufficientFunds)?;
        self.owned.insert(kind);
        Ok(())
    }

    pub(crate) fn assign(
        &mut self,
        slot: usize,
        kind: Option<ModuleKind>,
    ) -> Result<(), ShopError> {
        if slot >= self.loadout.slots().len() {
            return Err(ShopError::InvalidSlot(slot));
        }
        if let Some(kind) = kind
            && !self.owns(kind)
        {
            return Err(ShopError::NotOwned(kind));
        }

        let mut slots = *self.loadout.slots();
        if let Some(kind) = kind {
            for equipped in &mut slots {
                if *equipped == Some(kind) {
                    *equipped = None;
                }
            }
        }
        slots[slot] = kind;
        self.loadout = Loadout::new(slots).map_err(ShopError::InvalidLoadout)?;
        Ok(())
    }

    pub(crate) fn loadout(&self) -> &Loadout {
        &self.loadout
    }

    pub(crate) fn validate(&self) -> Result<(), ShopError> {
        let loadout = Loadout::new(*self.loadout.slots()).map_err(ShopError::InvalidLoadout)?;
        if let Some(kind) = loadout
            .slots()
            .iter()
            .flatten()
            .copied()
            .find(|&kind| !self.owns(kind))
        {
            return Err(ShopError::NotOwned(kind));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
