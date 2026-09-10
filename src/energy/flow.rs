use super::ChargerReserve;
use bevy::prelude::Entity;

pub(super) struct FlowResult {
    pub battery: f64,
    pub powered_seconds: f64,
    pub depleted_modules: bool,
    pub charging: Option<Entity>,
}

pub(super) struct FlowConfig {
    pub battery_capacity: f64,
    pub delivery_rate: f64,
    pub charger_capacity: f64,
    pub recovery_delay: f64,
    pub recovery_rate: f64,
}

/// Advances one power interval while conserving energy across battery and
/// charger boundaries. Chargers must already be sorted into source priority.
pub(super) fn advance(
    dt: f64,
    battery: f64,
    module_drain: f64,
    modules_enabled: bool,
    config: &FlowConfig,
    chargers: &mut [(Entity, ChargerReserve)],
) -> FlowResult {
    let dt = dt.max(0.);
    let battery_capacity = config.battery_capacity.max(0.);
    let delivery_rate = config.delivery_rate.max(0.);
    let module_drain = module_drain.max(0.);
    let charger_capacity = config.charger_capacity.max(0.);
    let recovery_delay = config.recovery_delay.max(0.);
    let recovery_rate = config.recovery_rate.max(0.);

    for (_, reserve) in chargers.iter_mut() {
        reserve.remaining = reserve.remaining.max(0.);
        if reserve.occupied {
            reserve.away_seconds = 0.;
        } else {
            let prior_away = reserve.away_seconds.max(0.);
            let next_away = prior_away + dt;
            let recovery_seconds =
                (next_away - recovery_delay).max(0.) - (prior_away - recovery_delay).max(0.);
            reserve.remaining =
                (reserve.remaining + recovery_rate * recovery_seconds).min(charger_capacity);
            reserve.away_seconds = next_away;
        }
    }

    let mut battery = battery.clamp(0., battery_capacity);
    let mut elapsed = 0.;
    let mut powered_seconds = 0.;
    let mut modules_powered = modules_enabled;
    let mut depleted_modules = false;

    while elapsed < dt {
        let source = chargers
            .iter()
            .position(|(_, reserve)| reserve.occupied && reserve.remaining > 0.);
        let demand = if modules_powered { module_drain } else { 0. };
        let potential_supply = source.map_or(0., |_| delivery_rate);

        if modules_powered && battery <= 0. && potential_supply < demand {
            modules_powered = false;
            depleted_modules = true;
            continue;
        }

        let supply = if battery >= battery_capacity {
            potential_supply.min(demand)
        } else {
            potential_supply
        };
        let battery_rate = supply - demand;
        let mut span = dt - elapsed;

        if let Some(index) = source
            && supply > 0.
        {
            span = span.min(chargers[index].1.remaining / supply);
        }
        if battery_rate > 0. && battery < battery_capacity {
            span = span.min((battery_capacity - battery) / battery_rate);
        } else if battery_rate < 0. && battery > 0. {
            span = span.min(battery / -battery_rate);
        }

        if span <= 0. {
            break;
        }
        if modules_powered {
            powered_seconds += span;
        }
        if let Some(index) = source {
            chargers[index].1.remaining = (chargers[index].1.remaining - supply * span).max(0.);
        }
        battery = (battery + battery_rate * span).clamp(0., battery_capacity);
        elapsed += span;

        if modules_powered && battery <= 0. && battery_rate < 0. {
            modules_powered = false;
            depleted_modules = true;
        }
    }

    let charging = chargers
        .iter()
        .find(|(_, reserve)| reserve.occupied && reserve.remaining > 0.)
        .map(|(entity, _)| *entity);
    FlowResult {
        battery,
        powered_seconds,
        depleted_modules,
        charging,
    }
}
