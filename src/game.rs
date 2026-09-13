use bevy::prelude::*;

#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum GamePhase {
    #[default]
    Playing,
    Hub,
    Briefing,
    Choosing,
    Dead,
    Survived,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GameplaySet {
    Transition,
    Baseline,
    Reset,
    ChoiceInput,
    Movement,
    Combat,
    Collection,
    Progression,
    Completion,
    Cleanup,
    Presentation,
}

/// Present only for the normal mission flow. Legacy fixtures retain their R reset.
#[derive(Resource, Default)]
pub(crate) struct MissionBoundary {
    pub reset: bool,
    pub cleanup: bool,
}

pub(crate) fn reset_requested(
    keys: Res<ButtonInput<KeyCode>>,
    boundary: Option<Res<MissionBoundary>>,
) -> bool {
    boundary.map_or_else(|| keys.just_pressed(KeyCode::KeyR), |b| b.reset)
}

pub(crate) fn is_playing(phase: Res<GamePhase>, boundary: Option<Res<MissionBoundary>>) -> bool {
    *phase == GamePhase::Playing && boundary.is_none_or(|b| !b.reset)
}
