use super::*;
use crate::energy::{Energy, EnergyConfig};
use crate::modules::{ModuleKind, Modules};

#[test]
fn overdrive_doubles_actual_shots_without_changing_projectiles() {
    for dt in [1. / 30., 1. / 120.] {
        for powered in [false, true] {
            let (mut app, _) = empty_app();
            app.world_mut()
                .resource_mut::<CombatConfig>()
                .enemy_flight
                .max_horizontal_speed = 0.;
            let target = enemy(&mut app, START + Vec3::X * 200., 10000);
            step(&mut app, 0., if powered { &[KeyCode::Digit1] } else { &[] });
            for _ in 0..(2. / dt) as usize {
                step(&mut app, dt, &[]);
            }
            let damage = 10000 - app.world().get::<Enemy>(target).unwrap().health;
            assert_eq!(damage, if powered { 70 } else { 40 });
        }
    }
}

#[test]
fn transitions_preserve_cooldown_progress_and_cannot_grant_bonus_shots() {
    let (mut app, _) = empty_app();
    enemy(&mut app, START + Vec3::X * 200., 10000);
    step(&mut app, 0., &[]);
    step(&mut app, 0.1, &[KeyCode::Digit1]);
    assert!((app.world().resource::<Weapon>().ready_at - 0.3).abs() < 1e-6);
    for _ in 0..4 {
        step(&mut app, 0., &[KeyCode::Digit1]);
    }
    assert_eq!(count::<Projectile>(&mut app), 1);
    assert!((app.world().resource::<Weapon>().ready_at - 0.3).abs() < 1e-6);
    step(&mut app, 0., &[KeyCode::Digit1]);
    assert!((app.world().resource::<Weapon>().ready_at - 0.5).abs() < 1e-6);
}

#[test]
fn zero_energy_keeps_flight_and_basic_weapon_operational() {
    let (mut app, drone) = empty_app();
    enemy(&mut app, START + Vec3::X * 200., 10000);
    {
        let mut e = app.world_mut().resource_mut::<Energy>();
        e.current = 0.1;
    }
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    step(&mut app, 0.1, &[KeyCode::KeyW, KeyCode::Space]);
    assert_ne!(position(&app, drone), START);
    assert_eq!(count::<Projectile>(&mut app), 1);
    let e = app.world().resource::<Energy>();
    assert_eq!(e.current, 0.);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    assert_eq!(app.world().resource::<Weapon>().interval, Some(0.5));
}

#[test]
fn outcome_frame_freezes_energy_and_restart_wins_over_everything() {
    for fatal in [false, true] {
        let (mut app, drone) = empty_app();
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = Vec3::new(-280., 90., 0.);
        if fatal {
            enemy(&mut app, Vec3::new(-280., 90., 0.), 10000);
            app.world_mut().resource_mut::<PlayerHealth>().current = 1;
        }
        let duration = app.world().resource::<WaveConfig>().duration;
        app.world_mut().resource_mut::<Encounter>().elapsed = duration - 0.1;
        {
            let mut e = app.world_mut().resource_mut::<Energy>();
            e.current = 40.;
        }
        app.world_mut().resource_mut::<Modules>().enabled[0] = true;
        step(&mut app, 0.2, &[KeyCode::Digit1]);
        assert_eq!(
            *app.world().resource::<GamePhase>(),
            if fatal {
                GamePhase::Dead
            } else {
                GamePhase::Survived
            }
        );
        assert_eq!(app.world().resource::<Energy>().current, 40.);
        assert!(
            app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        step(
            &mut app,
            1.,
            &[KeyCode::KeyR, KeyCode::Digit1, KeyCode::Space],
        );
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        assert_eq!(position(&app, drone), START);
        assert_eq!(
            app.world().resource::<Energy>().current,
            app.world().resource::<EnergyConfig>().capacity
        );
        assert!(
            !app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        assert_eq!(count::<Projectile>(&mut app), 0);
    }
}

#[test]
fn empty_drone_reaches_both_nodes_with_real_flight_under_enemy_pressure() {
    for key in [KeyCode::KeyQ, KeyCode::KeyE] {
        let (mut app, _) = empty_app();
        enemy(&mut app, START + Vec3::Z * 220., 10000);
        app.world_mut().resource_mut::<Energy>().current = 0.;
        let mut reached = false;
        for _ in 0..240 {
            step(&mut app, 1. / 120., &[key]);
            let energy = app.world().resource::<Energy>();
            if energy.charging.is_some() && energy.current > 0. {
                reached = true;
                break;
            }
        }
        assert!(reached, "failed to reach node with {key:?}");
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        assert!(
            !app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
    }
}

#[test]
fn movement_determines_charge_before_activation_and_same_frame_recharge() {
    use crate::arena::DroneFlight;
    let (mut app, drone) = empty_app();
    app.world_mut()
        .get_mut::<Transform>(drone)
        .unwrap()
        .translation = Vec3::new(-189., 90., 0.);
    app.world_mut()
        .get_mut::<DroneFlight>(drone)
        .unwrap()
        .velocity = Vec3::NEG_X * 100.;
    app.world_mut().resource_mut::<Energy>().current = 9.999;
    step(&mut app, 0.02, &[KeyCode::Digit1]);
    let energy = app.world().resource::<Energy>();
    assert!(energy.charging.is_some());
    assert!(energy.current > 10.);
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive)
    );
    assert!(app.world().resource::<Modules>().rejected_for[0] > 0.);
    let before = energy.current;
    app.world_mut()
        .get_mut::<DroneFlight>(drone)
        .unwrap()
        .velocity = Vec3::X * 100.;
    step(&mut app, 0.04, &[]);
    let energy = app.world().resource::<Energy>();
    assert!(energy.charging.is_none());
    assert_eq!(energy.current, before);
}
