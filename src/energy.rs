use crate::modules::{ModuleConfig, Modules};
use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

pub(crate) mod scene;

#[cfg(test)]
pub(crate) mod tests;

#[derive(Resource, Clone)]
pub(crate) struct EnergyConfig {
    pub capacity: f64,
    pub recharge: f64,
    pub activation: f64,
}
impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            capacity: 100.,
            recharge: 25.,
            activation: 10.,
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
}

#[derive(Component, Clone, Copy)]
pub(crate) struct ChargingNode {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
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
            .init_resource::<Energy>()
            .init_resource::<ModuleConfig>()
            .init_resource::<Modules>()
            .init_resource::<PowerFrame>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                reset
                    .in_set(GameplaySet::Reset)
                    .run_if(input_just_pressed(KeyCode::KeyR)),
            );
    }
}
fn setup(
    mut commands: Commands,
    config: Res<EnergyConfig>,
    mut energy: ResMut<Energy>,
    config_modules: Res<ModuleConfig>,
    mut modules: ResMut<Modules>,
) {
    *modules = Modules::new(modules.loadout.clone(), &config_modules);
    *energy = Energy {
        current: config.capacity,
        ..default()
    };
    for x in [-280., 280.] {
        commands.spawn(ChargingNode {
            center: Vec3::new(x, 0., 0.),
            radius: 90.,
            height: 160.,
        });
    }
}
fn reset(
    config: Res<EnergyConfig>,
    mut energy: ResMut<Energy>,
    config_modules: Res<ModuleConfig>,
    mut modules: ResMut<Modules>,
) {
    *modules = Modules::new(modules.loadout.clone(), &config_modules);
    *energy = Energy {
        current: config.capacity,
        ..default()
    };
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<EnergyConfig>,
    module_config: Res<ModuleConfig>,
    phase: Res<GamePhase>,
    drone: Single<&Transform, With<Drone>>,
    nodes: Query<(Entity, &ChargingNode)>,
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
    pending.energy.charging = nodes
        .iter()
        .filter(|(_, node)| node.contains(drone.translation))
        .map(|(id, _)| id)
        .min_by_key(|id| id.to_bits());
    let recharge = if pending.energy.charging.is_some() {
        config.recharge
    } else {
        0.
    };
    let net = recharge - pending.modules.drain(&module_config);
    let powered_seconds = if net < 0. {
        dt.min(energy.current / -net)
    } else {
        dt
    };
    pending
        .modules
        .recharge_shield(powered_seconds, &module_config);
    pending.energy.current = (energy.current + net * dt).clamp(0., config.capacity);
    if pending.energy.current == 0. {
        pending.modules.enabled = [false; 4];
    }
}
pub(crate) fn update(
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<GamePhase>,
    pending: Res<PowerFrame>,
    mut energy: ResMut<Energy>,
    mut modules: ResMut<Modules>,
) {
    if *phase == GamePhase::Playing && !keys.just_pressed(KeyCode::KeyR) {
        *energy = pending.energy.clone();
        *modules = pending.modules.clone();
    }
}
