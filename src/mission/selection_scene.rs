//! Campaign prerequisites and objective metadata.
use super::{Campaign, MissionAction, MissionSession, campaign::MissionId};
use crate::game::{GamePhase, GameplaySet};
use bevy::prelude::*;

const INK: Color = Color::srgb(0.025, 0.045, 0.065);
const PANEL: Color = Color::srgb(0.045, 0.075, 0.095);
const SILVER: Color = Color::srgb(0.9, 0.94, 0.92);
const MUTED: Color = Color::srgb(0.60, 0.75, 0.79);
const CYAN: Color = Color::srgb(0.40, 0.90, 0.88);

pub(crate) struct SelectionScenePlugin;
#[derive(Component)]
struct Overlay;
#[derive(Component)]
pub(super) struct ControlText;
#[derive(Component)]
struct Card(MissionId);
#[derive(Component)]
enum Copy {
    Progress,
    Selection,
    Feedback,
    Card(MissionId),
}

impl Plugin for SelectionScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, present.in_set(GameplaySet::Presentation));
    }
}
fn label(
    parent: &mut ChildSpawnerCommands,
    value: impl Into<String>,
    size: f32,
    color: Color,
) -> Entity {
    parent
        .spawn((
            Text::new(value),
            TextFont::from_font_size(size),
            TextColor(color),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            Node {
                min_width: px(0),
                width: percent(100),
                ..default()
            },
        ))
        .id()
}
fn setup(mut commands: Commands) {
    commands.spawn((Overlay, Name::new("Campaign mission selection"), GlobalZIndex(220),
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100),
            padding: UiRect::all(px(12)), align_items: AlignItems::Center, justify_content: JustifyContent::Center,
            display: Display::None, ..default() }, BackgroundColor(INK)))
        .with_children(|overlay| {
            overlay.spawn((Node { width: percent(100), max_width: px(940), padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column, row_gap: px(8), ..default() }, BackgroundColor(PANEL)))
                .with_children(|panel| {
                    let id = label(panel, "", 22., SILVER); panel.commands().entity(id).insert(Copy::Progress);
                    label(panel, "SHARED ARENA / Increasing chaser waves; finite chargers.", 13., MUTED);
                    label(panel, "Win: collected loot +10 salvage, +1 component. Loss: keep 25% of loot. Replays pay the same.", 12., MUTED);
                    panel.spawn(Node { width: percent(100), column_gap: px(8), ..default() }).with_children(|row| {
                        for act in 0..3 {
                            row.spawn(Node { flex_direction: FlexDirection::Column, flex_grow: 1., flex_basis: px(0), min_width: px(0), row_gap: px(6), ..default() }).with_children(|column| {
                                label(column, format!("ACT {}", act + 1), 14., CYAN);
                                for id in &MissionId::ALL[act * 4..act * 4 + 4] {
                                    column.spawn((Button, Card(*id), MissionAction::SelectMission(*id),
                                        Node { width: percent(100), min_height: px(48), padding: UiRect::all(px(6)), border: UiRect::all(px(1)), align_items: AlignItems::Center, ..default() },
                                        BackgroundColor(PANEL), BorderColor::all(MUTED)))
                                        .with_children(|button| {
                                            let entity = label(button, "", 12., SILVER);
                                            button.commands().entity(entity).insert((Copy::Card(*id), ControlText));
                                        });
                                }
                            });
                        }
                    });
                    let id = label(panel, "", 13., CYAN); panel.commands().entity(id).insert(Copy::Selection);
                    let id = label(panel, "", 12., MUTED); panel.commands().entity(id).insert(Copy::Feedback);
                    panel.spawn(Node { width: percent(100), column_gap: px(8), ..default() }).with_children(|row| {
                        for (action, text) in [(MissionAction::Briefing, "ENTER  Mission briefing"), (MissionAction::Hub, "BACKSPACE  Hub")] {
                            row.spawn((Button, action, Node { min_height: px(34), flex_grow: 1., padding: UiRect::all(px(8)), border: UiRect::all(px(1)), ..default() }, BackgroundColor(INK), BorderColor::all(CYAN)))
                                .with_children(|button| { let id = label(button, text, 13., SILVER);
                                    button.commands().entity(id).insert(ControlText); });
                        }
                    });
                    label(panel, "ARROWS  Select unlocked mission   /   Click a card to select   /   ESC  Quit", 11., MUTED);
                });
        });
}
fn present(
    phase: Res<GamePhase>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    mut overlay: Single<&mut Node, With<Overlay>>,
    mut copy: Query<(&Copy, &mut Text, &mut TextColor)>,
    mut cards: Query<(&Card, &Interaction, &mut BorderColor, &mut BackgroundColor)>,
) {
    overlay.display = if *phase == GamePhase::MissionSelect {
        Display::Flex
    } else {
        Display::None
    };
    if *phase != GamePhase::MissionSelect {
        return;
    }
    for (part, mut text, mut color) in &mut copy {
        let value = match part {
            Copy::Progress => format!(
                "{}/12 COMPLETE / {}",
                campaign.progress.count(),
                if campaign.progress.finished() {
                    "Campaign complete"
                } else {
                    "Choose mission"
                }
            ),
            Copy::Selection => format!(
                "SELECTED  {} / {}",
                session.selected_mission.title(),
                session.selected_mission.objective().label()
            ),
            Copy::Feedback => {
                if session.purchase_feedback.is_empty() {
                    "Win introduction, then both branches, then finale to unlock the next act. All 12 wins finish the campaign.".into()
                } else {
                    session.purchase_feedback.clone()
                }
            }
            Copy::Card(id) => {
                let role = match id.index() {
                    1 => "Reconnaissance",
                    2 => "Extraction",
                    _ => match id.index() % 4 {
                        0 => "Introduction",
                        1 => "Branch A",
                        2 => "Branch B",
                        _ => "Finale",
                    },
                };
                let status = if campaign.progress.completed(*id) {
                    "COMPLETE / Replay".into()
                } else if campaign.progress.unlocked(*id) {
                    "AVAILABLE".into()
                } else {
                    format!("LOCKED / {}", id.requirement())
                };
                color.0 = if campaign.progress.unlocked(*id) {
                    SILVER
                } else {
                    MUTED
                };
                format!("{:02}  {role}\n{status}", id.index() + 1)
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (Card(id), interaction, mut border, mut fill) in &mut cards {
        *border = BorderColor::all(if *id == session.selected_mission {
            CYAN
        } else {
            MUTED
        });
        fill.0 = if *id == session.selected_mission {
            Color::srgb(0.08, 0.23, 0.26)
        } else if *interaction != Interaction::None {
            Color::srgb(0.10, 0.18, 0.22)
        } else {
            INK
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_shows_locks_branches_completion_and_hides_without_accumulating_entities() {
        let mut app = App::new();
        app.insert_resource(GamePhase::MissionSelect)
            .init_resource::<Campaign>()
            .init_resource::<MissionSession>()
            .add_plugins(SelectionScenePlugin);
        app.update();
        let count = app.world().entities().len();
        let copy = |app: &mut App| {
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .map(|t| t.0.clone())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(copy(&mut app).contains("LOCKED / Complete 02 + 03"));
        app.world_mut()
            .resource_mut::<Campaign>()
            .progress
            .complete(MissionId::ALL[0]);
        app.update();
        assert_eq!(copy(&mut app).matches("AVAILABLE").count(), 2);
        for id in MissionId::ALL {
            app.world_mut()
                .resource_mut::<Campaign>()
                .progress
                .complete(id);
        }
        app.update();
        assert!(copy(&mut app).contains("12/12 COMPLETE / Campaign complete"));
        assert_eq!(copy(&mut app).matches("COMPLETE / Replay").count(), 12);
        for _ in 0..3 {
            for phase in [
                GamePhase::Hub,
                GamePhase::Briefing,
                GamePhase::Playing,
                GamePhase::MissionSelect,
            ] {
                *app.world_mut().resource_mut::<GamePhase>() = phase;
                app.update();
                assert_eq!(
                    app.world_mut()
                        .query_filtered::<&Node, With<Overlay>>()
                        .single(app.world())
                        .unwrap()
                        .display
                        == Display::Flex,
                    phase == GamePhase::MissionSelect
                );
                assert_eq!(app.world().entities().len(), count);
            }
        }
    }
}
