use super::{ChoiceAction, UpgradeKind, UpgradeRun};
use crate::{
    arena::{ArenaSceneSetup, FooterFont, UpgradeFooterSlot},
    combat::CombatSceneSetup,
    game::{GamePhase, GameplaySet},
};
use bevy::prelude::*;

use super::runtime::ExplorationPickup;

pub(crate) struct UpgradeScenePlugin;

#[derive(Component)]
struct UpgradeHud;

#[derive(Component)]
struct UpgradeModal;

#[derive(Component)]
struct UpgradeCard(usize);

#[derive(Component)]
struct PickupVisual;

#[derive(Component, Clone, Copy)]
enum UpgradeCopy {
    Hud,
    Queue,
    CardName(usize),
    CardBenefit(usize),
    CardDrawback(usize),
}

type PresentedNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static UpgradeModal>,
        Option<&'static UpgradeCard>,
    ),
    Or<(With<UpgradeModal>, With<UpgradeCard>)>,
>;

impl Plugin for UpgradeScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup.after(CombatSceneSetup).after(ArenaSceneSetup),
        )
        .add_systems(
            Update,
            (present, hover_buttons)
                .chain()
                .in_set(GameplaySet::Presentation),
        );
    }
}

fn setup(
    mut commands: Commands,
    pickup: Res<ExplorationPickup>,
    footer_slot: Single<Entity, With<UpgradeFooterSlot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    setup_pickup(&mut commands, &pickup, &mut meshes, &mut materials);

    let hud = commands
        .spawn((
            UpgradeHud,
            UpgradeCopy::Hud,
            FooterFont::new(14., 11.),
            Text::default(),
            TextFont::from_font_size(14.),
            TextColor(Color::srgb(0.66, 0.86, 0.88)),
            TextLayout::new(Justify::Right, LineBreak::WordBoundary),
            Node {
                width: percent(100),
                ..default()
            },
        ))
        .id();
    commands.entity(*footer_slot).add_child(hud);

    commands
        .spawn((
            Name::new("Upgrade choice overlay"),
            UpgradeModal,
            GlobalZIndex(100),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.025, 0.038, 0.91)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Name::new("Field retrofit panel"),
                    Node {
                        width: percent(100),
                        max_width: px(760),
                        padding: UiRect::all(px(12)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(8)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.045, 0.075, 0.095)),
                    BorderColor::all(Color::srgb(0.19, 0.48, 0.52)),
                ))
                .with_children(|panel| {
                    panel
                        .spawn(Node {
                            width: percent(100),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|header| {
                            header.spawn((
                                Text::new("FIELD RETROFIT AVAILABLE"),
                                TextFont::from_font_size(20.),
                                TextColor(Color::srgb(0.8, 1., 0.94)),
                            ));
                            header.spawn((
                                Text::new("THIS RUN ONLY"),
                                TextFont::from_font_size(12.),
                                TextColor(Color::srgb(0.34, 0.88, 0.8)),
                            ));
                        });
                    panel.spawn((
                        UpgradeCopy::Queue,
                        Text::default(),
                        TextFont::from_font_size(13.),
                        TextColor(Color::srgb(0.58, 0.72, 0.76)),
                    ));

                    for index in 0..3 {
                        spawn_card(panel, index);
                    }

                    panel
                        .spawn((
                            Name::new("Skip upgrade"),
                            Button,
                            ChoiceAction::Skip,
                            Node {
                                width: percent(100),
                                height: px(34),
                                padding: UiRect::horizontal(px(12)),
                                border: UiRect::all(px(1)),
                                border_radius: BorderRadius::all(px(5)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.075, 0.105, 0.125)),
                            BorderColor::all(Color::srgb(0.24, 0.34, 0.38)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("SKIP THIS UPGRADE"),
                                TextFont::from_font_size(14.),
                                TextColor(Color::srgb(0.78, 0.84, 0.84)),
                            ));
                            button.spawn((
                                Text::new("BACKSPACE"),
                                TextFont::from_font_size(12.),
                                TextColor(Color::srgb(0.95, 0.68, 0.38)),
                            ));
                        });
                    panel.spawn((
                        Text::new("1-3 SELECT  |  R RESTARTS THE ENCOUNTER"),
                        TextFont::from_font_size(11.),
                        TextColor(Color::srgb(0.48, 0.61, 0.65)),
                        Node {
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));
                });
        });
}

fn spawn_card(parent: &mut ChildSpawnerCommands, index: usize) {
    parent
        .spawn((
            Name::new(format!("Upgrade choice {}", index + 1)),
            Button,
            ChoiceAction::Pick(index),
            UpgradeCard(index),
            Node {
                display: Display::None,
                width: percent(100),
                height: px(74),
                padding: UiRect::all(px(7)),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Stretch,
                column_gap: px(10),
                ..default()
            },
            BackgroundColor(Color::srgb(0.065, 0.105, 0.13)),
            BorderColor::all(Color::srgb(0.17, 0.34, 0.38)),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    width: px(32),
                    min_width: px(32),
                    height: percent(100),
                    border_radius: BorderRadius::all(px(4)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.23, 0.26)),
            ))
            .with_child((
                Text::new((index + 1).to_string()),
                TextFont::from_font_size(18.),
                TextColor(Color::srgb(0.75, 1., 0.91)),
            ));
            card.spawn(Node {
                flex_grow: 1.,
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                ..default()
            })
            .with_children(|copy| {
                copy.spawn((
                    UpgradeCopy::CardName(index),
                    Text::default(),
                    TextFont::from_font_size(16.),
                    TextColor(Color::srgb(0.9, 0.98, 0.95)),
                ));
                copy.spawn((
                    UpgradeCopy::CardBenefit(index),
                    Text::default(),
                    TextFont::from_font_size(12.),
                    TextColor(Color::srgb(0.38, 0.92, 0.72)),
                ));
                copy.spawn((
                    UpgradeCopy::CardDrawback(index),
                    Text::default(),
                    TextFont::from_font_size(12.),
                    TextColor(Color::srgb(1., 0.62, 0.36)),
                ));
            });
        });
}

fn setup_pickup(
    commands: &mut Commands,
    pickup: &ExplorationPickup,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let ring = meshes.add(Torus::new(pickup.radius - 1.1, pickup.radius + 1.1));
    let core = meshes.add(Sphere::new(5.5));
    let ring_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.95, 0.78, 0.62),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let core_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.68, 1., 0.78),
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Name::new("Exploration XP pickup"),
            PickupVisual,
            Transform::from_translation(pickup.position),
            Visibility::Visible,
        ))
        .with_children(|marker| {
            for rotation in [
                Quat::IDENTITY,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            ] {
                marker.spawn((
                    Mesh3d(ring.clone()),
                    MeshMaterial3d(ring_material.clone()),
                    Transform::from_rotation(rotation),
                ));
            }
            marker.spawn((
                Mesh3d(core),
                MeshMaterial3d(core_material),
                Transform::default(),
            ));
        });
}

fn present(
    phase: Res<GamePhase>,
    run: Res<UpgradeRun>,
    pickup: Res<ExplorationPickup>,
    mut nodes: PresentedNodes,
    mut copy: Query<(&UpgradeCopy, &mut Text)>,
    mut marker: Single<(&mut Transform, &mut Visibility), With<PickupVisual>>,
) {
    let choosing = *phase == GamePhase::Choosing && !run.offer.is_empty();
    for (mut node, modal, card) in &mut nodes {
        if modal.is_some() {
            node.display = if choosing {
                Display::Flex
            } else {
                Display::None
            };
        } else if let Some(card) = card {
            node.display = if choosing && card.0 < run.offer.len() {
                Display::Flex
            } else {
                Display::None
            };
        }
    }

    for (target, mut text) in &mut copy {
        let value = match *target {
            UpgradeCopy::Hud => hud_copy(&run),
            UpgradeCopy::Queue => queue_copy(&run),
            UpgradeCopy::CardName(index) => card_kind(&run, index)
                .map(|kind| kind.name().to_uppercase())
                .unwrap_or_default(),
            UpgradeCopy::CardBenefit(index) => card_kind(&run, index)
                .map(|kind| format!("BENEFIT  {}", kind.benefit()))
                .unwrap_or_default(),
            UpgradeCopy::CardDrawback(index) => card_kind(&run, index)
                .map(|kind| format!("DRAWBACK  {}", drawback_copy(kind)))
                .unwrap_or_default(),
        };
        if text.0 != value {
            text.0 = value;
        }
    }

    marker.0.translation = pickup.position;
    *marker.1 = if pickup.collected {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}

fn hover_buttons(
    mut buttons: Query<
        (
            &ChoiceAction,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (action, interaction, mut background, mut border) in &mut buttons {
        let skip = matches!(action, ChoiceAction::Skip);
        let (fill, edge) = match (*interaction, skip) {
            (Interaction::Pressed, false) => {
                (Color::srgb(0.1, 0.3, 0.3), Color::srgb(0.52, 1., 0.82))
            }
            (Interaction::Hovered, false) => {
                (Color::srgb(0.085, 0.19, 0.2), Color::srgb(0.32, 0.9, 0.74))
            }
            (Interaction::None, false) => (
                Color::srgb(0.065, 0.105, 0.13),
                Color::srgb(0.17, 0.34, 0.38),
            ),
            (Interaction::Pressed, true) => {
                (Color::srgb(0.28, 0.18, 0.1), Color::srgb(1., 0.7, 0.4))
            }
            (Interaction::Hovered, true) => {
                (Color::srgb(0.16, 0.15, 0.12), Color::srgb(0.76, 0.55, 0.32))
            }
            (Interaction::None, true) => (
                Color::srgb(0.075, 0.105, 0.125),
                Color::srgb(0.24, 0.34, 0.38),
            ),
        };
        background.0 = fill;
        border.set_all(edge);
    }
}

fn card_kind(run: &UpgradeRun, index: usize) -> Option<UpgradeKind> {
    run.offer.get(index).copied()
}

fn drawback_copy(kind: UpgradeKind) -> &'static str {
    match kind {
        UpgradeKind::AgileFrame => "-20 maximum hull (100 -> 80 before other upgrades)",
        _ => kind.drawback(),
    }
}

fn hud_copy(run: &UpgradeRun) -> String {
    let selected = if run.selected.is_empty() {
        "NONE".to_string()
    } else {
        run.selected
            .iter()
            .map(|kind| kind.name())
            .collect::<Vec<_>>()
            .join("  |  ")
    };
    if run.exhausted {
        return format!("BUILD COMPLETE | No more upgrades this run\nRUN UPGRADES  |  {selected}");
    }
    format!(
        "LEVEL  {}   |   XP  {} / {}\nRUN UPGRADES  |  {selected}",
        run.level,
        run.xp,
        run.threshold(),
    )
}

fn queue_copy(run: &UpgradeRun) -> String {
    let choice_noun = if run.pending == 1 {
        "choice"
    } else {
        "choices"
    };
    let offer_noun = if run.offer.len() == 1 {
        "upgrade"
    } else {
        "upgrades"
    };
    format!(
        "{} eligible {offer_noun}  |  {} earned {choice_noun} remaining",
        run.offer.len(),
        run.pending,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene_app() -> App {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<ExplorationPickup>()
            .init_resource::<UpgradeRun>()
            .init_resource::<GamePhase>()
            .add_plugins(UpgradeScenePlugin);
        app.world_mut().spawn((UpgradeFooterSlot, Node::default()));
        app.update();
        app
    }

    fn visible_cards(app: &mut App) -> usize {
        app.world_mut()
            .query::<(&UpgradeCard, &Node)>()
            .iter(app.world())
            .filter(|(_, node)| node.display == Display::Flex)
            .count()
    }

    #[test]
    fn completed_build_hud_removes_progress_promises_and_reset_restores_them() {
        let mut app = scene_app();
        {
            let mut run = app.world_mut().resource_mut::<UpgradeRun>();
            run.selected = UpgradeKind::ALL.to_vec();
            run.exhausted = true;
            run.award(500);
        }
        app.update();
        let text = app
            .world_mut()
            .query_filtered::<&Text, With<UpgradeHud>>()
            .single(app.world())
            .unwrap();
        assert!(text.0.contains("BUILD COMPLETE"));
        assert!(!text.0.contains("XP"));
        assert!(!text.0.contains("LEVEL"));
        for kind in UpgradeKind::ALL {
            assert!(text.0.contains(kind.name()));
        }
        *app.world_mut().resource_mut::<UpgradeRun>() = UpgradeRun::default();
        app.update();
        let text = app
            .world_mut()
            .query_filtered::<&Text, With<UpgradeHud>>()
            .single(app.world())
            .unwrap();
        assert!(text.0.contains("XP"));
        assert!(!text.0.contains("BUILD COMPLETE"));
    }

    #[test]
    fn modal_presents_every_offer_size_with_skip_and_exact_copy() {
        let mut app = scene_app();
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;

        for size in 1..=3 {
            {
                let mut run = app.world_mut().resource_mut::<UpgradeRun>();
                run.pending = size as u32;
                run.offer = [
                    UpgradeKind::AgileFrame,
                    UpgradeKind::HeavyRounds,
                    UpgradeKind::Interceptor,
                ][..size]
                    .to_vec();
            }
            app.update();

            assert_eq!(visible_cards(&mut app), size);
            let modal = app
                .world_mut()
                .query_filtered::<&Node, With<UpgradeModal>>()
                .single(app.world())
                .unwrap();
            assert_eq!(modal.display, Display::Flex);
            assert!(
                app.world_mut()
                    .query::<&ChoiceAction>()
                    .iter(app.world())
                    .any(|action| *action == ChoiceAction::Skip)
            );
            let rendered = app
                .world_mut()
                .query::<(&UpgradeCopy, &Text)>()
                .iter(app.world())
                .map(|(_, text)| text.0.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(rendered.contains(&format!("{size} eligible upgrade")));
            assert!(rendered.contains("BENEFIT"));
            assert!(rendered.contains("DRAWBACK  -20 maximum hull"));
            if size >= 2 {
                assert!(rendered.contains("DRAWBACK  -10% acceleration"));
            }
        }

        app.world_mut().resource_mut::<UpgradeRun>().offer.clear();
        app.update();
        assert_eq!(visible_cards(&mut app), 0);
        let modal = app
            .world_mut()
            .query_filtered::<&Node, With<UpgradeModal>>()
            .single(app.world())
            .unwrap();
        assert_eq!(modal.display, Display::None);
    }
}
