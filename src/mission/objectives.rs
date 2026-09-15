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
                .after(crate::economy::runtime::collect)
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

impl ObjectiveKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Survival => "Survive 5:00",
            Self::Reconnaissance => "Scan 3 sites, then extract",
            Self::Extraction => "Collect 3 cargo, then extract",
        }
    }
    pub fn heading(self) -> &'static str {
        match self {
            Self::Survival => "Hold out for five minutes",
            Self::Reconnaissance => "Scan the sector",
            Self::Extraction => "Recover the cargo",
        }
    }
    pub fn briefing(self) -> &'static str {
        match self {
            Self::Survival => {
                "Survive for 5:00. Keep your hull above zero as enemy waves grow. Upgrade choices pause the clock."
            }
            Self::Reconnaissance => {
                "Visit 3 cyan scan sites, then reach the green extraction rings. Fly within 70 units of each beacon at height 90; scanning is automatic. No time limit. Keep your hull above zero."
            }
            Self::Extraction => {
                "Collect 3 orange cargo pickups, then reach the green extraction rings. Fly within 70 units at height 90 to collect automatically. Cargo is separate from loot. No time limit; zero hull fails."
            }
        }
    }
}
impl ObjectiveRun {
    pub fn next_target(&self, config: &ObjectiveConfig, position: Vec3) -> (Option<usize>, Vec3) {
        config
            .sites
            .iter()
            .enumerate()
            .filter(|(i, _)| !self.visited[*i])
            .min_by(|(_, a), (_, b)| {
                a.distance_squared(position)
                    .total_cmp(&b.distance_squared(position))
            })
            .map(|(i, p)| (Some(i), *p))
            .unwrap_or((None, config.extraction))
    }
    pub fn guidance(&self, config: &ObjectiveConfig, position: Vec3) -> String {
        let (index, target) = self.next_target(config, position);
        let task = if self.kind == ObjectiveKind::Extraction {
            "CARGO"
        } else {
            "SCAN"
        };
        let label = index.map_or_else(
            || "EXTRACT".into(),
            |i| {
                format!(
                    "{} {}",
                    if self.kind == ObjectiveKind::Extraction {
                        "Cargo"
                    } else {
                        "Site"
                    },
                    i + 1
                )
            },
        );
        let delta = target - position;
        let direction = if delta.x.abs().max(delta.z.abs()) < 5. {
            "HERE"
        } else {
            let angle = delta.x.atan2(-delta.z);
            let sector = (angle / std::f32::consts::FRAC_PI_4).round() as i32;
            ["N", "NE", "E", "SE", "S", "SW", "W", "NW"][sector.rem_euclid(8) as usize]
        };
        format!(
            "{task} {}/3 | {label}: {direction} {:.0}u / height {:.0}",
            self.count(),
            delta.length(),
            target.y
        )
    }
}
