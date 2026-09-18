use super::*;
use crate::{
    arena::{ArenaPlugin, DroneFlight},
    energy::Energy,
    modules::{Loadout, ModuleKind, Modules},
};
use std::time::Duration;

fn app(kind: ModuleKind) -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, CombatPlugin));
    app.update();
    app.world_mut()
        .resource_mut::<WaveConfig>()
        .disable_authored_waves();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    app.world_mut().resource_mut::<Modules>().loadout =
        Loadout::new([Some(kind), None, None, None]).unwrap();
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    app
}
fn step(app: &mut App, dt: f64) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(dt));
    app.update();
}
fn enemy(app: &mut App, offset: Vec3) -> Entity {
    let position = Vec3::new(0., 90., 0.) + offset;
    app.world_mut()
        .spawn((
            Enemy {
                kind: crate::economy::runtime::EnemyKind::Chaser,
                health: 20,
                previous: position,
                path: vec![],
            },
            Transform::from_translation(position),
            DroneFlight::default(),
        ))
        .id()
}
#[test]
fn support_repulsor_changes_velocity_without_damage_or_teleporting() {
    let mut app = app(ModuleKind::Repulsor);
    let id = enemy(&mut app, Vec3::X * 100.);
    let before = app.world().get::<Transform>(id).unwrap().translation;
    step(&mut app, 0.);
    assert_eq!(
        app.world().get::<DroneFlight>(id).unwrap().velocity,
        Vec3::X * 260.
    );
    assert_eq!(
        app.world().get::<Transform>(id).unwrap().translation,
        before
    );
    assert_eq!(app.world().get::<Enemy>(id).unwrap().health, 20);
    assert_eq!(app.world().resource::<Encounter>().kills, 0);
    assert!(app.world().resource::<CombatOutcomes>().0.is_empty());
}

#[test]
fn support_repair_rate_is_paid_and_frame_rate_independent() {
    for hz in [30, 60, 144] {
        let mut app = app(ModuleKind::Repair);
        app.world_mut().resource_mut::<PlayerHealth>().current = 50;
        for _ in 0..hz {
            step(&mut app, 1. / f64::from(hz));
        }
        assert_eq!(
            app.world().resource::<PlayerHealth>().current,
            56,
            "{hz} Hz"
        );
        assert!((app.world().resource::<Energy>().current - 88.).abs() < 1e-5);
    }
}
#[test]
fn support_repair_receives_paid_depletion_fraction_and_then_stops() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<PlayerHealth>().current = 50;
    app.world_mut().resource_mut::<Energy>().current = 6.;
    step(&mut app, 1.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 53);
    assert_eq!(app.world().resource::<Energy>().current, 0.);
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    step(&mut app, 1.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 53);
}
#[test]
fn support_repair_caps_effective_hull_and_discards_full_hull_credit() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<CombatConfig>().player_health = 120;
    app.world_mut().resource_mut::<PlayerHealth>().current = 119;
    step(&mut app, 0.3);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 120);
    step(&mut app, 1.);
    assert!((app.world().resource::<Energy>().current - 84.4).abs() < 1e-5);
    app.world_mut().resource_mut::<PlayerHealth>().current = 100;
    step(&mut app, 0.1);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    step(&mut app, 0.1);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 101);
}
#[test]
fn support_repair_disabled_empty_locked_and_paused_have_no_effect() {
    for mode in 0..4 {
        let mut app = app(ModuleKind::Repair);
        app.world_mut().resource_mut::<PlayerHealth>().current = 50;
        match mode {
            0 => app.world_mut().resource_mut::<Modules>().enabled[0] = false,
            1 => app.world_mut().resource_mut::<Energy>().current = 0.,
            2 => {
                app.world_mut().resource_mut::<Modules>().jam(0);
            }
            _ => *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing,
        }
        step(&mut app, 1.);
        assert_eq!(
            app.world().resource::<PlayerHealth>().current,
            50,
            "mode {mode}"
        );
    }
}
#[test]
fn support_repair_cannot_revive_terminal_damage_or_commit_terminal_power() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<PlayerHealth>().current = 25;
    app.world_mut().resource_mut::<bombs::BombState>().remaining = Some(0.1);
    step(&mut app, 1.);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 0);
    assert_eq!(app.world().resource::<Energy>().current, 100.);
}
#[test]
fn support_repair_happens_after_nonlethal_damage() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<PlayerHealth>().current = 50;
    app.world_mut().resource_mut::<bombs::BombState>().remaining = Some(0.1);
    step(&mut app, 1.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 31);
}
#[test]
fn support_modules_stay_out_of_campaign_and_save_codec() {
    use crate::modules::shop::{ModuleInventory, ShopError};
    for kind in [ModuleKind::Repair, ModuleKind::Repulsor] {
        let mut inventory = ModuleInventory::default();
        let mut wallet = crate::economy::Amounts {
            salvage: 1000,
            components: 100,
        };
        let before = wallet;
        assert_eq!(
            inventory.purchase(kind, &mut wallet),
            Err(ShopError::CatalogOnly)
        );
        assert!(inventory.assign(0, Some(kind)).is_err());
        assert_eq!(wallet, before);
        assert!(!ModuleKind::ALL.contains(&kind));
        assert!(serde_json::to_string(&kind).is_err());
        assert!(serde_json::from_str::<ModuleKind>(&format!("\"{kind:?}\"")).is_err());
        assert!(ModuleInventory::from_saved(vec![kind], [Some(kind), None, None, None]).is_err());
        assert!(ModuleInventory::from_saved(vec![], [Some(kind), None, None, None]).is_err());
    }
}
#[test]
fn support_repulsor_obeys_three_dimensional_range_and_cover() {
    let mut app = app(ModuleKind::Repulsor);
    let near = enemy(&mut app, Vec3::new(100., 100., 0.));
    let far = enemy(&mut app, Vec3::new(150., 120., 0.));
    let blocked = enemy(&mut app, Vec3::new(-100., 0., 0.));
    app.insert_resource(crate::world::WorldGeometry {
        solids: vec![crate::world::Solid {
            center: Vec3::new(-60., 90., 0.),
            half: Vec3::new(1., 200., 300.),
        }],
        hazard: None,
    });
    step(&mut app, 0.);
    assert!(
        (app.world()
            .get::<DroneFlight>(near)
            .unwrap()
            .velocity
            .length()
            - 260.)
            .abs()
            < 1e-4
    );
    assert_eq!(
        app.world().get::<DroneFlight>(far).unwrap().velocity,
        Vec3::ZERO
    );
    assert_eq!(
        app.world().get::<DroneFlight>(blocked).unwrap().velocity,
        Vec3::ZERO
    );
}
#[test]
fn support_repulsor_coincident_centers_stay_finite_and_dead_enemies_are_ignored() {
    let mut app = app(ModuleKind::Repulsor);
    let same = enemy(&mut app, Vec3::ZERO);
    let dead = enemy(&mut app, Vec3::X * 100.);
    app.world_mut().get_mut::<Enemy>(dead).unwrap().health = 0;
    step(&mut app, 0.);
    let velocity = app.world().get::<DroneFlight>(same).unwrap().velocity;
    assert!(velocity.is_finite());
    assert!((velocity.length() - 260.).abs() < 1e-4);
    assert_eq!(
        app.world().get::<DroneFlight>(dead).unwrap().velocity,
        Vec3::ZERO
    );
}
#[test]
fn support_repulsor_impulse_uses_collision_aware_enemy_movement() {
    let mut app = app(ModuleKind::Repulsor);
    let id = enemy(&mut app, Vec3::X * 100.);
    let wall = crate::world::Solid {
        center: Vec3::new(145., 150., 0.),
        half: Vec3::new(1., 150., 1000.),
    };
    app.insert_resource(crate::world::WorldGeometry {
        solids: vec![wall],
        hazard: None,
    });
    step(&mut app, 0.);
    assert_eq!(app.world().get::<DroneFlight>(id).unwrap().velocity.x, 260.);
    step(&mut app, 0.3);
    let transform = app.world().get::<Transform>(id).unwrap();
    assert!(transform.translation.x > 100.);
    assert!(!wall.overlaps(
        transform.translation,
        crate::arena::world_half_extents(transform.rotation, Vec3::splat(14.))
    ));
    assert!(transform.translation.x < 145.);
}

#[test]
fn support_repulsor_reverses_incoming_velocity_and_does_not_stack_outgoing_speed() {
    for incoming in [-260., 260.] {
        let mut app = app(ModuleKind::Repulsor);
        let id = enemy(&mut app, Vec3::X * 100.);
        app.world_mut().get_mut::<DroneFlight>(id).unwrap().velocity = Vec3::new(incoming, 17., 0.);
        step(&mut app, 0.);
        let velocity = app.world().get::<DroneFlight>(id).unwrap().velocity;
        assert_eq!(velocity, Vec3::new(260., 17., 0.));
    }
}
#[test]
fn support_repair_fraction_survives_choice_pause_but_not_restart_or_return() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<PlayerHealth>().current = 50;
    step(&mut app, 0.1);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
    step(&mut app, 8.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 50);
    assert!((app.world().resource::<Energy>().current - 98.8).abs() < 1e-5);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
    step(&mut app, 0.1);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 51);
    for boundary in [false, true] {
        // Start each boundary with a known, nonzero fractional paid remainder.
        app.world_mut()
            .resource_mut::<support::RepairState>()
            .credit = 0.8;
        if boundary {
            app.insert_resource(crate::game::MissionBoundary {
                reset: false,
                cleanup: true,
            });
            *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Hub;
        } else {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
        }
        step(&mut app, 0.);
        assert_eq!(app.world().resource::<support::RepairState>().credit, 0.);
        assert_eq!(
            app.world()
                .resource::<crate::energy::PowerFrame>()
                .repair_seconds,
            0.
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .remove_resource::<crate::game::MissionBoundary>();
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
        app.world_mut().resource_mut::<Modules>().enabled[0] = true;
        app.world_mut().resource_mut::<PlayerHealth>().current = 50;
        step(&mut app, 0.1);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 50);
    }
}
#[test]
fn support_repair_new_jammer_lock_blocks_healing_and_recovery_needs_manual_toggle() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<PlayerHealth>().current = 50;
    let id = enemy(&mut app, Vec3::X * 200.);
    app.world_mut().get_mut::<Enemy>(id).unwrap().kind = crate::economy::runtime::EnemyKind::Jammer;
    let mut attack = control::ControlAttack::default();
    attack.phase = control::ControlPhase::Windup;
    attack.elapsed = 1.2;
    attack.slot = Some(0);
    app.world_mut().entity_mut(id).insert(attack);
    step(&mut app, 0.2);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 50);
    assert_eq!(app.world().resource::<Modules>().disabled_for[0], 3.);
    app.world_mut().despawn(id);
    step(&mut app, 3.1);
    assert_eq!(app.world().resource::<Modules>().disabled_for[0], 0.);
    assert!(!app.world().resource::<Modules>().enabled[0]);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 50);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Digit1);
    step(&mut app, 0.5);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 53);
}
#[test]
fn support_repair_activation_threshold_is_not_a_separate_debit_in_any_slot() {
    for (slot, key) in crate::modules::SLOT_KEYS.into_iter().enumerate() {
        let mut app = app(ModuleKind::Repair);
        let mut slots = [None; 4];
        slots[slot] = Some(ModuleKind::Repair);
        let config = app
            .world()
            .resource::<crate::modules::ModuleConfig>()
            .clone();
        *app.world_mut().resource_mut::<Modules>() =
            Modules::new(Loadout::new(slots).unwrap(), &config);
        app.world_mut().resource_mut::<PlayerHealth>().current = 50;
        app.world_mut().resource_mut::<Energy>().current = 9.;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        step(&mut app, 0.5);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 50);
        assert_eq!(app.world().resource::<Energy>().current, 9.);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().resource_mut::<Energy>().current = 10.;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        step(&mut app, 0.5);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 53);
        assert_eq!(app.world().resource::<Energy>().current, 4.);
    }
}
#[test]
fn support_repair_paid_fraction_accounts_for_other_modules_and_charger_supply() {
    let mut app = app(ModuleKind::Repair);
    app.world_mut().resource_mut::<PlayerHealth>().current = 50;
    app.world_mut().resource_mut::<Modules>().loadout = Loadout::new([
        Some(ModuleKind::Repair),
        Some(ModuleKind::Overdrive),
        None,
        None,
    ])
    .unwrap();
    app.world_mut().resource_mut::<Modules>().enabled[1] = true;
    app.world_mut().resource_mut::<Energy>().current = 11.;
    step(&mut app, 1.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 53);
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    app.world_mut().spawn(crate::energy::ChargingNode {
        center: Vec3::ZERO,
        radius: 100.,
        height: 160.,
    });
    step(&mut app, 1.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 59);
    assert_eq!(app.world().resource::<Energy>().current, 13.);
}
#[test]
fn support_repulsor_config_controls_reach_impulse_drain_and_one_pulse_per_hitch() {
    let mut app = app(ModuleKind::Repulsor);
    let mut config = app
        .world_mut()
        .resource_mut::<crate::modules::ModuleConfig>();
    config.repulsor_radius = 120.;
    config.repulsor_impulse = 150.;
    config.repulsor_drain = 5.;
    config.repulsor_interval = 0.5;
    let near = enemy(&mut app, Vec3::X * 100.);
    let far = enemy(&mut app, Vec3::X * 130.);
    step(&mut app, 0.);
    assert_eq!(
        app.world().get::<DroneFlight>(near).unwrap().velocity,
        Vec3::X * 150.
    );
    assert_eq!(
        app.world().get::<DroneFlight>(far).unwrap().velocity,
        Vec3::ZERO
    );
    assert_eq!(
        app.world().resource::<bombs::BombState>().pulse_cooldown,
        0.5
    );
    app.world_mut().despawn(near);
    app.world_mut().despawn(far);
    step(&mut app, 5.);
    assert_eq!(
        app.world().resource::<bombs::BombState>().pulse_cooldown,
        0.5
    );
    assert_eq!(app.world().resource::<Energy>().current, 75.);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
    step(&mut app, 5.);
    assert_eq!(
        app.world().resource::<bombs::BombState>().pulse_cooldown,
        0.5
    );
}
