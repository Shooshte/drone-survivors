//! Opt-in native support-module fixture. Synthetic damage, enemies, bomb and XP.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    combat::{CombatConfig, Enemy, PlayerHealth, bombs::BombState},
    energy::Energy,
    upgrades::UpgradeRun,
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
    health: u32,
    power: f64,
    cooldown: f64,
    target: Option<Entity>,
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
        health: 0,
        power: 0.,
        cooldown: 0.,
        target: None,
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "MODULE FIXTURE: synthetic keyboard, hull damage, target spawn, bomb attachment and XP=50; auto-fire suppressed. Real power, repair, pulse, jam, pause and reset. Agent evidence, not a human/balance playtest."
    );
}
#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut config: ResMut<CombatConfig>,
    mut health: ResMut<PlayerHealth>,
    mut modules: ResMut<Modules>,
    mut energy: ResMut<Energy>,
    mut bomb: ResMut<BombState>,
    phase: Res<GamePhase>,
    mut upgrades: ResMut<UpgradeRun>,
    drone: Single<&Transform, With<Drone>>,
    enemies: Query<(&Enemy, &Transform), Without<Drone>>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    config.target_range = 0.;
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(elapsed < 50., "module fixture timeout at {}", fixture.step);
    if elapsed < fixture.next {
        return;
    }
    let mut advance = true;
    let label = match fixture.step {
        1..=6 => {
            keys.press(KeyCode::Digit1);
            None
        }
        7..=11 => {
            keys.press(KeyCode::Digit2);
            None
        }
        12 => Some("support-selector"),
        13 | 23 => {
            keys.press(KeyCode::Enter);
            None
        }
        14 => {
            assert_eq!(modules.loadout.slots()[0], Some(ModuleKind::Repair));
            assert_eq!(modules.loadout.slots()[1], Some(ModuleKind::Repulsor));
            health.current = 40;
            bomb.attach();
            let position = drone.translation + Vec3::X * 100.;
            fixture.target = Some(
                commands
                    .spawn((
                        Enemy {
                            kind: EnemyKind::Chaser,
                            health: 20,
                            previous: position,
                            path: vec![],
                        },
                        Transform::from_translation(position),
                        DroneFlight::default(),
                    ))
                    .id(),
            );
            keys.press(KeyCode::Digit1);
            keys.press(KeyCode::Digit2);
            None
        }
        15 => {
            assert!(bomb.remaining.is_none());
            assert_eq!(bomb.notice, "BOMB DISLODGED");
            assert!(bomb.pulse_flash > 0.);
            Some("support-pulse")
        }
        16 => {
            assert!(
                health.current >= 42 && health.current < 50,
                "repair hull={}",
                health.current
            );
            assert!(energy.current < 95.);
            let (enemy, transform) = enemies.get(fixture.target.unwrap()).unwrap();
            assert_eq!(enemy.health, 20);
            assert!(
                transform.translation.distance(drone.translation) > 100.,
                "pulse did not push target away"
            );
            println!(
                "MODULE FIXTURE push distance={:.1}, target hull={}",
                transform.translation.distance(drone.translation),
                enemy.health
            );
            commands.entity(fixture.target.take().unwrap()).despawn();
            Some("support-repair")
        }
        17 => {
            assert!(modules.jam(0));
            fixture.health = health.current;
            None
        }
        18 => {
            assert_eq!(health.current, fixture.health);
            assert!(!modules.active(ModuleKind::Repair));
            assert!(modules.disabled_for[0] > 0.);
            Some("support-locked")
        }
        19 => {
            upgrades.award(50);
            None
        }
        20 => {
            assert_eq!(*phase, GamePhase::Choosing);
            fixture.health = health.current;
            fixture.power = energy.current;
            fixture.cooldown = bomb.pulse_cooldown;
            Some("support-choice")
        }
        21 => {
            assert_eq!(health.current, fixture.health);
            assert_eq!(energy.current, fixture.power);
            assert_eq!(bomb.pulse_cooldown, fixture.cooldown);
            keys.press(KeyCode::KeyR);
            None
        }
        22 => {
            assert_eq!(health.current, 100);
            assert_eq!(energy.current, 100.);
            assert_eq!(modules.enabled, [false; 4]);
            assert_eq!(bomb.pulse_cooldown, 0.);
            keys.press(KeyCode::Tab);
            None
        }
        24 => {
            health.current = 99;
            keys.press(KeyCode::Digit1);
            None
        }
        25 => {
            assert_eq!(health.current, 100);
            assert!(energy.current < 100.);
            Some("support-full")
        }
        26 => {
            health.current = 50;
            energy.current = 0.;
            None
        }
        27 => {
            assert_eq!(health.current, 50);
            assert_eq!(modules.enabled, [false; 4]);
            Some("support-brownout")
        }
        28 => {
            keys.press(KeyCode::KeyR);
            None
        }
        29 => Some("support-reset"),
        30 => {
            keys.press(KeyCode::Tab);
            None
        }
        31 => {
            assert_eq!(*phase, GamePhase::Hub);
            assert_eq!(bomb.pulse_cooldown, 0.);
            Some("support-return")
        }
        32 => {
            keys.press(KeyCode::Digit3);
            None
        }
        33 => {
            keys.press(KeyCode::Digit4);
            None
        }
        34..=46 => {
            keys.press(KeyCode::ArrowRight);
            Some("support-scenario-sweep")
        }
        47 => Some("support-pressure-selector"),
        48 => {
            println!(
                "MODULE FIXTURE PASS at {}x{}: six-module selection, configured limits, paid repair/cap, non-damaging push, bomb dislodge, jammer lock, upgrade pause, brownout and restart/return.",
                window.width(),
                window.height()
            );
            exit.write(AppExit::Success);
            None
        }
        _ => {
            advance = false;
            None
        }
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
            "MODULE FIXTURE {label}: hull={}, battery={:.2}, cooldown={:.2}, phase={phase:?}",
            health.current, energy.current, bomb.pulse_cooldown
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
            + match fixture.step {
                15 => 0.,
                16 => 0.5,
                18 | 21 => 1.,
                _ => 0.3,
            };
    }
}
