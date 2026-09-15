//! Shared swept proximity and line-of-sight checks for automatic collection.
use super::WorldGeometry;
use bevy::prelude::*;

/// Earliest visible candidate within the segment's sphere intersection. Use
/// center proximity (not the swept hull envelope) to retain the trigger radius.
pub(crate) fn proximity_contact(
    start: Vec3,
    end: Vec3,
    center: Vec3,
    radius: f32,
    geometry: Option<&WorldGeometry>,
) -> Option<f32> {
    let delta = end - start;
    let length_squared = delta.length_squared();
    let visible = |at| geometry.is_none_or(|g| g.line_clear(start.lerp(end, at), center));
    if length_squared == 0. {
        return (start.distance_squared(center) <= radius * radius && visible(0.)).then_some(0.);
    }
    let closest = (center - start).dot(delta) / length_squared;
    let distance_squared = (start + delta * closest).distance_squared(center);
    let remaining = radius * radius - distance_squared;
    if remaining < 0. {
        return None;
    }
    let half_span = (remaining / length_squared).sqrt();
    let enter = (closest - half_span).max(0.);
    let exit = (closest + half_span).min(1.);
    if enter > exit {
        return None;
    }
    // A wall can obscure entry while the substep moves into view. Check the
    // nearest point and exit as well; never borrow LOS from the frame endpoint.
    [enter, closest.clamp(enter, exit), exit]
        .into_iter()
        .find(|&at| visible(at))
}
