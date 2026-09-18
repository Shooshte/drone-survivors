//! Cached type silhouettes and attack telegraphs; no gameplay state lives here.
use super::{
    Enemy,
    feedback::HitFlash,
    variants::{RamPhase, Rammer},
};
use crate::economy::runtime::EnemyKind;
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct VariantAssets {
    fast: Handle<StandardMaterial>,
    rammer: Handle<StandardMaterial>,
    warning: Handle<StandardMaterial>,
    charge: Handle<StandardMaterial>,
    retreat: Handle<StandardMaterial>,
    fast_body: Handle<Mesh>,
    rammer_body: Handle<Mesh>,
    fin: Handle<Mesh>,
    ring: Handle<Mesh>,
    pip: Handle<Mesh>,
    line: Handle<Mesh>,
}

#[derive(Component)]
pub(super) enum Cue {
    Ring,
    Pip(u8),
    Line,
}

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
    commands.insert_resource(VariantAssets {
        fast: material(Color::srgb(0.15, 0.9, 1.)),
        rammer: material(Color::srgb(0.9, 0.15, 0.65)),
        warning: material(Color::srgb(1., 0.85, 0.1)),
        charge: material(Color::srgb(1., 0.18, 0.08)),
        retreat: material(Color::srgb(0.2, 0.4, 1.)),
        fast_body: meshes.add(Cuboid::new(16., 16., 28.)),
        rammer_body: meshes.add(Cuboid::new(28., 24., 28.)),
        fin: meshes.add(Cuboid::new(5., 8., 22.)),
        ring: meshes.add(Torus::new(28., 31.)),
        pip: meshes.add(Sphere::new(4.)),
        line: meshes.add(Cuboid::new(2., 2., 1.)),
    });
}

pub(super) fn spawn(
    mut commands: Commands,
    assets: Res<VariantAssets>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
) {
    for (id, enemy) in &enemies {
        match enemy.kind {
            EnemyKind::Chaser | EnemyKind::Slower | EnemyKind::Jammer => {}
            EnemyKind::Fast => {
                commands
                    .entity(id)
                    .insert((
                        Mesh3d(assets.fast_body.clone()),
                        MeshMaterial3d(assets.fast.clone()),
                    ))
                    .with_children(|parent| {
                        for x in [-11., 11.] {
                            parent.spawn((
                                Mesh3d(assets.fin.clone()),
                                MeshMaterial3d(assets.fast.clone()),
                                Transform::from_xyz(x, 0., 0.),
                            ));
                        }
                    });
            }
            EnemyKind::Rammer => {
                commands
                    .entity(id)
                    .insert((
                        Mesh3d(assets.rammer_body.clone()),
                        MeshMaterial3d(assets.rammer.clone()),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Cue::Ring,
                            Mesh3d(assets.ring.clone()),
                            MeshMaterial3d(assets.rammer.clone()),
                            Transform::default(),
                        ));
                        for i in 0..2 {
                            parent.spawn((
                                Cue::Pip(i),
                                Mesh3d(assets.pip.clone()),
                                MeshMaterial3d(assets.warning.clone()),
                                Transform::from_xyz(-7. + f32::from(i) * 14., 18., 0.),
                            ));
                        }
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
    }
}

fn state_material(assets: &VariantAssets, rammer: Option<&Rammer>) -> Handle<StandardMaterial> {
    match rammer.map(|r| r.phase) {
        Some(RamPhase::Windup) => assets.warning.clone(),
        Some(RamPhase::Charge) => assets.charge.clone(),
        Some(RamPhase::Retreat) => assets.retreat.clone(),
        _ => assets.rammer.clone(),
    }
}

type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        &'static Enemy,
        &'static Transform,
        Option<&'static Rammer>,
        Option<&'static HitFlash>,
        &'static mut MeshMaterial3d<StandardMaterial>,
    ),
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
    Without<Enemy>,
>;

pub(super) fn present(assets: Res<VariantAssets>, mut bodies: Bodies, mut cues: Cues) {
    for (enemy, _, rammer, flash, mut material) in &mut bodies {
        if flash.is_none() {
            match enemy.kind {
                EnemyKind::Fast => material.0 = assets.fast.clone(),
                EnemyKind::Rammer => material.0 = assets.rammer.clone(),
                EnemyKind::Chaser | EnemyKind::Slower | EnemyKind::Jammer => {}
            }
        }
        let _ = rammer;
    }
    for (cue, parent, mut transform, mut visibility, mut material) in &mut cues {
        let Ok((_, body, Some(ram), _, _)) = bodies.get(parent.parent()) else {
            continue;
        };
        match cue {
            Cue::Ring => {
                material.0 = state_material(&assets, Some(ram));
            }
            Cue::Pip(index) => {
                *visibility = if *index < 2 - ram.impacts {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
            Cue::Line => {
                let aiming = matches!(ram.phase, RamPhase::Windup | RamPhase::Charge);
                *visibility = if aiming {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if aiming {
                    let offset = body.rotation.inverse() * (ram.target - body.translation);
                    transform.translation = offset * 0.5;
                    transform.rotation =
                        Quat::from_rotation_arc(Vec3::Z, offset.try_normalize().unwrap_or(Vec3::Z));
                    transform.scale = Vec3::new(1., 1., offset.length());
                    material.0 = state_material(&assets, Some(ram));
                }
            }
        }
    }
}
