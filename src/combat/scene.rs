use super::feedback::{self, DamageCue, FeedbackConfig};
use super::{
    CombatConfig, Encounter, Enemy, PlayerHealth, Projectile, SpawnWarning, WaveConfig,
    waves::WaveStatus,
};
use crate::game::{GamePhase, GameplaySet};
use bevy::prelude::*;

pub(crate) struct CombatScenePlugin;

#[derive(Component)]
pub(super) struct CombatHud;

#[derive(Component)]
pub(crate) struct CombatHudRoot;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CombatSceneSetup;

#[derive(Resource)]
pub(super) struct CombatAssets {
    enemy_mesh: Handle<Mesh>,
    pub(super) enemy_material: Handle<StandardMaterial>,
    projectile_mesh: Handle<Mesh>,
    projectile_material: Handle<StandardMaterial>,
    rocket_mesh: Handle<Mesh>,
    rocket_material: Handle<StandardMaterial>,
}

impl Plugin for CombatScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeedbackConfig>()
            .init_resource::<DamageCue>()
            .add_systems(
                Startup,
                (
                    setup.in_set(CombatSceneSetup),
                    feedback::setup,
                    super::variant_scene::setup,
                    super::control_scene::setup,
                    super::ordnance_scene::setup,
                ),
            )
            .add_systems(
                Update,
                (
                    add_visuals,
                    super::variant_scene::spawn,
                    super::control_scene::spawn,
                    super::ordnance_scene::spawn,
                    feedback::update,
                    super::variant_scene::present,
                    super::control_scene::present,
                    super::control_scene::status,
                    super::ordnance_scene::present,
                    super::ordnance_scene::status,
                    update_hud,
                )
                    .chain()
                    .in_set(GameplaySet::Presentation),
            );
    }
}

fn setup(
    mut commands: Commands,
    config: Res<CombatConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(CombatAssets {
        rocket_mesh: meshes.add(Cuboid::new(7., 7., 20.)),
        rocket_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.8, 1.),
            unlit: true,
            ..default()
        }),
        enemy_mesh: meshes.add(Cuboid::from_size(Vec3::splat(config.enemy_half_size * 2.))),
        enemy_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.26, 0.13),
            perceptual_roughness: 0.65,
            ..default()
        }),
        projectile_mesh: meshes.add(Sphere::new(config.projectile_radius)),
        projectile_material: materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0.9, 0.38),
            unlit: true,
            ..default()
        }),
    });
    commands
        .spawn((
            CombatHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                left: px(12),
                right: px(12),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                CombatHud,
                Text::default(),
                crate::arena::FooterFont::new(16., 14.),
                TextFont::from_font_size(16.),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Node {
                    width: percent(100),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.94, 0.92)),
            ));
            parent.spawn((
                super::control_scene::ControlHud,
                Text::default(),
                crate::arena::FooterFont::new(14., 12.),
                TextFont::from_font_size(14.),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Node {
                    width: percent(100),
                    display: Display::None,
                    ..default()
                },
                TextColor(Color::srgb(1., 0.88, 0.5)),
            ));
            parent.spawn((
                super::ordnance_scene::OrdnanceHud,
                Text::default(),
                crate::arena::FooterFont::new(14., 12.),
                TextFont::from_font_size(14.),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                Node {
                    width: percent(100),
                    display: Display::None,
                    ..default()
                },
                TextColor(Color::srgb(1., 0.88, 0.5)),
            ));
        });
}

fn add_visuals(
    mut commands: Commands,
    assets: Res<CombatAssets>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
    mut projectiles: Query<
        (
            Entity,
            &Projectile,
            &mut Transform,
            Option<&super::rockets::Rocket>,
        ),
        Added<Projectile>,
    >,
) {
    // Handles survive restarts, so firing and resetting allocate no new assets.
    for (entity, enemy) in &enemies {
        if enemy.kind != crate::economy::runtime::EnemyKind::Chaser {
            continue;
        }
        commands.entity(entity).insert((
            Mesh3d(assets.enemy_mesh.clone()),
            MeshMaterial3d(assets.enemy_material.clone()),
        ));
    }
    for (id, projectile, mut transform, rocket) in &mut projectiles {
        let (mesh, material) = if rocket.is_some() {
            (&assets.rocket_mesh, &assets.rocket_material)
        } else {
            (&assets.projectile_mesh, &assets.projectile_material)
        };
        commands
            .entity(id)
            .insert((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone())));
        if rocket.is_some() {
            transform.rotation =
                Quat::from_rotation_arc(Vec3::NEG_Z, projectile.velocity.normalize_or_zero());
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_hud(
    config: Res<CombatConfig>,
    health: Res<PlayerHealth>,
    phase: Res<GamePhase>,
    run: Res<Encounter>,
    waves: Res<WaveConfig>,
    time: Res<Time>,
    cue: Res<DamageCue>,
    objective: Option<Res<crate::mission::objectives::ObjectiveRun>>,
    targets: Option<Res<crate::mission::objectives::ObjectiveConfig>>,
    drone: Single<&Transform, With<crate::arena::Drone>>,
    enemies: Query<(), With<Enemy>>,
    warnings: Query<(), With<SpawnWarning>>,
    mut hud: Single<(&mut Text, &mut TextColor), With<CombatHud>>,
) {
    let count = enemies.iter().count();
    let mut status = if *phase == GamePhase::Dead {
        "DRONE DESTROYED | R to restart".to_string()
    } else if *phase == GamePhase::Survived {
        "SURVIVED | R to replay".to_string()
    } else if *phase == GamePhase::Choosing {
        "UPGRADE CHOICE | GAMEPLAY PAUSED".to_string()
    } else if waves.status_at(run.elapsed) == Some(WaveStatus::Lull) {
        "SPAWNING LULL | Keep moving".to_string()
    } else if let Some(wave_phase) = waves.phase_at(run.elapsed) {
        format!("{} | AUTO FIRE", wave_phase.label)
    } else {
        "AUTO FIRE".to_string()
    };
    let survival = objective.as_ref().is_none_or(|o| o.survival());
    if *phase == GamePhase::Playing
        && !survival
        && let (Some(objective), Some(targets)) = (&objective, &targets)
    {
        status = objective.guidance(targets, drone.translation);
    }
    let timer = if survival {
        format!("TIME {:.0}", (waves.duration - run.elapsed).max(0.).ceil())
    } else {
        format!("ELAPSED {:.0}", run.elapsed.floor())
    };
    let protection =
        if *phase == GamePhase::Playing && time.elapsed_secs_f64() < health.invulnerable_until {
            " | HULL PROTECTED"
        } else {
            ""
        };
    let incoming = if *phase == GamePhase::Playing && !warnings.is_empty() {
        format!(" | INCOMING {}", warnings.iter().count())
    } else {
        String::new()
    };
    let value = format!(
        "HULL {} / {} | {} | HOSTILES {} | KILLS {}\n{}{}{}",
        health.current, config.player_health, timer, count, run.kills, status, protection, incoming
    );
    let (text, color) = &mut *hud;
    if text.0 != value {
        text.0 = value;
    }
    color.0 = if *phase == GamePhase::Dead || cue.0 > 0. {
        Color::srgb(1., 0.45, 0.3)
    } else if *phase == GamePhase::Playing && time.elapsed_secs_f64() < health.invulnerable_until {
        Color::srgb(0.3, 0.95, 1.)
    } else {
        Color::srgb(0.9, 0.94, 0.92)
    };
}
