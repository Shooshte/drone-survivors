use super::*;
use std::time::Duration;

const START: Vec3 = Vec3::new(0., 90., 0.);
const MIN: Vec3 = Vec3::new(-462., 6., -252.);
const MAX: Vec3 = Vec3::new(462., 294., 252.);

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

fn directional_keys(x: i32, y: i32, z: i32) -> Vec<KeyCode> {
    [
        (x, KeyCode::KeyD, KeyCode::KeyA),
        (y, KeyCode::Space, KeyCode::ShiftLeft),
        (z, KeyCode::KeyS, KeyCode::KeyW),
    ]
    .into_iter()
    .filter_map(|(axis, positive, negative)| match axis {
        1 => Some(positive),
        -1 => Some(negative),
        _ => None,
    })
    .collect()
}

#[test]
fn drone_starts_hovering_above_the_ground() {
    let (mut app, drone) = test_app();
    near(position(&app, drone), START);
    step(&mut app, &[], 1.);
    near(position(&app, drone), START);
}

#[test]
fn all_six_directions_and_keyboard_aliases_work() {
    for (key, direction) in [
        (KeyCode::KeyW, Vec3::NEG_Z),
        (KeyCode::KeyS, Vec3::Z),
        (KeyCode::KeyA, Vec3::NEG_X),
        (KeyCode::KeyD, Vec3::X),
        (KeyCode::ArrowUp, Vec3::NEG_Z),
        (KeyCode::ArrowDown, Vec3::Z),
        (KeyCode::ArrowLeft, Vec3::NEG_X),
        (KeyCode::ArrowRight, Vec3::X),
        (KeyCode::Space, Vec3::Y),
        (KeyCode::ShiftLeft, Vec3::NEG_Y),
        (KeyCode::ShiftRight, Vec3::NEG_Y),
    ] {
        let (mut app, drone) = test_app();
        step(&mut app, &[key], 0.25);
        near(position(&app, drone), START + direction * 60.);
    }
}

#[test]
fn all_two_and_three_axis_diagonals_have_the_same_speed() {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let direction = Vec3::new(x as f32, y as f32, z as f32);
                if direction.length_squared() < 2. {
                    continue;
                }
                let (mut app, drone) = test_app();
                step(&mut app, &directional_keys(x, y, z), 0.25);
                near(position(&app, drone), START + direction.normalize() * 60.);
            }
        }
    }
}

#[test]
fn duplicate_bindings_do_not_skew_diagonal_direction() {
    let (mut app, drone) = test_app();
    step(
        &mut app,
        &[
            KeyCode::KeyW,
            KeyCode::ArrowUp,
            KeyCode::KeyD,
            KeyCode::ArrowRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ],
        0.25,
    );
    near(
        position(&app, drone),
        START + Vec3::new(1., -1., -1.).normalize() * 60.,
    );
}

#[test]
fn released_and_opposing_keys_do_not_move_the_drone() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyD, KeyCode::Space], 0.25);
    let before = position(&app, drone);
    step(&mut app, &[], 0.5);
    near(position(&app, drone), before);
    step(
        &mut app,
        &[
            KeyCode::KeyW,
            KeyCode::ArrowDown,
            KeyCode::KeyA,
            KeyCode::ArrowRight,
            KeyCode::Space,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ],
        0.5,
    );
    near(position(&app, drone), before);
}

#[test]
fn movement_depends_on_elapsed_time_not_frame_count() {
    let (mut slow, slow_drone) = test_app();
    let (mut fast, fast_drone) = test_app();
    let keys = [KeyCode::KeyD, KeyCode::Space, KeyCode::KeyW];
    for _ in 0..30 {
        step(&mut slow, &keys, 1. / 30.);
    }
    for _ in 0..120 {
        step(&mut fast, &keys, 1. / 120.);
    }
    near(position(&slow, slow_drone), position(&fast, fast_drone));
    near(
        position(&slow, slow_drone),
        START + Vec3::new(1., 1., -1.).normalize() * 240.,
    );
}

#[test]
fn entire_drone_stays_inside_all_faces_edges_and_corners_after_long_frames() {
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }
                let (mut app, drone) = test_app();
                let keys = directional_keys(x, y, z);
                let component = |axis: i32, min, start, max| match axis {
                    -1 => min,
                    1 => max,
                    _ => start,
                };
                let expected = Vec3::new(
                    component(x, MIN.x, START.x, MAX.x),
                    component(y, MIN.y, START.y, MAX.y),
                    component(z, MIN.z, START.z, MAX.z),
                );
                for _ in 0..3 {
                    step(&mut app, &keys, 10.);
                    near(position(&app, drone), expected);
                }
            }
        }
    }
}

#[test]
fn ground_and_ceiling_contact_allow_sliding_and_immediate_departure() {
    for (into, away, height, direction) in [
        (KeyCode::ShiftLeft, KeyCode::Space, MIN.y, 1.),
        (KeyCode::Space, KeyCode::ShiftRight, MAX.y, -1.),
    ] {
        let (mut app, drone) = test_app();
        step(&mut app, &[into], 10.);
        near(position(&app, drone), Vec3::new(0., height, 0.));
        step(&mut app, &[into, KeyCode::KeyD], 0.25);
        let slid = position(&app, drone);
        near(slid, Vec3::new(60. / 2_f32.sqrt(), height, 0.));
        step(&mut app, &[away], 0.25);
        near(position(&app, drone), slid + Vec3::Y * direction * 60.);
        step(&mut app, &[], 1.);
        near(position(&app, drone), slid + Vec3::Y * direction * 60.);
    }
}

#[test]
fn side_wall_contact_allows_vertical_movement_and_departure() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyD], 10.);
    step(&mut app, &[KeyCode::KeyD, KeyCode::Space], 0.25);
    let slid = Vec3::new(MAX.x, START.y + 60. / 2_f32.sqrt(), 0.);
    near(position(&app, drone), slid);
    step(&mut app, &[KeyCode::KeyA], 0.25);
    near(position(&app, drone), slid - Vec3::X * 60.);
}

#[test]
fn reset_restores_starting_transform_and_wins_over_movement_repeatedly() {
    let (mut app, drone) = test_app();
    let start = *app.world().get::<Transform>(drone).unwrap();
    let count = app.world().entities().len();
    for _ in 0..3 {
        step(
            &mut app,
            &[KeyCode::KeyD, KeyCode::KeyW, KeyCode::Space],
            1.,
        );
        assert_ne!(position(&app, drone), start.translation);
        // Reset must restore orientation and scale as well as altitude.
        let mut transform = app.world_mut().get_mut::<Transform>(drone).unwrap();
        transform.rotation = Quat::from_rotation_y(0.8);
        transform.scale = Vec3::splat(2.);
        step(
            &mut app,
            &[KeyCode::KeyR, KeyCode::KeyD, KeyCode::ShiftLeft],
            0.5,
        );
        assert_eq!(*app.world().get::<Transform>(drone).unwrap(), start);
        near(position(&app, drone), START);
        assert_eq!(app.world().entities().len(), count);
    }
}

#[test]
fn holding_reset_does_not_reset_again_until_repressed() {
    let (mut app, drone) = test_app();
    step(&mut app, &[KeyCode::KeyR, KeyCode::Space], 0.25);
    near(position(&app, drone), START);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.update();
    near(position(&app, drone), START + Vec3::Y * 60.);
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
        max.z - min.z > 30.,
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
