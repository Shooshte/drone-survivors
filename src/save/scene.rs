//! Campaign startup and recovery UI, sharing the operations palette.
use super::{Mode, SaveAction, SaveState};
use crate::game::{GamePhase, GameplaySet};
use bevy::prelude::*;

pub(crate) struct SaveScenePlugin;
#[derive(Component)]
struct Overlay;
#[derive(Component)]
struct Copy;
#[derive(Component)]
struct HubButton;
impl Plugin for SaveScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, present.in_set(GameplaySet::Presentation));
    }
}
fn button(parent: &mut ChildSpawnerCommands, action: SaveAction, label: &str) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(12), px(7)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.065, 0.15, 0.18)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: 15.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.94, 0.92)),
            ));
        });
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Overlay,
            GlobalZIndex(300),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(px(16)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.025, 0.045, 0.065)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: percent(100),
                    max_width: px(620),
                    padding: UiRect::all(px(18)),
                    row_gap: px(8),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.045, 0.075, 0.095)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("DRONE SURVIVORS / CAMPAIGN"),
                    TextFont {
                        font_size: 14.0.into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.9, 0.88)),
                ));
                panel.spawn((
                    Copy,
                    Text::default(),
                    TextFont {
                        font_size: 16.0.into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.94, 0.92)),
                ));
                button(panel, SaveAction::Continue, "Continue campaign  /  Enter");
                button(panel, SaveAction::New, "New campaign  /  N");
                button(
                    panel,
                    SaveAction::Confirm,
                    "Archive save and start new  /  Enter",
                );
                button(panel, SaveAction::Cancel, "Cancel  /  Backspace");
                button(panel, SaveAction::Retry, "Retry  /  R");
                panel.spawn((
                    Text::new("Esc  /  Quit"),
                    TextFont {
                        font_size: 12.0.into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.75, 0.79)),
                ));
            });
        });
    commands
        .spawn((
            HubButton,
            SaveAction::Menu,
            Button,
            GlobalZIndex(250),
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(10),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.065, 0.15, 0.18)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("Campaign menu / N"),
                TextFont {
                    font_size: 13.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.94, 0.92)),
            ));
        });
}
#[allow(clippy::type_complexity)]
fn present(
    state: Res<SaveState>,
    phase: Res<GamePhase>,
    mut overlay: Single<&mut Node, (With<Overlay>, Without<SaveAction>)>,
    mut copy: Single<&mut Text, With<Copy>>,
    mut buttons: Query<
        (&SaveAction, &mut Node, &Interaction, &mut BackgroundColor),
        Without<Overlay>,
    >,
) {
    overlay.display = if *phase == GamePhase::CampaignMenu {
        Display::Flex
    } else {
        Display::None
    };
    copy.0 = match state.mode {
        Mode::Confirm => "Start a new campaign?\n\nYour current save will be archived before resources, purchases and mission progress are reset. Cancel keeps it unchanged.".into(),
        Mode::Failed(_) => format!("Progress has not been saved.\n\n{}\n\nResolve the error and retry. Your progress is held in memory. Quitting loses changes since the last successful save.", short_error(&state.error)),
        _ if !state.error.is_empty() => format!("Campaign save needs attention.\n\n{}\n\nThe original is untouched. Retry after fixing the file, or choose New campaign to archive it and start over.", short_error(&state.error)),
        _ if state.loaded.is_some() => {
            let (campaign, _) = state.loaded.clone().unwrap().restore().expect("validated save");
            format!("Continue your campaign\n\n{}/12 missions complete\nSalvage {}  /  Components {}\n\nContinue returns to the hub. Interrupted missions restart fresh.", campaign.progress.count(), campaign.wallet.salvage, campaign.wallet.components)
        }
        _ => "Begin a new campaign\n\nNo saved campaign found. Purchases, loadout and mission results save automatically between missions.".into(),
    };
    for (action, mut node, interaction, mut color) in &mut buttons {
        let shown = match action {
            SaveAction::Menu => state.mode == Mode::Active && *phase == GamePhase::Hub,
            SaveAction::Continue => state.mode == Mode::Menu && state.loaded.is_some(),
            SaveAction::New => state.mode == Mode::Menu,
            SaveAction::Confirm | SaveAction::Cancel => state.mode == Mode::Confirm,
            SaveAction::Retry => {
                matches!(state.mode, Mode::Failed(_))
                    || (state.mode == Mode::Menu && !state.error.is_empty())
            }
        };
        node.display = if shown { Display::Flex } else { Display::None };
        color.0 = match interaction {
            Interaction::Pressed => Color::srgb(0.13, 0.35, 0.37),
            Interaction::Hovered => Color::srgb(0.1, 0.28, 0.31),
            Interaction::None => Color::srgb(0.065, 0.15, 0.18),
        };
    }
}
fn short_error(error: &str) -> String {
    let mut shortened: String = error.chars().take(220).collect();
    if error.chars().count() > 220 {
        shortened.push('…');
    }
    shortened
}
