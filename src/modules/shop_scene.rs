//! Compact loadout catalog for the paused mission hub.
use crate::{
    economy::Amounts,
    energy::ChargerConfig,
    game::{GamePhase, GameplaySet},
    mission::{Campaign, MissionAction, MissionSession},
    modules::{
        ModuleKind,
        shop::price,
        shop_preview::{catalog_detail, potential_power},
    },
    upgrades::runtime::Baseline,
};
use bevy::prelude::*;

const INK: Color = Color::srgb(0.025, 0.045, 0.065);
const PANEL: Color = Color::srgb(0.045, 0.075, 0.095);
const SILVER: Color = Color::srgb(0.90, 0.94, 0.92);
const MUTED: Color = Color::srgb(0.60, 0.75, 0.79);
const CYAN: Color = Color::srgb(0.40, 0.90, 0.88);
const AMBER: Color = Color::srgb(1., 0.72, 0.40);

pub(crate) struct ModuleShopScenePlugin;

#[derive(Component)]
struct ShopOverlay;
#[derive(Component)]
struct CatalogRow(ModuleKind);
#[derive(Component)]
pub(super) struct ShopControlText;
#[derive(Component)]
enum Copy {
    Bank,
    Detail,
    Power,
    Feedback,
}

impl Plugin for ModuleShopScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (present, hover).chain().in_set(GameplaySet::Presentation),
        );
    }
}

fn label(parent: &mut ChildSpawnerCommands, value: impl Into<String>, size: f32, color: Color) {
    parent.spawn((
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
        Node {
            min_width: px(0),
            ..default()
        },
    ));
}

fn control_label(
    parent: &mut ChildSpawnerCommands,
    value: impl Into<String>,
    size: f32,
    color: Color,
    fills_button: bool,
) {
    parent.spawn((
        ShopControlText,
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
        Node {
            min_width: px(0),
            width: if fills_button {
                percent(100)
            } else {
                Val::Auto
            },
            flex_grow: if fills_button { 1. } else { 0. },
            flex_shrink: if fills_button { 1. } else { 0. },
            ..default()
        },
    ));
}

fn hotkey_label(parent: &mut ChildSpawnerCommands, value: impl Into<String>, color: Color) {
    parent.spawn((
        ShopControlText,
        Text::new(value),
        TextFont::from_font_size(9.),
        TextColor(color),
        TextLayout::new(Justify::Right, LineBreak::NoWrap),
        Node {
            width: px(58),
            min_width: px(58),
            flex_shrink: 0.,
            ..default()
        },
    ));
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Module shop overlay"),
            ShopOverlay,
            GlobalZIndex(220),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(8)),
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
                    Name::new("Module shop manifest"),
                    Node {
                        width: percent(100),
                        max_width: px(980),
                        max_height: percent(100),
                        padding: UiRect::all(px(10)),
                        border: UiRect::left(px(3)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(3),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    BorderColor::all(CYAN),
                ))
                .with_children(|panel| {
                    label(panel, "DRONE SURVIVORS / MODULE BAY", 11., CYAN);
                    label(panel, "Loadout manifest", 24., SILVER);
                    panel.spawn((
                        Copy::Bank,
                        Text::default(),
                        TextFont::from_font_size(12.),
                        TextColor(MUTED),
                    ));
                    for kind in ModuleKind::ALL {
                        panel
                            .spawn((
                                Button,
                                CatalogRow(kind),
                                MissionAction::SelectModule(kind),
                                Name::new(format!("{} catalog row", kind.name())),
                                compact_button(21.),
                                BackgroundColor(PANEL),
                                BorderColor::all(MUTED),
                            ))
                            .with_children(|row| control_label(row, "", 11., SILVER, true));
                    }
                    panel.spawn((
                        Copy::Detail,
                        Text::default(),
                        TextFont::from_font_size(12.),
                        TextColor(SILVER),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                    ));
                    panel
                        .spawn((
                            Button,
                            MissionAction::BuyModule,
                            Name::new("Buy selected module"),
                            compact_button(23.),
                            BackgroundColor(Color::srgb(0.08, 0.23, 0.26)),
                            BorderColor::all(CYAN),
                        ))
                        .with_children(|button| {
                            control_label(button, "BUY SELECTED MODULE", 11., SILVER, true);
                            hotkey_label(button, "B", CYAN);
                        });
                    panel.spawn((
                        Copy::Power,
                        Text::default(),
                        TextFont::from_font_size(10.),
                        TextColor(MUTED),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                    ));
                    for slot in 0..4 {
                        panel
                            .spawn((Node {
                                width: percent(100),
                                min_height: px(21),
                                column_gap: px(5),
                                ..default()
                            },))
                            .with_children(|row| {
                                row.spawn((
                                    Button,
                                    MissionAction::AssignModule(slot),
                                    Name::new(format!("Equip module slot {}", slot + 1)),
                                    Node {
                                        width: percent(75),
                                        min_height: px(21),
                                        padding: UiRect::horizontal(px(8)),
                                        border: UiRect::all(px(1)),
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(PANEL),
                                    BorderColor::all(MUTED),
                                ))
                                .with_children(|button| {
                                    control_label(button, "", 10., SILVER, true)
                                });
                                row.spawn((
                                    Button,
                                    MissionAction::RemoveModule(slot),
                                    Name::new(format!("Remove module slot {}", slot + 1)),
                                    Node {
                                        flex_grow: 1.,
                                        min_height: px(21),
                                        padding: UiRect::horizontal(px(8)),
                                        border: UiRect::all(px(1)),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(PANEL),
                                    BorderColor::all(MUTED),
                                ))
                                .with_children(|button| {
                                    control_label(
                                        button,
                                        format!("REMOVE SHIFT+{}", slot + 1),
                                        9.,
                                        MUTED,
                                        true,
                                    )
                                });
                            });
                    }
                    panel.spawn((
                        Copy::Feedback,
                        Text::default(),
                        TextFont::from_font_size(11.),
                        TextColor(AMBER),
                        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                    ));
                    panel
                        .spawn((
                            Button,
                            MissionAction::Hub,
                            Name::new("Back to operations"),
                            compact_button(23.),
                            BackgroundColor(PANEL),
                            BorderColor::all(MUTED),
                        ))
                        .with_children(|button| {
                            control_label(button, "Back to operations", 11., SILVER, true);
                            hotkey_label(button, "BACKSPACE", MUTED);
                        });
                });
        });
}

fn compact_button(height: f32) -> Node {
    Node {
        width: percent(100),
        min_height: px(height),
        padding: UiRect::horizontal(px(8)),
        border: UiRect::all(px(1)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(8),
        ..default()
    }
}

#[allow(clippy::too_many_arguments)]
fn present(
    phase: Res<GamePhase>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    baseline: Res<Baseline>,
    chargers: Res<ChargerConfig>,
    mut overlay: Single<&mut Node, With<ShopOverlay>>,
    mut copy: Query<(&Copy, &mut Text, &mut TextColor)>,
    mut catalog: Query<(
        &CatalogRow,
        &Children,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut equip_buttons: Query<(&MissionAction, &Children)>,
    mut slot_text: SlotText,
) {
    overlay.display = if *phase == GamePhase::ModuleShop {
        Display::Flex
    } else {
        Display::None
    };
    if *phase != GamePhase::ModuleShop {
        return;
    }
    let (_, modules) = baseline.launch_power(&campaign);
    let mut preview_chargers = chargers.clone();
    preview_chargers.capacity =
        crate::world::regions::RegionProfile::for_mission(session.selected_mission)
            .charger_capacity;
    let selected = ModuleKind::ALL[session.selected_module.min(ModuleKind::ALL.len() - 1)];
    for (part, mut text, mut color) in &mut copy {
        let value = match part {
            Copy::Bank => bank_line(campaign.wallet),
            Copy::Detail => format!(
                "SELECTED / {}\n{}",
                selected.name(),
                catalog_detail(selected, &modules)
            ),
            Copy::Power => potential_power(&baseline, &campaign, &preview_chargers).display(),
            Copy::Feedback => session.purchase_feedback.clone(),
        };
        if text.0 != value {
            text.0 = value;
        }
        if matches!(part, Copy::Feedback) {
            color.0 = if session.purchase_feedback.is_empty() {
                MUTED
            } else {
                AMBER
            };
        }
    }
    for (row, children, mut fill, mut border) in &mut catalog {
        let owned = campaign.inventory.owns(row.0);
        let cost = price(row.0);
        let status = if owned {
            "OWNED"
        } else if campaign.wallet.salvage >= cost.salvage
            && campaign.wallet.components >= cost.components
        {
            "UNOWNED / AFFORDABLE"
        } else {
            "UNOWNED / INSUFFICIENT"
        };
        let text = format!(
            "{}  {}  |  S {}  C {}",
            row.0.name(),
            cost_line(status),
            cost.salvage,
            cost.components
        );
        for child in children.iter() {
            if let Ok(mut label) = slot_text.get_mut(child) {
                label.0 = text.clone();
            }
        }
        let selected_row = row.0 == selected;
        fill.0 = if selected_row {
            Color::srgb(0.08, 0.23, 0.26)
        } else {
            PANEL
        };
        *border = BorderColor::all(if selected_row {
            AMBER
        } else if owned {
            CYAN
        } else {
            MUTED
        });
    }
    for (action, children) in &mut equip_buttons {
        let MissionAction::AssignModule(slot) = *action else {
            continue;
        };
        let current = campaign.inventory.loadout().slots()[slot];
        let value = current.map_or_else(
            || format!("{}  SLOT {}  EMPTY", slot + 1, slot + 1),
            |kind| format!("{}  SLOT {}  {}", slot + 1, slot + 1, kind.name()),
        );
        for child in children {
            if let Ok(mut text) = slot_text.get_mut(*child) {
                text.0 = value.clone();
            }
        }
    }
}

fn cost_line(status: &str) -> &str {
    status
}

fn bank_line(wallet: Amounts) -> String {
    format!(
        "BANK  SALVAGE {}  |  COMPONENTS {}",
        wallet.salvage, wallet.components
    )
}

type HoverButtons<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (With<MissionAction>, Changed<Interaction>),
>;
type SlotText<'w, 's> =
    Query<'w, 's, &'static mut Text, (Without<Copy>, Without<CatalogRow>, Without<MissionAction>)>;

fn hover(mut buttons: HoverButtons) {
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
    use crate::{
        economy::Amounts,
        energy::ChargerConfig,
        game::GamePhase,
        mission::{Campaign, MissionSession},
        modules::{ModuleKind, shop::price},
        upgrades::runtime::Baseline,
    };

    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(GamePhase::ModuleShop)
            .init_resource::<Campaign>()
            .init_resource::<MissionSession>()
            .init_resource::<Baseline>()
            .init_resource::<ChargerConfig>()
            .add_plugins(ModuleShopScenePlugin);
        app.update();
        app
    }

    fn text(app: &mut App) -> String {
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|text| text.0.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn shop_power_preview_tracks_selected_region_before_launch() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<MissionSession>()
            .selected_mission = crate::mission::campaign::MissionId::ALL[8];
        app.update();
        assert!(text(&mut app).contains("300 per field"));
        app.world_mut()
            .resource_mut::<MissionSession>()
            .selected_mission = crate::mission::campaign::MissionId::ALL[0];
        app.world_mut().resource_mut::<ChargerConfig>().capacity = 300.;
        app.update();
        assert!(text(&mut app).contains("200 per field"));
    }

    #[test]
    fn shop_presents_every_catalog_price_selected_detail_and_empty_slots() {
        let mut app = app();
        let copy = text(&mut app);

        for kind in ModuleKind::ALL {
            assert!(copy.contains(kind.name()));
            let cost = price(kind);
            assert!(copy.contains(&format!("S {}  C {}", cost.salvage, cost.components)));
        }
        assert!(copy.contains("SELECTED / OVERDRIVE"));
        assert!(copy.contains("SLOT 1  EMPTY"));
        assert!(copy.contains("ALL-ON DRAIN 0/s"), "shop copy was:\n{copy}");
    }

    #[test]
    fn every_shop_control_has_nonempty_marked_copy() {
        let mut app = app();
        let labels = app
            .world_mut()
            .query_filtered::<&Text, With<ShopControlText>>()
            .iter(app.world())
            .collect::<Vec<_>>();

        assert_eq!(labels.len(), 16);
        assert!(labels.iter().all(|label| !label.0.is_empty()));
    }

    #[test]
    fn shop_refreshes_selected_catalog_status_loadout_and_feedback() {
        let mut app = app();
        {
            let mut campaign = app.world_mut().resource_mut::<Campaign>();
            campaign.wallet = Amounts {
                salvage: 20,
                components: 0,
            };
            let mut wallet = campaign.wallet;
            campaign
                .inventory
                .purchase(ModuleKind::Shield, &mut wallet)
                .unwrap();
            campaign.wallet = wallet;
            campaign
                .inventory
                .assign(2, Some(ModuleKind::Shield))
                .unwrap();
        }
        {
            let mut session = app.world_mut().resource_mut::<MissionSession>();
            session.selected_module = ModuleKind::Shield as usize;
            session.purchase_feedback = "SHIELD assigned to slot 3. No charge.".into();
        }
        app.update();
        let copy = text(&mut app);

        assert!(copy.contains("SELECTED / SHIELD"));
        assert!(copy.contains("SHIELD  OWNED"));
        assert!(copy.contains("SLOT 3  SHIELD"));
        assert!(copy.contains("assigned to slot 3"));
    }
}
