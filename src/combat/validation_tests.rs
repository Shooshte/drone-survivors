use super::super::validation::{self, ValidationConfig, ValidationMode};
use super::*;

#[test]
fn validation_arguments_are_opt_in_bounded_and_mode_specific() {
    let parse = |args: &[&str]| ValidationConfig::parse(args.iter().map(|s| s.to_string()));
    assert!(parse(&[]).unwrap().is_none());
    let stress = parse(&["--validate", "stress"]).unwrap().unwrap();
    assert_eq!(
        (stress.mode, stress.enemies, stress.seconds),
        (ValidationMode::Stress, 150, 30.)
    );
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
fn scripted_keyboard_pilot_completes_full_survival_with_real_health() {
    use crate::arena::DroneFlight;
    for rate in [30, 60, 120] {
        let (mut app, drone) = app();
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
