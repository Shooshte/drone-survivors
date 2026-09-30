//! Mission-local offers through the production progression/choice systems.
use super::{
    tests::{app, launch, select_placeholder, tick},
    *,
};
use crate::{
    combat::{Encounter, PlayerHealth},
    modules::{Loadout, ModuleKind, Modules},
    upgrades::{UpgradeKind, UpgradePool, UpgradeRun},
};

#[test]
fn mission01_earns_new_cards_without_catalog_preview_xp() {
    let mut app = app();
    launch(&mut app);
    assert!(!app.world().resource::<UpgradePool>().catalog);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
    app.world_mut().resource_mut::<Encounter>().kills = 13;
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    let run = app.world().resource::<UpgradeRun>();
    assert_eq!(run.total_xp, 52);
    assert!(
        run.offer
            .iter()
            .any(|kind| !UpgradeKind::ALL.contains(kind)),
        "{:?}",
        run.offer
    );
    assert!(!run.offer.contains(&UpgradeKind::RapidRepair));
    assert!(!run.offer.contains(&UpgradeKind::WideRepulsor));
}

#[test]
fn mission01_all_twelve_cards_are_reachable_with_at_most_three_modules() {
    let kinds = [
        ModuleKind::Overdrive,
        ModuleKind::Shield,
        ModuleKind::Mobility,
        ModuleKind::Rocket,
        ModuleKind::Repulsor,
        ModuleKind::Repair,
    ];
    let mut seen = Vec::new();
    for mask in 0u32..64 {
        if mask.count_ones() > 3 {
            continue;
        }
        let mut app = app();
        launch(&mut app);
        // Exercise all legal Mission01 module combinations independently of funding.
        let mut slots = [None; 4];
        for (slot, kind) in kinds
            .iter()
            .enumerate()
            .filter_map(|(i, kind)| (mask & (1 << i) != 0).then_some(*kind))
            .enumerate()
        {
            slots[slot] = Some(kind);
        }
        let loadout = Loadout::new(slots).unwrap();
        app.world_mut().resource_mut::<Modules>().loadout = loadout.clone();
        app.world_mut().resource_mut::<UpgradeRun>().award(1000);
        tick(&mut app, 0., &[]);
        for _ in 0..4 {
            let offer = app.world().resource::<UpgradeRun>().offer.clone();
            assert!(!offer.is_empty());
            assert!(offer.iter().all(|kind| kind.eligible(&loadout)));
            seen.extend(offer);
            tick(&mut app, 0., &[]);
            tick(&mut app, 0., &[KeyCode::Digit1]);
        }
        tick(&mut app, 0., &[]);
        let run = app.world().resource::<UpgradeRun>();
        assert_eq!(run.resolved, 4);
        assert_eq!(run.selected.len(), 4);
        assert_eq!(run.remaining(), 0);
        assert!(run.offer.is_empty());
        tick(&mut app, 0., &[KeyCode::KeyR]);
        assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
        assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
    }
    for kind in UpgradeKind::CATALOG {
        assert!(seen.contains(&kind), "unreachable {kind:?}");
    }
}

#[test]
fn leaving_mission01_restores_ordinary_six_card_offers() {
    let mut app = app();
    launch(&mut app);
    app.world_mut().resource_mut::<PlayerHealth>().current = 0;
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter]);
    select_placeholder(&mut app, 1);
    launch(&mut app);
    app.world_mut().resource_mut::<UpgradeRun>().award(1000);
    tick(&mut app, 0., &[]);
    for _ in 0..4 {
        assert!(
            app.world()
                .resource::<UpgradeRun>()
                .offer
                .iter()
                .all(|kind| UpgradeKind::ALL.contains(kind))
        );
        tick(&mut app, 0., &[]);
        tick(&mut app, 0., &[KeyCode::Digit1]);
    }
}
