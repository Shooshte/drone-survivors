//! Bridges run-local upgrade rules to existing gameplay and pauses its virtual clock.
use super::{ChoiceAction, ExperienceConfig, UpgradeModifiers, UpgradePool, UpgradeRun};
use crate::{
    arena::{Drone, FlightConfig},
    combat::{CombatConfig, Encounter, PlayerHealth},
    energy::{Energy, EnergyConfig},
    game::{GamePhase, GameplaySet},
    modules::{ModuleConfig, Modules},
};
use bevy::prelude::*;

#[derive(Resource, Clone, Default)]
pub(crate) struct Baseline {
    flight: FlightConfig,
    combat: CombatConfig,
    energy: EnergyConfig,
    modules: ModuleConfig,
}

impl Baseline {
    pub(crate) fn launch_power(
        &self,
        campaign: &crate::mission::Campaign,
    ) -> (EnergyConfig, ModuleConfig) {
        let effective = self.for_campaign(Some(campaign));
        (effective.energy, effective.modules)
    }

    fn for_campaign(&self, campaign: Option<&crate::mission::Campaign>) -> Self {
        let mut effective = self.clone();
        if let Some(campaign) = campaign {
            campaign
                .passives
                .apply(&mut effective.combat, &mut effective.energy);
            if campaign.secrets.reserve_battery {
                effective.energy.capacity += 25.;
            }
        }
        effective
    }
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
            .init_resource::<UpgradePool>()
            .init_resource::<ChoiceSession>()
            .init_resource::<ExplorationPickup>()
            .init_resource::<Time<Virtual>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_systems(PostStartup, capture_baseline)
            .add_systems(Update, begin_frame.in_set(GameplaySet::Baseline))
            .add_systems(
                Update,
                open_reset_preview
                    .after(GameplaySet::Reset)
                    .before(GameplaySet::ChoiceInput),
            )
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
    boundary: Option<Res<crate::game::MissionBoundary>>,
    baseline: Res<Baseline>,
    campaign: Option<Res<crate::mission::Campaign>>,
    run_state: (Res<UpgradePool>, ResMut<UpgradeRun>),
    mut session: ResMut<ChoiceSession>,
    mut pickup: ResMut<ExplorationPickup>,
    mut phase: ResMut<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
    mut flight: ResMut<FlightConfig>,
    mut combat: ResMut<CombatConfig>,
    mut energy: ResMut<EnergyConfig>,
    module_state: (ResMut<ModuleConfig>, Res<Modules>),
) {
    let (pool, mut run) = run_state;
    let (mut module_config, equipped) = module_state;
    if boundary
        .as_ref()
        .map_or_else(|| keys.just_pressed(KeyCode::KeyR), |b| b.reset)
    {
        let baseline = baseline.for_campaign(campaign.as_deref());
        *flight = baseline.flight;
        *combat = baseline.combat.clone();
        *energy = baseline.energy.clone();
        *module_config = baseline.modules.clone();
        run.reset_for_pool(&pool, &equipped.loadout);
        *session = ChoiceSession::default();
        *pickup = ExplorationPickup::default();
        clock.unpause();
        // Existing restart systems restore phase, HP, battery and entities afterward.
    } else if session.resume_pending {
        session.resume_pending = false;
        *phase = GamePhase::Playing;
    }
}

fn open_reset_preview(
    keys: Res<ButtonInput<KeyCode>>,
    boundary: Option<Res<crate::game::MissionBoundary>>,
    pool: Res<UpgradePool>,
    modules: Res<Modules>,
    mut run: ResMut<UpgradeRun>,
    mut session: ResMut<ChoiceSession>,
    mut phase: ResMut<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
) {
    let resetting = boundary
        .as_ref()
        .map_or_else(|| keys.just_pressed(KeyCode::KeyR), |b| b.reset);
    if !resetting || !pool.catalog || *phase != GamePhase::Playing || run.pending == 0 {
        return;
    }
    run.prepare_catalog_offer(&modules.loadout);
    if !run.offer.is_empty() {
        session.armed = false;
        *phase = GamePhase::Choosing;
        clock.pause();
    }
}

#[allow(clippy::too_many_arguments)]
fn earn(
    keys: Res<ButtonInput<KeyCode>>,
    boundary: Option<Res<crate::game::MissionBoundary>>,
    experience: Res<ExperienceConfig>,
    encounter: Res<Encounter>,
    drone: Single<&Transform, With<Drone>>,
    modules: Res<Modules>,
    pool: Res<UpgradePool>,
    mut run: ResMut<UpgradeRun>,
    mut session: ResMut<ChoiceSession>,
    mut pickup: ResMut<ExplorationPickup>,
    mut phase: ResMut<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
) {
    if boundary.is_some_and(|b| b.reset)
        || keys.just_pressed(KeyCode::KeyR)
        || matches!(
            *phase,
            GamePhase::Choosing
                | GamePhase::Hub
                | GamePhase::Passives
                | GamePhase::ModuleShop
                | GamePhase::MissionSelect
                | GamePhase::Briefing
        )
    {
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
    if pool.catalog {
        run.prepare_catalog_offer(&modules.loadout);
    } else {
        run.prepare_offer(&modules.loadout);
    }
    if !run.offer.is_empty() {
        session.armed = false;
        *phase = GamePhase::Choosing;
        clock.pause();
    }
}

#[allow(clippy::too_many_arguments)]
fn choose(
    input: (Res<ButtonInput<KeyCode>>, Res<ButtonInput<MouseButton>>),
    buttons: Query<(&ChoiceAction, &Interaction), Changed<Interaction>>,
    upgrade_state: (Res<UpgradePool>, ResMut<UpgradeRun>, ResMut<ChoiceSession>),
    phase: Res<GamePhase>,
    mut clock: ResMut<Time<Virtual>>,
    baseline: Res<Baseline>,
    campaign: Option<Res<crate::mission::Campaign>>,
    mut flight: ResMut<FlightConfig>,
    mut combat: ResMut<CombatConfig>,
    mut energy_config: ResMut<EnergyConfig>,
    mut module_config: ResMut<ModuleConfig>,
    mut health: ResMut<PlayerHealth>,
    mut energy: ResMut<Energy>,
    mut modules: ResMut<Modules>,
    cooldowns: (
        ResMut<crate::combat::RocketLauncher>,
        ResMut<crate::combat::bombs::BombState>,
    ),
) {
    let (pool, mut run, mut session) = upgrade_state;
    let (mut launcher, mut bomb) = cooldowns;
    let (keys, mouse) = input;
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
        let baseline = baseline.for_campaign(campaign.as_deref());
        let effective = UpgradeModifiers::from_selected(&run.selected);
        let old_hull = combat.player_health;
        let old_recharge = module_config.shield_recharge;
        let old_repulsor_interval = module_config.repulsor_interval;
        *flight = baseline.flight;
        flight.horizontal_acceleration_multiplier *= effective.horizontal_acceleration;
        flight.max_horizontal_speed *= effective.speed;
        flight.yaw_rate *= effective.handling;
        flight.bank_yaw_rate *= effective.handling;
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
        combat.target_range = baseline.combat.target_range * effective.target_range;
        combat.fire_interval = baseline.combat.fire_interval * effective.fire_interval;
        if combat.player_health > old_hull {
            health.current = health
                .current
                .saturating_add(combat.player_health - old_hull);
        }
        health.current = health.current.min(combat.player_health);
        *energy_config = baseline.energy.clone();
        energy_config.capacity *= effective.capacity;
        energy_config.activation *= effective.activation;
        energy.current = energy.current.min(energy_config.capacity);
        *module_config = baseline.modules.clone();
        for drain in &mut module_config.drains {
            *drain *= effective.module_drain;
        }
        module_config.drains[crate::modules::ModuleKind::Overdrive as usize] *=
            effective.overdrive_drain;
        module_config.shield_recharge *= effective.shield_recharge;
        module_config.drains[crate::modules::ModuleKind::Shield as usize] *= effective.shield_drain;
        module_config.rocket_radius *= effective.rocket_radius;
        module_config.rocket_interval *= effective.rocket_interval;
        module_config.overdrive_multiplier *= effective.overdrive_multiplier;
        module_config.repair_rate *= effective.repair_rate;
        module_config.repair_drain *= effective.module_drain * effective.repair_drain;
        module_config.repulsor_radius *= effective.repulsor_radius;
        module_config.repulsor_interval *= effective.repulsor_interval;
        module_config.repulsor_drain *= effective.module_drain;
        launcher.retime(clock.elapsed_secs_f64(), module_config.rocket_interval);
        modules.shield.remaining *= module_config.shield_recharge / old_recharge;
        bomb.pulse_cooldown *= module_config.repulsor_interval / old_repulsor_interval;
    }
    session.armed = false;
    if pool.catalog {
        run.prepare_catalog_offer(&modules.loadout);
    } else {
        run.prepare_offer(&modules.loadout);
    }
    if run.offer.is_empty() {
        // Keep Choosing this frame: no movement, firing or power input leakage.
        // TimePlugin resumes the clock before the next frame's gameplay.
        session.resume_pending = true;
        clock.unpause();
    }
}
