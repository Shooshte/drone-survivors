//! Read-only launch and power projections shared by the loadout shop and briefing.
use crate::{
    energy::ChargerConfig,
    mission::Campaign,
    modules::{Loadout, ModuleConfig, ModuleKind},
    upgrades::runtime::Baseline,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PotentialPower {
    pub(crate) battery_capacity: f64,
    pub(crate) potential_drain: f64,
    pub(crate) charger_supply: f64,
    pub(crate) net_rate: f64,
    reserve_cost: f64,
    reserve_capacity: f64,
}

impl PotentialPower {
    pub(crate) fn display(&self) -> String {
        format!(
            "POWER POTENTIAL / ALL MODULES START OFF\nBATTERY {}  |  ALL-ON DRAIN {}/s\nSUPPLIED CHARGER +{}/s  |  NET {}/s  |  RESERVE -{}/s ({} per field)\nSupply: battery room or running modules. Reserve can deplete.",
            number(self.battery_capacity),
            number(self.potential_drain),
            number(self.charger_supply),
            signed(self.net_rate),
            number(self.charger_supply * self.reserve_cost),
            number(self.reserve_capacity),
        )
    }
}

/// Derives launch numbers from the captured pristine baseline, deliberately
/// excluding a current attempt's temporary upgrades and toggled module state.
pub(crate) fn potential_power(
    baseline: &Baseline,
    campaign: &Campaign,
    chargers: &ChargerConfig,
) -> PotentialPower {
    let (energy, modules) = baseline.launch_power(campaign);
    PotentialPower {
        battery_capacity: energy.capacity,
        potential_drain: total_drain(campaign.inventory.loadout(), &modules),
        charger_supply: energy.recharge,
        net_rate: energy.recharge - total_drain(campaign.inventory.loadout(), &modules),
        reserve_cost: energy.reserve_cost,
        reserve_capacity: chargers.capacity,
    }
}

pub(crate) fn total_drain(loadout: &Loadout, modules: &ModuleConfig) -> f64 {
    loadout
        .slots()
        .iter()
        .flatten()
        .map(|kind| modules.drain(*kind))
        .sum::<f64>()
        .max(0.)
}

pub(crate) fn catalog_detail(kind: ModuleKind, modules: &ModuleConfig) -> String {
    let drain = number(modules.drain(kind));
    match kind {
        ModuleKind::Overdrive => format!(
            "Basic fire rate x{}  |  drain {drain}/s  |  Tradeoff: battery draw while ON.",
            number(modules.overdrive_multiplier)
        ),
        ModuleKind::Shield => format!(
            "Blocks {} hit{}; recharge {}s  |  drain {drain}/s  |  Tradeoff: recovery needs ON power.",
            modules.shield_blocks,
            if modules.shield_blocks == 1 { "" } else { "s" },
            number(modules.shield_recharge),
        ),
        ModuleKind::Mobility => format!(
            "Horizontal thrust/speed x{}  |  drain {drain}/s  |  Tradeoff: lower energy margin.",
            number(f64::from(modules.mobility_multiplier))
        ),
        ModuleKind::Rocket => format!(
            "{} damage / {} radius / every {}s  |  drain {drain}/s  |  Tradeoff: draw while ON.",
            modules.rocket_damage,
            number(f64::from(modules.rocket_radius)),
            number(modules.rocket_interval),
        ),
    }
}

pub(crate) fn loadout_lines(loadout: &Loadout, modules: &ModuleConfig) -> String {
    loadout
        .slots()
        .iter()
        .enumerate()
        .map(|(slot, kind)| match kind {
            Some(kind) => format!(
                "SLOT {}  {}  {} / {}/s",
                slot + 1,
                kind.name(),
                slot_effect(*kind, modules),
                number(modules.drain(*kind)),
            ),
            None => format!("SLOT {}  EMPTY", slot + 1),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn slot_effect(kind: ModuleKind, modules: &ModuleConfig) -> String {
    match kind {
        ModuleKind::Overdrive => format!("x{} basic fire", number(modules.overdrive_multiplier)),
        ModuleKind::Shield => format!("{} block", modules.shield_blocks),
        ModuleKind::Mobility => format!(
            "x{} horizontal",
            number(f64::from(modules.mobility_multiplier))
        ),
        ModuleKind::Rocket => format!("{} dmg", modules.rocket_damage),
    }
}

fn number(value: f64) -> String {
    if value.abs() < 1e-6 {
        "0".into()
    } else if (value - value.round()).abs() < 1e-6 {
        format!("{value:.0}")
    } else {
        let mut rendered = format!("{value:.2}");
        while rendered.ends_with('0') {
            rendered.pop();
        }
        rendered
    }
}

fn signed(value: f64) -> String {
    if value > 0. {
        format!("+{}", number(value))
    } else {
        number(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        economy::Amounts,
        energy::ChargerConfig,
        mission::Campaign,
        modules::{ModuleConfig, ModuleKind},
        passives::NodeId,
        upgrades::runtime::Baseline,
    };

    fn campaign_with(kind: ModuleKind) -> Campaign {
        let mut campaign = Campaign {
            wallet: Amounts {
                salvage: 500,
                components: 20,
            },
            ..Campaign::default()
        };
        campaign
            .inventory
            .purchase(kind, &mut campaign.wallet)
            .unwrap();
        campaign.inventory.assign(0, Some(kind)).unwrap();
        campaign
    }

    #[test]
    fn catalog_detail_reads_the_current_launch_module_tuning() {
        let config = ModuleConfig {
            drains: [12.5, 8., 8., 10.],
            overdrive_multiplier: 1.5,
            ..ModuleConfig::default()
        };

        let detail = catalog_detail(ModuleKind::Overdrive, &config);

        assert!(detail.contains("1.5"));
        assert!(detail.contains("12.5/s"));
        assert!(detail.contains("Tradeoff"));
    }

    #[test]
    fn potential_power_uses_pristine_baseline_plus_passives_and_equipped_modules() {
        let mut campaign = campaign_with(ModuleKind::Overdrive);
        campaign
            .passives
            .purchase(NodeId::Battery, &mut campaign.wallet)
            .unwrap();
        campaign
            .passives
            .purchase(NodeId::Reserve, &mut campaign.wallet)
            .unwrap();
        campaign
            .passives
            .purchase(NodeId::Charging, &mut campaign.wallet)
            .unwrap();
        let baseline = Baseline::default();

        let preview = potential_power(&baseline, &campaign, &ChargerConfig::default());

        assert_eq!(preview.battery_capacity, 120.);
        assert_eq!(preview.potential_drain, 10.);
        assert_eq!(preview.charger_supply, 30.);
        assert_eq!(preview.net_rate, 20.);
        assert!(preview.display().contains("ALL MODULES START OFF"));
        assert!(preview.display().contains("RESERVE -"));
    }

    #[test]
    fn slot_lines_show_empty_slots_and_real_equipped_effects() {
        let campaign = campaign_with(ModuleKind::Rocket);
        let lines = loadout_lines(campaign.inventory.loadout(), &ModuleConfig::default());

        assert!(lines.contains("SLOT 1  ROCKETS  20 dmg / 10/s"));
        assert!(lines.contains("SLOT 2  EMPTY"));
        assert!(lines.contains("SLOT 4  EMPTY"));
    }
}
