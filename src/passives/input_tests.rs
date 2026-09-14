use crate::{
    economy::Amounts,
    game::GamePhase,
    mission::{
        Campaign, MissionAction,
        tests::{app, launch, tick},
    },
    passives::NodeId,
};
use bevy::prelude::*;

fn shop() -> App {
    let mut app = app();
    app.world_mut().resource_mut::<Campaign>().wallet = Amounts {
        salvage: 1000,
        components: 100,
    };
    tick(&mut app, 0., &[KeyCode::KeyU]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Passives);
    tick(&mut app, 0., &[]);
    app
}
#[test]
fn purchase_keys_are_gated_to_shop_and_releasing_is_required_between_ranks() {
    let mut app = shop();
    tick(&mut app, 0., &[KeyCode::Digit1]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Damage),
        1
    );
    for _ in 0..4 {
        tick(&mut app, 0., &[KeyCode::Digit1]);
    }
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Damage),
        1
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Digit1, KeyCode::Digit4]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Damage),
        2
    );
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Hull),
        0
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    launch(&mut app);
    tick(&mut app, 0., &[KeyCode::Digit1, KeyCode::KeyU]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Damage),
        2
    );
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}
#[test]
fn held_mouse_and_multiple_button_events_purchase_at_most_one_rank() {
    let mut app = shop();
    let hull = app
        .world_mut()
        .spawn((MissionAction::Purchase(NodeId::Hull), Interaction::Pressed))
        .id();
    let damage = app
        .world_mut()
        .spawn((
            MissionAction::Purchase(NodeId::Damage),
            Interaction::Pressed,
        ))
        .id();
    tick(&mut app, 0., &[]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Damage),
        1
    );
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Hull),
        0
    );
    for _ in 0..3 {
        tick(&mut app, 0., &[]);
    }
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Damage),
        1
    );
    *app.world_mut().get_mut::<Interaction>(hull).unwrap() = Interaction::None;
    *app.world_mut().get_mut::<Interaction>(damage).unwrap() = Interaction::None;
    tick(&mut app, 0., &[]);
    *app.world_mut().get_mut::<Interaction>(hull).unwrap() = Interaction::Pressed;
    tick(&mut app, 0., &[]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Hull),
        1
    );
}
#[test]
fn shop_freezes_time_and_rejected_purchases_leave_bank_unchanged() {
    let mut app = shop();
    app.world_mut().resource_mut::<Campaign>().wallet = Amounts::default();
    let now = app.world().resource::<Time<Virtual>>().elapsed();
    for key in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::KeyR,
        KeyCode::Enter,
    ] {
        tick(&mut app, 20., &[]);
        tick(&mut app, 20., &[key]);
    }
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Passives);
    assert_eq!(app.world().resource::<Time<Virtual>>().elapsed(), now);
    assert_eq!(
        app.world().resource::<Campaign>().wallet,
        Amounts::default()
    );
    assert_eq!(app.world().resource::<Campaign>().passives, default());
}
#[test]
fn entering_shop_with_held_purchase_key_needs_release() {
    let mut app = app();
    app.world_mut().resource_mut::<Campaign>().wallet = Amounts {
        salvage: 1000,
        components: 100,
    };
    tick(&mut app, 0., &[KeyCode::KeyU, KeyCode::Digit7]);
    tick(&mut app, 0., &[KeyCode::Digit7]);
    tick(&mut app, 0., &[KeyCode::Digit7]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Battery),
        0
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Digit7]);
    assert_eq!(
        app.world()
            .resource::<Campaign>()
            .passives
            .rank(NodeId::Battery),
        1
    );
}
