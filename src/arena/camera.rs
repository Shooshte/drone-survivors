use super::Drone;
use crate::{
    combat::{Encounter, SpawnWarning},
    energy::ChargingNode,
    game::{GamePhase, GameplaySet},
};
use bevy::{camera::CameraUpdateSystems, prelude::*, ui::UiSystems};

#[derive(Component)]
pub(super) struct ArenaCamera;

pub(super) struct ArenaCameraPlugin;

impl Plugin for ArenaCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_indicators)
            .add_systems(Update, follow_scout.in_set(GameplaySet::Presentation))
            .add_systems(
                PostUpdate,
                update_indicators
                    .after(CameraUpdateSystems)
                    .before(UiSystems::Prepare),
            );
    }
}

fn follow_scout(
    phase: Res<GamePhase>,
    scout: Single<&Transform, With<Drone>>,
    mut camera: Single<&mut Transform, (With<ArenaCamera>, Without<Drone>)>,
) {
    if *phase == GamePhase::Playing {
        // Follow the grounded reference, preserving altitude, orientation and zoom.
        camera.translation = Vec3::new(scout.translation.x, 950., 1100. + scout.translation.z);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IndicatorEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl IndicatorEdge {
    const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    fn arrow(self) -> &'static str {
        match self {
            Self::Left => "<",
            Self::Right => ">",
            Self::Top => "^",
            Self::Bottom => "v",
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IndicatorPlacement {
    pub position: Vec2,
    pub edge: IndicatorEdge,
}

// Center coordinates allow the complete 126x28 label to clear status bands.
fn indicator_bounds(viewport: Rect) -> Option<Rect> {
    let bounds = Rect {
        min: viewport.min + Vec2::new(70., 124.),
        max: viewport.max - Vec2::new(70., 129.),
    };
    // Leave enough room for two chargers and one grouped warning on an edge.
    bounds
        .size()
        .cmpge(Vec2::new(268., 136.))
        .all()
        .then_some(bounds)
}

pub(super) fn project_indicator(
    camera: &Camera,
    transform: &GlobalTransform,
    target: Vec3,
) -> Option<IndicatorPlacement> {
    let viewport = camera.logical_viewport_rect()?;
    let bounds = indicator_bounds(viewport)?;
    let ndc: Vec3 = camera.world_to_ndc(transform, target)?;
    if !ndc.is_finite() || (ndc.x.abs() <= 1. && ndc.y.abs() <= 1. && (0. ..=1.).contains(&ndc.z)) {
        return None;
    }
    let screen = viewport.min + (Vec2::new(ndc.x, -ndc.y) + Vec2::ONE) * viewport.size() / 2.;
    let mut direction = screen - viewport.center();
    if direction.length_squared() < 0.001 {
        // Depth-clipped targets still get a stable edge marker, even on the axis.
        direction = Vec2::Y;
    }
    let half = bounds.size() / 2.;
    let tx = half.x / direction.x.abs();
    let ty = half.y / direction.y.abs();
    let edge = if tx <= ty {
        if direction.x < 0. {
            IndicatorEdge::Left
        } else {
            IndicatorEdge::Right
        }
    } else if direction.y < 0. {
        IndicatorEdge::Top
    } else {
        IndicatorEdge::Bottom
    };
    Some(IndicatorPlacement {
        position: (bounds.center() + direction * tx.min(ty)).clamp(bounds.min, bounds.max),
        edge,
    })
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum IndicatorKind {
    ChargerLeft,
    ChargerRight,
    Incoming(IndicatorEdge),
}

pub(super) fn setup_indicators(mut commands: Commands) {
    let kinds = [IndicatorKind::ChargerLeft, IndicatorKind::ChargerRight]
        .into_iter()
        .chain(IndicatorEdge::ALL.map(IndicatorKind::Incoming));
    for kind in kinds {
        let charger = matches!(
            kind,
            IndicatorKind::ChargerLeft | IndicatorKind::ChargerRight
        );
        commands.spawn((
            Name::new("Navigation edge indicator"),
            kind,
            Text::default(),
            TextFont::from_font_size(11.),
            TextColor(if charger {
                Color::srgb(0.60, 0.96, 0.85)
            } else {
                Color::srgb(1., 0.69, 0.40)
            }),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            BackgroundColor(Color::srgba(0.025, 0.05, 0.065, 0.90)),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                width: px(126),
                height: px(28),
                padding: UiRect::top(px(6)),
                ..default()
            },
        ));
    }
}

struct Label {
    kind: IndicatorKind,
    text: String,
    placement: IndicatorPlacement,
}

// Preserve projection direction while separating labels on each edge. Side
// labels leave room for the top/bottom labels at the corners as well as the HUD.
fn separate_labels(labels: &mut [Label], bounds: Rect) {
    for edge in IndicatorEdge::ALL {
        let horizontal = edge.horizontal();
        let axis = if horizontal { 0 } else { 1 };
        let (min, max, gap) = if horizontal {
            (bounds.min.x, bounds.max.x, 134.)
        } else {
            (bounds.min.y + 34., bounds.max.y - 34., 34.)
        };
        let mut indices: Vec<_> = labels
            .iter()
            .enumerate()
            .filter(|(_, label)| label.placement.edge == edge)
            .map(|(index, _)| index)
            .collect();
        indices.sort_by(|&a, &b| {
            labels[a].placement.position[axis].total_cmp(&labels[b].placement.position[axis])
        });
        let mut previous = min - gap;
        for &index in &indices {
            let value = labels[index].placement.position[axis]
                .clamp(min, max)
                .max(previous + gap);
            labels[index].placement.position[axis] = value;
            previous = value;
        }
        let mut next = max + gap;
        for &index in indices.iter().rev() {
            let value = labels[index].placement.position[axis].min(next - gap);
            labels[index].placement.position[axis] = value;
            next = value;
        }
    }
}

fn update_indicators(
    phase: Res<GamePhase>,
    camera: Single<(&Camera, &Transform), With<ArenaCamera>>,
    run: Option<Res<Encounter>>,
    chargers: Query<&ChargingNode>,
    warnings: Query<(&SpawnWarning, &Transform)>,
    mut indicators: Query<(&IndicatorKind, &mut Text, &mut Node)>,
) {
    let (camera, transform) = *camera;
    // This camera is an unparented scene root. Its current transform is ready
    // before UI layout; waiting for propagation would defer labels by a frame.
    let transform = GlobalTransform::from(*transform);
    let bounds = camera.logical_viewport_rect().and_then(indicator_bounds);
    let mut labels = Vec::new();
    if *phase == GamePhase::Playing
        && let Some(bounds) = bounds
    {
        // Query the actual authored charging nodes; layout changes flow through.
        let mut nodes: Vec<_> = chargers.iter().collect();
        nodes.sort_by(|a, b| a.center.x.total_cmp(&b.center.x));
        for (node, kind, name) in [
            (nodes.first(), IndicatorKind::ChargerLeft, "LEFT CHARGER"),
            (
                nodes.last().filter(|_| nodes.len() > 1),
                IndicatorKind::ChargerRight,
                "RIGHT CHARGER",
            ),
        ] {
            if let Some(node) = node
                && let Some(placement) = project_indicator(camera, &transform, node.center)
            {
                labels.push(Label {
                    kind,
                    text: format!("{} {name}", placement.edge.arrow()),
                    placement,
                });
            }
        }
        if let Some(run) = run {
            for edge in IndicatorEdge::ALL {
                let placements: Vec<_> = warnings
                    .iter()
                    .filter(|(warning, _)| {
                        !warning.cancelled && warning.ready_at > run.elapsed + 1e-7
                    })
                    .filter_map(|(_, target)| {
                        project_indicator(camera, &transform, target.translation)
                    })
                    .filter(|placement| placement.edge == edge)
                    .collect();
                if !placements.is_empty() {
                    let position = placements.iter().map(|p| p.position).sum::<Vec2>()
                        / placements.len() as f32;
                    labels.push(Label {
                        kind: IndicatorKind::Incoming(edge),
                        text: format!("{} INCOMING x{}", edge.arrow(), placements.len()),
                        placement: IndicatorPlacement { position, edge },
                    });
                }
            }
        }
        separate_labels(&mut labels, bounds);
    }
    for (kind, mut text, mut node) in &mut indicators {
        if let Some(label) = labels.iter().find(|label| label.kind == *kind) {
            node.display = Display::Flex;
            node.left = px(label.placement.position.x - 63.);
            node.top = px(label.placement.position.y - 14.);
            if text.0 != label.text {
                text.0.clone_from(&label.text);
            }
        } else {
            node.display = Display::None;
        }
    }
}
