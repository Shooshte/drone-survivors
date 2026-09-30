use super::*;
use crate::{
    arena::Drone,
    mission::objectives::{ObjectiveConfig, ObjectiveKind, ObjectiveRun},
    world::{MotionSegment, PlayerPath},
};

fn fixture(carrying: bool, enemy_x: Option<f32>, health: u32) -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<CombatConfig>()
        .init_resource::<CombatOutcomes>()
        .init_resource::<crate::energy::PowerFrame>()
        .init_resource::<crate::modules::ModuleConfig>()
        .init_resource::<bombs::BombState>()
        .insert_resource(GamePhase::Playing)
        .insert_resource(PlayerHealth {
            current: health,
            invulnerable_until: 0.,
        })
        .insert_resource(ObjectiveRun {
            kind: ObjectiveKind::Payload,
            visited: [carrying, false, false],
        })
        .insert_resource(ObjectiveConfig {
            sites: [Vec3::new(200., 90., 0.); 3],
            extraction: Vec3::new(800., 90., 0.),
            visit_radius: 50.,
            extraction_radius: 50.,
        })
        .insert_resource(PlayerPath {
            segments: vec![MotionSegment {
                start: Vec3::new(0., 90., 0.),
                end: Vec3::new(1000., 90., 0.),
                from: 0.,
                to: 1.,
                half: Vec3::new(35., 15., 45.),
            }],
        })
        .add_systems(Update, lifecycle::contact_damage);
    app.world_mut()
        .spawn((Drone, Transform::from_xyz(1000., 90., 0.)));
    if let Some(x) = enemy_x {
        let position = Vec3::new(x, 90., 0.);
        app.world_mut().spawn((
            Enemy {
                kind: crate::economy::runtime::EnemyKind::Chaser,
                health: 20,
                previous: position,
                path: vec![],
            },
            Transform::from_translation(position),
        ));
    }
    app
}

#[test]
fn payload_delivery_contact_resolves_in_travel_order_and_ties_win() {
    // Drone front=35, enemy half=14, collision epsilon=.001.
    for (enemy_x, expected) in [
        (Some(600.), GamePhase::Dead),
        (Some(799.001), GamePhase::Survived),
        (Some(950.), GamePhase::Survived),
        (None, GamePhase::Survived),
    ] {
        let mut app = fixture(true, enemy_x, 10);
        app.update();
        assert_eq!(
            *app.world().resource::<GamePhase>(),
            expected,
            "enemy x {enemy_x:?}"
        );
    }
}
#[test]
fn payload_can_collect_then_deliver_but_cannot_win_dead_or_without_cargo() {
    let mut app = fixture(false, None, 100);
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
    assert!(app.world().resource::<ObjectiveRun>().ready());
    let mut app = fixture(false, None, 0);
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    let mut app = fixture(false, None, 100);
    app.world_mut().resource_mut::<ObjectiveConfig>().sites = [Vec3::new(-200., 90., 0.); 3];
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}
#[test]
fn payload_delivery_before_pickup_is_not_retroactive_and_column_has_no_altitude_gate() {
    let mut app = fixture(false, None, 100);
    {
        let mut config = app.world_mut().resource_mut::<ObjectiveConfig>();
        config.sites = [Vec3::new(800., 90., 0.); 3];
        config.extraction = Vec3::new(200., 90., 0.);
    }
    for segment in &mut app.world_mut().resource_mut::<PlayerPath>().segments {
        segment.start.y = 260.;
        segment.end.y = 260.;
    }
    app.update();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!(app.world().resource::<ObjectiveRun>().ready());
}
