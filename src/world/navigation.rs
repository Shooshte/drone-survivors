use super::WorldGeometry;
use bevy::prelude::*;

/// A dozen authored turning points. High cruise altitude clears both low blocks;
/// every connection is still checked against the caller's whole body.
pub(crate) fn next_point(
    world: &WorldGeometry,
    start: Vec3,
    mut target: Vec3,
    half: Vec3,
) -> Option<Vec3> {
    // A scout resting on cover can be closer to its top than a banking
    // pursuer's hull allows. Approach from just above that surface instead of
    // repeatedly losing the route as the pursuer changes attitude.
    for solid in &world.solids {
        let top = solid.center.y + solid.half.y;
        let horizontal = (target - solid.center).abs();
        if target.y >= top
            && horizontal.x < solid.half.x + half.x
            && horizontal.z < solid.half.z + half.z
        {
            target.y = target.y.max(top + half.y + 1.);
        }
    }
    if world.clear_body(start, target, half + Vec3::splat(12.)) {
        return Some(target);
    }
    let mut points = vec![start, target];
    for x in [-270., 55., 185., 405.] {
        for z in [-235., 0., 195.] {
            points.push(Vec3::new(x, 150., z));
        }
    }
    let n = points.len();
    let mut cost = vec![f32::INFINITY; n];
    let mut previous = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    cost[0] = 0.;
    for _ in 0..n {
        let current = (0..n)
            .filter(|&i| !visited[i])
            .min_by(|&a, &b| cost[a].total_cmp(&cost[b]))?;
        if !cost[current].is_finite() {
            return None;
        }
        if current == 1 {
            break;
        }
        visited[current] = true;
        for next in 1..n {
            if visited[next]
                || !world.clear_body(
                    points[current],
                    points[next],
                    // Endpoints may rest against cover. Turn padding belongs to
                    // graph corners, not the final physical approach to a target.
                    if current == 0 || next == 1 {
                        half
                    } else {
                        half + Vec3::splat(12.)
                    },
                )
            {
                continue;
            }
            let distance = cost[current] + points[current].distance(points[next]);
            if distance < cost[next] {
                cost[next] = distance;
                previous[next] = current;
            }
        }
    }
    let mut next = 1;
    while previous[next] != 0 {
        next = previous[next];
        if next == usize::MAX {
            return None;
        }
    }
    Some(points[next])
}
