use super::tests::{app, launch, tick};
use super::*;

#[test]
fn reconnaissance_does_not_finish_at_survival_deadline() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<Campaign>()
        .progress
        .complete(MissionId::ALL[0]);
    app.world_mut()
        .resource_mut::<MissionSession>()
        .selected_mission = MissionId::ALL[1];
    launch(&mut app);
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!(app.world().resource::<Campaign>().history.is_empty());
}

use super::objectives::{ObjectiveConfig, ObjectiveKind, ObjectiveRun};
use crate::{
    arena::Drone,
    combat::PlayerHealth,
    economy::{Amounts, AttemptResources},
};

pub(super) fn visit(app: &mut App, position: Vec3) {
    app.world_mut()
        .query_filtered::<&mut Transform, With<Drone>>()
        .single_mut(app.world_mut())
        .unwrap()
        .translation = position;
    tick(app, 0., &[]);
}
fn objective_app(index: usize) -> App {
    let mut app = app();
    app.world_mut()
        .resource_mut::<Campaign>()
        .progress
        .complete(MissionId::ALL[0]);
    app.world_mut()
        .resource_mut::<MissionSession>()
        .selected_mission = MissionId::ALL[index];
    launch(&mut app);
    app
}
pub(super) fn finish_objective(app: &mut App) {
    let config = app.world().resource::<ObjectiveConfig>().clone();
    for site in config.sites {
        visit(app, site);
    }
    visit(app, config.extraction);
}

#[test]
fn distinct_visits_and_extraction_required_and_rewards_settle_once_for_both_types() {
    for index in [1, 2] {
        let mut app = objective_app(index);
        let config = app.world().resource::<ObjectiveConfig>().clone();
        visit(&mut app, config.extraction);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        for _ in 0..3 {
            visit(&mut app, config.sites[0]);
        }
        assert_eq!(app.world().resource::<ObjectiveRun>().count(), 1);
        visit(&mut app, config.sites[1]);
        visit(&mut app, config.extraction);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        visit(&mut app, config.sites[2]);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        assert_eq!(
            app.world().resource::<AttemptResources>().collected,
            Amounts::default()
        );
        app.world_mut().resource_mut::<AttemptResources>().collected = Amounts {
            salvage: 9,
            components: 2,
        };
        app.world_mut()
            .resource_mut::<crate::upgrades::UpgradeRun>()
            .award(50);
        visit(&mut app, config.extraction);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
        for _ in 0..3 {
            visit(&mut app, config.extraction);
        }
        let campaign = app.world().resource::<Campaign>();
        assert_eq!(campaign.history.len(), 1);
        assert_eq!(campaign.history[0].mission, MissionId::ALL[index]);
        assert_eq!(
            campaign.wallet,
            Amounts {
                salvage: 19,
                components: 3
            }
        );
        assert!(campaign.progress.completed(MissionId::ALL[index]));
    }
}

#[test]
fn death_beats_ready_extraction_and_never_unlocks_mission() {
    for index in [1, 2] {
        let mut app = objective_app(index);
        let config = app.world().resource::<ObjectiveConfig>().clone();
        for site in config.sites {
            visit(&mut app, site);
        }
        app.world_mut().resource_mut::<PlayerHealth>().current = 0;
        app.world_mut()
            .resource_mut::<AttemptResources>()
            .collected
            .salvage = 9;
        visit(&mut app, config.extraction);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
        let campaign = app.world().resource::<Campaign>();
        assert_eq!(campaign.wallet.salvage, 2);
        assert!(!campaign.progress.completed(MissionId::ALL[index]));
        assert_eq!(campaign.history.len(), 1);
        assert!(!campaign.history[0].succeeded);
    }
}

#[test]
fn choice_pauses_triggers_and_restart_uses_active_mission_and_clears_progress() {
    for index in [1, 2] {
        let mut app = objective_app(index);
        let config = app.world().resource::<ObjectiveConfig>().clone();
        visit(&mut app, config.sites[0]);
        app.world_mut()
            .resource_mut::<crate::upgrades::UpgradeRun>()
            .award(50);
        tick(&mut app, 0., &[]);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
        visit(&mut app, config.sites[1]);
        tick(&mut app, 30., &[]);
        assert_eq!(app.world().resource::<ObjectiveRun>().count(), 1);
        app.world_mut()
            .resource_mut::<MissionSession>()
            .selected_mission = MissionId::ALL[0];
        app.world_mut().resource_mut::<PlayerHealth>().current = 0;
        tick(&mut app, 0., &[KeyCode::KeyR]);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        assert_eq!(app.world().resource::<ObjectiveRun>().count(), 0);
        assert_eq!(
            app.world().resource::<ObjectiveRun>().kind,
            MissionId::ALL[index].objective()
        );
        assert!(app.world().resource::<Campaign>().history.is_empty());
        finish_objective(&mut app);
        assert_eq!(app.world().resource::<Campaign>().history[0].attempt, 2);
    }
}

#[test]
fn visits_require_three_dimensional_proximity_and_clear_line_of_sight() {
    let mut app = objective_app(1);
    let config = app.world().resource::<ObjectiveConfig>().clone();
    visit(
        &mut app,
        config.sites[0] + Vec3::Y * (config.visit_radius + 1.),
    );
    assert_eq!(app.world().resource::<ObjectiveRun>().count(), 0);
    let mut geometry = crate::world::WorldGeometry {
        solids: vec![],
        ..default()
    };
    geometry.solids.push(crate::world::Solid {
        center: config.sites[0] + Vec3::X * 30.,
        half: Vec3::new(5., 100., 100.),
    });
    app.insert_resource(geometry);
    visit(&mut app, config.sites[0] + Vec3::X * 60.);
    assert_eq!(app.world().resource::<ObjectiveRun>().count(), 0);
    visit(&mut app, config.sites[0]);
    assert_eq!(app.world().resource::<ObjectiveRun>().count(), 1);
}

#[test]
fn objective_elapsed_time_continues_after_wave_schedule_and_survival_stays_capped() {
    for index in [1, 2] {
        let mut app = objective_app(index);
        app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
        tick(&mut app, 1., &[]);
        assert_eq!(app.world().resource::<Encounter>().elapsed, 301.);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    }
    let mut app = app();
    launch(&mut app);
    assert_eq!(
        app.world().resource::<ObjectiveRun>().kind,
        ObjectiveKind::Survival
    );
    app.world_mut().resource_mut::<Encounter>().elapsed = 299.;
    tick(&mut app, 2., &[]);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 300.);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
}

#[test]
fn placeholder_sites_and_extraction_have_body_clearance_and_connected_routes() {
    let config = ObjectiveConfig::default();
    let geometry = crate::world::WorldGeometry::default();
    let arena = crate::arena::Arena::default();
    let half = Vec3::splat(crate::arena::DRONE_HALF_EXTENTS.length());
    let points: Vec<_> = std::iter::once(crate::arena::DRONE_START.translation)
        .chain(config.sites)
        .chain([config.extraction])
        .collect();
    for &point in &points {
        assert!(point.y - half.y >= 0. && point.y + half.y <= arena.half_size.y * 2.);
        assert!(
            point.x.abs() + half.x < arena.half_size.x
                && point.z.abs() + half.z < arena.half_size.z
        );
        assert!(!geometry.solids.iter().any(|s| s.overlaps(point, half)));
        assert!(!geometry.hazard.is_some_and(|s| s.overlaps(point, half)));
        for &target in &points {
            assert!(crate::world::navigation::next_point(&geometry, point, target, half).is_some());
        }
    }
}

#[test]
fn extraction_banks_loot_collected_on_the_same_frame() {
    let mut app = objective_app(2);
    let config = app.world().resource::<ObjectiveConfig>().clone();
    for point in config.sites {
        visit(&mut app, point);
    }
    app.world_mut().spawn((
        crate::economy::runtime::Pickup {
            amount: Amounts {
                salvage: 4,
                components: 0,
            },
            radius: 100.,
            attracted: false,
        },
        Transform::from_translation(config.extraction),
    ));
    visit(&mut app, config.extraction);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 14);
}

#[test]
fn guidance_skips_visited_sites_and_routes_to_extraction_only_when_ready() {
    let config = ObjectiveConfig::default();
    let mut run = ObjectiveRun {
        kind: ObjectiveKind::Reconnaissance,
        ..default()
    };
    assert_eq!(run.next_target(&config, config.sites[0]).0, Some(0));
    run.visited[0] = true;
    assert_ne!(run.next_target(&config, config.sites[0]).0, Some(0));
    assert!(
        run.guidance(&config, config.extraction)
            .contains("SCAN 1/3")
    );
    assert!(
        !run.guidance(&config, config.extraction)
            .contains("EXTRACT:")
    );
    run.visited = [true; 3];
    assert_eq!(
        run.next_target(&config, config.sites[0]),
        (None, config.extraction)
    );
    assert!(
        run.guidance(&config, config.sites[0])
            .contains("EXTRACT: S")
    );
    assert!(
        run.guidance(&config, config.extraction + Vec3::Y * 100.)
            .contains("HERE")
    );
}
