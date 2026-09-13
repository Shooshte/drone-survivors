//! Opt-in native menu fixture. Its outcome/XP overrides are never installed in normal play.
use super::{Campaign, MissionPlugin, MissionSession, scene::MissionScenePlugin};
use crate::{
    combat::{Encounter, PlayerHealth},
    game::GamePhase,
    upgrades::UpgradeRun,
};
use bevy::{
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct MissionPreview {
    start: Instant,
    step: usize,
    seconds: f64,
    captures: Option<PathBuf>,
    minimum: bool,
}

pub(crate) fn install(app: &mut App, seconds: f64) {
    let captures = std::env::var_os("DRONE_CAPTURE_DIR")
        .map(PathBuf::from)
        .filter(|path| {
            if let Err(error) = std::fs::create_dir_all(path) {
                eprintln!("Mission capture directory unavailable: {error}");
                false
            } else {
                true
            }
        });
    println!(
        "MISSION UI FIXTURE: synthetic XP=50, resources=19/3 and 7/3, success timer=300, failure hull/phase override; not gameplay-success evidence"
    );
    app.add_plugins((
        MissionPlugin,
        MissionScenePlugin,
        crate::economy::scene::EconomyScenePlugin,
    ))
    .insert_resource(MissionPreview {
        start: Instant::now(),
        step: 0,
        seconds,
        captures,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
    })
    .add_systems(Startup, resize)
    .add_systems(PreUpdate, drive.after(InputSystems));
}

fn resize(mut preview: ResMut<MissionPreview>, mut window: Single<&mut Window>) {
    preview.start = Instant::now();
    if preview.minimum {
        window.resolution.set(640., 480.);
    }
}

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut preview: ResMut<MissionPreview>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut phase: ResMut<GamePhase>,
    mut encounter: ResMut<Encounter>,
    mut health: ResMut<PlayerHealth>,
    mut upgrades: ResMut<UpgradeRun>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    mut resources: ResMut<crate::economy::AttemptResources>,
    mut exit: MessageWriter<AppExit>,
    primary_labels: Query<(&Text, &ComputedNode), With<super::scene::PrimaryLabel>>,
) {
    // The fixture owns only its navigation keys; release between every action.
    for key in [KeyCode::Enter, KeyCode::Backspace, KeyCode::KeyR] {
        keys.reset(key);
    }
    let elapsed = preview.start.elapsed().as_secs_f64();
    if elapsed >= preview.seconds {
        println!(
            "MISSION UI FIXTURE stopped at limit, completed_steps={}",
            preview.step
        );
        exit.write(AppExit::Success);
        return;
    }
    if preview.step >= 32 || elapsed < (preview.step + 1) as f64 {
        return;
    }
    let step = preview.step + 1;
    let expected = match step {
        1 | 6 | 17 | 18 | 25 | 26 => Some(GamePhase::Hub),
        3 | 4 | 8 | 20 | 28 => Some(GamePhase::Briefing),
        9 | 10 | 14 | 22 | 29 | 32 => Some(GamePhase::Playing),
        11 | 12 => Some(GamePhase::Choosing),
        15 | 16 => Some(GamePhase::Survived),
        23 | 24 => Some(GamePhase::Dead),
        _ => None,
    };
    if let Some(expected) = expected {
        assert_eq!(*phase, expected, "mission preview step {step}");
    }
    match step {
        2 | 6 | 8 | 16 | 18 | 20 | 24 | 26 | 28 => keys.press(KeyCode::Enter),
        4 => keys.press(KeyCode::Backspace),
        10 => upgrades.award(50),
        12 => keys.press(KeyCode::KeyR),
        13 => {
            assert_eq!(
                resources.collected,
                crate::economy::Amounts::default(),
                "restart discards unbanked resources"
            );
            resources.collected = crate::economy::Amounts {
                salvage: 19,
                components: 3,
            };
        }
        14 => encounter.elapsed = 300.,
        15 => assert_eq!(
            campaign.wallet,
            crate::economy::Amounts {
                salvage: 29,
                components: 4
            }
        ),
        21 => {
            resources.collected = crate::economy::Amounts {
                salvage: 7,
                components: 3,
            }
        }
        23 => assert_eq!(
            campaign.wallet,
            crate::economy::Amounts {
                salvage: 30,
                components: 4
            }
        ),
        22 => {
            health.current = 0;
            *phase = GamePhase::Dead;
        }
        32 => {
            assert_eq!(campaign.history.len(), 2);
            assert!(campaign.mission_succeeded);
            assert!(campaign.history[0].succeeded);
            assert!(!campaign.history[1].succeeded);
            assert!(session.result.is_none());
            assert_eq!(resources.collected, crate::economy::Amounts::default());
            assert_eq!(
                campaign.wallet,
                crate::economy::Amounts {
                    salvage: 30,
                    components: 4
                }
            );
            println!(
                "MISSION UI FIXTURE PASS: choice restart, success, failure, final relaunch; history=2, prior success retained"
            );
            exit.write(AppExit::Success);
        }
        _ => {}
    }
    let label = match step {
        1 => Some("hub"),
        3 => Some("briefing"),
        9 => Some("combat"),
        11 => Some("choice"),
        15 => Some("success"),
        17 => Some("hub-success"),
        23 => Some("failure"),
        25 => Some("hub-history"),
        29 => Some("relaunch"),
        _ => None,
    };
    if let Some(label) = label {
        if matches!(
            *phase,
            GamePhase::Hub | GamePhase::Briefing | GamePhase::Dead | GamePhase::Survived
        ) {
            let (text, node) = primary_labels
                .single()
                .expect("exactly one primary action label");
            assert!(!text.0.is_empty(), "primary action label is empty");
            assert!(
                node.size().x > 100.,
                "primary label collapsed: {:?}",
                node.size()
            );
        }
        println!(
            "MISSION UI FIXTURE {label}: phase={:?}, results={}, elapsed={:.3}, unbanked={:?}, bank={:?}",
            *phase,
            campaign.history.len(),
            encounter.elapsed,
            resources.collected,
            campaign.wallet
        );
        if let Some(directory) = &preview.captures {
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
