//! Opt-in agent UI evidence; synthetic positioning, damage and XP, not a playtest.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    combat::{CombatConfig, Enemy, PlayerHealth},
    upgrades::UpgradeRun,
    world::environment::{Environment, REPAIR_CENTER},
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
    paused: f64,
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
        paused: 0.,
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "ENVIRONMENT FIXTURE: synthetic keyboard, positions, damage and XP=50; auto-fire suppressed. Production fields, cycle, repair, pause/reset and spawns. Not human/balance evidence."
    );
}
#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut config: ResMut<CombatConfig>,
    mut health: ResMut<PlayerHealth>,
    environment: Res<Environment>,
    phase: Res<GamePhase>,
    mut upgrades: ResMut<UpgradeRun>,
    mut drone: Single<(&mut Transform, &mut DroneFlight), With<Drone>>,
    enemies: Query<&Enemy>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    config.target_range = 0.;
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 60.,
        "environment fixture timed out at {}",
        fixture.step
    );
    if elapsed < fixture.next {
        return;
    }
    let mut advance = true;
    let mut position = None;
    let label = match fixture.step {
        1..=12 | 31 | 38 | 39 => {
            keys.press(KeyCode::ArrowRight);
            None
        }
        13 => Some("environment-selector"),
        14 | 33 | 40 => {
            keys.press(KeyCode::Enter);
            None
        }
        15 => {
            assert!(environment.enabled());
            position = Some(REPAIR_CENTER);
            None
        }
        16 => {
            assert_eq!(health.current, 100);
            assert!(environment.repair_ready);
            Some("environment-ready")
        }
        17 => {
            health.current = 40;
            None
        }
        18 => {
            assert_eq!(health.current, 75);
            assert!(!environment.repair_ready);
            Some("repair-used")
        }
        19 => {
            health.current = 40;
            None
        }
        20 => {
            assert_eq!(health.current, 40);
            None
        }
        21 | 28 => {
            keys.press(KeyCode::KeyR);
            None
        }
        22 => {
            assert!(environment.repair_ready);
            assert!(environment.elapsed < 0.5);
            assert_eq!(health.current, 100);
            position = Some(Vec3::new(-420., 90., -190.));
            None
        }
        23 => Some("permanent-field"),
        24 => {
            if environment.cycle().0 {
                advance = false;
            } else {
                position = Some(Vec3::new(-420., 90., 190.));
            }
            None
        }
        25 => Some("cycling-field-off"),
        26 => {
            upgrades.award(50);
            None
        }
        27 => {
            assert_eq!(*phase, GamePhase::Choosing);
            fixture.paused = environment.elapsed;
            Some("environment-choice")
        }
        29 => {
            assert!(environment.repair_ready);
            assert!(environment.elapsed < 0.5);
            Some("environment-reset")
        }
        30 | 37 => {
            keys.press(KeyCode::Tab);
            None
        }
        32 => Some("pressure-selector"),
        34 => {
            position = Some(REPAIR_CENTER);
            None
        }
        35 => {
            if enemies.iter().count() < 2 {
                advance = false;
            }
            None
        }
        36 => Some("environment-pressure"),
        41 => {
            assert!(!environment.enabled());
            Some("ordinary-scenario")
        }
        42 => {
            println!(
                "ENVIRONMENT FIXTURE PASS at {}x{}: selector, full-hull preserves charge, 35 repair once, permanent/cycling cues, upgrade pause, restart/return, pressure roster and ordinary-scenario isolation",
                window.width(),
                window.height()
            );
            exit.write(AppExit::Success);
            None
        }
        _ => None,
    };
    if fixture.step == 28 {
        assert_eq!(environment.elapsed, fixture.paused);
    }
    if let Some(position) = position {
        drone.0.translation = position;
        *drone.1 = default();
    }
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
            "ENVIRONMENT FIXTURE {label}: clock={:.2}, hull={}, repair_ready={}, repaired={}, cycle={:?}",
            environment.elapsed,
            health.current,
            environment.repair_ready,
            environment.repaired,
            environment.cycle()
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
        fixture.next = elapsed + if fixture.step == 28 { 1. } else { 0.3 };
    }
}
