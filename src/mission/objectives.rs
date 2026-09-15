//! Attempt-local objective rules. Completion still crosses the common mission boundary.
use super::{MissionSession, campaign::MissionId};
use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
    world::WorldGeometry,
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ObjectiveKind {
    #[default]
    Survival,
    Reconnaissance,
    Extraction,
}
impl MissionId {
    pub(crate) fn objective(self) -> ObjectiveKind {
        match self.index() {
            1 => ObjectiveKind::Reconnaissance,
            2 => ObjectiveKind::Extraction,
            _ => ObjectiveKind::Survival,
        }
    }
}
#[derive(Resource, Clone)]
pub(crate) struct ObjectiveConfig {
    pub sites: [Vec3; 3],
    pub extraction: Vec3,
    pub visit_radius: f32,
    pub extraction_radius: f32,
}
impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            sites: [
                Vec3::new(-560., 90., -900.),
                Vec3::new(560., 90., -900.),
                Vec3::new(560., 90., 900.),
            ],
            extraction: Vec3::new(-560., 90., 900.),
            visit_radius: 70.,
            extraction_radius: 95.,
        }
    }
}
#[derive(Resource, Default)]
pub(crate) struct ObjectiveRun {
    pub kind: ObjectiveKind,
    pub visited: [bool; 3],
}
impl ObjectiveRun {
    pub fn count(&self) -> usize {
        self.visited.iter().filter(|v| **v).count()
    }
    pub fn ready(&self) -> bool {
        self.visited.iter().all(|v| *v)
    }
    pub fn survival(&self) -> bool {
        self.kind == ObjectiveKind::Survival
    }
}
pub(crate) fn install(app: &mut App) {
    app.init_resource::<ObjectiveConfig>()
        .init_resource::<ObjectiveRun>()
        .add_systems(
            Update,
            reset
                .in_set(GameplaySet::Reset)
                .run_if(crate::game::reset_requested),
        )
        .add_systems(
            Update,
            update
                .in_set(GameplaySet::Collection)
                .run_if(crate::game::is_playing),
        );
}
fn reset(session: Res<MissionSession>, mut run: ResMut<ObjectiveRun>) {
    *run = ObjectiveRun {
        kind: session.active_mission.unwrap_or_default().objective(),
        ..default()
    };
}
fn update(
    config: Res<ObjectiveConfig>,
    mut run: ResMut<ObjectiveRun>,
    drone: Single<&Transform, With<Drone>>,
    geometry: Option<Res<WorldGeometry>>,
    mut phase: ResMut<GamePhase>,
) {
    if run.survival() {
        return;
    }
    let reached = |position: Vec3, radius: f32| {
        drone.translation.distance(position) <= radius
            && geometry
                .as_ref()
                .is_none_or(|g| g.line_clear(drone.translation, position))
    };
    for (index, position) in config.sites.iter().enumerate() {
        if reached(*position, config.visit_radius) {
            run.visited[index] = true;
        }
    }
    if run.ready() && reached(config.extraction, config.extraction_radius) {
        *phase = GamePhase::Survived;
    }
}
