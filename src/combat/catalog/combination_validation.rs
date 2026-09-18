//! Opt-in native combined-catalog check. Synthetic keyboard and normal preview XP only.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    combat::{Encounter, Enemy, PlayerHealth},
    upgrades::{UpgradeKind, UpgradeRun},
    world::environment::Environment,
};
use bevy::{
    input::InputSystems,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

const CARDS: [UpgradeKind; 4] = [
    UpgradeKind::EfficientCoils,
    UpgradeKind::HeavyRounds,
    UpgradeKind::RapidRepair,
    UpgradeKind::WideRepulsor,
];
const SLOTS: [Option<ModuleKind>; 4] = [
    Some(ModuleKind::Overdrive),
    Some(ModuleKind::Shield),
    Some(ModuleKind::Repair),
    Some(ModuleKind::Repulsor),
];

#[derive(Resource)]
struct Fixture {
    start: Instant,
    next: f64,
    stage: u8,
    choice: usize,
    seen: Vec<EnemyKind>,
    captures: usize,
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
        stage: 0,
        choice: 0,
        seen: vec![],
        captures: 0,
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "COMBINATION FIXTURE: synthetic keyboard and normal four-card preview XP; no health, energy, damage or enemy overrides. Agent presentation check, not human acceptance."
    );
}

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    arena: Res<CatalogArena>,
    run: Res<UpgradeRun>,
    phase: Res<GamePhase>,
    encounter: Res<Encounter>,
    modules: Res<Modules>,
    health: Res<PlayerHealth>,
    environment: Res<Environment>,
    drone: Single<(&Transform, &DroneFlight), With<Drone>>,
    enemies: Query<&Enemy>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 100.,
        "combination fixture timeout at {}",
        fixture.stage
    );
    // Exercise ordinary flight and costs between snapshots; lower-left loop visits both fields.
    if fixture.stage == 6 && *phase == GamePhase::Playing {
        let t = encounter.elapsed;
        let target = match (t / 5.) as usize % 4 {
            0 => Vec3::new(-620., 90., -190.),
            1 => Vec3::new(-250., 90., -190.),
            2 => Vec3::new(-250., 90., 190.),
            _ => Vec3::new(-620., 90., 190.),
        };
        for key in crate::combat::validation::routes::keys(drone.0.translation, drone.1, target) {
            keys.press(key);
        }
        for (slot, key) in SLOT_KEYS.into_iter().enumerate() {
            let desired = slot != 2 || health.current < 85;
            if modules.enabled[slot] != desired && modules.disabled_for[slot] <= 0. {
                keys.press(key);
            }
        }
        for enemy in &enemies {
            if !fixture.seen.contains(&enemy.kind) {
                fixture.seen.push(enemy.kind);
            }
        }
    }
    if elapsed < fixture.next {
        return;
    }
    fixture.next = elapsed + 0.12;
    let mut capture = None;
    match fixture.stage {
        0 => {
            if arena.scenario != SCENARIOS.len() - 1 {
                keys.press(KeyCode::ArrowLeft);
            } else if let Some(slot) = (0..4).find(|&i| arena.slots[i] != SLOTS[i]) {
                keys.press(SLOT_KEYS[slot]);
            } else {
                capture = Some("combined-setup");
                fixture.stage = 1;
            }
        }
        1 => {
            if !arena.upgrades_page {
                keys.press(KeyCode::KeyU);
            } else if let Some(slot) = (0..4).find(|&i| arena.preview[i] != Some(CARDS[i])) {
                keys.press(SLOT_KEYS[slot]);
            } else {
                capture = Some("combined-preview");
                fixture.stage = 2;
            }
        }
        2 => {
            keys.press(KeyCode::Enter);
            fixture.stage = 3;
        }
        3 => {
            assert_eq!(*phase, GamePhase::Choosing);
            assert_eq!(encounter.elapsed, 0.);
            assert_eq!(run.offer, vec![CARDS[fixture.choice]]);
            capture = Some("combined-choice");
            fixture.stage = 4;
        }
        4 => {
            keys.press(KeyCode::Digit1);
            fixture.choice += 1;
            fixture.stage = if fixture.choice == 4 { 5 } else { 3 };
        }
        5 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(run.selected, CARDS);
            assert_eq!(run.resolved, 4);
            assert!(environment.enabled());
            fixture.stage = 6;
        }
        6 => {
            if encounter.elapsed >= [3., 10., 20.][fixture.captures] || *phase == GamePhase::Dead {
                capture = Some(
                    ["combined-wave", "combined-fields", "combined-pressure"][fixture.captures],
                );
                fixture.captures += 1;
                if fixture.captures == 3 || *phase == GamePhase::Dead {
                    fixture.stage = 7;
                }
            }
        }
        7 => {
            assert_eq!(
                fixture.seen.len(),
                7,
                "all enemy silhouettes must be observed"
            );
            keys.press(KeyCode::KeyR);
            fixture.stage = 8;
        }
        8 => {
            assert_eq!(encounter.elapsed, 0.);
            assert_eq!(health.current, 100);
            assert!(environment.repair_ready);
            assert_eq!(modules.enabled, [false; 4]);
            assert_eq!(run.resolved, 0);
            assert_eq!(run.offer, vec![CARDS[0]]);
            keys.press(KeyCode::Tab);
            fixture.stage = 9;
        }
        9 => {
            assert_eq!(*phase, GamePhase::Hub);
            assert!(!environment.enabled());
            assert!(run.selected.is_empty());
            capture = Some("combined-return");
            fixture.stage = 10;
        }
        10 => {
            println!(
                "COMBINATION FIXTURE PASS at {}x{}: seven kinds, four-card build, real mixed combat/fields, reset/return and text bounds. Seen: {:?}",
                window.width(),
                window.height(),
                fixture.seen
            );
            exit.write(AppExit::Success);
            fixture.next = elapsed + 100.;
        }
        _ => unreachable!(),
    }
    if let Some(label) = capture {
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
            "COMBINATION FIXTURE {label}: phase={phase:?} t={:.2} hull={} kills={} seen={:?}",
            encounter.elapsed, health.current, encounter.kills, fixture.seen
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
        fixture.next = elapsed + 0.5;
    }
}
