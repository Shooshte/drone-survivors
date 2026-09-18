use super::*;
use crate::{
    arena::{ArenaPlugin, DroneFlight},
    combat::{CombatPlugin, Encounter, Enemy, WaveConfig},
    energy::Energy,
    modules::{Loadout, Modules},
};
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_plugins((ArenaPlugin, CombatPlugin));
    app.update();
    app.world_mut()
        .resource_mut::<WaveConfig>()
        .disable_authored_waves();
    app.world_mut().resource_mut::<CombatConfig>().target_range = 0.;
    app
}
fn step(app: &mut App, dt: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(dt));
    app.update();
}
fn run(app: &mut App, seconds: f32, hz: u32) {
    for _ in 0..(seconds * hz as f32).round() as u32 {
        step(app, 1. / hz as f32);
    }
}
fn carrier(app: &mut App) -> Entity {
    let position = Vec3::new(0., 90., 0.);
    app.world_mut()
        .spawn((
            Enemy {
                kind: crate::economy::runtime::EnemyKind::Bomber,
                health: 20,
                previous: position,
                path: vec![],
            },
            Transform::from_translation(position),
            DroneFlight::default(),
        ))
        .id()
}
fn repulsor(app: &mut App) {
    app.world_mut().resource_mut::<Modules>().loadout =
        Loadout::new([Some(ModuleKind::Repulsor), None, None, None]).unwrap();
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
}
#[test]
fn bomb_contact_consumes_carrier_without_damage_or_reward_then_detonates_once() {
    for hz in [30, 60, 144] {
        let mut app = app();
        let id = carrier(&mut app);
        step(&mut app, 1. / hz as f32);
        assert!(app.world().get_entity(id).is_err());
        assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
        assert_eq!(app.world().resource::<BombState>().remaining, Some(3.));
        assert_eq!(app.world().resource::<Encounter>().kills, 0);
        run(&mut app, 2.9, hz);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
        run(&mut app, 0.2, hz);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
        run(&mut app, 4., hz);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
        assert!(app.world().resource::<BombState>().remaining.is_none());
        assert_eq!(app.world().resource::<Encounter>().kills, 0);
    }
}
#[test]
fn bomb_does_not_stack_or_refresh_and_attachment_gets_full_fuse_on_hitch() {
    let mut app = app();
    carrier(&mut app);
    carrier(&mut app);
    step(&mut app, 5.);
    assert_eq!(app.world().resource::<BombState>().remaining, Some(3.));
    step(&mut app, 1.);
    carrier(&mut app);
    step(&mut app, 0.);
    assert_eq!(app.world().resource::<BombState>().remaining, Some(2.));
    step(&mut app, 2.);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
}
#[test]
fn bomb_uses_shield_and_invulnerability_without_retrying_damage() {
    for shield in [false, true] {
        let mut app = app();
        carrier(&mut app);
        step(&mut app, 0.);
        if shield {
            app.world_mut().resource_mut::<Modules>().enabled[1] = true;
        } else {
            app.world_mut()
                .resource_mut::<PlayerHealth>()
                .invulnerable_until = 10.;
        }
        step(&mut app, 3.);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
        assert!(app.world().resource::<BombState>().remaining.is_none());
        if shield {
            assert_eq!(app.world().resource::<Modules>().shield.blocks, 0);
        }
        step(&mut app, 10.);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    }
}
#[test]
fn bomb_powered_repulsor_removes_bomb_and_wins_deadline_tie() {
    let mut app = app();
    carrier(&mut app);
    step(&mut app, 0.);
    repulsor(&mut app);
    step(&mut app, 3.);
    assert!(app.world().resource::<BombState>().remaining.is_none());
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(app.world().resource::<BombState>().notice, "BOMB DISLODGED");
    assert_eq!(app.world().resource::<Energy>().current, 76.);
}
#[test]
fn bomb_repulsor_requires_power_and_unlocked_slot_and_cannot_toggle_reset_cooldown() {
    for jam in [false, true] {
        let mut app = app();
        carrier(&mut app);
        step(&mut app, 0.);
        repulsor(&mut app);
        if jam {
            app.world_mut().resource_mut::<Modules>().disabled_for[0] = 4.;
        } else {
            app.world_mut().resource_mut::<Energy>().current = 0.;
        }
        step(&mut app, 3.);
        assert_eq!(app.world().resource::<PlayerHealth>().current, 75);
    }
    let mut app = app();
    repulsor(&mut app);
    step(&mut app, 0.01);
    carrier(&mut app);
    step(&mut app, 0.01);
    assert!(app.world().resource::<BombState>().remaining.is_some());
    app.world_mut().resource_mut::<Modules>().enabled[0] = false;
    step(&mut app, 0.2);
    app.world_mut().resource_mut::<Modules>().enabled[0] = true;
    step(&mut app, 0.2);
    assert!(app.world().resource::<BombState>().remaining.is_some());
    run(&mut app, 1.7, 60);
    assert!(app.world().resource::<BombState>().remaining.is_none());
}
#[test]
fn bomb_choice_pauses_and_restart_and_terminal_clear_all_transients() {
    let mut app = app();
    carrier(&mut app);
    step(&mut app, 0.);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Choosing;
    step(&mut app, 8.);
    assert_eq!(app.world().resource::<BombState>().remaining, Some(3.));
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Playing;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    step(&mut app, 1.);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    assert!(app.world().resource::<BombState>().remaining.is_none());
    run(&mut app, 4., 60);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    carrier(&mut app);
    step(&mut app, 0.);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Survived;
    step(&mut app, 4.);
    assert!(app.world().resource::<BombState>().remaining.is_none());
}
#[test]
fn bomb_catalog_repulsor_is_rejected_by_campaign_and_save_codec() {
    use crate::modules::shop::ModuleInventory;
    let mut inventory = ModuleInventory::default();
    let mut wallet = crate::economy::Amounts {
        salvage: 1000,
        components: 100,
    };
    let before = wallet;
    assert!(
        inventory
            .purchase(ModuleKind::Repulsor, &mut wallet)
            .is_err()
    );
    assert_eq!(wallet, before);
    assert!(!ModuleKind::ALL.contains(&ModuleKind::Repulsor));
    assert!(serde_json::from_str::<ModuleKind>("\"Repulsor\"").is_err());
    assert!(
        ModuleInventory::from_saved(
            vec![ModuleKind::Repulsor],
            [Some(ModuleKind::Repulsor), None, None, None]
        )
        .is_err()
    );
}

#[test]
fn bomb_carrier_destroyed_before_contact_awards_one_typed_kill_and_never_attaches() {
    use crate::combat::{CombatOutcome, Projectile, ShotPayload};
    let mut app = app();
    let id = carrier(&mut app);
    for _ in 0..2 {
        app.world_mut().spawn((
            Projectile {
                velocity: Vec3::ZERO,
                remaining: 1.,
            },
            ShotPayload {
                damage: 20,
                radius: 3.,
            },
            Transform::from_xyz(0., 90., 0.),
        ));
    }
    step(&mut app, 0.01);
    assert!(app.world().get_entity(id).is_err());
    assert!(app.world().resource::<BombState>().remaining.is_none());
    assert_eq!(app.world().resource::<Encounter>().kills, 1);
    assert_eq!(app.world().resource::<CombatOutcomes>().0.iter().filter(|e|matches!(e,CombatOutcome::Hit{entity,killed:true,kind:crate::economy::runtime::EnemyKind::Bomber,..} if *entity==id)).count(),1);
    run(&mut app, 4., 60);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
}
#[test]
fn bomb_carrier_cannot_attach_through_cover() {
    let mut app = app();
    let id = carrier(&mut app);
    let position = Vec3::new(30., 90., 0.);
    app.world_mut()
        .get_mut::<Transform>(id)
        .unwrap()
        .translation = position;
    app.world_mut().get_mut::<Enemy>(id).unwrap().previous = position;
    app.insert_resource(crate::world::WorldGeometry {
        solids: vec![crate::world::Solid {
            center: Vec3::new(15., 90., 0.),
            half: Vec3::new(1., 200., 300.),
        }],
        hazard: None,
    });
    step(&mut app, 0.);
    assert!(app.world().resource::<BombState>().remaining.is_none());
}
#[test]
fn bomb_repulsor_power_depletion_prevents_a_free_pulse() {
    let mut app = app();
    carrier(&mut app);
    step(&mut app, 0.);
    repulsor(&mut app);
    app.world_mut().resource_mut::<Energy>().current = 0.1;
    step(&mut app, 0.1);
    assert!(app.world().resource::<BombState>().remaining.is_some());
    assert!(
        !app.world()
            .resource::<Modules>()
            .active(ModuleKind::Repulsor)
    );
}

#[test]
fn ordnance_lethal_bomb_prevents_parent_launch_in_the_same_frame() {
    use crate::combat::mothership::Mothership;
    let mut app = app();
    app.world_mut().resource_mut::<PlayerHealth>().current = 25;
    app.world_mut().resource_mut::<BombState>().remaining = Some(0.01);
    let position = Vec3::new(300., 90., 0.);
    let mut parent = Mothership::default();
    parent.ready_at = Some(0.);
    app.world_mut().spawn((
        Enemy {
            kind: crate::economy::runtime::EnemyKind::Mothership,
            health: 100,
            previous: position,
            path: vec![],
        },
        parent,
        Transform::from_translation(position),
        DroneFlight::default(),
    ));
    step(&mut app, 0.02);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert_eq!(app.world().resource::<Encounter>().spawns.requested, 0);
    assert!(app.world().resource::<BombState>().remaining.is_none());
}
#[test]
fn ordnance_lethal_projectile_cancels_parent_delivery_at_every_frame_rate() {
    use crate::combat::{
        Projectile, ShotPayload, SpawnWarning,
        mothership::{Mothership, SpawnParent},
    };
    use crate::economy::runtime::EnemyKind;
    for hz in [30, 60, 144] {
        let mut app = app();
        let position = Vec3::new(300., 90., 0.);
        let id = app
            .world_mut()
            .spawn((
                Enemy {
                    kind: EnemyKind::Mothership,
                    health: 100,
                    previous: position,
                    path: vec![],
                },
                Mothership::default(),
                Transform::from_translation(position),
                DroneFlight::default(),
            ))
            .id();
        app.world_mut().spawn((
            SpawnWarning {
                ready_at: 1. / hz as f64,
                kind: EnemyKind::Fast,
                ..default()
            },
            SpawnParent(id),
            Transform::from_xyz(410., 90., 0.),
        ));
        app.world_mut().spawn((
            Projectile {
                velocity: Vec3::ZERO,
                remaining: 1.,
            },
            ShotPayload {
                damage: 100,
                radius: 3.,
            },
            Transform::from_translation(position),
        ));
        step(&mut app, 1. / hz as f32);
        assert!(app.world().get_entity(id).is_err());
        assert_eq!(
            app.world_mut().query::<&Enemy>().iter(app.world()).count(),
            0
        );
        assert_eq!(
            app.world_mut()
                .query::<&SpawnWarning>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(app.world().resource::<Encounter>().kills, 1);
        assert_eq!(app.world().resource::<Encounter>().spawns.cancelled, 1);
    }
}
