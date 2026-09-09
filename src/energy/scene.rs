use super::*;

pub(crate) struct EnergyScenePlugin;
#[derive(Component)]
struct EnergyHud;
#[derive(Component)]
struct EnergyFill;
#[derive(Component)]
struct FieldVisual(Entity);
#[derive(Resource)]
struct FieldMaterials {
    idle: Handle<StandardMaterial>,
    charging: Handle<StandardMaterial>,
}

impl Plugin for EnergyScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_scene
                .after(super::setup)
                .after(crate::combat::CombatSceneSetup),
        )
        .add_systems(Update, present.in_set(GameplaySet::Presentation));
    }
}

fn setup_scene(
    mut commands: Commands,
    nodes: Query<(Entity, &ChargingNode)>,
    hud_root: Single<Entity, With<crate::combat::CombatHudRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let idle = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.8, 0.85, 0.055),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let charging = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 1., 0.75, 0.13),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let edge = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.6, 0.65),
        unlit: true,
        ..default()
    });
    let core = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.95, 0.85),
        unlit: true,
        ..default()
    });
    let cylinder = meshes.add(Cylinder::new(1., 1.));
    let cube = meshes.add(Cuboid::default());
    for (id, node) in &nodes {
        commands.spawn((
            FieldVisual(id),
            Mesh3d(cylinder.clone()),
            MeshMaterial3d(idle.clone()),
            Transform::from_translation(node.center + Vec3::Y * (node.height / 2.))
                .with_scale(Vec3::new(node.radius, node.height, node.radius)),
        ));
        let ring = meshes.add(Annulus::new(node.radius - 1.5, node.radius));
        for height in [0.3, node.height] {
            commands.spawn((
                Mesh3d(ring.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_translation(node.center + Vec3::Y * height)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ));
        }
        for axis in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_translation(
                    node.center + axis * node.radius + Vec3::Y * (node.height / 2.),
                )
                .with_scale(Vec3::new(0.8, node.height, 0.8)),
            ));
        }
        commands.spawn((
            Mesh3d(cylinder.clone()),
            MeshMaterial3d(core.clone()),
            Transform::from_translation(node.center + Vec3::Y).with_scale(Vec3::new(16., 2., 16.)),
        ));
    }
    commands.insert_resource(FieldMaterials { idle, charging });
    let panel = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                EnergyHud,
                Text::default(),
                TextFont::from_font_size(15.),
                TextColor(Color::srgb(0.75, 0.96, 0.9)),
            ));
            parent
                .spawn((
                    Node {
                        width: px(220),
                        height: px(7),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.09, 0.2, 0.23)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        EnergyFill,
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.9, 0.75)),
                    ));
                });
        })
        .id();
    commands.entity(*hud_root).add_child(panel);
}

fn present(
    energy: Res<Energy>,
    config: Res<EnergyConfig>,
    phase: Res<GamePhase>,
    materials: Res<FieldMaterials>,
    mut hud: Single<&mut Text, With<EnergyHud>>,
    mut fill: Single<(&mut Node, &mut BackgroundColor), With<EnergyFill>>,
    mut fields: Query<(&FieldVisual, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let active = *phase == GamePhase::Playing;
    let state = if energy.overdrive {
        "ON"
    } else if energy.current == 0. {
        "EMPTY"
    } else {
        "OFF"
    };
    let flow = if !active {
        "POWER PAUSED".to_string()
    } else if energy.charging.is_some() {
        let net = config.recharge - if energy.overdrive { config.drain } else { 0. };
        format!("CHARGING +{net:.0}/s")
    } else if energy.overdrive {
        format!("DRAINING -{:.0}/s", config.drain)
    } else {
        "Enter a charging field to recharge".to_string()
    };
    let rejected = if energy.rejected_for > 0. {
        format!(" | Need {:.0} energy to activate", config.activation)
    } else {
        String::new()
    };
    let value = format!(
        "ENERGY  {:.0} / {:.0}   |   OVERDRIVE {state}  [1]\n{flow}{rejected}",
        energy.current.floor(),
        config.capacity
    );
    if hud.0 != value {
        hud.0 = value;
    }
    let (bar, color) = &mut *fill;
    bar.width = percent((energy.current / config.capacity * 100.) as f32);
    color.0 = if energy.current < config.activation {
        Color::srgb(1., 0.55, 0.25)
    } else {
        Color::srgb(0.2, 0.9, 0.75)
    };
    for (field, mut material) in &mut fields {
        let desired = if active && energy.charging == Some(field.0) {
            &materials.charging
        } else {
            &materials.idle
        };
        if material.0 != *desired {
            material.0 = desired.clone();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::tests::{at, step};

    fn text(app: &mut App) -> String {
        app.world_mut()
            .query_filtered::<&Text, With<EnergyHud>>()
            .single(app.world())
            .unwrap()
            .0
            .clone()
    }
    #[test]
    fn hud_tracks_charge_rejection_and_freeze_without_restart_asset_growth() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_plugins((
                crate::arena::ArenaPlugin,
                crate::combat::CombatPlugin,
                crate::combat::CombatScenePlugin,
                EnergyScenePlugin,
            ));
        app.update();
        let drone = app
            .world_mut()
            .query_filtered::<Entity, With<Drone>>()
            .single(app.world())
            .unwrap();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<EnergyHud>>()
                .iter(app.world())
                .count(),
            1
        );
        assert!(text(&mut app).contains("OVERDRIVE OFF"));
        let meshes = app.world().resource::<Assets<Mesh>>().len();
        let materials = app.world().resource::<Assets<StandardMaterial>>().len();
        let entities = app.world().entities().count_spawned();
        app.world_mut().resource_mut::<Energy>().current = 0.;
        step(&mut app, 0., &[KeyCode::Digit1]);
        assert!(text(&mut app).contains("EMPTY"));
        assert!(text(&mut app).contains("Need 10"));
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        step(&mut app, 1., &[]);
        assert!(text(&mut app).contains("CHARGING +25/s"));
        step(&mut app, 0., &[KeyCode::Digit1]);
        assert!(text(&mut app).contains("OVERDRIVE ON"));
        assert!(text(&mut app).contains("CHARGING +15/s"));
        let fill = app
            .world_mut()
            .query_filtered::<&Node, With<EnergyFill>>()
            .single(app.world())
            .unwrap();
        assert_eq!(fill.width, percent(25));
        for _ in 0..3 {
            *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
            step(&mut app, 1., &[]);
            assert!(text(&mut app).contains("POWER PAUSED"));
            step(&mut app, 0., &[KeyCode::KeyR]);
            *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
            step(&mut app, 0., &[]);
            assert!(text(&mut app).contains("100 / 100"));
            assert!(text(&mut app).contains("OVERDRIVE OFF"));
            assert_eq!(app.world().entities().count_spawned(), entities);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
            assert_eq!(
                app.world().resource::<Assets<StandardMaterial>>().len(),
                materials
            );
        }
    }
}
