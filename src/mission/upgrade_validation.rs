//! Opt-in combined Mission 01 presentation check; never reads or writes saves.
use super::{Campaign, MissionAction, MissionSession, objectives::ObjectiveRun};
use crate::{
    arena::{Drone, DroneFlight, FlightConfig},
    combat::CombatConfig,
    energy::EnergyConfig,
    game::{GamePhase, GameplaySet},
    modules::{ModuleConfig, ModuleKind, Modules},
    upgrades::{ChoiceAction, UpgradeKind, UpgradeRun},
    world::mission01 as map,
};
use bevy::{
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    ui::UiSystems,
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct Check {
    started: Instant,
    step: usize,
    next_step_at: f64,
    directory: PathBuf,
    minimum: bool,
    pending: Option<&'static str>,
    baseline: Vec<f64>,
    captured_choices: usize,
}

pub(crate) fn install(app: &mut App) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/drone-mission01-upgrades-check"));
    std::fs::create_dir_all(&directory).expect("capture directory must be writable");
    println!(
        "MISSION01 UPGRADES FIXTURE: synthetic bank=40 salvage and XP=1000; keyboard shop purchases/equipment and ordinary three-card earned offers; scripted payload poses; contact damage disabled every active frame. No save access. Presentation/integration evidence, not natural progression or balance evidence."
    );
    app.insert_resource(Check {
        started: Instant::now(),
        step: 0,
        next_step_at: 0.75,
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
        pending: None,
        baseline: Vec::new(),
        captured_choices: 0,
    })
    .add_systems(Startup, resize)
    .add_systems(PreUpdate, drive.after(InputSystems))
    .add_systems(
        Update,
        protect
            .after(GameplaySet::ChoiceInput)
            .before(GameplaySet::Combat),
    )
    .add_systems(PostUpdate, capture.after(UiSystems::PostLayout));
}

fn resize(mut check: ResMut<Check>, mut window: Single<&mut Window>) {
    check.started = Instant::now();
    if check.minimum {
        window.resolution.set(640., 480.);
    }
}

fn protect(phase: Res<GamePhase>, mut combat: ResMut<CombatConfig>) {
    // Fixture-only survivability override follows choices/reset because both
    // restore CombatConfig from baseline. Ordinary gameplay never installs it.
    if matches!(*phase, GamePhase::Playing | GamePhase::Choosing) {
        combat.contact_damage = 0;
    }
}

fn expected() -> [Option<ModuleKind>; 4] {
    [
        Some(ModuleKind::Repair),
        Some(ModuleKind::Repulsor),
        Some(ModuleKind::Overdrive),
        None,
    ]
}

fn press(world: &mut World, key: KeyCode) {
    world.resource_mut::<ButtonInput<KeyCode>>().press(key);
}

fn drive(world: &mut World) {
    world.resource_scope(|world, mut check: Mut<Check>| {
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        let elapsed = check.started.elapsed().as_secs_f64();
        assert!(elapsed < 40., "Mission01 upgrades fixture timed out at step {}", check.step);
        if elapsed < check.next_step_at {
            return;
        }
        let phase = *world.resource::<GamePhase>();
        match check.step {
            0 => {
                world.resource_mut::<Campaign>().wallet.salvage = 40;
                press(world, KeyCode::KeyM);
            }
            1 => check.pending = Some("shop-six-rows"),
            2 | 6 | 10 => press(world, KeyCode::KeyB),
            3 => press(world, KeyCode::Digit3),
            4 | 8 => press(world, KeyCode::ArrowUp),
            5 => {
                assert_eq!(world.resource::<MissionSession>().selected_module, ModuleKind::Repair as usize);
                check.pending = Some("shop-repair-detail");
            }
            7 => press(world, KeyCode::Digit1),
            9 => {
                assert_eq!(world.resource::<MissionSession>().selected_module, ModuleKind::Repulsor as usize);
                check.pending = Some("shop-repulsor-detail");
            }
            11 => press(world, KeyCode::Digit2),
            12 => {
                let campaign = world.resource::<Campaign>();
                assert_eq!(campaign.wallet.salvage, 0);
                assert_eq!(campaign.inventory.loadout().slots(), &expected());
                for kind in expected().into_iter().flatten() {
                    assert!(campaign.inventory.owns(kind));
                }
                check.pending = Some("shop-equipped");
            }
            13 => press(world, KeyCode::Backspace),
            14 | 16 | 34 => press(world, KeyCode::Enter),
            15 => {
                assert_eq!(phase, GamePhase::Briefing);
                check.pending = Some("briefing");
            }
            17 => {
                assert_eq!(phase, GamePhase::Playing);
                assert!(world.resource::<super::blockout::BlockoutRun>().enabled);
                assert_eq!(world.resource::<Modules>().loadout.slots(), &expected());
                assert_eq!(world.resource::<UpgradeRun>().total_xp, 0);
                check.baseline = effective_config(world);
                check.pending = Some("launch");
            }
            18 => world.resource_mut::<UpgradeRun>().award(1_000),
            19 | 21 | 23 | 25 => {
                assert_eq!(phase, GamePhase::Choosing);
                let run = world.resource::<UpgradeRun>();
                assert_eq!(run.offer.len(), 3, "fixture must use ordinary three-card offers");
                assert_eq!(run.resolved as usize, (check.step - 19) / 2);
                assert!(run.offer.iter().all(|kind| kind.eligible(&world.resource::<Modules>().loadout)));
                check.pending = Some(match check.step {
                    19 => "choice-1", 21 => "choice-2", 23 => "choice-3", _ => "choice-4",
                });
            }
            20 | 22 | 24 | 26 => {
                assert_eq!(phase, GamePhase::Choosing);
                let run = world.resource::<UpgradeRun>();
                // Prefer newly enabled cards, without injecting/reordering offers.
                let choice = run.offer.iter().position(|kind| !UpgradeKind::ALL.contains(kind)).unwrap_or(0);
                println!("MISSION01 UPGRADES select: offered={:?} chosen={:?}", run.offer, run.offer[choice]);
                press(world, [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3][choice]);
            }
            27 => {
                assert_eq!(phase, GamePhase::Playing);
                let run = world.resource::<UpgradeRun>();
                assert_eq!(run.resolved, 4);
                assert_eq!(run.selected.len(), 4);
                assert_eq!(run.remaining(), 0);
                assert!(run.offer.is_empty() && run.pending == 0 && run.exhausted);
                assert!(run.selected.iter().any(|kind| !UpgradeKind::ALL.contains(kind)));
                assert_ne!(effective_config(world), check.baseline, "selected cards must change live gameplay config");
                assert_eq!(world.resource::<Modules>().loadout.slots(), &expected());
                check.pending = Some("selected-build");
            }
            28 => press(world, KeyCode::KeyR),
            29 => {
                assert_eq!(phase, GamePhase::Playing);
                let run = world.resource::<UpgradeRun>();
                assert_eq!(run.resolved, 0);
                assert_eq!(run.total_xp, 0);
                assert!(run.selected.is_empty() && run.offer.is_empty());
                assert_eq!(effective_config(world), check.baseline, "restart must restore every modified config field");
                assert_eq!(world.resource::<Modules>().loadout.slots(), &expected());
                check.pending = Some("reset-cleared");
            }
            30 => teleport(world, map::pickup()),
            31 => {
                assert!(world.resource::<ObjectiveRun>().ready());
                check.pending = Some("payload");
            }
            32 => teleport(world, map::delivery()),
            33 => {
                assert_eq!(phase, GamePhase::Survived);
                assert!(world.resource::<MissionSession>().result.as_ref().is_some_and(|result| result.succeeded));
                check.pending = Some("result");
            }
            35 => {
                assert_eq!(phase, GamePhase::Hub);
                assert_eq!(world.resource::<Campaign>().inventory.loadout().slots(), &expected());
                assert_eq!(check.captured_choices, 4);
                println!("MISSION01 UPGRADES FIXTURE PASS: six shop rows; Repair/Repulsor purchases; three equipped slots; four ordinary three-card choices captured after layout; live effects applied; reset cleared effects; payload delivered; owned support loadout retained. No save access.");
                world.write_message(AppExit::Success);
            }
            _ => panic!("unexpected fixture step {}", check.step),
        }
        check.step += 1;
        // Preserve a released-input frame even after a slow screenshot/render.
        check.next_step_at = elapsed + 0.75;
    });
}

/// All live configuration fields changed by the twelve temporary upgrades.
fn effective_config(world: &World) -> Vec<f64> {
    let flight = world.resource::<FlightConfig>();
    let combat = world.resource::<CombatConfig>();
    let energy = world.resource::<EnergyConfig>();
    let modules = world.resource::<ModuleConfig>();
    let mut values = vec![
        flight.horizontal_acceleration_multiplier as f64,
        flight.max_horizontal_speed as f64,
        flight.acceleration_multiplier as f64,
        flight.yaw_rate as f64,
        flight.bank_yaw_rate as f64,
        flight.tilt_rate as f64,
        flight.leveling_rate as f64,
        combat.player_health as f64,
        combat.shot_damage as f64,
        combat.target_range as f64,
        combat.fire_interval,
        energy.capacity,
        energy.activation,
        modules.shield_recharge,
        modules.rocket_radius as f64,
        modules.rocket_interval,
        modules.overdrive_multiplier,
        modules.repair_rate,
        modules.repair_drain,
        modules.repulsor_radius as f64,
        modules.repulsor_interval,
        modules.repulsor_drain,
    ];
    values.extend(modules.drains);
    values
}

fn teleport(world: &mut World, position: Vec3) {
    let (mut transform, mut flight) = world
        .query_filtered::<(&mut Transform, &mut DroneFlight), With<Drone>>()
        .single_mut(world)
        .unwrap();
    transform.translation = position;
    *flight = default();
}

#[allow(clippy::too_many_arguments)]
fn capture(
    mut commands: Commands,
    mut check: ResMut<Check>,
    phase: Res<GamePhase>,
    run: Res<UpgradeRun>,
    campaign: Res<Campaign>,
    modules: Res<Modules>,
    actions: Query<(&MissionAction, &ComputedNode)>,
    cards: Query<(&ChoiceAction, &ComputedNode)>,
    texts: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
) {
    let Some(label) = check.pending.take() else {
        return;
    };
    if *phase == GamePhase::ModuleShop {
        assert_eq!(
            actions
                .iter()
                .filter(
                    |(action, node)| matches!(action, MissionAction::SelectModule(_))
                        && node.size().min_element() > 1.
                )
                .count(),
            6,
            "six visible catalog rows required"
        );
    }
    if *phase == GamePhase::Choosing {
        assert_eq!(
            cards
                .iter()
                .filter(|(action, node)| matches!(action, ChoiceAction::Pick(_))
                    && node.size().min_element() > 1.)
                .count(),
            3
        );
        check.captured_choices += 1;
    }
    if matches!(
        *phase,
        GamePhase::ModuleShop | GamePhase::Briefing | GamePhase::Choosing
    ) {
        for (text, node, transform) in &texts {
            if node.size().min_element() <= 1. {
                continue;
            }
            let center = transform.translation / window.scale_factor();
            let half = node.size() / window.scale_factor() / 2.;
            assert!(
                center.x - half.x >= -1.
                    && center.y - half.y >= -1.
                    && center.x + half.x <= window.width() + 1.
                    && center.y + half.y <= window.height() + 1.,
                "{label}: text outside viewport: {:?} center={center:?} half={half:?}",
                text.0
            );
        }
    }
    println!(
        "MISSION01 UPGRADES {label}: phase={phase:?} bank={:?} loadout={:?} resolved={} pending={} offer={:?} selected={:?}",
        campaign.wallet,
        modules.loadout.slots(),
        run.resolved,
        run.pending,
        run.offer,
        run.selected
    );
    let size = if check.minimum { "640x480" } else { "1120x720" };
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(
            check
                .directory
                .join(format!("mission01-upgrades-{size}-{label}.png")),
        ));
}
