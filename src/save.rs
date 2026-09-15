//! Durable campaign snapshots; installed only for normal play.
#[cfg(test)]
mod runtime_tests;
pub(crate) mod scene;
mod snapshot;
mod storage;
#[cfg(test)]
mod tests;

use crate::{
    game::{GamePhase, GameplaySet, MissionBoundary},
    mission::{Campaign, MissionSession},
};
use bevy::prelude::*;
use snapshot::Snapshot;
use storage::Store;

pub(crate) struct SavePlugin {
    path: Result<std::path::PathBuf, String>,
}
impl Default for SavePlugin {
    fn default() -> Self {
        Self {
            path: storage::default_path(),
        }
    }
}
impl SavePlugin {
    #[cfg(test)]
    pub(crate) fn at(path: std::path::PathBuf) -> Self {
        Self { path: Ok(path) }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Menu,
    Confirm,
    Active,
    Failed(GamePhase),
}
#[derive(Resource)]
struct SaveState {
    store: Option<Store>,
    loaded: Option<Snapshot>,
    last_saved: Option<Snapshot>,
    mode: Mode,
    error: String,
    armed: bool,
}
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SaveAction {
    Continue,
    New,
    Confirm,
    Cancel,
    Retry,
    Menu,
}

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        let mut state = SaveState {
            store: self.path.as_ref().ok().cloned().map(Store::new),
            loaded: None,
            last_saved: None,
            mode: Mode::Menu,
            error: self.path.as_ref().err().cloned().unwrap_or_default(),
            armed: false,
        };
        state.reload();
        app.insert_resource(state)
            .insert_resource(GamePhase::CampaignMenu)
            .add_systems(
                Update,
                input
                    .in_set(GameplaySet::Transition)
                    .after(crate::mission::input),
            )
            .add_systems(
                Update,
                autosave
                    .after(GameplaySet::Completion)
                    .before(GameplaySet::Cleanup),
            );
    }
}
impl SaveState {
    fn reload(&mut self) {
        self.loaded = None;
        if let Some(store) = &mut self.store {
            match store.read() {
                Ok(snapshot) => {
                    self.loaded = snapshot;
                    self.error.clear();
                }
                Err(error) => self.error = error,
            }
        }
    }
    fn write(&mut self, snapshot: &Snapshot, replace: bool) -> bool {
        let result = self
            .store
            .as_mut()
            .ok_or_else(|| self.error.clone())
            .and_then(|store| store.write(snapshot, replace));
        match result {
            Ok(()) => {
                self.last_saved = Some(snapshot.clone());
                self.error.clear();
                true
            }
            Err(error) => {
                self.error = error;
                false
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    buttons: Query<(&SaveAction, Ref<Interaction>)>,
    mut state: ResMut<SaveState>,
    mut phase: ResMut<GamePhase>,
    mut campaign: ResMut<Campaign>,
    mut session: ResMut<MissionSession>,
    mut boundary: ResMut<MissionBoundary>,
    mut clock: ResMut<Time<Virtual>>,
) {
    if !state.armed {
        if !keys.any_pressed([
            KeyCode::Enter,
            KeyCode::KeyN,
            KeyCode::KeyR,
            KeyCode::Backspace,
        ]) && !mouse.pressed(MouseButton::Left)
            && !buttons
                .iter()
                .any(|(_, interaction)| *interaction == Interaction::Pressed)
        {
            state.armed = true;
        }
        return;
    }
    let clicked = buttons
        .iter()
        .filter_map(|(action, interaction)| {
            (interaction.is_changed() && *interaction == Interaction::Pressed).then_some(*action)
        })
        .min();
    if state.mode == Mode::Active {
        if *phase == GamePhase::Hub
            && (keys.just_pressed(KeyCode::KeyN) || clicked == Some(SaveAction::Menu))
        {
            state.reload();
            state.mode = Mode::Menu;
            state.armed = false;
            *phase = GamePhase::CampaignMenu;
        }
        return;
    }
    let action = if keys.just_pressed(KeyCode::Enter) {
        Some(if state.mode == Mode::Confirm {
            SaveAction::Confirm
        } else {
            SaveAction::Continue
        })
    } else if keys.just_pressed(KeyCode::KeyN) {
        Some(SaveAction::New)
    } else if keys.just_pressed(KeyCode::KeyR) {
        Some(SaveAction::Retry)
    } else if keys.just_pressed(KeyCode::Backspace) {
        Some(SaveAction::Cancel)
    } else {
        clicked
    };
    let Some(action) = action else {
        return;
    };
    state.armed = false;
    match (state.mode, action) {
        (Mode::Menu, SaveAction::Continue) if state.loaded.is_some() => {
            match state.loaded.clone().unwrap().restore() {
                Ok((restored, resumed)) => {
                    *campaign = restored;
                    *session = resumed;
                    state.last_saved = state.loaded.clone();
                    state.mode = Mode::Active;
                    *phase = GamePhase::Hub;
                    boundary.cleanup = true;
                }
                Err(error) => state.error = error,
            }
        }
        (Mode::Menu, SaveAction::New) => {
            if state.loaded.is_some() || !state.error.is_empty() {
                state.mode = Mode::Confirm;
            } else {
                new_campaign(
                    &mut state,
                    &mut campaign,
                    &mut session,
                    &mut phase,
                    &mut boundary,
                    false,
                );
            }
        }
        (Mode::Confirm, SaveAction::Confirm) => {
            new_campaign(
                &mut state,
                &mut campaign,
                &mut session,
                &mut phase,
                &mut boundary,
                true,
            );
        }
        (Mode::Confirm, SaveAction::Cancel) => {
            state.mode = Mode::Menu;
        }
        (Mode::Menu, SaveAction::Retry) => state.reload(),
        (Mode::Failed(previous), SaveAction::Retry) => {
            let snapshot = Snapshot::capture(&campaign, &session);
            if state.write(&snapshot, false) {
                state.mode = Mode::Active;
                *phase = previous;
                if previous == GamePhase::Playing {
                    clock.unpause();
                }
            }
        }
        _ => {}
    }
}
fn new_campaign(
    state: &mut SaveState,
    campaign: &mut Campaign,
    session: &mut MissionSession,
    phase: &mut GamePhase,
    boundary: &mut MissionBoundary,
    replace: bool,
) {
    let fresh = Campaign::default();
    let fresh_session = MissionSession::default();
    if state.write(&Snapshot::capture(&fresh, &fresh_session), replace) {
        *campaign = fresh;
        *session = fresh_session;
        state.mode = Mode::Active;
        *phase = GamePhase::Hub;
        boundary.cleanup = true;
    } else {
        state.mode = Mode::Menu;
    }
}
fn autosave(
    mut state: ResMut<SaveState>,
    mut phase: ResMut<GamePhase>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    mut clock: ResMut<Time<Virtual>>,
) {
    if state.mode != Mode::Active || *phase == GamePhase::CampaignMenu {
        return;
    }
    // In-flight snapshots exclude attempt loot and temporary powers. Only
    // campaign changes warrant an immediate write while an attempt is active.
    if matches!(*phase, GamePhase::Playing | GamePhase::Choosing) && !campaign.is_changed() {
        return;
    }
    let snapshot = Snapshot::capture(&campaign, &session);
    if state.last_saved.as_ref() == Some(&snapshot) {
        return;
    }
    if !state.write(&snapshot, false) {
        state.mode = Mode::Failed(*phase);
        state.armed = false;
        *phase = GamePhase::CampaignMenu;
        clock.pause();
    }
}
