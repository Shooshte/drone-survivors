use super::{
    WorldGeometry,
    layout::{CHARGER_X, DETOUR_LEFT_X, DETOUR_RIGHT_X, DETOUR_Z},
};
use bevy::prelude::*;

/// Built once for an immutable blockout. Pairwise swept-body visibility and
/// shortest-path distances are reused by every pursuer, including spawn checks.
pub(crate) struct NavigationGraph {
    points: Vec<Vec3>,
    distances: Vec<Vec<f32>>,
    clearance: Vec3,
}
impl NavigationGraph {
    pub(crate) fn build(world: &WorldGeometry, points: Vec<Vec3>, clearance: Vec3) -> Self {
        let count = points.len();
        let mut distances = vec![vec![f32::INFINITY; count]; count];
        for (i, a) in points.iter().enumerate() {
            distances[i][i] = 0.;
            for (j, b) in points.iter().enumerate().skip(i + 1) {
                if world.clear_body(*a, *b, clearance) {
                    distances[i][j] = a.distance(*b);
                    distances[j][i] = distances[i][j];
                }
            }
        }
        for via in 0..count {
            for a in 0..count {
                for b in 0..count {
                    distances[a][b] = distances[a][b].min(distances[a][via] + distances[via][b]);
                }
            }
        }
        Self {
            points,
            distances,
            clearance,
        }
    }
    fn next(&self, world: &WorldGeometry, start: Vec3, target: Vec3, half: Vec3) -> Option<Vec3> {
        // Larger future actors need their own clearance graph; never silently
        // route them down an edge checked for a smaller body.
        if half.cmpgt(self.clearance).any() {
            return None;
        }
        let from: Vec<_> = self
            .points
            .iter()
            .map(|&point| {
                let distance = start.distance(point);
                if distance > 1. && world.clear_body(start, point, half) {
                    distance
                } else {
                    f32::INFINITY
                }
            })
            .collect();
        let to: Vec<_> = self
            .points
            .iter()
            .map(|&point| {
                if world.clear_body(point, target, half) {
                    point.distance(target)
                } else {
                    f32::INFINITY
                }
            })
            .collect();
        let mut best = f32::INFINITY;
        let mut next = None;
        for (a, from_cost) in from.iter().enumerate() {
            if !from_cost.is_finite() {
                continue;
            }
            for (b, to_cost) in to.iter().enumerate() {
                let cost = from_cost + self.distances[a][b] + to_cost;
                if cost < best {
                    best = cost;
                    next = Some(self.points[a]);
                }
            }
        }
        next
    }
}

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
    if let Some(graph) = &world.navigation {
        // Turn padding protects graph corners, not the final physical approach.
        // A target beside terrain can fit the body without the extra padding.
        if world.clear_body(start, target, half) {
            return Some(target);
        }
        return graph.next(world, start, target, half);
    }
    let mut points = vec![start, target];
    for x in [-CHARGER_X, DETOUR_LEFT_X, DETOUR_RIGHT_X, CHARGER_X] {
        for z in [-450., 0., DETOUR_Z] {
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
