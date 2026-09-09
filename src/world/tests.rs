use super::*;
#[test]
fn swept_body_detects_thin_wall_before_endpoint() {
    let wall = Solid {
        center: Vec3::ZERO,
        half: Vec3::new(1., 150., 100.),
    };
    let hit = wall
        .segment_hit(
            Vec3::new(-100., 0., 0.),
            Vec3::new(100., 0., 0.),
            Vec3::splat(10.),
        )
        .expect("crossing a thin wall must hit");
    assert!((hit - 0.445).abs() < 0.0001);
}

use crate::arena::{
    Arena, DRONE_HALF_EXTENTS, DroneFlight, FlightConfig, FlightInput, world_half_extents,
};
fn move_actor(
    world: &WorldGeometry,
    transform: &mut Transform,
    flight: &mut DroneFlight,
    input: &FlightInput,
    dt: f32,
) {
    let profile = FlightConfig {
        max_horizontal_speed: 525.,
        ..default()
    };
    let steps = (dt / (1. / 120.)).ceil().max(1.) as u32;
    for _ in 0..steps {
        flight.step_in_world(
            transform,
            input,
            &profile,
            &Arena::default(),
            DRONE_HALF_EXTENTS,
            dt / steps as f32,
            Some(world),
        );
    }
}
fn neutral() -> FlightInput {
    FlightInput {
        tilt: Vec2::ZERO,
        yaw: 0.,
        thrust: 1.,
    }
}
#[test]
fn terrain_stops_boosted_crossings_and_preserves_sliding_at_all_frame_rates() {
    let world = WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(1., 150., 270.),
        }],
        hazard: None,
    };
    for dt in [1. / 30., 1. / 60., 1. / 144., 0.1] {
        let mut transform = Transform::from_xyz(-80., 150., -100.);
        let mut flight = DroneFlight {
            velocity: Vec3::new(500., 0., 100.).normalize() * 525.,
            ..default()
        };
        for _ in 0..(0.5 / dt) as usize {
            move_actor(&world, &mut transform, &mut flight, &neutral(), dt);
        }
        assert!(
            transform.translation.x <= -36.,
            "dt {dt}: {:?}",
            transform.translation
        );
        assert!(transform.translation.z > -60.);
        assert!(flight.velocity.x.abs() < 0.01);
        assert!(flight.velocity.z > 80.);
    }
}
#[test]
fn turning_beside_wall_never_expands_body_into_solid() {
    let world = WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(1., 150., 270.),
        }],
        hazard: None,
    };
    for dt in [1. / 30., 1. / 60., 1. / 144., 0.1] {
        let mut transform = Transform::from_xyz(-36.1, 150., 0.);
        let mut flight = DroneFlight::default();
        let input = FlightInput {
            yaw: 1.,
            tilt: Vec2::X,
            thrust: 1.,
        };
        for _ in 0..(1. / dt) as usize {
            move_actor(&world, &mut transform, &mut flight, &input, dt);
            assert!(!world.solids[0].overlaps(
                transform.translation,
                world_half_extents(transform.rotation, DRONE_HALF_EXTENTS)
            ));
        }
    }
}
#[test]
fn low_cover_can_be_flown_over_but_full_height_wall_cannot() {
    for (height, y, blocked) in [(60., 90., false), (300., 280., true), (300., 20., true)] {
        let world = WorldGeometry {
            solids: vec![Solid {
                center: Vec3::new(0., height / 2., 0.),
                half: Vec3::new(5., height / 2., 100.),
            }],
            hazard: None,
        };
        for dt in [1. / 30., 1. / 60., 1. / 144., 0.1] {
            let mut t = Transform::from_xyz(-80., y, 0.);
            let mut f = DroneFlight {
                velocity: Vec3::X * 525.,
                ..default()
            };
            for _ in 0..(1. / dt) as usize {
                move_actor(&world, &mut t, &mut f, &neutral(), dt);
            }
            assert_eq!(t.translation.x < 0., blocked);
        }
    }
}

#[test]
fn corner_contacts_stop_both_inward_components_without_penetration() {
    let world = WorldGeometry {
        solids: vec![
            Solid {
                center: Vec3::new(0., 150., 0.),
                half: Vec3::new(1., 150., 270.),
            },
            Solid {
                center: Vec3::new(-240., 150., 0.),
                half: Vec3::new(240., 150., 1.),
            },
        ],
        hazard: None,
    };
    for dt in [1. / 30., 1. / 60., 1. / 144., 0.1] {
        let mut t = Transform::from_xyz(-100., 150., -100.);
        let mut f = DroneFlight {
            velocity: Vec3::new(360., 0., 360.),
            ..default()
        };
        for _ in 0..(1. / dt) as usize {
            move_actor(&world, &mut t, &mut f, &neutral(), dt);
            assert!(world.solids.iter().all(|s| !s.overlaps(
                t.translation,
                world_half_extents(t.rotation, DRONE_HALF_EXTENTS)
            )));
        }
        assert!(f.velocity.x.abs() < 0.001 && f.velocity.z.abs() < 0.001);
    }
}
#[test]
fn authored_routes_fit_scout_and_keep_spawn_and_chargers_clear() {
    let world = WorldGeometry::default();
    let half = Vec3::splat(DRONE_HALF_EXTENTS.length());
    assert!(world.clear_body(
        crate::arena::DRONE_START.translation,
        crate::arena::DRONE_START.translation,
        half
    ));
    for x in [-280., 280.] {
        let cylinder_bounds = Vec3::new(90., 80., 90.);
        assert!(
            world
                .solids
                .iter()
                .all(|s| !s.overlaps(Vec3::new(x, 80., 0.), cylinder_bounds))
        );
        assert!(
            !world
                .hazard
                .unwrap()
                .overlaps(Vec3::new(x, 80., 0.), cylinder_bounds)
        );
    }
    assert!(world.clear_body(Vec3::new(-280., 100., 0.), Vec3::new(280., 100., 0.), half));
    for (a, b) in [
        (Vec3::new(-280., 150., 0.), Vec3::new(25., 150., 195.)),
        (Vec3::new(25., 150., 195.), Vec3::new(215., 150., 195.)),
        (Vec3::new(215., 150., 195.), Vec3::new(280., 150., 0.)),
    ] {
        assert!(world.clear_body(a, b, half));
        assert!(world.hazard.unwrap().segment_hit(a, b, half).is_none());
    }
}

#[test]
fn wall_contact_allows_banking_away_from_rest() {
    let world = WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(1., 150., 270.),
        }],
        hazard: None,
    };
    let mut t = Transform::from_xyz(-36.1, 150., 0.);
    let mut f = DroneFlight::default();
    move_actor(
        &world,
        &mut t,
        &mut f,
        &FlightInput {
            tilt: Vec2::ZERO,
            yaw: 1.,
            thrust: 1.,
        },
        0.1,
    );
    for _ in 0..20 {
        move_actor(
            &world,
            &mut t,
            &mut f,
            &FlightInput {
                tilt: Vec2::NEG_X,
                yaw: 0.,
                thrust: 1.,
            },
            0.1,
        );
    }
    assert!(
        t.translation.x < -100.,
        "must back away: {:?}",
        t.translation
    );
}

#[test]
fn rotation_contact_does_not_create_timed_translation_for_stationary_actors() {
    let world = WorldGeometry {
        solids: vec![Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(1., 150., 270.),
        }],
        hazard: None,
    };
    // With no forces or initial velocity, banking/yawing may correct contact,
    // but it cannot produce translation over elapsed simulation time.
    let config = FlightConfig {
        gravity: 0.,
        ..default()
    };
    for half in [DRONE_HALF_EXTENTS, Vec3::splat(14.)] {
        for dt in [1. / 120., 1. / 144.] {
            let mut transform = Transform::from_xyz(-1. - half.x - 0.1, 150., 0.);
            let origin = transform.translation;
            let mut flight = DroneFlight::default();
            let path = flight.step_in_world(
                &mut transform,
                &FlightInput {
                    tilt: Vec2::X,
                    yaw: 1.,
                    thrust: 1.,
                },
                &config,
                &Arena::default(),
                half,
                dt,
                Some(&world),
            );
            assert!(
                transform.translation.x < origin.x - 0.1,
                "fixture must correct contact"
            );
            assert!(path.iter().any(|segment| segment.to > segment.from));
            for segment in path.iter().filter(|segment| segment.to > segment.from) {
                assert!(
                    segment.start.distance(segment.end) < 0.0001,
                    "contact correction was spread over elapsed time: {segment:?}"
                );
                assert!(segment.start.distance(transform.translation) < 0.0001);
            }
        }
    }
}
