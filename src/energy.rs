use crate::modules::{ModuleConfig, Modules};
use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
};
use bevy::prelude::*;

mod flow;
pub(crate) mod scene;

#[cfg(test)]
pub(crate) mod tests;

#[derive(Resource, Clone)]
pub(crate) struct EnergyConfig {
    pub capacity: f64,
    pub recharge: f64,
    pub activation: f64,
    pub reserve_cost: f64,
}
impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            capacity: 100.,
            recharge: 25.,
            activation: 10.,
            reserve_cost: 1.,
        }
    }
}

#[derive(Resource, Clone)]
pub(crate) struct ChargerConfig {
    pub capacity: f64,
    pub recovery_delay: f64,
    pub recovery_rate: f64,
}
impl Default for ChargerConfig {
    fn default() -> Self {
        Self {
            capacity: 200.,
            recovery_delay: 8.,
            recovery_rate: 10.,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub(crate) struct Energy {
    pub current: f64,
    pub charging: Option<Entity>,
}

/// Stage power before contact, commit after outcome resolution. Terminal frames
/// leave battery/toggles unchanged; contact still observes depletion immediately.
#[derive(Resource, Default)]
pub(crate) struct PowerFrame {
    pub energy: Energy,
    pub modules: Modules,
    pub chargers: Vec<(Entity, ChargerReserve)>,
}

#[derive(Component, Clone, Copy)]
#[require(ChargerReserve)]
pub(crate) struct ChargingNode {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct ChargingNodeLabel(pub &'static str);

pub(crate) fn charging_node_name(
    node: &ChargingNode,
    label: Option<&ChargingNodeLabel>,
) -> &'static str {
    if let Some(label) = label {
        return label.0;
    }
    match (node.center.x < 0., node.center.z.total_cmp(&0.)) {
        (true, std::cmp::Ordering::Less) => "NW",
        (false, std::cmp::Ordering::Less) => "NE",
        (true, std::cmp::Ordering::Greater) => "SW",
        (false, std::cmp::Ordering::Greater) => "SE",
        (true, _) => "LEFT",
        (false, _) => "RIGHT",
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct ChargerReserve {
    pub remaining: f64,
    pub away_seconds: f64,
    pub occupied: bool,
}
impl ChargerReserve {
    fn full(config: &ChargerConfig) -> Self {
        Self {
            remaining: config.capacity,
            away_seconds: 0.,
            occupied: false,
        }
    }
}
impl Default for ChargerReserve {
    fn default() -> Self {
        Self::full(&ChargerConfig::default())
    }
}
impl ChargingNode {
    pub fn contains(&self, point: Vec3) -> bool {
        let offset = point - self.center;
        (0. ..=self.height).contains(&offset.y)
            && offset.xz().length_squared() <= self.radius * self.radius
    }
}

pub(crate) struct EnergyPlugin;
impl Plugin for EnergyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnergyConfig>()
            .init_resource::<ChargerConfig>()
            .init_resource::<Energy>()
            .init_resource::<ModuleConfig>()
            .init_resource::<Modules>()
            .init_resource::<PowerFrame>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                reset
                    .in_set(GameplaySet::Reset)
                    .run_if(crate::game::reset_requested),
            );
    }
}
fn setup(
    mut commands: Commands,
    config: Res<EnergyConfig>,
    charger_config: Res<ChargerConfig>,
    mut energy: ResMut<Energy>,
    config_modules: Res<ModuleConfig>,
    mut modules: ResMut<Modules>,
) {
    *modules = Modules::new(modules.loadout.clone(), &config_modules);
    *energy = Energy {
        current: config.capacity,
        ..default()
    };
    for (label, center) in crate::world::layout::CHARGERS {
        commands.spawn((
            ChargingNode {
                center,
                radius: 90.,
                height: 160.,
            },
            ChargerReserve::full(&charger_config),
            ChargingNodeLabel(label),
        ));
    }
}
#[allow(clippy::too_many_arguments)]
fn reset(
    config: Res<EnergyConfig>,
    charger_config: Res<ChargerConfig>,
    mut energy: ResMut<Energy>,
    config_modules: Res<ModuleConfig>,
    mut modules: ResMut<Modules>,
    mut chargers: Query<&mut ChargerReserve>,
    mut pending: ResMut<PowerFrame>,
    boundary: Option<Res<crate::game::MissionBoundary>>,
) {
    if boundary.is_some() {
        modules.loadout = default();
    }
    *pending = default();
    *modules = Modules::new(modules.loadout.clone(), &config_modules);
    *energy = Energy {
        current: config.capacity,
        ..default()
    };
    for mut reserve in &mut chargers {
        *reserve = ChargerReserve::full(&charger_config);
    }
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<EnergyConfig>,
    charger_config: Res<ChargerConfig>,
    module_config: Res<ModuleConfig>,
    phase: Res<GamePhase>,
    drone: Single<&Transform, With<Drone>>,
    nodes: Query<(Entity, &ChargingNode, &ChargerReserve)>,
    energy: Res<Energy>,
    modules: Res<Modules>,
    mut pending: ResMut<PowerFrame>,
) {
    if *phase != GamePhase::Playing || keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    let dt = time.delta_secs_f64();
    pending.energy = energy.clone();
    pending.modules = modules.clone();
    pending
        .modules
        .toggle(&keys, energy.current, config.activation, dt);
    pending.chargers = nodes
        .iter()
        .map(|(entity, node, reserve)| {
            let mut reserve = *reserve;
            reserve.occupied = node.contains(drone.translation);
            (entity, reserve)
        })
        .collect();
    pending.chargers.sort_by_key(|(entity, _)| entity.to_bits());
    let module_drain = pending.modules.drain(&module_config);
    let modules_enabled = pending.modules.enabled.iter().any(|enabled| *enabled);
    let result = flow::advance(
        dt,
        energy.current,
        module_drain,
        modules_enabled,
        &flow::FlowConfig {
            battery_capacity: config.capacity,
            reserve_cost: config.reserve_cost,
            delivery_rate: config.recharge,
            charger_capacity: charger_config.capacity,
            recovery_delay: charger_config.recovery_delay,
            recovery_rate: charger_config.recovery_rate,
        },
        &mut pending.chargers,
    );
    pending
        .modules
        .recharge_shield(result.powered_seconds, &module_config);
    pending.energy.current = result.battery;
    pending.energy.charging = result.charging;
    if result.depleted_modules {
        pending.modules.enabled = [false; 4];
    }
}
pub(crate) fn update(
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<GamePhase>,
    pending: Res<PowerFrame>,
    mut energy: ResMut<Energy>,
    mut modules: ResMut<Modules>,
    mut chargers: Query<&mut ChargerReserve>,
) {
    if *phase == GamePhase::Playing && !keys.just_pressed(KeyCode::KeyR) {
        *energy = pending.energy.clone();
        *modules = pending.modules.clone();
        for (entity, reserve) in &pending.chargers {
            if let Ok(mut current) = chargers.get_mut(*entity) {
                *current = *reserve;
            }
        }
    }
}

#[cfg(test)]
mod efficiency_tests;
