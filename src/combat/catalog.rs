//! Interactive content experiments, installed only by --validate catalog.
use super::{WaveConfig, variants::SpawnRoster};
use crate::economy::runtime::EnemyKind;
use crate::{
    game::{GamePhase, GameplaySet, MissionBoundary},
    modules::{Loadout, ModuleKind, Modules, SLOT_KEYS},
};
use bevy::prelude::*;

mod scene;
mod validation;

#[derive(Resource)]
struct CatalogArena {
    scenario: usize,
    slots: [Option<ModuleKind>; 4],
    selecting: bool,
    duration: f64,
}

struct Scenario {
    name: &'static str,
    instruction: &'static str,
    bursts: &'static [(f64, usize)],
    kinds: &'static [EnemyKind],
}

const SCENARIOS: [Scenario; 6] = [
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
];

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CatalogAction {
    Previous,
    Next,
    CycleSlot(usize),
    Launch,
    Return,
    Restart,
}

pub(super) fn install(app: &mut App, duration: f64) {
    app.insert_resource(CatalogArena {
        scenario: 0,
        slots: [None; 4],
        selecting: true,
        duration,
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
    if std::env::var_os("DRONE_CATALOG_SMOKE").is_some() {
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

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Query<(&CatalogAction, &Interaction), Changed<Interaction>>,
    mut arena: ResMut<CatalogArena>,
    mut boundary: ResMut<MissionBoundary>,
    mut modules: ResMut<Modules>,
    mut waves: ResMut<WaveConfig>,
    mut roster: ResMut<SpawnRoster>,
) {
    *boundary = MissionBoundary::default();
    let keyboard = if arena.selecting {
        if keys.just_pressed(KeyCode::Enter) {
            Some(CatalogAction::Launch)
        } else if keys.just_pressed(KeyCode::ArrowLeft) {
            Some(CatalogAction::Previous)
        } else if keys.just_pressed(KeyCode::ArrowRight) {
            Some(CatalogAction::Next)
        } else {
            SLOT_KEYS
                .iter()
                .position(|key| keys.just_pressed(*key))
                .map(CatalogAction::CycleSlot)
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
        Some(CatalogAction::CycleSlot(slot)) if arena.selecting => cycle_slot(&mut arena, slot),
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
