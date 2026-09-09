use bevy::prelude::*;

/// First entry into an axis-aligned box, as a fraction of the segment.
/// Starting inside counts as an immediate hit; parallel misses stay misses.
pub(super) fn segment_box(start: Vec3, end: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
    if !start.is_finite() || !end.is_finite() || !center.is_finite() || !half.is_finite() {
        return None;
    }
    let origin = start - center;
    let direction = end - start;
    let mut enter = 0_f32;
    let mut exit = 1_f32;
    for axis in 0..3 {
        if direction[axis] == 0. {
            if origin[axis].abs() > half[axis] {
                return None;
            }
        } else {
            let a = (-half[axis] - origin[axis]) / direction[axis];
            let b = (half[axis] - origin[axis]) / direction[axis];
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
            if enter > exit {
                return None;
            }
        }
    }
    Some(enter)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nonfinite_segments_cannot_report_hits() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                segment_box(
                    Vec3::new(invalid, 0., 0.),
                    Vec3::ZERO,
                    Vec3::ZERO,
                    Vec3::ONE
                )
                .is_none()
            );
            assert!(
                segment_box(
                    Vec3::ZERO,
                    Vec3::new(invalid, 0., 0.),
                    Vec3::ZERO,
                    Vec3::ONE
                )
                .is_none()
            );
        }
        assert_eq!(
            segment_box(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ONE),
            Some(0.)
        );
    }
}
