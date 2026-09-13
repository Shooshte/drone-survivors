use super::*;
use crate::{
    economy::{
        Amounts, AttemptResources,
        runtime::{ComponentCache, EconomyConfig, Pickup},
    },
    mission::{
        Campaign, MissionSession,
        tests::{app as mission_app, launch, tick},
    },
};
fn economy_app() -> App {
    let mut app = mission_app();
    launch(&mut app);
    quiet(&mut app);
    app.world_mut().resource_mut::<WaveConfig>().bursts.clear();
    app.world_mut()
        .resource_mut::<EconomyConfig>()
        .chaser
        .chance_percent = 100;
    app
}
fn salvage_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<&Pickup, Without<ComponentCache>>()
        .iter(app.world())
        .count()
}
#[test]
fn actual_bullets_and_rocket_multikills_drop_once_per_victim() {
    for rocket in [false, true] {
        let mut app = economy_app();
        let p = START + Vec3::X * 250.;
        enemy(&mut app, p, 10);
        if rocket {
            enemy(&mut app, p + Vec3::Z * 30., 10);
        }
        for _ in 0..2 {
            let s = shot(&mut app, p, Vec3::ZERO, 1.);
            if rocket {
                app.world_mut().entity_mut(s).insert(rockets::Rocket);
            }
        }
        tick(&mut app, 0., &[]);
        let expected = if rocket { 2 } else { 1 };
        assert_eq!(app.world().resource::<Encounter>().kills, expected);
        assert_eq!(salvage_count(&mut app), expected as usize);
        for _ in 0..3 {
            tick(&mut app, 0., &[]);
        }
        assert_eq!(salvage_count(&mut app), expected as usize);
        assert_eq!(
            app.world().resource::<AttemptResources>().collected,
            Amounts::default()
        );
    }
}
#[test]
fn hazard_kill_supplies_drop_and_no_roll_repeats_in_later_frames() {
    let mut app = economy_app();
    app.world_mut()
        .init_resource::<crate::world::WorldGeometry>();
    enemy(&mut app, Vec3::new(120., 90., 0.), 10);
    let mut hazard = app
        .world_mut()
        .resource_mut::<crate::world::hazard::HazardState>();
    hazard.phase = crate::world::hazard::HazardPhase::Active;
    hazard.elapsed = 0.;
    tick(&mut app, 0.01, &[]);
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
    assert_eq!(salvage_count(&mut app), 1);
    tick(&mut app, 0.01, &[]);
    assert_eq!(salvage_count(&mut app), 1);
}
#[test]
fn terminal_or_restart_frame_cannot_collect_nearby_loot_or_bank_new_deaths() {
    for (fatal, restart) in [(false, false), (true, false), (true, true)] {
        let mut app = economy_app();
        app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
        app.world_mut().resource_mut::<AttemptResources>().collected = Amounts {
            salvage: 19,
            components: 3,
        };
        let p = app
            .world_mut()
            .spawn((
                Pickup {
                    amount: Amounts {
                        salvage: 100,
                        components: 100,
                    },
                    radius: 100.,
                    attracted: true,
                },
                Transform::from_translation(START),
            ))
            .id();
        if fatal {
            app.world_mut().resource_mut::<PlayerHealth>().current = 0;
        }
        tick(&mut app, 0., if restart { &[KeyCode::KeyR] } else { &[] });
        assert!(app.world().get_entity(p).is_err());
        if restart {
            assert_eq!(
                app.world().resource::<Campaign>().wallet,
                Amounts::default()
            );
            assert!(app.world().resource::<MissionSession>().result.is_none());
            assert_eq!(count::<ComponentCache>(&mut app), 3);
        } else {
            let result = app
                .world()
                .resource::<MissionSession>()
                .result
                .as_ref()
                .unwrap();
            assert_eq!(
                result.rewards.collected,
                Amounts {
                    salvage: 19,
                    components: 3
                }
            );
            assert_eq!(
                result.rewards.credited,
                if fatal {
                    Amounts {
                        salvage: 4,
                        components: 0,
                    }
                } else {
                    Amounts {
                        salvage: 29,
                        components: 4,
                    }
                }
            );
            assert_eq!(count::<Pickup>(&mut app), 0);
        }
    }
}
