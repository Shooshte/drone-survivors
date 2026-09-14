//! Whole-number campaign accounting, independent of pickup and menu systems.
use bevy::prelude::*;
pub(crate) mod runtime;
pub(crate) mod scene;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Amounts {
    pub salvage: u64,
    pub components: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransactionError {
    InsufficientFunds,
    Overflow,
}
impl Amounts {
    pub(crate) fn try_credit(&mut self, amount: Self) -> Result<(), TransactionError> {
        let salvage = self
            .salvage
            .checked_add(amount.salvage)
            .ok_or(TransactionError::Overflow)?;
        let components = self
            .components
            .checked_add(amount.components)
            .ok_or(TransactionError::Overflow)?;
        *self = Self {
            salvage,
            components,
        };
        Ok(())
    }
    // Passive/shop purchases share this all-or-nothing transaction.
    pub(crate) fn try_spend(&mut self, cost: Self) -> Result<(), TransactionError> {
        let salvage = self
            .salvage
            .checked_sub(cost.salvage)
            .ok_or(TransactionError::InsufficientFunds)?;
        let components = self
            .components
            .checked_sub(cost.components)
            .ok_or(TransactionError::InsufficientFunds)?;
        *self = Self {
            salvage,
            components,
        };
        Ok(())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RewardReceipt {
    pub collected: Amounts,
    pub lost: Amounts,
    pub bonus: Amounts,
    pub credited: Amounts,
    pub balance: Amounts,
    pub error: Option<TransactionError>,
}
impl RewardReceipt {
    pub(crate) fn settle(collected: Amounts, succeeded: bool, wallet: &mut Amounts) -> Self {
        let retained = if succeeded {
            collected
        } else {
            Amounts {
                salvage: collected.salvage / 4,
                components: collected.components / 4,
            }
        };
        let bonus = if succeeded {
            Amounts {
                salvage: 10,
                components: 1,
            }
        } else {
            Amounts::default()
        };
        let mut receipt = Self {
            collected,
            lost: Amounts {
                salvage: collected.salvage - retained.salvage,
                components: collected.components - retained.components,
            },
            bonus,
            balance: *wallet,
            ..default()
        };
        let mut credit = retained;
        match credit
            .try_credit(bonus)
            .and_then(|()| wallet.try_credit(credit))
        {
            Ok(()) => {
                receipt.credited = credit;
                receipt.balance = *wallet;
            }
            Err(error) => receipt.error = Some(error),
        }
        receipt
    }
}
#[derive(Resource, Default)]
pub(crate) struct AttemptResources {
    pub collected: Amounts,
}
#[cfg(test)]
mod tests;
