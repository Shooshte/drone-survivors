use super::*;
use crate::mission::{
    Campaign, MissionSession,
    tests::{app as mission_app, launch, tick},
};
use crate::{
    arena::{DRONE_START, DroneFlight, FlightConfig},
    energy::{ChargerReserve, Energy, EnergyConfig, PowerFrame},
    modules::{ModuleConfig, Modules},
    upgrades::{UpgradeKind, UpgradeRun, runtime::ExplorationPickup},
    world::hazard::{HazardPhase, HazardState},
};

#[test]
fn launch_resets_dirty_baseline_transients_cooldowns_and_all_run_bookkeeping() {
    let mut app = mission_app();
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    // Dirty every run-owned subsystem before first launch, including derived stats.
    app.world_mut()
        .resource_mut::<FlightConfig>()
        .max_horizontal_speed = 1.;
    app.world_mut().resource_mut::<CombatConfig>().player_health = 150;
    app.world_mut().resource_mut::<EnergyConfig>().capacity = 75.;
    app.world_mut()
        .resource_mut::<ModuleConfig>()
        .shield_recharge = 2.5;
    *app.world_mut().get_mut::<Transform>(drone).unwrap() =
        Transform::from_xyz(100., 200., 300.).with_rotation(Quat::from_rotation_y(1.));
    *app.world_mut().get_mut::<DroneFlight>(drone).unwrap() = DroneFlight {
        velocity: Vec3::ONE,
        heading: 1.,
        tilt: Vec2::ONE,
    };
    {
        let mut hp = app.world_mut().resource_mut::<PlayerHealth>();
        hp.current = 1;
        hp.invulnerable_until = 900.;
    }
    app.world_mut().resource_mut::<Weapon>().ready_at = 900.;
    app.world_mut().resource_mut::<RocketLauncher>().ready_at = 900.;
    app.world_mut()
        .resource_mut::<RocketLauncher>()
        .retime(0., 900.);
    {
        let mut modules = app.world_mut().resource_mut::<Modules>();
        modules.enabled = [true; 4];
        modules.rejected_for = [1.; 4];
        modules.shield.blocks = 0;
        modules.shield.remaining = 3.;
    }
    app.world_mut().resource_mut::<Energy>().current = 1.;
    for mut reserve in app
        .world_mut()
        .query::<&mut ChargerReserve>()
        .iter_mut(app.world_mut())
    {
        reserve.remaining = 1.;
        reserve.away_seconds = 7.;
        reserve.occupied = true;
    }
    {
        let mut run = app.world_mut().resource_mut::<Encounter>();
        run.elapsed = 50.;
        run.kills = 20;
        run.next_burst = 4;
        run.candidate = 30;
        run.spawns.admitted = 12;
    }
    {
        let mut run = app.world_mut().resource_mut::<UpgradeRun>();
        run.award(1000);
        run.selected.push(UpgradeKind::HeavyArmor);
    }
    app.world_mut()
        .resource_mut::<ExplorationPickup>()
        .collected = true;
    {
        let mut state = app.world_mut().resource_mut::<HazardState>();
        state.phase = HazardPhase::Active;
        state.elapsed = 0.5;
        state.cycle = 5;
    }
    let foe = enemy(&mut app, START + Vec3::X * 200., 20);
    let projectile = shot(&mut app, START, Vec3::ONE, 1.);
    let warning = app
        .world_mut()
        .spawn((SpawnWarning::default(), Transform::default()))
        .id();
    let effect = app
        .world_mut()
        .spawn(feedback::KillEffect { remaining: 1. })
        .id();
    launch(&mut app);
    for entity in [foe, projectile, warning, effect] {
        assert!(app.world().get_entity(entity).is_err());
    }
    assert_eq!(app.world().get::<Transform>(drone).unwrap(), &DRONE_START);
    assert_eq!(
        *app.world().get::<DroneFlight>(drone).unwrap(),
        DroneFlight::default()
    );
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(
        app.world().resource::<PlayerHealth>().invulnerable_until,
        0.
    );
    assert_eq!(app.world().resource::<Weapon>().ready_at, 0.);
    assert_eq!(app.world().resource::<RocketLauncher>().ready_at, 0.);
    assert_eq!(
        app.world().resource::<FlightConfig>().max_horizontal_speed,
        FlightConfig::default().max_horizontal_speed
    );
    assert_eq!(app.world().resource::<EnergyConfig>().capacity, 100.);
    assert_eq!(app.world().resource::<Energy>().current, 100.);
    assert_eq!(
        app.world().resource::<ModuleConfig>().shield_recharge,
        ModuleConfig::default().shield_recharge
    );
    let modules = app.world().resource::<Modules>();
    assert_eq!(modules.enabled, [false; 4]);
    assert_eq!(modules.rejected_for, [0.; 4]);
    assert_eq!(modules.shield.blocks, ModuleConfig::default().shield_blocks);
    assert_eq!(modules.shield.remaining, 0.);
    for reserve in app.world_mut().query::<&ChargerReserve>().iter(app.world()) {
        assert_eq!(reserve.remaining, 200.);
        assert_eq!(reserve.away_seconds, 0.);
        assert!(!reserve.occupied);
    }
    let run = app.world().resource::<Encounter>();
    assert_eq!(run.elapsed, 0.);
    assert_eq!(run.kills, 0);
    assert_eq!(run.next_burst, 0);
    assert_eq!(run.candidate, 0);
    assert_eq!(run.spawns, default());
    let upgrades = app.world().resource::<UpgradeRun>();
    assert_eq!(upgrades.total_xp, 0);
    assert_eq!(upgrades.pending, 0);
    assert_eq!(upgrades.resolved, 0);
    assert!(upgrades.offer.is_empty());
    assert!(upgrades.selected.is_empty());
    assert!(!app.world().resource::<ExplorationPickup>().collected);
    let hazard = app.world().resource::<HazardState>();
    assert_eq!(hazard.phase, HazardPhase::Inactive);
    assert_eq!(hazard.cycle, 0);
    assert_eq!(hazard.elapsed, 0.);
    assert!(hazard.active_window.is_none());
    assert!(app.world().resource::<PowerFrame>().chargers.is_empty());
}

#[test]
fn fatal_contact_beats_deadline_and_pending_upgrade_then_cleans_entities_once() {
    let mut app = mission_app();
    launch(&mut app);
    app.world_mut().resource_mut::<Encounter>().elapsed = 299.99;
    app.world_mut().resource_mut::<PlayerHealth>().current = 1;
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    enemy(&mut app, START, 20);
    tick(&mut app, 0.01, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    let result = app
        .world()
        .resource::<MissionSession>()
        .result
        .as_ref()
        .unwrap();
    assert!(!result.succeeded);
    assert_eq!(result.elapsed, 300.);
    assert_eq!(count::<Enemy>(&mut app), 0);
    assert_eq!(count::<Projectile>(&mut app), 0);
    tick(&mut app, 20., &[]);
    assert_eq!(app.world().resource::<Campaign>().history.len(), 1);
}

#[test]
fn restart_beats_fatal_contact_and_deadline_and_replacement_has_new_identity() {
    let mut app = mission_app();
    launch(&mut app);
    app.world_mut().resource_mut::<Encounter>().elapsed = 299.99;
    app.world_mut().resource_mut::<PlayerHealth>().current = 1;
    enemy(&mut app, START, 20);
    tick(&mut app, 1., &[KeyCode::KeyR]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    assert!(app.world().resource::<Campaign>().history.is_empty());
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Survived;
    tick(&mut app, 0., &[]);
    assert_eq!(
        app.world()
            .resource::<MissionSession>()
            .result
            .as_ref()
            .unwrap()
            .attempt,
        2
    );
}
