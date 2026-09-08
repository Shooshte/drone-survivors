use bevy::prelude::*;

/// First entry into an axis-aligned box, as a fraction of the segment.
/// Starting inside counts as an immediate hit; parallel misses stay misses.
pub(super) fn segment_box(start: Vec3, end: Vec3, center: Vec3, half: Vec3) -> Option<f32> {
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
