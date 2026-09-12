//! Authored arena landmarks shared by geometry, navigation, visuals and pilots.
use bevy::prelude::*;

pub(crate) const CHARGER_X: f32 = 560.;
pub(crate) const DIVIDER_X: f32 = 120.;
pub(crate) const DETOUR_Z: f32 = 410.;
pub(crate) const DETOUR_LEFT_X: f32 = 0.;
pub(crate) const DETOUR_RIGHT_X: f32 = 240.;
pub(crate) const LOW_COVER_LEFT: Vec3 = Vec3::new(-680., 30., -380.);
pub(crate) const LOW_COVER_RIGHT: Vec3 = Vec3::new(700., 35., 390.);

pub(crate) fn detour(height: f32) -> [Vec3; 4] {
    [
        Vec3::new(-CHARGER_X, height, 0.),
        Vec3::new(DETOUR_LEFT_X, height, DETOUR_Z),
        Vec3::new(DETOUR_RIGHT_X, height, DETOUR_Z),
        Vec3::new(CHARGER_X, height, 0.),
    ]
}
