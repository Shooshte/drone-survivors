//! Opt-in agent campaign runs: fixed 30 Hz simulation, ordinary gameplay and inputs.
use super::*;
use crate::{
    arena::{Drone, DroneFlight},
    mission::{Campaign, MissionSession, campaign::MissionId, tests::launch},
    modules::{ModuleKind, Modules},
    save::SavePlugin,
    upgrades::{UpgradeKind, UpgradeRun},
    world::{PlayerPath, WorldGeometry},
};

fn app(path: &std::path::Path) -> App {
    let mut app = crate::mission::tests::app();
    app.init_resource::<WorldGeometry>()
        .init_resource::<PlayerPath>()
        .add_plugins(SavePlugin::at(path.to_path_buf()));
    advance(&mut app, 0., &[]);
    app
}
/// Preserve held keys and real press/release edges across simulated frames.
fn advance(app: &mut App, dt: f64, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.clear();
    let held: Vec<_> = input.get_pressed().copied().collect();
    for key in held {
        if !keys.contains(&key) {
            input.release(key);
        }
    }
    for &key in keys {
        input.press(key);
    }
    *app.world_mut()
        .resource_mut::<bevy::time::TimeUpdateStrategy>() =
        bevy::time::TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f64(dt));
    app.update();
}

fn key(app: &mut App, key: KeyCode) {
    advance(app, 0., &[]);
    advance(app, 0., &[key]);
}

// Faster agent steering for the long arena; the older navigation fixture caps at
// 180u/s while production chasers can reach 260u/s. These are ordinary flight keys.
fn cruise(position: Vec3, flight: &DroneFlight, target: Vec3) -> Vec<KeyCode> {
    let delta = target - position;
    let horizontal = delta.with_y(0.);
    let distance = horizontal.length();
    let desired = horizontal.normalize_or_zero()
        * (distance * 1.5).min((200. * distance).sqrt()).min(360.)
        + Vec3::Y * (delta.y * 2.).clamp(-40., 40.);
    let acceleration =
        (desired - flight.velocity) * 3. + flight.velocity * Vec3::new(0.25, 0.5, 0.25);
    let local = Quat::from_rotation_y(-flight.heading) * acceleration;
    let mut keys = Vec::new();
    if local.x.abs() > 15. {
        keys.push(if local.x > 0. {
            KeyCode::KeyE
        } else {
            KeyCode::KeyQ
        });
    }
    if local.z.abs() > 15. {
        keys.push(if local.z > 0. {
            KeyCode::KeyS
        } else {
            KeyCode::KeyW
        });
    }
    if acceleration.y > 10. {
        keys.push(KeyCode::Space);
    } else if acceleration.y < -10. {
        keys.push(KeyCode::ShiftLeft);
    }
    keys
}

fn play(app: &mut App, recon: bool) {
    let route = [
        Vec3::new(-560., 90., -900.),
        Vec3::new(-560., 90., 410.),
        Vec3::new(560., 90., 410.),
        Vec3::new(560., 90., -900.),
        Vec3::new(560., 90., 900.),
        Vec3::new(-560., 90., 900.),
    ];
    let mut waypoint = 0;
    let mut choice_release = true;
    let mut enabled_seconds = 0.;
    let mut sampled_charge_decreases = 0;
    for _ in 0..30 * 420 {
        let phase = *app.world().resource::<GamePhase>();
        if matches!(phase, GamePhase::Dead | GamePhase::Survived) {
            break;
        }
        let mut keys = vec![];
        if phase == GamePhase::Choosing {
            if choice_release {
                choice_release = false;
            } else {
                let run = app.world().resource::<UpgradeRun>();
                let index = [
                    UpgradeKind::HeavyRounds,
                    UpgradeKind::HeavyArmor,
                    UpgradeKind::Interceptor,
                ]
                .iter()
                .find_map(|k| run.offer.iter().position(|offered| offered == k));
                keys.push(
                    index
                        .map(|i| [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3][i])
                        .unwrap_or(KeyCode::Backspace),
                );
                choice_release = true;
            }
        } else if phase == GamePhase::Playing {
            choice_release = true;
            let (position, flight) = app
                .world_mut()
                .query_filtered::<(&Transform, &DroneFlight), With<Drone>>()
                .single(app.world())
                .map(|(t, f)| (t.translation, *f))
                .unwrap();
            if recon {
                if position.distance(route[waypoint]) < 45. && waypoint + 1 < route.len() {
                    waypoint += 1;
                }
                keys.extend(cruise(position, &flight, route[waypoint]));
                // Conserve power between contacts. Overdrive uses weapon range;
                // shield is reserved for close threats instead of draining in transit.
                let nearest = app
                    .world_mut()
                    .query_filtered::<&Transform, With<Enemy>>()
                    .iter(app.world())
                    .map(|enemy| position.distance(enemy.translation))
                    .min_by(f32::total_cmp)
                    .unwrap_or(f32::INFINITY);
                let modules = app.world().resource::<Modules>();
                let should_enable = match modules.loadout.slots()[0] {
                    Some(ModuleKind::Overdrive) => nearest <= 400.,
                    Some(ModuleKind::Shield) => nearest <= 180.,
                    _ => false,
                };
                if modules.enabled[0] != should_enable {
                    keys.push(KeyCode::Digit1);
                }
                if modules.enabled[0] {
                    enabled_seconds += 1. / 30.;
                }
            } else {
                // Continuous southern circuit avoids stopping in approaching swarms.
                let elapsed = app.world().resource::<Encounter>().elapsed;
                let theta = elapsed as f32 * 0.5;
                let target = Vec3::new(650. * theta.cos(), 60., 1000. + 440. * theta.sin());
                keys.extend(cruise(position, &flight, target));
                if ((elapsed * 30.).round() as usize).is_multiple_of(900) {
                    println!(
                        "SLICE trace t={elapsed:.1} position={position:?} speed={:.1} hull={} kills={}",
                        flight.velocity.length(),
                        app.world().resource::<PlayerHealth>().current,
                        app.world().resource::<Encounter>().kills
                    );
                }
            }
        } else {
            panic!("unexpected phase {phase:?}");
        }
        let before_blocks = app.world().resource::<Modules>().shield.blocks;
        advance(app, 1. / 30., &keys);
        let after_blocks = app.world().resource::<Modules>().shield.blocks;
        sampled_charge_decreases += before_blocks.saturating_sub(after_blocks);
    }
    let result = app.world().resource::<MissionSession>().result.as_ref();
    println!(
        "SLICE recon={recon} waypoint={waypoint} sampled_enabled_seconds={enabled_seconds:.2} sampled_charge_decreases={sampled_charge_decreases} hull={} upgrades={:?} result={result:?}",
        app.world().resource::<PlayerHealth>().current,
        app.world().resource::<UpgradeRun>().selected
    );
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
    assert!(app.world().resource::<Encounter>().spawns.requested > 0);
    if recon {
        assert!(
            app.world()
                .resource::<crate::mission::objectives::ObjectiveRun>()
                .ready()
        );
    }
}

#[test]
#[ignore = "explicit two-loadout disk-backed campaign playthroughs"]
fn vertical_slice_probe() {
    // Empty loadout is a control, followed by two complete purchase/equip loops.
    for module in [None, Some(ModuleKind::Overdrive), Some(ModuleKind::Shield)] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("campaign.json");
        let mut app = self::app(&path);
        key(&mut app, KeyCode::KeyN);
        launch(&mut app);
        play(&mut app, false);
        let reward_bank = app.world().resource::<Campaign>().wallet;
        // Reload immediately after settlement: no second payout and completion survives.
        drop(app);
        let mut app = self::app(&path);
        key(&mut app, KeyCode::Enter);
        assert_eq!(app.world().resource::<Campaign>().wallet, reward_bank);
        assert_eq!(app.world().resource::<Campaign>().history.len(), 1);
        assert!(
            app.world()
                .resource::<Campaign>()
                .progress
                .completed(MissionId::ALL[0])
        );
        if let Some(module) = module {
            key(&mut app, KeyCode::KeyM);
            if module == ModuleKind::Shield {
                key(&mut app, KeyCode::ArrowDown);
            }
            key(&mut app, KeyCode::KeyB);
            key(&mut app, KeyCode::Digit1);
            assert!(app.world().resource::<Campaign>().inventory.owns(module));
            let cost = crate::modules::shop::price(module);
            let bank = app.world().resource::<Campaign>().wallet;
            assert_eq!(bank.salvage, reward_bank.salvage - cost.salvage);
            assert_eq!(bank.components, reward_bank.components - cost.components);
            key(&mut app, KeyCode::Backspace);
        }
        key(&mut app, KeyCode::KeyC);
        key(&mut app, KeyCode::ArrowRight);
        let bank = app.world().resource::<Campaign>().wallet;
        assert_eq!(
            app.world().resource::<MissionSession>().selected_mission,
            MissionId::ALL[1]
        );
        drop(app);
        let mut app = self::app(&path);
        key(&mut app, KeyCode::Enter);
        assert_eq!(app.world().resource::<Campaign>().wallet, bank);
        assert_eq!(
            app.world()
                .resource::<Campaign>()
                .inventory
                .loadout()
                .slots()[0],
            module
        );
        println!("SLICE resumed module={module:?} bank={bank:?}");
        launch(&mut app);
        assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
        assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
        assert_eq!(app.world().resource::<Modules>().loadout.slots()[0], module);
        play(&mut app, true);
        let campaign = app.world().resource::<Campaign>();
        assert_eq!(campaign.history.len(), 2);
        assert!(campaign.progress.completed(MissionId::ALL[1]));
        let wallet = campaign.wallet;
        drop(app);
        let mut app = self::app(&path);
        key(&mut app, KeyCode::Enter);
        assert_eq!(app.world().resource::<Campaign>().wallet, wallet);
        assert_eq!(app.world().resource::<Campaign>().history.len(), 2);
    }
}
