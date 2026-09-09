use crate::game::{GamePhase, GameplaySet, is_playing};
use bevy::prelude::*;

mod flight;

pub(crate) use flight::{
    DroneFlight, FlightConfig, FlightInput, drone_world_half_extents, world_half_extents,
};

mod scene;
pub use scene::ArenaScenePlugin;

#[cfg(test)]
mod tests;

pub(crate) const DRONE_START: Transform = Transform::from_xyz(0., 90., 0.);
// Matches the enlarged scout GLB, including its wing-mounted rotors.
pub(crate) const DRONE_HALF_EXTENTS: Vec3 = Vec3::new(35., 15., 45.);

#[derive(Resource, Clone, Copy)]
pub(crate) struct Arena {
    pub(crate) half_size: Vec3,
}

impl Default for Arena {
    fn default() -> Self {
        Self {
            half_size: Vec3::new(480., 150., 270.),
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
            .init_resource::<FlightConfig>()
            .init_resource::<GamePhase>()
            .configure_sets(
                Update,
                (
                    GameplaySet::Reset,
                    GameplaySet::ChoiceInput,
                    GameplaySet::Movement,
                    GameplaySet::Combat,
                    GameplaySet::Progression,
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
    commands.spawn((Drone, DRONE_START, DroneFlight::default()));
}

fn move_drone(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    arena: Res<Arena>,
    config: Res<FlightConfig>,
    mut drones: Query<(&mut Transform, &mut DroneFlight), With<Drone>>,
    modules: Option<Res<crate::modules::Modules>>,
    module_config: Option<Res<crate::modules::ModuleConfig>>,
) {
    let mut config = *config;
    if let (Some(modules), Some(tuning)) = (modules, module_config)
        && modules.active(crate::modules::ModuleKind::Mobility)
    {
        config.max_horizontal_speed *= tuning.mobility_multiplier;
        config.horizontal_acceleration_multiplier *= tuning.mobility_multiplier;
    }
    let input = FlightInput::read(&keys, &config);
    for (mut transform, mut flight) in &mut drones {
        // Reset wins over every held control and restores the model root too.
        if keys.just_pressed(KeyCode::KeyR) {
            *transform = DRONE_START;
            *flight = DroneFlight::default();
            continue;
        }
        flight.advance(&mut transform, &input, &config, &arena, time.delta_secs());
    }
}
