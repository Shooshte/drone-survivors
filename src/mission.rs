//! Session-only mission transitions and completion boundary.
use bevy::prelude::*;
pub(crate) mod scene;
pub(crate) mod validation;
use crate::{
    combat::Encounter,
    game::{GamePhase, GameplaySet, MissionBoundary},
};
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MissionAction {
    Briefing,
    Launch,
    Hub,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MissionResult {
    pub attempt: u64,
    pub succeeded: bool,
    pub elapsed: f64,
    pub kills: u32,
    pub rewards: crate::economy::RewardReceipt,
}
#[derive(Resource, Default)]
pub(crate) struct Campaign {
    pub history: Vec<MissionResult>,
    pub mission_succeeded: bool,
    pub wallet: crate::economy::Amounts,
}
#[derive(Resource, Default)]
pub(crate) struct MissionSession {
    pub result: Option<MissionResult>,
    next_attempt: u64,
    active_attempt: Option<u64>,
    armed: bool,
}
pub(crate) struct MissionPlugin;
impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Campaign>()
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
        if !keys.any_pressed([KeyCode::Enter, KeyCode::Backspace])
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
        GamePhase::Hub => Some(MissionAction::Briefing),
        GamePhase::Briefing => Some(MissionAction::Launch),
        GamePhase::Dead | GamePhase::Survived => Some(MissionAction::Hub),
        _ => None,
    };
    let action = if keys.just_pressed(KeyCode::Enter) {
        primary
    } else if keys.just_pressed(KeyCode::Backspace) && *phase == GamePhase::Briefing {
        Some(MissionAction::Hub)
    } else {
        buttons.iter().find_map(|(action, interaction)| {
            (interaction.is_changed() && *interaction == Interaction::Pressed).then_some(*action)
        })
    };
    match (*phase, action) {
        (GamePhase::Hub, Some(MissionAction::Briefing)) => {
            *phase = GamePhase::Briefing;
            session.armed = false;
        }
        (GamePhase::Briefing, Some(MissionAction::Launch)) => {
            launch(&mut session, &mut boundary, &mut phase, &mut clock)
        }
        (GamePhase::Briefing | GamePhase::Dead | GamePhase::Survived, Some(MissionAction::Hub)) => {
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
        succeeded: *phase == GamePhase::Survived,
        elapsed: encounter.elapsed,
        kills: encounter.kills,
        rewards: crate::economy::RewardReceipt::settle(
            std::mem::take(&mut resources.collected),
            *phase == GamePhase::Survived,
            &mut campaign.wallet,
        ),
    };
    campaign.mission_succeeded |= result.succeeded;
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
