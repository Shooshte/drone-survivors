//! Opt-in native purchase/lifecycle fixture; synthetic funds and outcomes are disclosed.
use super::NodeId;
use crate::{
    combat::{CombatConfig, Encounter},
    economy::Amounts,
    energy::{Energy, EnergyConfig},
    game::GamePhase,
    mission::{Campaign, MissionAction, MissionSession},
    upgrades::{UpgradeKind, UpgradeRun},
};
use bevy::{
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};
#[derive(Resource)]
struct Preview {
    start: Instant,
    step: usize,
    seconds: f64,
    directory: Option<PathBuf>,
    minimum: bool,
}

pub(crate) fn install(app: &mut App, seconds: f64) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).expect("passive capture directory");
    }
    println!(
        "PASSIVE UI FIXTURE: synthetic bank=1500/50, XP/offer and success timer; purchases use normal input rules. Not natural progression or balance evidence."
    );
    app.add_plugins((
        crate::mission::MissionPlugin,
        crate::mission::scene::MissionScenePlugin,
        crate::economy::scene::EconomyScenePlugin,
    ))
    .insert_resource(Preview {
        start: Instant::now(),
        step: 0,
        seconds,
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
    })
    .add_systems(Startup, resize)
    .add_systems(PreUpdate, drive.after(InputSystems));
}
fn resize(mut preview: ResMut<Preview>, mut window: Single<&mut Window>) {
    preview.start = Instant::now();
    if preview.minimum {
        window.resolution.set(640., 480.);
    }
}
#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut preview: ResMut<Preview>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    phase: Res<GamePhase>,
    mut campaign: ResMut<Campaign>,
    session: Res<MissionSession>,
    mut encounter: ResMut<Encounter>,
    mut run: ResMut<UpgradeRun>,
    energy: Res<Energy>,
    config: Res<EnergyConfig>,
    combat: Res<CombatConfig>,
    mut exit: MessageWriter<AppExit>,
    cards: Query<(
        &super::scene::PassiveCard,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    window: Single<&Window>,
    mut buttons: Query<(&MissionAction, &mut Interaction)>,
) {
    for key in [
        KeyCode::KeyU,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::KeyR,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit7,
    ] {
        keys.reset(key);
    }
    // Clear only the fixture's simulated click after its normal Update consumed it.
    if preview.step == 7 {
        for (action, mut interaction) in &mut buttons {
            if *action == MissionAction::Purchase(NodeId::Battery) {
                *interaction = Interaction::None;
            }
        }
    }
    let elapsed = preview.start.elapsed().as_secs_f64();
    if elapsed >= preview.seconds {
        panic!("passive fixture timed out after step {}", preview.step);
    }
    if elapsed < (preview.step + 1) as f64 {
        return;
    }
    let step = preview.step + 1;
    match step {
        2 | 35 => keys.press(KeyCode::KeyU),
        4 => keys.press(KeyCode::Digit2),
        5 => {
            assert_eq!(campaign.wallet, Amounts::default());
            assert!(session.purchase_feedback.contains("rank 1 first"));
        }
        6 => {
            campaign.wallet = Amounts {
                salvage: 1500,
                components: 50,
            }
        }
        7 => {
            for (action, mut interaction) in &mut buttons {
                if *action == MissionAction::Purchase(NodeId::Battery) {
                    *interaction = Interaction::Pressed;
                }
            }
        }
        8 => {
            assert_eq!(campaign.passives.rank(NodeId::Battery), 1);
            assert_eq!(campaign.wallet.salvage, 1490);
            assert_eq!(config.capacity, 100.);
        }
        9 | 11 | 13 | 15 | 17 => keys.press(KeyCode::Digit7),
        16 | 18 => {
            assert_eq!(campaign.passives.rank(NodeId::Battery), 5);
            assert_eq!(
                campaign.wallet,
                Amounts {
                    salvage: 1350,
                    components: 46
                }
            );
        }
        19 => keys.press(KeyCode::Backspace),
        21 | 23 | 33 => keys.press(KeyCode::Enter),
        24 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(energy.current, 200.);
            assert_eq!(combat.shot_damage, 10);
        }
        25 => run.award(50),
        26 => {
            assert_eq!(*phase, GamePhase::Choosing);
            run.offer = vec![UpgradeKind::Interceptor];
        }
        27 => keys.press(KeyCode::Digit1),
        28 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(config.capacity, 150.);
        }
        29 => keys.press(KeyCode::KeyR),
        30 => {
            assert_eq!(energy.current, 200.);
            assert!(run.selected.is_empty());
        }
        31 => encounter.elapsed = 300.,
        32 => {
            assert_eq!(*phase, GamePhase::Survived);
            assert_eq!(campaign.history.len(), 1);
        }
        34 => {
            assert_eq!(*phase, GamePhase::Hub);
            assert_eq!(campaign.passives.rank(NodeId::Battery), 5);
        }
        38 => {
            println!(
                "PASSIVE UI FIXTURE PASS: locked/empty, mouse purchase, rank 5 cap, launch, temporary stack, restart, success, hub persistence."
            );
            exit.write(AppExit::Success);
        }
        _ => {}
    }
    let label = match step {
        1 => Some("hub"),
        3 => Some("empty"),
        8 => Some("purchased"),
        16 => Some("maxed"),
        24 => Some("launch"),
        28 => Some("temporary"),
        30 => Some("restart"),
        36 => Some("persistent"),
        _ => None,
    };
    if let Some(label) = label {
        if *phase == GamePhase::Passives {
            let scale = window.scale_factor();
            assert_eq!(cards.iter().count(), 9);
            for (card, node, transform) in &cards {
                let center = transform.translation / scale;
                let half = node.size() / scale / 2.;
                assert!(half.x > 50. && half.y > 25., "collapsed card {:?}", card.0);
                assert!(
                    center.x - half.x >= 0.
                        && center.y - half.y >= 0.
                        && center.x + half.x <= window.width() + 1.
                        && center.y + half.y <= window.height() + 1.,
                    "card outside viewport {:?}: {center:?} {half:?}",
                    card.0
                );
            }
        }
        println!(
            "PASSIVE UI FIXTURE {label}: phase={phase:?} bank={:?} battery_rank={} capacity={} battery={}",
            campaign.wallet,
            campaign.passives.rank(NodeId::Battery),
            config.capacity,
            energy.current
        );
        if let Some(directory) = &preview.directory {
            let size = if preview.minimum {
                "640x480"
            } else {
                "1120x720"
            };
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(directory.join(format!("{size}-{label}.png"))));
        }
    }
    preview.step = step;
}
