//! Explicit comparison fixture: same ordinary rules and controls on each git revision.
use super::*;
use crate::arena::DroneFlight;
use crate::{
    energy::ChargingNode,
    modules::{ModuleKind, Modules},
    upgrades::{UpgradeRun, runtime::UpgradePlugin},
    world::{PlayerPath, WorldGeometry},
};

#[test]
#[ignore = "explicit four-run arena pressure measurement"]
fn arena_pressure_probe() {
    for tactic in ["left-camp", "right-camp", "moving", "relay"] {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<WorldGeometry>()
            .init_resource::<PlayerPath>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::ZERO,
            ))
            .add_plugins((
                bevy::time::TimePlugin,
                ArenaPlugin,
                CombatPlugin,
                UpgradePlugin,
            ));
        app.update();
        let drone = app
            .world_mut()
            .query_filtered::<Entity, With<Drone>>()
            .single(app.world())
            .unwrap();
        if tactic.ends_with("camp") {
            let node = app
                .world_mut()
                .query::<&ChargingNode>()
                .iter(app.world())
                .find(|node| (node.center.x < 0.) == (tactic == "left-camp"))
                .unwrap()
                .center;
            app.world_mut()
                .get_mut::<Transform>(drone)
                .unwrap()
                .translation = node + Vec3::Y * 90.;
        }
        let mut chargers: Vec<_> = app
            .world_mut()
            .query::<&ChargingNode>()
            .iter(app.world())
            .map(|n| n.center.with_y(150.))
            .collect();
        chargers.sort_by(|a, b| a.x.total_cmp(&b.x));
        let detour_z = (app
            .world()
            .resource::<WorldGeometry>()
            .solids
            .iter()
            .filter(|s| s.half.y >= 100.)
            .map(|s| s.center.z + s.half.z)
            .max_by(f32::total_cmp)
            .unwrap_or(0.)
            + 110.)
            .min(app.world().resource::<crate::arena::Arena>().half_size.z - 75.);
        let relay = [
            chargers[0],
            Vec3::new(chargers[0].x, 150., detour_z),
            Vec3::new(chargers[1].x, 150., detour_z),
            chargers[1],
            Vec3::new(chargers[1].x, 150., detour_z),
            Vec3::new(chargers[0].x, 150., detour_z),
        ];
        let mut waypoint = 0;
        let mut peak = 0;
        let mut population_time = 0.;
        let mut in_range_time = 0.;
        let mut no_enemies_time = 0.;
        let mut distance_sum = 0.;
        let mut distance_samples = 0;
        let mut choice_armed = false;
        let mut choices = vec![];
        let mut damages = vec![];
        for _ in 0..30 * 310 + 256 {
            let phase = *app.world().resource::<GamePhase>();
            if matches!(phase, GamePhase::Dead | GamePhase::Survived) {
                break;
            }
            let elapsed = app.world().resource::<Encounter>().elapsed;
            let mut keys = vec![];
            if phase == GamePhase::Choosing {
                if choice_armed {
                    keys.push(KeyCode::Backspace);
                    choice_armed = false;
                } else {
                    if !app.world().resource::<UpgradeRun>().offer.is_empty() {
                        choices.push(elapsed);
                    }
                    choice_armed = true;
                }
            } else {
                choice_armed = false;
                if !app
                    .world()
                    .resource::<Modules>()
                    .active(ModuleKind::Overdrive)
                {
                    keys.push(KeyCode::Digit1);
                }
                let threats: Vec<_> = app
                    .world_mut()
                    .query_filtered::<(&Transform, &DroneFlight), With<Enemy>>()
                    .iter(app.world())
                    .map(|(t, f)| (t.translation, f.velocity))
                    .collect();
                let p = position(&app, drone);
                if tactic == "moving" {
                    keys.extend(super::super::validation::pilot_keys(
                        elapsed,
                        p,
                        app.world().get::<DroneFlight>(drone).unwrap(),
                        &threats,
                    ));
                }
                if tactic == "relay" {
                    let flight = app.world().get::<DroneFlight>(drone).unwrap();
                    if p.distance(relay[waypoint]) < 25. && flight.velocity.length() < 45. {
                        waypoint = (waypoint + 1) % relay.len();
                    }
                    keys.extend(super::super::validation::routes::keys(
                        p,
                        flight,
                        relay[waypoint],
                    ));
                }
                peak = peak.max(threats.len());
                population_time += threats.len() as f64 / 30.;
                if let Some(distance) = threats
                    .iter()
                    .map(|(t, _)| t.distance(p))
                    .min_by(f32::total_cmp)
                {
                    distance_sum += f64::from(distance);
                    distance_samples += 1;
                    if distance <= 400. {
                        in_range_time += 1. / 30.;
                    }
                } else {
                    no_enemies_time += 1. / 30.;
                }
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            for key in keys {
                app.world_mut()
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(key);
            }
            *app.world_mut()
                .resource_mut::<bevy::time::TimeUpdateStrategy>() =
                bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1. / 30.));
            app.update();
            if app
                .world()
                .resource::<CombatOutcomes>()
                .0
                .iter()
                .any(|event| matches!(event, CombatOutcome::PlayerDamaged))
            {
                damages.push((
                    app.world().resource::<Encounter>().elapsed,
                    position(&app, drone),
                ));
            }
        }
        let phase = *app.world().resource::<GamePhase>();
        assert!(matches!(phase, GamePhase::Dead | GamePhase::Survived));
        let run = app.world().resource::<Encounter>();
        let spawns = run.spawns;
        assert_eq!(
            spawns.requested,
            spawns.admitted
                + spawns.rejected_cap
                + spawns.rejected_space
                + spawns.skipped_hitch
                + spawns.skipped_terminal
        );
        assert!(peak <= 30);
        assert!(
            app.world().resource::<UpgradeRun>().selected.is_empty(),
            "all naturally earned offers are skipped"
        );
        println!(
            "ARENA PRESSURE size={:?} tactic={tactic} phase={phase:?} seconds={:.3} hull={} kills={} peak={peak} average_population={:.2} nearest_mean={:.1} in_weapon_range_seconds={in_range_time:.2} empty_seconds={no_enemies_time:.2} choices={choices:?} damage_positions={damages:?} spawns={spawns:?}",
            app.world().resource::<crate::arena::Arena>().half_size * 2.,
            run.elapsed,
            app.world().resource::<PlayerHealth>().current,
            run.kills,
            population_time / run.elapsed,
            distance_sum / distance_samples.max(1) as f64
        );
    }
}
