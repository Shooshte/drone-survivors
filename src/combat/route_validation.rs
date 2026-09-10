//! Repeatable keyboard-only terrain fixture; no waves, health overrides or module use.
use super::*;
use crate::world::hazard::{HazardPhase, HazardState};

#[derive(Resource, Default)]
pub(super) struct RouteProbe {
    pub leg: usize,
    waypoint: usize,
    started: Option<f64>,
    pub results: Vec<f64>,
    committed: bool,
    waiting: f64,
    last_update: f64,
}
impl RouteProbe {
    fn target(
        &mut self,
        position: Vec3,
        velocity: Vec3,
        elapsed: f64,
        hazard: &HazardState,
    ) -> Vec3 {
        let left = Vec3::new(-280., 150., 0.);
        let right = Vec3::new(280., 150., 0.);
        let arrived = |point: Vec3| position.distance(point) < 18. && velocity.length() < 35.;
        let dt = (elapsed - self.last_update).max(0.);
        self.last_update = elapsed;
        if self.leg >= 4 {
            return left;
        }
        if self.started.is_none() {
            if !arrived(left) {
                return left;
            }
            self.started = Some(elapsed);
        }
        let points: &[Vec3] = match self.leg {
            0 => &[Vec3::new(0., 150., 0.), right],
            1 => &[Vec3::new(240., 150., 0.), left],
            2 => &[
                Vec3::new(0., 150., 195.),
                Vec3::new(210., 150., 195.),
                right,
            ],
            _ => &[Vec3::new(210., 150., 195.), Vec3::new(0., 150., 195.), left],
        };
        if self.leg < 2 && self.waypoint == 1 && !self.committed {
            if hazard.phase != HazardPhase::Inactive || hazard.remaining() < 2.5 {
                self.waiting += dt;
                return points[0];
            }
            self.committed = true;
        }
        let goal = points[self.waypoint];
        if arrived(goal) {
            self.waypoint += 1;
            if self.waypoint == points.len() {
                let seconds = elapsed - self.started.unwrap();
                println!(
                    "ROUTE leg={} seconds={seconds:.3} waiting={:.3} flight={:.3}",
                    self.leg,
                    self.waiting,
                    seconds - self.waiting
                );
                self.results.push(seconds);
                self.leg += 1;
                self.waypoint = 0;
                self.committed = false;
                self.waiting = 0.;
                self.started = Some(elapsed);
            }
        }
        goal
    }
}

pub(crate) fn keys(position: Vec3, flight: &DroneFlight, target: Vec3) -> Vec<KeyCode> {
    let delta = target - position;
    let horizontal = delta.with_y(0.);
    let distance = horizontal.length();
    let desired = horizontal.normalize_or_zero()
        * (distance * 1.5).min((200. * distance).sqrt()).min(180.)
        + Vec3::Y * (delta.y * 2.).clamp(-40., 40.);
    super::steering_keys(
        (desired - flight.velocity) * 3. + flight.velocity * Vec3::new(0.25, 0.5, 0.25),
        flight,
        position,
        &[],
    )
}

pub(super) fn input(
    run: Res<Encounter>,
    drone: Single<(&Transform, &DroneFlight), With<Drone>>,
    hazard: Res<HazardState>,
    mut probe: ResMut<RouteProbe>,
    mut input: ResMut<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        *probe = RouteProbe::default();
        return;
    }
    let target = probe.target(drone.0.translation, drone.1.velocity, run.elapsed, &hazard);
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::KeyQ,
        KeyCode::KeyE,
        KeyCode::Space,
        KeyCode::ShiftLeft,
    ] {
        input.reset(key);
    }
    for key in keys(drone.0.translation, drone.1, target) {
        input.press(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyboard_pilot_completes_both_routes_both_directions_without_hazard_damage() {
        for rate in [30, 60, 144] {
            for offset in [0., 1.5] {
                let mut app = App::new();
                app.init_resource::<Time>()
                    .init_resource::<ButtonInput<KeyCode>>()
                    .init_resource::<crate::world::WorldGeometry>()
                    .init_resource::<crate::world::PlayerPath>()
                    .add_message::<AppExit>()
                    .add_plugins((crate::arena::ArenaPlugin, CombatPlugin));
                install(
                    &mut app,
                    ValidationConfig {
                        mode: ValidationMode::Routes,
                        seconds: 90.,
                        enemies: 150,
                    },
                );
                app.update();
                // Isolate depleted-flight viability: this fixture has no recharge or powered modules.
                app.world_mut()
                    .resource_mut::<crate::energy::Energy>()
                    .current = 0.;
                app.world_mut()
                    .resource_mut::<crate::energy::EnergyConfig>()
                    .recharge = 0.;
                app.world_mut().resource_mut::<HazardState>().elapsed = offset;
                for _ in 0..90 * rate {
                    app.world_mut()
                        .resource_mut::<Time>()
                        .advance_by(std::time::Duration::from_secs_f32(1. / rate as f32));
                    app.update();
                    if app.world().resource::<RouteProbe>().results.len() == 4 {
                        break;
                    }
                }
                let probe = app.world().resource::<RouteProbe>();
                println!(
                    "ROUTE TIMES rate={rate} phase_offset={offset} {:?}",
                    probe.results
                );
                assert_eq!(
                    probe.results.len(),
                    4,
                    "route probe stalled on leg {} waypoint {}",
                    probe.leg,
                    probe.waypoint
                );
                assert_eq!(app.world().resource::<PlayerHealth>().current, 100);
                assert_eq!(app.world().resource::<crate::energy::Energy>().current, 0.);
                assert!(
                    app.world()
                        .resource::<crate::modules::Modules>()
                        .enabled
                        .iter()
                        .all(|&on| !on)
                );
            }
        }
    }
}
