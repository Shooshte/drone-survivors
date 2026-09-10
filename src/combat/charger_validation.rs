//! Opt-in charger UI fixture: actual keyboard flight, no waves or synthetic XP.
use super::*;
use crate::energy::{ChargerReserve, ChargingNode};
use crate::modules::{ModuleKind, Modules};

#[derive(Resource, Default)]
pub(super) struct ChargerPreview {
    waypoint: usize,
    depleted_at: Option<f64>,
}

pub(super) fn input(
    run: Res<Encounter>,
    phase: Res<GamePhase>,
    drone: Single<(&Transform, &DroneFlight), With<Drone>>,
    nodes: Query<(&ChargingNode, &ChargerReserve)>,
    modules: Res<Modules>,
    mut probe: ResMut<ChargerPreview>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        *probe = default();
        return;
    }
    if *phase != GamePhase::Playing {
        return;
    }
    let route = [
        Vec3::new(-280., 90., 0.),
        Vec3::new(-280., 150., 195.),
        Vec3::new(210., 150., 195.),
        Vec3::new(280., 90., 0.),
    ];
    if probe.waypoint == 0 {
        if nodes
            .iter()
            .any(|(node, state)| node.center.x < 0. && state.remaining == 0.)
        {
            let since = *probe.depleted_at.get_or_insert(run.elapsed);
            if run.elapsed - since >= 2. {
                println!(
                    "CHARGER PREVIEW left depleted at {since:.3}; departing at {:.3}",
                    run.elapsed
                );
                probe.waypoint = 1;
            }
        }
    } else if probe.waypoint < route.len() - 1
        && drone.0.translation.distance(route[probe.waypoint]) < 25.
        && drone.1.velocity.length() < 45.
    {
        probe.waypoint += 1;
    }
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::KeyQ,
        KeyCode::KeyE,
        KeyCode::Space,
        KeyCode::ShiftLeft,
    ] {
        keys.reset(key);
    }
    for key in routes::keys(drone.0.translation, drone.1, route[probe.waypoint]) {
        keys.press(key);
    }
    // Remain powered at the first charger until it empties; conserve on the trip.
    let want_overdrive = probe.waypoint == 0;
    if modules.active(ModuleKind::Overdrive) != want_overdrive {
        keys.press(KeyCode::Digit1);
    }
}
