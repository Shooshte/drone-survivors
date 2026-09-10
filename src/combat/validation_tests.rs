use super::super::validation::{self, ValidationConfig, ValidationMode};
use super::*;
use crate::upgrades::{UpgradeKind, UpgradeRun};
use std::time::{Duration, Instant};

#[test]
fn validation_arguments_are_opt_in_bounded_and_mode_specific() {
    let parse = |args: &[&str]| ValidationConfig::parse(args.iter().map(|s| s.to_string()));
    assert!(parse(&[]).unwrap().is_none());
    let stress = parse(&["--validate", "stress"]).unwrap().unwrap();
    assert_eq!(
        (stress.mode, stress.enemies, stress.seconds),
        (ValidationMode::Stress, 150, 30.)
    );
    for (name, mode, seconds) in [
        ("mobile", ValidationMode::Mobile, 305.),
        ("armored", ValidationMode::Armored, 305.),
        ("choices", ValidationMode::Choices, 60.),
    ] {
        let config = parse(&["--validate", name]).unwrap().unwrap();
        assert_eq!((config.mode, config.seconds), (mode, seconds));
    }
    for args in [
        vec!["--seconds", "30"],
        vec!["--validate", "stress", "--seconds", "NaN"],
        vec!["--validate", "stress", "--enemies", "0"],
        vec!["--validate", "survival", "--enemies", "150"],
        vec!["--validate", "stress", "--validate", "idle"],
        vec!["--validate"],
    ] {
        assert!(parse(&args).is_err(), "{args:?}");
    }
}

#[test]
fn final_build_summary_reports_acquired_names_and_battery_values() {
    let mut run = UpgradeRun::default();
    run.selected = vec![UpgradeKind::Interceptor, UpgradeKind::AgileFrame];
    let energy = crate::energy::Energy {
        current: 41.25,
        ..default()
    };
    let config = crate::energy::EnergyConfig {
        capacity: 75.,
        ..default()
    };

    assert_eq!(
        validation::build_summary(Some(&run), Some(&energy), Some(&config)),
        "acquired=[Interceptor|Agile frame] energy=41.250 capacity=75.000"
    );
}

fn upgrade_validation_app(mode: ValidationMode) -> App {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1. / 30.),
        ))
        .add_message::<AppExit>()
        .add_plugins((
            bevy::time::TimePlugin,
            crate::arena::ArenaPlugin,
            CombatPlugin,
            crate::upgrades::runtime::UpgradePlugin,
        ));
    validation::install(
        &mut app,
        ValidationConfig {
            mode,
            seconds: if mode == ValidationMode::Choices {
                60.
            } else {
                305.
            },
            enemies: 150,
        },
    );
    app.update();
    app
}

fn validation_tick(app: &mut App, dt: f32, keys: &[KeyCode]) {
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
fn mobile_mode_uses_fresh_normal_choice_input_and_records_the_resolution() {
    let mut app = upgrade_validation_app(ValidationMode::Mobile);
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    validation_tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    let offer = app.world().resource::<UpgradeRun>().offer.clone();
    let expected = [
        UpgradeKind::Interceptor,
        UpgradeKind::AgileFrame,
        UpgradeKind::RapidShield,
    ]
    .into_iter()
    .find(|kind| offer.contains(kind));

    validation_tick(&mut app, 0., &[]);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 1);
    validation_tick(&mut app, 0., &[]);

    assert_eq!(
        app.world()
            .resource::<UpgradeRun>()
            .selected
            .last()
            .copied(),
        expected
    );
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 0);
    let choices = app.world().resource::<validation::ValidationChoices>();
    assert_eq!(choices.entries.len(), 1);
    assert_eq!(choices.entries[0].action, expected);
    assert_eq!(choices.entries[0].level, 2);
}

#[test]
fn legacy_modes_skip_earned_choices_through_the_runtime() {
    for mode in [
        ValidationMode::Survival,
        ValidationMode::Stress,
        ValidationMode::Idle,
    ] {
        let mut app = upgrade_validation_app(mode);
        app.world_mut().resource_mut::<UpgradeRun>().award(50);
        validation_tick(&mut app, 0., &[]);
        validation_tick(&mut app, 0., &[]);
        validation_tick(&mut app, 0., &[]);

        let run = app.world().resource::<UpgradeRun>();
        assert_eq!(run.pending, 0, "{mode:?}");
        assert!(run.selected.is_empty(), "{mode:?}");
        assert_eq!(
            app.world()
                .resource::<validation::ValidationChoices>()
                .entries[0]
                .action,
            None
        );
    }
}

#[test]
fn choices_mode_opens_two_user_controlled_choices_at_640_by_480() {
    let mut app = App::new();
    app.world_mut().spawn(Window::default());
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
        ));
    validation::install(
        &mut app,
        ValidationConfig {
            mode: ValidationMode::Choices,
            seconds: 60.,
            enemies: 150,
        },
    );
    app.update();

    let run = app.world().resource::<UpgradeRun>();
    assert_eq!((run.level, run.xp, run.pending), (3, 15, 2));
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    let window = app
        .world_mut()
        .query::<&Window>()
        .single(app.world())
        .unwrap();
    assert_eq!((window.width(), window.height()), (640., 480.));

    validation_tick(&mut app, 0., &[]);
    validation_tick(&mut app, 0., &[KeyCode::Backspace]);
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 1);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());

    app.world_mut()
        .resource_mut::<validation::ValidationChoices>()
        .preview_started = Some(Instant::now() - Duration::from_secs(2));
    validation_tick(&mut app, 0., &[]);
    assert!(
        app.world()
            .resource::<validation::ValidationChoices>()
            .preview_requested
    );
    assert_eq!(
        app.world_mut()
            .query::<&bevy::render::view::screenshot::Screenshot>()
            .iter(app.world())
            .count(),
        1
    );
}

#[test]
fn full_mobile_and_armored_runs_finish_and_log_deterministic_builds() {
    for (mode, priorities) in [
        (
            ValidationMode::Mobile,
            &[
                UpgradeKind::Interceptor,
                UpgradeKind::AgileFrame,
                UpgradeKind::RapidShield,
            ][..],
        ),
        (
            ValidationMode::Armored,
            &[
                UpgradeKind::HeavyArmor,
                UpgradeKind::HeavyRounds,
                UpgradeKind::WideAreaRockets,
            ][..],
        ),
    ] {
        let mut app = upgrade_validation_app(mode);
        for _ in 0..(310 * 30) {
            validation_tick(&mut app, 1. / 30., &[]);
            if matches!(
                *app.world().resource::<GamePhase>(),
                GamePhase::Dead | GamePhase::Survived
            ) {
                break;
            }
        }

        assert!(
            matches!(
                *app.world().resource::<GamePhase>(),
                GamePhase::Dead | GamePhase::Survived
            ),
            "{mode:?} did not reach an outcome"
        );
        let run = app.world().resource::<UpgradeRun>();
        println!(
            "deterministic validation: mode={mode:?} outcome={:?} run_seconds={:.3} kills={} choices={:?}",
            app.world().resource::<GamePhase>(),
            app.world().resource::<Encounter>().elapsed,
            app.world().resource::<Encounter>().kills,
            app.world()
                .resource::<validation::ValidationChoices>()
                .entries
        );
        assert!(!run.selected.is_empty(), "{mode:?}");
        assert!(run.selected.iter().all(|kind| priorities.contains(kind)));
        let choices = app.world().resource::<validation::ValidationChoices>();
        assert!(!choices.entries.is_empty(), "{mode:?}");
        assert_eq!(
            choices
                .entries
                .iter()
                .filter_map(|entry| entry.action)
                .collect::<Vec<_>>(),
            run.selected
        );
    }
}

#[test]
fn stress_harness_maintains_actual_live_population_while_real_hits_and_kills_run() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<AppExit>()
        .add_plugins((crate::arena::ArenaPlugin, CombatPlugin));
    validation::install(
        &mut app,
        ValidationConfig {
            mode: ValidationMode::Stress,
            seconds: 30.,
            enemies: 150,
        },
    );
    app.update();
    assert_eq!(count::<Enemy>(&mut app), 150);
    assert!(app.world().resource::<WaveConfig>().bursts.is_empty());
    for _ in 0..600 {
        step(&mut app, 1. / 60., &[]);
        assert_eq!(count::<Enemy>(&mut app), 150);
        assert_eq!(count::<SpawnWarning>(&mut app), 0);
    }
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert!(app.world().resource::<Encounter>().kills > 0);
}

#[test]
fn legacy_three_minute_empty_arena_pilot_survives_the_previous_schedule() {
    use crate::arena::DroneFlight;
    for rate in [30, 60, 120] {
        let (mut app, drone) = app();
        {
            let mut waves = app.world_mut().resource_mut::<WaveConfig>();
            waves.duration = 180.;
            waves.bursts = vec![
                (3., 3),
                (15., 3),
                (27., 3),
                (39., 3),
                (60., 4),
                (70., 4),
                (80., 4),
                (90., 4),
                (120., 5),
                (128., 5),
                (136., 5),
                (144., 5),
                (152., 5),
            ];
        }
        let mut peak_enemies = 0;
        for _ in 0..(181 * rate) {
            let run = app.world().resource::<Encounter>().elapsed;
            let threats: Vec<_> = app
                .world_mut()
                .query_filtered::<(&Transform, &DroneFlight), With<Enemy>>()
                .iter(app.world())
                .map(|(t, f)| (t.translation, f.velocity))
                .collect();
            let input = super::super::validation::pilot_keys(
                run,
                position(&app, drone),
                app.world().get::<DroneFlight>(drone).unwrap(),
                &threats,
            );
            step(&mut app, 1. / rate as f32, &input);
            peak_enemies = peak_enemies.max(count::<Enemy>(&mut app));
            if *app.world().resource::<GamePhase>() != GamePhase::Playing {
                break;
            }
        }
        println!(
            "survival probe: phase={:?}, seconds={:.3}, health={}, kills={}, peak_enemies={peak_enemies}",
            app.world().resource::<GamePhase>(),
            app.world().resource::<Encounter>().elapsed,
            app.world().resource::<PlayerHealth>().current,
            app.world().resource::<Encounter>().kills
        );
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
        assert!(app.world().resource::<Encounter>().kills >= 15);
        assert!(app.world().resource::<PlayerHealth>().current > 0);
    }
}

#[test]
fn validation_pilot_preserves_restart_and_escape_input() {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<AppExit>()
        .add_plugins((crate::arena::ArenaPlugin, CombatPlugin));
    validation::install(
        &mut app,
        ValidationConfig {
            mode: ValidationMode::Survival,
            seconds: 185.,
            enemies: 150,
        },
    );
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    step(&mut app, 1., &[]);
    assert_ne!(position(&app, drone), START);
    step(&mut app, 1., &[KeyCode::KeyR, KeyCode::Escape]);
    assert_eq!(position(&app, drone), START);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape)
    );
}

#[test]
fn wave_override_modes_do_not_report_authored_phases_or_lulls() {
    use super::super::scene::{CombatHud, CombatScenePlugin};
    for mode in [ValidationMode::Stress, ValidationMode::Routes] {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_message::<AppExit>()
            .add_plugins((crate::arena::ArenaPlugin, CombatPlugin, CombatScenePlugin));
        validation::install(
            &mut app,
            ValidationConfig {
                mode,
                seconds: 30.,
                enemies: 3,
            },
        );
        app.update();
        let waves = app.world().resource::<WaveConfig>();
        assert!(waves.bursts.is_empty());
        for at in [0., 39., 45., 60., 105., 299.] {
            assert!(waves.phase_at(at).is_none(), "{mode:?} stale phase at {at}");
            assert!(
                waves.status_at(at).is_none(),
                "{mode:?} stale status at {at}"
            );
        }
        for at in [0., 39., 45.] {
            app.world_mut().resource_mut::<Encounter>().elapsed = at;
            step(&mut app, 0., &[]);
            let text = app
                .world_mut()
                .query_filtered::<&Text, With<CombatHud>>()
                .single(app.world())
                .unwrap();
            assert!(text.0.contains("AUTO FIRE"));
            for stale in ["OPENING", "PRESSURE", "SPAWNING LULL"] {
                assert!(!text.0.contains(stale), "{mode:?}: {}", text.0);
            }
        }
    }
}
