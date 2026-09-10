use super::super::waves::SpawnCounts;
use super::*;
use crate::{
    arena::DRONE_START,
    energy::Energy,
    modules::Modules,
    upgrades::{UpgradeKind, UpgradeRun},
};
use std::time::Duration;

fn parse(args: &[&str]) -> ValidationConfig {
    ValidationConfig::parse(args.iter().map(|arg| arg.to_string()))
        .unwrap()
        .unwrap()
}

fn manual_app() -> App {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::ZERO,
        ))
        .add_message::<AppExit>()
        .add_plugins((
            bevy::time::TimePlugin,
            crate::arena::ArenaPlugin,
            CombatPlugin,
            crate::upgrades::runtime::UpgradePlugin,
        ))
        .init_resource::<crate::world::WorldGeometry>();
    install(
        &mut app,
        ValidationConfig {
            mode: ValidationMode::Manual,
            seconds: 305.,
            enemies: 150,
        },
    );
    app.update();
    app
}

fn tick(app: &mut App, dt: f32, keys: &[KeyCode]) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    for &key in keys {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    *app.world_mut()
        .resource_mut::<bevy::time::TimeUpdateStrategy>() =
        bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(dt));
    app.update();
}

#[test]
fn encounter_modes_default_to_a_five_minute_sample_with_short_probe_exceptions() {
    for name in ["survival", "idle", "routes", "mobile", "armored", "manual"] {
        assert_eq!(parse(&["--validate", name]).seconds, 305., "{name}");
    }
    assert_eq!(parse(&["--validate", "stress"]).seconds, 30.);
    assert_eq!(parse(&["--validate", "choices"]).seconds, 60.);
    assert_eq!(
        parse(&["--validate", "manual"]).mode,
        ValidationMode::Manual
    );
}

#[test]
fn manual_mode_preserves_waves_stats_and_fresh_player_input() {
    let mut app = manual_app();
    let original_waves = {
        let waves = app.world().resource::<WaveConfig>();
        (
            waves.duration,
            waves.cap,
            waves.warning_seconds,
            waves.clearance,
            waves.bursts.clone(),
        )
    };
    assert_eq!(original_waves, {
        let waves = WaveConfig::default();
        (
            waves.duration,
            waves.cap,
            waves.warning_seconds,
            waves.clearance,
            waves.bursts,
        )
    });
    assert_eq!(
        {
            let run = app.world().resource::<UpgradeRun>();
            (run.level, run.xp, run.pending, run.selected.clone())
        },
        (1, 0, 0, Vec::new())
    );

    app.world_mut().resource_mut::<PlayerHealth>().current = 42;
    app.world_mut().resource_mut::<Energy>().current = 33.;
    tick(&mut app, 0., &[KeyCode::KeyW, KeyCode::Digit1]);
    let keys = app.world().resource::<ButtonInput<KeyCode>>();
    assert!(keys.pressed(KeyCode::KeyW));
    assert!(keys.pressed(KeyCode::Digit1));
    assert_eq!(app.world().resource::<PlayerHealth>().current, 42);
    assert_eq!(app.world().resource::<Energy>().current, 33.);
    assert!(app.world().resource::<Modules>().enabled[0]);

    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    tick(&mut app, 0.5, &[KeyCode::KeyW]);
    assert_ne!(
        app.world().get::<Transform>(drone).unwrap().translation,
        DRONE_START.translation
    );

    let mut no_input = manual_app();
    let no_input_drone = no_input
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(no_input.world())
        .unwrap();
    tick(&mut no_input, 0.5, &[]);
    assert_eq!(
        no_input
            .world()
            .get::<Transform>(no_input_drone)
            .unwrap()
            .translation,
        DRONE_START.translation
    );
}

#[test]
fn manual_mode_leaves_upgrade_choices_under_player_control() {
    let mut app = manual_app();
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    let expected = app.world().resource::<UpgradeRun>().offer[0];

    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 1);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());

    tick(&mut app, 0., &[KeyCode::Digit1]);
    assert_eq!(
        app.world().resource::<UpgradeRun>().selected,
        vec![expected]
    );
    assert_eq!(
        app.world()
            .resource::<ValidationChoices>()
            .entries
            .last()
            .unwrap()
            .action,
        Some(expected)
    );
}

#[test]
fn first_playing_frame_after_a_long_choice_pause_is_not_sampled() {
    let start = Instant::now();
    let mut measurements = Measurements::default();
    measurements.observe_frame_boundary(start, GamePhase::Playing);
    measurements.observe_frame_boundary(start + Duration::from_secs(5), GamePhase::Playing);
    measurements.observe_frame_boundary(start + Duration::from_millis(5_016), GamePhase::Playing);
    measurements.observe_frame_boundary(start + Duration::from_secs(65), GamePhase::Choosing);
    measurements.observe_frame_boundary(start + Duration::from_millis(65_016), GamePhase::Playing);
    measurements.observe_frame_boundary(start + Duration::from_millis(65_032), GamePhase::Playing);

    assert_eq!(measurements.samples.len(), 2);
    assert!(
        measurements
            .samples
            .iter()
            .all(|sample| (*sample - 16.).abs() < 0.001),
        "{:?}",
        measurements.samples
    );
}

#[test]
fn stage_population_summary_exposes_live_and_reserved_saturation() {
    let mut measurements = Measurements::default();
    measurements.observe_population("Opening", 4, 2);
    measurements.observe_population("Opening", 7, 1);
    measurements.observe_population("Final", 28, 2);

    assert_eq!(
        measurements.stage_population_summary(),
        "Opening:live=4..7 active_peak=8|Final:live=28..28 active_peak=30"
    );
}

#[test]
fn spawn_summary_names_every_outcome_counter() {
    assert_eq!(
        spawn_summary(SpawnCounts {
            requested: 40,
            admitted: 30,
            rejected_cap: 4,
            rejected_space: 3,
            skipped_hitch: 3,
            skipped_terminal: 0,
            activated: 26,
            cancelled: 2,
        }),
        "requested=40 admitted=30 rejected_cap=4 rejected_space=3 skipped_hitch=3 skipped_terminal=0 activated=26 cancelled=2 pending=2"
    );
}

#[test]
fn manual_observations_record_changes_and_restart_clears_the_record() {
    let mut app = manual_app();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();

    tick(&mut app, 0., &[KeyCode::Digit1]);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = Vec3::new(-280., 90., 0.);
    tick(&mut app, 0., &[]);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = Vec3::new(200., 90., 150.);
    tick(&mut app, 0., &[]);

    let observations = app.world().resource::<ManualObservations>();
    assert!(observations.events.iter().any(|event| matches!(
        event,
        ObservationEvent::Module {
            slot: 0,
            enabled: true,
            ..
        }
    )));
    assert!(
        observations
            .events
            .iter()
            .any(|event| matches!(event, ObservationEvent::Charger { entered: true, .. }))
    );
    assert!(
        observations
            .events
            .iter()
            .any(|event| matches!(event, ObservationEvent::Charger { entered: false, .. }))
    );
    assert!(observations.events.iter().any(|event| matches!(
        event,
        ObservationEvent::RouteCrossing { z, .. } if (*z - 125.).abs() < 0.001
    )));

    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert!(
        app.world()
            .resource::<ManualObservations>()
            .events
            .is_empty()
    );
    assert!(
        app.world()
            .resource::<ValidationChoices>()
            .entries
            .is_empty()
    );
}

#[test]
fn route_crossing_survives_a_frame_exactly_on_the_divider_plane() {
    let mut observations = ManualObservations::default();
    observations.observe_route(1., Vec3::new(100., 90., 60.), 120.);
    observations.observe_route(2., Vec3::new(120., 90., 80.), 120.);
    observations.observe_route(3., Vec3::new(140., 90., 100.), 120.);

    assert_eq!(
        observations
            .events
            .iter()
            .filter(|event| matches!(event, ObservationEvent::RouteCrossing { .. }))
            .count(),
        1
    );
}

#[test]
fn initial_manual_state_is_a_baseline_not_a_deliberate_decision() {
    let mut app = manual_app();
    app.world_mut().resource_mut::<Modules>().enabled = [true, false, true, false];
    {
        let mut observations = app.world_mut().resource_mut::<ManualObservations>();
        *observations = ManualObservations::default();
    }
    tick(&mut app, 0., &[]);

    assert!(
        app.world()
            .resource::<ManualObservations>()
            .events
            .is_empty()
    );
}

#[test]
fn observation_choice_entry_keeps_the_selected_upgrade_name() {
    let entry = ValidationChoiceEntry {
        run_seconds: 12.,
        level: 2,
        action: Some(UpgradeKind::AgileFrame),
    };
    assert_eq!(entry.action.unwrap().name(), "Agile frame");
}
