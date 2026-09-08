use super::{Arena, DRONE_HALF_SIZE, Drone, spawn_drone};
use bevy::{camera::ScalingMode, prelude::*};

pub struct ArenaScenePlugin;

impl Plugin for ArenaScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene.after(spawn_drone));
    }
}

fn setup_scene(mut commands: Commands, arena: Res<Arena>, drone: Single<Entity, With<Drone>>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: arena.half_size.x * 2. + 160.,
                min_height: arena.half_size.y * 2. + 180.,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let size = arena.half_size * 2.;
    let floor = Color::srgb(0.055, 0.10, 0.13);
    let grid = Color::srgb(0.085, 0.15, 0.18);
    let boundary = Color::srgb(0.30, 0.56, 0.59);
    commands.spawn((
        Sprite::from_color(floor, size),
        Transform::from_xyz(0., 0., -2.),
    ));

    // The grid gives motion a stable visual reference without external assets.
    for x in -8..=8 {
        commands.spawn((
            Sprite::from_color(grid, Vec2::new(1., size.y)),
            Transform::from_xyz(x as f32 * 60., 0., -1.),
        ));
    }
    for y in -4..=4 {
        commands.spawn((
            Sprite::from_color(grid, Vec2::new(size.x, 1.)),
            Transform::from_xyz(0., y as f32 * 60., -1.),
        ));
    }

    // Boundary strips sit outside the playable rectangle; the clamp uses its inner edge.
    let wall = 4.;
    for sign in [-1., 1.] {
        commands.spawn((
            Sprite::from_color(boundary, Vec2::new(size.x + wall * 2., wall)),
            Transform::from_xyz(0., sign * (arena.half_size.y + wall / 2.), 0.),
        ));
        commands.spawn((
            Sprite::from_color(boundary, Vec2::new(wall, size.y)),
            Transform::from_xyz(sign * (arena.half_size.x + wall / 2.), 0., 0.),
        ));
    }
    for size in [Vec2::new(48., 2.), Vec2::new(2., 48.)] {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.31, 0.38, 0.31), size),
            Transform::from_xyz(0., 0., 0.5),
        ));
    }

    commands
        .entity(*drone)
        .insert(Sprite::from_color(
            Color::srgb(0.36, 0.93, 0.80),
            Vec2::splat(22.),
        ))
        .with_children(|parent| {
            for x in [-1., 1.] {
                for y in [-1., 1.] {
                    parent.spawn((
                        Sprite::from_color(Color::srgb(0.76, 0.96, 0.93), Vec2::splat(10.)),
                        Transform::from_xyz(
                            x * (DRONE_HALF_SIZE - 5.),
                            y * (DRONE_HALF_SIZE - 5.),
                            0.1,
                        ),
                    ));
                }
            }
            parent.spawn((
                Sprite::from_color(Color::srgb(0.04, 0.22, 0.22), Vec2::new(4., 8.)),
                Transform::from_xyz(0., 5., 0.2),
            ));
        });

    commands.spawn((
        Text::new("DRONE SURVIVORS  /  TEST ARENA"),
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
        Text::new("WASD / Arrows  Move    |    R  Reset    |    Esc  Quit"),
        TextFont::from_font_size(16.),
        TextColor(Color::srgb(0.63, 0.74, 0.77)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(20),
            left: px(24),
            ..default()
        },
    ));
}
