use super::{Enemy, SpawnWarning};
use bevy::prelude::*;

/// Gameplay facts for consumers later in this update (presentation now, XP later).
/// Cleared before gameplay each frame, including when the encounter is frozen.
#[derive(Resource, Default)]
pub(super) struct CombatOutcomes(pub Vec<CombatOutcome>);

pub(super) enum CombatOutcome {
    Hit {
        entity: Entity,
        position: Vec3,
        killed: bool,
    },
    PlayerDamaged,
}

pub(super) fn clear(mut outcomes: ResMut<CombatOutcomes>) {
    outcomes.0.clear();
}

#[derive(Resource)]
pub(super) struct FeedbackConfig {
    pub hit_seconds: f32,
    pub kill_seconds: f32,
    pub damage_seconds: f32,
    pub effect_cap: usize,
}
impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            hit_seconds: 0.12,
            kill_seconds: 0.35,
            damage_seconds: 0.22,
            effect_cap: 48,
        }
    }
}

#[derive(Component)]
pub(super) struct HitFlash(pub f32);
#[derive(Component)]
pub(super) struct KillEffect {
    pub remaining: f32,
}
#[derive(Resource, Default)]
pub(super) struct DamageCue(pub f32);

#[derive(Resource)]
pub(super) struct FeedbackAssets {
    pub flash: Handle<StandardMaterial>,
    pub warning: Handle<StandardMaterial>,
    pub kill: Handle<StandardMaterial>,
    pub warning_mesh: Handle<Mesh>,
    pub kill_mesh: Handle<Mesh>,
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
    commands.insert_resource(FeedbackAssets {
        flash: material(Color::srgb(1., 0.98, 0.82)),
        warning: material(Color::srgb(0.95, 0.25, 0.7)),
        kill: material(Color::srgb(1., 0.72, 0.25)),
        warning_mesh: meshes.add(Torus::new(19., 23.)),
        kill_mesh: meshes.add(Sphere::new(1.)),
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<FeedbackConfig>,
    assets: Res<FeedbackAssets>,
    base: Res<super::scene::CombatAssets>,
    outcomes: Res<CombatOutcomes>,
    mut cue: ResMut<DamageCue>,
    mut flashes: Query<(Entity, &mut HitFlash, &mut MeshMaterial3d<StandardMaterial>), With<Enemy>>,
    mut effects: Query<(Entity, &mut KillEffect, &mut Transform)>,
    enemies: Query<(), With<Enemy>>,
    warnings: Query<Entity, Added<SpawnWarning>>,
) {
    let dt = time.delta_secs();
    cue.0 = (cue.0 - dt).max(0.);
    if keys.just_pressed(KeyCode::KeyR) {
        cue.0 = 0.;
        return;
    }
    for id in &warnings {
        commands.entity(id).insert((
            Mesh3d(assets.warning_mesh.clone()),
            MeshMaterial3d(assets.warning.clone()),
        ));
    }
    for (id, mut flash, mut material) in &mut flashes {
        flash.0 -= dt;
        if flash.0 <= 0. {
            material.0 = base.enemy_material.clone();
            commands.entity(id).remove::<HitFlash>();
        }
    }
    let mut count = 0;
    for (id, mut effect, mut transform) in &mut effects {
        effect.remaining -= dt;
        if effect.remaining <= 0. {
            commands.entity(id).despawn();
        } else {
            count += 1;
            transform.scale = Vec3::splat(24. * effect.remaining / config.kill_seconds);
        }
    }
    for event in &outcomes.0 {
        match *event {
            CombatOutcome::PlayerDamaged => cue.0 = config.damage_seconds,
            CombatOutcome::Hit {
                entity,
                killed: false,
                ..
            } => {
                if enemies.contains(entity) {
                    commands.entity(entity).insert((
                        HitFlash(config.hit_seconds),
                        MeshMaterial3d(assets.flash.clone()),
                    ));
                }
            }
            CombatOutcome::Hit {
                position,
                killed: true,
                ..
            } => {
                if count < config.effect_cap {
                    commands.spawn((
                        KillEffect {
                            remaining: config.kill_seconds,
                        },
                        Transform::from_translation(position).with_scale(Vec3::splat(24.)),
                        Mesh3d(assets.kill_mesh.clone()),
                        MeshMaterial3d(assets.kill.clone()),
                    ));
                    count += 1;
                }
            }
        }
    }
}
