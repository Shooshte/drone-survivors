//! Region resource tuning over the shared placeholder arena.
use crate::{
    economy::runtime::EconomyConfig,
    energy::ChargerConfig,
    game::GameplaySet,
    mission::{MissionSession, campaign::MissionId},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RegionProfile {
    pub name: &'static str,
    pub salvage_chance: u8,
    pub cache_components: u64,
    pub charger_capacity: f64,
}
impl RegionProfile {
    pub fn for_mission(mission: MissionId) -> Self {
        match mission.index() / 4 {
            0 => Self {
                name: "Scrapyard",
                salvage_chance: 50,
                cache_components: 1,
                charger_capacity: 200.,
            },
            1 => Self {
                name: "Ruins",
                salvage_chance: 25,
                cache_components: 2,
                charger_capacity: 200.,
            },
            _ => Self {
                name: "Power station",
                salvage_chance: 25,
                cache_components: 1,
                charger_capacity: 300.,
            },
        }
    }
    pub fn summary(self) -> String {
        format!(
            "{} / Salvage: {}% per chaser\n3 caches: {} component(s) each / 6 chargers: {:.0} energy each",
            self.name, self.salvage_chance, self.cache_components, self.charger_capacity
        )
    }
}

pub(crate) fn install(app: &mut App) {
    app.add_systems(
        Update,
        configure
            .in_set(GameplaySet::Baseline)
            .run_if(crate::game::reset_requested),
    );
}
fn configure(
    session: Option<Res<MissionSession>>,
    mut economy: ResMut<EconomyConfig>,
    mut chargers: ResMut<ChargerConfig>,
) {
    let Some(mission) = session.and_then(|s| s.active_mission) else {
        return;
    };
    let profile = RegionProfile::for_mission(mission);
    economy.chaser.chance_percent = profile.salvage_chance;
    economy.chaser.amount = 1;
    economy.cache_components = profile.cache_components;
    chargers.capacity = profile.charger_capacity;
}
