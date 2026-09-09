use super::{CombatConfig, Enemy, enemies::spawn_enemy};
use crate::arena::{Arena, Drone, drone_world_half_extents, world_half_extents};
use crate::game::GamePhase;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct WaveConfig {
    pub duration: f64,
    pub cap: usize,
    pub warning_seconds: f64,
    pub clearance: f32,
    pub bursts: Vec<(f64, usize)>,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            duration: 180.,
            cap: 30,
            warning_seconds: 0.75,
            clearance: 120.,
            bursts: [
                (3., 3),
                (15., 3),
                (27., 3),
                (39., 3),
                (60., 4),
                (70., 4),
                (80., 4),
                (90., 4),
                (120., 5),
                (128., 5),
                (136., 5),
                (144., 5),
                (152., 5),
            ]
            .into(),
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct Encounter {
    pub elapsed: f64,
    pub kills: u32,
    pub next_burst: usize,
    pub candidate: usize,
}

#[derive(Component)]
pub(super) struct SpawnWarning {
    pub ready_at: f64,
}

pub(super) fn advance_clock(time: Res<Time>, config: Res<WaveConfig>, mut run: ResMut<Encounter>) {
    run.elapsed = (run.elapsed + time.delta_secs_f64()).min(config.duration);
}

pub(super) fn finish(config: Res<WaveConfig>, run: Res<Encounter>, mut phase: ResMut<GamePhase>) {
    if *phase == GamePhase::Playing && run.elapsed + 1e-7 >= config.duration {
        *phase = GamePhase::Survived;
    }
}

// Conservative for any heading/tilt at activation, including a moving player.
fn spawn_half(config: &CombatConfig) -> Vec3 {
    Vec3::splat(config.enemy_half_size * 3_f32.sqrt())
}

fn safe(
    position: Vec3,
    half: Vec3,
    arena: &Arena,
    player: &Transform,
    clearance: f32,
    occupied: &[(Entity, Vec3, Vec3)],
    ignore: Option<Entity>,
) -> bool {
    let fits = (position - arena.center())
        .abs()
        .cmple(arena.half_size - half)
        .all();
    let gap =
        ((position - player.translation).abs() - half - drone_world_half_extents(player.rotation))
            .max(Vec3::ZERO)
            .length();
    fits && gap >= clearance
        && occupied.iter().all(|(id, p, h)| {
            Some(*id) == ignore
                || (position - *p)
                    .abs()
                    .cmpgt(half + *h + Vec3::splat(1.))
                    .any()
        })
}

// 4 sides x 5 offsets x 3 altitudes, permuted to alternate sides and heights.
fn candidate(index: usize, arena: &Arena, half: Vec3) -> Vec3 {
    let n = (index % 60) * 37 % 60;
    let side = n % 4;
    let along = ((n / 4) % 5) as f32 / 4. * 1.6 - 0.8;
    let height = (n / 20) as f32 / 2. * 0.7 + 0.15;
    let extent = (arena.half_size - half - Vec3::splat(2.)).max(Vec3::ZERO);
    let mut p = arena.center();
    p.y += (height * 2. - 1.) * extent.y;
    if side < 2 {
        p.x = if side == 0 { -extent.x } else { extent.x };
        p.z = along * extent.z;
    } else {
        p.z = if side == 2 { -extent.z } else { extent.z };
        p.x = along * extent.x;
    }
    p
}

type Occupants<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        Option<&'static Enemy>,
        Option<&'static SpawnWarning>,
    ),
    Or<(With<Enemy>, With<SpawnWarning>)>,
>;

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    mut commands: Commands,
    config: Res<WaveConfig>,
    combat: Res<CombatConfig>,
    arena: Res<Arena>,
    player: Single<&Transform, With<Drone>>,
    phase: Res<GamePhase>,
    mut run: ResMut<Encounter>,
    occupants: Occupants,
) {
    let mut warnings = Vec::new();
    let mut occupied = Vec::new();
    let half = spawn_half(&combat);
    let mut live = 0;
    for (id, transform, enemy, warning) in &occupants {
        if let Some(w) = warning {
            warnings.push((id, transform.translation, w.ready_at));
            occupied.push((id, transform.translation, half));
        } else if enemy.is_some_and(|e| e.health > 0) {
            live += 1;
            occupied.push((
                id,
                transform.translation,
                world_half_extents(transform.rotation, Vec3::splat(combat.enemy_half_size)),
            ));
        }
    }
    warnings.sort_by_key(|(id, _, _)| id.to_bits());
    if *phase == GamePhase::Choosing {
        return;
    }
    if *phase != GamePhase::Playing {
        for (id, _, _) in warnings {
            commands.entity(id).despawn();
        }
        return;
    }
    let mut pending = warnings.len();
    for (id, position, ready_at) in warnings {
        if run.elapsed + 1e-7 < ready_at {
            continue;
        }
        commands.entity(id).despawn();
        pending -= 1;
        if live < config.cap
            && safe(
                position,
                half,
                &arena,
                &player,
                config.clearance,
                &occupied,
                Some(id),
            )
        {
            spawn_enemy(&mut commands, &combat, position, player.translation);
            live += 1;
        } else {
            occupied.retain(|(other, _, _)| *other != id);
        }
    }
    let mut burst = None;
    while let Some(&(at, count)) = config.bursts.get(run.next_burst) {
        if run.elapsed + 1e-7 < at {
            break;
        }
        burst = Some(count);
        run.next_burst += 1;
    }
    let count = burst
        .unwrap_or(0)
        .min(config.cap.saturating_sub(live + pending));
    for _ in 0..count {
        let mut selected = None;
        for _ in 0..60 {
            let position = candidate(run.candidate, &arena, half);
            run.candidate = (run.candidate + 1) % 60;
            if safe(
                position,
                half,
                &arena,
                &player,
                config.clearance,
                &occupied,
                None,
            ) {
                selected = Some(position);
                break;
            }
        }
        let Some(position) = selected else {
            break;
        };
        let id = commands
            .spawn((
                SpawnWarning {
                    ready_at: run.elapsed + config.warning_seconds,
                },
                Transform::from_translation(position),
            ))
            .id();
        occupied.push((id, position, half));
    }
}
