use super::super::waves::{SpawnCounts, WaveStatus};
use super::*;

fn wave_app() -> (App, Entity) {
    let (mut app, drone) = app();
    quiet(&mut app);
    (app, drone)
}

fn spawn_counts(app: &App) -> SpawnCounts {
    app.world().resource::<Encounter>().spawns
}

#[test]
fn warning_is_a_reservation_then_spawns_at_its_warned_position() {
    let (mut app, _) = wave_app();
    step(&mut app, 2.99, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    step(&mut app, 0.01, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 3);
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert_eq!(
        spawn_counts(&app),
        SpawnCounts {
            requested: 3,
            admitted: 3,
            ..default()
        }
    );
    let positions: Vec<_> = app
        .world_mut()
        .query_filtered::<&Transform, With<SpawnWarning>>()
        .iter(app.world())
        .map(|t| t.translation)
        .collect();
    step(&mut app, 0.74, &[]);
    assert_eq!(count::<Enemy>(&mut app), 0);
    step(&mut app, 0.01, &[]);
    assert_eq!(count::<Enemy>(&mut app), 3);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(
        spawn_counts(&app),
        SpawnCounts {
            requested: 3,
            admitted: 3,
            activated: 3,
            ..default()
        }
    );
    for t in app
        .world_mut()
        .query_filtered::<&Transform, With<Enemy>>()
        .iter(app.world())
    {
        assert!(positions.contains(&t.translation));
    }
}

#[test]
fn cap_counts_reservations_and_saturation_creates_no_spawn_debt() {
    let (mut app, _) = wave_app();
    {
        let mut c = app.world_mut().resource_mut::<WaveConfig>();
        c.cap = 4;
        c.bursts = vec![(0., 3), (0.1, 3), (0.2, 3)];
    }
    step(&mut app, 0., &[]);
    step(&mut app, 0.1, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 4);
    step(&mut app, 0.1, &[]);
    step(&mut app, 0.75, &[]);
    assert_eq!(count::<Enemy>(&mut app), 4);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(
        spawn_counts(&app),
        SpawnCounts {
            requested: 9,
            admitted: 4,
            rejected_cap: 5,
            activated: 4,
            ..default()
        }
    );
    let ids: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .collect();
    for id in ids {
        app.world_mut().despawn(id);
    }
    step(&mut app, 1., &[]);
    assert_eq!(
        count::<SpawnWarning>(&mut app) + count::<Enemy>(&mut app),
        0
    );
}

#[test]
fn moving_into_warning_cancels_spawn_without_relocation() {
    let (mut app, drone) = wave_app();
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(0., 1)];
    step(&mut app, 0., &[]);
    let warned = app
        .world_mut()
        .query_filtered::<&Transform, With<SpawnWarning>>()
        .single(app.world())
        .unwrap()
        .translation;
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = warned;
    step(&mut app, 0.75, &[]);
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(app.world().resource::<Encounter>().next_burst, 1);
    assert_eq!(
        spawn_counts(&app),
        SpawnCounts {
            requested: 1,
            admitted: 1,
            cancelled: 1,
            ..default()
        }
    );
}

#[test]
fn occupied_or_too_small_arena_skips_candidates_and_finishes_search() {
    let (mut app, _) = wave_app();
    app.world_mut()
        .resource_mut::<crate::arena::Arena>()
        .half_size = Vec3::splat(20.);
    step(&mut app, 3., &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(
        spawn_counts(&app),
        SpawnCounts {
            requested: 3,
            rejected_space: 3,
            ..default()
        }
    );
}

#[test]
fn hitch_only_warns_latest_due_burst_and_gives_full_warning_duration() {
    let (mut app, _) = wave_app();
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(1., 1), (2., 2), (3., 3)];
    step(&mut app, 4., &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 3);
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert_eq!(
        spawn_counts(&app),
        SpawnCounts {
            requested: 6,
            admitted: 3,
            skipped_hitch: 3,
            ..default()
        }
    );
    step(&mut app, 0.74, &[]);
    assert_eq!(count::<Enemy>(&mut app), 0);
    step(&mut app, 0.01, &[]);
    assert_eq!(count::<Enemy>(&mut app), 3);
    assert_eq!(spawn_counts(&app).activated, 3);
}

#[test]
fn default_schedule_generates_authored_windows_and_phase_lulls() {
    let config = WaveConfig::default();
    assert_eq!(&config.bursts[..3], &[(3., 3), (13., 3), (23., 3)]);
    for (start, end, size, interval, expected) in [
        (30, 105, 3, 8, 9),
        (105, 165, 4, 8, 7),
        (165, 225, 6, 6, 9),
        (225, 300, 8, 4, 18),
    ] {
        let in_window: Vec<_> = config
            .bursts
            .iter()
            .copied()
            .filter(|(at, _)| *at >= start as f64 && *at < end as f64)
            .collect();
        assert_eq!(in_window.len(), expected);
        assert!(in_window.iter().all(|(_, count)| *count == size));
        assert!(
            in_window
                .windows(2)
                .all(|pair| pair[1].0 - pair[0].0 == interval as f64)
        );
        assert!(in_window.iter().all(|(at, _)| *at < (end - 6) as f64));
    }
    assert_eq!(config.bursts.len(), 46);
    assert_eq!(
        config.bursts.iter().map(|(_, count)| count).sum::<usize>(),
        262
    );

    for (at, label) in [
        (24., "OPENING"),
        (99., "PRESSURE I"),
        (159., "PRESSURE II"),
        (219., "PRESSURE III"),
        (294., "FINAL PUSH"),
    ] {
        let phase = config.phase_at(at).unwrap();
        assert_eq!(phase.label, label);
        assert_eq!(config.status_at(at), Some(WaveStatus::Lull));
        assert_eq!(config.status_at(phase.start), Some(WaveStatus::Active));
    }
    assert!(config.phase_at(300.).is_none());
    assert_eq!(config.status_at(300.), None);
}

#[test]
fn each_pressure_boundary_creates_one_real_warning_with_fixed_enemy_stats() {
    for (at, size) in [(30., 3), (105., 4), (165., 6), (225., 8)] {
        let (mut app, _) = wave_app();
        let burst_index = app
            .world()
            .resource::<WaveConfig>()
            .bursts
            .iter()
            .position(|(warning_at, _)| *warning_at == at)
            .unwrap();
        {
            let mut run = app.world_mut().resource_mut::<Encounter>();
            run.elapsed = at;
            run.next_burst = burst_index;
        }
        step(&mut app, 0., &[]);
        assert_eq!(count::<SpawnWarning>(&mut app), size);
        step(&mut app, 0., &[]);
        assert_eq!(count::<SpawnWarning>(&mut app), size, "duplicate at {at}");
        step(&mut app, 0.74, &[]);
        assert_eq!(count::<Enemy>(&mut app), 0, "early activation at {at}");
        step(&mut app, 0.01, &[]);
        assert_eq!(count::<Enemy>(&mut app), size);
        for (enemy, flight) in app
            .world_mut()
            .query::<(&Enemy, &crate::arena::DroneFlight)>()
            .iter(app.world())
        {
            assert_eq!(enemy.health, 20);
            assert_eq!(flight.velocity, Vec3::ZERO);
            assert_eq!(flight.tilt, Vec2::ZERO);
        }
    }
}

#[test]
fn survival_freezes_clock_and_gameplay_cancels_warnings_and_resets_completely() {
    let (mut app, drone) = wave_app();
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(299.5, 3), (300., 3)];
    step(&mut app, 180., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 180.);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    step(&mut app, 119.5, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 3);
    step(&mut app, 0.5, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    assert_eq!(spawn_counts(&app).cancelled, 3);
    let before = position(&app, drone);
    step(&mut app, 5., &[KeyCode::Space]);
    assert_eq!(position(&app, drone), before);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 300.);
    step(&mut app, 1., &[KeyCode::KeyR, KeyCode::Space]);
    let run = app.world().resource::<Encounter>();
    assert_eq!(
        (run.elapsed, run.kills, run.next_burst, run.candidate),
        (0., 0, 0, 0)
    );
    assert_eq!(run.spawns, SpawnCounts::default());
    assert_eq!(position(&app, drone), START);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}

#[test]
fn fatal_damage_wins_over_completion_and_restart_wins_over_both() {
    for reset in [false, true] {
        let (mut app, _) = wave_app();
        app.world_mut().resource_mut::<WaveConfig>().bursts.clear();
        let duration = app.world().resource::<WaveConfig>().duration;
        app.world_mut().resource_mut::<Encounter>().elapsed = duration - 0.1;
        app.world_mut().resource_mut::<PlayerHealth>().current = 1;
        enemy(&mut app, START, 100);
        step(&mut app, 0.1, if reset { &[KeyCode::KeyR] } else { &[] });
        assert_eq!(
            *app.world().resource::<GamePhase>(),
            if reset {
                GamePhase::Playing
            } else {
                GamePhase::Dead
            }
        );
        assert_eq!(
            app.world().resource::<PlayerHealth>().current,
            if reset { 100 } else { 0 }
        );
        if reset {
            assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
        }
    }
}

#[test]
fn restart_during_warning_repeats_positions_and_drops_old_reservations() {
    let (mut app, _) = wave_app();
    let mut previous = None;
    for _ in 0..3 {
        step(&mut app, 3., &[]);
        let mut points: Vec<_> = app
            .world_mut()
            .query_filtered::<&Transform, With<SpawnWarning>>()
            .iter(app.world())
            .map(|t| t.translation.to_array())
            .collect();
        points.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if let Some(p) = previous {
            assert_eq!(points, p);
        }
        previous = Some(points);
        step(&mut app, 1., &[KeyCode::KeyR]);
        assert_eq!(
            count::<Enemy>(&mut app) + count::<SpawnWarning>(&mut app),
            0
        );
    }
}

#[test]
fn terminal_update_accounts_due_requests_without_creating_warnings() {
    for survive in [true, false] {
        let (mut app, _) = wave_app();
        let at = if survive { 299. } else { 0.1 };
        app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(at, 2)];
        if survive {
            app.world_mut().resource_mut::<Encounter>().elapsed = 298.;
        } else {
            app.world_mut().resource_mut::<PlayerHealth>().current = 1;
            enemy(&mut app, START, 100);
        }
        step(&mut app, if survive { 2. } else { 0.1 }, &[]);
        let run = app.world().resource::<Encounter>();
        assert_eq!(run.spawns.requested, 2);
        assert_eq!(run.next_burst, 1);
        assert_eq!(run.spawns.admitted, 0);
        assert_eq!(run.spawns.skipped_terminal, 2);
        let accounted = run.spawns;
        assert_eq!(count::<SpawnWarning>(&mut app), 0);
        step(&mut app, 1., &[]);
        assert_eq!(app.world().resource::<Encounter>().spawns, accounted);
    }
}

#[test]
fn terminal_cancellation_counts_each_warning_once_without_intermediate_flushes() {
    use bevy::ecs::schedule::{ScheduleBuildSettings, SingleThreadedExecutor};
    for survive in [true, false] {
        let (mut app, _) = wave_app();
        step(&mut app, 3., &[]);
        assert_eq!(count::<SpawnWarning>(&mut app), 3);
        app.edit_schedule(Update, |schedule| {
            schedule.set_executor(SingleThreadedExecutor::new());
            schedule.set_build_settings(ScheduleBuildSettings {
                auto_insert_apply_deferred: false,
                ..default()
            });
        });
        if survive {
            app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
        } else {
            app.world_mut().resource_mut::<PlayerHealth>().current = 1;
            enemy(&mut app, START, 100);
        }
        step(&mut app, 0., &[]);
        assert_eq!(
            *app.world().resource::<GamePhase>(),
            if survive {
                GamePhase::Survived
            } else {
                GamePhase::Dead
            }
        );
        let counts = spawn_counts(&app);
        assert_eq!(counts.cancelled, 3);
        assert_eq!(counts.admitted, counts.activated + counts.cancelled);
        assert_eq!(count::<SpawnWarning>(&mut app), 0);
        step(&mut app, 0., &[]);
        assert_eq!(spawn_counts(&app), counts);
    }
}

#[test]
fn repeated_terminal_cleanup_before_despawn_counts_warnings_once() {
    for phase in [GamePhase::Dead, GamePhase::Survived] {
        let (mut app, _) = wave_app();
        step(&mut app, 3., &[]);
        assert_eq!(count::<SpawnWarning>(&mut app), 3);
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        // Exercise the two update registrations' shared deferred-despawn window.
        let mut cleanup = Schedule::default();
        cleanup.add_systems((waves::update, waves::update).chain_ignore_deferred());
        cleanup.run(app.world_mut());
        assert_eq!(spawn_counts(&app).cancelled, 3);
        assert_eq!(count::<SpawnWarning>(&mut app), 0);
        let accounted = spawn_counts(&app);
        cleanup.run(app.world_mut());
        assert_eq!(spawn_counts(&app), accounted);
    }
}
