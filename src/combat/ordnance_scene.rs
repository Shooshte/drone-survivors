//! Cached carrier/launch silhouettes and player attachment/pulse cues.
use super::{
    Encounter, Enemy, SpawnWarning,
    bombs::BombState,
    feedback::HitFlash,
    mothership::{Mothership, SpawnParent},
};
use crate::{arena::Drone, economy::runtime::EnemyKind, game::GamePhase};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct OrdnanceHud;
#[derive(Component)]
pub(super) struct PlayerCues;
#[derive(Component)]
pub(super) enum Cue {
    Bomb,
    Pulse,
    Launch,
}
#[derive(Resource)]
pub(super) struct Assets {
    body: Handle<Mesh>,
    spike: Handle<Mesh>,
    ring: Handle<Mesh>,
    badge: Handle<Mesh>,
    red: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    cyan: Handle<StandardMaterial>,
    amber: Handle<StandardMaterial>,
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<bevy::asset::Assets<Mesh>>,
    mut materials: ResMut<bevy::asset::Assets<StandardMaterial>>,
) {
    let mut material = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(Assets {
        body: meshes.add(Sphere::new(14.).mesh().ico(0).unwrap()),
        spike: meshes.add(Cuboid::new(5., 5., 18.)),
        ring: meshes.add(Torus::new(23., 26.)),
        badge: meshes.add(Sphere::new(9.)),
        red: material(Color::srgb(1., 0.22, 0.18)),
        white: material(Color::srgb(0.9, 0.95, 1.)),
        cyan: material(Color::srgb(0.15, 1., 0.85)),
        amber: material(Color::srgb(1., 0.8, 0.15)),
    });
}
pub(super) fn spawn(
    mut commands: Commands,
    assets: Res<Assets>,
    roster: Option<Res<super::variants::SpawnRoster>>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
    drones: Query<Entity, (With<Drone>, Without<PlayerCues>)>,
) {
    for (id, enemy) in &enemies {
        if !matches!(enemy.kind, EnemyKind::Bomber | EnemyKind::Mothership) {
            continue;
        }
        let bomber = enemy.kind == EnemyKind::Bomber;
        let material = if bomber { &assets.red } else { &assets.white };
        commands
            .entity(id)
            .insert((
                Mesh3d(assets.body.clone()),
                MeshMaterial3d(material.clone()),
            ))
            .with_children(|parent| {
                if bomber {
                    for direction in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
                        parent.spawn((
                            Mesh3d(assets.spike.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::from_translation(direction * 15.)
                                .with_rotation(Quat::from_rotation_arc(Vec3::Z, direction)),
                        ));
                    }
                } else {
                    for y in [-10., 10.] {
                        parent.spawn((
                            Cue::Launch,
                            Mesh3d(assets.ring.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::from_xyz(0., y, 0.),
                        ));
                    }
                }
            });
    }
    if roster.is_none() {
        return;
    }
    for id in &drones {
        commands
            .entity(id)
            .insert(PlayerCues)
            .with_children(|parent| {
                parent.spawn((
                    Cue::Bomb,
                    Mesh3d(assets.badge.clone()),
                    MeshMaterial3d(assets.red.clone()),
                    Transform::from_xyz(0., 30., 0.),
                    Visibility::Hidden,
                ));
                parent.spawn((
                    Cue::Pulse,
                    Mesh3d(assets.ring.clone()),
                    MeshMaterial3d(assets.cyan.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                ));
            });
    }
}
type Cues<'w, 's> = Query<
    'w,
    's,
    (
        &'static Cue,
        &'static ChildOf,
        &'static mut Transform,
        &'static mut Visibility,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
    Without<Enemy>,
>;
pub(super) fn present(
    assets: Res<Assets>,
    bomb: Res<BombState>,
    modules: Res<crate::modules::ModuleConfig>,
    phase: Res<GamePhase>,
    warnings: Query<&SpawnParent>,
    mut bodies: Query<(
        Entity,
        &Enemy,
        Option<&HitFlash>,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut cues: Cues,
) {
    for (_, enemy, flash, mut material) in &mut bodies {
        if flash.is_none() {
            match enemy.kind {
                EnemyKind::Bomber => material.0 = assets.red.clone(),
                EnemyKind::Mothership => material.0 = assets.white.clone(),
                _ => {}
            }
        }
    }
    for (cue, parent, mut transform, mut visibility, mut material) in &mut cues {
        let visible = match cue {
            Cue::Bomb => {
                let scale = 1.
                    + bomb
                        .remaining
                        .map_or(0., |t| (t * 10.).sin().abs() as f32 * 0.35);
                transform.scale = Vec3::splat(scale);
                bomb.remaining.is_some()
            }
            Cue::Pulse => {
                transform.scale = Vec3::splat(
                    1. + (1. - bomb.pulse_flash as f32 / 0.3)
                        * (modules.repulsor_radius / 26. - 1.),
                );
                bomb.pulse_flash > 0.
            }
            Cue::Launch => {
                let launching = warnings.iter().any(|p| p.0 == parent.parent());
                material.0 = if launching {
                    assets.amber.clone()
                } else {
                    assets.white.clone()
                };
                transform.scale = Vec3::splat(if launching { 1.4 } else { 1. });
                true
            }
        };
        *visibility = if visible && matches!(*phase, GamePhase::Playing | GamePhase::Choosing) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
pub(super) fn status(
    bomb: Res<BombState>,
    run: Res<Encounter>,
    phase: Res<GamePhase>,
    parents: Query<(&Enemy, &Mothership)>,
    warnings: Query<(&SpawnWarning, &SpawnParent)>,
    mut hud: Query<(&mut Text, &mut Node), With<OrdnanceHud>>,
) {
    let mut rows = Vec::new();
    if let Some(fuse) = bomb.remaining {
        rows.push(format!(
            "BOMB {fuse:.1}s / 25 HULL | Repulsor pulse removes; shield blocks."
        ));
    } else if bomb.notice_for > 0. {
        rows.push(bomb.notice.into());
    }
    if let Some((warning, _)) = warnings
        .iter()
        .filter(|(w, _)| !w.cancelled)
        .min_by(|a, b| a.0.ready_at.total_cmp(&b.0.ready_at))
    {
        rows.push(format!(
            "LAUNCH {:?} in {:.1}s | destroy mothership to cancel.",
            warning.kind,
            (warning.ready_at - run.elapsed).max(0.)
        ));
    } else if let Some((_, parent)) = parents.iter().filter(|(e, _)| e.health > 0).min_by(|a, b| {
        a.1.ready_at
            .unwrap_or(f64::MAX)
            .total_cmp(&b.1.ready_at.unwrap_or(f64::MAX))
    }) {
        rows.push(format!(
            "MOTHERSHIP: next {:?} warning in {:.1}s",
            parent.next_kind,
            (parent.ready_at.unwrap_or(run.elapsed + 6.) - run.elapsed).max(0.)
        ));
    }
    let value = rows.join("\n");
    for (mut text, mut node) in &mut hud {
        node.display =
            if !value.is_empty() && matches!(*phase, GamePhase::Playing | GamePhase::Choosing) {
                Display::Flex
            } else {
                Display::None
            };
        text.0.clone_from(&value);
    }
}
