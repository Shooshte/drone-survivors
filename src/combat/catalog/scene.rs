use super::*;

const INK: Color = Color::srgb(0.025, 0.045, 0.065);
const PANEL: Color = Color::srgb(0.045, 0.075, 0.095);
const TEXT: Color = Color::srgb(0.90, 0.94, 0.92);
const CYAN: Color = Color::srgb(0.40, 0.90, 0.88);

#[derive(Component)]
pub(super) struct Selector;
#[derive(Component)]
pub(super) struct RoundControls;
#[derive(Component)]
pub(super) enum Copy {
    Scenario,
    Instruction,
    Slot(usize),
    Duration,
}

fn label(parent: &mut ChildSpawnerCommands, value: &str, size: f32) -> Entity {
    parent
        .spawn((
            Text::new(value),
            TextFont::from_font_size(size),
            TextColor(TEXT),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
        ))
        .id()
}

fn button(parent: &mut ChildSpawnerCommands, action: CatalogAction, value: &str) -> Entity {
    let mut text = Entity::PLACEHOLDER;
    parent
        .spawn((
            Button,
            action,
            Node {
                width: percent(100),
                min_height: px(30),
                padding: UiRect::axes(px(8), px(5)),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(INK),
            BorderColor::all(CYAN),
        ))
        .with_children(|parent| {
            text = label(parent, value, 14.);
        });
    text
}

pub(super) fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    if std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some() {
        for mut window in &mut windows {
            window.resolution.set(640., 480.);
        }
    }
    commands.spawn((Selector, Name::new("Catalog arena selector"), GlobalZIndex(300),
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100),
            padding: UiRect::all(px(12)), align_items: AlignItems::Center,
            justify_content: JustifyContent::Center, ..default() }, BackgroundColor(INK)))
        .with_children(|root| {
            root.spawn((Node { width: percent(100), max_width: px(740), padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column, row_gap: px(7), ..default() }, BackgroundColor(PANEL)))
                .with_children(|panel| {
                    label(panel, "COMBAT TEST ARENA", 24.);
                    label(panel, "Practice and compare. Campaign progress is separate; this arena does not save.", 13.);
                    let entity = label(panel, "", 19.); panel.commands().entity(entity).insert(Copy::Scenario);
                    panel.spawn(Node { width: percent(100), column_gap: px(8), ..default() }).with_children(|row| {
                        button(row, CatalogAction::Previous, "LEFT  Previous scenario");
                        button(row, CatalogAction::Next, "RIGHT  Next scenario");
                    });
                    let entity = label(panel, "", 13.); panel.commands().entity(entity).insert(Copy::Instruction);
                    label(panel, "Choose modules: click a slot or press 1-4 to cycle. Each type fits once.", 13.);
                    for slot in 0..4 {
                        let entity = button(panel, CatalogAction::CycleSlot(slot), "");
                        panel.commands().entity(entity).insert(Copy::Slot(slot));
                    }
                    let entity = label(panel, "", 13.); panel.commands().entity(entity).insert(Copy::Duration);
                    button(panel, CatalogAction::Launch, "ENTER  Launch scenario");
                    label(panel, "In a round: 1-4 toggle modules / R restart / TAB return / ESC quit", 12.);
                });
        });
    commands
        .spawn((
            RoundControls,
            GlobalZIndex(250),
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                right: px(12),
                width: px(158),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                display: Display::None,
                ..default()
            },
            BackgroundColor(PANEL),
        ))
        .with_children(|panel| {
            button(panel, CatalogAction::Return, "TAB  Arena setup");
            button(panel, CatalogAction::Restart, "R  Restart round");
        });
}

type LayoutNodes<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Node,
        Option<&'static Selector>,
        Option<&'static RoundControls>,
        Option<&'static super::super::CombatHudRoot>,
        Option<&'static CatalogAction>,
    ),
    Or<(
        With<Selector>,
        With<RoundControls>,
        With<super::super::CombatHudRoot>,
        With<CatalogAction>,
    )>,
>;

pub(super) fn present(
    arena: Res<CatalogArena>,
    phase: Res<GamePhase>,
    mut nodes: LayoutNodes,
    mut labels: Query<(&Copy, &mut Text)>,
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<CatalogAction>>,
) {
    for (mut node, selector, round, hud, action) in &mut nodes {
        if selector.is_some() {
            node.display = if arena.selecting {
                Display::Flex
            } else {
                Display::None
            };
        } else if round.is_some() {
            node.display = if arena.selecting {
                Display::None
            } else {
                Display::Flex
            };
        } else if hud.is_some() {
            node.right = px(182);
        } else if action == Some(&CatalogAction::Restart) {
            node.display = if *phase == GamePhase::Choosing {
                Display::None
            } else {
                Display::Flex
            };
        }
    }
    for (copy, mut text) in &mut labels {
        let value = match copy {
            Copy::Scenario => format!(
                "{} / {}   {}",
                arena.scenario + 1,
                SCENARIOS.len(),
                SCENARIOS[arena.scenario].name
            ),
            Copy::Instruction => SCENARIOS[arena.scenario].instruction.to_owned(),
            Copy::Slot(slot) => format!(
                "{}   {}",
                slot + 1,
                arena.slots[*slot].map_or("EMPTY", ModuleKind::name)
            ),
            Copy::Duration => format!(
                "Round: {:.0} active seconds. Fresh hull, battery, chargers and upgrades on launch.",
                arena.duration
            ),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (interaction, mut color) in &mut buttons {
        color.0 = if *interaction == Interaction::None {
            INK
        } else {
            Color::srgb(0.10, 0.22, 0.25)
        };
    }
}
