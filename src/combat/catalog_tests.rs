use super::*;
use crate::{
    arena::{ArenaPlugin, Drone, DroneFlight},
    energy::{ChargerConfig, ChargerReserve, Energy, EnergyConfig},
    modules::{ModuleKind, Modules},
    upgrades::{UpgradeRun, runtime::UpgradePlugin},
};
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

#[test]
fn catalog_selects_fast_pursuer_and_preserves_type_through_spawn_warning() {
    let mut app = app();
    for _ in 0..3 {
        step(&mut app, &[KeyCode::ArrowRight]);
    }
    assert!(
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|t| t.0.contains("Fast pursuer"))
    );
    step(&mut app, &[KeyCode::Enter]);
    for _ in 0..120 {
        step(&mut app, &[]);
    }
    let kinds: Vec<_> = app
        .world_mut()
        .query::<&Enemy>()
        .iter(app.world())
        .map(|e| format!("{:?}", e.kind))
        .collect();
    assert_eq!(kinds, vec!["Fast"]);
}

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<crate::world::WorldGeometry>()
        .init_resource::<crate::world::PlayerPath>()
        .add_plugins((ArenaPlugin, CombatPlugin, UpgradePlugin));
    let config = validation::ValidationConfig::parse(["--validate", "catalog"].map(str::to_owned))
        .unwrap()
        .unwrap();
    validation::install(&mut app, config);
    app.update();
    app
}

fn step(app: &mut App, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for &key in keys {
        input.press(key);
    }
    app.update();
}

fn count<T: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .iter(app.world())
        .count()
}

#[test]
fn catalog_selection_is_paused_and_has_no_campaign_state() {
    let mut app = app();
    for _ in 0..120 {
        step(&mut app, &[]);
    }
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert!(!app.world().contains_resource::<crate::mission::Campaign>());
    assert!(
        !app.world()
            .contains_resource::<crate::mission::MissionSession>()
    );
    assert!(!app.is_plugin_added::<crate::save::SavePlugin>());
}

#[test]
fn catalog_launch_uses_unique_selected_slots_and_scenario_warnings() {
    let mut app = app();
    step(&mut app, &[KeyCode::Digit1]);
    step(&mut app, &[KeyCode::Digit2]);
    step(&mut app, &[KeyCode::ArrowRight]);
    step(&mut app, &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    let modules = app.world().resource::<Modules>();
    assert_eq!(
        modules.loadout.slots(),
        &[
            Some(ModuleKind::Overdrive),
            Some(ModuleKind::Shield),
            None,
            None
        ]
    );
    assert_eq!(modules.enabled, [false; 4]);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    for _ in 0..90 {
        step(&mut app, &[]);
    }
    assert_eq!(count::<SpawnWarning>(&mut app), 1);
    for _ in 0..30 {
        step(&mut app, &[]);
    }
    assert_eq!(app.world().resource::<Encounter>().spawns.activated, 1);
    step(&mut app, &[KeyCode::Digit1]);
    assert!(
        app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
}

#[test]
fn catalog_return_from_paused_upgrade_resets_every_attempt_system() {
    let mut app = app();
    step(&mut app, &[KeyCode::Digit1]);
    step(&mut app, &[KeyCode::Enter]);
    step(&mut app, &[]);
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    step(&mut app, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    app.world_mut().resource_mut::<PlayerHealth>().current = 7;
    app.world_mut().resource_mut::<Energy>().current = 3.;
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    for mut reserve in app
        .world_mut()
        .query::<&mut ChargerReserve>()
        .iter_mut(app.world_mut())
    {
        reserve.remaining = 0.;
    }
    app.world_mut().spawn((
        Projectile {
            velocity: Vec3::X,
            remaining: 10.,
        },
        Transform::default(),
    ));
    step(&mut app, &[KeyCode::Tab]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    assert_eq!(count::<Projectile>(&mut app), 0);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(
        app.world().resource::<Energy>().current,
        app.world().resource::<EnergyConfig>().capacity
    );
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    assert!(app.world().resource::<UpgradeRun>().offer.is_empty());
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    let capacity = app.world().resource::<ChargerConfig>().capacity;
    assert!(
        app.world_mut()
            .query::<&ChargerReserve>()
            .iter(app.world())
            .all(|r| r.remaining == capacity)
    );
    let (transform, flight) = app
        .world_mut()
        .query_filtered::<(&Transform, &DroneFlight), With<Drone>>()
        .single(app.world())
        .unwrap();
    assert_eq!(transform.translation, crate::arena::DRONE_START.translation);
    assert_eq!(flight.velocity, Vec3::ZERO);
    step(&mut app, &[KeyCode::Enter]);
    step(&mut app, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!(app.world().resource::<Encounter>().elapsed > 0.);
    assert_eq!(
        app.world().resource::<Modules>().loadout.slots()[0],
        Some(ModuleKind::Overdrive)
    );
}

fn click(app: &mut App, target: catalog::CatalogAction) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    let entity = app
        .world_mut()
        .query::<(Entity, &catalog::CatalogAction)>()
        .iter(app.world())
        .find(|(_, action)| **action == target)
        .unwrap()
        .0;
    *app.world_mut().get_mut::<Interaction>(entity).unwrap() = Interaction::Pressed;
    app.update();
    *app.world_mut().get_mut::<Interaction>(entity).unwrap() = Interaction::None;
    app.update();
}

#[test]
fn catalog_mouse_controls_cycle_slots_and_restart_terminal_rounds() {
    use catalog::CatalogAction::*;
    let mut app = app();
    click(&mut app, Next);
    click(&mut app, Next);
    click(&mut app, Previous);
    for slot in 0..4 {
        click(&mut app, CycleSlot(slot));
    }
    click(&mut app, Launch);
    assert_eq!(
        app.world().resource::<Modules>().loadout.slots(),
        &ModuleKind::ALL.map(Some)
    );
    assert_eq!(app.world().resource::<WaveConfig>().bursts, vec![(1., 1)]);
    app.world_mut().resource_mut::<PlayerHealth>().current = 0;
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    click(&mut app, Restart);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    click(&mut app, Return);
    click(&mut app, CycleSlot(0));
    click(&mut app, Launch);
    assert_eq!(
        app.world().resource::<Modules>().loadout.slots()[0],
        Some(ModuleKind::Repulsor)
    );
}

#[test]
fn catalog_held_controls_do_not_repeat_across_launch_or_return() {
    let mut app = app();
    step(&mut app, &[KeyCode::Digit1]);
    // Preserve held state exactly as real InputPlugin does between frames.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Tab);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    step(&mut app, &[KeyCode::Enter]);
    assert_eq!(
        app.world().resource::<Modules>().loadout.slots()[0],
        Some(ModuleKind::Overdrive)
    );
}

#[test]
fn catalog_choice_keeps_return_but_hides_restart_button_clear_of_modal_heading() {
    let mut app = app();
    step(&mut app, &[KeyCode::Enter]);
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    step(&mut app, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    for (action, node) in app
        .world_mut()
        .query::<(&catalog::CatalogAction, &Node)>()
        .iter(app.world())
    {
        if *action == catalog::CatalogAction::Restart {
            assert_eq!(node.display, Display::None);
        }
        if *action == catalog::CatalogAction::Return {
            assert_eq!(node.display, Display::Flex);
        }
    }
}

#[test]
fn catalog_selects_both_control_types_and_mixed_roster() {
    use crate::economy::runtime::EnemyKind;
    for (index, expected) in [
        (6, vec![EnemyKind::Slower]),
        (7, vec![EnemyKind::Jammer]),
        (
            8,
            vec![EnemyKind::Slower, EnemyKind::Jammer, EnemyKind::Fast],
        ),
    ] {
        let mut app = app();
        for _ in 0..index {
            step(&mut app, &[KeyCode::ArrowRight]);
        }
        step(&mut app, &[KeyCode::Enter]);
        for _ in 0..120 {
            step(&mut app, &[]);
        }
        let kinds: Vec<_> = app
            .world_mut()
            .query::<&Enemy>()
            .iter(app.world())
            .map(|e| e.kind)
            .collect();
        assert_eq!(kinds.len(), expected.len());
        for kind in expected {
            assert!(kinds.contains(&kind), "{kinds:?} missing {kind:?}");
        }
        step(&mut app, &[KeyCode::Tab]);
        assert_eq!(count::<Enemy>(&mut app), 0);
        assert_eq!(app.world().resource::<Modules>().disabled_for, [0.; 4]);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    }
}

#[test]
fn catalog_selects_bomb_mothership_and_mixed_ordnance() {
    use crate::economy::runtime::EnemyKind;
    for (index, expected) in [
        (9, vec![EnemyKind::Bomber]),
        (10, vec![EnemyKind::Mothership]),
        (
            11,
            vec![EnemyKind::Bomber, EnemyKind::Mothership, EnemyKind::Jammer],
        ),
    ] {
        let mut app = app();
        for _ in 0..index {
            step(&mut app, &[KeyCode::ArrowRight]);
        }
        for _ in 0..5 {
            step(&mut app, &[KeyCode::Digit1]);
        }
        step(&mut app, &[KeyCode::Enter]);
        assert_eq!(
            app.world().resource::<Modules>().loadout.slots()[0],
            Some(ModuleKind::Repulsor)
        );
        for _ in 0..120 {
            step(&mut app, &[]);
        }
        let kinds: Vec<_> = app
            .world_mut()
            .query::<&Enemy>()
            .iter(app.world())
            .map(|e| e.kind)
            .collect();
        assert_eq!(kinds.len(), expected.len());
        for kind in expected {
            assert!(kinds.contains(&kind));
        }
        app.world_mut().resource_mut::<bombs::BombState>().attach();
        step(&mut app, &[KeyCode::Tab]);
        assert!(
            app.world()
                .resource::<bombs::BombState>()
                .remaining
                .is_none()
        );
        assert_eq!(count::<Enemy>(&mut app), 0);
        assert_eq!(count::<SpawnWarning>(&mut app), 0);
    }
}

#[path = "catalog/environment_tests.rs"]
mod environment_tests;

#[test]
fn catalog_cycles_all_six_modules_with_readable_limits_and_unique_slots() {
    let mut app = app();
    let mut names = Vec::new();
    for _ in 0..6 {
        step(&mut app, &[KeyCode::Digit1]);
        let text = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .find(|t| t.0.starts_with("1   "))
            .unwrap()
            .0
            .clone();
        names.push(text);
    }
    assert!(names[5].contains("REPAIR"), "{names:?}");
    assert!(names[5].contains("6 hull/s"));
    assert!(names[5].contains("12 energy/s"));
    assert!(names[4].contains("180"));
    assert!(names[4].contains("2s"));
    for _ in 0..6 {
        step(&mut app, &[KeyCode::Digit2]);
    }
    step(&mut app, &[KeyCode::Enter]);
    let slots = app.world().resource::<Modules>().loadout.slots();
    assert_eq!(slots[0].unwrap().name(), "REPAIR");
    assert_eq!(slots[1], None, "cycling must skip Repair already in slot 1");
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
}
