use super::{CombatConfig, Encounter, Enemy, PlayerHealth, Projectile, WaveConfig};
use crate::game::{GamePhase, GameplaySet};
use bevy::prelude::*;

pub(crate) struct CombatScenePlugin;

#[derive(Component)]
pub(super) struct CombatHud;

#[derive(Resource)]
struct CombatAssets {
    enemy_mesh: Handle<Mesh>,
    enemy_material: Handle<StandardMaterial>,
    projectile_mesh: Handle<Mesh>,
    projectile_material: Handle<StandardMaterial>,
}

impl Plugin for CombatScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (add_visuals, update_hud).in_set(GameplaySet::Presentation),
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
    commands.spawn((
        CombatHud,
        Text::default(),
        TextFont::from_font_size(18.),
        TextColor(Color::srgb(0.9, 0.94, 0.92)),
        Node {
            position_type: PositionType::Absolute,
            top: px(52),
            left: px(24),
            right: px(24),
            ..default()
        },
    ));
}

fn add_visuals(
    mut commands: Commands,
    assets: Res<CombatAssets>,
    enemies: Query<Entity, Added<Enemy>>,
    projectiles: Query<Entity, Added<Projectile>>,
) {
    // Handles survive restarts, so firing and resetting allocate no new assets.
    for entity in &enemies {
        commands.entity(entity).insert((
            Mesh3d(assets.enemy_mesh.clone()),
            MeshMaterial3d(assets.enemy_material.clone()),
        ));
    }
    for entity in &projectiles {
        commands.entity(entity).insert((
            Mesh3d(assets.projectile_mesh.clone()),
            MeshMaterial3d(assets.projectile_material.clone()),
        ));
    }
}

fn update_hud(
    config: Res<CombatConfig>,
    health: Res<PlayerHealth>,
    phase: Res<GamePhase>,
    run: Res<Encounter>,
    waves: Res<WaveConfig>,
    enemies: Query<(), With<Enemy>>,
    mut hud: Single<(&mut Text, &mut TextColor), With<CombatHud>>,
) {
    let count = enemies.iter().count();
    let status = if *phase == GamePhase::Dead {
        "DRONE DESTROYED | R to restart"
    } else if *phase == GamePhase::Survived {
        "SURVIVED | R to replay"
    } else if count == 0 {
        "SPAWNING LULL | Keep moving"
    } else {
        "AUTO FIRE | Keep moving"
    };
    let value = format!(
        "HULL  {} / {}   |   HOSTILES  {}   |   KILLS  {}   |   TIME  {:.0}\n{}",
        health.current,
        config.player_health,
        count,
        run.kills,
        (waves.duration - run.elapsed).max(0.).ceil(),
        status
    );
    let (text, color) = &mut *hud;
    if text.0 != value {
        text.0 = value;
    }
    color.0 = if *phase == GamePhase::Dead {
        Color::srgb(1., 0.45, 0.3)
    } else {
        Color::srgb(0.9, 0.94, 0.92)
    };
}
