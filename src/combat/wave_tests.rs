use super::*;

fn wave_app() -> (App, Entity) {
    let (mut app, drone) = app();
    quiet(&mut app);
    (app, drone)
}

#[test]
fn warning_is_a_reservation_then_spawns_at_its_warned_position() {
    let (mut app, _) = wave_app();
    step(&mut app, 2.99, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    step(&mut app, 0.01, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 3);
    assert_eq!(count::<Enemy>(&mut app), 0);
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
}

#[test]
fn occupied_or_too_small_arena_skips_candidates_and_finishes_search() {
    let (mut app, _) = wave_app();
    app.world_mut()
        .resource_mut::<crate::arena::Arena>()
        .half_size = Vec3::splat(20.);
    step(&mut app, 3., &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
}

#[test]
fn hitch_only_warns_latest_due_burst_and_gives_full_warning_duration() {
    let (mut app, _) = wave_app();
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(1., 1), (2., 2), (3., 3)];
    step(&mut app, 4., &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 3);
    assert_eq!(count::<Enemy>(&mut app), 0);
    step(&mut app, 0.74, &[]);
    assert_eq!(count::<Enemy>(&mut app), 0);
    step(&mut app, 0.01, &[]);
    assert_eq!(count::<Enemy>(&mut app), 3);
}

#[test]
fn survival_freezes_clock_and_gameplay_cancels_warnings_and_resets_completely() {
    let (mut app, drone) = wave_app();
    app.world_mut().resource_mut::<WaveConfig>().bursts = vec![(179.5, 3), (180., 3)];
    step(&mut app, 179.5, &[]);
    assert_eq!(count::<SpawnWarning>(&mut app), 3);
    step(&mut app, 0.5, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
    assert_eq!(count::<SpawnWarning>(&mut app), 0);
    let before = position(&app, drone);
    step(&mut app, 5., &[KeyCode::Space]);
    assert_eq!(position(&app, drone), before);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 180.);
    step(&mut app, 1., &[KeyCode::KeyR, KeyCode::Space]);
    let run = app.world().resource::<Encounter>();
    assert_eq!(
        (run.elapsed, run.kills, run.next_burst, run.candidate),
        (0., 0, 0, 0)
    );
    assert_eq!(position(&app, drone), START);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}

#[test]
fn fatal_damage_wins_over_completion_and_restart_wins_over_both() {
    for reset in [false, true] {
        let (mut app, _) = wave_app();
        app.world_mut().resource_mut::<WaveConfig>().bursts.clear();
        app.world_mut().resource_mut::<Encounter>().elapsed = 179.9;
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
