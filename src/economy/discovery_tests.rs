//! Exploration rewards and region configuration through the live collection rules.
use super::*;

#[test]
fn region_launch_configures_real_loot_and_chargers_without_moving_sites() {
    use crate::{
        energy::{ChargerReserve, ChargingNode},
        mission::{Campaign, MissionSession, campaign::MissionId},
    };
    let mut app = crate::mission::tests::app();
    let original_sites = app.world().resource::<EconomyConfig>().caches;
    let mut original_chargers = app
        .world_mut()
        .query::<&ChargingNode>()
        .iter(app.world())
        .map(|n| n.center.to_array())
        .collect::<Vec<_>>();
    original_chargers.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for (index, chance, components, capacity) in [
        (0, 50, 1, 200.),
        (4, 25, 2, 200.),
        (8, 25, 1, 300.),
        (0, 50, 1, 200.),
    ] {
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Hub;
        for id in &MissionId::ALL[..index] {
            app.world_mut()
                .resource_mut::<Campaign>()
                .progress
                .complete(*id);
        }
        app.world_mut()
            .resource_mut::<MissionSession>()
            .selected_mission = MissionId::ALL[index];
        crate::mission::tests::launch(&mut app);
        let config = app.world().resource::<EconomyConfig>();
        assert_eq!(config.caches, original_sites);
        assert_eq!(
            config.chaser.chance_percent,
            chance,
            "mission {}",
            index + 1
        );
        let cache_amounts = app
            .world_mut()
            .query_filtered::<&Pickup, With<ComponentCache>>()
            .iter(app.world())
            .map(|p| p.amount.components)
            .collect::<Vec<_>>();
        assert_eq!(cache_amounts, vec![components; 3]);
        let mut chargers = Vec::new();
        for (node, reserve) in app
            .world_mut()
            .query::<(&ChargingNode, &ChargerReserve)>()
            .iter(app.world())
        {
            chargers.push(node.center.to_array());
            assert_eq!(reserve.remaining, capacity);
        }
        chargers.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(chargers, original_chargers);
    }
}

#[test]
fn discovery_awards_xp_once_per_attempt_and_fast_crossings_collect() {
    use crate::{
        upgrades::UpgradeRun,
        world::{MotionSegment, PlayerPath},
    };
    let mut app = fixture();
    app.init_resource::<UpgradeRun>()
        .init_resource::<PlayerPath>();
    app.world_mut().run_system_once(reset).unwrap();
    let center = app.world().resource::<EconomyConfig>().caches[0];
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = center + Vec3::new(100., 20., 0.);
    app.world_mut()
        .resource_mut::<PlayerPath>()
        .segments
        .push(MotionSegment {
            start: center + Vec3::new(-100., 20., 0.),
            end: center + Vec3::new(100., 20., 0.),
            from: 0.,
            to: 1.,
            half: crate::arena::DRONE_HALF_EXTENTS,
        });
    run(&mut app);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 30);
    assert_eq!(
        app.world()
            .resource::<AttemptResources>()
            .collected
            .components,
        1
    );
    run(&mut app);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 30);
    app.world_mut().run_system_once(reset).unwrap();
    run(&mut app);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 60);
}

#[test]
fn discovery_unlocks_are_durable_while_xp_and_sites_reset() {
    use crate::{mission::Campaign, upgrades::UpgradeRun};
    let mut app = crate::mission::tests::app();
    crate::mission::tests::launch(&mut app);
    let sites = app.world().resource::<EconomyConfig>().caches;
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = sites[0] + Vec3::Y * 20.;
    crate::mission::tests::tick(&mut app, 0., &[]);
    assert!(app.world().resource::<Campaign>().secrets.blueprint);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 30);
    crate::mission::tests::tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 30);
    crate::mission::tests::tick(&mut app, 0., &[KeyCode::KeyR]);
    assert!(app.world().resource::<Campaign>().secrets.blueprint);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = sites[1] + Vec3::Y * 20.;
    crate::mission::tests::tick(&mut app, 0., &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 30);
    assert_eq!(
        app.world().resource::<Campaign>().progress.routes(),
        [true, false, false]
    );
    assert_eq!(app.world().resource::<Campaign>().progress.count(), 0);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    crate::mission::tests::tick(&mut app, 0., &[]);
    assert!(app.world().resource::<Campaign>().secrets.blueprint);
    assert!(app.world().resource::<Campaign>().progress.routes()[0]);
}

#[test]
fn discovery_never_grants_through_cover_pause_terminal_reset_or_full_counter() {
    use crate::{mission::Campaign, upgrades::UpgradeRun};
    let mut app = fixture();
    app.init_resource::<Campaign>()
        .init_resource::<UpgradeRun>();
    let id = pickup(&mut app, Vec3::new(0., 30., 40.), true);
    app.world_mut().entity_mut(id).insert(DiscoverySite(0));
    for phase in [
        GamePhase::Choosing,
        GamePhase::Dead,
        GamePhase::Survived,
        GamePhase::Hub,
        GamePhase::Briefing,
        GamePhase::CampaignMenu,
    ] {
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        run(&mut app);
    }
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
    app.world_mut().resource_mut::<MissionBoundary>().reset = true;
    run(&mut app);
    app.world_mut().resource_mut::<MissionBoundary>().reset = false;
    app.world_mut()
        .resource_mut::<AttemptResources>()
        .collected
        .components = u64::MAX;
    run(&mut app);
    app.world_mut()
        .resource_mut::<AttemptResources>()
        .collected
        .components = 0;
    app.world_mut()
        .resource_mut::<WorldGeometry>()
        .solids
        .push(crate::world::Solid {
            center: Vec3::new(0., 30., 20.),
            half: Vec3::new(10., 30., 1.),
        });
    run(&mut app);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
    assert!(!app.world().resource::<Campaign>().secrets.blueprint);
    assert!(app.world().get_entity(id).is_ok());
    app.world_mut().resource_mut::<WorldGeometry>().solids.pop();
    run(&mut app);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 30);
    assert!(app.world().resource::<Campaign>().secrets.blueprint);
}
