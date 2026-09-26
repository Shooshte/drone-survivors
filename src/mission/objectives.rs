//! Attempt-local objective rules. Completion still crosses the common mission boundary.
use super::{MissionSession, campaign::MissionId};
use crate::world::proximity_contact as contact;
use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
    world::{PlayerPath, WorldGeometry},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ObjectiveKind {
    #[default]
    Survival,
    Reconnaissance,
    Extraction,
    Payload,
}
impl MissionId {
    pub(crate) fn objective(self) -> ObjectiveKind {
        match self.index() {
            0 => ObjectiveKind::Payload,
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
        if self.kind == ObjectiveKind::Payload {
            usize::from(self.visited[0])
        } else {
            self.visited.iter().filter(|v| **v).count()
        }
    }
    pub fn ready(&self) -> bool {
        if self.kind == ObjectiveKind::Payload {
            self.visited[0]
        } else {
            self.visited.iter().all(|v| *v)
        }
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
    path: Option<Res<PlayerPath>>,
    mut phase: ResMut<GamePhase>,
) {
    if run.survival() || run.kind == ObjectiveKind::Payload {
        return;
    }
    // Movement records collision-adjusted substeps in travel order. Never join
    // them into a chord, which could invent a crossing through solid terrain.
    if let Some(path) = path {
        for segment in &path.segments {
            if traverse(
                &config,
                &mut run,
                segment.start,
                segment.end,
                geometry.as_deref(),
            ) {
                *phase = GamePhase::Survived;
                return;
            }
        }
    }
    // Also supports stationary players and fixtures without a movement path.
    if traverse(
        &config,
        &mut run,
        drone.translation,
        drone.translation,
        geometry.as_deref(),
    ) {
        *phase = GamePhase::Survived;
    }
}

fn traverse(
    config: &ObjectiveConfig,
    run: &mut ObjectiveRun,
    start: Vec3,
    end: Vec3,
    geometry: Option<&WorldGeometry>,
) -> bool {
    let mut ready_at = 0_f32;
    for (index, position) in config.sites.iter().enumerate() {
        if !run.visited[index]
            && let Some(at) = contact(start, end, *position, config.visit_radius, geometry)
        {
            run.visited[index] = true;
            ready_at = ready_at.max(at);
        }
    }
    // The exit is only eligible on the portion travelled after the last site.
    // Crossing a locked exit earlier in this same frame is not retroactive.
    run.ready()
        && contact(
            start.lerp(end, ready_at),
            end,
            config.extraction,
            config.extraction_radius,
            geometry,
        )
        .is_some()
}

#[cfg(test)]
#[path = "objective_sweep_tests.rs"]
mod sweep_tests;

impl ObjectiveKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Payload => "Collect payload, then deliver",
            Self::Survival => "Survive 5:00",
            Self::Reconnaissance => "Scan 3 sites, then extract",
            Self::Extraction => "Collect 3 cargo, then extract",
        }
    }
    pub fn heading(self) -> &'static str {
        match self {
            Self::Payload => "Deliver the payload",
            Self::Survival => "Hold out for five minutes",
            Self::Reconnaissance => "Scan the sector",
            Self::Extraction => "Recover the cargo",
        }
    }
    pub fn briefing(self) -> &'static str {
        match self {
            Self::Payload => {
                "Collect the western payload (2), then enter northeast delivery (3). Slot 4 is reserved. No timer. Optional holdout (7): stay 30s; leaving ends your only attempt. Pickup activates both permanent enemy sources."
            }
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
            .filter(|(i, _)| !self.visited[*i] && (self.kind != ObjectiveKind::Payload || *i == 0))
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
        if self.kind == ObjectiveKind::Payload {
            return format!(
                "PAYLOAD {} | {}: {direction} {:.0}u",
                if self.ready() { "LOADED" } else { "RESERVED" },
                if self.ready() {
                    "3 DELIVERY"
                } else {
                    "2 PICKUP"
                },
                delta.with_y(0.).length()
            );
        }
        format!(
            "{task} {}/3 | {label}: {direction} {:.0}u / height {:.0}",
            self.count(),
            delta.length(),
            target.y
        )
    }
}

/// Payload contacts are committed by combat in the same travel order as damage.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PayloadState<'w> {
    config: Option<Res<'w, ObjectiveConfig>>,
    run: Option<ResMut<'w, ObjectiveRun>>,
}
impl PayloadState<'_> {
    pub fn enabled(&self) -> bool {
        self.run
            .as_ref()
            .is_some_and(|r| r.kind == ObjectiveKind::Payload)
    }
    pub fn events(
        &self,
        position: Vec3,
        path: Option<&PlayerPath>,
        geometry: Option<&WorldGeometry>,
    ) -> (Option<f32>, Option<f32>) {
        if let (Some(config), Some(run)) = (&self.config, &self.run)
            && self.enabled()
        {
            payload_events(config, run.ready(), position, path, geometry)
        } else {
            (None, None)
        }
    }
    pub fn resolve(&mut self, at: f32, events: (Option<f32>, Option<f32>), phase: &mut GamePhase) {
        if *phase != GamePhase::Playing {
            return;
        }
        if events.0.is_some_and(|t| t <= at + 1e-6)
            && let Some(run) = self.run.as_mut()
        {
            run.visited[0] = true;
        }
        if events.1.is_some_and(|t| t <= at + 1e-6) {
            *phase = GamePhase::Survived;
        }
    }
}

pub(crate) fn payload_events(
    config: &ObjectiveConfig,
    carrying: bool,
    position: Vec3,
    path: Option<&PlayerPath>,
    geometry: Option<&WorldGeometry>,
) -> (Option<f32>, Option<f32>) {
    let fallback = [crate::world::MotionSegment {
        start: position,
        end: position,
        from: 0.,
        to: 1.,
        half: Vec3::ZERO,
    }];
    let segments = path
        .filter(|p| !p.segments.is_empty())
        .map_or(&fallback[..], |p| p.segments.as_slice());
    let mut pickup = None;
    let mut ready = carrying;
    for segment in segments {
        // Zones span the flight column; full-height terrain still blocks LOS.
        let start = segment.start.with_y(config.sites[0].y);
        let end = segment.end.with_y(config.sites[0].y);
        let mut ready_at = 0.;
        if !ready
            && let Some(t) = contact(start, end, config.sites[0], config.visit_radius, geometry)
        {
            pickup = Some(segment.from + (segment.to - segment.from) * t);
            ready_at = t;
            ready = true;
        }
        if ready
            && let Some(t) = contact(
                start.lerp(end, ready_at),
                end,
                config.extraction,
                config.extraction_radius,
                geometry,
            )
        {
            let t = ready_at + (1. - ready_at) * t;
            return (pickup, Some(segment.from + (segment.to - segment.from) * t));
        }
    }
    (pickup, None)
}
