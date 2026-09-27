//! Mission 01 schematic blockout: north is negative Z; every ridge spans flight height.
use super::{Solid, WorldGeometry, environment::DirectionalField};
use crate::arena::Arena;
use bevy::prelude::*;

/// 12 world units per schematic unit: compact playtest target of about one minute.
pub(crate) const SCALE: f32 = 12.;
pub(crate) fn point(east: f32, south: f32) -> Vec3 {
    Vec3::new((east - 500.) * SCALE, 90., (south - 500.) * SCALE)
}
pub(crate) fn arena() -> Arena {
    Arena {
        half_size: Vec3::new(500. * SCALE, 150., 500. * SCALE),
    }
}
pub(crate) fn start() -> Vec3 {
    point(875., 850.)
}
pub(crate) fn pickup() -> Vec3 {
    point(125., 575.)
}
pub(crate) fn delivery() -> Vec3 {
    point(860., 180.)
}
pub(crate) fn hidden() -> Vec3 {
    point(535., 935.)
}
pub(crate) fn challenge() -> Vec3 {
    point(235., 855.)
}
pub(crate) fn chargers() -> [(&'static str, Vec3); 4] {
    [
        ("WEST", point(30., 435.).with_y(0.)),
        ("NORTH", point(568., 312.).with_y(0.)),
        ("CHALLENGE", point(195., 964.).with_y(0.)),
        ("HIDDEN", point(577., 965.).with_y(0.)),
    ]
}
pub(crate) fn route() -> Vec<Vec3> {
    [
        (875., 850.),
        (690., 805.),
        (550., 750.),
        (520., 665.),
        (410., 615.),
        (225., 575.),
        (125., 575.),
        (210., 440.),
        (260., 390.),
        (360., 340.),
        (550., 335.),
        (710., 270.),
        (860., 180.),
    ]
    .into_iter()
    .map(|(x, z)| point(x, z))
    .collect()
}
fn rectangle(west: f32, north: f32, east: f32, south: f32) -> Solid {
    Solid {
        center: point((west + east) * 0.5, (north + south) * 0.5).with_y(150.),
        half: Vec3::new(
            (east - west) * SCALE * 0.5,
            150.,
            (south - north) * SCALE * 0.5,
        ),
    }
}
pub(crate) fn fields() -> Vec<DirectionalField> {
    [
        (305., 46., 440., 86., Vec3::X),
        (180., 200., 280., 300., Vec3::new(77., 0., 74.)),
        (320., 640., 415., 695., Vec3::new(-81., 0., 34.)),
        (220., 595., 290., 645., Vec3::new(-51., 0., -32.)),
    ]
    .into_iter()
    .map(|(w, n, e, s, direction)| DirectionalField {
        bounds: rectangle(w, n, e, s),
        direction: direction.normalize(),
        cycling: false,
    })
    .collect()
}
pub(crate) fn geometry() -> WorldGeometry {
    // Seven staircase silhouettes retain the sketch's connected ridges, islands,
    // passages and optional pockets. They use the shared swept-AABB collision path.
    let rectangles = [
        (0., 255., 130., 300.),
        (0., 300., 190., 355.),
        (0., 355., 170., 390.),
        (0., 390., 85., 410.),
        (310., 145., 390., 265.),
        (390., 155., 450., 295.),
        (450., 205., 480., 295.),
        (480., 245., 505., 270.),
        (300., 440., 450., 475.),
        (270., 475., 580., 510.),
        (280., 510., 1000., 550.),
        (360., 550., 1000., 580.),
        (450., 580., 1000., 615.),
        (570., 615., 1000., 650.),
        (575., 650., 1000., 700.),
        (600., 700., 1000., 750.),
        (700., 750., 1000., 775.),
        (190., 655., 220., 695.),
        (220., 680., 255., 725.),
        (255., 705., 285., 750.),
        (285., 720., 335., 770.),
        (335., 720., 380., 755.),
        (380., 700., 435., 740.),
        (0., 670., 60., 715.),
        (0., 715., 110., 765.),
        (0., 765., 125., 830.),
        (0., 830., 110., 880.),
        (0., 880., 80., 925.),
        (0., 925., 105., 965.),
        (0., 965., 130., 1000.),
        (390., 780., 570., 800.),
        (365., 800., 570., 835.),
        (350., 835., 570., 885.),
        (345., 885., 480., 915.),
        (330., 915., 475., 950.),
        (300., 950., 470., 1000.),
        (720., 845., 765., 885.),
        (700., 885., 780., 930.),
        (660., 930., 800., 955.),
        (620., 955., 830., 980.),
        (605., 980., 865., 1000.),
    ];
    let mut world = WorldGeometry {
        solids: rectangles
            .into_iter()
            .map(|(w, n, e, s)| rectangle(w, n, e, s))
            .collect(),
        hazard: None,
        navigation: None,
    };
    let mut points = route();
    points.extend(
        [
            (235., 855.),
            (170., 750.),
            (160., 660.),
            (625., 860.),
            (600., 920.),
            (535., 935.),
            (577., 965.),
            (195., 964.),
            (65., 470.),
            (30., 435.),
            (245., 285.),
            (250., 175.),
            (280., 90.),
            (500., 70.),
            (685., 115.),
            (960., 65.),
            (568., 312.),
            (860., 385.),
            (80., 85.),
            (100., 190.),
            (935., 900.),
            (875., 965.),
            (600., 805.),
        ]
        .into_iter()
        .map(|(x, z)| point(x, z)),
    );
    world.navigation = Some(super::navigation::NavigationGraph::build(
        &world,
        points,
        Vec3::splat(90.),
    ));
    world
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::{DRONE_HALF_EXTENTS, world_half_extents};

    fn schematic(east: f32, south: f32) -> Vec3 {
        point(east, south)
    }

    #[test]
    fn mission01_terrain_preserves_seven_full_height_landforms() {
        let world = geometry();
        for (east, south) in [
            (50., 330.),
            (400., 220.),
            (800., 600.),
            (310., 735.),
            (50., 800.),
            (420., 840.),
            (750., 920.),
        ] {
            for height in [15., 90., 285.] {
                let at = schematic(east, south).with_y(height);
                assert!(
                    world.solids.iter().any(|s| s.overlaps(at, Vec3::ONE)),
                    "missing full-height solid at {east}, {south}, {height}"
                );
            }
        }
    }

    #[test]
    fn mission01_schematic_passages_fit_full_rotated_scout() {
        let world = geometry();
        let routes: &[&[(f32, f32)]] = &[
            &[
                (875., 850.),
                (690., 805.),
                (550., 750.),
                (520., 665.),
                (410., 615.),
                (225., 575.),
                (125., 575.),
                (210., 440.),
                (260., 390.),
                (360., 340.),
                (550., 335.),
                (710., 270.),
                (860., 180.),
            ],
            &[(235., 855.), (170., 750.), (160., 660.), (225., 575.)],
            &[
                (690., 805.),
                (625., 860.),
                (600., 920.),
                (535., 935.),
                (577., 965.),
            ],
            &[(235., 855.), (195., 964.)],
            &[(125., 575.), (65., 470.), (30., 435.)],
            &[
                (260., 390.),
                (245., 285.),
                (250., 175.),
                (280., 90.),
                (500., 70.),
                (685., 115.),
                (860., 180.),
                (960., 65.),
            ],
            &[(550., 335.), (568., 312.), (685., 115.)],
            &[(550., 335.), (860., 385.), (860., 180.)],
        ];
        for yaw in 0..8 {
            for pitch in [-30_f32, 0., 30.] {
                let rotation = Quat::from_rotation_y(yaw as f32 * std::f32::consts::FRAC_PI_4)
                    * Quat::from_rotation_x(pitch.to_radians());
                let half = world_half_extents(rotation, DRONE_HALF_EXTENTS);
                for route in routes {
                    for pair in route.windows(2) {
                        assert!(
                            world.clear_body(
                                schematic(pair[0].0, pair[0].1),
                                schematic(pair[1].0, pair[1].1),
                                half
                            ),
                            "blocked passage {pair:?}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn mission01_navigation_connects_landmarks_around_ridges_and_pockets() {
        let world = geometry();
        let half = Vec3::splat(60.);
        for origin in [
            start(),
            point(80., 85.),
            point(960., 65.),
            challenge(),
            hidden(),
        ] {
            for target in [pickup(), delivery(), start(), challenge(), hidden()] {
                let mut at = origin;
                for _ in 0..600 {
                    if at.distance(target) < 30. {
                        break;
                    }
                    let next = super::super::navigation::next_point(&world, at, target, half)
                        .unwrap_or_else(|| panic!("missing route from {at:?} to {target:?}"));
                    let end = at + (next - at).clamp_length_max(250.);
                    assert!(world.clear_body(at, end, half));
                    at = end;
                }
                assert!(
                    at.distance(target) < 30.,
                    "stalled from {origin:?} at {at:?} toward {target:?}"
                );
            }
        }
    }

    #[test]
    fn mission01_navigation_preserves_body_clear_final_approach_beside_ridge() {
        let world = geometry();
        let start = point(440., 430.);
        // Keep the same body clearance from the ridge at any map scale.
        let target = point(440., 440.) - Vec3::Z * 35.1;
        let half = Vec3::splat(24.);
        assert!(world.clear_body(start, target, half));
        assert!(!world.clear_body(start, target, half + Vec3::splat(12.)));
        assert_eq!(
            super::super::navigation::next_point(&world, start, target, half),
            Some(target),
            "a clear final approach must not detour to a cached anchor"
        );
    }

    #[test]
    fn mission01_ordinary_rotor_flight_calibrates_main_route() {
        use crate::arena::{DroneFlight, FlightConfig, FlightInput, VerticalControl};
        let world = geometry();
        let arena = arena();
        let config = FlightConfig::default();
        let mut transform = Transform::from_translation(start());
        let mut flight = DroneFlight::default();
        let mut elapsed = 0.;
        for goal in route().into_iter().skip(1) {
            let deadline = elapsed + 60.;
            while transform.translation.distance(goal) > 120. && elapsed < deadline {
                let offset = (goal - transform.translation).with_y(0.);
                let desired = offset.normalize_or_zero()
                    * config.max_horizontal_speed.min(offset.length() * 2.);
                let acceleration = (desired - flight.velocity.with_y(0.)) * 3.
                    + flight.velocity.with_y(0.) * config.horizontal_drag;
                let angle = (acceleration.length()
                    / (config.gravity * config.horizontal_acceleration_multiplier))
                    .clamp(0., 1.)
                    .asin()
                    .min(config.max_tilt);
                let input = FlightInput {
                    tilt: Vec2::new(acceleration.x, -acceleration.z).normalize_or_zero()
                        * (angle / config.max_tilt),
                    yaw: 0.,
                    yaw_override: true,
                    thrust: 1.,
                    vertical: VerticalControl::AltitudeHold,
                };
                flight.step_in_world(
                    &mut transform,
                    &input,
                    &config,
                    &arena,
                    DRONE_HALF_EXTENTS,
                    1. / 120.,
                    Some(&world),
                );
                assert!(world.solids.iter().all(|s| !s.overlaps(
                    transform.translation,
                    world_half_extents(transform.rotation, DRONE_HALF_EXTENTS)
                )));
                elapsed += 1. / 120.;
            }
            assert!(
                transform.translation.distance(goal) <= 120.,
                "flight stalled at {:?} toward {goal:?}",
                transform.translation
            );
        }
        let length: f32 = route().windows(2).map(|p| p[0].distance(p[1])).sum();
        eprintln!(
            "Mission 01 route: {length:.0} world units, ordinary rotor flight {elapsed:.2}s, no equipment/boosts, altitude hold, 120Hz"
        );
        assert!(
            (50. ..75.).contains(&elapsed),
            "ordinary route took {elapsed}s"
        );
    }

    #[test]
    fn mission01_landmarks_chargers_and_fields_are_clear_and_bounded() {
        let world = geometry();
        let arena = arena();
        for at in [start(), pickup(), delivery(), hidden(), challenge()]
            .into_iter()
            .chain(chargers().map(|(_, at)| at.with_y(90.)))
        {
            assert!(world.clear_body(at, at, Vec3::splat(DRONE_HALF_EXTENTS.length())));
            assert!((at - arena.center()).abs().cmplt(arena.half_size).all());
        }
        for field in fields() {
            assert!(world.clear_body(field.bounds.center, field.bounds.center, Vec3::splat(60.)));
            assert!((field.direction.length() - 1.).abs() < 0.0001);
        }
    }
}
