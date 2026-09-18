//! Catalog-only ground-travel fields and single-use hull repair state.
use super::Solid;
use bevy::prelude::*;

pub(crate) const REPAIR_CENTER: Vec3 = Vec3::new(-420., 90., 0.);
pub(crate) const REPAIR_RADIUS: f32 = 70.;
pub(crate) const REPAIR_AMOUNT: u32 = 35;

#[derive(Clone, Copy)]
pub(crate) struct DirectionalField {
    pub bounds: Solid,
    pub cycling: bool,
}
impl DirectionalField {
    pub fn active(self, elapsed: f64) -> bool {
        !self.cycling || elapsed.rem_euclid(10.) < 6.
    }
}

#[derive(Resource, Default)]
pub(crate) struct Environment {
    pub fields: Vec<DirectionalField>,
    pub elapsed: f64,
    pub repair_ready: bool,
    pub repaired: u32,
}
impl Environment {
    pub fn reset(&mut self, enabled: bool) {
        *self = Self::default();
        if enabled {
            self.fields = [-190., 190.]
                .into_iter()
                .enumerate()
                .map(|(i, z)| DirectionalField {
                    bounds: Solid {
                        center: Vec3::new(-420., 150., z),
                        half: Vec3::new(220., 150., 125.),
                    },
                    cycling: i == 1,
                })
                .collect();
            self.repair_ready = true;
        }
    }
    pub fn enabled(&self) -> bool {
        !self.fields.is_empty()
    }
    /// Both authored fields point east. Union membership prevents overlap stacking.
    /// Apply to this substep's displacement only, never the stored rotor velocity.
    pub fn travel(&self, position: Vec3, displacement: Vec3, elapsed: f64) -> Vec3 {
        if self
            .fields
            .iter()
            .any(|f| f.active(elapsed) && f.bounds.overlaps(position, Vec3::ZERO))
        {
            displacement.with_x(displacement.x * if displacement.x >= 0. { 1.4 } else { 0.6 })
        } else {
            displacement
        }
    }
    pub fn cycle(&self) -> (bool, f64) {
        let phase = self.elapsed.rem_euclid(10.);
        (
            phase < 6.,
            if phase < 6. { 6. - phase } else { 10. - phase },
        )
    }
    pub fn repair(&mut self, hull: &mut u32, maximum: u32) {
        if self.enabled() && self.repair_ready && *hull > 0 && *hull < maximum {
            self.repaired = REPAIR_AMOUNT.min(maximum - *hull);
            *hull += self.repaired;
            self.repair_ready = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn environment_travel_axes_overlap_and_exit_are_bounded() {
        let mut env = Environment::default();
        env.reset(true);
        let at = env.fields[0].bounds.center;
        let positive = Vec3::new(10., 3., 7.);
        let negative = Vec3::new(-10., 3., 7.);
        assert_eq!(env.travel(at, positive, 0.), Vec3::new(14., 3., 7.));
        assert_eq!(env.travel(at, negative, 0.), Vec3::new(-6., 3., 7.));
        assert_eq!(env.travel(at, Vec3::Y + Vec3::Z, 0.), Vec3::Y + Vec3::Z);
        env.fields.extend(env.fields.clone());
        assert_eq!(env.travel(at, positive, 0.), Vec3::new(14., 3., 7.));
        assert_eq!(env.travel(at + Vec3::X * 221., positive, 0.), positive);
        assert_eq!(env.travel(at + Vec3::Y * 151., negative, 0.), negative);
    }
    #[test]
    fn environment_cycle_boundaries_and_repair_limits() {
        let mut env = Environment::default();
        env.reset(true);
        let at = env.fields[1].bounds.center;
        for t in [0., 5.999, 10., 25.] {
            assert_eq!(env.travel(at, Vec3::X, t).x, 1.4);
        }
        for t in [6., 9.999, 16.] {
            assert_eq!(env.travel(at, Vec3::X, t), Vec3::X);
        }
        let mut hull = 150;
        env.repair(&mut hull, 150);
        assert!(env.repair_ready);
        hull = 0;
        env.repair(&mut hull, 150);
        assert_eq!(hull, 0);
        assert!(env.repair_ready);
        hull = 149;
        env.repair(&mut hull, 150);
        assert_eq!(hull, 150);
        assert!(!env.repair_ready);
        hull = 50;
        env.repair(&mut hull, 150);
        assert_eq!(hull, 50);
        env.reset(true);
        env.repair(&mut hull, 150);
        assert_eq!(hull, 85);
    }
}
