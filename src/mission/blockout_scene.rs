//! Persistent Mission 01 placeholder art and a compact route map.
use super::{
    blockout::{BlockoutRun, DELIVERY_RADIUS, HOLDOUT_RADIUS, Holdout, PICKUP_RADIUS},
    objectives::ObjectiveRun,
};
use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
    world::mission01 as map,
};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct SharedMapVisual;
#[derive(Component)]
struct MapVisual;
#[derive(Component)]
struct MapPanel;
#[derive(Component)]
struct MapPlayer;
#[derive(Component)]
struct MapStatus;
#[derive(Component)]
struct PayloadVisual;
#[derive(Component)]
struct HiddenVisual;
pub(crate) struct BlockoutScenePlugin;
impl Plugin for BlockoutScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, present.after(GameplaySet::Presentation));
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::default());
    let floor = materials.add(Color::srgb(0.055, 0.10, 0.13));
    let terrain = materials.add(Color::srgb(0.19, 0.27, 0.30));
    let mut mat = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    let grid = mat(Color::srgb(0.085, 0.15, 0.18));
    let edge = mat(Color::srgb(0.34, 0.49, 0.50));
    let cyan = mat(Color::srgb(0.25, 0.88, 0.93));
    let gold = mat(Color::srgb(1., 0.63, 0.2));
    let green = mat(Color::srgb(0.3, 0.9, 0.5));
    let purple = mat(Color::srgb(0.8, 0.4, 0.8));
    let size = map::arena().half_size;
    commands.spawn((
        MapVisual,
        Mesh3d(cube.clone()),
        MeshMaterial3d(floor),
        Transform::from_xyz(0., -1., 0.).with_scale(Vec3::new(size.x * 2., 2., size.z * 2.)),
        Visibility::Hidden,
    ));
    for n in -(size.x / 200.) as i32..=(size.x / 200.) as i32 {
        for (position, scale) in [
            (
                Vec3::new(n as f32 * 200., 0.1, 0.),
                Vec3::new(1., 0.2, size.z * 2.),
            ),
            (
                Vec3::new(0., 0.1, n as f32 * 200.),
                Vec3::new(size.x * 2., 0.2, 1.),
            ),
        ] {
            commands.spawn((
                MapVisual,
                Mesh3d(cube.clone()),
                MeshMaterial3d(grid.clone()),
                Transform::from_translation(position).with_scale(scale),
                Visibility::Hidden,
            ));
        }
    }
    let world = map::geometry();
    for solid in &world.solids {
        commands.spawn((
            MapVisual,
            Mesh3d(cube.clone()),
            MeshMaterial3d(terrain.clone()),
            Transform::from_translation(solid.center).with_scale(solid.half * 2.),
            Visibility::Hidden,
        ));
        for sign in [-1., 1.] {
            for (offset, scale) in [
                (
                    Vec3::new(sign * solid.half.x, solid.half.y, 0.),
                    Vec3::new(5., 2., solid.half.z * 2.),
                ),
                (
                    Vec3::new(0., solid.half.y, sign * solid.half.z),
                    Vec3::new(solid.half.x * 2., 2., 5.),
                ),
            ] {
                commands.spawn((
                    MapVisual,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(edge.clone()),
                    Transform::from_translation(solid.center + offset).with_scale(scale),
                    Visibility::Hidden,
                ));
            }
        }
    }
    for sign in [-1., 1.] {
        for (position, scale) in [
            (
                Vec3::new(sign * size.x, 2., 0.),
                Vec3::new(12., 4., size.z * 2.),
            ),
            (
                Vec3::new(0., 2., sign * size.z),
                Vec3::new(size.x * 2., 4., 12.),
            ),
        ] {
            commands.spawn((
                MapVisual,
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_translation(position).with_scale(scale),
                Visibility::Hidden,
            ));
        }
    }
    for (position, radius, material) in [
        (map::pickup(), PICKUP_RADIUS, gold.clone()),
        (map::delivery(), DELIVERY_RADIUS, green.clone()),
        (map::challenge(), HOLDOUT_RADIUS, green.clone()),
    ] {
        let ring = meshes.add(Torus::new(radius - 8., radius));
        commands.spawn((
            MapVisual,
            Mesh3d(ring),
            MeshMaterial3d(material),
            Transform::from_translation(position.with_y(3.)),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        MapVisual,
        PayloadVisual,
        Mesh3d(cube.clone()),
        MeshMaterial3d(gold.clone()),
        Transform::from_translation(map::pickup()).with_scale(Vec3::splat(48.)),
        Visibility::Hidden,
    ));
    commands.spawn((
        MapVisual,
        HiddenVisual,
        Mesh3d(cube.clone()),
        MeshMaterial3d(gold.clone()),
        Transform::from_translation(map::hidden()).with_scale(Vec3::new(55., 30., 55.)),
        Visibility::Hidden,
    ));
    for (_, position) in map::chargers() {
        let ring = meshes.add(Torus::new(86., 90.));
        commands.spawn((
            MapVisual,
            Mesh3d(ring),
            MeshMaterial3d(cyan.clone()),
            Transform::from_translation(position.with_y(3.)),
            Visibility::Hidden,
        ));
        for scale in [Vec3::new(40., 3., 8.), Vec3::new(8., 3., 40.)] {
            commands.spawn((
                MapVisual,
                Mesh3d(cube.clone()),
                MeshMaterial3d(cyan.clone()),
                Transform::from_translation(position.with_y(3.)).with_scale(scale),
                Visibility::Hidden,
            ));
        }
    }
    for (east, south) in [
        (640., 825.),
        (65., 470.),
        (515., 690.),
        (685., 115.),
        (860., 385.),
    ] {
        let ring = meshes.add(Torus::new(170., 180.));
        commands.spawn((
            MapVisual,
            Mesh3d(ring),
            MeshMaterial3d(purple.clone()),
            Transform::from_translation(map::point(east, south).with_y(3.)),
            Visibility::Hidden,
        ));
    }
    for position in [map::point(80., 85.), map::point(960., 65.)] {
        let ring = meshes.add(Torus::new(100., 112.));
        commands.spawn((
            MapVisual,
            Mesh3d(ring),
            MeshMaterial3d(purple.clone()),
            Transform::from_translation(position.with_y(8.)),
            Visibility::Hidden,
        ));
    }
    for field in map::fields() {
        let direction = field.direction.with_y(0.).normalize();
        let side = Vec3::new(-direction.z, 0., direction.x);
        let center = field.bounds.center.with_y(2.);
        for sign in [-1., 1.] {
            for (offset, scale) in [
                (
                    Vec3::X * sign * field.bounds.half.x,
                    Vec3::new(4., 2., field.bounds.half.z * 2.),
                ),
                (
                    Vec3::Z * sign * field.bounds.half.z,
                    Vec3::new(field.bounds.half.x * 2., 2., 4.),
                ),
            ] {
                commands.spawn((
                    MapVisual,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(cyan.clone()),
                    Transform::from_translation(center + offset).with_scale(scale),
                    Visibility::Hidden,
                ));
            }
        }
        for x in -2..=2 {
            for z in -1..=1 {
                let at = center
                    + Vec3::new(
                        x as f32 * field.bounds.half.x / 3.,
                        0.,
                        z as f32 * field.bounds.half.z / 2.,
                    );
                for sign in [-1., 1.] {
                    let tail = at - direction * 65. + side * sign * 45.;
                    let delta = at - tail;
                    commands.spawn((
                        MapVisual,
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(cyan.clone()),
                        Transform::from_translation((at + tail) * 0.5)
                            .with_rotation(Quat::from_rotation_y(delta.x.atan2(delta.z)))
                            .with_scale(Vec3::new(8., 3., delta.length())),
                        Visibility::Hidden,
                    ));
                }
            }
        }
    }
    commands
        .spawn((
            MapPanel,
            Node {
                position_type: PositionType::Absolute,
                right: px(12),
                top: px(135),
                width: px(180),
                height: px(205),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.045, 0.06, 0.93)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("MISSION 01 / N ^"),
                TextFont::from_font_size(11.),
                TextColor(Color::srgb(0.7, 0.85, 0.88)),
            ));
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        aspect_ratio: Some(1.),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.12, 0.14)),
                ))
                .with_children(|map_ui| {
                    for solid in &world.solids {
                        let min = solid.center - solid.half;
                        map_ui.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent((min.x / map::SCALE + 500.) / 10.),
                                top: percent((min.z / map::SCALE + 500.) / 10.),
                                width: percent(solid.half.x * 2. / map::SCALE / 10.),
                                height: percent(solid.half.z * 2. / map::SCALE / 10.),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.39, 0.41)),
                        ));
                    }
                    for (label, east, south, color) in [
                        ("1", 875., 850., Color::WHITE),
                        ("2", 125., 575., Color::srgb(1., 0.65, 0.2)),
                        ("3", 860., 180., Color::srgb(0.4, 1., 0.6)),
                        ("7", 235., 855., Color::srgb(0.4, 1., 0.6)),
                        ("8", 80., 85., Color::srgb(0.95, 0.5, 0.9)),
                        ("11", 950., 65., Color::srgb(0.95, 0.5, 0.9)),
                    ] {
                        map_ui.spawn((
                            Text::new(label),
                            TextFont::from_font_size(11.),
                            TextColor(color),
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(east / 10. - 2.),
                                top: percent(south / 10. - 3.),
                                ..default()
                            },
                        ));
                    }
                    for (_, at) in map::chargers() {
                        map_ui.spawn((
                            Text::new("+"),
                            TextFont::from_font_size(11.),
                            TextColor(Color::srgb(0.3, 1., 1.)),
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent((at.x / map::SCALE + 500.) / 10. - 2.),
                                top: percent((at.z / map::SCALE + 500.) / 10. - 3.),
                                ..default()
                            },
                        ));
                    }
                    map_ui.spawn((
                        MapPlayer,
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(5),
                            height: px(5),
                            ..default()
                        },
                        BackgroundColor(Color::WHITE),
                    ));
                });
            panel.spawn((
                MapStatus,
                Text::default(),
                TextFont::from_font_size(10.),
                TextColor(Color::srgb(0.7, 0.9, 0.83)),
            ));
        });
}
type MapVisuals<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Visibility,
        Option<&'static PayloadVisual>,
        Option<&'static HiddenVisual>,
    ),
    (With<MapVisual>, Without<SharedMapVisual>),
>;
#[allow(clippy::too_many_arguments)]
fn present(
    run: Res<BlockoutRun>,
    objective: Res<ObjectiveRun>,
    phase: Res<GamePhase>,
    drone: Single<&Transform, With<Drone>>,
    windows: Query<&Window>,
    mut shared: Query<&mut Visibility, (With<SharedMapVisual>, Without<MapVisual>)>,
    mut visuals: MapVisuals,
    mut panel: Single<&mut Node, (With<MapPanel>, Without<MapPlayer>)>,
    mut player: Single<&mut Node, (With<MapPlayer>, Without<MapPanel>)>,
    mut status: Single<&mut Text, With<MapStatus>>,
) {
    let active = run.enabled && matches!(*phase, GamePhase::Playing | GamePhase::Choosing);
    if run.enabled || run.is_changed() {
        for mut visibility in &mut shared {
            *visibility = if run.enabled {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
    for (mut visibility, payload, hidden) in &mut visuals {
        *visibility = if active
            && !(payload.is_some() && objective.ready() || hidden.is_some() && run.hidden_collected)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    panel.display = if active { Display::Flex } else { Display::None };
    let compact = windows.iter().next().is_some_and(|w| w.width() < 800.);
    panel.width = px(if compact { 120. } else { 180. });
    panel.height = px(if compact { 155. } else { 220. });
    panel.top = px(if compact { 118. } else { 135. });
    player.left = percent((drone.translation.x / map::SCALE + 500.) / 10.);
    player.top = percent((drone.translation.z / map::SCALE + 500.) / 10.);
    status.0 = match run.holdout {
        Holdout::Available => "7: optional 30s holdout".into(),
        Holdout::Active { elapsed, .. } => {
            format!("7: HOLD {:.0}s", (30. - elapsed).max(0.).ceil())
        }
        Holdout::Forfeited => "7: reward forfeited".into(),
        Holdout::Complete => "7: +5 components".into(),
    };
}
