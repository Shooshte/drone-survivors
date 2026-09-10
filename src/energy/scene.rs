use super::*;

pub(crate) struct EnergyScenePlugin;
#[derive(Component)]
struct EnergyHud;
#[derive(Component)]
struct EnergyFill;
#[derive(Component)]
struct ChargerHud(Entity);
#[derive(Component)]
struct FieldVisual(Entity);
#[derive(Component)]
struct WorldReserveFill(Entity);
#[derive(Component)]
struct WorldReserveOutline;
#[derive(Resource)]
struct FieldMaterials {
    idle: Handle<StandardMaterial>,
    charging: Handle<StandardMaterial>,
}

const WORLD_RESERVE_WIDTH: f32 = 52.;
const WORLD_RESERVE_DEPTH: f32 = 10.;
const WORLD_RESERVE_EDGE: f32 = 2.;
const WORLD_RESERVE_Z: f32 = 48.;

impl Plugin for EnergyScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_scene
                .after(super::setup)
                .after(crate::combat::CombatSceneSetup),
        )
        .add_systems(
            Update,
            (present, crate::modules::scene::present).in_set(GameplaySet::Presentation),
        );
    }
}

fn setup_scene(
    mut commands: Commands,
    nodes: Query<(Entity, &ChargingNode)>,
    hud_root: Single<Entity, With<crate::combat::CombatHudRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chargers = nodes
        .iter()
        .map(|(entity, node)| (entity, *node))
        .collect::<Vec<_>>();
    chargers.sort_by(|a, b| a.1.center.x.total_cmp(&b.1.center.x));
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
    for (id, node) in &chargers {
        commands.spawn((
            FieldVisual(*id),
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
        let gauge_center = node.center + Vec3::new(0., 2., WORLD_RESERVE_Z);
        commands.spawn((
            WorldReserveFill(*id),
            Mesh3d(cube.clone()),
            MeshMaterial3d(core.clone()),
            Transform::from_translation(gauge_center).with_scale(Vec3::new(
                WORLD_RESERVE_WIDTH,
                1.5,
                WORLD_RESERVE_DEPTH - 4.,
            )),
        ));
        for z in [-1., 1.] {
            commands.spawn((
                WorldReserveOutline,
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_translation(
                    gauge_center
                        + Vec3::Z * z * (WORLD_RESERVE_DEPTH / 2. + WORLD_RESERVE_EDGE / 2.),
                )
                .with_scale(Vec3::new(
                    WORLD_RESERVE_WIDTH + WORLD_RESERVE_EDGE * 2.,
                    2.,
                    WORLD_RESERVE_EDGE,
                )),
            ));
        }
        for x in [-1., 1.] {
            commands.spawn((
                WorldReserveOutline,
                Mesh3d(cube.clone()),
                MeshMaterial3d(edge.clone()),
                Transform::from_translation(
                    gauge_center
                        + Vec3::X * x * (WORLD_RESERVE_WIDTH / 2. + WORLD_RESERVE_EDGE / 2.),
                )
                .with_scale(Vec3::new(WORLD_RESERVE_EDGE, 2., WORLD_RESERVE_DEPTH)),
            ));
        }
    }
    commands.insert_resource(FieldMaterials { idle, charging });
    let panel = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
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
            for (entity, _) in &chargers {
                parent.spawn((
                    ChargerHud(*entity),
                    Text::default(),
                    TextFont::from_font_size(13.),
                    TextColor(Color::srgb(0.75, 0.96, 0.9)),
                ));
            }
            parent.spawn((
                Text::new("MODULES | current / enabled drain"),
                TextFont::from_font_size(13.),
                TextColor(Color::srgb(0.6, 0.7, 0.75)),
            ));
            for index in 0..4 {
                parent.spawn((
                    crate::modules::scene::ModuleHud(index),
                    Text::default(),
                    TextFont::from_font_size(15.),
                    TextColor::default(),
                ));
            }
        })
        .id();
    commands.entity(*hud_root).add_child(panel);
}

fn charger_label(center: Vec3) -> &'static str {
    if center.x < 0. { "LEFT" } else { "RIGHT" }
}

fn charger_status(reserve: &ChargerReserve, config: &ChargerConfig, active: bool) -> String {
    let status = if reserve.occupied {
        if reserve.remaining <= 0. {
            "DEPLETED | LEAVE TO RECOVER".to_string()
        } else {
            "IN USE".to_string()
        }
    } else if reserve.remaining >= config.capacity {
        "READY".to_string()
    } else if reserve.away_seconds < config.recovery_delay {
        format!(
            "RECOVER IN {:.0}s",
            (config.recovery_delay - reserve.away_seconds).ceil()
        )
    } else {
        "RECOVERING".to_string()
    };
    if active {
        status
    } else {
        format!("{status} / PAUSED")
    }
}

#[allow(clippy::too_many_arguments)]
fn present(
    energy: Res<Energy>,
    config: Res<EnergyConfig>,
    charger_config: Res<ChargerConfig>,
    phase: Res<GamePhase>,
    modules: Res<Modules>,
    module_config: Res<ModuleConfig>,
    materials: Res<FieldMaterials>,
    mut hud: Single<&mut Text, (With<EnergyHud>, Without<ChargerHud>)>,
    mut fill: Single<(&mut Node, &mut BackgroundColor), With<EnergyFill>>,
    mut fields: Query<(&FieldVisual, &mut MeshMaterial3d<StandardMaterial>)>,
    chargers: Query<(&ChargingNode, &ChargerReserve)>,
    mut charger_hud: Query<(&ChargerHud, &mut Text, &mut TextColor), Without<EnergyHud>>,
    mut world_fill: Query<(&WorldReserveFill, &mut Transform)>,
) {
    let active = *phase == GamePhase::Playing;
    let drain = modules.drain(&module_config);
    let flow = if !active {
        "POWER PAUSED".to_string()
    } else if energy.charging.is_some() {
        if energy.current >= config.capacity && config.recharge >= drain {
            "BATTERY FULL | IN CHARGING FIELD".to_string()
        } else {
            let net = config.recharge - drain;
            format!("CHARGING {net:+.0}/s net")
        }
    } else if drain > 0. {
        format!("DRAINING -{drain:.0}/s")
    } else if chargers
        .iter()
        .any(|(_, reserve)| reserve.occupied && reserve.remaining <= 0.)
    {
        "CHARGER DEPLETED | LEAVE FIELD TO RECOVER".to_string()
    } else {
        "Enter a charging field to recharge".to_string()
    };
    let actual_drain = if active { drain } else { 0. };
    let value = format!(
        "ENERGY  {:.0} / {:.0}  |  MODULE DRAIN {actual_drain:.0}/s\n{flow}",
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
    for (marker, mut text, mut color) in &mut charger_hud {
        let Ok((node, reserve)) = chargers.get(marker.0) else {
            continue;
        };
        let label = charger_label(node.center);
        let value = format!(
            "{label:<5}  {:.0} / {:.0}   {}",
            reserve.remaining.floor(),
            charger_config.capacity,
            charger_status(reserve, &charger_config, active)
        );
        if text.0 != value {
            text.0 = value;
        }
        color.0 = if !active {
            Color::srgb(0.6, 0.7, 0.75)
        } else if reserve.occupied && reserve.remaining <= 0. {
            Color::srgb(1., 0.55, 0.25)
        } else if reserve.occupied || reserve.away_seconds >= charger_config.recovery_delay {
            Color::srgb(0.2, 0.9, 0.75)
        } else {
            Color::srgb(0.75, 0.96, 0.9)
        };
    }
    for (marker, mut transform) in &mut world_fill {
        let Ok((node, reserve)) = chargers.get(marker.0) else {
            continue;
        };
        let ratio = if charger_config.capacity > 0. {
            (reserve.remaining / charger_config.capacity).clamp(0., 1.) as f32
        } else {
            0.
        };
        let width = WORLD_RESERVE_WIDTH * ratio;
        transform.scale.x = width;
        transform.translation.x = node.center.x - WORLD_RESERVE_WIDTH / 2. + width / 2.;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::tests::{at, step};

    fn text(app: &mut App) -> String {
        let mut values: Vec<String> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        values.sort();
        values.join("\n")
    }
    fn scene_app() -> (App, Entity) {
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
        (app, drone)
    }

    #[test]
    fn full_battery_in_field_reports_full_instead_of_positive_flow() {
        for overdrive in [false, true] {
            let (mut app, drone) = scene_app();
            at(&mut app, drone, Vec3::new(-280., 90., 0.));
            step(
                &mut app,
                0.1,
                if overdrive { &[KeyCode::Digit1] } else { &[] },
            );
            assert_eq!(app.world().resource::<Energy>().current, 100.);
            assert!(text(&mut app).contains("BATTERY FULL"));
            assert!(text(&mut app).contains("IN CHARGING FIELD"));
            assert!(!text(&mut app).contains("CHARGING +"));
            assert_eq!(
                app.world()
                    .resource::<Modules>()
                    .active(crate::modules::ModuleKind::Overdrive),
                overdrive
            );

            app.world_mut().resource_mut::<Energy>().current = 99.9;
            step(&mut app, 0., &[]);
            let expected = if overdrive {
                "CHARGING +15/s"
            } else {
                "CHARGING +25/s"
            };
            assert!(text(&mut app).contains(expected));
            step(&mut app, 0.01, &[]);
            assert!(text(&mut app).contains("BATTERY FULL"));

            *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
            step(&mut app, 0., &[]);
            assert!(text(&mut app).contains("POWER PAUSED"));
            *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
            at(&mut app, drone, Vec3::new(0., 90., 0.));
            step(&mut app, 0., &[]);
            assert!(!text(&mut app).contains("IN CHARGING FIELD"));
            if overdrive {
                assert!(text(&mut app).contains("DRAINING -10/s"));
            }
        }
    }

    #[test]
    fn hud_tracks_charge_rejection_and_freeze_without_restart_asset_growth() {
        let (mut app, drone) = scene_app();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<EnergyHud>>()
                .iter(app.world())
                .count(),
            1
        );
        assert!(text(&mut app).contains("OVERDRIVE OFF"));
        assert!(!text(&mut app).contains("-0/s"));
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
            assert!(!text(&mut app).contains("-0/s"));
            assert_eq!(app.world().entities().count_spawned(), entities);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
            assert_eq!(
                app.world().resource::<Assets<StandardMaterial>>().len(),
                materials
            );
        }
    }
    #[test]
    fn all_slots_show_independent_states_and_signed_net_drain() {
        let (mut app, drone) = scene_app();
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        step(&mut app, 1., &crate::modules::SLOT_KEYS);
        let value = text(&mut app);
        for name in ["OVERDRIVE ON", "SHIELD ON", "MOBILITY ON", "ROCKETS ON"] {
            assert!(value.contains(name), "{value}");
        }
        assert!(value.contains("CHARGING -11/s"));
        assert!(value.contains("MODULE DRAIN 36/s"));
        assert!(value.contains("READY 1"));
        app.world_mut()
            .resource_mut::<Modules>()
            .block(&ModuleConfig::default());
        step(&mut app, 1., &[]);
        assert!(text(&mut app).contains("4.0s recharge"));
        step(&mut app, 0., &[KeyCode::Digit2]);
        assert!(text(&mut app).contains("4.0s paused"));
        assert!(text(&mut app).contains("SHIELD OFF"));
    }

    #[test]
    fn charger_rows_name_both_full_reserves_as_ready() {
        let (mut app, _) = scene_app();
        let value = text(&mut app);
        assert!(value.contains("LEFT   200 / 200   READY"), "{value}");
        assert!(value.contains("RIGHT  200 / 200   READY"), "{value}");
    }

    #[test]
    fn charger_rows_report_use_depletion_delay_recovery_and_pause() {
        let (mut app, drone) = scene_app();
        let mut chargers = app
            .world_mut()
            .query::<(Entity, &ChargingNode)>()
            .iter(app.world())
            .map(|(entity, node)| (entity, node.center.x))
            .collect::<Vec<_>>();
        chargers.sort_by(|a, b| a.1.total_cmp(&b.1));
        let left = chargers[0].0;

        app.world_mut().resource_mut::<Energy>().current = 0.;
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        step(&mut app, 2., &[]);
        let value = text(&mut app);
        assert!(value.contains("LEFT   150 / 200   IN USE"), "{value}");
        assert!(value.contains("RIGHT  200 / 200   READY"), "{value}");

        at(&mut app, drone, Vec3::new(0., 90., 0.));
        step(&mut app, 2., &[]);
        let value = text(&mut app);
        assert!(
            value.contains("LEFT   150 / 200   RECOVER IN 6s"),
            "{value}"
        );

        app.world_mut().entity_mut(left).insert(ChargerReserve {
            remaining: 0.,
            away_seconds: 0.,
            occupied: false,
        });
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        step(&mut app, 0., &[]);
        let value = text(&mut app);
        assert!(
            value.contains("LEFT   0 / 200   DEPLETED | LEAVE TO RECOVER"),
            "{value}"
        );
        assert!(
            value.contains("CHARGER DEPLETED | LEAVE FIELD TO RECOVER"),
            "{value}"
        );
        assert!(!value.contains("Enter a charging field to recharge"));

        at(&mut app, drone, Vec3::new(0., 90., 0.));
        app.world_mut().entity_mut(left).insert(ChargerReserve {
            remaining: 50.,
            away_seconds: 8.,
            occupied: false,
        });
        step(&mut app, 1., &[]);
        let value = text(&mut app);
        assert!(value.contains("LEFT   60 / 200   RECOVERING"), "{value}");

        app.world_mut().entity_mut(left).insert(ChargerReserve {
            remaining: 60.,
            away_seconds: 4.,
            occupied: false,
        });
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
        step(&mut app, 3., &[]);
        let value = text(&mut app);
        assert!(
            value.contains("LEFT   60 / 200   RECOVER IN 4s / PAUSED"),
            "{value}"
        );
    }

    #[test]
    fn world_reserve_fills_shrink_inside_persistent_outlines() {
        let (mut app, _) = scene_app();
        let mut chargers = app
            .world_mut()
            .query::<(Entity, &ChargingNode)>()
            .iter(app.world())
            .map(|(entity, node)| (entity, node.center))
            .collect::<Vec<_>>();
        chargers.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
        let (left, center) = chargers[0];

        let fills = app
            .world_mut()
            .query::<(&WorldReserveFill, &Transform)>()
            .iter(app.world())
            .map(|(marker, transform)| (marker.0, *transform))
            .collect::<Vec<_>>();
        assert_eq!(fills.len(), 2);
        assert!(fills.iter().all(|(_, transform)| transform.scale.x == 52.));

        app.world_mut().entity_mut(left).insert(ChargerReserve {
            remaining: 50.,
            away_seconds: 0.,
            occupied: false,
        });
        step(&mut app, 0., &[]);
        let (_, quarter) = app
            .world_mut()
            .query::<(&WorldReserveFill, &Transform)>()
            .iter(app.world())
            .find(|(marker, _)| marker.0 == left)
            .map(|(marker, transform)| (marker.0, *transform))
            .unwrap();
        assert_eq!(quarter.scale.x, 13.);
        assert_eq!(quarter.translation.x, center.x - 19.5);

        app.world_mut().entity_mut(left).insert(ChargerReserve {
            remaining: 0.,
            away_seconds: 0.,
            occupied: false,
        });
        step(&mut app, 0., &[]);
        let empty = app
            .world_mut()
            .query::<(&WorldReserveFill, &Transform)>()
            .iter(app.world())
            .find(|(marker, _)| marker.0 == left)
            .unwrap()
            .1;
        assert_eq!(empty.scale.x, 0.);
        assert_eq!(
            app.world_mut()
                .query::<&WorldReserveOutline>()
                .iter(app.world())
                .count(),
            8
        );
    }
}
