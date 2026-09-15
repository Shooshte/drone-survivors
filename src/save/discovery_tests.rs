//! Prove real cache collection reaches durable storage before a mission result.
use super::*;
use crate::{arena::Drone, economy::runtime::EconomyConfig, upgrades::UpgradeRun};

#[test]
fn actual_cache_discoveries_save_during_play_and_upgrade_choice_without_banking_loot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    key(&mut app, KeyCode::KeyN);
    launch(&mut app);
    let sites = app.world().resource::<EconomyConfig>().caches;
    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    for (index, phase) in [(0, GamePhase::Playing), (1, GamePhase::Choosing)] {
        app.world_mut()
            .get_mut::<Transform>(drone)
            .unwrap()
            .translation = sites[index] + Vec3::Y * 20.;
        tick(&mut app, 0., &[]);
        assert_eq!(*app.world().resource::<GamePhase>(), phase);
        let saved = Snapshot::decode(&std::fs::read(&path).unwrap())
            .unwrap()
            .restore()
            .unwrap()
            .0;
        assert!(saved.secrets.blueprint);
        assert_eq!(saved.progress.routes()[0], index == 1);
        assert_eq!(saved.wallet, crate::economy::Amounts::default());
        assert!(saved.history.is_empty());
        assert_eq!(
            app.world().resource::<UpgradeRun>().total_xp,
            (index as u32 + 1) * 30
        );
    }
    key(&mut app, KeyCode::KeyR);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
    drop(app);
    let mut restored = super::app(&path);
    key(&mut restored, KeyCode::Enter);
    assert!(restored.world().resource::<Campaign>().secrets.blueprint);
    assert!(restored.world().resource::<Campaign>().progress.routes()[0]);
}

#[test]
fn shortcut_victories_reload_without_inventing_branch_completion() {
    use crate::mission::campaign::MissionId;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut app = app(&path);
    key(&mut app, KeyCode::KeyN);
    for act in 0..3 {
        assert!(
            app.world_mut()
                .resource_mut::<Campaign>()
                .progress
                .discover_route(act)
        );
        app.world_mut()
            .resource_mut::<MissionSession>()
            .selected_mission = MissionId::ALL[act * 4 + 3];
        launch(&mut app);
        app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
        tick(&mut app, 0., &[]);
        assert!(
            app.world()
                .resource::<MissionSession>()
                .result
                .as_ref()
                .unwrap()
                .succeeded
        );
        drop(app);
        app = super::app(&path);
        key(&mut app, KeyCode::Enter);
        let progress = &app.world().resource::<Campaign>().progress;
        assert_eq!(progress.count(), act + 1);
        assert!(!progress.finished());
        assert!(!progress.completed(MissionId::ALL[act * 4 + 1]));
        assert!(!progress.completed(MissionId::ALL[act * 4 + 2]));
        if act < 2 {
            assert!(progress.unlocked(MissionId::ALL[(act + 1) * 4]));
        }
    }
}
