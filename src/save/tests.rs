use super::{snapshot::Snapshot, storage::Store};
use crate::{
    mission::{Campaign, MissionSession, campaign::MissionId},
    modules::ModuleKind,
    passives::NodeId,
};

fn populated() -> (Campaign, MissionSession) {
    let mut campaign = Campaign::default();
    campaign.wallet.salvage = 100;
    campaign.wallet.components = 7;
    campaign
        .passives
        .purchase(NodeId::Damage, &mut campaign.wallet)
        .unwrap();
    campaign
        .inventory
        .purchase(ModuleKind::Shield, &mut campaign.wallet)
        .unwrap();
    campaign
        .inventory
        .assign(2, Some(ModuleKind::Shield))
        .unwrap();
    campaign.progress.complete(MissionId::ALL[0]);
    let mut session = MissionSession::default();
    session.selected_mission = MissionId::ALL[2];
    (campaign, session)
}

#[test]
fn round_trip_restores_permanent_state_without_active_attempt() {
    let (campaign, session) = populated();
    let snapshot = Snapshot::capture(&campaign, &session);
    let decoded = Snapshot::decode(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
    let (restored, session) = decoded.restore().unwrap();
    assert_eq!(restored.wallet, campaign.wallet);
    assert_eq!(restored.passives.rank(NodeId::Damage), 1);
    assert!(restored.inventory.owns(ModuleKind::Shield));
    assert_eq!(restored.inventory.loadout(), campaign.inventory.loadout());
    assert_eq!(restored.progress.count(), 1);
    assert_eq!(session.selected_mission, MissionId::ALL[2]);
    assert!(session.result.is_none());
    assert!(session.active_mission.is_none());
}

#[test]
fn rejects_future_malformed_and_impossible_snapshots() {
    let (campaign, session) = populated();
    let value = serde_json::to_value(Snapshot::capture(&campaign, &session)).unwrap();
    for (field, invalid) in [
        ("version", serde_json::json!(999)),
        ("ranks", serde_json::json!([6, 0, 0, 0, 0, 0, 0, 0, 0])),
        ("ranks", serde_json::json!([0, 1, 0, 0, 0, 0, 0, 0, 0])),
        (
            "completed",
            serde_json::json!([
                false, true, false, false, false, false, false, false, false, false, false, false
            ]),
        ),
        ("selected_mission", serde_json::json!(99)),
        ("selected_mission", serde_json::json!(11)),
        ("owned", serde_json::json!([])),
        ("slots", serde_json::json!(["Shield", "Shield", null, null])),
        ("slots", serde_json::json!(["Unknown", null, null, null])),
    ] {
        let mut invalid_value = value.clone();
        invalid_value[field] = invalid;
        assert!(
            Snapshot::decode(&serde_json::to_vec(&invalid_value).unwrap()).is_err(),
            "{field}"
        );
    }
    assert!(Snapshot::decode(b"not json").is_err());
    assert!(Snapshot::decode(b"{}").is_err());
}

#[test]
fn atomic_save_keeps_previous_and_rejects_external_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut store = Store::new(path.clone());
    assert!(store.read().unwrap().is_none());
    let (mut campaign, session) = populated();
    let first = Snapshot::capture(&campaign, &session);
    store.write(&first, false).unwrap();
    let original = std::fs::read(&path).unwrap();
    campaign.wallet.salvage += 1;
    store
        .write(&Snapshot::capture(&campaign, &session), false)
        .unwrap();
    assert_eq!(
        std::fs::read(path.with_extension("previous.json")).unwrap(),
        original
    );
    std::fs::write(&path, b"external edits").unwrap();
    assert!(store.write(&first, false).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"external edits");
}

#[test]
fn invalid_save_requires_explicit_replacement_and_is_archived() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    std::fs::write(&path, b"precious invalid bytes").unwrap();
    let mut store = Store::new(path.clone());
    assert!(store.read().is_err());
    let fresh = Snapshot::capture(&Campaign::default(), &MissionSession::default());
    assert!(store.write(&fresh, false).is_err());
    store.write(&fresh, true).unwrap();
    assert!(store.read().unwrap().is_some());
    assert!(
        std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("campaign-replaced-")
                && std::fs::read(entry.path()).unwrap() == b"precious invalid bytes")
    );
}

#[test]
fn failed_replacement_leaves_original_intact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut store = Store::new(path.clone());
    store.read().unwrap();
    let fresh = Snapshot::capture(&Campaign::default(), &MissionSession::default());
    store.write(&fresh, false).unwrap();
    let original = std::fs::read(&path).unwrap();
    std::fs::create_dir(path.with_extension("previous.json")).unwrap();
    assert!(store.write(&fresh, false).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn all_unlocks_and_maxed_purchases_survive_round_trip() {
    let mut campaign = Campaign::default();
    campaign.wallet.salvage = 10_000;
    campaign.wallet.components = 1_000;
    for node in NodeId::ALL {
        for _ in 0..5 {
            campaign
                .passives
                .purchase(node, &mut campaign.wallet)
                .unwrap();
        }
    }
    for (slot, kind) in ModuleKind::ALL.into_iter().enumerate() {
        campaign
            .inventory
            .purchase(kind, &mut campaign.wallet)
            .unwrap();
        campaign.inventory.assign(slot, Some(kind)).unwrap();
    }
    for id in MissionId::ALL {
        campaign.progress.complete(id);
    }
    let mut session = MissionSession::default();
    session.selected_mission = MissionId::ALL[11];
    let first = Snapshot::capture(&campaign, &session);
    let (restored, resumed) = Snapshot::decode(&serde_json::to_vec(&first).unwrap())
        .unwrap()
        .restore()
        .unwrap();
    assert!(restored.progress.finished());
    assert_eq!(first, Snapshot::capture(&restored, &resumed));
}

#[test]
fn busy_lock_and_stale_missing_reader_cannot_overwrite_campaign() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("campaign.json");
    let mut first = Store::new(path.clone());
    let mut second = Store::new(path.clone());
    first.read().unwrap();
    second.read().unwrap();
    let snapshot = Snapshot::capture(&Campaign::default(), &MissionSession::default());
    first.write(&snapshot, false).unwrap();
    assert!(
        second
            .write(&snapshot, false)
            .unwrap_err()
            .contains("changed in another app")
    );
    let original = std::fs::read(&path).unwrap();
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(first.write(&snapshot, false).unwrap_err().contains("busy"));
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

#[test]
fn snapshot_secret_defaults_routes_and_purchase_validation() {
    let (mut campaign, mut session) = populated();
    let legacy = serde_json::to_value(Snapshot::capture(&campaign, &session)).unwrap();
    let mut legacy = legacy.as_object().unwrap().clone();
    legacy.remove("secrets");
    legacy.remove("routes");
    let (restored, _) = Snapshot::decode(&serde_json::to_vec(&legacy).unwrap())
        .unwrap()
        .restore()
        .unwrap();
    assert!(!restored.secrets.blueprint);
    assert!(!restored.secrets.reserve_battery);
    assert_eq!(restored.progress.routes(), [false; 3]);

    let mut invalid = serde_json::to_value(Snapshot::capture(&campaign, &session)).unwrap();
    invalid["secrets"] = serde_json::json!({"blueprint": false, "reserve_battery": true});
    assert!(Snapshot::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
    invalid["secrets"] = serde_json::json!({"blueprint": true, "reserve_battery": true});
    invalid["routes"] = serde_json::json!([false, true, false]);
    assert!(Snapshot::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());

    campaign.secrets.discover_blueprint();
    campaign.secrets.purchase(&mut campaign.wallet).unwrap();
    campaign.progress.discover_route(0);
    session.selected_mission = MissionId::ALL[3];
    let snap = Snapshot::capture(&campaign, &session);
    let (restored, resumed) = Snapshot::decode(&serde_json::to_vec(&snap).unwrap())
        .unwrap()
        .restore()
        .unwrap();
    assert!(restored.secrets.reserve_battery);
    assert!(restored.progress.unlocked(resumed.selected_mission));
    assert_eq!(restored.progress.count(), 1);
}
