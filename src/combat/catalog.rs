//! Interactive content experiments, installed only by --validate catalog.
use super::{WaveConfig, variants::SpawnRoster};
use crate::economy::runtime::EnemyKind;
use crate::{
    game::{GamePhase, GameplaySet, MissionBoundary},
    modules::{Loadout, ModuleKind, Modules, SLOT_KEYS},
};
use bevy::prelude::*;

mod combination_validation;
mod control_validation;
mod environment;
mod environment_scene;
mod environment_validation;
mod module_validation;
mod ordnance_validation;
mod scene;
mod upgrade_validation;
mod validation;
mod variant_validation;

#[derive(Resource)]
struct CatalogArena {
    scenario: usize,
    slots: [Option<ModuleKind>; 4],
    selecting: bool,
    duration: f64,
    upgrades_page: bool,
    preview: [Option<crate::upgrades::UpgradeKind>; 4],
}

struct Scenario {
    name: &'static str,
    instruction: &'static str,
    bursts: &'static [(f64, usize)],
    kinds: &'static [EnemyKind],
}

const SCENARIOS: [Scenario; 15] = [
    Scenario {
        name: "Flight practice",
        instruction: "No enemies. Practice banking, altitude, chargers and the timed hazard.",
        kinds: &[EnemyKind::Chaser],
        bursts: &[],
    },
    Scenario {
        name: "Single pursuer",
        instruction: "One chaser. Watch its warning, keep moving and compare module use.",
        kinds: &[EnemyKind::Chaser],
        bursts: &[(1., 1)],
    },
    Scenario {
        name: "Small swarm",
        instruction: "Three waves of five. Avoid encirclement and conserve energy between waves.",
        kinds: &[EnemyKind::Chaser],
        bursts: &[(1., 5), (9., 5), (17., 5)],
    },
    Scenario {
        name: "Fast pursuer",
        instruction: "Cyan fins: faster pursuit. Change heading/altitude and use cover.",
        bursts: &[(1., 1)],
        kinds: &[EnemyKind::Fast],
    },
    Scenario {
        name: "Double-impact rammer",
        instruction: "Yellow ring: dodge the line. Red: charge. Blue: retreat. Two pips = two impacts.",
        bursts: &[(1., 1)],
        kinds: &[EnemyKind::Rammer],
    },
    Scenario {
        name: "Mixed pursuers",
        instruction: "Orange chasers, cyan fast pursuers, magenta rammers. Break charge lines with cover.",
        bursts: &[(1., 3), (12., 3)],
        kinds: &[EnemyKind::Chaser, EnemyKind::Fast, EnemyKind::Rammer],
    },
    Scenario {
        name: "Slowing beam",
        instruction: "Cross emitter: break sight/range during yellow warning. Active beam slows horizontal flight to 60%.",
        bursts: &[(1., 1)],
        kinds: &[EnemyKind::Slower],
    },
    Scenario {
        name: "Module jammer",
        instruction: "Antenna: announced slot locks for 3s. Break sight/range to dodge; press its key after recovery.",
        bursts: &[(1., 1)],
        kinds: &[EnemyKind::Jammer],
    },
    Scenario {
        name: "Mixed control",
        instruction: "Beam, jammer and fast pursuer. Use cover/altitude; locks do not stack. Blue rings mark recovery.",
        bursts: &[(1., 3), (12., 3)],
        kinds: &[EnemyKind::Slower, EnemyKind::Jammer, EnemyKind::Fast],
    },
    Scenario {
        name: "Collision bomb",
        instruction: "Red spiked carrier: avoid contact. One bomb, 3s fuse, 25 damage. Powered Repulsor dislodges it; shield blocks the blast.",
        bursts: &[(1., 1)],
        kinds: &[EnemyKind::Bomber],
    },
    Scenario {
        name: "Mothership",
        instruction: "White launch ring: destroy the parent to stop launches. One child per 6s, 1.2s warning, shared cap 30.",
        bursts: &[(1., 1)],
        kinds: &[EnemyKind::Mothership],
    },
    Scenario {
        name: "Mixed ordnance",
        instruction: "Carrier, mothership and jammer. Repulsor needs power and an unlocked slot. Destroy parents to stop launches.",
        bursts: &[(1., 3)],
        kinds: &[EnemyKind::Bomber, EnemyKind::Mothership, EnemyKind::Jammer],
    },
    Scenario {
        name: "Environment practice",
        instruction: "East arrows: +40% east / -40% west; north/south neutral. North field permanent; south 6s on / 4s off. Repair cross: one +35 hull charge, within 70 at height 90. Full hull saves it.",
        bursts: &[],
        kinds: &[EnemyKind::Chaser],
    },
    Scenario {
        name: "Environment pressure",
        instruction: "Same fields and one +35 hull repair charge. Fast pursuer + slowing beam: route with the east arrows; leave fields to restore normal travel. R restores repair and cycles.",
        bursts: &[(1., 2), (15., 2)],
        kinds: &[EnemyKind::Fast, EnemyKind::Slower],
    },
    Scenario {
        name: "Combined catalog",
        instruction: "All seven enemy kinds in three waves. Break beam/charge lines, stop motherships, save power for bombs. East fields aid routes; the repair cross restores hull once. Compare builds with U.",
        bursts: &[(1., 7), (16., 7), (31., 7)],
        kinds: &[
            EnemyKind::Chaser,
            EnemyKind::Fast,
            EnemyKind::Rammer,
            EnemyKind::Slower,
            EnemyKind::Jammer,
            EnemyKind::Bomber,
            EnemyKind::Mothership,
        ],
    },
];

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CatalogAction {
    Previous,
    Next,
    CycleSlot(usize),
    ToggleUpgrades,
    CycleUpgrade(usize),
    Launch,
    Return,
    Restart,
}

pub(super) fn install(app: &mut App, duration: f64) {
    environment::install(app);
    environment_scene::install(app);
    app.insert_resource(CatalogArena {
        scenario: 0,
        slots: [None; 4],
        selecting: true,
        duration,
        upgrades_page: false,
        preview: [None; 4],
    })
    .insert_resource(crate::upgrades::UpgradePool {
        catalog: true,
        preview: Vec::new(),
    })
    .init_resource::<MissionBoundary>()
    .init_resource::<SpawnRoster>()
    .insert_resource(GamePhase::Hub)
    .add_systems(Update, controls.in_set(GameplaySet::Transition))
    .add_systems(
        Update,
        pause_selector
            .after(GameplaySet::Reset)
            .before(GameplaySet::ChoiceInput),
    )
    .add_systems(Startup, scene::setup)
    .add_systems(Update, scene::present.in_set(GameplaySet::Presentation));
    app.world_mut()
        .resource_mut::<WaveConfig>()
        .disable_authored_waves();
    if std::env::var_os("DRONE_COMBINATION_SMOKE").is_some() {
        combination_validation::install(app);
    } else if std::env::var_os("DRONE_UPGRADE_SMOKE").is_some() {
        upgrade_validation::install(app);
    } else if std::env::var_os("DRONE_MODULE_SMOKE").is_some() {
        module_validation::install(app);
    } else if std::env::var_os("DRONE_ENVIRONMENT_SMOKE").is_some() {
        environment_validation::install(app);
    } else if std::env::var_os("DRONE_ORDNANCE_SMOKE").is_some() {
        ordnance_validation::install(app);
    } else if std::env::var_os("DRONE_CONTROL_SMOKE").is_some() {
        control_validation::install(app);
    } else if std::env::var_os("DRONE_VARIANT_SMOKE").is_some() {
        variant_validation::install(app);
    } else if std::env::var_os("DRONE_CATALOG_SMOKE").is_some() {
        validation::install(app);
    }
}

fn cycle_slot(arena: &mut CatalogArena, slot: usize) {
    if slot >= arena.slots.len() {
        return;
    }
    let choices = [
        None,
        Some(ModuleKind::Overdrive),
        Some(ModuleKind::Shield),
        Some(ModuleKind::Mobility),
        Some(ModuleKind::Rocket),
        Some(ModuleKind::Repulsor),
        Some(ModuleKind::Repair),
    ];
    let current = choices
        .iter()
        .position(|kind| *kind == arena.slots[slot])
        .unwrap_or(0);
    for offset in 1..=choices.len() {
        let candidate = choices[(current + offset) % choices.len()];
        if candidate.is_none() || !arena.slots.contains(&candidate) {
            arena.slots[slot] = candidate;
            break;
        }
    }
}

fn cycle_upgrade(arena: &mut CatalogArena, slot: usize) {
    use crate::upgrades::UpgradeKind;
    if slot >= 4 {
        return;
    }
    let loadout = Loadout::new(arena.slots).expect("unique catalog modules");
    let choices: Vec<_> = std::iter::once(None)
        .chain(
            UpgradeKind::CATALOG
                .into_iter()
                .filter(|kind| kind.eligible(&loadout))
                .map(Some),
        )
        .collect();
    let current = arena.preview[slot];
    let index = choices
        .iter()
        .position(|kind| *kind == current)
        .unwrap_or(0);
    for offset in 1..=choices.len() {
        let candidate = choices[(index + offset) % choices.len()];
        if candidate.is_none() || !arena.preview.contains(&candidate) {
            arena.preview[slot] = candidate;
            break;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Query<(&CatalogAction, &Interaction), Changed<Interaction>>,
    mut arena: ResMut<CatalogArena>,
    mut boundary: ResMut<MissionBoundary>,
    mut modules: ResMut<Modules>,
    mut waves: ResMut<WaveConfig>,
    mut roster: ResMut<SpawnRoster>,
    mut pool: ResMut<crate::upgrades::UpgradePool>,
) {
    *boundary = MissionBoundary::default();
    let keyboard = if arena.selecting {
        if keys.just_pressed(KeyCode::KeyU) {
            Some(CatalogAction::ToggleUpgrades)
        } else if keys.just_pressed(KeyCode::Enter) {
            Some(CatalogAction::Launch)
        } else if !arena.upgrades_page && keys.just_pressed(KeyCode::ArrowLeft) {
            Some(CatalogAction::Previous)
        } else if !arena.upgrades_page && keys.just_pressed(KeyCode::ArrowRight) {
            Some(CatalogAction::Next)
        } else {
            SLOT_KEYS
                .iter()
                .position(|key| keys.just_pressed(*key))
                .map(|slot| {
                    if arena.upgrades_page {
                        CatalogAction::CycleUpgrade(slot)
                    } else {
                        CatalogAction::CycleSlot(slot)
                    }
                })
        }
    } else if keys.just_pressed(KeyCode::Tab) {
        Some(CatalogAction::Return)
    } else {
        None
    };
    let clicked = buttons.iter().find_map(|(action, interaction)| {
        (*interaction == Interaction::Pressed).then_some(*action)
    });
    match keyboard.or(clicked) {
        Some(CatalogAction::Previous) if arena.selecting => {
            arena.scenario = (arena.scenario + SCENARIOS.len() - 1) % SCENARIOS.len()
        }
        Some(CatalogAction::Next) if arena.selecting => {
            arena.scenario = (arena.scenario + 1) % SCENARIOS.len()
        }
        Some(CatalogAction::ToggleUpgrades) if arena.selecting => {
            arena.upgrades_page = !arena.upgrades_page
        }
        Some(CatalogAction::CycleUpgrade(slot)) if arena.selecting && arena.upgrades_page => {
            cycle_upgrade(&mut arena, slot);
        }
        Some(CatalogAction::CycleSlot(slot)) if arena.selecting && !arena.upgrades_page => {
            cycle_slot(&mut arena, slot);
            let loadout = Loadout::new(arena.slots).expect("unique catalog modules");
            for card in &mut arena.preview {
                if card.is_some_and(|kind| !kind.eligible(&loadout)) {
                    *card = None;
                }
            }
        }
        Some(CatalogAction::Launch) if arena.selecting => {
            arena.selecting = false;
            boundary.reset = true;
        }
        Some(CatalogAction::Return) if !arena.selecting => {
            arena.selecting = true;
            boundary.reset = true;
        }
        Some(CatalogAction::Restart) if !arena.selecting => {
            boundary.reset = true;
        }
        _ => {}
    }
    if !arena.selecting && keys.just_pressed(KeyCode::KeyR) {
        boundary.reset = true;
    }
    if boundary.reset {
        pool.preview = arena.preview.iter().flatten().copied().collect();
        modules.loadout = Loadout::new(arena.slots).expect("arena keeps module types unique");
        waves.disable_authored_waves();
        waves.duration = arena.duration;
        waves.bursts = SCENARIOS[arena.scenario].bursts.to_vec();
        roster.0 = SCENARIOS[arena.scenario].kinds.to_vec();
    }
}

fn pause_selector(
    arena: Res<CatalogArena>,
    mut phase: ResMut<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
) {
    if arena.selecting {
        *phase = GamePhase::Hub;
        clock.pause();
    }
}
