//! Contact of two swept hulls at the same instant, including collision corrections.
use super::{CombatConfig, Enemy, collision::segment_box};
use crate::{
    arena::drone_world_half_extents,
    world::{MotionSegment, PlayerPath, WorldGeometry},
};
use bevy::prelude::*;

pub(super) fn contact(
    enemy: &Enemy,
    target: &Transform,
    player: &Transform,
    player_path: Option<&PlayerPath>,
    config: &CombatConfig,
    world: Option<&WorldGeometry>,
) -> Option<f32> {
    let enemy_fallback = [MotionSegment {
        start: enemy.previous,
        end: target.translation,
        from: 0.,
        to: 1.,
        half: crate::arena::world_half_extents(
            target.rotation,
            Vec3::splat(config.enemy_half_size),
        ),
    }];
    let player_fallback = [MotionSegment {
        start: player.translation,
        end: player.translation,
        from: 0.,
        to: 1.,
        half: drone_world_half_extents(player.rotation),
    }];
    let enemies = if enemy.path.is_empty() {
        &enemy_fallback[..]
    } else {
        &enemy.path
    };
    let players = player_path
        .filter(|p| !p.segments.is_empty())
        .map_or(&player_fallback[..], |p| &p.segments);
    enemies
        .iter()
        .flat_map(|enemy| {
            players.iter().filter_map(move |player| {
                let from = enemy.from.max(player.from);
                let to = enemy.to.min(player.to);
                if from > to {
                    return None;
                }
                let start_a = at(enemy, from, false);
                let end_a = at(enemy, to, true);
                let start_b = at(player, from, false);
                let end_b = at(player, to, true);
                let hit = segment_box(
                    start_a - start_b,
                    end_a - end_b,
                    Vec3::ZERO,
                    enemy.half + player.half + Vec3::splat(0.001),
                )?;
                let a = start_a.lerp(end_a, hit);
                let b = start_b.lerp(end_b, hit);
                world
                    .is_none_or(|w| w.line_clear(a, b))
                    .then_some(from + (to - from) * hit)
            })
        })
        .min_by(f32::total_cmp)
}

fn at(segment: &MotionSegment, time: f32, end: bool) -> Vec3 {
    if segment.to <= segment.from {
        if end { segment.end } else { segment.start }
    } else {
        segment.start.lerp(
            segment.end,
            ((time - segment.from) / (segment.to - segment.from)).clamp(0., 1.),
        )
    }
}
