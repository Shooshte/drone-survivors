//! Bridges run-local upgrade rules to existing gameplay and pauses its virtual clock.
use super::{ChoiceAction, ExperienceConfig, UpgradeModifiers, UpgradeRun};
use crate::{
    arena::{Drone, FlightConfig},
    combat::{CombatConfig, Encounter, PlayerHealth},
    energy::{Energy, EnergyConfig},
    game::{GamePhase, GameplaySet},
    modules::{ModuleConfig, Modules},
};
use bevy::prelude::*;

#[derive(Resource, Clone)]
struct Baseline {
    flight: FlightConfig,
    combat: CombatConfig,
    energy: EnergyConfig,
    modules: ModuleConfig,
}

#[derive(Resource)]
pub(crate) struct ExplorationPickup {
    pub position: Vec3,
    pub radius: f32,
    pub collected: bool,
}
impl Default for ExplorationPickup {
    fn default() -> Self {
        Self {
            position: Vec3::new(0., 90., -180.),
            radius: 30.,
            collected: false,
        }
    }
}

#[derive(Resource, Default)]
struct ChoiceSession {
    armed: bool,
    resume_pending: bool,
    credited_kills: u32,
}

pub(crate) struct UpgradePlugin;
impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExperienceConfig>()
            .init_resource::<UpgradeRun>()
            .init_resource::<ChoiceSession>()
            .init_resource::<ExplorationPickup>()
            .init_resource::<Time<Virtual>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(PostStartup, capture_baseline)
            .add_systems(Update, begin_frame.before(GameplaySet::Reset))
            .add_systems(Update, choose.in_set(GameplaySet::ChoiceInput))
            .add_systems(Update, earn.in_set(GameplaySet::Progression));
    }
}

fn capture_baseline(
    mut commands: Commands,
    flight: Res<FlightConfig>,
    combat: Res<CombatConfig>,
    energy: Res<EnergyConfig>,
    modules: Res<ModuleConfig>,
) {
    commands.insert_resource(Baseline {
        flight: *flight,
        combat: combat.clone(),
        energy: energy.clone(),
        modules: modules.clone(),
    });
}

#[allow(clippy::too_many_arguments)]
fn begin_frame(
    keys: Res<ButtonInput<KeyCode>>,
    baseline: Res<Baseline>,
    mut run: ResMut<UpgradeRun>,
    mut session: ResMut<ChoiceSession>,
    mut pickup: ResMut<ExplorationPickup>,
    mut phase: ResMut<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
    mut flight: ResMut<FlightConfig>,
    mut combat: ResMut<CombatConfig>,
    mut energy: ResMut<EnergyConfig>,
    mut modules: ResMut<ModuleConfig>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        *flight = baseline.flight;
        *combat = baseline.combat.clone();
        *energy = baseline.energy.clone();
        *modules = baseline.modules.clone();
        *run = UpgradeRun::default();
        *session = ChoiceSession::default();
        pickup.collected = false;
        clock.unpause();
        // Existing restart systems restore phase, HP, battery and entities afterward.
    } else if session.resume_pending {
        session.resume_pending = false;
        *phase = GamePhase::Playing;
    }
}

#[allow(clippy::too_many_arguments)]
fn earn(
    keys: Res<ButtonInput<KeyCode>>,
    experience: Res<ExperienceConfig>,
    encounter: Res<Encounter>,
    drone: Single<&Transform, With<Drone>>,
    modules: Res<Modules>,
    mut run: ResMut<UpgradeRun>,
    mut session: ResMut<ChoiceSession>,
    mut pickup: ResMut<ExplorationPickup>,
    mut phase: ResMut<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
) {
    if keys.just_pressed(KeyCode::KeyR) || *phase == GamePhase::Choosing {
        return;
    }
    let kills = encounter.kills.saturating_sub(session.credited_kills);
    session.credited_kills = encounter.kills;
    run.award(kills.saturating_mul(experience.kill_xp_at(encounter.elapsed)));
    if *phase != GamePhase::Playing {
        return;
    }
    if !pickup.collected
        && drone.translation.distance_squared(pickup.position) <= pickup.radius.powi(2)
    {
        pickup.collected = true;
        run.award(30);
    }
    run.prepare_offer(&modules.loadout);
    if !run.offer.is_empty() {
        session.armed = false;
        *phase = GamePhase::Choosing;
        clock.pause();
    }
}

#[allow(clippy::too_many_arguments)]
fn choose(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    buttons: Query<(&ChoiceAction, &Interaction), Changed<Interaction>>,
    mut run: ResMut<UpgradeRun>,
    mut session: ResMut<ChoiceSession>,
    phase: Res<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
    baseline: Res<Baseline>,
    mut flight: ResMut<FlightConfig>,
    mut combat: ResMut<CombatConfig>,
    mut energy_config: ResMut<EnergyConfig>,
    mut module_config: ResMut<ModuleConfig>,
    mut health: ResMut<PlayerHealth>,
    mut energy: ResMut<Energy>,
    mut modules: ResMut<Modules>,
    mut launcher: ResMut<crate::combat::RocketLauncher>,
) {
    if *phase != GamePhase::Choosing || keys.just_pressed(KeyCode::KeyR) || session.resume_pending {
        return;
    }
    let hotkeys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Backspace,
    ];
    if !session.armed {
        if !keys.any_pressed(hotkeys) && !mouse.pressed(MouseButton::Left) {
            session.armed = true;
        }
        return;
    }
    let keyboard = hotkeys
        .iter()
        .position(|&key| keys.just_pressed(key))
        .map(|i| {
            if i == 3 {
                ChoiceAction::Skip
            } else {
                ChoiceAction::Pick(i)
            }
        });
    let clicked = buttons.iter().find_map(|(action, interaction)| {
        (*interaction == Interaction::Pressed).then_some(*action)
    });
    let Some(action) = keyboard.or(clicked) else {
        return;
    };
    let index = match action {
        ChoiceAction::Pick(i) => Some(i),
        ChoiceAction::Skip => None,
    };
    if !run.resolve(index) {
        return;
    }
    if index.is_some() {
        let effective = UpgradeModifiers::from_selected(&run.selected);
        let old_hull = combat.player_health;
        let old_recharge = module_config.shield_recharge;
        *flight = baseline.flight;
        flight.horizontal_acceleration_multiplier *= effective.horizontal_acceleration;
        flight.max_horizontal_speed *= effective.speed;
        flight.yaw_rate *= effective.handling;
        flight.tilt_rate *= effective.handling;
        flight.leveling_rate *= effective.handling;
        flight.acceleration_multiplier *= effective.acceleration;
        *combat = baseline.combat.clone();
        combat.player_health = baseline
            .combat
            .player_health
            .saturating_add_signed(effective.hull_delta);
        combat.shot_damage = baseline
            .combat
            .shot_damage
            .saturating_mul(effective.shot_damage);
        if combat.player_health > old_hull {
            health.current = health
                .current
                .saturating_add(combat.player_health - old_hull);
        }
        health.current = health.current.min(combat.player_health);
        *energy_config = baseline.energy.clone();
        energy_config.capacity *= effective.capacity;
        energy.current = energy.current.min(energy_config.capacity);
        *module_config = baseline.modules.clone();
        module_config.shield_recharge *= effective.shield_recharge;
        module_config.drains[crate::modules::ModuleKind::Shield as usize] *= effective.shield_drain;
        module_config.rocket_radius *= effective.rocket_radius;
        module_config.rocket_interval *= effective.rocket_interval;
        launcher.retime(clock.elapsed_secs_f64(), module_config.rocket_interval);
        modules.shield.remaining *= module_config.shield_recharge / old_recharge;
    }
    session.armed = false;
    run.prepare_offer(&modules.loadout);
    if run.offer.is_empty() {
        // Keep Choosing this frame: no movement, firing or power input leakage.
        // TimePlugin resumes the clock before the next frame's gameplay.
        session.resume_pending = true;
        clock.unpause();
    }
}
