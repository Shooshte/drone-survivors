use super::*;
use crate::{
    arena::Drone,
    combat::{CombatConfig, PlayerHealth},
    game::{is_playing, reset_requested},
    world::{
        PlayerPath, WorldGeometry,
        environment::{Environment, REPAIR_CENTER, REPAIR_RADIUS},
        proximity_contact,
    },
};

pub(super) fn install(app: &mut App) {
    app.init_resource::<Environment>()
        .add_systems(
            Update,
            reset.in_set(GameplaySet::Reset).run_if(reset_requested),
        )
        .add_systems(
            Update,
            (tick, repair)
                .chain()
                .in_set(GameplaySet::Collection)
                .run_if(is_playing),
        );
}
fn reset(arena: Res<CatalogArena>, mut environment: ResMut<Environment>) {
    environment.reset(!arena.selecting && arena.scenario >= 12);
}
fn tick(time: Res<Time>, mut environment: ResMut<Environment>) {
    if environment.enabled() {
        environment.elapsed += time.delta_secs_f64();
    }
}
fn repair(
    mut environment: ResMut<Environment>,
    mut health: ResMut<PlayerHealth>,
    config: Res<CombatConfig>,
    drone: Single<&Transform, With<Drone>>,
    path: Option<Res<PlayerPath>>,
    world: Option<Res<WorldGeometry>>,
) {
    if !environment.enabled() || !environment.repair_ready {
        return;
    }
    let contact = |start, end| {
        proximity_contact(start, end, REPAIR_CENTER, REPAIR_RADIUS, world.as_deref()).is_some()
    };
    let visited = path
        .as_ref()
        .is_some_and(|p| p.segments.iter().any(|s| contact(s.start, s.end)))
        || contact(drone.translation, drone.translation);
    if visited {
        environment.repair(&mut health.current, config.player_health);
    }
}
