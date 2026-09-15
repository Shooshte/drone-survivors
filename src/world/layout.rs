//! Authored arena landmarks shared by geometry, navigation, visuals and pilots.
use bevy::prelude::*;

pub(crate) const CHARGER_X: f32 = 560.;
pub(crate) const CHARGER_Z: f32 = 1080.;
/// One reusable east/west charger pair and an optional resource site. The three
/// instances preserve the original center, north, and south arena content.
struct ArenaPiece {
    z: f32,
    labels: [&'static str; 2],
    cache_index: usize,
    cache_offset: Vec3,
}
impl ArenaPiece {
    const fn charger(&self, side: usize) -> (&'static str, Vec3) {
        (
            self.labels[side],
            Vec3::new(if side == 0 { -CHARGER_X } else { CHARGER_X }, 0., self.z),
        )
    }
    const fn cache(&self) -> Vec3 {
        Vec3::new(
            self.cache_offset.x,
            self.cache_offset.y,
            self.z + self.cache_offset.z,
        )
    }
}
const PIECES: [ArenaPiece; 3] = [
    ArenaPiece {
        z: 0.,
        labels: ["LEFT", "RIGHT"],
        cache_index: 2,
        cache_offset: Vec3::new(120., 10., 0.),
    },
    ArenaPiece {
        z: -CHARGER_Z,
        labels: ["NW", "NE"],
        cache_index: 0,
        cache_offset: Vec3::new(-780., 10., -400.),
    },
    ArenaPiece {
        z: CHARGER_Z,
        labels: ["SW", "SE"],
        cache_index: 1,
        cache_offset: Vec3::new(780., 10., 400.),
    },
];
pub(crate) const CHARGERS: [(&str, Vec3); 6] = [
    PIECES[0].charger(0),
    PIECES[0].charger(1),
    PIECES[1].charger(0),
    PIECES[1].charger(1),
    PIECES[2].charger(0),
    PIECES[2].charger(1),
];
pub(crate) fn cache_sites() -> [Vec3; 3] {
    let mut sites = [Vec3::ZERO; 3];
    for piece in &PIECES {
        sites[piece.cache_index] = piece.cache();
    }
    sites
}
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
