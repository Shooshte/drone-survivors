use super::*;
use crate::energy::{
    Energy,
    tests::{app, at, step},
};

#[test]
fn every_module_operates_in_every_slot_and_empty_keys_are_safe() {
    for kind in [
        ModuleKind::Overdrive,
        ModuleKind::Shield,
        ModuleKind::Mobility,
        ModuleKind::Rocket,
    ] {
        for (index, key) in SLOT_KEYS.into_iter().enumerate() {
            let (mut app, _) = app();
            let mut slots = [None; 4];
            slots[index] = Some(kind);
            *app.world_mut().resource_mut::<Modules>() =
                Modules::new(Loadout::new(slots).unwrap(), &ModuleConfig::default());
            step(&mut app, 1., &SLOT_KEYS);
            let modules = app.world().resource::<Modules>();
            assert!(modules.active(kind));
            assert_eq!(modules.enabled.iter().filter(|on| **on).count(), 1);
            let expected = app.world().resource::<ModuleConfig>().drain(kind);
            assert_eq!(app.world().resource::<Energy>().current, 100. - expected);
            step(&mut app, 0., &[key]);
            assert!(!app.world().resource::<Modules>().active(kind));
        }
    }
}

#[test]
fn duplicate_types_are_rejected_in_any_positions() {
    for left in 0..4 {
        for right in left + 1..4 {
            let mut slots = [None; 4];
            slots[left] = Some(ModuleKind::Shield);
            slots[right] = Some(ModuleKind::Shield);
            assert!(Loadout::new(slots).is_err());
        }
    }
}

#[test]
fn every_slot_handles_threshold_held_input_and_depletion() {
    for (index, key) in SLOT_KEYS.into_iter().enumerate() {
        let (mut app, drone) = app();
        app.world_mut().resource_mut::<Energy>().current = 9.99;
        step(&mut app, 0., &[key]);
        assert!(!app.world().resource::<Modules>().enabled[index]);
        assert!(app.world().resource::<Modules>().rejected_for[index] > 0.);
        app.world_mut().resource_mut::<Energy>().current = 10.;
        step(&mut app, 0., &[key]);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert!(app.world().resource::<Modules>().enabled[index]);
        step(&mut app, 2., &[]);
        assert_eq!(app.world().resource::<Energy>().current, 0.);
        assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
        at(&mut app, drone, Vec3::new(-280., 90., 0.));
        step(&mut app, 1., &[]);
        assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    }
}

#[test]
fn shield_only_recharges_for_paid_time_and_toggling_preserves_progress() {
    let (mut app, _) = app();
    step(&mut app, 0., &[KeyCode::Digit2]);
    assert!(
        app.world_mut()
            .resource_mut::<Modules>()
            .block(&ModuleConfig::default())
    );
    step(&mut app, 2., &[]);
    assert_eq!(app.world().resource::<Modules>().shield.remaining, 3.);
    step(&mut app, 0., &[KeyCode::Digit2]);
    step(&mut app, 10., &[]);
    assert_eq!(app.world().resource::<Modules>().shield.remaining, 3.);
    step(&mut app, 0., &[KeyCode::Digit2]);
    app.world_mut().resource_mut::<Energy>().current = 8.;
    step(&mut app, 10., &[]);
    assert_eq!(app.world().resource::<Modules>().shield.remaining, 2.);
    assert_eq!(app.world().resource::<Modules>().shield.blocks, 0);
    app.world_mut().resource_mut::<Energy>().current = 100.;
    step(&mut app, 2., &[KeyCode::Digit2]);
    assert_eq!(app.world().resource::<Modules>().shield.blocks, 1);
}

#[test]
fn configured_shield_blocks_are_consumed_before_recharge_cycle() {
    let config = ModuleConfig {
        shield_blocks: 3,
        shield_recharge: 2.,
        ..default()
    };
    let mut modules = Modules::new(Loadout::default(), &config);
    modules.enabled[1] = true;
    for _ in 0..3 {
        assert!(modules.block(&config));
    }
    assert!(!modules.block(&config));
    modules.recharge_shield(1., &config);
    assert!(!modules.block(&config));
    modules.recharge_shield(1., &config);
    assert_eq!(modules.shield.blocks, 3);
}

#[test]
fn simultaneous_toggles_are_independent_and_rates_match_frame_sizes() {
    for rate in [30, 120] {
        let (mut app, _) = app();
        step(&mut app, 0., &SLOT_KEYS);
        for _ in 0..rate {
            step(&mut app, 1. / rate as f64, &[]);
        }
        assert!((app.world().resource::<Energy>().current - 64.).abs() < 1e-5);
        let before = app.world().resource::<Modules>().enabled;
        step(&mut app, 0., &[KeyCode::Digit2]);
        assert_eq!(
            app.world().resource::<Modules>().enabled,
            [before[0], false, before[2], before[3]]
        );
    }
}
