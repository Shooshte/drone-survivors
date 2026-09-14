use super::{
    Campaign,
    tests::{app, launch, tick},
};
use crate::{game::GamePhase, modules::Modules};
use bevy::prelude::*;

#[test]
fn new_campaign_can_launch_with_four_empty_slots() {
    let mut app = app();
    launch(&mut app);
    assert_eq!(
        app.world().resource::<Modules>().loadout.slots(),
        &[None; 4]
    );
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    assert_eq!(app.world().resource::<Campaign>().wallet, default());
}

#[test]
fn module_shop_opens_from_hub_and_freezes_attempt_time() {
    let mut app = app();
    tick(&mut app, 0., &[KeyCode::KeyM]);
    assert_ne!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    let now = app.world().resource::<Time<Virtual>>().elapsed();
    tick(&mut app, 30., &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<Time<Virtual>>().elapsed(), now);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
}

fn press(app: &mut App, keys: &[KeyCode]) {
    tick(app, 0., &[]);
    tick(app, 0., keys);
}
fn shop() -> App {
    let mut app = app();
    app.world_mut().resource_mut::<Campaign>().wallet = crate::economy::Amounts {
        salvage: 65,
        components: 1,
    };
    press(&mut app, &[KeyCode::KeyM]);
    tick(&mut app, 0., &[]);
    app
}
fn click(app: &mut App, action: super::MissionAction) {
    tick(app, 0., &[]);
    let button = app.world_mut().spawn((action, Interaction::Pressed)).id();
    tick(app, 0., &[]);
    app.world_mut().despawn(button);
}
#[test]
fn buy_assign_move_remove_and_duplicate_purchase_are_atomic_through_input() {
    use crate::modules::ModuleKind;
    let mut app = shop();
    press(&mut app, &[KeyCode::Digit1]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .inventory
            .loadout()
            .slots(),
        &[None; 4]
    );
    assert!(
        app.world()
            .resource::<super::MissionSession>()
            .purchase_feedback
            .contains("Buy")
    );
    press(&mut app, &[KeyCode::KeyB]);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 55);
    press(&mut app, &[KeyCode::KeyB]);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 55);
    press(&mut app, &[KeyCode::Digit4]);
    press(&mut app, &[KeyCode::Digit2]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .inventory
            .loadout()
            .slots(),
        &[None, Some(ModuleKind::Overdrive), None, None]
    );
    press(&mut app, &[KeyCode::ShiftLeft, KeyCode::Digit2]);
    assert!(
        app.world()
            .resource::<Campaign>()
            .inventory
            .owns(ModuleKind::Overdrive)
    );
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .inventory
            .loadout()
            .slots(),
        &[None; 4]
    );
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 55);
    click(
        &mut app,
        super::MissionAction::SelectModule(ModuleKind::Rocket),
    );
    app.world_mut().resource_mut::<Campaign>().wallet.components = 0;
    click(&mut app, super::MissionAction::BuyModule);
    assert!(
        !app.world()
            .resource::<Campaign>()
            .inventory
            .owns(ModuleKind::Rocket)
    );
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 55);
}
#[test]
fn held_inputs_and_simultaneous_mouse_actions_do_not_chain_purchase_and_assignment() {
    let mut app = shop();
    press(&mut app, &[KeyCode::KeyB, KeyCode::Digit1]);
    for _ in 0..3 {
        tick(&mut app, 0., &[KeyCode::Digit1]);
    }
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 55);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .inventory
            .loadout()
            .slots(),
        &[None; 4]
    );
    tick(&mut app, 0., &[]);
    let a = app
        .world_mut()
        .spawn((super::MissionAction::AssignModule(0), Interaction::Pressed))
        .id();
    let b = app
        .world_mut()
        .spawn((super::MissionAction::AssignModule(3), Interaction::Pressed))
        .id();
    for _ in 0..3 {
        tick(&mut app, 0., &[]);
    }
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .inventory
            .loadout()
            .slots(),
        &[
            Some(crate::modules::ModuleKind::Overdrive),
            None,
            None,
            None
        ]
    );
    app.world_mut().despawn(a);
    app.world_mut().despawn(b);
    press(&mut app, &[KeyCode::Backspace]);
    press(&mut app, &[KeyCode::KeyM, KeyCode::KeyB]);
    for _ in 0..3 {
        tick(&mut app, 0., &[KeyCode::KeyB]);
    }
    assert!(
        app.world()
            .resource::<super::MissionSession>()
            .purchase_feedback
            .is_empty()
    );
}
#[test]
fn launch_and_restart_use_snapshot_and_replay_uses_next_hub_arrangement() {
    use crate::modules::ModuleKind;
    let mut app = shop();
    for (slot, kind) in ModuleKind::ALL.into_iter().rev().enumerate() {
        click(&mut app, super::MissionAction::SelectModule(kind));
        click(&mut app, super::MissionAction::BuyModule);
        press(&mut app, &[crate::modules::SLOT_KEYS[slot]]);
    }
    let expected = app
        .world()
        .resource::<Campaign>()
        .inventory
        .loadout()
        .clone();
    assert_eq!(app.world().resource::<Campaign>().wallet, default());
    press(&mut app, &[KeyCode::Backspace]);
    launch(&mut app);
    assert_eq!(app.world().resource::<Modules>().loadout, expected);
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    press(&mut app, &[KeyCode::Digit1]);
    assert!(app.world().resource::<Modules>().active(ModuleKind::Rocket));
    // Campaign state cannot be edited via combat UI. Even an external change
    // must not replace the running attempt's restart snapshot.
    app.world_mut()
        .resource_mut::<Campaign>()
        .inventory
        .assign(0, None)
        .unwrap();
    press(&mut app, &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<Modules>().loadout, expected);
    assert_eq!(app.world().resource::<Modules>().enabled, [false; 4]);
    app.world_mut()
        .resource_mut::<crate::upgrades::UpgradeRun>()
        .award(50);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    press(&mut app, &[KeyCode::KeyR]);
    assert_eq!(app.world().resource::<Modules>().loadout, expected);
    for outcome in [GamePhase::Survived, GamePhase::Dead] {
        *app.world_mut().resource_mut::<GamePhase>() = outcome;
        tick(&mut app, 0., &[]);
        press(&mut app, &[KeyCode::Enter]);
        assert!(
            app.world()
                .resource::<Campaign>()
                .inventory
                .owns(ModuleKind::Rocket)
        );
        launch(&mut app);
        assert_eq!(app.world().resource::<Modules>().loadout.slots()[0], None);
    }
    let bank = app.world().resource::<Campaign>().wallet;
    click(&mut app, super::MissionAction::BuyModule);
    assert_eq!(app.world().resource::<Campaign>().wallet, bank);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}
#[test]
fn shop_does_not_award_stale_xp_or_advance_wave_state() {
    let mut app = shop();
    let before = app
        .world()
        .resource::<crate::combat::Encounter>()
        .next_burst;
    app.world_mut()
        .resource_mut::<crate::combat::Encounter>()
        .kills = 99;
    app.world_mut()
        .resource_mut::<crate::combat::Encounter>()
        .elapsed = 120.;
    press(&mut app, &[KeyCode::KeyR, KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::ModuleShop);
    assert_eq!(
        app.world()
            .resource::<crate::upgrades::UpgradeRun>()
            .total_xp,
        0
    );
    assert_eq!(
        app.world()
            .resource::<crate::combat::Encounter>()
            .next_burst,
        before
    );
}
