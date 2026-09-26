//! Mission 01 map selection and attempt-local optional rewards.
use super::{MissionSession, objectives::ObjectiveConfig};
use crate::{
    arena::Arena,
    game::GameplaySet,
    world::{WorldGeometry, environment::Environment, mission01 as map},
};
use bevy::prelude::*;

pub(crate) const PICKUP_RADIUS: f32 = 600.;
pub(crate) const DELIVERY_RADIUS: f32 = 1000.;
pub(crate) const HIDDEN_RADIUS: f32 = 350.;
pub(crate) const HIDDEN_COMPONENTS: u64 = 3;
pub(crate) const HOLDOUT_COMPONENTS: u64 = 5;
pub(crate) const HOLDOUT_RADIUS: f32 = 2800.;
pub(crate) const HOLDOUT_SECONDS: f64 = 30.;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Holdout {
    #[default]
    Available,
    Active {
        elapsed: f64,
        waves: usize,
    },
    Forfeited,
    Complete,
}
#[derive(Resource, Default)]
pub(crate) struct BlockoutRun {
    pub enabled: bool,
    pub hidden_collected: bool,
    pub holdout: Holdout,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<BlockoutRun>()
        .init_resource::<WorldGeometry>()
        .init_resource::<crate::world::PlayerPath>()
        .init_resource::<Environment>()
        .add_systems(
            Update,
            configure
                .in_set(GameplaySet::Baseline)
                .run_if(crate::game::reset_requested),
        )
        .add_systems(
            Update,
            disable_shared_pickup
                .in_set(GameplaySet::Reset)
                .run_if(crate::game::reset_requested),
        );
}
#[allow(clippy::too_many_arguments)]
fn configure(
    session: Res<MissionSession>,
    mut run: ResMut<BlockoutRun>,
    mut arena: ResMut<Arena>,
    mut geometry: ResMut<WorldGeometry>,
    mut objectives: ResMut<ObjectiveConfig>,
    mut environment: ResMut<Environment>,
    mut waves: ResMut<crate::combat::WaveConfig>,
    mut chargers: Query<(
        &mut crate::energy::ChargingNode,
        &mut crate::energy::ChargingNodeLabel,
    )>,
) {
    let enabled = session.active_mission.is_some_and(|id| id.index() == 0);
    *run = BlockoutRun {
        enabled,
        ..default()
    };
    *arena = if enabled {
        map::arena()
    } else {
        Arena::default()
    };
    *geometry = if enabled {
        map::geometry()
    } else {
        WorldGeometry::default()
    };
    *objectives = if enabled {
        ObjectiveConfig {
            sites: [map::pickup(); 3],
            extraction: map::delivery(),
            visit_radius: PICKUP_RADIUS,
            extraction_radius: DELIVERY_RADIUS,
        }
    } else {
        ObjectiveConfig::default()
    };
    *environment = if enabled {
        Environment {
            fields: map::fields(),
            ..default()
        }
    } else {
        Environment::default()
    };
    *waves = crate::combat::WaveConfig::default();
    if enabled {
        waves.disable_authored_waves();
    }
    // Reuse the six original charger entities/labels. The two unused slots are
    // disabled outside the flight volume; menus and navigation omit them.
    let mission_nodes = map::chargers();
    for (i, (mut node, mut label)) in chargers.iter_mut().enumerate() {
        let (name, center) = if enabled {
            mission_nodes
                .get(i)
                .copied()
                .unwrap_or(("", Vec3::new(0., -1000., 0.)))
        } else {
            crate::world::layout::CHARGERS[i]
        };
        label.0 = name;
        node.center = center;
        node.radius = if enabled && i >= mission_nodes.len() {
            0.
        } else {
            90.
        };
        node.height = 160.;
    }
}

fn disable_shared_pickup(
    run: Res<BlockoutRun>,
    pickup: Option<ResMut<crate::upgrades::runtime::ExplorationPickup>>,
) {
    if run.enabled
        && let Some(mut pickup) = pickup
    {
        pickup.collected = true;
    }
}
