//! Opt-in native check. Selection/spawning and rammer cycles use production systems.
//! Reposition each single enemy once and suppress auto-fire to expose its behavior.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    combat::{
        CombatConfig, Encounter, Enemy, PlayerHealth,
        variants::{RamPhase, Rammer},
    },
};
use bevy::{
    input::InputSystems,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct Fixture {
    start: Instant,
    next: f64,
    step: usize,
    seen: [bool; 4],
    directory: Option<PathBuf>,
}

pub(super) fn install(app: &mut App) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(path) = &directory {
        std::fs::create_dir_all(path).expect("capture directory");
    }
    app.insert_resource(Fixture {
        start: Instant::now(),
        next: 1.,
        step: 1,
        seen: [false; 4],
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "VARIANT FIXTURE: synthetic keyboard, auto-fire suppressed, single enemies repositioned once after production warning activation. Natural rammer cycles; no granted damage/XP. Not human/balance evidence."
    );
}

type Enemies<'w, 's> = Query<
    'w,
    's,
    (
        &'static Enemy,
        &'static mut Transform,
        &'static mut DroneFlight,
        Option<&'static Rammer>,
    ),
    Without<Drone>,
>;

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut config: ResMut<CombatConfig>,
    phase: Res<GamePhase>,
    run: Res<Encounter>,
    health: Res<PlayerHealth>,
    drone: Single<&Transform, With<Drone>>,
    mut enemies: Enemies,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    config.target_range = 0.;
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 65.,
        "variant fixture timeout at step {} hull={} enemies={}",
        fixture.step,
        health.current,
        enemies.iter().count()
    );
    if elapsed < fixture.next {
        return;
    }
    let mut advance = true;
    let label = match fixture.step {
        1..=3 => {
            keys.press(KeyCode::ArrowRight);
            None
        }
        4 => Some("fast-selector"),
        5 => {
            keys.press(KeyCode::Enter);
            None
        }
        6 | 12 => {
            if let Ok((enemy, mut transform, mut flight, _)) = enemies.single_mut() {
                let kind = if fixture.step == 6 {
                    EnemyKind::Fast
                } else {
                    EnemyKind::Rammer
                };
                assert_eq!(enemy.kind, kind);
                assert_eq!(enemy.health, if kind == EnemyKind::Fast { 20 } else { 80 });
                transform.translation = drone.translation - Vec3::X * 250.;
                *flight = DroneFlight {
                    heading: -std::f32::consts::FRAC_PI_2,
                    ..default()
                };
                transform.rotation = flight.rotation();
                println!("VARIANT FIXTURE activated {kind:?}; repositioned for visibility");
            } else {
                advance = false;
            }
            None
        }
        7 => Some("fast-flight"),
        8 | 16 => {
            keys.press(KeyCode::Tab);
            None
        }
        9 | 17 => {
            keys.press(KeyCode::ArrowRight);
            None
        }
        10 => Some("rammer-selector"),
        11 | 19 => {
            keys.press(KeyCode::Enter);
            None
        }
        13 => {
            advance = false;
            if let Ok((_, _, _, Some(ram))) = enemies.single() {
                let cue = match (ram.phase, ram.impacts) {
                    (RamPhase::Windup, 0) => Some((0, "rammer-warning")),
                    (RamPhase::Charge, 0) => Some((1, "rammer-charge")),
                    (RamPhase::Retreat, 1) => Some((2, "rammer-retreat")),
                    (RamPhase::Windup, 1) => Some((3, "rammer-second-warning")),
                    _ => None,
                };
                if let Some((index, name)) = cue
                    && !fixture.seen[index]
                {
                    fixture.seen[index] = true;
                    Some(name)
                } else {
                    None
                }
            } else if enemies.is_empty() {
                assert_eq!(health.current, 80);
                assert_eq!(run.kills, 0);
                assert!(fixture.seen.iter().all(|seen| *seen));
                println!(
                    "VARIANT FIXTURE two separate impacts; hull=80 kills=0; all cues observed"
                );
                advance = true;
                None
            } else {
                None
            }
        }
        14 => {
            keys.press(KeyCode::KeyR);
            None
        }
        15 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(health.current, 100);
            assert!(run.elapsed < 1.5);
            Some("rammer-restart")
        }
        18 => Some("mixed-selector"),
        20 => {
            if enemies.iter().count() == 3 {
                for kind in [EnemyKind::Chaser, EnemyKind::Fast, EnemyKind::Rammer] {
                    assert!(enemies.iter().any(|(e, _, _, _)| e.kind == kind));
                }
                Some("mixed-round")
            } else {
                advance = false;
                None
            }
        }
        21 => {
            println!(
                "VARIANT FIXTURE PASS at {}x{}: new selectors, typed warning activation, natural two-impact cycle, restart and mixed roster",
                window.width(),
                window.height()
            );
            exit.write(AppExit::Success);
            None
        }
        _ => None,
    };
    if let Some(label) = label {
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
                "text outside viewport: {:?}",
                text.0
            );
        }
        println!(
            "VARIANT FIXTURE {label}: hull={} time={:.2}",
            health.current, run.elapsed
        );
        if let Some(directory) = &fixture.directory {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(directory.join(format!(
                    "{}x{}-{label}.png",
                    window.width(),
                    window.height()
                ))));
        }
    }
    if advance {
        fixture.step += 1;
        fixture.next = elapsed + if fixture.step == 13 { 0.1 } else { 1. };
    }
}
