//! At-site guidance for the optional, one-attempt holdout.
use super::blockout::{BlockoutRun, HOLDOUT_COMPONENTS, HOLDOUT_RADIUS, HOLDOUT_SECONDS, Holdout};
use crate::{
    arena::Drone,
    game::{GamePhase, GameplaySet},
    world::mission01 as map,
};
use bevy::prelude::*;

pub(super) struct HoldoutScenePlugin;
#[derive(Component)]
struct HintPanel;
#[derive(Component)]
struct HintText;
#[derive(Component)]
struct ProgressFill;
#[derive(Component)]
struct BoundaryRing;
#[derive(Resource)]
struct RingMaterials {
    available: Handle<StandardMaterial>,
    active: Handle<StandardMaterial>,
    complete: Handle<StandardMaterial>,
    forfeited: Handle<StandardMaterial>,
}
impl Plugin for HoldoutScenePlugin {
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
    let mut material = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        })
    };
    let ring_materials = RingMaterials {
        available: material(Color::srgb(0.3, 0.9, 0.5)),
        active: material(Color::srgb(1., 0.7, 0.25)),
        complete: material(Color::srgb(0.25, 0.9, 0.95)),
        forfeited: material(Color::srgb(0.45, 0.48, 0.5)),
    };
    commands.spawn((
        BoundaryRing,
        Mesh3d(meshes.add(Torus::new(HOLDOUT_RADIUS - 8., HOLDOUT_RADIUS))),
        MeshMaterial3d(ring_materials.available.clone()),
        Transform::from_translation(map::challenge().with_y(3.)),
        Visibility::Hidden,
    ));
    commands.insert_resource(ring_materials);
    commands
        .spawn((
            HintPanel,
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                top: px(155),
                width: px(340),
                padding: UiRect::all(px(8)),
                row_gap: px(6),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.06, 0.075, 0.95)),
            GlobalZIndex(2),
        ))
        .with_children(|panel| {
            panel.spawn((
                HintText,
                Text::default(),
                TextFont::from_font_size(13.),
                TextColor(Color::srgb(0.8, 1., 0.85)),
                Node {
                    width: percent(100),
                    ..default()
                },
            ));
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(4),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.25, 0.28)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        ProgressFill,
                        Node {
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.95, 0.65)),
                    ));
                });
        });
}
#[allow(clippy::too_many_arguments)]
fn present(
    run: Res<BlockoutRun>,
    phase: Res<GamePhase>,
    materials: Res<RingMaterials>,
    drone: Single<&Transform, With<Drone>>,
    windows: Query<&Window>,
    mut panel: Single<&mut Node, (With<HintPanel>, Without<ProgressFill>)>,
    mut fill: Single<&mut Node, (With<ProgressFill>, Without<HintPanel>)>,
    mut text: Single<(&mut Text, &mut TextFont), With<HintText>>,
    mut ring: Single<(&mut Visibility, &mut MeshMaterial3d<StandardMaterial>), With<BoundaryRing>>,
) {
    let playing = run.enabled && matches!(*phase, GamePhase::Playing | GamePhase::Choosing);
    let near = (drone.translation - map::challenge()).with_y(0.).length() <= HOLDOUT_RADIUS + 500.;
    panel.display = if playing && (near || matches!(run.holdout, Holdout::Active { .. })) {
        Display::Flex
    } else {
        Display::None
    };
    let compact = windows.iter().next().is_some_and(|w| w.width() < 800.);
    panel.width = px(if compact { 340. } else { 410. });
    panel.top = px(if compact { 155. } else { 170. });
    text.1.font_size = (if compact { 12. } else { 14. }).into();
    let (value, progress, material) = match run.holdout {
        Holdout::Available => (
            format!(
                "OPTIONAL HOLDOUT / +{HOLDOUT_COMPONENTS} COMPONENTS\nStay inside for {HOLDOUT_SECONDS:.0}s. Leaving forfeits the reward."
            ),
            0.,
            &materials.available,
        ),
        Holdout::Active { elapsed, .. } => {
            let left = (HOLDOUT_SECONDS - elapsed).max(0.).ceil();
            let label = if *phase == GamePhase::Choosing {
                "PAUSED"
            } else {
                "HOLDOUT"
            };
            (
                format!(
                    "{label}: {left:.0}s LEFT / +{HOLDOUT_COMPONENTS} COMPONENTS\nStay inside the ring. Leaving forfeits the reward."
                ),
                (elapsed / HOLDOUT_SECONDS).clamp(0., 1.),
                &materials.active,
            )
        }
        Holdout::Complete => (
            format!(
                "HOLDOUT COMPLETE / +{HOLDOUT_COMPONENTS} COMPONENTS\nReward collected. Deliver the payload to bank all loot."
            ),
            1.,
            &materials.complete,
        ),
        Holdout::Forfeited => (
            "HOLDOUT FORFEITED\nLeaving ended this attempt. Restart for another try.".into(),
            0.,
            &materials.forfeited,
        ),
    };
    text.0.0 = value;
    fill.width = percent((progress * 100.) as f32);
    ring.0.set_if_neq(if playing {
        Visibility::Visible
    } else {
        Visibility::Hidden
    });
    ring.1.0 = material.clone();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn holdout_prompt_tracks_pause_progress_terminal_states_and_map_visibility() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(BlockoutRun {
                enabled: true,
                ..default()
            })
            .insert_resource(GamePhase::Playing)
            .add_plugins(HoldoutScenePlugin);
        let drone = app
            .world_mut()
            .spawn((Drone, Transform::from_translation(map::challenge())))
            .id();
        app.update();
        for (state, phase, needle, progress) in [
            (
                Holdout::Active {
                    elapsed: 12.,
                    waves: 2,
                },
                GamePhase::Playing,
                "18s LEFT",
                40.,
            ),
            (
                Holdout::Active {
                    elapsed: 12.,
                    waves: 2,
                },
                GamePhase::Choosing,
                "PAUSED",
                40.,
            ),
            (
                Holdout::Complete,
                GamePhase::Playing,
                "HOLDOUT COMPLETE / +5 COMPONENTS",
                100.,
            ),
            (
                Holdout::Forfeited,
                GamePhase::Playing,
                "HOLDOUT FORFEITED",
                0.,
            ),
        ] {
            app.world_mut().resource_mut::<BlockoutRun>().holdout = state;
            *app.world_mut().resource_mut::<GamePhase>() = phase;
            app.update();
            let text = &app
                .world_mut()
                .query_filtered::<&Text, With<HintText>>()
                .single(app.world())
                .unwrap()
                .0;
            assert!(text.contains(needle), "{text}");
            let fill = app
                .world_mut()
                .query_filtered::<&Node, With<ProgressFill>>()
                .single(app.world())
                .unwrap();
            assert_eq!(fill.width, percent(progress));
        }
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = map::start();
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<&Node, With<HintPanel>>()
                .single(app.world())
                .unwrap()
                .display,
            Display::None
        );
        app.world_mut().resource_mut::<BlockoutRun>().enabled = false;
        app.update();
        assert_eq!(
            *app.world_mut()
                .query_filtered::<&Visibility, With<BoundaryRing>>()
                .single(app.world())
                .unwrap(),
            Visibility::Hidden
        );
    }
}
