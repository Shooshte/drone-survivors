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
