//! Environment cues are installed by catalog only, using cached native meshes.
use crate::{
    arena::{Drone, FooterFont},
    combat::{CombatHudRoot, CombatSceneSetup},
    game::{GamePhase, GameplaySet},
    world::environment::{Environment, REPAIR_CENTER, REPAIR_RADIUS},
};
use bevy::prelude::*;

#[derive(Component)]
struct EnvironmentHud;
#[derive(Component)]
struct Cue {
    field: Option<usize>,
    arrow: bool,
}
#[derive(Resource)]
struct Materials {
    permanent: Handle<StandardMaterial>,
    cycling: Handle<StandardMaterial>,
    ready: Handle<StandardMaterial>,
    off: Handle<StandardMaterial>,
}

pub(super) fn install(app: &mut App) {
    app.add_systems(Startup, setup.after(CombatSceneSetup))
        .add_systems(Update, present.in_set(GameplaySet::Presentation));
}
fn setup(
    mut commands: Commands,
    hud: Query<Entity, With<CombatHudRoot>>,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let (Some(mut meshes), Some(mut materials)) = (meshes, materials) else {
        return;
    };
    if let Ok(hud) = hud.single() {
        commands.entity(hud).with_child((
            EnvironmentHud,
            Text::new(""),
            FooterFont::new(13., 12.),
            TextFont::from_font_size(13.),
            TextColor(Color::srgb(0.8, 0.92, 0.95)),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            Node {
                width: percent(100),
                display: Display::None,
                ..default()
            },
        ));
    }
    let mut material = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    let assets = Materials {
        permanent: material(Color::srgb(0.35, 0.78, 1.)),
        cycling: material(Color::srgb(0.85, 0.62, 1.)),
        ready: material(Color::srgb(0.30, 1., 0.55)),
        off: material(Color::srgb(0.36, 0.40, 0.44)),
    };
    let cube = meshes.add(Cuboid::default());
    let ring = meshes.add(Torus::new(REPAIR_RADIUS - 2., REPAIR_RADIUS));
    let mut prototype = Environment::default();
    prototype.reset(true);
    for (i, field) in prototype.fields.iter().enumerate() {
        let mat = if i == 0 {
            &assets.permanent
        } else {
            &assets.cycling
        };
        let center = field.bounds.center.with_y(1.);
        let half = field.bounds.half;
        // Floor outline plus corner uprights make the full-height footprint explicit.
        for sign in [-1., 1.] {
            for (offset, scale) in [
                (Vec3::Z * sign * half.z, Vec3::new(half.x * 2., 2., 3.)),
                (Vec3::X * sign * half.x, Vec3::new(3., 2., half.z * 2.)),
            ] {
                commands.spawn((
                    Cue {
                        field: Some(i),
                        arrow: false,
                    },
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(center + offset).with_scale(scale),
                    Visibility::Hidden,
                ));
            }
            for z in [-1., 1.] {
                commands.spawn((
                    Cue {
                        field: Some(i),
                        arrow: false,
                    },
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(
                        field.bounds.center + Vec3::new(sign * half.x, 0., z * half.z),
                    )
                    .with_scale(Vec3::new(1.5, half.y * 2., 1.5)),
                    Visibility::Hidden,
                ));
            }
        }
        // Repeated chevrons point east; switching them off is redundant with OFF text.
        for x in [-140., 0., 140.] {
            for sign in [-1., 1.] {
                commands.spawn((
                    Cue {
                        field: Some(i),
                        arrow: true,
                    },
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(center + Vec3::new(x, 2., sign * 15.))
                        .with_rotation(Quat::from_rotation_y(sign * std::f32::consts::FRAC_PI_4))
                        .with_scale(Vec3::new(45., 3., 4.)),
                    Visibility::Hidden,
                ));
            }
        }
    }
    for scale in [Vec3::new(52., 4., 12.), Vec3::new(12., 4., 52.)] {
        commands.spawn((
            Cue {
                field: None,
                arrow: true,
            },
            Mesh3d(cube.clone()),
            MeshMaterial3d(assets.ready.clone()),
            Transform::from_translation(REPAIR_CENTER).with_scale(scale),
            Visibility::Hidden,
        ));
    }
    commands.spawn((
        Cue {
            field: None,
            arrow: false,
        },
        Mesh3d(ring),
        MeshMaterial3d(assets.ready.clone()),
        Transform::from_translation(REPAIR_CENTER.with_y(2.)),
        Visibility::Hidden,
    ));
    commands.insert_resource(assets);
}
fn present(
    environment: Res<Environment>,
    phase: Res<GamePhase>,
    drone: Single<&Transform, (With<Drone>, Without<Cue>)>,
    materials: Option<Res<Materials>>,
    mut cues: Query<
        (
            &Cue,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Transform,
        ),
        Without<Drone>,
    >,
    mut text: Query<(&mut Text, &mut Node), With<EnvironmentHud>>,
) {
    let shown = environment.enabled() && *phase != GamePhase::Hub;
    for (mut text, mut node) in &mut text {
        node.display = if shown && *phase != GamePhase::Choosing {
            Display::Flex
        } else {
            Display::None
        };
        if !shown {
            continue;
        }
        let (on, remaining) = environment.cycle();
        let inside = environment
            .fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.bounds.overlaps(drone.translation, Vec3::ZERO));
        let location = match inside {
            Some((0, _)) => "IN NORTH",
            Some((_, _)) if on => "IN SOUTH",
            Some(_) => "SOUTH OFF",
            None => "OUTSIDE",
        };
        let delta = REPAIR_CENTER - drone.translation;
        let bearing = if delta.with_y(0.).length() < 10. {
            "HERE"
        } else if delta.x.abs() > delta.z.abs() {
            if delta.x > 0. { "E" } else { "W" }
        } else if delta.z > 0. {
            "S"
        } else {
            "N"
        };
        let repair = if environment.repair_ready {
            "READY +35".to_owned()
        } else {
            format!("USED +{}", environment.repaired)
        };
        text.0 = format!(
            "FIELDS > E +40% / W -40% / N-S neutral\nN permanent | S {} {:.1}s | {}\nREPAIR {} | {:.0}u {} | H90 / R70",
            if on { "ON" } else { "OFF" },
            remaining,
            location,
            repair,
            delta.length(),
            bearing
        );
    }
    let Some(materials) = materials else {
        return;
    };
    for (cue, mut visibility, mut material, mut transform) in &mut cues {
        let active = cue.field.map_or(environment.repair_ready, |i| {
            environment
                .fields
                .get(i)
                .is_some_and(|f| f.active(environment.elapsed))
        });
        *visibility = if shown && !(cue.field.is_some() && cue.arrow && !active) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        material.0 = if !active {
            materials.off.clone()
        } else {
            match cue.field {
                Some(0) => materials.permanent.clone(),
                Some(_) => materials.cycling.clone(),
                None => materials.ready.clone(),
            }
        };
        if cue.field.is_none() && cue.arrow {
            transform.rotation = Quat::from_rotation_y(if environment.repair_ready {
                0.
            } else {
                std::f32::consts::FRAC_PI_4
            });
        }
    }
}
