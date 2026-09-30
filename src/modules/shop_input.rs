//! Shop actions pass through the shared mission input release gate.
use super::{ModuleKind, SLOT_KEYS, shop::ShopError};
use crate::mission::{Campaign, MissionAction, MissionSession};
use bevy::prelude::*;

pub(crate) const ACTION_KEYS: [KeyCode; 3] = [KeyCode::KeyB, KeyCode::ArrowUp, KeyCode::ArrowDown];

pub(crate) fn keyboard(keys: &ButtonInput<KeyCode>, selected: usize) -> Option<MissionAction> {
    if keys.just_pressed(KeyCode::ArrowUp) {
        Some(MissionAction::SelectModule(
            ModuleKind::CAMPAIGN
                [(selected + ModuleKind::CAMPAIGN.len() - 1) % ModuleKind::CAMPAIGN.len()],
        ))
    } else if keys.just_pressed(KeyCode::ArrowDown) {
        Some(MissionAction::SelectModule(
            ModuleKind::CAMPAIGN[(selected + 1) % ModuleKind::CAMPAIGN.len()],
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
    let selected = ModuleKind::CAMPAIGN[session.selected_module];
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
        MissionAction::AssignModule(3) if session.selected_mission.index() == 0 => {
            session.purchase_feedback = "Slot 4 is reserved for the Mission 01 payload. Equip slots 1–3; owned modules are kept.".into();
            return true;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_visits_six_modules_and_wraps_in_both_directions() {
        let expected = [
            ModuleKind::Overdrive,
            ModuleKind::Shield,
            ModuleKind::Mobility,
            ModuleKind::Rocket,
            ModuleKind::Repulsor,
            ModuleKind::Repair,
        ];
        for (index, _) in expected.iter().enumerate() {
            let mut keys = ButtonInput::default();
            keys.press(KeyCode::ArrowDown);
            assert_eq!(
                keyboard(&keys, index),
                Some(MissionAction::SelectModule(expected[(index + 1) % 6]))
            );
            keys.reset_all();
            keys.press(KeyCode::ArrowUp);
            assert_eq!(
                keyboard(&keys, index),
                Some(MissionAction::SelectModule(expected[(index + 5) % 6]))
            );
        }
    }

    #[test]
    fn support_selection_purchase_and_assignment_use_normal_campaign_actions() {
        let mut campaign = Campaign::default();
        campaign.wallet.salvage = 30;
        let mut session = MissionSession::default();
        for (slot, kind) in [ModuleKind::Repulsor, ModuleKind::Repair]
            .into_iter()
            .enumerate()
        {
            assert!(apply(
                MissionAction::SelectModule(kind),
                &mut campaign,
                &mut session
            ));
            assert!(apply(MissionAction::BuyModule, &mut campaign, &mut session));
            assert!(campaign.inventory.owns(kind));
            assert!(apply(
                MissionAction::AssignModule(slot),
                &mut campaign,
                &mut session
            ));
            assert_eq!(campaign.inventory.loadout().slots()[slot], Some(kind));
            apply(MissionAction::AssignModule(3), &mut campaign, &mut session);
            assert!(session.purchase_feedback.contains("reserved"));
            assert_eq!(campaign.inventory.loadout().slots()[3], None);
        }
        assert_eq!(campaign.wallet.salvage, 0);
    }
}
