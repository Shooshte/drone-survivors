use super::*;
use crate::world::{MotionSegment, Solid};

fn fixture(points: &[Vec3], sites: [Vec3; 3], extraction: Vec3, visited: [bool; 3]) -> App {
    let mut app = App::new();
    app.insert_resource(ObjectiveConfig {
        sites,
        extraction,
        visit_radius: 1.,
        extraction_radius: 1.,
    })
    .insert_resource(ObjectiveRun {
        kind: ObjectiveKind::Extraction,
        visited,
    })
    .insert_resource(GamePhase::Playing)
    .insert_resource(PlayerPath {
        segments: points
            .windows(2)
            .enumerate()
            .map(|(i, p)| MotionSegment {
                start: p[0],
                end: p[1],
                from: i as f32 / (points.len() - 1) as f32,
                to: (i + 1) as f32 / (points.len() - 1) as f32,
                half: Vec3::splat(100.),
            })
            .collect(),
    })
    .add_systems(Update, update);
    app.world_mut()
        .spawn((Drone, Transform::from_translation(*points.last().unwrap())));
    app
}

#[test]
fn site_crossings_are_three_dimensional_and_do_not_expand_by_hull_size() {
    let mut app = fixture(
        &[Vec3::new(-4., 0., 0.), Vec3::new(4., 0., 0.)],
        [Vec3::ZERO, Vec3::new(0., 1.01, 0.), Vec3::new(0., 0., 1.01)],
        Vec3::splat(20.),
        [false; 3],
    );
    app.update();
    assert_eq!(
        app.world().resource::<ObjectiveRun>().visited,
        [true, false, false]
    );
}
#[test]
fn piecewise_path_does_not_invent_a_crossing_along_the_frame_chord() {
    let mut app = fixture(
        &[
            Vec3::new(-4., 0., 0.),
            Vec3::new(-4., 0., 4.),
            Vec3::new(4., 0., 4.),
            Vec3::new(4., 0., 0.),
        ],
        [Vec3::ZERO, Vec3::new(0., 0., 4.), Vec3::splat(20.)],
        Vec3::splat(30.),
        [false; 3],
    );
    app.update();
    assert_eq!(
        app.world().resource::<ObjectiveRun>().visited,
        [false, true, false]
    );
}
#[test]
fn locked_extraction_crossed_before_final_cargo_is_not_retroactive() {
    for points in [
        vec![Vec3::new(-5., 0., 0.), Vec3::new(5., 0., 0.)],
        vec![Vec3::new(-5., 0., 0.), Vec3::ZERO, Vec3::new(5., 0., 0.)],
    ] {
        let mut app = fixture(
            &points,
            [Vec3::new(3., 0., 0.); 3],
            Vec3::new(-3., 0., 0.),
            [true, true, false],
        );
        app.update();
        assert!(app.world().resource::<ObjectiveRun>().ready());
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    }
}
#[test]
fn final_cargo_then_exit_in_one_segment_completes_even_if_site_order_differs() {
    let mut app = fixture(
        &[Vec3::new(-5., 0., 0.), Vec3::new(8., 0., 0.)],
        [Vec3::new(2., 0., 0.), Vec3::new(-3., 0., 0.), Vec3::ZERO],
        Vec3::new(6., 0., 0.),
        [false; 3],
    );
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
}
#[test]
fn line_of_sight_is_checked_at_the_crossing_not_the_frame_endpoint() {
    let mut app = fixture(
        &[Vec3::new(-4., 0., 0.), Vec3::new(4., 0., 0.)],
        [Vec3::ZERO; 3],
        Vec3::splat(20.),
        [false; 3],
    );
    // The endpoint is behind a wall; the trigger crossing happened before it.
    app.insert_resource(WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(2., 0., 0.),
            half: Vec3::splat(0.1),
        }],
        hazard: None,
    });
    app.update();
    assert!(app.world().resource::<ObjectiveRun>().ready());
    // An entirely obstructed crossing remains invalid for sites and extraction.
    let mut app = fixture(
        &[Vec3::new(-4., 0., 0.), Vec3::new(4., 0., 0.)],
        [Vec3::new(0., 0.8, 0.); 3],
        Vec3::new(0., 0.8, 0.),
        [false; 3],
    );
    app.insert_resource(WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(0., 0.4, 0.),
            half: Vec3::new(10., 0.1, 10.),
        }],
        hazard: None,
    });
    app.update();
    assert_eq!(app.world().resource::<ObjectiveRun>().count(), 0);
    app.world_mut().resource_mut::<ObjectiveRun>().visited = [true; 3];
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}
#[test]
fn tangency_and_stationary_proximity_still_count() {
    for points in [
        vec![Vec3::new(-4., 1., 0.), Vec3::new(4., 1., 0.)],
        vec![Vec3::ZERO],
    ] {
        let mut app = fixture(&points, [Vec3::ZERO; 3], Vec3::splat(20.), [false; 3]);
        app.update();
        assert!(app.world().resource::<ObjectiveRun>().ready());
    }
}
