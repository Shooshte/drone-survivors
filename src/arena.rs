use bevy::prelude::*;

mod scene;
pub use scene::ArenaScenePlugin;

#[cfg(test)]
mod tests;

const DRONE_START: Transform = Transform::from_xyz(0., 0., 2.);
const DRONE_HALF_SIZE: f32 = 18.;

#[derive(Resource, Clone, Copy)]
struct Arena {
    half_size: Vec2,
    drone_speed: f32,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            half_size: Vec2::new(480., 270.),
            drone_speed: 240.,
        }
    }
}

#[derive(Component)]
struct Drone;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Arena>()
            .add_systems(Startup, spawn_drone)
            .add_systems(Update, move_drone);
    }
}

fn spawn_drone(mut commands: Commands) {
    commands.spawn((Drone, DRONE_START));
}

fn move_drone(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    arena: Res<Arena>,
    mut drones: Query<&mut Transform, With<Drone>>,
) {
    let axis = |positive: [KeyCode; 2], negative: [KeyCode; 2]| {
        f32::from(keys.any_pressed(positive)) - f32::from(keys.any_pressed(negative))
    };
    let direction = Vec2::new(
        axis(
            [KeyCode::KeyD, KeyCode::ArrowRight],
            [KeyCode::KeyA, KeyCode::ArrowLeft],
        ),
        axis(
            [KeyCode::KeyW, KeyCode::ArrowUp],
            [KeyCode::KeyS, KeyCode::ArrowDown],
        ),
    )
    .normalize_or_zero();
    let limit = arena.half_size - Vec2::splat(DRONE_HALF_SIZE);
    for mut transform in &mut drones {
        // Reset wins over movement on this frame and never creates duplicate entities.
        if keys.just_pressed(KeyCode::KeyR) {
            *transform = DRONE_START;
            continue;
        }
        let position =
            transform.translation.truncate() + direction * arena.drone_speed * time.delta_secs();
        transform.translation = position
            .clamp(-limit, limit)
            .extend(transform.translation.z);
    }
}
