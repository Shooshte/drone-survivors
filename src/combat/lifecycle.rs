use super::feedback::KillEffect;
use super::{
    CombatConfig, CombatOutcome, CombatOutcomes, Encounter, Enemy, PlayerHealth, Projectile,
    SpawnWarning, Weapon,
};
use crate::{
    arena::{Drone, drone_world_half_extents, world_half_extents},
    game::GamePhase,
};
use bevy::prelude::*;

type CombatEntities = Or<(
    With<Enemy>,
    With<Projectile>,
    With<SpawnWarning>,
    With<KillEffect>,
)>;

pub(super) fn setup(mut commands: Commands, config: Res<CombatConfig>) {
    commands.insert_resource(PlayerHealth {
        current: config.player_health,
        invulnerable_until: 0.,
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn restart(
    mut commands: Commands,
    config: Res<CombatConfig>,
    mut health: ResMut<PlayerHealth>,
    mut weapon: ResMut<Weapon>,
    mut rockets: ResMut<super::rockets::RocketLauncher>,
    mut phase: ResMut<GamePhase>,
    mut run: ResMut<Encounter>,
    transient: Query<Entity, CombatEntities>,
) {
    for entity in &transient {
        commands.entity(entity).despawn();
    }
    health.current = config.player_health;
    health.invulnerable_until = 0.;
    *weapon = Weapon::default();
    *rockets = default();
    *run = Encounter::default();
    *phase = GamePhase::Playing;
}

type ContactEnemies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Enemy,
        &'static Transform,
        Option<&'static mut super::variants::Rammer>,
    ),
>;

#[allow(clippy::too_many_arguments)]
pub(super) fn contact_damage(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<CombatConfig>,
    drone: Single<&Transform, With<Drone>>,
    path: Option<Res<crate::world::PlayerPath>>,
    world: Option<Res<crate::world::WorldGeometry>>,
    mut enemies: ContactEnemies,
    mut health: ResMut<PlayerHealth>,
    mut phase: ResMut<GamePhase>,
    mut outcomes: ResMut<CombatOutcomes>,
    mut power: ResMut<crate::energy::PowerFrame>,
    modules: Res<crate::modules::ModuleConfig>,
    mut bomb: ResMut<super::bombs::BombState>,
) {
    use super::variants::RamPhase;
    use crate::economy::runtime::EnemyKind;
    let now = time.elapsed_secs_f64();
    if *phase != GamePhase::Playing {
        return;
    }
    let mut contacts: Vec<_> = enemies
        .iter()
        .filter_map(|(id, enemy, target, rammer)| {
            if enemy.health == 0
                || (enemy.kind == EnemyKind::Rammer
                    && !rammer.is_some_and(|r| r.phase == RamPhase::Charge))
            {
                return None;
            }
            let at = if enemy.kind == EnemyKind::Chaser {
                let half = drone_world_half_extents(drone.rotation)
                    + world_half_extents(target.rotation, Vec3::splat(config.enemy_half_size))
                    + Vec3::splat(0.001);
                (target.translation - drone.translation)
                    .abs()
                    .cmple(half)
                    .all()
                    .then_some(1.)
            } else {
                super::variant_contact::contact(
                    enemy,
                    target,
                    &drone,
                    path.as_deref(),
                    &config,
                    world.as_deref(),
                )
            }?;
            Some((id, at))
        })
        .collect();
    contacts.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.to_bits().cmp(&b.0.to_bits())));
    for (id, _) in contacts {
        if *phase != GamePhase::Playing {
            break;
        }
        if let Ok((_, mut enemy, _, _)) = enemies.get_mut(id)
            && enemy.kind == EnemyKind::Bomber
        {
            bomb.attach();
            enemy.health = 0;
            commands.entity(id).despawn();
            continue;
        }
        let accepted = apply_player_damage(
            config.contact_damage,
            now,
            &config,
            &mut health,
            &mut phase,
            &mut outcomes,
            &mut power,
            &modules,
        );
        if let Ok((_, mut enemy, _, Some(mut rammer))) = enemies.get_mut(id)
            && rammer.impact(accepted)
        {
            // Expending a hull on impact is not a player kill or a loot event.
            enemy.health = 0;
            commands.entity(id).despawn();
        }
    }
}

/// Returns true only when damage or a shield block was actually consumed.
#[allow(clippy::too_many_arguments)]
pub(super) fn apply_player_damage(
    amount: u32,
    now: f64,
    config: &CombatConfig,
    health: &mut PlayerHealth,
    phase: &mut GamePhase,
    outcomes: &mut CombatOutcomes,
    power: &mut crate::energy::PowerFrame,
    modules: &crate::modules::ModuleConfig,
) -> bool {
    if *phase != GamePhase::Playing || now + 1e-7 < health.invulnerable_until {
        return false;
    }
    if !power.modules.block(modules) {
        health.current = health.current.saturating_sub(amount);
        outcomes.0.push(CombatOutcome::PlayerDamaged);
    }
    health.invulnerable_until = now + config.invulnerability;
    if health.current == 0 {
        *phase = GamePhase::Dead;
    }
    true
}

/// Clear transients at the completion boundary before presentation can consume them.
pub(super) fn cleanup(
    mut commands: Commands,
    boundary: Res<crate::game::MissionBoundary>,
    transient: Query<Entity, CombatEntities>,
    mut outcomes: ResMut<CombatOutcomes>,
    mut cue: Option<ResMut<super::feedback::DamageCue>>,
) {
    if !boundary.cleanup && !boundary.reset {
        return;
    }
    for entity in &transient {
        commands.entity(entity).despawn();
    }
    outcomes.0.clear();
    if let Some(cue) = cue.as_mut() {
        cue.0 = 0.;
    }
}
