use super::*;
use std::time::Duration;

const START: Vec3 = Vec3::new(0., 90., 0.);

fn test_app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins(ArenaPlugin);
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    (app, drone)
}

fn step(app: &mut App, keys: &[KeyCode], seconds: f32) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for &key in keys {
        input.press(key);
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

fn position(app: &App, drone: Entity) -> Vec3 {
    app.world().get::<Transform>(drone).unwrap().translation
}

fn near(actual: Vec3, expected: Vec3) {
    assert!(
        (actual - expected).length() < 0.002,
        "{actual:?} != {expected:?}"
    );
}

fn scene_app() -> App {
    let (mut app, _) = test_app();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .add_systems(Update, scene::setup_scene.run_if(run_once));
    app.update();
    app
}

#[test]
fn rendered_drone_geometry_fits_the_movement_bounds() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/models/scout_drone.glb"
    ))
    .expect("the scout GLB must ship with the arena");
    let gltf = bevy::gltf::gltf::Gltf::from_slice(&bytes).expect("valid glTF 2.0");
    let blob = gltf.blob.as_deref().expect("self-contained GLB buffer");
    let mut vertices = 0;
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut pending: Vec<_> = gltf
        .default_scene()
        .expect("default scout scene")
        .nodes()
        .map(|node| (node, Mat4::IDENTITY))
        .collect();
    while let Some((node, parent)) = pending.pop() {
        let transform = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        pending.extend(node.children().map(|child| (child, transform)));
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|_| Some(blob));
                let positions = reader.read_positions().expect("mesh vertices");
                for position in positions {
                    let point = transform.transform_point3(Vec3::from_array(position));
                    assert!(point.is_finite(), "non-finite mesh vertex");
                    min = min.min(point);
                    max = max.max(point);
                    vertices += 1;
                }
                assert!(reader.read_normals().is_some(), "export shading normals");
            }
        }
    }
    assert!(vertices > 0, "the exported scene must contain geometry");
    let tolerance = Vec3::splat(0.001);
    assert!(
        min.cmpge(-DRONE_HALF_EXTENTS - tolerance).all(),
        "mesh below bounds: {min:?}"
    );
    assert!(
        max.cmple(DRONE_HALF_EXTENTS + tolerance).all(),
        "mesh above bounds: {max:?}"
    );
    assert!(
        max.z - min.z > 75.,
        "scout should fill its flight footprint"
    );
    assert!(
        gltf.textures().next().is_none(),
        "materials need no external textures"
    );
}

#[test]
fn camera_keeps_the_whole_flight_volume_visible_after_resizing() {
    use bevy::camera::CameraProjection;
    let mut app = scene_app();
    let (transform, projection) = app
        .world_mut()
        .query_filtered::<(&Transform, &Projection), With<Camera3d>>()
        .single(app.world())
        .unwrap();
    let Projection::Orthographic(projection) = projection else {
        panic!("expected orthographic camera")
    };
    let view = transform.to_matrix().inverse();
    for (width, height) in [(1120., 720.), (640., 480.), (1600., 480.), (640., 1000.)] {
        let mut projection = projection.clone();
        projection.update(width, height);
        for x in [-482., 482.] {
            for y in [-2., 302.] {
                for z in [-272., 272.] {
                    let point = view.transform_point3(Vec3::new(x, y, z));
                    assert!(
                        projection.area.contains(point.truncate()),
                        "corner clipped at {width}x{height}: {point:?}"
                    );
                    assert!(-point.z >= projection.near && -point.z <= projection.far);
                }
            }
        }
    }
}

#[test]
fn scout_scene_loads_under_player_and_survives_encounter_restarts() {
    use bevy::{
        asset::LoadState, gltf::GltfPlugin, mesh::MeshPlugin,
        world_serialization::WorldSerializationPlugin,
    };
    let mut app = App::new();
    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        AssetPlugin {
            file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/assets").into(),
            ..default()
        },
        WorldSerializationPlugin,
        bevy::scene::ScenePlugin,
        TransformPlugin,
        MeshPlugin,
        ImagePlugin::default(),
        GltfPlugin::default(),
    ))
    .init_asset::<StandardMaterial>()
    .init_resource::<Time>()
    .init_resource::<ButtonInput<KeyCode>>()
    .add_plugins((ArenaPlugin, crate::combat::CombatPlugin))
    .add_systems(Startup, scene::setup_drone_model.after(spawn_drone));
    app.finish();
    app.cleanup();
    app.update();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    let (model, scene) = app
        .world_mut()
        .query::<(Entity, &WorldAssetRoot)>()
        .single(app.world())
        .map(|(entity, root)| (entity, root.0.clone()))
        .unwrap();
    assert_eq!(app.world().get::<ChildOf>(model).unwrap().parent(), drone);
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        app.update();
        if let Some(LoadState::Failed(error)) = app
            .world()
            .resource::<AssetServer>()
            .get_load_state(scene.id())
        {
            panic!("scout failed to load: {error}");
        }
        if app.world_mut().query::<&Mesh3d>().iter(app.world()).count() == 6 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "scout scene did not spawn"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let parts: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Mesh3d>>()
        .iter(app.world())
        .collect();
    let mesh_count = app.world().resource::<Assets<Mesh>>().len();
    let material_count = app.world().resource::<Assets<StandardMaterial>>().len();
    for part in &parts {
        let mut ancestor = *part;
        while ancestor != drone {
            ancestor = app
                .world()
                .get::<ChildOf>(ancestor)
                .expect("mesh belongs to player")
                .parent();
        }
    }
    for _ in 0..3 {
        step(&mut app, &[KeyCode::KeyD, KeyCode::Space], 0.1);
        step(&mut app, &[KeyCode::KeyR], 0.01);
        near(position(&app, drone), START);
        assert_eq!(app.world().get::<WorldAssetRoot>(model).unwrap().0, scene);
        for part in &parts {
            assert!(
                app.world().get::<Mesh3d>(*part).is_some(),
                "restart must retain model parts"
            );
        }
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(
            app.world().resource::<Assets<StandardMaterial>>().len(),
            material_count
        );
    }
}

#[test]
fn yaw_turns_in_place_and_preserves_heading() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyD], 0.75);
    near(position(&app, drone), START);
    let rotation = app.world().get::<Transform>(drone).unwrap().rotation;
    near(rotation * Vec3::NEG_Z, Vec3::X);
    step(&mut app, &[], 0.25);
    assert_eq!(
        app.world().get::<Transform>(drone).unwrap().rotation,
        rotation
    );
}

#[test]
fn bank_redirects_visible_thrust_sideways() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyE], 0.5);
    let transform = app.world().get::<Transform>(drone).unwrap();
    assert!(transform.translation.x > 1.);
    assert!((transform.rotation * Vec3::Y).x > 0.4);
}

#[test]
fn neutral_pitch_loses_altitude_and_keeps_drifting_after_leveling() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyW], 0.5);
    let tilted = position(&app, drone);
    assert!(tilted.y < START.y - 0.1);
    step(&mut app, &[], 0.5);
    let leveled = position(&app, drone);
    assert!(leveled.y < tilted.y);
    assert!(leveled.z < tilted.z);
    near(
        app.world().get::<Transform>(drone).unwrap().rotation * Vec3::Y,
        Vec3::Y,
    );
}

fn state(app: &App, drone: Entity) -> DroneFlight {
    *app.world().get::<DroneFlight>(drone).unwrap()
}

fn fly(app: &mut App, keys: &[KeyCode], seconds: f32, rate: u32) {
    for _ in 0..(seconds * rate as f32).round() as u32 {
        step(app, keys, 1. / rate as f32);
    }
}

#[test]
fn neutral_hover_and_all_bindings_follow_heading() {
    let (mut app, drone) = test_app();
    step(&mut app, &[], 2.);
    near(position(&app, drone), START);
    for heading in [0., std::f32::consts::FRAC_PI_2] {
        for (key, direction) in [
            (KeyCode::KeyW, Vec3::NEG_Z),
            (KeyCode::ArrowUp, Vec3::NEG_Z),
            (KeyCode::KeyS, Vec3::Z),
            (KeyCode::ArrowDown, Vec3::Z),
            (KeyCode::KeyQ, Vec3::NEG_X),
            (KeyCode::KeyE, Vec3::X),
            (KeyCode::Space, Vec3::Y),
            (KeyCode::ShiftLeft, Vec3::NEG_Y),
            (KeyCode::ShiftRight, Vec3::NEG_Y),
        ] {
            let (mut app, drone) = test_app();
            app.world_mut()
                .get_mut::<DroneFlight>(drone)
                .unwrap()
                .heading = heading;
            step(&mut app, &[key], 0.5);
            let velocity = state(&app, drone).velocity;
            let expected = Quat::from_rotation_y(heading) * direction;
            assert!(velocity.dot(expected) > 5., "{key:?}: {velocity:?}");
            if direction.y == 0. {
                let up = app.world().get::<Transform>(drone).unwrap().rotation * Vec3::Y;
                near(velocity.with_y(0.).normalize(), up.with_y(0.).normalize());
                assert!(velocity.y < 0.);
            }
        }
    }
    for (key, sign) in [
        (KeyCode::KeyA, 1.),
        (KeyCode::ArrowLeft, 1.),
        (KeyCode::KeyD, -1.),
        (KeyCode::ArrowRight, -1.),
    ] {
        let (mut app, drone) = test_app();
        step(&mut app, &[key], 0.25);
        assert!(
            (Quat::from_rotation_y(state(&app, drone).heading) * Vec3::NEG_Z)
                .distance(Quat::from_rotation_y(sign * std::f32::consts::FRAC_PI_6) * Vec3::NEG_Z)
                < 0.001
        );
    }
}

#[test]
fn aliases_do_not_stack_and_all_opposites_cancel() {
    for (single, aliases) in [
        (
            vec![KeyCode::KeyW, KeyCode::KeyA, KeyCode::ShiftLeft],
            vec![
                KeyCode::KeyW,
                KeyCode::ArrowUp,
                KeyCode::KeyA,
                KeyCode::ArrowLeft,
                KeyCode::ShiftLeft,
                KeyCode::ShiftRight,
            ],
        ),
        (
            vec![KeyCode::KeyS, KeyCode::KeyD],
            vec![
                KeyCode::KeyS,
                KeyCode::ArrowDown,
                KeyCode::KeyD,
                KeyCode::ArrowRight,
            ],
        ),
        (
            vec![],
            vec![
                KeyCode::KeyW,
                KeyCode::ArrowDown,
                KeyCode::KeyA,
                KeyCode::ArrowRight,
                KeyCode::KeyQ,
                KeyCode::KeyE,
                KeyCode::Space,
                KeyCode::ShiftRight,
            ],
        ),
    ] {
        let (mut a, da) = test_app();
        let (mut b, db) = test_app();
        step(&mut a, &single, 0.5);
        step(&mut b, &aliases, 0.5);
        near(position(&a, da), position(&b, db));
        assert_eq!(state(&a, da), state(&b, db));
    }
}

#[test]
fn tilt_progresses_to_shared_limit_and_axes_level_independently() {
    for held in [KeyCode::KeyW, KeyCode::KeyE] {
        let (mut app, drone) = test_app();
        step(&mut app, &[KeyCode::KeyW, KeyCode::KeyE], 0.1);
        let first = state(&app, drone).tilt.length();
        assert!(first > 0. && first < 30_f32.to_radians());
        step(&mut app, &[KeyCode::KeyW, KeyCode::KeyE], 1.);
        let both = state(&app, drone).tilt;
        assert!((both.length() - 30_f32.to_radians()).abs() < 0.001);
        let up = app.world().get::<Transform>(drone).unwrap().rotation * Vec3::Y;
        assert!((up.angle_between(Vec3::Y) - 30_f32.to_radians()).abs() < 0.001);
        step(&mut app, &[held], 0.05);
        let after = state(&app, drone).tilt;
        if held == KeyCode::KeyW {
            assert!(after.x < both.x && after.y > both.y);
        } else {
            assert!(after.y < both.y && after.x > both.x);
        }
        step(&mut app, &[held], 0.5);
        let after = state(&app, drone).tilt;
        if held == KeyCode::KeyW {
            assert_eq!(after.x, 0.);
        } else {
            assert_eq!(after.y, 0.);
        }
        step(&mut app, &[], 0.05);
        assert!(state(&app, drone).tilt.length() > 0.);
        step(&mut app, &[], 2.);
        assert_eq!(state(&app, drone).tilt, Vec2::ZERO);
    }
}

#[test]
fn yaw_preserves_world_drift_and_counter_tilt_brakes_faster() {
    let (mut coast, dc) = test_app();
    let (mut brake, db) = test_app();
    for (app, drone) in [(&mut coast, dc), (&mut brake, db)] {
        app.world_mut()
            .get_mut::<DroneFlight>(drone)
            .unwrap()
            .velocity = Vec3::NEG_Z * 100.;
    }
    step(&mut coast, &[KeyCode::KeyD], 0.75);
    let drift = state(&coast, dc).velocity;
    assert!(drift.z < -70. && drift.x.abs() < 0.001);
    step(&mut brake, &[KeyCode::KeyS], 0.75);
    assert!(state(&brake, db).velocity.z > drift.z + 10.);
}

#[test]
fn boost_arrests_descent_and_thrust_is_not_latched() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyW], 1.);
    step(&mut app, &[], 0.5);
    assert!(state(&app, drone).velocity.y < -5.);
    step(&mut app, &[KeyCode::Space], 0.5);
    assert!(state(&app, drone).velocity.y > 0.);
    let up = state(&app, drone).velocity.y;
    step(&mut app, &[], 0.1);
    assert!(state(&app, drone).velocity.y < up && state(&app, drone).velocity.y > 0.);
    step(&mut app, &[KeyCode::ShiftLeft], 0.1);
    let reduced = state(&app, drone).velocity.y;
    step(&mut app, &[], 0.1);
    assert!((state(&app, drone).velocity.y - reduced * (-0.05_f32).exp()).abs() < 0.001);
}

#[test]
fn comparable_trajectories_at_common_frame_rates_and_long_frame() {
    let mut results = vec![];
    for rate in [30, 60, 120, 144] {
        let (mut app, drone) = test_app();
        fly(
            &mut app,
            &[KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyD, KeyCode::Space],
            1.,
            rate,
        );
        fly(&mut app, &[], 0.5, rate);
        results.push((position(&app, drone), state(&app, drone)));
    }
    for (pos, flight) in &results {
        assert!(pos.distance(results[0].0) < 0.4, "{results:?}");
        assert!(flight.velocity.distance(results[0].1.velocity) < 0.4);
    }
    let (mut app, drone) = test_app();
    step(
        &mut app,
        &[KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyD, KeyCode::Space],
        1.,
    );
    step(&mut app, &[], 0.5);
    assert!(position(&app, drone).distance(results[0].0) < 0.05);
    let (mut app, drone) = test_app();
    app.world_mut()
        .get_mut::<DroneFlight>(drone)
        .unwrap()
        .velocity = Vec3::new(1., 1., -1.);
    step(&mut app, &[], 10.);
    let v = state(&app, drone).velocity;
    assert!(v.x > 0. && v.x < 0.1 && v.z < 0. && v.y > 0. && v.y < 0.01);
}

#[test]
fn horizontal_speed_limit_preserves_braking_and_steering() {
    for keys in [
        vec![KeyCode::KeyW, KeyCode::Space],
        vec![KeyCode::KeyS, KeyCode::Space],
        vec![KeyCode::KeyQ, KeyCode::Space],
        vec![KeyCode::KeyE, KeyCode::Space],
        vec![KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyD, KeyCode::Space],
    ] {
        let (mut app, drone) = test_app();
        app.world_mut().resource_mut::<Arena>().half_size = Vec3::splat(1_000_000.);
        for _ in 0..600 {
            step(&mut app, &keys, 1. / 30.);
            assert!(state(&app, drone).velocity.with_y(0.).length() <= 240.001);
        }
    }
    for (key, steering) in [(KeyCode::KeyS, false), (KeyCode::KeyE, true)] {
        let (mut app, drone) = test_app();
        app.world_mut()
            .get_mut::<DroneFlight>(drone)
            .unwrap()
            .velocity = Vec3::NEG_Z * 240.;
        step(&mut app, &[key, KeyCode::Space], 0.25);
        let v = state(&app, drone).velocity;
        assert!(v.with_y(0.).length() <= 240.001);
        if steering {
            assert!(v.x > 5.);
        } else {
            assert!(v.z > -220.);
        }
    }
}

#[test]
fn rotated_bounds_contain_all_eight_corners() {
    for heading in [0., 0.7, 1.57] {
        for tilt in [Vec2::ZERO, Vec2::new(0.2, 0.4), Vec2::new(-0.4, 0.2)] {
            let rotation = Quat::from_rotation_y(heading)
                * Quat::from_scaled_axis(Vec3::new(-tilt.y, 0., -tilt.x));
            let half = drone_world_half_extents(rotation);
            for x in [-1., 1.] {
                for y in [-1., 1.] {
                    for z in [-1., 1.] {
                        let point = rotation * (DRONE_HALF_EXTENTS * Vec3::new(x, y, z));
                        assert!(point.abs().cmple(half + Vec3::splat(0.001)).all());
                    }
                }
            }
        }
    }
}

#[test]
fn every_rotated_boundary_removes_only_velocity_into_contact() {
    for axis in 0..3 {
        for sign in [-1., 1.] {
            for heading in [0., 0.8, 1.57] {
                for departing in [false, true] {
                    let (mut app, drone) = test_app();
                    let mut p = Vec3::new(0., 150., 0.);
                    p[axis] = if axis == 1 { 150. } else { 0. } + sign * 1000.;
                    app.world_mut()
                        .get_mut::<Transform>(drone)
                        .unwrap()
                        .translation = p;
                    let mut velocity = Vec3::splat(10.);
                    velocity[axis] = sign * if departing { -10. } else { 10. };
                    *app.world_mut().get_mut::<DroneFlight>(drone).unwrap() = DroneFlight {
                        heading,
                        tilt: Vec2::new(0.2, 0.3),
                        velocity,
                    };
                    step(&mut app, &[], 0.001);
                    let transform = app.world().get::<Transform>(drone).unwrap();
                    let half = drone_world_half_extents(transform.rotation);
                    let arena = app.world().resource::<Arena>();
                    assert!(
                        (transform.translation - arena.center())
                            .abs()
                            .cmple(arena.half_size - half + Vec3::splat(0.001))
                            .all()
                    );
                    let v = state(&app, drone).velocity;
                    if departing {
                        assert!(v[axis] * sign < 0.);
                    } else {
                        assert_eq!(v[axis], 0.);
                    }
                    assert!(v[(axis + 1) % 3] > 9.);
                }
            }
        }
    }
}

#[test]
fn floor_takeoff_ceiling_departure_and_rotation_at_wall_work() {
    for (key, y, sign) in [(KeyCode::Space, 15., 1.), (KeyCode::ShiftRight, 285., -1.)] {
        let (mut app, drone) = test_app();
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation
            .y = y;
        step(&mut app, &[key], 0.25);
        assert!((position(&app, drone).y - y) * sign > 1.);
    }
    let (mut app, drone) = test_app();
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation
        .x = 445.;
    step(&mut app, &[KeyCode::KeyD], 0.375);
    assert!(position(&app, drone).x < 430.);
    near(state(&app, drone).velocity, Vec3::ZERO);
    step(&mut app, &[KeyCode::KeyQ], 0.25);
    assert!(state(&app, drone).velocity.x < 0.);
}

#[test]
fn reset_clears_all_flight_state_wins_over_input_and_held_reset_does_not_repeat() {
    let (mut app, drone) = test_app();
    for _ in 0..3 {
        step(
            &mut app,
            &[KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyD, KeyCode::Space],
            0.5,
        );
        app.world_mut().get_mut::<Transform>(drone).unwrap().scale = Vec3::splat(2.);
        step(
            &mut app,
            &[KeyCode::KeyR, KeyCode::Space, KeyCode::KeyW],
            1.,
        );
        assert_eq!(*app.world().get::<Transform>(drone).unwrap(), DRONE_START);
        assert_eq!(state(&app, drone), DroneFlight::default());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_ne!(state(&app, drone), DroneFlight::default());
    }
}

#[test]
fn diagonal_tilt_and_leveling_use_configured_total_angular_rates() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyW, KeyCode::KeyE], 0.1);
    assert!((state(&app, drone).tilt.length() - 9_f32.to_radians()).abs() < 0.001);
    step(&mut app, &[KeyCode::KeyW, KeyCode::KeyE], 0.5);
    step(&mut app, &[], 0.05);
    assert!((state(&app, drone).tilt.length() - 24_f32.to_radians()).abs() < 0.001);
}

#[test]
fn speed_clamp_is_horizontal_only_and_opposing_tilt_levels_each_axis() {
    let (mut app, drone) = test_app();
    app.world_mut().resource_mut::<Arena>().half_size = Vec3::splat(1_000_000.);
    app.world_mut()
        .get_mut::<DroneFlight>(drone)
        .unwrap()
        .velocity = Vec3::new(500., 500., 500.);
    step(&mut app, &[], 0.001);
    let velocity = state(&app, drone).velocity;
    assert!(velocity.with_y(0.).length() <= 240.001);
    assert!(velocity.y > 499.);
    step(&mut app, &[KeyCode::KeyW, KeyCode::KeyE], 0.5);
    step(
        &mut app,
        &[KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyE],
        0.5,
    );
    assert_eq!(state(&app, drone).tilt.y, 0.);
    assert!(state(&app, drone).tilt.x > 0.5);
    step(
        &mut app,
        &[KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyQ],
        0.5,
    );
    assert_eq!(state(&app, drone).tilt.x, 0.);
    assert!(state(&app, drone).tilt.y > 0.5);
}

#[test]
fn sustained_flight_contacts_all_faces_edges_and_corners_without_velocity_buildup() {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }
                let inputs = |reverse: bool| {
                    let sign = if reverse { -1 } else { 1 };
                    [
                        (x * sign, KeyCode::KeyE, KeyCode::KeyQ),
                        (y * sign, KeyCode::Space, KeyCode::ShiftLeft),
                        (z * sign, KeyCode::KeyS, KeyCode::KeyW),
                    ]
                    .into_iter()
                    .filter_map(|(direction, positive, negative)| match direction {
                        1 => Some(positive),
                        -1 => Some(negative),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                };
                let (mut app, drone) = test_app();
                for seconds in [10., 2.] {
                    step(&mut app, &inputs(false), seconds);
                    let transform = app.world().get::<Transform>(drone).unwrap();
                    let half = drone_world_half_extents(transform.rotation);
                    let arena = app.world().resource::<Arena>();
                    let relative = transform.translation - arena.center();
                    assert!(
                        relative
                            .abs()
                            .cmple(arena.half_size - half + Vec3::splat(0.001))
                            .all()
                    );
                    for (axis, sign) in [x, y, z].into_iter().enumerate() {
                        if sign != 0 {
                            assert!(
                                (relative[axis]
                                    - sign as f32 * (arena.half_size[axis] - half[axis]))
                                    .abs()
                                    < 0.001,
                                "{x}, {y}, {z}: {relative:?}"
                            );
                            assert_eq!(state(&app, drone).velocity[axis], 0.);
                        }
                    }
                }
                let before = position(&app, drone);
                step(&mut app, &inputs(true), 1.);
                let delta = position(&app, drone) - before;
                for (axis, sign) in [x, y, z].into_iter().enumerate() {
                    if sign != 0 {
                        assert!(
                            delta[axis] * (sign as f32) < -0.1,
                            "departure {x}, {y}, {z}: {delta:?}"
                        );
                    }
                }
            }
        }
    }
}
