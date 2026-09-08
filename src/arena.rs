use crate::game::{GamePhase, GameplaySet, is_playing};
use bevy::prelude::*;

mod scene;
pub use scene::ArenaScenePlugin;

#[cfg(test)]
mod tests;

const DRONE_START: Transform = Transform::from_xyz(0., 90., 0.);
// Matches the enlarged scout GLB, including its wing-mounted rotors.
pub(crate) const DRONE_HALF_EXTENTS: Vec3 = Vec3::new(35., 15., 45.);

#[derive(Resource, Clone, Copy)]
pub(crate) struct Arena {
    pub(crate) half_size: Vec3,
    drone_speed: f32,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            half_size: Vec3::new(480., 150., 270.),
            drone_speed: 240.,
        }
    }
}

impl Arena {
    pub(crate) fn center(&self) -> Vec3 {
        // Center the flight volume above the ground plane at Y = 0.
        Vec3::Y * self.half_size.y
    }
}

#[derive(Component)]
pub(crate) struct Drone;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Arena>()
            .init_resource::<GamePhase>()
            .configure_sets(
                Update,
                (
                    GameplaySet::Reset,
                    GameplaySet::Movement,
                    GameplaySet::Combat,
                    GameplaySet::Presentation,
                )
                    .chain(),
            )
            .add_systems(Startup, spawn_drone)
            .add_systems(
                Update,
                move_drone.in_set(GameplaySet::Movement).run_if(is_playing),
            );
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
    let axis = |positive: &[KeyCode], negative: &[KeyCode]| {
        f32::from(keys.any_pressed(positive.iter().copied()))
            - f32::from(keys.any_pressed(negative.iter().copied()))
    };
    let direction = Vec3::new(
        axis(
            &[KeyCode::KeyD, KeyCode::ArrowRight],
            &[KeyCode::KeyA, KeyCode::ArrowLeft],
        ),
        axis(
            &[KeyCode::Space],
            &[KeyCode::ShiftLeft, KeyCode::ShiftRight],
        ),
        axis(
            &[KeyCode::KeyS, KeyCode::ArrowDown],
            &[KeyCode::KeyW, KeyCode::ArrowUp],
        ),
    )
    .normalize_or_zero();
    let limit = arena.half_size - DRONE_HALF_EXTENTS;
    for mut transform in &mut drones {
        // Reset wins over movement on this frame and never creates duplicate entities.
        if keys.just_pressed(KeyCode::KeyR) {
            *transform = DRONE_START;
            continue;
        }
        let position = transform.translation + direction * arena.drone_speed * time.delta_secs();
        // Clamp axes independently: contact stops travel into a surface, while
        // allowing movement along it or back into the flight volume.
        transform.translation = position.clamp(arena.center() - limit, arena.center() + limit);
    }
}
