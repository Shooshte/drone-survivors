use super::{
    ChargerReserve,
    flow::{FlowConfig, advance},
};
use bevy::prelude::*;
fn config() -> FlowConfig {
    FlowConfig {
        battery_capacity: 200.,
        delivery_rate: 25.,
        charger_capacity: 200.,
        recovery_delay: 8.,
        recovery_rate: 10.,
        reserve_cost: 0.75,
    }
}
fn source(reserve: f64) -> (Entity, ChargerReserve) {
    (
        Entity::PLACEHOLDER,
        ChargerReserve {
            remaining: reserve,
            occupied: true,
            away_seconds: 0.,
        },
    )
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-6, "{a} != {b}");
}
#[test]
fn efficient_delivery_costs_less_reserve_and_stops_exactly_at_empty() {
    let mut chargers = [source(200.)];
    let result = advance(1., 0., 0., false, &config(), &mut chargers);
    near(result.battery, 25.);
    near(chargers[0].1.remaining, 181.25);
    let mut chargers = [source(15.)];
    let result = advance(2., 0., 0., false, &config(), &mut chargers);
    near(result.battery, 20.);
    near(chargers[0].1.remaining, 0.);
    assert!(result.charging.is_none());
}
#[test]
fn full_battery_only_debits_delivered_module_power() {
    let mut chargers = [source(200.)];
    let result = advance(2., 200., 10., true, &config(), &mut chargers);
    near(result.battery, 200.);
    near(chargers[0].1.remaining, 185.);
    near(result.powered_seconds, 2.);
    let result = advance(2., 200., 0., false, &config(), &mut chargers);
    near(result.battery, 200.);
    near(chargers[0].1.remaining, 185.);
}
fn simulate(dt: f64) -> (f64, f64, f64) {
    let mut chargers = [source(15.), source(22.5)];
    let mut battery = 10.;
    let mut powered = 0.;
    let mut enabled = true;
    for _ in 0..(4. / dt).round() as usize {
        let result = advance(dt, battery, 36., enabled, &config(), &mut chargers);
        battery = result.battery;
        powered += result.powered_seconds;
        if result.depleted_modules {
            enabled = false;
        }
    }
    (
        battery,
        chargers.iter().map(|(_, r)| r.remaining).sum(),
        powered,
    )
}
#[test]
fn efficiency_matches_split_frames_across_overlapping_sources_and_power_loss() {
    let long = simulate(4.);
    near(long.1, 0.);
    for dt in [1. / 30., 1. / 60., 1. / 120.] {
        let split = simulate(dt);
        near(split.0, long.0);
        near(split.1, long.1);
        near(split.2, long.2);
    }
}
