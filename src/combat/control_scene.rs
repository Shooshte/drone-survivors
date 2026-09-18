//! Cached emitter silhouettes and redundant phase/slot telegraphs.
use super::{
    Enemy,
    control::{ControlAttack, ControlEffects, ControlPhase, RANGE},
    feedback::HitFlash,
};
use crate::{arena::Drone, economy::runtime::EnemyKind, game::GamePhase, modules::Modules};
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct ControlAssets {
    slower: Handle<StandardMaterial>,
    jammer: Handle<StandardMaterial>,
    warning: Handle<StandardMaterial>,
    active: Handle<StandardMaterial>,
    recovery: Handle<StandardMaterial>,
    body: Handle<Mesh>,
    bar: Handle<Mesh>,
    ring: Handle<Mesh>,
    line: Handle<Mesh>,
}
#[derive(Component)]
pub(super) enum Cue {
    Ring,
    Line,
}
#[derive(Component)]
pub(super) struct ControlHud;

pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut material = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(ControlAssets {
        slower: material(Color::srgb(0.1, 1., 0.7)),
        jammer: material(Color::srgb(0.7, 0.35, 1.)),
        warning: material(Color::srgb(1., 0.85, 0.1)),
        active: material(Color::srgb(1., 0.35, 0.12)),
        recovery: material(Color::srgb(0.25, 0.5, 1.)),
        body: meshes.add(Sphere::new(14.)),
        bar: meshes.add(Cuboid::new(38., 5., 5.)),
        ring: meshes.add(Torus::new(24., 27.)),
        line: meshes.add(Cuboid::new(1., 1., 1.)),
    });
}

pub(super) fn spawn(
    mut commands: Commands,
    assets: Res<ControlAssets>,
    enemies: Query<(Entity, &Enemy), Added<ControlAttack>>,
) {
    for (id, enemy) in &enemies {
        let jammer = enemy.kind == EnemyKind::Jammer;
        let material = if jammer {
            &assets.jammer
        } else {
            &assets.slower
        };
        commands
            .entity(id)
            .insert((
                Mesh3d(assets.body.clone()),
                MeshMaterial3d(material.clone()),
            ))
            .with_children(|parent| {
                // Crossed horizontal bars versus a vertical aerial remain distinct without color.
                for turn in [0., std::f32::consts::FRAC_PI_2] {
                    let rotation = if jammer {
                        Quat::from_rotation_z(turn)
                    } else {
                        Quat::from_rotation_y(turn)
                    };
                    parent.spawn((
                        Mesh3d(assets.bar.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(0., if jammer { 18. } else { 0. }, 0.)
                            .with_rotation(rotation),
                    ));
                }
                parent.spawn((
                    Cue::Ring,
                    Mesh3d(assets.ring.clone()),
                    MeshMaterial3d(assets.warning.clone()),
                    Transform::default(),
                ));
                parent.spawn((
                    Cue::Line,
                    Mesh3d(assets.line.clone()),
                    MeshMaterial3d(assets.warning.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                ));
            });
    }
}

type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        &'static Enemy,
        &'static Transform,
        &'static ControlAttack,
        Option<&'static HitFlash>,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
    Without<Drone>,
>;
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
    (Without<Enemy>, Without<Drone>),
>;

pub(super) fn present(
    assets: Res<ControlAssets>,
    drone: Single<&Transform, With<Drone>>,
    world: Option<Res<crate::world::WorldGeometry>>,
    mut bodies: Bodies,
    mut cues: Cues,
) {
    for (enemy, _, _, flash, mut material) in &mut bodies {
        if flash.is_none() {
            material.0 = if enemy.kind == EnemyKind::Jammer {
                assets.jammer.clone()
            } else {
                assets.slower.clone()
            };
        }
    }
    for (cue, parent, mut transform, mut visibility, mut material) in &mut cues {
        let Ok((enemy, body, attack, _, _)) = bodies.get(parent.parent()) else {
            continue;
        };
        material.0 = match attack.phase {
            ControlPhase::Windup => assets.warning.clone(),
            ControlPhase::Active => assets.active.clone(),
            ControlPhase::Recovery => assets.recovery.clone(),
            ControlPhase::Approach => {
                if enemy.kind == EnemyKind::Jammer {
                    assets.jammer.clone()
                } else {
                    assets.slower.clone()
                }
            }
        };
        match cue {
            Cue::Ring => {
                let scale = match attack.phase {
                    ControlPhase::Windup => 1. + 0.5 * (attack.elapsed / 1.2).min(1.),
                    ControlPhase::Active => 1.6,
                    ControlPhase::Recovery => 0.75,
                    ControlPhase::Approach => 1.,
                };
                transform.scale = Vec3::splat(scale);
            }
            Cue::Line => {
                let visible = matches!(attack.phase, ControlPhase::Windup | ControlPhase::Active)
                    && body.translation.distance(drone.translation) <= RANGE
                    && world
                        .as_ref()
                        .is_none_or(|w| w.line_clear(body.translation, drone.translation));
                *visibility = if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if visible {
                    let offset = body.rotation.inverse() * (drone.translation - body.translation);
                    transform.translation = offset * 0.5;
                    transform.rotation =
                        Quat::from_rotation_arc(Vec3::Z, offset.try_normalize().unwrap_or(Vec3::Z));
                    let thickness = if attack.phase == ControlPhase::Active {
                        5.
                    } else {
                        1.5
                    };
                    transform.scale = Vec3::new(thickness, thickness, offset.length());
                }
            }
        }
    }
}

pub(super) fn status(
    phase: Res<GamePhase>,
    effects: Res<ControlEffects>,
    modules: Res<Modules>,
    enemies: Query<(&Enemy, &ControlAttack)>,
    mut hud: Query<(&mut Text, &mut Node), With<ControlHud>>,
) {
    let mut messages = Vec::new();
    for kind in [EnemyKind::Slower, EnemyKind::Jammer] {
        let selected = enemies
            .iter()
            .filter(|(e, _)| e.health > 0 && e.kind == kind)
            .min_by_key(|(_, a)| {
                (
                    match a.phase {
                        ControlPhase::Windup => 0,
                        ControlPhase::Active => 1,
                        ControlPhase::Recovery => 2,
                        ControlPhase::Approach => 3,
                    },
                    a.slot,
                )
            });
        if let Some((_, attack)) = selected {
            let name = if kind == EnemyKind::Slower {
                "BEAM"
            } else {
                "JAM"
            };
            let slot = attack
                .slot
                .map_or(String::new(), |i| format!(" [{}]", i + 1));
            let state = match attack.phase {
                ControlPhase::Windup => format!("WARNING {:.1}s", (1.2 - attack.elapsed).max(0.)),
                ControlPhase::Active => "ACTIVE".into(),
                ControlPhase::Recovery => format!("RECOVER {:.1}s", (3. - attack.elapsed).max(0.)),
                ControlPhase::Approach => "APPROACH".into(),
            };
            messages.push(format!("{name}{slot} {state}"));
        }
    }
    let mut text = messages.join(" | ");
    let effect = if effects.slowed {
        "SLOW: 60% horizontal speed. Break sight/range."
    } else {
        "Break sight/range during warning, or destroy the source."
    };
    if !text.is_empty() {
        text.push_str(&format!("\n{effect}"));
    }
    if let Some(slot) = modules.disabled_for.iter().position(|&t| t > 0.) {
        text.push_str(&format!(
            "\nLOCK [{}] {:.1}s; then press {} to enable.",
            slot + 1,
            modules.disabled_for[slot],
            slot + 1
        ));
    } else if modules.jam_grace > 0. {
        text.push_str(&format!(
            "\nJAM IMMUNE {:.1}s; unlocked modules stay OFF.",
            modules.jam_grace
        ));
    }
    for (mut label, mut node) in &mut hud {
        node.display =
            if !text.is_empty() && matches!(*phase, GamePhase::Playing | GamePhase::Choosing) {
                Display::Flex
            } else {
                Display::None
            };
        label.0.clone_from(&text);
    }
}
