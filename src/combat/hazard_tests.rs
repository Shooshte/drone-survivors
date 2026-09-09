use super::*;
use crate::world::{
    Solid, WorldGeometry,
    hazard::{HazardPhase, HazardState},
};

fn hazard_app() -> (App, Entity) {
    let (mut app, drone) = empty_app();
    quiet(&mut app);
    app.insert_resource(WorldGeometry {
        solids: vec![],
        hazard: Some(Solid {
            center: Vec3::new(0., 150., 0.),
            half: Vec3::new(60., 150., 60.),
        }),
    });
    (app, drone)
}

fn activate(app: &mut App, cycle: u64) {
    let mut state = app.world_mut().resource_mut::<HazardState>();
    state.phase = HazardPhase::Active;
    state.elapsed = 0.;
    state.cycle = cycle;
}

#[test]
fn hazard_warns_then_hits_only_once_per_actor_per_window() {
    let (mut app, _) = hazard_app();
    step(&mut app, 3., &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(
        app.world().resource::<HazardState>().phase,
        HazardPhase::Warning
    );
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    step(&mut app, 0.6, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 90);
    step(&mut app, 0.8, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 90);
}

#[test]
fn shield_and_contact_share_protection_and_toggling_does_not_rearm_window() {
    let (mut app, drone) = hazard_app();
    activate(&mut app, 1);
    let chaser = enemy(&mut app, START, 100);
    step(&mut app, 0.1, &[KeyCode::Digit2]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(
        app.world()
            .resource::<crate::modules::Modules>()
            .shield
            .blocks,
        0
    );
    app.world_mut().despawn(chaser);
    step(&mut app, 0.4, &[]);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation
        .x = 200.;
    step(&mut app, 0.01, &[KeyCode::Digit2]);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation
        .x = 0.;
    step(&mut app, 0.4, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    activate(&mut app, 2);
    step(&mut app, 0.1, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 90);
}

#[test]
fn depleted_shield_does_not_block_and_hazard_kill_is_counted_once() {
    let (mut app, _) = hazard_app();
    activate(&mut app, 1);
    app.world_mut()
        .resource_mut::<crate::energy::Energy>()
        .current = 0.01;
    app.world_mut()
        .resource_mut::<crate::modules::Modules>()
        .enabled[1] = true;
    let chaser = enemy(&mut app, START + Vec3::Z * 65., 10);
    step(&mut app, 0.1, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 90);
    assert!(app.world().get_entity(chaser).is_err());
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
    step(&mut app, 0.8, &[]);
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
}

#[test]
fn hazard_death_freezes_and_restart_restores_cycle_without_damage() {
    let (mut app, _) = hazard_app();
    activate(&mut app, 5);
    app.world_mut().resource_mut::<PlayerHealth>().current = 1;
    step(&mut app, 0.1, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    let elapsed = app.world().resource::<HazardState>().elapsed;
    step(&mut app, 20., &[]);
    assert_eq!(app.world().resource::<HazardState>().elapsed, elapsed);
    step(&mut app, 1., &[KeyCode::KeyR, KeyCode::Digit2]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    let hazard = app.world().resource::<HazardState>();
    assert_eq!(hazard.phase, HazardPhase::Inactive);
    assert_eq!(hazard.cycle, 0);
}

#[test]
fn physical_crossing_hits_even_when_both_frame_endpoints_are_outside() {
    for active_elapsed in [0., 0.9] {
        let (mut app, drone) = hazard_app();
        app.init_resource::<crate::world::PlayerPath>();
        activate(&mut app, 1);
        app.world_mut().resource_mut::<HazardState>().elapsed = active_elapsed;
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation
            .x = -160.;
        app.world_mut()
            .get_mut::<crate::arena::DroneFlight>(drone)
            .unwrap()
            .velocity
            .x = 420.;
        step(&mut app, 0.8, &[]);
        assert!(position(&app, drone).x > 95.);
        assert_eq!(
            app.world().resource::<PlayerHealth>().current,
            if active_elapsed == 0. { 90 } else { 100 }
        );
    }
}
