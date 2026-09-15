use super::*;
use crate::{
    combat::Encounter,
    game::GamePhase,
    mission::{
        Campaign, MissionSession,
        tests::{launch, tick},
    },
    passives::NodeId,
};

fn app(path: &std::path::Path) -> App {
    let mut app = crate::mission::tests::app();
    app.add_plugins(SavePlugin::at(path.to_path_buf()));
    tick(&mut app, 0., &[]);
    app
}
fn key(app: &mut App, key: KeyCode) {
    tick(app, 0., &[]);
    tick(app, 0., &[key]);
}

#[test]
fn missing_save_new_campaign_and_reload_never_credit_result_twice() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::CampaignMenu
    );
    key(&mut app, KeyCode::KeyN);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    launch(&mut app);
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    tick(&mut app, 0., &[]);
    let bank = app.world().resource::<Campaign>().wallet;
    assert_eq!(bank.salvage, 10);
    drop(app);
    let mut restored = self::app(&path);
    key(&mut restored, KeyCode::Enter);
    assert_eq!(*restored.world().resource::<GamePhase>(), GamePhase::Hub);
    assert_eq!(restored.world().resource::<Campaign>().wallet, bank);
    assert_eq!(restored.world().resource::<Campaign>().history.len(), 1);
    assert!(
        restored
            .world()
            .resource::<MissionSession>()
            .result
            .is_none()
    );
    tick(&mut restored, 500., &[]);
    assert_eq!(restored.world().resource::<Campaign>().wallet, bank);
    launch(&mut restored);
    restored.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    tick(&mut restored, 0., &[]);
    let campaign = restored.world().resource::<Campaign>();
    assert_eq!(campaign.wallet.salvage, 20);
    assert_eq!(campaign.history[1].attempt, 2);
}

#[test]
fn purchases_save_immediately_and_interrupted_loot_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    key(&mut app, KeyCode::KeyN);
    app.world_mut().resource_mut::<Campaign>().wallet.salvage = 50;
    key(&mut app, KeyCode::KeyU);
    key(&mut app, KeyCode::Digit1);
    let expected = app.world().resource::<Campaign>().wallet;
    key(&mut app, KeyCode::Backspace);
    launch(&mut app);
    app.world_mut()
        .resource_mut::<crate::economy::AttemptResources>()
        .collected
        .salvage = 900;
    drop(app);
    let mut restored = self::app(&path);
    key(&mut restored, KeyCode::Enter);
    let campaign = restored.world().resource::<Campaign>();
    assert_eq!(campaign.wallet, expected);
    assert_eq!(campaign.passives.rank(NodeId::Damage), 1);
    assert!(campaign.history.is_empty());
}

#[test]
fn replacement_requires_fresh_confirmation_and_cancel_preserves_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    key(&mut app, KeyCode::KeyN);
    app.world_mut().resource_mut::<Campaign>().wallet.salvage = 42;
    tick(&mut app, 0., &[]);
    let before = std::fs::read(&path).unwrap();
    key(&mut app, KeyCode::KeyN); // open campaign menu from hub
    key(&mut app, KeyCode::KeyN); // request replacement
    tick(&mut app, 0., &[KeyCode::Enter]); // held/chorded input cannot confirm
    assert_eq!(std::fs::read(&path).unwrap(), before);
    key(&mut app, KeyCode::Backspace);
    assert_eq!(std::fs::read(&path).unwrap(), before);
    key(&mut app, KeyCode::KeyN);
    key(&mut app, KeyCode::Enter);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 0);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
}

#[test]
fn failed_autosave_blocks_launch_and_retry_commits_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    key(&mut app, KeyCode::KeyN);
    launch(&mut app);
    std::fs::create_dir(path.with_extension("previous.json")).unwrap();
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    tick(&mut app, 0., &[]);
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::CampaignMenu
    );
    assert!(matches!(
        app.world().resource::<SaveState>().mode,
        Mode::Failed(_)
    ));
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 10);
    key(&mut app, KeyCode::Enter);
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::CampaignMenu
    );
    std::fs::remove_dir(path.with_extension("previous.json")).unwrap();
    key(&mut app, KeyCode::KeyR);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Survived);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 10);
    let mut restored = self::app(&path);
    key(&mut restored, KeyCode::Enter);
    assert_eq!(restored.world().resource::<Campaign>().wallet.salvage, 10);
}

#[test]
fn invalid_and_future_save_can_retry_or_cancel_recovery_without_writes() {
    for bytes in [b"broken save".as_slice(), br#"{"version":99}"#.as_slice()] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("campaign.json");
        std::fs::write(&path, bytes).unwrap();
        let mut app = app(&path);
        key(&mut app, KeyCode::Enter);
        assert_eq!(
            *app.world().resource::<GamePhase>(),
            GamePhase::CampaignMenu
        );
        key(&mut app, KeyCode::KeyN);
        key(&mut app, KeyCode::Backspace);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        key(&mut app, KeyCode::KeyR);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        key(&mut app, KeyCode::KeyN);
        key(&mut app, KeyCode::Enter);
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
        assert!(Snapshot::decode(&std::fs::read(&path).unwrap()).is_ok());
    }
}

#[test]
fn module_edits_and_mission_selection_persist_without_launch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    key(&mut app, KeyCode::KeyN);
    {
        let mut campaign = app.world_mut().resource_mut::<Campaign>();
        campaign.wallet.salvage = 100;
        campaign
            .progress
            .complete(crate::mission::campaign::MissionId::ALL[0]);
    }
    key(&mut app, KeyCode::KeyM);
    key(&mut app, KeyCode::KeyB);
    key(&mut app, KeyCode::Digit3);
    key(&mut app, KeyCode::Backspace);
    key(&mut app, KeyCode::KeyC);
    key(&mut app, KeyCode::ArrowRight);
    let before = Snapshot::capture(
        app.world().resource::<Campaign>(),
        app.world().resource::<MissionSession>(),
    );
    drop(app);
    let mut restored = self::app(&path);
    key(&mut restored, KeyCode::Enter);
    assert_eq!(
        Snapshot::capture(
            restored.world().resource::<Campaign>(),
            restored.world().resource::<MissionSession>()
        ),
        before
    );
    assert_eq!(
        restored
            .world()
            .resource::<Campaign>()
            .inventory
            .loadout()
            .slots()[2],
        Some(crate::modules::ModuleKind::Overdrive)
    );
    assert_eq!(
        restored
            .world()
            .resource::<MissionSession>()
            .selected_mission,
        crate::mission::campaign::MissionId::ALL[1]
    );
}
