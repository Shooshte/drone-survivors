//! Opt-in native menu fixture with synthetic funds and completion time.
use super::{ModuleKind, Modules};
use crate::{
    combat::Encounter,
    economy::Amounts,
    energy::Energy,
    game::GamePhase,
    mission::{Campaign, MissionAction, MissionSession},
};
use bevy::{
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct Fixture {
    start: Instant,
    step: usize,
    seconds: f64,
    directory: Option<PathBuf>,
    minimum: bool,
    clicked: bool,
}

pub(crate) fn install(app: &mut App, seconds: f64) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).expect("shop capture directory");
    }
    println!(
        "MODULE SHOP FIXTURE: synthetic bank=65/1 and success timer; normal menu input and transaction rules. Not natural progression or balance evidence."
    );
    app.add_plugins((
        crate::mission::MissionPlugin,
        crate::mission::scene::MissionScenePlugin,
        crate::economy::scene::EconomyScenePlugin,
    ))
    .insert_resource(Fixture {
        start: Instant::now(),
        step: 0,
        seconds,
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
        clicked: false,
    })
    .add_systems(Startup, resize)
    .add_systems(PreUpdate, drive.after(InputSystems));
}
fn resize(mut fixture: ResMut<Fixture>, mut window: Single<&mut Window>) {
    fixture.start = Instant::now();
    if fixture.minimum {
        window.resolution.set(640., 480.);
    }
}

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut campaign: ResMut<Campaign>,
    session: Res<MissionSession>,
    phase: Res<GamePhase>,
    modules: Res<Modules>,
    energy: Res<Energy>,
    mut encounter: ResMut<Encounter>,
    mut buttons: Query<(&MissionAction, &mut Interaction)>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    control_labels: Query<(&Text, &ComputedNode), With<super::shop_scene::ShopControlText>>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    for key in [
        KeyCode::KeyM,
        KeyCode::KeyB,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::ShiftLeft,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::KeyR,
    ] {
        keys.reset(key);
    }
    if fixture.clicked {
        for (action, mut interaction) in &mut buttons {
            if *action == MissionAction::BuyModule {
                *interaction = Interaction::None;
            }
        }
        fixture.clicked = false;
    }
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < fixture.seconds,
        "shop fixture timed out at step {}",
        fixture.step
    );
    if elapsed < (fixture.step + 1) as f64 * 0.6 {
        return;
    }
    let step = fixture.step + 1;
    match step {
        2 | 36 => keys.press(KeyCode::KeyM),
        4 | 10 | 13 | 16 | 19 => keys.press(KeyCode::KeyB),
        5 => {
            assert_eq!(campaign.wallet, Amounts::default());
            assert!(session.purchase_feedback.contains("Not enough"));
            campaign.wallet = Amounts {
                salvage: 65,
                components: 1,
            };
        }
        6 => {
            for (action, mut interaction) in &mut buttons {
                if *action == MissionAction::BuyModule {
                    *interaction = Interaction::Pressed;
                }
            }
            fixture.clicked = true;
        }
        7 => {
            assert!(campaign.inventory.owns(ModuleKind::Overdrive));
            assert_eq!(campaign.wallet.salvage, 55);
            assert_eq!(campaign.inventory.loadout().slots(), &[None; 4]);
        }
        8 | 22 => keys.press(KeyCode::Digit4),
        9 | 12 | 15 => keys.press(KeyCode::ArrowDown),
        11 => keys.press(KeyCode::Digit2),
        14 => keys.press(KeyCode::Digit3),
        17 | 24 => keys.press(KeyCode::Digit1),
        18 | 20 => {
            assert_eq!(campaign.wallet, Amounts::default());
            for kind in ModuleKind::ALL {
                assert!(campaign.inventory.owns(kind));
            }
        }
        21 => {
            keys.press(KeyCode::ShiftLeft);
            keys.press(KeyCode::Digit3);
        }
        23 => keys.press(KeyCode::ArrowUp),
        25 => assert_eq!(campaign.inventory.loadout().slots(), &expected()),
        26 | 41 => keys.press(KeyCode::Backspace),
        27 | 29 | 35 | 42 | 44 => keys.press(KeyCode::Enter),
        30 | 32 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(modules.loadout.slots(), &expected());
            assert_eq!(modules.enabled, [false; 4]);
            assert_eq!(energy.current, 100.);
        }
        31 => keys.press(KeyCode::KeyR),
        33 => encounter.elapsed = 300.,
        34 => {
            assert_eq!(*phase, GamePhase::Survived);
            assert_eq!(campaign.history.len(), 1);
        }
        37 => {
            assert_eq!(*phase, GamePhase::ModuleShop);
            assert_eq!(campaign.inventory.loadout().slots(), &expected());
            assert_eq!(
                campaign.wallet,
                Amounts {
                    salvage: 10,
                    components: 1
                }
            );
        }
        38..=40 => {
            keys.press(KeyCode::ShiftLeft);
            keys.press(match step {
                38 => KeyCode::Digit1,
                39 => KeyCode::Digit2,
                _ => KeyCode::Digit4,
            });
        }
        45 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(modules.loadout.slots(), &[None; 4]);
        }
        47 => {
            println!(
                "MODULE SHOP FIXTURE PASS: empty bank rejection, mouse purchase, four purchases, repeat purchase, free remove/move/replace, briefing, launch, restart, results persistence, empty launch."
            );
            exit.write(AppExit::Success);
        }
        _ => {}
    }
    let label = match step {
        1 => Some("hub"),
        3 => Some("empty"),
        7 => Some("purchased"),
        18 => Some("full"),
        25 => Some("rearranged"),
        28 => Some("briefing"),
        30 => Some("launch"),
        32 => Some("restart"),
        37 => Some("persistent"),
        43 => Some("empty-briefing"),
        45 => Some("empty-launch"),
        _ => None,
    };
    if let Some(label) = label {
        if *phase == GamePhase::ModuleShop {
            assert!(control_labels.iter().count() >= 14, "missing shop labels");
            for (text, node) in &control_labels {
                assert!(
                    !text.0.is_empty() && node.size().x > 1. && node.size().y > 1.,
                    "collapsed shop button label: {:?} {:?}",
                    text.0,
                    node.size()
                );
            }
        }
        if matches!(
            *phase,
            GamePhase::ModuleShop | GamePhase::Briefing | GamePhase::Hub
        ) {
            for (text, node, transform) in &text_nodes {
                if node.size().x < 1. || node.size().y < 1. {
                    continue;
                }
                let center = transform.translation / window.scale_factor();
                let half = node.size() / window.scale_factor() / 2.;
                assert!(
                    center.x - half.x >= -1.
                        && center.y - half.y >= -1.
                        && center.x + half.x <= window.width() + 1.
                        && center.y + half.y <= window.height() + 1.,
                    "text outside viewport: {:?} center={center:?} half={half:?}",
                    text.0
                );
            }
        }
        println!(
            "MODULE SHOP FIXTURE {label}: phase={phase:?} bank={:?} planned={:?} active={:?}",
            campaign.wallet,
            campaign.inventory.loadout().slots(),
            modules.loadout.slots()
        );
        if let Some(directory) = &fixture.directory {
            let size = if fixture.minimum {
                "640x480"
            } else {
                "1120x720"
            };
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(directory.join(format!("{size}-{label}.png"))));
        }
    }
    fixture.step = step;
}
fn expected() -> [Option<ModuleKind>; 4] {
    [
        Some(ModuleKind::Mobility),
        Some(ModuleKind::Shield),
        None,
        Some(ModuleKind::Rocket),
    ]
}
