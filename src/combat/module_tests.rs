use super::*;

#[test]
fn shield_activation_blocks_contact_and_shares_protection_window() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    for _ in 0..3 {
        enemy(&mut app, START, 100);
    }
    step(&mut app, 0., &[KeyCode::Digit2]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(
        app.world().resource::<PlayerHealth>().invulnerable_until,
        0.75
    );
    step(&mut app, 0.5, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    step(&mut app, 0.3, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
}

#[test]
fn mobility_increases_horizontal_response_without_changing_vertical_motion() {
    let mut velocities = Vec::new();
    for powered in [false, true] {
        let (mut app, drone) = empty_app();
        step(&mut app, 0., if powered { &[KeyCode::Digit3] } else { &[] });
        step(&mut app, 0.25, &[KeyCode::KeyW, KeyCode::Space]);
        velocities.push(
            app.world()
                .get::<crate::arena::DroneFlight>(drone)
                .unwrap()
                .velocity,
        );
    }
    assert!((velocities[1].z / velocities[0].z - 1.25).abs() < 0.001);
    assert!((velocities[1].y - velocities[0].y).abs() < 0.001);
}

#[test]
fn depleted_shield_cannot_block_contact_in_the_same_update() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    step(&mut app, 0., &[KeyCode::Digit2]);
    app.world_mut()
        .resource_mut::<crate::energy::Energy>()
        .current = 0.1;
    enemy(&mut app, START, 100);
    step(&mut app, 0.1, &[]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    assert_eq!(
        app.world().resource::<crate::modules::Modules>().enabled,
        [false; 4]
    );
}

#[test]
fn turning_shield_off_before_contact_prevents_a_block() {
    let (mut app, _) = empty_app();
    quiet(&mut app);
    step(&mut app, 0., &[KeyCode::Digit2]);
    enemy(&mut app, START, 100);
    step(&mut app, 0., &[KeyCode::Digit2]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    assert_eq!(
        app.world()
            .resource::<crate::modules::Modules>()
            .shield
            .blocks,
        1
    );
}

#[test]
fn mobility_restores_cap_on_next_movement_integration() {
    let (mut app, drone) = empty_app();
    step(&mut app, 0., &[KeyCode::Digit3]);
    app.world_mut()
        .get_mut::<crate::arena::DroneFlight>(drone)
        .unwrap()
        .velocity = Vec3::X * 525.;
    step(&mut app, 0., &[]);
    assert_eq!(
        app.world()
            .get::<crate::arena::DroneFlight>(drone)
            .unwrap()
            .velocity
            .x,
        525.
    );
    step(&mut app, 0., &[KeyCode::Digit3]);
    step(&mut app, 0., &[]);
    assert_eq!(
        app.world()
            .get::<crate::arena::DroneFlight>(drone)
            .unwrap()
            .velocity
            .x,
        420.
    );
}

#[test]
fn every_module_resets_and_freezes_with_the_encounter() {
    use crate::modules::{Modules, SLOT_KEYS};
    let (mut app, _) = empty_app();
    step(&mut app, 0., &SLOT_KEYS);
    app.world_mut().resource_mut::<Modules>().shield.blocks = 0;
    app.world_mut().resource_mut::<Modules>().shield.remaining = 3.;
    for phase in [GamePhase::Dead, GamePhase::Survived] {
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        step(&mut app, 1., &SLOT_KEYS);
        assert_eq!(app.world().resource::<Modules>().enabled, [true; 4]);
        assert_eq!(app.world().resource::<Modules>().shield.remaining, 3.);
    }
    step(
        &mut app,
        1.,
        &[
            KeyCode::KeyR,
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
        ],
    );
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    assert_eq!(app.world().resource::<Modules>().shield.blocks, 1);
    assert_eq!(app.world().resource::<Modules>().shield.remaining, 0.);
}

#[test]
fn module_effects_follow_each_of_the_four_equipped_positions() {
    use crate::modules::{Loadout, ModuleConfig, ModuleKind, Modules, SLOT_KEYS};
    for (slot, key) in SLOT_KEYS.into_iter().enumerate() {
        for kind in [
            ModuleKind::Overdrive,
            ModuleKind::Shield,
            ModuleKind::Mobility,
            ModuleKind::Rocket,
        ] {
            let (mut app, drone) = empty_app();
            let mut slots = [None; 4];
            slots[slot] = Some(kind);
            *app.world_mut().resource_mut::<Modules>() =
                Modules::new(Loadout::new(slots).unwrap(), &ModuleConfig::default());
            app.world_mut()
                .resource_mut::<CombatConfig>()
                .enemy_flight
                .max_horizontal_speed = 0.;
            enemy(
                &mut app,
                START
                    + if kind == ModuleKind::Shield {
                        Vec3::ZERO
                    } else {
                        Vec3::X * 150.
                    },
                1000,
            );
            step(&mut app, 0., &[key]);
            match kind {
                ModuleKind::Overdrive => {
                    assert_eq!(app.world().resource::<Weapon>().interval, Some(0.25))
                }
                ModuleKind::Shield => {
                    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
                    assert_eq!(app.world().resource::<Modules>().shield.blocks, 0);
                }
                ModuleKind::Rocket => assert_eq!(count::<rockets::Rocket>(&mut app), 1),
                ModuleKind::Mobility => {
                    app.world_mut()
                        .get_mut::<crate::arena::DroneFlight>(drone)
                        .unwrap()
                        .velocity = Vec3::X * 525.;
                    step(&mut app, 0., &[]);
                    assert_eq!(
                        app.world()
                            .get::<crate::arena::DroneFlight>(drone)
                            .unwrap()
                            .velocity
                            .x,
                        525.
                    );
                }
            }
        }
    }
}

#[test]
fn compare_maximum_output_and_conservation_in_the_same_authored_encounter() {
    use crate::{
        arena::DroneFlight,
        energy::Energy,
        modules::{Modules, SLOT_KEYS},
    };
    for maximum in [true, false] {
        let (mut app, drone) = app();
        let mut activations = 0;
        let mut powered_seconds = 0.;
        let mut charging_seconds = 0.;
        for _ in 0..(310 * 60) {
            let elapsed = app.world().resource::<Encounter>().elapsed;
            let threats: Vec<_> = app
                .world_mut()
                .query_filtered::<(&Transform, &DroneFlight), With<Enemy>>()
                .iter(app.world())
                .map(|(t, f)| (t.translation, f.velocity))
                .collect();
            let mut keys = super::super::validation::pilot_keys(
                elapsed,
                position(&app, drone),
                app.world().get::<DroneFlight>(drone).unwrap(),
                &threats,
            );
            let modules = app.world().resource::<Modules>();
            let energy = app.world().resource::<Energy>();
            if elapsed >= 3. && energy.current >= if maximum { 50. } else { 20. } {
                for (slot, key) in SLOT_KEYS.into_iter().enumerate() {
                    if (maximum || slot == 3) && !modules.enabled[slot] {
                        keys.push(key);
                        activations += 1;
                    }
                }
            }
            if modules.enabled.iter().any(|on| *on) {
                powered_seconds += 1. / 60.;
            }
            if energy.charging.is_some() {
                charging_seconds += 1. / 60.;
            }
            step(&mut app, 1. / 60., &keys);
            assert!((0. ..=100.).contains(&app.world().resource::<Energy>().current));
            if *app.world().resource::<GamePhase>() != GamePhase::Playing {
                break;
            }
        }
        println!(
            "MODULE COMPARISON mode={} phase={:?} seconds={:.2} hull={} kills={} energy={:.2} activations={} powered_seconds={:.2} charging_seconds={:.2}",
            if maximum { "maximum" } else { "conservation" },
            app.world().resource::<GamePhase>(),
            app.world().resource::<Encounter>().elapsed,
            app.world().resource::<PlayerHealth>().current,
            app.world().resource::<Encounter>().kills,
            app.world().resource::<Energy>().current,
            activations,
            powered_seconds,
            charging_seconds
        );
        assert!(matches!(
            *app.world().resource::<GamePhase>(),
            GamePhase::Survived | GamePhase::Dead
        ));
        assert!(activations > 0);
        assert!(app.world().resource::<Encounter>().kills > 0);
    }
}
