//! Attempt-local pickups. Installed only by the normal mission lifecycle.
use super::{Amounts, AttemptResources};
use crate::{
    arena::Drone,
    combat::{CombatOutcome, CombatOutcomes},
    game::{GamePhase, GameplaySet, MissionBoundary},
    world::WorldGeometry,
};
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum EnemyKind {
    #[default]
    Chaser,
    Fast,
    Rammer,
}
#[derive(Clone, Copy)]
pub(crate) struct DropRule {
    pub chance_percent: u8,
    pub amount: u64,
}
impl DropRule {
    fn amount_for_roll(self, roll: u8) -> u64 {
        if roll < self.chance_percent.min(100) {
            self.amount
        } else {
            0
        }
    }
}
#[derive(Resource)]
pub(crate) struct EconomyConfig {
    pub chaser: DropRule,
    pub caches: [Vec3; 3],
    pub cache_components: u64,
}
impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            chaser: DropRule {
                chance_percent: 25,
                amount: 1,
            },
            caches: crate::world::layout::cache_sites(),
            cache_components: 1,
        }
    }
}
impl EconomyConfig {
    fn rule(&self, kind: EnemyKind) -> DropRule {
        match kind {
            EnemyKind::Chaser | EnemyKind::Fast | EnemyKind::Rammer => self.chaser,
        }
    }
}
#[derive(Resource)]
struct LootState {
    random: u64,
    seen: HashSet<Entity>,
}
impl Default for LootState {
    fn default() -> Self {
        Self {
            random: 0xd014_5a17_2026,
            seen: HashSet::new(),
        }
    }
}
#[derive(Component)]
pub(crate) struct Pickup {
    pub amount: Amounts,
    pub radius: f32,
    pub attracted: bool,
}
#[derive(Component)]
pub(crate) struct ComponentCache;
#[derive(Component)]
pub(crate) struct DiscoverySite(pub usize);
#[derive(Resource, Default)]
pub(crate) struct DiscoveryNotice {
    pub text: String,
    pub remaining: f32,
}

pub(crate) struct EconomyPlugin;
impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        crate::world::regions::install(app);
        app.init_resource::<crate::energy::ChargerConfig>()
            .init_resource::<EconomyConfig>()
            .init_resource::<LootState>()
            .init_resource::<DiscoveryNotice>()
            .add_systems(
                Update,
                reset
                    .in_set(GameplaySet::Reset)
                    .run_if(crate::game::reset_requested),
            )
            .add_systems(
                Update,
                (spawn_drops, collect)
                    .chain()
                    .in_set(GameplaySet::Collection),
            )
            .add_systems(Update, cleanup.in_set(GameplaySet::Cleanup));
    }
}
fn reset(
    mut commands: Commands,
    config: Res<EconomyConfig>,
    mut loot: ResMut<LootState>,
    pickups: Query<Entity, With<Pickup>>,
    notice: Option<ResMut<DiscoveryNotice>>,
) {
    for entity in &pickups {
        commands.entity(entity).despawn();
    }
    *loot = default();
    if let Some(mut notice) = notice {
        *notice = default();
    }
    for (index, position) in config.caches.into_iter().enumerate() {
        commands.spawn((
            Name::new("Component cache"),
            ComponentCache,
            DiscoverySite(index),
            Pickup {
                amount: Amounts {
                    salvage: 0,
                    components: config.cache_components,
                },
                radius: 50.,
                attracted: false,
            },
            Transform::from_translation(position),
        ));
    }
}

fn spawn_drops(
    mut commands: Commands,
    config: Res<EconomyConfig>,
    mut loot: ResMut<LootState>,
    outcomes: Res<CombatOutcomes>,
    world: Option<Res<WorldGeometry>>,
    phase: Res<GamePhase>,
    boundary: Res<MissionBoundary>,
) {
    if *phase != GamePhase::Playing || boundary.reset || boundary.cleanup {
        return;
    }
    for outcome in &outcomes.0 {
        let &CombatOutcome::Hit {
            entity,
            position,
            killed: true,
            kind,
        } = outcome
        else {
            continue;
        };
        if !loot.seen.insert(entity) {
            continue;
        }
        // SplitMix64 has its own fixed attempt seed; economy never consumes wave/XP randomness.
        loot.random = loot.random.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = loot.random;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^= value >> 31;
        let amount = config.rule(kind).amount_for_roll((value % 100) as u8);
        if amount > 0 {
            commands.spawn((
                Name::new("Salvage drop"),
                Pickup {
                    amount: Amounts {
                        salvage: amount,
                        components: 0,
                    },
                    radius: 100.,
                    attracted: false,
                },
                Transform::from_translation(ground_position(position, world.as_deref())),
            ));
        }
    }
}

type MovingPickups<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Pickup,
        &'static mut Transform,
        Option<&'static ComponentCache>,
        Option<&'static DiscoverySite>,
    ),
    Without<Drone>,
>;

#[allow(clippy::too_many_arguments)]
pub(crate) fn collect(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<GamePhase>,
    boundary: Res<MissionBoundary>,
    world: Option<Res<WorldGeometry>>,
    drone: Single<&Transform, With<Drone>>,
    mut pickups: MovingPickups,
    mut resources: ResMut<AttemptResources>,
    path: Option<Res<crate::world::PlayerPath>>,
    mut run: Option<ResMut<crate::upgrades::UpgradeRun>>,
    mut campaign: Option<ResMut<crate::mission::Campaign>>,
    session: Option<Res<crate::mission::MissionSession>>,
    mut notice: Option<ResMut<DiscoveryNotice>>,
) {
    if *phase != GamePhase::Playing || boundary.reset || boundary.cleanup {
        return;
    }
    if let Some(notice) = notice.as_deref_mut() {
        notice.remaining = (notice.remaining - time.delta_secs()).max(0.);
    }
    for (entity, mut pickup, mut transform, cache, discovery) in &mut pickups {
        let distance = transform.translation.distance(drone.translation);
        if cache.is_some() {
            let contact = |start, end| {
                crate::world::proximity_contact(
                    start,
                    end,
                    transform.translation,
                    pickup.radius,
                    world.as_deref(),
                )
                .is_some()
            };
            // Test actual movement substeps, never a chord across a wall or LOS
            // borrowed from the final frame position.
            pickup.attracted = contact(drone.translation, drone.translation)
                || path.as_ref().is_some_and(|path| {
                    path.segments
                        .iter()
                        .any(|segment| contact(segment.start, segment.end))
                });
        } else {
            if !world
                .as_ref()
                .is_none_or(|w| w.line_clear(transform.translation, drone.translation))
            {
                continue;
            }
            if distance <= pickup.radius {
                pickup.attracted = true;
            }
        }
        if !pickup.attracted {
            continue;
        }
        let step = 900. * time.delta_secs();
        if cache.is_some() || distance <= step.max(8.) {
            // If the counter is full, keep the pickup instead of losing it or partially crediting.
            if resources.collected.try_credit(pickup.amount).is_ok() {
                if let Some(site) = discovery {
                    if let Some(run) = run.as_deref_mut() {
                        run.award(30);
                    }
                    let mut reward = "Cache recovered".to_string();
                    if let Some(campaign) = campaign.as_deref_mut() {
                        match site.0 {
                            0 if campaign.secrets.discover_blueprint() => {
                                reward = "Reserve battery blueprint unlocked".into();
                            }
                            1 => {
                                if let Some(mission) =
                                    session.as_ref().and_then(|s| s.active_mission)
                                {
                                    let act = mission.index() / 4;
                                    if campaign.progress.discover_route(act) {
                                        reward = format!(
                                            "Secret route: mission {:02} unlocked",
                                            act * 4 + 4
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(notice) = notice.as_deref_mut() {
                        notice.text = format!("{reward} / +30 XP");
                        notice.remaining = 8.;
                    }
                }
                commands.entity(entity).despawn();
            }
        } else {
            transform.translation = transform.translation.move_towards(drone.translation, step);
        }
    }
}

fn cleanup(
    mut commands: Commands,
    boundary: Res<MissionBoundary>,
    pickups: Query<Entity, With<Pickup>>,
) {
    if boundary.cleanup {
        for entity in &pickups {
            commands.entity(entity).despawn();
        }
    }
}

fn ground_position(mut position: Vec3, world: Option<&WorldGeometry>) -> Vec3 {
    let mut height = 0_f32;
    if let Some(world) = world {
        for solid in &world.solids {
            let top = solid.center.y + solid.half.y;
            let offset = (position - solid.center).abs();
            if offset.x <= solid.half.x && offset.z <= solid.half.z && top <= position.y {
                height = height.max(top);
            }
        }
    }
    position.y = height + 6.;
    position
}
#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
