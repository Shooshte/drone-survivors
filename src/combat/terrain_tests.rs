use super::*;
use crate::arena::world_half_extents;
use crate::world::{WorldGeometry, navigation::next_point};

#[test]
fn pursuers_arrive_across_divider_and_replan_when_target_changes_sides() {
    let world = WorldGeometry::default();
    let combat = CombatConfig::default();
    let half = Vec3::splat(combat.enemy_half_size);
    for (start, target) in [
        (Vec3::new(-280., 90., -180.), Vec3::new(280., 90., -180.)),
        (Vec3::new(280., 90., -180.), Vec3::new(-280., 90., -180.)),
        (Vec3::new(280., 90., 195.), Vec3::new(-280., 90., 195.)),
        (Vec3::new(-280., 90., 195.), Vec3::new(280., 90., 195.)),
    ] {
        let mut t = Transform::from_translation(start);
        let mut f = DroneFlight::default();
        for goal in [target, start] {
            for _ in 0..15 * 60 {
                let next = next_point(
                    &world,
                    t.translation,
                    goal,
                    world_half_extents(t.rotation, half),
                )
                .unwrap_or_else(|| {
                    panic!("connected route from {:?} to {:?}", t.translation, goal)
                });
                for _ in 0..2 {
                    let input = pilot(t.translation, &f, next, &combat.enemy_flight, Vec3::ZERO);
                    f.step_in_world(
                        &mut t,
                        &input,
                        &combat.enemy_flight,
                        &Arena::default(),
                        half,
                        1. / 120.,
                        Some(&world),
                    );
                    assert!(
                        world
                            .solids
                            .iter()
                            .all(|s| !s
                                .overlaps(t.translation, world_half_extents(t.rotation, half)))
                    );
                }
            }
            assert!(
                t.translation.distance(goal) < 45.,
                "from {start:?} to {goal:?}, ended {:?}",
                t.translation
            );
        }
    }
}

#[test]
fn thirty_pursuers_do_not_stall_at_walls_when_target_changes_sides() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .insert_resource(Arena::default())
        .insert_resource(CombatConfig::default())
        .insert_resource(FlightConfig::default())
        .insert_resource(WorldGeometry::default())
        .add_systems(Update, chase);
    let drone = app
        .world_mut()
        .spawn((Drone, Transform::from_xyz(300., 90., -180.)))
        .id();
    let mut ids = Vec::new();
    for i in 0..30 {
        let point = Vec3::new(
            -350. + (i % 3) as f32 * 60.,
            90. + (i / 15) as f32 * 120.,
            -120. + ((i / 3) % 5) as f32 * 70.,
        );
        ids.push(
            app.world_mut()
                .spawn((
                    Enemy {
                        health: 1000,
                        previous: point,
                        path: Vec::new(),
                    },
                    Transform::from_translation(point),
                    DroneFlight::default(),
                ))
                .id(),
        );
    }
    let mut checkpoints: Vec<_> = ids
        .iter()
        .map(|&id| app.world().get::<Transform>(id).unwrap().translation)
        .collect();
    for frame in 0..30 * 60 {
        if frame == 3 * 60 {
            app.world_mut()
                .get_mut::<Transform>(drone)
                .unwrap()
                .translation = Vec3::new(-300., 90., -180.);
        }
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1. / 60.));
        app.update();
        let world = app.world().resource::<WorldGeometry>();
        for &id in &ids {
            let t = app.world().get::<Transform>(id).unwrap();
            assert!(world.solids.iter().all(|s| !s.overlaps(
                t.translation,
                world_half_extents(t.rotation, Vec3::splat(14.))
            )));
        }
        if (frame + 1) % (5 * 60) == 0 {
            let goal = app.world().get::<Transform>(drone).unwrap().translation;
            for (index, &id) in ids.iter().enumerate() {
                let point = app.world().get::<Transform>(id).unwrap().translation;
                if point.distance(goal) > 110. {
                    assert!(
                        point.distance(checkpoints[index]) > 20.,
                        "pursuer {index} stalled at {point:?}, goal {goal:?}, frame {frame}"
                    );
                }
                checkpoints[index] = point;
            }
        }
    }
}
