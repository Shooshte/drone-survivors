//! Session menus over the reusable arena scene.
use super::{Campaign, MissionAction, MissionSession};
use crate::game::{GamePhase, GameplaySet};
use bevy::prelude::*;

pub(crate) struct MissionScenePlugin;

// Match the existing flight instruments: deep blue, cyan, silver, amber.
const INK: Color = Color::srgb(0.025, 0.045, 0.065);
const PANEL: Color = Color::srgb(0.045, 0.075, 0.095);
const SILVER: Color = Color::srgb(0.9, 0.94, 0.92);
const MUTED: Color = Color::srgb(0.60, 0.75, 0.79);
const CYAN: Color = Color::srgb(0.40, 0.90, 0.88);
const AMBER: Color = Color::srgb(1., 0.72, 0.40);

#[derive(Component)]
struct MissionOverlay;
#[derive(Component)]
struct PrimaryAction;
#[derive(Component)]
pub(super) struct PrimaryLabel;
#[derive(Component)]
struct BackAction;
#[derive(Component)]
struct ShopAction;
#[derive(Component)]
enum MenuCopy {
    Eyebrow,
    Heading,
    Description,
    Body,
    Note,
    PrimaryLabel,
}

impl Plugin for MissionScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::passives::scene::PassiveScenePlugin)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (present, hover).chain().in_set(GameplaySet::Presentation),
            );
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Mission menu"),
            MissionOverlay,
            GlobalZIndex(200),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(12)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(INK),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Name::new("Mission operations panel"),
                    Node {
                        width: percent(100),
                        max_width: px(680),
                        padding: UiRect::all(px(16)),
                        border: UiRect::left(px(3)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(CYAN),
                ))
                .with_children(|panel| {
                    copy(panel, MenuCopy::Eyebrow, 12., CYAN);
                    copy(panel, MenuCopy::Heading, 30., SILVER);
                    copy(panel, MenuCopy::Description, 15., SILVER);
                    panel.spawn((
                        Node {
                            width: percent(100),
                            height: px(1),
                            flex_shrink: 0.,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.29, 0.33)),
                    ));
                    copy(panel, MenuCopy::Body, 15., SILVER);
                    copy(panel, MenuCopy::Note, 13., MUTED);
                    panel
                        .spawn((
                            Button,
                            PrimaryAction,
                            MissionAction::Briefing,
                            Name::new("Mission primary action"),
                            button_node(42.),
                            BackgroundColor(Color::srgb(0.08, 0.23, 0.26)),
                            BorderColor::all(CYAN),
                        ))
                        .with_children(|button| {
                            copy(button, MenuCopy::PrimaryLabel, 16., SILVER);
                            button.spawn((
                                Text::new("ENTER"),
                                TextFont::from_font_size(12.),
                                TextColor(CYAN),
                            ));
                        });
                    panel
                        .spawn((
                            Button,
                            ShopAction,
                            MissionAction::Passives,
                            Name::new("Permanent upgrades"),
                            button_node(34.),
                            BackgroundColor(PANEL),
                            BorderColor::all(CYAN),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Permanent upgrades"),
                                TextFont::from_font_size(14.),
                                TextColor(SILVER),
                            ));
                            button.spawn((
                                Text::new("U"),
                                TextFont::from_font_size(12.),
                                TextColor(CYAN),
                            ));
                        });
                    panel
                        .spawn((
                            Button,
                            BackAction,
                            MissionAction::Hub,
                            Name::new("Back to hub"),
                            button_node(34.),
                            BackgroundColor(PANEL),
                            BorderColor::all(MUTED),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Back to hub"),
                                TextFont::from_font_size(14.),
                                TextColor(SILVER),
                            ));
                            button.spawn((
                                Text::new("BACKSPACE"),
                                TextFont::from_font_size(12.),
                                TextColor(MUTED),
                            ));
                        });
                    panel.spawn((
                        Text::new("ESC  Quit"),
                        TextFont::from_font_size(12.),
                        TextColor(MUTED),
                    ));
                });
        });
}

fn copy(parent: &mut ChildSpawnerCommands, part: MenuCopy, size: f32, color: Color) {
    let primary_label = matches!(part, MenuCopy::PrimaryLabel);
    let mut entity = parent.spawn((
        part,
        Text::default(),
        TextFont::from_font_size(size),
        TextColor(color),
        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
        Node {
            min_width: px(0),
            flex_grow: if primary_label { 1. } else { 0. },
            ..default()
        },
    ));
    if primary_label {
        entity.insert(PrimaryLabel);
    }
}

fn button_node(height: f32) -> Node {
    Node {
        width: percent(100),
        min_height: px(height),
        flex_shrink: 0.,
        padding: UiRect::horizontal(px(12)),
        border: UiRect::all(px(1)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(12),
        ..default()
    }
}

type MenuVisibility<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Has<MissionOverlay>,
        Has<BackAction>,
        Has<ShopAction>,
    ),
    Or<(With<MissionOverlay>, With<BackAction>, With<ShopAction>)>,
>;

fn present(
    phase: Res<GamePhase>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    mut panels: MenuVisibility,
    mut primary: Single<&mut MissionAction, With<PrimaryAction>>,
    mut text: Query<(&MenuCopy, &mut Text, &mut TextColor)>,
) {
    let shown = matches!(
        *phase,
        GamePhase::Hub | GamePhase::Briefing | GamePhase::Dead | GamePhase::Survived
    );
    for (mut node, overlay, back, shop) in &mut panels {
        let visible = (overlay && shown)
            || (back && *phase == GamePhase::Briefing)
            || (shop && *phase == GamePhase::Hub);
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !shown {
        return;
    }
    **primary = match *phase {
        GamePhase::Hub => MissionAction::Briefing,
        GamePhase::Briefing => MissionAction::Launch,
        _ => MissionAction::Hub,
    };
    let result = session.result.as_ref();
    let won = result.is_some_and(|result| result.succeeded);
    for (part, mut text, mut color) in &mut text {
        let value = match (part, *phase) {
            (MenuCopy::Eyebrow, GamePhase::Hub) => "DRONE SURVIVORS / OPERATIONS".into(),
            (MenuCopy::Eyebrow, GamePhase::Briefing) => "MISSION BRIEFING / SURVIVAL".into(),
            (MenuCopy::Eyebrow, _) => "MISSION RESULTS / SURVIVAL".into(),
            (MenuCopy::Heading, GamePhase::Hub) => "Ready for deployment".into(),
            (MenuCopy::Heading, GamePhase::Briefing) => "Hold out for five minutes".into(),
            (MenuCopy::Heading, _) => if won { "Mission survived" } else { "Drone lost" }.into(),
            (MenuCopy::Description, GamePhase::Hub) => "The Scout is ready. Review the mission, then launch into the arena.".into(),
            (MenuCopy::Description, GamePhase::Briefing) => "Survive for 5:00. Keep your hull above zero as enemy waves grow. Upgrade choices pause the clock.".into(),
            (MenuCopy::Description, _) => if result.is_some_and(|r| r.rewards.error.is_some()) {
                "Reward transaction failed; your previous balances are unchanged."
            } else if won {
                "Survival complete. Your rewards have been banked."
            } else {
                "Hull depleted. A quarter of collected resources is kept."
            }.into(),
            (MenuCopy::Body, GamePhase::Hub) => {
                let count = campaign.history.len();
                let noun = if count == 1 { "attempt" } else { "attempts" };
                let status = if campaign.mission_succeeded { "Survival achieved" } else { "Survival not yet achieved" };
                format!("SURVIVAL / 5:00\n{count} completed {noun} this session / {status}\nBANK  Salvage {}  |  Components {}", campaign.wallet.salvage, campaign.wallet.components)
            },
            (MenuCopy::Body, GamePhase::Briefing) => "FIXED LOADOUT / SCOUT\n1  Weapon overdrive  |  2  Shield\n3  Mobility          |  4  Rocket launcher".into(),
            (MenuCopy::Body, _) => result.map(|r| {
                let seconds = r.elapsed.floor() as u64;
                format!("ATTEMPT {}  |  Active time {}:{:02}  |  Kills {}\n\nSALVAGE  {} collected -{} lost +{} bonus\nBanked +{}  |  Balance {}\nCOMPONENTS  {} collected -{} lost +{} bonus\nBanked +{}  |  Balance {}",
                    r.attempt, seconds / 60, seconds % 60, r.kills,
                    r.rewards.collected.salvage, r.rewards.lost.salvage, r.rewards.bonus.salvage, r.rewards.credited.salvage, r.rewards.balance.salvage,
                    r.rewards.collected.components, r.rewards.lost.components, r.rewards.bonus.components, r.rewards.credited.components, r.rewards.balance.components)
            }).unwrap_or_default(),
            (MenuCopy::Note, GamePhase::Hub) => "Launch and equipment are free. Replays earn rewards.\nBalances, permanent upgrades and history last until you quit.".into(),
            (MenuCopy::Note, GamePhase::Briefing) => "Fly nearby: gold salvage 100 units / purple caches 50 units.\nSuccess: loot +10 salvage, +1 component. Failure: keep 25%.\n1-4 toggle modules; R discards loot and restarts.".into(),
            (MenuCopy::Note, _) => if result.is_some_and(|r| r.rewards.error.is_some()) {
                "Balance limit reached: reward could not be banked.\nThe result retains the collection and bonus amounts."
            } else {
                "Banked once. Uncollected pickups award nothing.\nYour next launch restores the Scout and resets temporary upgrades."
            }.into(),
            (MenuCopy::PrimaryLabel, GamePhase::Hub) => "Mission briefing".into(),
            (MenuCopy::PrimaryLabel, GamePhase::Briefing) => "Launch mission".into(),
            (MenuCopy::PrimaryLabel, _) => "Return to hub".into(),
        };
        if text.0 != value {
            text.0 = value;
        }
        if matches!(part, MenuCopy::Heading) {
            color.0 = if matches!(*phase, GamePhase::Dead | GamePhase::Survived) {
                if won { CYAN } else { AMBER }
            } else {
                SILVER
            };
        }
    }
}

type ButtonColors<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (With<MissionAction>, Changed<Interaction>),
>;

fn hover(mut buttons: ButtonColors) {
    for (interaction, mut fill) in &mut buttons {
        fill.0 = match *interaction {
            Interaction::Pressed => Color::srgb(0.13, 0.35, 0.37),
            Interaction::Hovered => Color::srgb(0.10, 0.28, 0.31),
            Interaction::None => Color::srgb(0.065, 0.15, 0.18),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(GamePhase::Hub)
            .init_resource::<Campaign>()
            .init_resource::<MissionSession>()
            .add_plugins(MissionScenePlugin);
        app.update();
        app
    }

    #[test]
    fn menu_routes_and_visibility_follow_phase_without_accumulating_entities() {
        let mut app = app();
        let count = app.world().entities().len();
        for _ in 0..3 {
            for (phase, action, visible, back) in [
                (GamePhase::Hub, MissionAction::Briefing, true, false),
                (GamePhase::Briefing, MissionAction::Launch, true, true),
                (GamePhase::Playing, MissionAction::Launch, false, false),
                (GamePhase::Choosing, MissionAction::Launch, false, false),
                (GamePhase::Dead, MissionAction::Hub, true, false),
                (GamePhase::Survived, MissionAction::Hub, true, false),
            ] {
                *app.world_mut().resource_mut::<GamePhase>() = phase;
                app.update();
                let world = app.world_mut();
                let overlay = world
                    .query_filtered::<&Node, With<MissionOverlay>>()
                    .single(world)
                    .unwrap();
                assert_eq!(overlay.display != Display::None, visible);
                let primary = world
                    .query_filtered::<&MissionAction, With<PrimaryAction>>()
                    .single(world)
                    .unwrap();
                if visible {
                    assert_eq!(*primary, action);
                }
                let secondary = world
                    .query_filtered::<&Node, With<BackAction>>()
                    .single(world)
                    .unwrap();
                assert_eq!(secondary.display != Display::None, back);
                assert_eq!(world.entities().len(), count);
            }
        }
    }

    #[test]
    fn results_use_completed_snapshot_and_hub_preserves_campaign_success() {
        let mut app = app();
        let result = super::super::MissionResult {
            rewards: default(),
            attempt: 7,
            succeeded: true,
            elapsed: 300.,
            kills: 123,
        };
        app.world_mut()
            .resource_mut::<Campaign>()
            .history
            .push(result.clone());
        app.world_mut().resource_mut::<Campaign>().mission_succeeded = true;
        app.world_mut().resource_mut::<MissionSession>().result = Some(result);
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Survived;
        app.update();
        let body = |app: &mut App| {
            app.world_mut()
                .query::<(&MenuCopy, &Text)>()
                .iter(app.world())
                .find(|(part, _)| matches!(part, MenuCopy::Body))
                .unwrap()
                .1
                .0
                .clone()
        };
        let snapshot = body(&mut app);
        assert!(snapshot.contains("5:00"));
        assert!(snapshot.contains("123"));
        app.update();
        assert_eq!(body(&mut app), snapshot);
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Hub;
        app.update();
        assert!(body(&mut app).contains("1 completed attempt"));
        assert!(body(&mut app).contains("Survival achieved"));
    }
    #[test]
    fn economy_menus_show_bank_and_stable_reward_breakdown() {
        let mut app = app();
        let mut wallet = crate::economy::Amounts {
            salvage: 30,
            components: 6,
        };
        let rewards = crate::economy::RewardReceipt::settle(
            crate::economy::Amounts {
                salvage: 19,
                components: 3,
            },
            false,
            &mut wallet,
        );
        app.world_mut().resource_mut::<Campaign>().wallet = wallet;
        app.world_mut().resource_mut::<MissionSession>().result =
            Some(super::super::MissionResult {
                attempt: 1,
                succeeded: false,
                elapsed: 37.,
                kills: 18,
                rewards,
            });
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
        app.update();
        let all_text = |app: &mut App| {
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .map(|t| t.0.clone())
                .collect::<Vec<_>>()
                .join("\n")
        };
        let result = all_text(&mut app);
        assert!(result.contains("19 collected"));
        assert!(result.contains("15 lost"));
        assert!(result.contains("Banked +4"));
        assert!(result.contains("Balance 34"));
        app.world_mut().resource_mut::<Campaign>().wallet = crate::economy::Amounts::default();
        app.update();
        assert_eq!(all_text(&mut app), result);
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Hub;
        app.update();
        assert!(all_text(&mut app).contains("Salvage 0"));
        assert!(all_text(&mut app).contains("Components 0"));
    }
}
