use super::{Arena, DRONE_HALF_EXTENTS, Drone, move_drone, spawn_drone};
use bevy::{camera::ScalingMode, prelude::*};

pub struct ArenaScenePlugin;

#[derive(Component)]
struct GroundMarker;

impl Plugin for ArenaScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene.after(spawn_drone))
            .add_systems(Update, track_ground_position.after(move_drone));
    }
}

pub(super) fn setup_scene(
    mut commands: Commands,
    arena: Res<Arena>,
    drone: Single<Entity, With<Drone>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: arena.half_size.x * 2. + 160.,
                min_height: 800.,
            },
            far: 3000.,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0., 950., 1100.).looking_at(arena.center(), Vec3::Y),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 300.,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: 5000.,
            ..default()
        },
        Transform::from_xyz(-300., 800., 400.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let size = arena.half_size * 2.;
    let floor = materials.add(Color::srgb(0.055, 0.10, 0.13));
    let grid = materials.add(StandardMaterial {
        base_color: Color::srgb(0.085, 0.15, 0.18),
        unlit: true,
        ..default()
    });
    let boundary = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.44, 0.47),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(size.x, size.z))),
        MeshMaterial3d(floor),
    ));

    // Reuse a unit cube for the floor grid and open frame. Frame strips sit
    // outside the playable volume, so its inner surfaces match the clamp.
    let cube = meshes.add(Cuboid::default());
    for x in -8..=8 {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(grid.clone()),
            Transform::from_xyz(x as f32 * 60., 0.05, 0.).with_scale(Vec3::new(1., 0.1, size.z)),
        ));
    }
    for z in -4..=4 {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(grid.clone()),
            Transform::from_xyz(0., 0.05, z as f32 * 60.).with_scale(Vec3::new(size.x, 0.1, 1.)),
        ));
    }
    let wall = 2.;
    for y in [-wall / 2., size.y + wall / 2.] {
        for sign in [-1., 1.] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(boundary.clone()),
                Transform::from_xyz(0., y, sign * (arena.half_size.z + wall / 2.))
                    .with_scale(Vec3::new(size.x + wall * 2., wall, wall)),
            ));
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(boundary.clone()),
                Transform::from_xyz(sign * (arena.half_size.x + wall / 2.), y, 0.)
                    .with_scale(Vec3::new(wall, wall, size.z)),
            ));
        }
    }
    for x in [-1., 1.] {
        for z in [-1., 1.] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(boundary.clone()),
                Transform::from_xyz(
                    x * (arena.half_size.x + wall / 2.),
                    arena.center().y,
                    z * (arena.half_size.z + wall / 2.),
                )
                .with_scale(Vec3::new(wall, size.y, wall)),
            ));
        }
    }
    let home = materials.add(StandardMaterial {
        base_color: Color::srgb(0.31, 0.38, 0.31),
        unlit: true,
        ..default()
    });
    for scale in [Vec3::new(48., 0.1, 2.), Vec3::new(2., 0.1, 48.)] {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(home.clone()),
            Transform::from_xyz(0., 0.15, 0.).with_scale(scale),
        ));
    }

    let body = materials.add(Color::srgb(0.36, 0.93, 0.80));
    let rotor = materials.add(Color::srgb(0.76, 0.96, 0.93));
    let nose = materials.add(Color::srgb(0.04, 0.22, 0.22));
    let rotor_mesh = meshes.add(Cylinder::new(5., 3.));
    let arm_mesh = meshes.add(Cuboid::new(26., 2., 3.));
    commands
        .entity(*drone)
        .insert((
            Mesh3d(meshes.add(Cuboid::new(22., DRONE_HALF_EXTENTS.y * 2., 22.))),
            MeshMaterial3d(body.clone()),
        ))
        .with_children(|parent| {
            for z in [-1., 1.] {
                parent.spawn((
                    Mesh3d(arm_mesh.clone()),
                    MeshMaterial3d(body.clone()),
                    Transform::from_xyz(0., 0., z * (DRONE_HALF_EXTENTS.z - 5.)),
                ));
                for x in [-1., 1.] {
                    parent.spawn((
                        Mesh3d(rotor_mesh.clone()),
                        MeshMaterial3d(rotor.clone()),
                        Transform::from_xyz(
                            x * (DRONE_HALF_EXTENTS.x - 5.),
                            2.,
                            z * (DRONE_HALF_EXTENTS.z - 5.),
                        ),
                    ));
                }
            }
            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(4., 4., 4.))),
                MeshMaterial3d(nose),
                Transform::from_xyz(0., 2., -12.),
            ));
        });

    commands.spawn((
        GroundMarker,
        Mesh3d(meshes.add(Annulus::new(16., 18.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.30, 0.65, 0.60),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0., 0.3, 0.)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    commands.spawn((
        Text::new("DRONE SURVIVORS  /  COMBAT TEST ARENA"),
        TextFont::from_font_size(20.),
        TextColor(Color::srgb(0.76, 0.96, 0.93)),
        Node {
            position_type: PositionType::Absolute,
            top: px(20),
            left: px(24),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("WASD / Arrows  Move  |  Space  Up  |  Shift  Down\nAuto fire  |  R  Restart encounter  |  Esc  Quit"),
        TextFont::from_font_size(16.),
        TextColor(Color::srgb(0.63, 0.74, 0.77)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(20),
            left: px(24),
            right: px(24),
            ..default()
        },
    ));
}

fn track_ground_position(
    drone: Single<&Transform, With<Drone>>,
    mut marker: Single<&mut Transform, (With<GroundMarker>, Without<Drone>)>,
    mut gizmos: Gizmos,
) {
    marker.translation.x = drone.translation.x;
    marker.translation.z = drone.translation.z;
    gizmos.line(
        marker.translation,
        drone.translation - Vec3::Y * DRONE_HALF_EXTENTS.y,
        Color::srgb(0.18, 0.36, 0.35),
    );
}
