//! Four persistent beacons reuse meshes and materials across mission attempts.
use super::objectives::{ObjectiveConfig, ObjectiveKind, ObjectiveRun};
use crate::game::{GamePhase, GameplaySet};
use bevy::{camera::CameraUpdateSystems, prelude::*, ui::UiSystems};

pub(crate) struct ObjectiveScenePlugin;
#[derive(Component)]
struct Marker(Option<usize>);
#[derive(Component)]
pub(super) struct MarkerLabel(Option<usize>);
#[derive(Resource)]
struct MarkerAssets {
    scan: Handle<Mesh>,
    cargo: Handle<Mesh>,
    exit: Handle<Mesh>,
    cyan: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
    locked: Handle<StandardMaterial>,
}
impl Plugin for ObjectiveScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, present.in_set(GameplaySet::Presentation))
            .add_systems(
                PostUpdate,
                labels.after(CameraUpdateSystems).before(UiSystems::Prepare),
            );
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = |color| StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    };
    let assets = MarkerAssets {
        scan: meshes.add(Torus::new(22., 28.)),
        cargo: meshes.add(Cuboid::new(32., 32., 32.)),
        exit: meshes.add(Torus::new(85., 95.)),
        cyan: materials.add(material(Color::srgb(0.2, 0.85, 1.))),
        orange: materials.add(material(Color::srgb(1., 0.48, 0.1))),
        green: materials.add(material(Color::srgb(0.3, 1., 0.5))),
        locked: materials.add(material(Color::srgb(0.45, 0.50, 0.53))),
    };
    let ground_ring = meshes.add(Torus::new(62., 67.));
    for index in [Some(0), Some(1), Some(2), None] {
        commands
            .spawn((
                Name::new("Mission objective beacon"),
                Marker(index),
                Mesh3d(assets.scan.clone()),
                MeshMaterial3d(assets.cyan.clone()),
                Transform::default(),
                Visibility::Hidden,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(ground_ring.clone()),
                    MeshMaterial3d(assets.locked.clone()),
                    Transform::from_xyz(0., -87., 0.),
                ));
            });
        commands.spawn((
            MarkerLabel(index),
            Text::default(),
            TextFont::from_font_size(12.),
            TextColor(Color::WHITE),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            BackgroundColor(Color::srgba(0.025, 0.05, 0.065, 0.90)),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                width: px(180),
                height: px(22),
                padding: UiRect::top(px(3)),
                ..default()
            },
        ));
    }
    commands.insert_resource(assets);
}
fn present(
    config: Res<ObjectiveConfig>,
    run: Res<ObjectiveRun>,
    phase: Res<GamePhase>,
    assets: Res<MarkerAssets>,
    mut markers: Query<(
        &Marker,
        &mut Transform,
        &mut Visibility,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let active = !run.survival() && matches!(*phase, GamePhase::Playing | GamePhase::Choosing);
    for (Marker(index), mut transform, mut visibility, mut mesh, mut material) in &mut markers {
        let done = index.is_some_and(|i| run.visited[i]);
        *visibility = if active && !(run.kind == ObjectiveKind::Extraction && done) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = index.map_or(config.extraction, |i| config.sites[i]);
        mesh.0 = if index.is_none() {
            &assets.exit
        } else if run.kind == ObjectiveKind::Extraction {
            &assets.cargo
        } else {
            &assets.scan
        }
        .clone();
        material.0 = if index.is_none() {
            if run.ready() {
                &assets.green
            } else {
                &assets.locked
            }
        } else if done {
            &assets.green
        } else if run.kind == ObjectiveKind::Extraction {
            &assets.orange
        } else {
            &assets.cyan
        }
        .clone();
    }
}
fn labels(
    config: Res<ObjectiveConfig>,
    run: Res<ObjectiveRun>,
    phase: Res<GamePhase>,
    camera: Single<(&Camera, &Transform), With<Camera3d>>,
    mut labels: Query<(&MarkerLabel, &mut Text, &mut Node)>,
) {
    let (camera, transform) = *camera;
    let transform = GlobalTransform::from(*transform);
    for (MarkerLabel(index), mut text, mut node) in &mut labels {
        node.display = Display::None;
        if run.survival()
            || !matches!(*phase, GamePhase::Playing | GamePhase::Choosing)
            || (run.kind == ObjectiveKind::Extraction && index.is_some_and(|i| run.visited[i]))
        {
            continue;
        }
        let target = index.map_or(config.extraction, |i| config.sites[i]) + Vec3::Y * 45.;
        let Ok(screen) = camera.world_to_viewport(&transform, target) else {
            continue;
        };
        let Some(viewport) = camera.logical_viewport_rect() else {
            continue;
        };
        // Keep labels inside the flight area; HUD guidance remains available offscreen.
        if screen.x < viewport.min.x + 90.
            || screen.x > viewport.max.x - 90.
            || screen.y < viewport.min.y + 125.
            || screen.y > viewport.max.y - 150.
        {
            continue;
        }
        node.display = Display::Flex;
        node.left = px(screen.x - 90.);
        node.top = px(screen.y - 11.);

        let value = index.map_or_else(
            || {
                if run.ready() {
                    "EXTRACT / READY".into()
                } else {
                    format!("EXTRACT / LOCKED {}/3", run.count())
                }
            },
            |i| {
                format!(
                    "{} {}{}",
                    if run.kind == ObjectiveKind::Extraction {
                        "CARGO"
                    } else {
                        "SCAN"
                    },
                    i + 1,
                    if run.visited[i] { " / DONE" } else { "" }
                )
            },
        );
        if text.0 != value {
            text.0 = value;
        }
    }
}
