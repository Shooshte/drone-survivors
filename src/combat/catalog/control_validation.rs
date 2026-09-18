//! Opt-in presentation fixture; synthetic input/positions, not a balance playtest.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    combat::{
        CombatConfig, Encounter, Enemy,
        control::{ControlAttack, ControlEffects, ControlPhase},
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
    seen: [bool; 3],
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
        seen: [false; 3],
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "CONTROL FIXTURE: synthetic keys; auto-fire suppressed; each single source repositioned once. Production warnings, attacks, power and recovery. Not human/balance evidence."
    );
}
type Enemies<'w, 's> = Query<
    'w,
    's,
    (
        &'static Enemy,
        &'static mut Transform,
        &'static mut DroneFlight,
        Option<&'static mut ControlAttack>,
    ),
    Without<Drone>,
>;

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut config: ResMut<CombatConfig>,
    modules: Res<Modules>,
    effects: Res<ControlEffects>,
    phase: Res<GamePhase>,
    run: Res<Encounter>,
    drone: Single<&Transform, With<Drone>>,
    mut enemies: Enemies,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    hud: Single<(&ComputedNode, &UiGlobalTransform), With<crate::combat::CombatHudRoot>>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    config.target_range = 0.;
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 65.,
        "control fixture timeout at step {}",
        fixture.step
    );
    // Check every rendered gameplay frame, including newly revealed status rows.
    if *phase == GamePhase::Playing {
        for (text, node, transform) in &text_nodes {
            if text.0.starts_with("^ ") && node.size().y > 0. {
                let top = (transform.translation.y - node.size().y / 2.) / window.scale_factor();
                let hud_bottom =
                    (hud.1.translation.y + hud.0.size().y / 2.) / window.scale_factor();
                assert!(
                    top >= hud_bottom,
                    "navigation overlaps status HUD: top={top} hud={hud_bottom}"
                );
            }
        }
    }
    if elapsed < fixture.next {
        return;
    }
    let mut advance = true;
    let label = match fixture.step {
        1..=6 | 14 | 27 => {
            keys.press(KeyCode::ArrowRight);
            None
        }
        7 => Some("beam-selector"),
        8 | 17 | 29 => {
            keys.press(KeyCode::Enter);
            None
        }
        9 | 18 => {
            if let Ok((enemy, mut transform, mut flight, Some(mut attack))) = enemies.single_mut() {
                assert_eq!(
                    enemy.kind,
                    if fixture.step == 9 {
                        EnemyKind::Slower
                    } else {
                        EnemyKind::Jammer
                    }
                );
                assert_eq!(enemy.health, 40);
                transform.translation = drone.translation - Vec3::X * 240.;
                *flight = DroneFlight {
                    heading: -std::f32::consts::FRAC_PI_2,
                    ..default()
                };
                transform.rotation = flight.rotation();
                *attack = default();
                println!(
                    "CONTROL FIXTURE activated {:?}; repositioned for visibility",
                    enemy.kind
                );
            } else {
                advance = false;
            }
            None
        }
        10 => {
            advance = false;
            if let Ok((_, _, _, Some(attack))) = enemies.single() {
                let cue = match attack.phase {
                    ControlPhase::Windup => Some((0, "beam-warning")),
                    ControlPhase::Active => {
                        assert!(effects.slowed);
                        Some((1, "beam-active"))
                    }
                    ControlPhase::Recovery => {
                        assert!(!effects.slowed);
                        Some((2, "beam-recovery"))
                    }
                    _ => None,
                };
                if let Some((index, label)) = cue
                    && !fixture.seen[index]
                {
                    fixture.seen[index] = true;
                    advance = fixture.seen.iter().all(|seen| *seen);
                    Some(label)
                } else {
                    None
                }
            } else {
                None
            }
        }
        11 | 24 => {
            keys.press(KeyCode::KeyR);
            None
        }
        12 | 25 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert!(run.elapsed < 1.5);
            assert!(!effects.slowed);
            assert_eq!(modules.disabled_for, [0.; 4]);
            assert_eq!(modules.enabled, [false; 4]);
            Some(if fixture.step == 12 {
                "beam-restart"
            } else {
                "jam-restart"
            })
        }
        13 | 26 => {
            keys.press(KeyCode::Tab);
            None
        }
        15 | 19 | 22 => {
            keys.press(KeyCode::Digit1);
            if fixture.step == 19 {
                Some("jam-warning")
            } else {
                None
            }
        }
        16 => Some("jam-selector"),
        20 => {
            if modules.disabled_for[0] > 0. {
                assert!(!modules.enabled[0]);
                assert!(!modules.active(ModuleKind::Overdrive));
                assert_eq!(modules.drain(&crate::modules::ModuleConfig::default()), 0.);
                Some("jam-lock")
            } else {
                advance = false;
                None
            }
        }
        21 => {
            if modules.disabled_for[0] == 0. {
                assert!(!modules.enabled[0]);
                assert!(modules.jam_grace > 0.);
                Some("jam-unlocked-off")
            } else {
                advance = false;
                None
            }
        }
        23 => {
            assert!(modules.active(ModuleKind::Overdrive));
            Some("jam-manual-enable")
        }
        28 => Some("control-mixed-selector"),
        30 => {
            if enemies.iter().count() == 3 {
                for kind in [EnemyKind::Slower, EnemyKind::Jammer, EnemyKind::Fast] {
                    assert!(enemies.iter().any(|(e, _, _, _)| e.kind == kind));
                }
                Some("control-mixed-round")
            } else {
                advance = false;
                None
            }
        }
        31 => {
            println!(
                "CONTROL FIXTURE PASS at {}x{}: beam phases, jam target/lock/OFF expiry/manual enable, resets and mixed roster",
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
            "CONTROL FIXTURE {label}: time={:.2} slow={} lock={:?}",
            run.elapsed, effects.slowed, modules.disabled_for
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
        fixture.next = elapsed
            + if matches!(fixture.step, 10 | 20 | 21) {
                0.1
            } else {
                1.
            };
    }
}
