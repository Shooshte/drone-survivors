//! Opt-in native card fixture; synthetic keyboard and preview XP, ordinary choices/effects.
use super::*;
use crate::{
    combat::CombatConfig,
    energy::EnergyConfig,
    modules::ModuleConfig,
    upgrades::{UpgradeKind, UpgradeRun},
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
    build: usize,
    stage: u8,
    choice: usize,
    directory: Option<PathBuf>,
}

const BUILDS: [[UpgradeKind; 4]; 3] = [
    [
        UpgradeKind::EfficientCoils,
        UpgradeKind::ReserveBattery,
        UpgradeKind::LongRangeRounds,
        UpgradeKind::HotOverdrive,
    ],
    [
        UpgradeKind::RapidRepair,
        UpgradeKind::WideRepulsor,
        UpgradeKind::Interceptor,
        UpgradeKind::AgileFrame,
    ],
    [
        UpgradeKind::HeavyArmor,
        UpgradeKind::HeavyRounds,
        UpgradeKind::RapidShield,
        UpgradeKind::WideAreaRockets,
    ],
];

pub(super) fn install(app: &mut App) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(path) = &directory {
        std::fs::create_dir_all(path).unwrap();
    }
    app.insert_resource(Fixture {
        start: Instant::now(),
        next: 1.,
        build: 0,
        stage: 0,
        choice: 0,
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "UPGRADE FIXTURE: synthetic keyboard and preview XP; ordinary eligibility, four-opportunity choices, modifiers and reset. Agent evidence, not a human balance playtest."
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
    encounter: Res<crate::combat::Encounter>,
    modules: Res<ModuleConfig>,
    energy: Res<EnergyConfig>,
    combat: Res<CombatConfig>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    keys.reset_all();
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 100.,
        "upgrade fixture timeout at build {} stage {}",
        fixture.build,
        fixture.stage
    );
    if elapsed < fixture.next {
        return;
    }
    fixture.next = elapsed + 0.2;
    let desired = BUILDS[fixture.build];
    let desired_modules = if fixture.build == 2 {
        [
            Some(ModuleKind::Overdrive),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Mobility),
            Some(ModuleKind::Rocket),
        ]
    } else {
        [
            Some(ModuleKind::Overdrive),
            Some(ModuleKind::Shield),
            Some(ModuleKind::Repulsor),
            Some(ModuleKind::Repair),
        ]
    };
    let mut capture = None;
    match fixture.stage {
        0 => {
            if arena.upgrades_page {
                keys.press(KeyCode::KeyU);
            } else if let Some(slot) =
                (0..4).find(|&slot| arena.slots[slot] != desired_modules[slot])
            {
                keys.press(SLOT_KEYS[slot]);
            } else {
                fixture.stage = 1;
            }
        }
        1 => {
            if !arena.upgrades_page {
                keys.press(KeyCode::KeyU);
            } else if let Some(slot) =
                (0..4).find(|&slot| arena.preview[slot] != Some(desired[slot]))
            {
                keys.press(SLOT_KEYS[slot]);
            } else {
                fixture.stage = 2;
            }
        }
        2 => {
            capture = Some(format!("build-{}-setup", fixture.build + 1));
            fixture.stage = 3;
        }
        3 => {
            keys.press(KeyCode::Enter);
            fixture.stage = 4;
            fixture.choice = 0;
        }
        4 => {
            assert_eq!(*phase, GamePhase::Choosing);
            assert_eq!(
                encounter.elapsed, 0.,
                "preview must precede scenario gameplay"
            );
            assert_eq!(run.offer, vec![desired[fixture.choice]]);
            assert_eq!(run.resolved as usize, fixture.choice);
            capture = Some(format!(
                "card-{}",
                desired[fixture.choice]
                    .name()
                    .to_lowercase()
                    .replace(' ', "-")
            ));
            fixture.stage = 5;
        }
        5 => {
            keys.press(KeyCode::Digit1);
            fixture.choice += 1;
            fixture.stage = if fixture.choice == 4 { 6 } else { 4 };
        }
        6 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(run.selected, desired);
            assert_eq!(run.resolved, 4);
            assert!(run.exhausted);
            println!(
                "UPGRADE FIXTURE build {}: capacity={:.2}, activation={:.2}, range={:.2}, interval={:.3}, overdrive={:.2}, drains={:?}, repair={:.2}/{:.2}, repulsor={:.2}/{:.2}/{:.2}",
                fixture.build + 1,
                energy.capacity,
                energy.activation,
                combat.target_range,
                combat.fire_interval,
                modules.overdrive_multiplier,
                modules.drains,
                modules.repair_rate,
                modules.repair_drain,
                modules.repulsor_radius,
                modules.repulsor_interval,
                modules.repulsor_drain
            );
            capture = Some(format!("build-{}-active", fixture.build + 1));
            fixture.stage = 7;
        }
        7 => {
            keys.press(KeyCode::KeyR);
            fixture.stage = 8;
        }
        8 => {
            assert!(run.selected.is_empty());
            assert_eq!(run.resolved, 0);
            assert_eq!(energy.capacity, EnergyConfig::default().capacity);
            assert_eq!(energy.activation, EnergyConfig::default().activation);
            assert_eq!(modules.repair_rate, ModuleConfig::default().repair_rate);
            assert_eq!(
                modules.repulsor_interval,
                ModuleConfig::default().repulsor_interval
            );
            assert_eq!(run.offer, vec![desired[0]]);
            keys.press(KeyCode::Tab);
            fixture.stage = 9;
        }
        9 => {
            assert_eq!(*phase, GamePhase::Hub);
            assert!(run.selected.is_empty());
            if fixture.build == BUILDS.len() - 1 {
                println!(
                    "UPGRADE FIXTURE PASS at {}x{}: all twelve cards, three four-card builds, configured benefits/drawbacks, normal choice budget, restart and return.",
                    window.width(),
                    window.height()
                );
                exit.write(AppExit::Success);
                fixture.next = elapsed + 100.;
            } else {
                fixture.build += 1;
                fixture.stage = 0;
            }
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
        println!("UPGRADE FIXTURE {label}: phase={phase:?}");
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
