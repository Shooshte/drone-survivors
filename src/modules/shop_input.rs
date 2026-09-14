//! Shop actions pass through the shared mission input release gate.
use super::{ModuleKind, SLOT_KEYS, shop::ShopError};
use crate::mission::{Campaign, MissionAction, MissionSession};
use bevy::prelude::*;

pub(crate) const ACTION_KEYS: [KeyCode; 3] = [KeyCode::KeyB, KeyCode::ArrowUp, KeyCode::ArrowDown];

pub(crate) fn keyboard(keys: &ButtonInput<KeyCode>, selected: usize) -> Option<MissionAction> {
    if keys.just_pressed(KeyCode::ArrowUp) {
        Some(MissionAction::SelectModule(
            ModuleKind::ALL[(selected + 3) % 4],
        ))
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Some(MissionAction::SelectModule(
            ModuleKind::ALL[(selected + 1) % 4],
        ))
    } else if keys.just_pressed(KeyCode::KeyB) {
        Some(MissionAction::BuyModule)
    } else {
        SLOT_KEYS
            .iter()
            .position(|key| keys.just_pressed(*key))
            .map(|slot| {
                if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                    MissionAction::RemoveModule(slot)
                } else {
                    MissionAction::AssignModule(slot)
                }
            })
    }
}

pub(crate) fn apply(
    action: MissionAction,
    campaign: &mut Campaign,
    session: &mut MissionSession,
) -> bool {
    let selected = ModuleKind::ALL[session.selected_module];
    let result = match action {
        MissionAction::SelectModule(kind) => {
            session.selected_module = kind as usize;
            session.purchase_feedback.clear();
            return true;
        }
        MissionAction::BuyModule => campaign
            .inventory
            .purchase(selected, &mut campaign.wallet)
            .map(|()| format!("{} purchased. Choose a slot to equip it.", selected.name())),
        MissionAction::AssignModule(slot) => {
            campaign.inventory.assign(slot, Some(selected)).map(|()| {
                format!(
                    "{} assigned to slot {}. No charge.",
                    selected.name(),
                    slot + 1
                )
            })
        }
        MissionAction::RemoveModule(slot) => campaign
            .inventory
            .assign(slot, None)
            .map(|()| format!("Slot {} cleared. Module remains owned.", slot + 1)),
        _ => return false,
    };
    session.purchase_feedback = match result {
        Ok(message) => message,
        Err(ShopError::AlreadyOwned(kind)) => format!(
            "{} is already owned. Assign it to a slot for free.",
            kind.name()
        ),
        Err(ShopError::InsufficientFunds) => "Not enough banked salvage or components.".into(),
        Err(ShopError::NotOwned(kind)) => format!("Buy {} before equipping it.", kind.name()),
        Err(ShopError::InvalidSlot(_)) => "Choose a slot from 1 to 4.".into(),
        Err(ShopError::InvalidLoadout(reason)) => reason.into(),
    };
    true
}
