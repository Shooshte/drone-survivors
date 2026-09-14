use crate::{
    combat::CombatConfig,
    energy::{Energy, EnergyConfig},
    game::GamePhase,
    mission::{
        Campaign,
        tests::{app, launch, tick},
    },
    passives::NodeId,
    upgrades::{UpgradeKind, UpgradeRun},
};

fn buy(app: &mut bevy::prelude::App, node: NodeId) {
    let mut campaign = app.world_mut().resource_mut::<Campaign>();
    campaign.wallet = crate::economy::Amounts {
        salvage: 1000,
        components: 1000,
    };
    let Campaign {
        passives, wallet, ..
    } = &mut *campaign;
    passives.purchase(node, wallet).unwrap();
}

#[test]
fn permanent_baseline_survives_choices_restarts_and_completed_missions() {
    use bevy::prelude::*;
    let mut app = app();
    buy(&mut app, NodeId::Damage);
    buy(&mut app, NodeId::Hull);
    buy(&mut app, NodeId::Battery);
    // Shopping affects next launch, not an old attempt's current tuning.
    assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 10);
    launch(&mut app);
    assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 11);
    assert_eq!(app.world().resource::<CombatConfig>().player_health, 120);
    assert_eq!(app.world().resource::<Energy>().current, 120.);
    for (kind, xp) in [
        (UpgradeKind::HeavyRounds, 50),
        (UpgradeKind::Interceptor, 150),
    ] {
        app.world_mut().resource_mut::<UpgradeRun>().award(xp);
        tick(&mut app, 0., &[]);
        app.world_mut().resource_mut::<UpgradeRun>().offer = vec![kind];
        tick(&mut app, 0., &[]);
        tick(&mut app, 0., &[KeyCode::Digit1]);
        tick(&mut app, 0., &[]);
    }
    assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 22);
    assert_eq!(app.world().resource::<EnergyConfig>().capacity, 90.);
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 11);
    assert_eq!(app.world().resource::<EnergyConfig>().capacity, 120.);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
    for phase in [GamePhase::Survived, GamePhase::Dead, GamePhase::Survived] {
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        tick(&mut app, 0., &[]);
        tick(&mut app, 0., &[]);
        tick(&mut app, 0., &[KeyCode::Enter]);
        launch(&mut app);
        assert_eq!(app.world().resource::<CombatConfig>().shot_damage, 11);
        assert_eq!(app.world().resource::<Energy>().current, 120.);
        assert_eq!(
            app.world()
                .resource::<Campaign>()
                .passives
                .rank(NodeId::Battery),
            1
        );
    }
}

#[test]
fn launch_applies_every_permanent_effect_from_pristine_values() {
    let mut app = app();
    for node in NodeId::ALL {
        for _ in 0..5 {
            buy(&mut app, node);
        }
    }
    launch(&mut app);
    let combat = app.world().resource::<CombatConfig>();
    assert_eq!(combat.shot_damage, 15);
    assert_eq!(combat.player_health, 200);
    assert_eq!(combat.contact_damage, 5);
    assert!((combat.fire_interval - 1. / 3.).abs() < 1e-9);
    assert_eq!(combat.target_range, 600.);
    assert_eq!(combat.projectile_lifetime, 1.5);
    assert_eq!(combat.invulnerability, 1.125);
    let energy = app.world().resource::<EnergyConfig>();
    assert_eq!(energy.capacity, 200.);
    assert_eq!(energy.recharge, 50.);
    assert_eq!(energy.reserve_cost, 0.75);
}
