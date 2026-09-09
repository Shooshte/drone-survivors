use bevy::prelude::*;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Solid {
    pub center: Vec3,
    pub half: Vec3,
}
#[derive(Resource)]
pub(crate) struct WorldGeometry {
    pub solids: Vec<Solid>,
    pub hazard: Option<Solid>,
}
impl Default for WorldGeometry {
    fn default() -> Self {
        Self {
            solids: vec![
                Solid {
                    center: Vec3::new(120., 150., -170.),
                    half: Vec3::new(12., 150., 100.),
                },
                Solid {
                    center: Vec3::new(120., 150., 90.),
                    half: Vec3::new(12., 150., 20.),
                },
                Solid {
                    center: Vec3::new(-170., 30., -185.),
                    half: Vec3::new(65., 30., 30.),
                },
                Solid {
                    center: Vec3::new(320., 35., 175.),
                    half: Vec3::new(60., 35., 25.),
                },
            ],
            hazard: Some(Solid {
                center: Vec3::new(120., 150., 0.),
                half: Vec3::new(24., 150., 70.),
            }),
        }
    }
}
impl Solid {
    pub(crate) fn overlaps(&self, center: Vec3, half: Vec3) -> bool {
        (center - self.center).abs().cmplt(self.half + half).all()
    }
    pub(crate) fn segment_hit(&self, start: Vec3, end: Vec3, half: Vec3) -> Option<f32> {
        self.sweep(start, end, half).map(|(time, _)| time)
    }
    /// Slab intersection in normalized segment time. The entry normal permits sliding.
    pub(crate) fn sweep(&self, start: Vec3, end: Vec3, half: Vec3) -> Option<(f32, Vec3)> {
        let min = self.center - self.half - half;
        let max = self.center + self.half + half;
        let delta = end - start;
        let mut enter = 0_f32;
        let mut exit = 1_f32;
        let mut normal = Vec3::ZERO;
        for axis in 0..3 {
            if delta[axis].abs() < 1e-8 {
                if start[axis] < min[axis] || start[axis] > max[axis] {
                    return None;
                }
            } else {
                let a = (min[axis] - start[axis]) / delta[axis];
                let b = (max[axis] - start[axis]) / delta[axis];
                let near = a.min(b);
                if near >= enter {
                    enter = near;
                    normal = Vec3::ZERO;
                    normal[axis] = -delta[axis].signum();
                }
                exit = exit.min(a.max(b));
                if enter > exit {
                    return None;
                }
            }
        }
        if normal == Vec3::ZERO && self.overlaps(start, half) {
            let mut depth = f32::INFINITY;
            for axis in 0..3 {
                for sign in [-1., 1.] {
                    let gap = if sign < 0. {
                        start[axis] - min[axis]
                    } else {
                        max[axis] - start[axis]
                    };
                    if gap < depth {
                        depth = gap;
                        normal = Vec3::ZERO;
                        normal[axis] = sign;
                    }
                }
            }
        }
        if exit < 0. || enter > 1. {
            None
        } else {
            Some((enter, normal))
        }
    }
}
impl WorldGeometry {
    pub(crate) fn first_hit(&self, start: Vec3, end: Vec3, radius: f32) -> Option<f32> {
        self.solids
            .iter()
            .filter_map(|s| s.segment_hit(start, end, Vec3::splat(radius)))
            .min_by(f32::total_cmp)
    }
    pub(crate) fn line_clear(&self, start: Vec3, end: Vec3) -> bool {
        self.first_hit(start, end, 0.).is_none()
    }
    pub(crate) fn clear_body(&self, start: Vec3, end: Vec3, half: Vec3) -> bool {
        self.solids
            .iter()
            .all(|s| s.segment_hit(start, end, half).is_none())
    }
}
