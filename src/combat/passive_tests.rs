use super::*;
use crate::{
    economy::Amounts,
    energy::EnergyConfig,
    modules::{Loadout, Modules},
    passives::{NodeId, PassiveTree},
};
fn tree() -> PassiveTree {
    let mut tree = PassiveTree::default();
    let mut wallet = Amounts {
        salvage: 10000,
        components: 10000,
    };
    for node in NodeId::ALL {
        for _ in 0..5 {
            tree.purchase(node, &mut wallet).unwrap();
        }
    }
    tree
}
fn fixture(upgraded: bool) -> App {
    let (mut app, _) = empty_app();
    let mut combat = CombatConfig::default();
    let mut energy = EnergyConfig::default();
    if upgraded {
        tree().apply(&mut combat, &mut energy);
    }
    combat.enemy_flight.max_horizontal_speed = 0.;
    app.world_mut().resource_mut::<PlayerHealth>().current = combat.player_health;
    app.insert_resource(combat).insert_resource(energy);
    app.world_mut().resource_mut::<Modules>().loadout = Loadout::new([None; 4]).unwrap();
    app
}
#[test]
fn passive_fixed_encounter_compares_base_and_upgraded_without_optional_modules() {
    let mut results = Vec::new();
    for upgraded in [false, true] {
        let mut app = fixture(upgraded);
        for offset in [100., 150., 200., 250., 300., 350.] {
            enemy(&mut app, START + Vec3::Z * offset, 100);
        }
        for _ in 0..720 {
            step(&mut app, 1. / 60., &[]);
        }
        let kills = app.world().resource::<Encounter>().kills;
        let remaining: u32 = app
            .world_mut()
            .query::<&Enemy>()
            .iter(app.world())
            .map(|e| e.health)
            .sum();
        let mut contact = fixture(upgraded);
        contact
            .world_mut()
            .resource_mut::<CombatConfig>()
            .target_range = 0.;
        enemy(&mut contact, START, 10000);
        let initial = contact.world().resource::<PlayerHealth>().current;
        for _ in 0..240 {
            step(&mut contact, 1. / 60., &[]);
        }
        let hull = contact.world().resource::<PlayerHealth>().current;
        results.push((kills, remaining, initial - hull, hull));
    }
    println!(
        "DRO-15 FIXED ENCOUNTER (all modules absent): 12s offense (six stationary 100-HP enemies), 4s contact; base={:?}, upgraded={:?}; tuple=(kills, enemy hull remaining, contact hull lost, player hull remaining)",
        results[0], results[1]
    );
    assert!(results[1].0 > results[0].0);
    assert!(results[1].1 < results[0].1);
    assert!(results[1].2 < results[0].2);
    assert!(results[1].3 > results[0].3);
}
#[test]
fn upgraded_targeting_fires_and_hits_at_extended_reach() {
    for upgraded in [false, true] {
        let mut app = fixture(upgraded);
        let target = enemy(&mut app, START + Vec3::Z * 590., 100);
        for _ in 0..66 {
            step(&mut app, 1. / 60., &[]);
        }
        let remaining = app.world().get::<Enemy>(target).unwrap().health;
        if upgraded {
            assert!(remaining < 100);
        } else {
            assert_eq!(remaining, 100);
        }
    }
}
#[test]
fn armor_does_not_reduce_hazard_damage_and_protection_extends_shared_window() {
    let app = fixture(true);
    let config = app.world().resource::<CombatConfig>().clone();
    let mut health = PlayerHealth {
        current: 200,
        invulnerable_until: 0.,
    };
    let mut phase = GamePhase::Playing;
    let mut outcomes = CombatOutcomes::default();
    let mut power = crate::energy::PowerFrame::default();
    let modules = crate::modules::ModuleConfig::default();
    assert!(lifecycle::apply_player_damage(
        crate::world::hazard::HAZARD_DAMAGE,
        0.,
        &config,
        &mut health,
        &mut phase,
        &mut outcomes,
        &mut power,
        &modules
    ));
    assert_eq!(health.current, 200 - crate::world::hazard::HAZARD_DAMAGE);
    assert!(!lifecycle::apply_player_damage(
        10,
        1.,
        &config,
        &mut health,
        &mut phase,
        &mut outcomes,
        &mut power,
        &modules
    ));
    assert!(lifecycle::apply_player_damage(
        10,
        1.125,
        &config,
        &mut health,
        &mut phase,
        &mut outcomes,
        &mut power,
        &modules
    ));
}
