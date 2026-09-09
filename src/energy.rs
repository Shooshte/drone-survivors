use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

#[cfg(test)]
mod tests;

#[derive(Resource)]
pub(crate) struct EnergyConfig {
    pub capacity: f64,
    pub drain: f64,
    pub recharge: f64,
    pub activation: f64,
}
impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            capacity: 100.,
            drain: 10.,
            recharge: 25.,
            activation: 10.,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct Energy {
    pub current: f64,
    pub overdrive: bool,
    pub charging: Option<Entity>,
    pub rejected_for: f64,
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
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                reset
                    .in_set(GameplaySet::Reset)
                    .run_if(input_just_pressed(KeyCode::KeyR)),
            );
    }
}
fn setup(mut commands: Commands, config: Res<EnergyConfig>, mut energy: ResMut<Energy>) {
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
fn reset(config: Res<EnergyConfig>, mut energy: ResMut<Energy>) {
    *energy = Energy {
        current: config.capacity,
        ..default()
    };
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn update(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<EnergyConfig>,
    phase: Res<GamePhase>,
    drone: Single<&Transform, With<Drone>>,
    nodes: Query<(Entity, &ChargingNode)>,
    mut energy: ResMut<Energy>,
) {
    if *phase != GamePhase::Playing || keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    let dt = time.delta_secs_f64();
    energy.rejected_for = (energy.rejected_for - dt).max(0.);
    if keys.just_pressed(KeyCode::Digit1) {
        if energy.overdrive {
            energy.overdrive = false;
        } else if energy.current >= config.activation {
            energy.overdrive = true;
            energy.rejected_for = 0.;
        } else {
            energy.rejected_for = 1.5;
        }
    }
    // Select one deterministic source: overlapping fields never stack.
    energy.charging = nodes
        .iter()
        .filter(|(_, node)| node.contains(drone.translation))
        .map(|(id, _)| id)
        .min_by_key(|id| id.to_bits());
    let recharge = if energy.charging.is_some() {
        config.recharge
    } else {
        0.
    };
    let drain = if energy.overdrive { config.drain } else { 0. };
    energy.current = (energy.current + (recharge - drain) * dt).clamp(0., config.capacity);
    if energy.current == 0. {
        energy.overdrive = false;
    }
}
