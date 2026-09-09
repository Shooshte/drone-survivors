use bevy::prelude::*;

#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum GamePhase {
    #[default]
    Playing,
    Choosing,
    Dead,
    Survived,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GameplaySet {
    Reset,
    ChoiceInput,
    Movement,
    Combat,
    Progression,
    Presentation,
}

pub(crate) fn is_playing(phase: Res<GamePhase>) -> bool {
    *phase == GamePhase::Playing
}
