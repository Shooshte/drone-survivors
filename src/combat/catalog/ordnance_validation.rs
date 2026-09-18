//! Explicit synthetic positioning/input fixture, never a balance playtest.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    combat::{
        CombatConfig, Encounter, Enemy, PlayerHealth, Projectile, ShotPayload, SpawnWarning,
        bombs::BombState,
        mothership::{Mothership, SpawnParent},
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
    directory: Option<PathBuf>,
}
pub(super) fn install(app: &mut App) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(path) = &directory {
        std::fs::create_dir_all(path).unwrap();
    }
    app.insert_resource(Fixture {
        start: Instant::now(),
        next: 1.,
        step: 1,
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "ORDNANCE FIXTURE: synthetic keys, carrier/parent repositioning and lethal parent projectile; auto-fire suppressed. Production contact, fuse, power, spawn warnings and death. Not balance/human evidence."
    );
}
type Enemies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Enemy,
        &'static mut Transform,
        &'static mut DroneFlight,
        Option<&'static Mothership>,
    ),
    Without<Drone>,
>;
#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut config: ResMut<CombatConfig>,
    bomb: Res<BombState>,
    health: Res<PlayerHealth>,
    modules: Res<Modules>,
    run: Res<Encounter>,
    phase: Res<GamePhase>,
    drone: Single<&Transform, With<Drone>>,
    mut enemies: Enemies,
    warnings: Query<(&SpawnWarning, &SpawnParent)>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    hud: Single<(&ComputedNode, &UiGlobalTransform), With<crate::combat::CombatHudRoot>>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    config.target_range = 0.;
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 85.,
        "ordnance fixture timeout step {}",
        fixture.step
    );
    if *phase == GamePhase::Playing {
        for (text, node, transform) in &text_nodes {
            if text.0.starts_with("^ ") && node.size().y > 0. {
                let top = (transform.translation.y - node.size().y / 2.) / window.scale_factor();
                let bottom = (hud.1.translation.y + hud.0.size().y / 2.) / window.scale_factor();
                assert!(
                    top >= bottom,
                    "navigation overlaps ordnance HUD: {top} < {bottom}"
                );
            }
        }
    }
    if elapsed < fixture.next {
        return;
    }
    let mut advance = true;
    let label = match fixture.step {
        1..=9 | 27 | 36 => {
            keys.press(KeyCode::ArrowRight);
            None
        }
        10..=14 => {
            keys.press(KeyCode::Digit1);
            None
        }
        15 => Some("bomb-selector"),
        16 | 29 | 38 => {
            keys.press(KeyCode::Enter);
            None
        }
        17 | 23 => {
            if let Some((_, _, mut transform, mut flight, _)) = enemies
                .iter_mut()
                .find(|(_, e, _, _, _)| e.kind == EnemyKind::Bomber)
            {
                transform.translation = drone.translation;
                *flight = default();
                println!("ORDNANCE FIXTURE moved spawned carrier to contact");
            } else {
                advance = false;
            }
            None
        }
        18 | 24 => {
            assert!(bomb.remaining.is_some());
            assert_eq!(health.current, 100);
            Some("bomb-countdown")
        }
        19 => {
            keys.press(KeyCode::Digit1);
            None
        }
        20 => {
            assert!(bomb.remaining.is_none());
            assert_eq!(bomb.notice, "BOMB DISLODGED");
            assert_eq!(health.current, 100);
            assert!(modules.active(ModuleKind::Repulsor));
            Some("bomb-dislodged")
        }
        21 => {
            keys.press(KeyCode::KeyR);
            None
        }
        22 => {
            assert!(bomb.remaining.is_none());
            assert_eq!(modules.enabled, [false; 4]);
            assert_eq!(health.current, 100);
            Some("bomb-restart")
        }
        25 => {
            if bomb.remaining.is_none() {
                assert_eq!(health.current, 75);
                assert_eq!(bomb.notice, "BOMB DETONATED: 25 HULL");
                Some("bomb-detonated")
            } else {
                advance = false;
                None
            }
        }
        26 | 35 => {
            keys.press(KeyCode::Tab);
            None
        }
        28 => Some("mothership-selector"),
        30 => {
            if let Some((_, _, mut transform, mut flight, _)) = enemies
                .iter_mut()
                .find(|(_, e, _, _, _)| e.kind == EnemyKind::Mothership)
            {
                transform.translation = drone.translation + Vec3::new(-350., 0., 0.);
                *flight = default();
                println!("ORDNANCE FIXTURE repositioned spawned mothership for visibility");
            } else {
                advance = false;
            }
            None
        }
        31 | 33 => {
            if warnings.iter().any(|(w, _)| w.ready_at - run.elapsed > 0.4) {
                if fixture.step == 33 {
                    let (_, _, transform, _, _) = enemies
                        .iter()
                        .find(|(_, e, _, _, _)| e.kind == EnemyKind::Mothership)
                        .unwrap();
                    commands.spawn((
                        Projectile {
                            velocity: Vec3::ZERO,
                            remaining: 1.,
                        },
                        ShotPayload {
                            damage: 100,
                            radius: 3.,
                        },
                        Transform::from_translation(transform.translation),
                    ));
                    println!(
                        "ORDNANCE FIXTURE fired lethal synthetic projectile at parent with pending launch"
                    );
                    None
                } else {
                    Some("mothership-warning")
                }
            } else {
                advance = false;
                None
            }
        }
        32 => {
            if enemies
                .iter()
                .any(|(_, e, _, _, _)| e.kind == EnemyKind::Chaser)
            {
                Some("mothership-child")
            } else {
                advance = false;
                None
            }
        }
        34 => {
            assert!(
                !enemies
                    .iter()
                    .any(|(_, e, _, _, _)| e.kind == EnemyKind::Mothership)
            );
            assert!(warnings.is_empty());
            assert_eq!(run.kills, 1);
            Some("mothership-destroyed")
        }
        37 => Some("ordnance-mixed-selector"),
        39 => {
            if enemies.iter().count() == 3 {
                for kind in [EnemyKind::Bomber, EnemyKind::Mothership, EnemyKind::Jammer] {
                    assert!(enemies.iter().any(|(_, e, _, _, _)| e.kind == kind));
                }
                Some("ordnance-mixed-round")
            } else {
                advance = false;
                None
            }
        }
        40 => {
            for (_, enemy, mut transform, mut flight, _) in &mut enemies {
                transform.translation = drone.translation
                    + match enemy.kind {
                        EnemyKind::Bomber => Vec3::ZERO,
                        EnemyKind::Jammer => Vec3::X * 240.,
                        EnemyKind::Mothership => Vec3::NEG_X * 350.,
                        _ => continue,
                    };
                *flight = default();
            }
            println!(
                "ORDNANCE FIXTURE repositioned mixed enemies to expose simultaneous HUD states"
            );
            None
        }
        41 => {
            assert!(bomb.remaining.is_some());
            Some("ordnance-mixed-threats")
        }
        42 => {
            if modules.disabled_for[0] > 0. {
                assert!(bomb.remaining.is_some());
                assert!(!modules.active(ModuleKind::Repulsor));
                Some("ordnance-mixed-lock")
            } else {
                advance = false;
                None
            }
        }
        43 => {
            println!(
                "ORDNANCE FIXTURE PASS at {}x{}: bomb countdown, powered dislodge, 25 damage, restart, warned child, parent death cancellation, mixed roster and simultaneous bomb/jam HUD",
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
            "ORDNANCE FIXTURE {label}: time={:.2}, hull={}, fuse={:?}, kills={}, spawn={:?}",
            run.elapsed, health.current, bomb.remaining, run.kills, run.spawns
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
        fixture.next = elapsed + 0.4;
    }
}
