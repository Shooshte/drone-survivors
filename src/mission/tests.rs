use super::*;
use crate::{
    arena::{ArenaPlugin, DRONE_START, Drone},
    combat::{CombatPlugin, Encounter, PlayerHealth},
    energy::Energy,
    game::GamePhase,
    upgrades::{UpgradeRun, runtime::UpgradePlugin},
};
use std::time::Duration;
pub(crate) fn app() -> App {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::ZERO,
        ))
        .add_plugins((
            bevy::time::TimePlugin,
            ArenaPlugin,
            CombatPlugin,
            UpgradePlugin,
            MissionPlugin,
        ));
    app.update();
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_max_delta(Duration::from_secs(600));
    app
}
pub(crate) fn tick(app: &mut App, dt: f64, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.reset_all();
    for key in keys {
        input.press(*key);
    }
    *app.world_mut()
        .resource_mut::<bevy::time::TimeUpdateStrategy>() =
        bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(dt));
    app.update();
}
pub(crate) fn launch(app: &mut App) {
    tick(app, 0., &[]);
    tick(app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    tick(app, 0., &[]);
    tick(app, 1., &[KeyCode::Enter, KeyCode::KeyW, KeyCode::Digit1]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
}
#[test]
fn hub_freezes_gameplay_and_ignores_restart() {
    let mut app = app();
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    app.world_mut().resource_mut::<Energy>().current = 37.;
    tick(&mut app, 100., &[KeyCode::KeyR, KeyCode::KeyW]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    assert_eq!(app.world().resource::<Energy>().current, 37.);
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
}
#[test]
fn launch_and_choice_restart_restore_baseline_without_completing_interrupted_attempt() {
    let mut app = app();
    launch(&mut app);
    assert_eq!(
        app.world_mut()
            .query_filtered::<&Transform, With<Drone>>()
            .single(app.world())
            .unwrap(),
        &DRONE_START
    );
    app.world_mut().resource_mut::<PlayerHealth>().current = 9;
    app.world_mut().resource_mut::<Energy>().current = 8.;
    app.world_mut().resource_mut::<Encounter>().kills = 42;
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    tick(&mut app, 20., &[KeyCode::KeyR, KeyCode::Digit1]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
    assert_eq!(app.world().resource::<Energy>().current, 100.);
    assert_eq!(app.world().resource::<Encounter>().kills, 0);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
    assert!(app.world().resource::<Campaign>().history.is_empty());
}
#[test]
fn terminal_results_are_stable_deduplicated_and_campaign_retains_success() {
    let mut app = app();
    launch(&mut app);
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    app.world_mut().resource_mut::<Encounter>().kills = 3;
    tick(&mut app, 0., &[]);
    let first = app
        .world()
        .resource::<MissionSession>()
        .result
        .clone()
        .unwrap();
    assert!(first.succeeded);
    assert_eq!(first.kills, 3);
    tick(&mut app, 40., &[KeyCode::KeyR]);
    tick(&mut app, 40., &[]);
    assert_eq!(
        app.world().resource::<MissionSession>().result.as_ref(),
        Some(&first)
    );
    assert_eq!(app.world().resource::<Campaign>().history.len(), 1);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    launch(&mut app);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    tick(&mut app, 0., &[]);
    let campaign = app.world().resource::<Campaign>();
    assert_eq!(campaign.history.len(), 2);
    assert!(
        campaign
            .progress
            .completed(super::campaign::MissionId::ALL[0])
    );
    assert_ne!(campaign.history[0].attempt, campaign.history[1].attempt);
    assert!(!campaign.history[1].succeeded);
}
#[test]
fn held_keyboard_cannot_advance_two_screens() {
    let mut app = app();
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}

#[test]
fn held_click_cannot_follow_a_reused_buttons_new_action() {
    let mut app = app();
    let button = app
        .world_mut()
        .spawn((MissionAction::Briefing, Interaction::Pressed))
        .id();
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    *app.world_mut().get_mut::<MissionAction>(button).unwrap() = MissionAction::Launch;
    tick(&mut app, 0., &[]);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
    tick(&mut app, 0., &[]);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}
#[test]
fn objective_and_virtual_clock_resume_without_catching_up_choice_or_menu_time() {
    let mut app = app();
    tick(&mut app, 100., &[]);
    launch(&mut app);
    app.world_mut().resource_mut::<Encounter>().elapsed = 299.5;
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
    let now = app.world().resource::<Time<Virtual>>().elapsed_secs_f64();
    tick(&mut app, 100., &[]);
    assert_eq!(
        app.world().resource::<Time<Virtual>>().elapsed_secs_f64(),
        now
    );
    assert_eq!(app.world().resource::<Encounter>().elapsed, 299.5);
    tick(&mut app, 100., &[KeyCode::Backspace]);
    tick(&mut app, 0.1, &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    assert!((app.world().resource::<Encounter>().elapsed - 299.6).abs() < 1e-5);
    tick(&mut app, 0.4, &[]);
    assert!(
        app.world()
            .resource::<MissionSession>()
            .result
            .as_ref()
            .unwrap()
            .succeeded
    );
}

#[test]
fn menu_frames_do_not_consume_wave_bookkeeping() {
    let mut app = app();
    app.world_mut().resource_mut::<Encounter>().elapsed = 20.;
    tick(&mut app, 20., &[]);
    assert_eq!(app.world().resource::<Encounter>().next_burst, 0);
}

#[test]
fn zero_hull_is_terminal_before_pending_choice() {
    let mut app = app();
    launch(&mut app);
    app.world_mut().resource_mut::<PlayerHealth>().current = 0;
    app.world_mut().resource_mut::<UpgradeRun>().award(50);
    tick(&mut app, 0., &[]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Dead);
    assert!(app.world().resource::<UpgradeRun>().offer.is_empty());
}

#[path = "economy_tests.rs"]
mod economy_tests;

#[test]
fn purchased_secret_survives_success_but_restart_and_failure_forfeit_it() {
    let mut app = app();
    {
        let mut campaign = app.world_mut().resource_mut::<Campaign>();
        campaign.wallet.salvage = 60;
        campaign.wallet.components = 3;
        assert!(campaign.secrets.discover_blueprint());
    }
    tick(&mut app, 0., &[KeyCode::KeyU]);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::KeyB]);
    assert!(app.world().resource::<Campaign>().secrets.reserve_battery);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 40);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    launch(&mut app);
    assert_eq!(
        app.world()
            .resource::<crate::energy::EnergyConfig>()
            .capacity,
        125.
    );
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    tick(&mut app, 0., &[]);
    assert!(app.world().resource::<Campaign>().secrets.reserve_battery);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    launch(&mut app);
    tick(&mut app, 0., &[KeyCode::KeyR]);
    assert!(!app.world().resource::<Campaign>().secrets.reserve_battery);
    assert_eq!(
        app.world()
            .resource::<crate::energy::EnergyConfig>()
            .capacity,
        100.
    );
    assert!(app.world().resource::<Campaign>().secrets.blueprint);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    tick(&mut app, 0., &[]);
    assert!(!app.world().resource::<Campaign>().secrets.reserve_battery);
}

#[test]
fn death_forfeits_active_secret_and_blueprint_supports_repurchase_with_passive_bonus() {
    let mut app = app();
    {
        let mut campaign = app.world_mut().resource_mut::<Campaign>();
        campaign.wallet.salvage = 100;
        campaign.wallet.components = 3;
        let Campaign {
            passives, wallet, ..
        } = &mut *campaign;
        passives
            .purchase(crate::passives::NodeId::Battery, wallet)
            .unwrap();
        campaign.secrets.discover_blueprint();
        let Campaign {
            secrets, wallet, ..
        } = &mut *campaign;
        secrets.purchase(wallet).unwrap();
    }
    launch(&mut app);
    assert_eq!(
        app.world()
            .resource::<crate::energy::EnergyConfig>()
            .capacity,
        145.
    );
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    tick(&mut app, 0., &[]);
    assert!(!app.world().resource::<Campaign>().secrets.reserve_battery);
    assert!(app.world().resource::<Campaign>().secrets.blueprint);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter]);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::KeyU]);
    tick(&mut app, 0., &[]);
    let before = app.world().resource::<Campaign>().wallet;
    tick(&mut app, 0., &[KeyCode::KeyB]);
    let after = app.world().resource::<Campaign>();
    assert!(after.secrets.reserve_battery);
    assert_eq!(after.wallet.salvage, before.salvage - 20);
    assert_eq!(after.wallet.components, before.components - 1);
}
