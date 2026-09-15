//! Three branch columns; each card buys one rank using the mission input gate.
use super::{NodeId, PurchaseError};
use crate::{
    game::{GamePhase, GameplaySet},
    mission::{Campaign, MissionAction, MissionSession},
};
use bevy::prelude::*;

const INK: Color = Color::srgb(0.025, 0.045, 0.065);
const PANEL: Color = Color::srgb(0.045, 0.075, 0.095);
const SILVER: Color = Color::srgb(0.90, 0.94, 0.92);
const MUTED: Color = Color::srgb(0.60, 0.75, 0.79);
const CYAN: Color = Color::srgb(0.40, 0.90, 0.88);
const AMBER: Color = Color::srgb(1., 0.72, 0.40);

pub(crate) struct PassiveScenePlugin;
#[derive(Component)]
struct PassiveOverlay;
#[derive(Component)]
pub(super) struct PassiveCard(pub NodeId);
#[derive(Component)]
pub(super) struct PassiveCardText;
#[derive(Component)]
enum Copy {
    Bank,
    Feedback,
    Card(NodeId),
    ReserveBattery,
}

impl Plugin for PassiveScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, present.in_set(GameplaySet::Presentation));
    }
}
fn text(parent: &mut ChildSpawnerCommands, value: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
    ));
}
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Permanent passive tree"),
            PassiveOverlay,
            GlobalZIndex(210),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                display: Display::None,
                ..default()
            },
            BackgroundColor(INK),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: percent(100),
                        max_width: px(940),
                        padding: UiRect::all(px(12)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|panel| {
                    text(panel, "Upgrades", 22., SILVER);
                    panel.spawn((
                        Copy::Bank,
                        Text::default(),
                        TextFont::from_font_size(14.),
                        TextColor(CYAN),
                    ));
                    text(
                        panel,
                        "Permanent passives: 5 ranks each. Rank 1 unlocks the next node.",
                        12.,
                        MUTED,
                    );
                    panel
                        .spawn(Node {
                            width: percent(100),
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|branches| {
                            for (branch, title) in ["OFFENSE", "RESILIENCE", "ENERGY EFFICIENCY"]
                                .into_iter()
                                .enumerate()
                            {
                                branches
                                    .spawn(Node {
                                        flex_basis: px(0),
                                        flex_grow: 1.,
                                        min_width: px(0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(5),
                                        ..default()
                                    })
                                    .with_children(|column| {
                                        text(column, title, 12., CYAN);
                                        for node in &NodeId::ALL[branch * 3..branch * 3 + 3] {
                                            column
                                                .spawn((
                                                    Button,
                                                    MissionAction::Purchase(*node),
                                                    PassiveCard(*node),
                                                    Name::new(node.name()),
                                                    Node {
                                                        width: percent(100),
                                                        min_height: px(74),
                                                        padding: UiRect::all(px(6)),
                                                        border: UiRect::left(px(2)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(INK),
                                                    BorderColor::all(MUTED),
                                                ))
                                                .with_children(|card| {
                                                    card.spawn((
                                                        Copy::Card(*node),
                                                        PassiveCardText,
                                                        Text::default(),
                                                        TextFont::from_font_size(12.),
                                                        TextColor(SILVER),
                                                        TextLayout::new(
                                                            Justify::Left,
                                                            LineBreak::WordBoundary,
                                                        ),
                                                        Node {
                                                            min_width: px(0),
                                                            width: percent(100),
                                                            flex_grow: 1.,
                                                            ..default()
                                                        },
                                                    ));
                                                });
                                        }
                                    });
                            }
                        });
                    panel
                        .spawn((
                            Button,
                            MissionAction::BuyReserveBattery,
                            Name::new("Reserve battery purchase"),
                            Node {
                                width: percent(100),
                                min_height: px(42),
                                padding: UiRect::all(px(6)),
                                border: UiRect::left(px(2)),
                                ..default()
                            },
                            BackgroundColor(INK),
                            BorderColor::all(CYAN),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Copy::ReserveBattery,
                                Text::default(),
                                TextFont::from_font_size(12.),
                                TextColor(CYAN),
                                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                                Node {
                                    width: percent(100),
                                    min_width: px(0),
                                    ..default()
                                },
                            ));
                        });
                    panel.spawn((
                        Copy::Feedback,
                        Text::default(),
                        TextFont::from_font_size(12.),
                        TextColor(AMBER),
                        Node {
                            min_height: px(16),
                            ..default()
                        },
                    ));
                    panel
                        .spawn((
                            Button,
                            MissionAction::Hub,
                            Node {
                                width: percent(100),
                                min_height: px(30),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect::horizontal(px(10)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.08, 0.23, 0.26)),
                        ))
                        .with_children(|button| {
                            text(button, "Back to hub", 14., SILVER);
                            text(button, "BACKSPACE", 12., CYAN);
                        });
                    text(
                        panel,
                        "1-9 or click: buy one rank. Applies next launch. Saved between missions.",
                        12.,
                        MUTED,
                    );
                });
        });
}
fn card_copy(node: NodeId, campaign: &Campaign) -> String {
    let tree = &campaign.passives;
    let rank = tree.rank(node);
    let bonus = tree.bonus_percent(node);
    let sign = if matches!(node, NodeId::Armor | NodeId::Reserve) {
        "-"
    } else {
        "+"
    };
    let effect = if rank == 5 {
        format!("{sign}{bonus}% total")
    } else {
        format!(
            "{sign}{bonus}% -> {sign}{}%",
            bonus + node.percent_per_rank()
        )
    };
    let price = tree.next_cost(node).map_or_else(
        || "All ranks purchased".into(),
        |cost| format!("{} salvage / {} components", cost.salvage, cost.components),
    );
    let state = match tree.availability(node, &campaign.wallet) {
        Ok(()) => "Buy next rank".into(),
        Err(PurchaseError::Locked(previous)) => {
            format!("Needs node {} at rank 1", previous as usize + 1)
        }
        Err(PurchaseError::Maxed) => "MAXED".into(),
        Err(PurchaseError::InsufficientFunds) => "Not enough resources".into(),
    };
    format!(
        "{}  {}\nRank {rank}/5  {effect}\n{price}\n{state}",
        node as usize + 1,
        node.name()
    )
}
fn present(
    phase: Res<GamePhase>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    mut overlay: Single<&mut Node, With<PassiveOverlay>>,
    mut copies: Query<(&Copy, &mut Text)>,
    mut cards: Query<(
        &PassiveCard,
        &Interaction,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    overlay.display = if *phase == GamePhase::Passives {
        Display::Flex
    } else {
        Display::None
    };
    if *phase != GamePhase::Passives {
        return;
    }
    for (part, mut text) in &mut copies {
        let next = match part {
            Copy::Bank => format!(
                "BANK   {} salvage   /   {} components",
                campaign.wallet.salvage, campaign.wallet.components
            ),
            Copy::Feedback => session.purchase_feedback.clone(),
            Copy::Card(node) => card_copy(*node, &campaign),
            Copy::ReserveBattery => {
                let state = if campaign.secrets.reserve_battery {
                    "ACTIVE"
                } else if !campaign.secrets.blueprint {
                    "LOCKED / Find blueprint in a cache"
                } else if campaign.wallet.salvage < 20 || campaign.wallet.components < 1 {
                    "Need 20 salvage + 1 component"
                } else {
                    "BUY: 20 salvage + 1 component"
                };
                format!(
                    "B  Reserve battery (+25 capacity) / {state}\nLost on defeat or restart. Blueprint stays unlocked."
                )
            }
        };
        if text.0 != next {
            text.0 = next;
        }
    }
    for (card, interaction, mut fill, mut border) in &mut cards {
        let state = campaign.passives.availability(card.0, &campaign.wallet);
        *border = BorderColor::all(match state {
            Ok(()) => CYAN,
            Err(PurchaseError::Maxed) => SILVER,
            _ => MUTED,
        });
        fill.0 = if state.is_ok() && *interaction != Interaction::None {
            Color::srgb(0.10, 0.28, 0.31)
        } else {
            INK
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cards_explain_locked_unaffordable_owned_and_maxed_states() {
        let mut campaign = Campaign::default();
        assert!(card_copy(NodeId::FireRate, &campaign).contains("Needs node 1 at rank 1"));
        assert!(card_copy(NodeId::Battery, &campaign).contains("Not enough resources"));
        campaign.wallet = crate::economy::Amounts {
            salvage: 1000,
            components: 100,
        };
        for _ in 0..5 {
            campaign
                .passives
                .purchase(NodeId::Battery, &mut campaign.wallet)
                .unwrap();
        }
        let text = card_copy(NodeId::Battery, &campaign);
        assert!(text.contains("Rank 5/5"));
        assert!(text.contains("+100% total"));
        assert!(text.contains("MAXED"));
    }
    #[test]
    fn only_shop_phase_shows_overlay_and_repeated_navigation_reuses_entities() {
        let mut app = App::new();
        app.init_resource::<Campaign>()
            .init_resource::<MissionSession>()
            .insert_resource(GamePhase::Hub)
            .add_plugins(PassiveScenePlugin);
        app.update();
        let count = app.world().entities().len();
        for phase in [
            GamePhase::Passives,
            GamePhase::Hub,
            GamePhase::Passives,
            GamePhase::Playing,
        ] {
            *app.world_mut().resource_mut::<GamePhase>() = phase;
            app.update();
            let node = app
                .world_mut()
                .query_filtered::<&Node, With<PassiveOverlay>>()
                .single(app.world())
                .unwrap();
            assert_eq!(node.display == Display::Flex, phase == GamePhase::Passives);
            assert_eq!(app.world().entities().len(), count);
        }
    }
}
