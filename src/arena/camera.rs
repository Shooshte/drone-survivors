use super::Drone;
use crate::{
    combat::{Encounter, SpawnWarning},
    energy::{ChargingNode, ChargingNodeLabel, charging_node_name},
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
    // Match responsive typography: the normal footer is taller than the compact one.
    let (top, bottom) = if viewport.width() < super::scene::COMPACT_HUD_WIDTH {
        (110., 125.)
    } else {
        (120., 145.)
    };
    let bounds = Rect {
        min: viewport.min + Vec2::new(70., top + 14.),
        max: viewport.max - Vec2::new(70., bottom + 14.),
    };
    // Leave enough room for one charger group and one warning group on an edge.
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
    Charger(IndicatorEdge),
    Incoming(IndicatorEdge),
}

pub(super) fn setup_indicators(mut commands: Commands) {
    let kinds = IndicatorEdge::ALL
        .map(IndicatorKind::Charger)
        .into_iter()
        .chain(IndicatorEdge::ALL.map(IndicatorKind::Incoming));
    for kind in kinds {
        let charger = matches!(kind, IndicatorKind::Charger(_));
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
    compact: bool,
}

fn charger_group_text(edge: IndicatorEdge, names: &[&str]) -> String {
    if names.len() == 1 {
        return format!("{} {} CHARGER", edge.arrow(), names[0]);
    }
    let mut result = format!("{} CHG ", edge.arrow());
    let mut line_len = result.len();
    for (index, name) in names.iter().enumerate() {
        let separator = usize::from(index > 0);
        if line_len + separator + name.len() > 18 {
            result.push('\n');
            line_len = 0;
        } else if index > 0 {
            result.push('/');
            line_len += 1;
        }
        result.push_str(name);
        line_len += name.len();
    }
    result
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

// Computed HUD layout comes from the previous frame; keep one line of slack
// when a transient status adds a row before this frame's UI layout runs.
fn place_clear_of_hud(labels: &mut Vec<Label>, mut bounds: Rect, hud_bottom: f32) {
    bounds.min.y = bounds.min.y.max(hud_bottom + 24. + 14.);
    if bounds.height() < 136. {
        labels.clear();
        return;
    }
    for label in labels.iter_mut() {
        label.placement.position = label.placement.position.clamp(bounds.min, bounds.max);
    }
    separate_labels(labels, bounds);
}

#[allow(clippy::too_many_arguments)]
fn update_indicators(
    phase: Res<GamePhase>,
    camera: Single<(&Camera, &Transform), With<ArenaCamera>>,
    run: Option<Res<Encounter>>,
    chargers: Query<(&ChargingNode, Option<&ChargingNodeLabel>)>,
    warnings: Query<(&SpawnWarning, &Transform)>,
    mut indicators: Query<(&IndicatorKind, &mut Text, &mut Node, &mut TextFont)>,
    hud: Query<
        (&ComputedNode, &UiGlobalTransform, Option<&Children>),
        With<crate::combat::CombatHudRoot>,
    >,
    hud_children: Query<(&Node, &ComputedNode), Without<IndicatorKind>>,
    windows: Query<&Window>,
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
        // Every live node contributes its shared identity to its projected edge.
        for edge in IndicatorEdge::ALL {
            let mut projected: Vec<_> = chargers
                .iter()
                .filter_map(|(node, label)| {
                    project_indicator(camera, &transform, node.center)
                        .filter(|placement| placement.edge == edge)
                        .map(|placement| (charging_node_name(node, label), placement))
                })
                .collect();
            projected.sort_by(|a, b| a.0.cmp(b.0));
            if !projected.is_empty() {
                let position = projected
                    .iter()
                    .map(|(_, placement)| placement.position)
                    .sum::<Vec2>()
                    / projected.len() as f32;
                let names: Vec<_> = projected.iter().map(|(name, _)| *name).collect();
                labels.push(Label {
                    kind: IndicatorKind::Charger(edge),
                    text: charger_group_text(edge, &names),
                    placement: IndicatorPlacement { position, edge },
                    compact: names.len() > 1,
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
                        compact: false,
                    });
                }
            }
        }
        let scale = windows.single().map_or(1., Window::scale_factor);
        let hud_bottom = hud
            .iter()
            .map(|(node, transform, _)| (transform.translation.y + node.size().y / 2.) / scale)
            .fold(0., f32::max);
        // A newly revealed status can add several lines. Its zero-sized old
        // layout is not a usable measurement; defer indicators for this frame.
        let awaiting_layout = hud.iter().any(|(_, _, children)| {
            children.is_some_and(|children| {
                children.iter().any(|child| {
                    hud_children.get(child).is_ok_and(|(node, computed)| {
                        node.display != Display::None && computed.size().y == 0.
                    })
                })
            })
        });
        if awaiting_layout {
            labels.clear();
        } else {
            place_clear_of_hud(&mut labels, bounds, hud_bottom);
        }
    }
    for (kind, mut text, mut node, mut font) in &mut indicators {
        if let Some(label) = labels.iter().find(|label| label.kind == *kind) {
            node.display = Display::Flex;
            font.font_size = bevy::text::FontSize::Px(if label.compact { 10. } else { 11. });
            node.padding.top = px(if label.compact { 2. } else { 6. });
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

#[cfg(test)]
mod hud_clearance_tests {
    use super::*;

    #[test]
    fn expanded_status_band_keeps_navigation_labels_below_hud() {
        let bounds =
            indicator_bounds(Rect::from_corners(Vec2::ZERO, Vec2::new(640., 480.))).unwrap();
        let mut labels = vec![Label {
            kind: IndicatorKind::Charger(IndicatorEdge::Top),
            text: "^ CHG NW/NE".into(),
            placement: IndicatorPlacement {
                position: Vec2::new(320., bounds.min.y),
                edge: IndicatorEdge::Top,
            },
            compact: true,
        }];
        place_clear_of_hud(&mut labels, bounds, 145.);
        assert_eq!(labels.len(), 1);
        assert!(labels[0].placement.position.y - 14. >= 153.);
        place_clear_of_hud(&mut labels, bounds, 300.);
        assert!(
            labels.is_empty(),
            "no safe area: hide rather than overlap HUD/footer"
        );
    }
}
