use super::*;

#[test]
fn simultaneous_lethal_hits_count_one_kill() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    enemy(&mut app, START + Vec3::X * 80., 10);
    for _ in 0..2 {
        shot(&mut app, START, Vec3::X * 650., 1.);
    }
    step(&mut app, 0.2, &[]);
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
    step(&mut app, 0.1, &[]);
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
}

#[test]
fn coincident_chasers_separate_gradually_using_physical_flight() {
    let (mut app, _) = empty_app();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    let from = START + Vec3::X * 200.;
    let a = enemy(&mut app, from, 100);
    let b = enemy(&mut app, from, 100);
    step(&mut app, 1. / 120., &[]);
    assert!(position(&app, a).distance(from) < 1.);
    assert!(position(&app, b).distance(from) < 1.);
    for _ in 0..119 {
        step(&mut app, 1. / 120., &[]);
    }
    assert!(position(&app, a).distance(position(&app, b)) > 15.);
    for id in [a, b] {
        assert!(position(&app, id).distance(START) < from.distance(START));
        let flight = app.world().get::<crate::arena::DroneFlight>(id).unwrap();
        assert!(flight.tilt.length() <= 20_f32.to_radians() + 0.001);
    }
}

fn scene_app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_plugins((crate::arena::ArenaPlugin, CombatPlugin, CombatScenePlugin));
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    quiet(&mut app);
    (app, drone)
}

#[test]
fn hit_flash_is_local_expires_and_terminal_events_do_not_replay() {
    use super::super::feedback::{DamageCue, HitFlash, KillEffect};
    let (mut app, _) = scene_app();
    let a = enemy(&mut app, START + Vec3::X * 80., 100);
    let b = enemy(&mut app, START - Vec3::X * 100., 100);
    step(&mut app, 0., &[]);
    let normal = app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(b)
        .unwrap()
        .0
        .clone();
    shot(&mut app, START, Vec3::X * 650., 1.);
    step(&mut app, 0.2, &[]);
    assert!(app.world().get::<HitFlash>(a).is_some());
    assert_ne!(
        app.world()
            .get::<MeshMaterial3d<StandardMaterial>>(a)
            .unwrap()
            .0,
        normal
    );
    assert_eq!(
        app.world()
            .get::<MeshMaterial3d<StandardMaterial>>(b)
            .unwrap()
            .0,
        normal
    );
    step(&mut app, 0.13, &[]);
    assert!(app.world().get::<HitFlash>(a).is_none());
    assert_eq!(
        app.world()
            .get::<MeshMaterial3d<StandardMaterial>>(a)
            .unwrap()
            .0,
        normal
    );
    enemy(&mut app, START, 100);
    step(&mut app, 0., &[]);
    let initial = app.world().resource::<DamageCue>().0;
    assert!(initial > 0.);
    step(&mut app, 0.1, &[]);
    assert!(app.world().resource::<DamageCue>().0 < initial);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    step(&mut app, 1., &[]);
    assert_eq!(app.world().resource::<DamageCue>().0, 0.);
    assert_eq!(count::<KillEffect>(&mut app), 0);
}

#[test]
fn effect_cap_does_not_drop_kills_and_restart_reuses_assets() {
    use super::super::feedback::{DamageCue, FeedbackConfig, HitFlash, KillEffect};
    let (mut app, _) = scene_app();
    app.world_mut().resource_mut::<FeedbackConfig>().effect_cap = 1;
    for direction in [Vec3::X, Vec3::NEG_X, Vec3::Z] {
        enemy(&mut app, START + direction * 100., 10);
        shot(&mut app, START, direction * 650., 1.);
    }
    step(&mut app, 0.2, &[]);
    assert_eq!(app.world().resource::<Encounter>().kills, 3);
    assert_eq!(count::<KillEffect>(&mut app), 1);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    step(&mut app, 1., &[]);
    assert_eq!(count::<KillEffect>(&mut app), 0);
    let assets = (
        app.world().resource::<Assets<Mesh>>().len(),
        app.world().resource::<Assets<StandardMaterial>>().len(),
    );
    for _ in 0..3 {
        step(&mut app, 0., &[KeyCode::KeyR]);
        step(&mut app, 3., &[]);
        assert_eq!(count::<SpawnWarning>(&mut app), 5);
        for entity in app
            .world_mut()
            .query_filtered::<Entity, With<SpawnWarning>>()
            .iter(app.world())
        {
            assert!(app.world().get::<Mesh3d>(entity).is_some());
        }
        step(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(
            count::<KillEffect>(&mut app)
                + count::<HitFlash>(&mut app)
                + count::<SpawnWarning>(&mut app),
            0
        );
        assert_eq!(app.world().resource::<DamageCue>().0, 0.);
        assert_eq!(
            assets,
            (
                app.world().resource::<Assets<Mesh>>().len(),
                app.world().resource::<Assets<StandardMaterial>>().len()
            )
        );
    }
}

#[test]
fn hud_uses_authored_non_minute_lulls_and_phase_boundaries() {
    use super::super::scene::CombatHud;
    let (mut app, _) = scene_app();
    enemy(&mut app, START + Vec3::X * 200., 100);
    app.world_mut().resource_mut::<Encounter>().elapsed = 29.;
    step(&mut app, 0., &[]);
    let mut text = app
        .world_mut()
        .query_filtered::<&Text, With<CombatHud>>()
        .single(app.world())
        .unwrap();
    assert!(text.0.contains("LULL"), "{}", text.0);
    app.world_mut().resource_mut::<Encounter>().elapsed = 30.;
    step(&mut app, 0., &[]);
    text = app
        .world_mut()
        .query_filtered::<&Text, With<CombatHud>>()
        .single(app.world())
        .unwrap();
    assert!(text.0.contains("PRESSURE I"), "{}", text.0);
    app.world_mut().resource_mut::<Encounter>().elapsed = 104.;
    step(&mut app, 0., &[]);
    text = app
        .world_mut()
        .query_filtered::<&Text, With<CombatHud>>()
        .single(app.world())
        .unwrap();
    assert!(text.0.contains("LULL"), "{}", text.0);
    app.world_mut().resource_mut::<Encounter>().elapsed = 105.;
    step(&mut app, 0., &[]);
    text = app
        .world_mut()
        .query_filtered::<&Text, With<CombatHud>>()
        .single(app.world())
        .unwrap();
    assert!(text.0.contains("PRESSURE II"), "{}", text.0);
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    step(&mut app, 0., &[]);
    text = app
        .world_mut()
        .query_filtered::<&Text, With<CombatHud>>()
        .single(app.world())
        .unwrap();
    assert!(text.0.contains("SURVIVED") && text.0.contains("R") && !text.0.contains("ARENA CLEAR"));
}

#[test]
fn rocket_visuals_preserve_flight_and_explosions_reuse_assets_on_restart() {
    use super::super::feedback::{ExplosionRadius, KillEffect};
    let (mut app, _) = scene_app();
    let meshes = app.world().resource::<Assets<Mesh>>().len();
    let materials = app.world().resource::<Assets<StandardMaterial>>().len();
    for _ in 0..3 {
        enemy(&mut app, START + Vec3::X * 100., 100);
        let rocket = shot(&mut app, START, Vec3::X * 500., 1.5);
        app.world_mut().entity_mut(rocket).insert(rockets::Rocket);
        step(&mut app, 0., &[]);
        assert!(app.world().get::<Mesh3d>(rocket).is_some());
        assert_eq!(position(&app, rocket), START);
        assert!(
            (app.world().get::<Transform>(rocket).unwrap().rotation * Vec3::NEG_Z)
                .distance(Vec3::X)
                < 0.001
        );
        step(&mut app, 0.2, &[]);
        assert!(app.world().get_entity(rocket).is_err());
        assert_eq!(count::<ExplosionRadius>(&mut app), 1);
        let (radius, transform) = app
            .world_mut()
            .query::<(&ExplosionRadius, &Transform)>()
            .single(app.world())
            .unwrap();
        assert_eq!(radius.0, 70.);
        assert_eq!(transform.scale, Vec3::splat(70.));
        step(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(count::<KillEffect>(&mut app), 0);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
        assert_eq!(
            app.world().resource::<Assets<StandardMaterial>>().len(),
            materials
        );
    }
}
