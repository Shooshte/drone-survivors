use bevy::prelude::*;
mod geometry;
pub(crate) mod hazard;
pub(crate) mod navigation;
pub(crate) mod scene;
pub(crate) use geometry::{Solid, WorldGeometry};
#[derive(Resource, Default)]
pub(crate) struct PlayerPath {
    pub segments: Vec<MotionSegment>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct MotionSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub from: f32,
    pub to: f32,
    pub half: Vec3,
}
#[cfg(test)]
mod tests;
