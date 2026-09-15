//! Session-only mission transitions and completion boundary.
use bevy::prelude::*;
pub(crate) mod campaign;
#[cfg(test)]
mod module_tests;
use campaign::MissionId;
pub(crate) mod scene;
pub(crate) mod selection_scene;
pub(crate) mod validation;
use crate::{
    combat::Encounter,
    game::{GamePhase, GameplaySet, MissionBoundary},
};
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MissionAction {
    MissionSelect,
    SelectMission(MissionId),
    ModuleShop,
    SelectModule(crate::modules::ModuleKind),
    BuyModule,
    AssignModule(usize),
    RemoveModule(usize),
    Passives,
    Purchase(crate::passives::NodeId),
    Briefing,
    Launch,
    Hub,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MissionResult {
    pub attempt: u64,
    pub mission: MissionId,
    pub succeeded: bool,
    pub elapsed: f64,
    pub kills: u32,
    pub rewards: crate::economy::RewardReceipt,
}
#[derive(Resource, Default)]
pub(crate) struct Campaign {
    pub history: Vec<MissionResult>,
    pub progress: campaign::Progress,
    pub wallet: crate::economy::Amounts,
    pub passives: crate::passives::PassiveTree,
    pub inventory: crate::modules::shop::ModuleInventory,
}
#[derive(Resource, Default)]
pub(crate) struct MissionSession {
    pub result: Option<MissionResult>,
    pub purchase_feedback: String,
    pub selected_module: usize,
    pub active_loadout: Option<crate::modules::Loadout>,
    pub selected_mission: MissionId,
    pub active_mission: Option<MissionId>,
    next_attempt: u64,
    active_attempt: Option<u64>,
    armed: bool,
}
pub(crate) struct MissionPlugin;
impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::economy::runtime::EconomyPlugin)
            .init_resource::<Campaign>()
            .init_resource::<MissionSession>()
            .init_resource::<crate::economy::AttemptResources>()
            .init_resource::<MissionBoundary>()
            .init_resource::<Time<Virtual>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(GamePhase::Hub)
            .add_systems(Update, input.in_set(GameplaySet::Transition))
            .add_systems(
                Update,
                reset_resources
                    .in_set(GameplaySet::Reset)
                    .run_if(crate::game::reset_requested),
            )
            .add_systems(Update, finalize.in_set(GameplaySet::Completion));
        app.world_mut().resource_mut::<Time<Virtual>>().pause();
    }
}

#[allow(clippy::too_many_arguments)]
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    buttons: Query<(&MissionAction, Ref<Interaction>)>,
    mut phase: ResMut<GamePhase>,
    mut session: ResMut<MissionSession>,
    mut campaign: ResMut<Campaign>,
    mut boundary: ResMut<MissionBoundary>,
    mut clock: ResMut<Time<Virtual>>,
) {
    *boundary = default();
    let restart = matches!(*phase, GamePhase::Playing | GamePhase::Choosing)
        && keys.just_pressed(KeyCode::KeyR);
    if restart {
        launch(&mut session, &mut boundary, &mut phase, &mut clock);
        return;
    }
    if !session.armed {
        if !keys.any_pressed([
            KeyCode::Enter,
            KeyCode::Backspace,
            KeyCode::KeyU,
            KeyCode::KeyM,
            KeyCode::KeyC,
        ]) && !(*phase == GamePhase::MissionSelect
            && keys.any_pressed([
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ]))
            && !(*phase == GamePhase::ModuleShop
                && keys.any_pressed(crate::modules::shop_input::ACTION_KEYS))
            && !keys.any_pressed(crate::passives::PURCHASE_KEYS)
            && !mouse.pressed(MouseButton::Left)
            && !buttons
                .iter()
                .any(|(_, interaction)| *interaction == Interaction::Pressed)
        {
            session.armed = true;
        }
        return;
    }
    let primary = match *phase {
        GamePhase::Hub | GamePhase::MissionSelect => Some(MissionAction::Briefing),
        GamePhase::Briefing => Some(MissionAction::Launch),
        GamePhase::Dead | GamePhase::Survived => Some(MissionAction::Hub),
        _ => None,
    };
    let action = if keys.just_pressed(KeyCode::Enter) {
        primary
    } else if keys.just_pressed(KeyCode::KeyC) && *phase == GamePhase::Hub {
        Some(MissionAction::MissionSelect)
    } else if keys.just_pressed(KeyCode::KeyU) && *phase == GamePhase::Hub {
        Some(MissionAction::Passives)
    } else if keys.just_pressed(KeyCode::KeyM) && *phase == GamePhase::Hub {
        Some(MissionAction::ModuleShop)
    } else if keys.just_pressed(KeyCode::Backspace)
        && matches!(
            *phase,
            GamePhase::Briefing
                | GamePhase::Passives
                | GamePhase::ModuleShop
                | GamePhase::MissionSelect
        )
    {
        Some(MissionAction::Hub)
    } else {
        let purchase_key = (*phase == GamePhase::Passives)
            .then(|| {
                crate::passives::PURCHASE_KEYS
                    .iter()
                    .position(|key| keys.just_pressed(*key))
                    .map(|index| MissionAction::Purchase(crate::passives::NodeId::ALL[index]))
            })
            .flatten();
        let selection_key = if *phase == GamePhase::MissionSelect {
            let reverse = keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowLeft]);
            let forward = keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::ArrowRight]);
            (reverse || forward).then(|| {
                let current = session.selected_mission.index();
                let id = (1..=12)
                    .map(|offset| {
                        MissionId::ALL[(current + if reverse { 12 - offset } else { offset }) % 12]
                    })
                    .find(|id| campaign.progress.unlocked(*id))
                    .unwrap_or_default();
                MissionAction::SelectMission(id)
            })
        } else {
            None
        };
        selection_key
            .or(purchase_key)
            .or_else(|| {
                (*phase == GamePhase::ModuleShop)
                    .then(|| crate::modules::shop_input::keyboard(&keys, session.selected_module))
                    .flatten()
            })
            .or_else(|| {
                buttons
                    .iter()
                    .filter_map(|(action, interaction)| {
                        (interaction.is_changed() && *interaction == Interaction::Pressed)
                            .then_some(*action)
                    })
                    .min()
            })
    };
    match (*phase, action) {
        (GamePhase::Hub, Some(MissionAction::MissionSelect)) => {
            *phase = GamePhase::MissionSelect;
            session.armed = false;
            session.purchase_feedback.clear();
        }
        (GamePhase::MissionSelect, Some(MissionAction::SelectMission(id))) => {
            if campaign.progress.unlocked(id) {
                session.selected_mission = id;
                session.purchase_feedback.clear();
            } else {
                session.purchase_feedback = format!(
                    "Mission {:02} locked. {}.",
                    id.index() + 1,
                    id.requirement()
                );
            }
            session.armed = false;
        }
        (GamePhase::Hub, Some(MissionAction::ModuleShop)) => {
            *phase = GamePhase::ModuleShop;
            session.armed = false;
            session.purchase_feedback.clear();
        }
        (GamePhase::ModuleShop, Some(action)) if action != MissionAction::Hub => {
            if crate::modules::shop_input::apply(action, &mut campaign, &mut session) {
                session.armed = false;
            }
        }
        (GamePhase::Hub, Some(MissionAction::Passives)) => {
            *phase = GamePhase::Passives;
            session.armed = false;
            session.purchase_feedback.clear();
        }
        (GamePhase::Passives, Some(MissionAction::Purchase(node))) => {
            let Campaign {
                passives, wallet, ..
            } = &mut *campaign;
            session.purchase_feedback = match passives.purchase(node, wallet) {
                Ok(rank) => format!(
                    "{} rank {rank} purchased. Applies next launch.",
                    node.name()
                ),
                Err(crate::passives::PurchaseError::Locked(previous)) => {
                    format!("Buy {} rank 1 first.", previous.name())
                }
                Err(crate::passives::PurchaseError::Maxed) => {
                    format!("{} is already at rank 5.", node.name())
                }
                Err(crate::passives::PurchaseError::InsufficientFunds) => {
                    "Not enough banked resources for this rank.".into()
                }
            };
            session.armed = false;
        }
        (GamePhase::Hub | GamePhase::MissionSelect, Some(MissionAction::Briefing)) => {
            if !campaign.progress.unlocked(session.selected_mission) {
                session.purchase_feedback = session.selected_mission.requirement();
                session.armed = false;
                return;
            }
            *phase = GamePhase::Briefing;
            session.armed = false;
            session.purchase_feedback.clear();
        }
        (GamePhase::Briefing, Some(MissionAction::Launch)) => {
            if !campaign.progress.unlocked(session.selected_mission) {
                session.purchase_feedback = session.selected_mission.requirement();
                session.armed = false;
                return;
            }
            match campaign.inventory.validate() {
                Ok(()) => {
                    session.active_loadout = Some(campaign.inventory.loadout().clone());
                    session.active_mission = Some(session.selected_mission);
                    launch(&mut session, &mut boundary, &mut phase, &mut clock);
                }
                Err(_) => {
                    session.purchase_feedback =
                        "Invalid loadout. Return to the hub and equip owned modules only.".into();
                    session.armed = false;
                }
            }
        }
        (
            GamePhase::Briefing
            | GamePhase::MissionSelect
            | GamePhase::Passives
            | GamePhase::ModuleShop
            | GamePhase::Dead
            | GamePhase::Survived,
            Some(MissionAction::Hub),
        ) => {
            *phase = GamePhase::Hub;
            session.armed = false;
            boundary.cleanup = true;
        }
        _ => {}
    }
}
fn launch(
    session: &mut MissionSession,
    boundary: &mut MissionBoundary,
    phase: &mut GamePhase,
    clock: &mut Time<Virtual>,
) {
    session.next_attempt += 1;
    session.active_attempt = Some(session.next_attempt);
    session.result = None;
    session.armed = false;
    boundary.reset = true;
    *phase = GamePhase::Playing;
    clock.unpause();
}
#[allow(clippy::too_many_arguments)]
fn finalize(
    phase: Res<GamePhase>,
    encounter: Res<Encounter>,
    mut session: ResMut<MissionSession>,
    mut campaign: ResMut<Campaign>,
    mut boundary: ResMut<MissionBoundary>,
    mut clock: ResMut<Time<Virtual>>,
    mut resources: ResMut<crate::economy::AttemptResources>,
) {
    if boundary.reset {
        return;
    }
    if !matches!(*phase, GamePhase::Dead | GamePhase::Survived) {
        return;
    }
    clock.pause();
    let Some(attempt) = session.active_attempt.take() else {
        return;
    };
    let result = MissionResult {
        attempt,
        mission: session
            .active_mission
            .expect("launched attempts have a mission"),
        succeeded: *phase == GamePhase::Survived,
        elapsed: encounter.elapsed,
        kills: encounter.kills,
        rewards: crate::economy::RewardReceipt::settle(
            std::mem::take(&mut resources.collected),
            *phase == GamePhase::Survived,
            &mut campaign.wallet,
        ),
    };
    if result.succeeded {
        campaign.progress.complete(result.mission);
    }
    campaign.history.push(result.clone());
    session.result = Some(result);
    session.armed = false;
    boundary.cleanup = true;
}
fn reset_resources(mut resources: ResMut<crate::economy::AttemptResources>) {
    *resources = default();
}
#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod campaign_tests;
