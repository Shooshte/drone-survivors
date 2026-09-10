use super::{
    WorldGeometry,
    hazard::{HazardPhase, HazardState},
};
use crate::{
    arena::{ArenaSceneSetup, FooterFont, HazardFooterSlot},
    game::{GamePhase, GameplaySet},
};
use bevy::prelude::*;

pub(crate) struct WorldScenePlugin;
#[derive(Component)]
struct HazardHud;
#[derive(Component)]
struct HazardVisual;
#[derive(Component)]
struct HazardBeam(usize);
#[derive(Component)]
struct WorldVisual;
#[derive(Resource)]
struct HazardMaterials {
    volumes: [Handle<StandardMaterial>; 3],
    lines: [Handle<StandardMaterial>; 3],
}

impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(ArenaSceneSetup))
            .add_systems(Update, present.in_set(GameplaySet::Presentation));
    }
}
fn setup(
    mut commands: Commands,
    world: Res<WorldGeometry>,
    footer_slot: Single<Entity, With<HazardFooterSlot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::default());
    let low = materials.add(Color::srgb(0.22, 0.29, 0.33));
    let tall = materials.add(StandardMaterial {
        base_color: Color::srgba(0.43, 0.57, 0.65, 0.12),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let edge = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.65, 0.70),
        unlit: true,
        ..default()
    });
    for solid in &world.solids {
        commands.spawn((
            WorldVisual,
            Mesh3d(cube.clone()),
            MeshMaterial3d(if solid.half.y > 100. {
                tall.clone()
            } else {
                low.clone()
            }),
            Transform::from_translation(solid.center).with_scale(solid.half * 2.),
        ));
        for axis in 0..3 {
            let a = (axis + 1) % 3;
            let b = (axis + 2) % 3;
            for sa in [-1., 1.] {
                for sb in [-1., 1.] {
                    let mut center = solid.center;
                    center[a] += sa * solid.half[a];
                    center[b] += sb * solid.half[b];
                    let mut scale = Vec3::splat(1.5);
                    scale[axis] = solid.half[axis] * 2.;
                    commands.spawn((
                        WorldVisual,
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(edge.clone()),
                        Transform::from_translation(center).with_scale(scale),
                    ));
                }
            }
        }
    }
    let colors = [
        Color::srgb(0.80, 0.60, 0.18),
        Color::srgb(1., 0.65, 0.12),
        Color::srgb(1., 0.22, 0.10),
    ];
    let volumes = std::array::from_fn(|i| {
        materials.add(StandardMaterial {
            base_color: colors[i].with_alpha([0.025, 0.055, 0.12][i]),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })
    });
    let lines = std::array::from_fn(|i| {
        materials.add(StandardMaterial {
            base_color: colors[i],
            unlit: true,
            ..default()
        })
    });
    if let Some(field) = world.hazard {
        commands.spawn((
            WorldVisual,
            HazardVisual,
            Mesh3d(cube.clone()),
            MeshMaterial3d(volumes[0].clone()),
            Transform::from_translation(field.center).with_scale(field.half * 2.),
        ));
        for x in [-1., 1.] {
            for z in [-1., 1.] {
                let center = field.center + Vec3::new(x * field.half.x, 0., z * field.half.z);
                commands.spawn((
                    WorldVisual,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(lines[0].clone()),
                    Transform::from_translation(center).with_scale(Vec3::new(
                        3.,
                        field.half.y * 2.,
                        3.,
                    )),
                ));
                for y in [-1., 1.] {
                    let base = center + Vec3::Y * (y * (field.half.y - 3.));
                    commands.spawn((
                        WorldVisual,
                        Mesh3d(cube.clone()),
                        MeshMaterial3d(edge.clone()),
                        Transform::from_translation(base).with_scale(Vec3::new(12., 6., 12.)),
                    ));
                }
            }
        }
        for y in [-1., 1.] {
            for z in [-1., 1.] {
                commands.spawn((
                    WorldVisual,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(lines[0].clone()),
                    Transform::from_translation(
                        field.center + Vec3::new(0., y * (field.half.y - 0.5), z * field.half.z),
                    )
                    .with_scale(Vec3::new(field.half.x * 2., 1., 2.)),
                ));
            }
            for x in [-1., 1.] {
                commands.spawn((
                    WorldVisual,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(lines[0].clone()),
                    Transform::from_translation(
                        field.center + Vec3::new(x * field.half.x, y * (field.half.y - 0.5), 0.),
                    )
                    .with_scale(Vec3::new(2., 1., field.half.z * 2.)),
                ));
            }
        }
        for i in 0..7 {
            let z = -field.half.z + (i as f32 + 0.5) * field.half.z * 2. / 7.;
            commands.spawn((
                WorldVisual,
                HazardVisual,
                HazardBeam(i),
                Visibility::Hidden,
                Mesh3d(cube.clone()),
                MeshMaterial3d(lines[0].clone()),
                Transform::from_translation(field.center + Vec3::Z * z).with_scale(Vec3::new(
                    2.,
                    field.half.y * 2.,
                    2.,
                )),
            ));
        }
    }
    commands.insert_resource(HazardMaterials { volumes, lines });
    let safe = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.68, 0.48),
        unlit: true,
        ..default()
    });
    let shortcut = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.55, 0.16),
        unlit: true,
        ..default()
    });
    let routes = [
        (
            vec![
                Vec3::new(-280., 1., 0.),
                Vec3::new(25., 1., 195.),
                Vec3::new(215., 1., 195.),
                Vec3::new(280., 1., 0.),
            ],
            safe,
        ),
        (
            vec![Vec3::new(-280., 1., 0.), Vec3::new(280., 1., 0.)],
            shortcut,
        ),
    ];
    for (points, material) in routes {
        for pair in points.windows(2) {
            let steps = (pair[0].distance(pair[1]) / 24.).ceil() as usize;
            for i in 0..=steps {
                commands.spawn((
                    WorldVisual,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(pair[0].lerp(pair[1], i as f32 / steps as f32))
                        .with_scale(Vec3::new(5., 0.5, 5.)),
                ));
            }
        }
    }
    let hud = commands
        .spawn((
            HazardHud,
            FooterFont::new(14., 11.),
            Text::default(),
            TextFont::from_font_size(14.),
            TextColor(Color::srgb(0.95, 0.8, 0.45)),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            Node {
                width: percent(100),
                ..default()
            },
        ))
        .id();
    commands.entity(*footer_slot).add_child(hud);
}

type HazardVisuals<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut MeshMaterial3d<StandardMaterial>,
        &'static mut Visibility,
        Option<&'static HazardBeam>,
    ),
    With<HazardVisual>,
>;

fn present(
    state: Res<HazardState>,
    phase: Res<GamePhase>,
    assets: Res<HazardMaterials>,
    mut hud: Single<(&mut Text, &mut TextColor), With<HazardHud>>,
    mut visuals: HazardVisuals,
) {
    let (index, label) = match state.phase {
        HazardPhase::Inactive => (0, "OPEN"),
        HazardPhase::Warning => (1, "WARNING - field about to fire"),
        HazardPhase::Active => (2, "ACTIVE - electrical damage"),
    };
    let suffix = if *phase == GamePhase::Playing {
        ""
    } else {
        " / PAUSED"
    };
    hud.0.0 = format!(
        "SHORTCUT  {label}  {:.1}s{suffix}\nGold dots: timed crossing | Green dots: longer detour\nLow blocks: fly over | Tall walls: go around",
        state.remaining()
    );
    hud.1.0 = if state.phase == HazardPhase::Active {
        Color::srgb(1., 0.42, 0.25)
    } else {
        Color::srgb(0.95, 0.8, 0.45)
    };
    for (mut material, mut visibility, beam) in &mut visuals {
        material.0 = if beam.is_some() {
            assets.lines[index].clone()
        } else {
            assets.volumes[index].clone()
        };
        *visibility = if let Some(beam) = beam {
            match state.phase {
                HazardPhase::Inactive => Visibility::Hidden,
                HazardPhase::Warning if (state.elapsed * 8.) as usize % 2 == beam.0 % 2 => {
                    Visibility::Hidden
                }
                _ => Visibility::Visible,
            }
        } else {
            Visibility::Visible
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn visible_hazard_states_follow_gameplay_and_restart_reuses_assets() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<WorldGeometry>()
            .add_plugins((
                crate::arena::ArenaPlugin,
                crate::combat::CombatPlugin,
                WorldScenePlugin,
            ));
        app.world_mut().spawn((HazardFooterSlot, Node::default()));
        app.update();
        let count = app
            .world_mut()
            .query_filtered::<Entity, With<WorldVisual>>()
            .iter(app.world())
            .count();
        assert!(count > 0);
        let meshes = app.world().resource::<Assets<Mesh>>().len();
        let materials = app.world().resource::<Assets<StandardMaterial>>().len();
        for (phase, label) in [
            (HazardPhase::Inactive, "OPEN"),
            (HazardPhase::Warning, "WARNING"),
            (HazardPhase::Active, "ACTIVE"),
        ] {
            app.world_mut().resource_mut::<HazardState>().phase = phase;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::ZERO);
            app.update();
            let text = app
                .world_mut()
                .query_filtered::<&Text, With<HazardHud>>()
                .single(app.world())
                .unwrap();
            assert!(text.0.contains(label));
        }
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(2));
        app.update();
        assert_eq!(
            app.world().resource::<HazardState>().phase,
            HazardPhase::Active
        );
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            app.update();
            assert_eq!(
                app.world().resource::<HazardState>().phase,
                HazardPhase::Inactive
            );
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
            assert_eq!(
                app.world().resource::<Assets<StandardMaterial>>().len(),
                materials
            );
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<WorldVisual>>()
                    .iter(app.world())
                    .count(),
                count
            );
        }
    }
}
