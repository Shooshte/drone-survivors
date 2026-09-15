use crate::{
    economy::Amounts,
    mission::{
        Campaign, MissionResult, MissionSession,
        campaign::{MissionId, Progress},
    },
    modules::{ModuleKind, shop::ModuleInventory},
    passives::{NodeId, PassiveTree},
};
use serde::{Deserialize, Serialize};

/// Deliberately excludes ECS entities, active results, loot and temporary upgrades.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Snapshot {
    version: u32,
    wallet: Amounts,
    ranks: [u8; 9],
    owned: Vec<ModuleKind>,
    slots: [Option<ModuleKind>; 4],
    completed: [bool; 12],
    selected_mission: MissionId,
    next_attempt: u64,
    history: Vec<MissionResult>,
}

impl Snapshot {
    pub fn capture(campaign: &Campaign, session: &MissionSession) -> Self {
        Self {
            version: 1,
            wallet: campaign.wallet,
            ranks: NodeId::ALL.map(|node| campaign.passives.rank(node)),
            owned: ModuleKind::ALL
                .into_iter()
                .filter(|&kind| campaign.inventory.owns(kind))
                .collect(),
            slots: *campaign.inventory.loadout().slots(),
            completed: MissionId::ALL.map(|id| campaign.progress.completed(id)),
            selected_mission: session.selected_mission,
            next_attempt: session.next_attempt,
            history: campaign.history.clone(),
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|error| format!("Invalid save: {error}"))?;
        if value.get("version").and_then(|v| v.as_u64()) != Some(1) {
            return Err("Incompatible save version. This build supports version 1.".into());
        }
        let snapshot: Self =
            serde_json::from_value(value).map_err(|error| format!("Invalid save: {error}"))?;
        snapshot.clone().restore()?;
        Ok(snapshot)
    }

    pub fn restore(self) -> Result<(Campaign, MissionSession), String> {
        let invalid = |reason| format!("Invalid save: {reason}");
        let passives = PassiveTree::from_ranks(self.ranks).map_err(invalid)?;
        let inventory = ModuleInventory::from_saved(self.owned, self.slots).map_err(invalid)?;
        let mut progress = Progress::default();
        for id in MissionId::ALL {
            if self.completed[id.index()] {
                if !progress.unlocked(id) {
                    return Err(invalid("mission prerequisites are incomplete"));
                }
                progress.complete(id);
            }
        }
        if !progress.unlocked(self.selected_mission) {
            return Err(invalid("selected mission is locked"));
        }
        if self.next_attempt == u64::MAX {
            return Err(invalid("attempt counter is exhausted"));
        }
        let mut previous = 0;
        for result in &self.history {
            if result.attempt <= previous
                || result.attempt > self.next_attempt
                || !result.elapsed.is_finite()
                || result.elapsed < 0.
                || !progress.unlocked(result.mission)
                || (result.succeeded && !progress.completed(result.mission))
            {
                return Err(invalid("attempt history is inconsistent"));
            }
            previous = result.attempt;
        }
        let campaign = Campaign {
            wallet: self.wallet,
            passives,
            inventory,
            progress,
            history: self.history,
        };
        let mut session = MissionSession::default();
        session.selected_mission = self.selected_mission;
        session.next_attempt = self.next_attempt;
        Ok((campaign, session))
    }
}
