use super::{Encounter, ValidationConfig, ValidationMode};
use crate::{
    arena::Drone,
    energy::Energy,
    game::GamePhase,
    modules::{ModuleKind, Modules},
    world::WorldGeometry,
};
use bevy::prelude::*;

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ObservationEvent {
    Module {
        run_seconds: f64,
        slot: usize,
        kind: ModuleKind,
        enabled: bool,
    },
    Charger {
        run_seconds: f64,
        entered: bool,
        node: Entity,
    },
    RouteCrossing {
        run_seconds: f64,
        from_left: bool,
        z: f32,
    },
}

#[derive(Resource, Default)]
pub(super) struct ManualObservations {
    pub(super) events: Vec<ObservationEvent>,
    modules: Option<[bool; 4]>,
    charger: Option<Option<Entity>>,
    position: Option<Vec3>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record(
    config: Res<ValidationConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<GamePhase>,
    encounter: Res<Encounter>,
    drone: Single<&Transform, With<Drone>>,
    modules: Res<Modules>,
    energy: Res<Energy>,
    world: Option<Res<WorldGeometry>>,
    mut observations: ResMut<ManualObservations>,
) {
    if config.mode != ValidationMode::Manual || *phase != GamePhase::Playing {
        return;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        observations.seed(&modules, &energy, drone.translation);
        return;
    }
    observations.observe_modules(encounter.elapsed, &modules);
    observations.observe_charger(encounter.elapsed, &energy);
    if let Some(divider_x) = world
        .as_deref()
        .and_then(|world| world.hazard.map(|hazard| hazard.center.x))
    {
        observations.observe_route(encounter.elapsed, drone.translation, divider_x);
    } else {
        observations.position = Some(drone.translation);
    }
}

impl ManualObservations {
    pub(super) fn summary(&self) -> String {
        let mut module_changes = 0;
        let mut charger_entries = 0;
        let mut charger_exits = 0;
        let mut route_crossings = 0;
        for event in &self.events {
            match event {
                ObservationEvent::Module { .. } => module_changes += 1,
                ObservationEvent::Charger { entered: true, .. } => charger_entries += 1,
                ObservationEvent::Charger { entered: false, .. } => charger_exits += 1,
                ObservationEvent::RouteCrossing { .. } => route_crossings += 1,
            }
        }
        format!(
            "module_changes={module_changes} charger_entries={charger_entries} charger_exits={charger_exits} route_crossings={route_crossings}"
        )
    }

    fn seed(&mut self, modules: &Modules, energy: &Energy, position: Vec3) {
        self.modules = Some(modules.enabled);
        self.charger = Some(energy.charging);
        self.position = Some(position);
    }

    fn observe_modules(&mut self, run_seconds: f64, modules: &Modules) {
        let Some(previous) = self.modules.replace(modules.enabled) else {
            return;
        };
        for (slot, (&was_enabled, &enabled)) in previous.iter().zip(&modules.enabled).enumerate() {
            if was_enabled == enabled {
                continue;
            }
            let Some(kind) = modules.loadout.slots()[slot] else {
                continue;
            };
            println!(
                "VALIDATION OBSERVATION run_seconds={run_seconds:.3} module={} slot={} enabled={enabled}",
                kind.name(),
                slot + 1
            );
            self.events.push(ObservationEvent::Module {
                run_seconds,
                slot,
                kind,
                enabled,
            });
        }
    }

    fn observe_charger(&mut self, run_seconds: f64, energy: &Energy) {
        let Some(previous) = self.charger.replace(energy.charging) else {
            return;
        };
        if previous == energy.charging {
            return;
        }
        if let Some(node) = previous {
            println!(
                "VALIDATION OBSERVATION run_seconds={run_seconds:.3} charger=exit node={}",
                node.to_bits()
            );
            self.events.push(ObservationEvent::Charger {
                run_seconds,
                entered: false,
                node,
            });
        }
        if let Some(node) = energy.charging {
            println!(
                "VALIDATION OBSERVATION run_seconds={run_seconds:.3} charger=enter node={}",
                node.to_bits()
            );
            self.events.push(ObservationEvent::Charger {
                run_seconds,
                entered: true,
                node,
            });
        }
    }

    pub(super) fn observe_route(&mut self, run_seconds: f64, position: Vec3, divider_x: f32) {
        let Some(previous) = self.position else {
            self.position = Some(position);
            return;
        };
        let previous_offset = previous.x - divider_x;
        let current_offset = position.x - divider_x;
        if current_offset == 0. {
            return;
        }
        self.position = Some(position);
        if previous_offset == 0. || previous_offset.signum() == current_offset.signum() {
            return;
        }
        let fraction = previous_offset.abs() / (previous_offset.abs() + current_offset.abs());
        let crossing_z = previous.z + (position.z - previous.z) * fraction;
        let from_left = previous_offset < 0.;
        println!(
            "VALIDATION OBSERVATION run_seconds={run_seconds:.3} route_crossing={} z={crossing_z:.1}",
            if from_left {
                "left_to_right"
            } else {
                "right_to_left"
            }
        );
        self.events.push(ObservationEvent::RouteCrossing {
            run_seconds,
            from_left,
            z: crossing_z,
        });
    }
}
