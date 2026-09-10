//! Slow, explicitly requested balance comparison. Run only with
//! `cargo test --locked camping_balance_probe -- --ignored --nocapture`.
use super::super::waves::SpawnCounts;
use super::*;
use crate::{
    arena::DroneFlight,
    energy::{ChargerConfig, ChargerReserve, ChargingNode, Energy},
    modules::{ModuleKind, Modules},
    upgrades::{ExperienceConfig, UpgradeKind, UpgradeRun, runtime::UpgradePlugin},
    world::{PlayerPath, WorldGeometry},
};

const DT: f32 = 1. / 30.;
const ACTIVE_LIMIT: f64 = 310.;
const FRAME_LIMIT: usize = 30 * 310 + 256;
const OFFENSE_FIRST: [UpgradeKind; 6] = [
    UpgradeKind::HeavyRounds,
    UpgradeKind::HeavyArmor,
    UpgradeKind::RapidShield,
    UpgradeKind::Interceptor,
    UpgradeKind::AgileFrame,
    UpgradeKind::WideAreaRockets,
];
const SHIELD_FIRST: [UpgradeKind; 6] = [
    UpgradeKind::HeavyRounds,
    UpgradeKind::RapidShield,
    UpgradeKind::HeavyArmor,
    UpgradeKind::AgileFrame,
    UpgradeKind::Interceptor,
    UpgradeKind::WideAreaRockets,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Balance {
    Baseline,
    Tuned,
    Depleting,
}

impl Balance {
    fn label(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Tuned => "tuned-unlimited",
            Self::Depleting => "tuned-depleting",
        }
    }

    fn expected_requests(self) -> usize {
        match self {
            Self::Baseline => 375,
            Self::Tuned | Self::Depleting => 447,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tactic {
    Camp { x: f32, shield: bool },
    Moving,
    Relay,
}

impl Tactic {
    fn label(self) -> &'static str {
        match self {
            Self::Camp {
                x: -280.,
                shield: false,
            } => "camp-left-overdrive",
            Self::Camp {
                x: 280.,
                shield: false,
            } => "camp-right-overdrive",
            Self::Camp {
                x: -280.,
                shield: true,
            } => "camp-left-overdrive-shield",
            Self::Camp {
                x: 280.,
                shield: true,
            } => "camp-right-overdrive-shield",
            Self::Camp { .. } => "camp",
            Self::Moving => "moving-keyboard-pilot",
            Self::Relay => "alternating-conservation-pilot",
        }
    }

    fn priorities(self) -> &'static [UpgradeKind] {
        match self {
            Self::Camp { shield: true, .. } => &SHIELD_FIRST,
            Self::Camp { .. } | Self::Moving | Self::Relay => &OFFENSE_FIRST,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ChoiceRecord {
    at: f64,
    level: u32,
    selected: UpgradeKind,
}

#[derive(Clone, Debug)]
struct ResultRow {
    balance: Balance,
    tactic: Tactic,
    phase: GamePhase,
    active_seconds: f64,
    hull: u32,
    kills: u32,
    choices: Vec<ChoiceRecord>,
    peak_enemies: usize,
    peak_warnings: usize,
    spawns: SpawnCounts,
    final_energy: f64,
    level: u32,
    xp: u32,
    upgrade_pending: u32,
    pickup_collected: bool,
    earned_kill_xp: u64,
    supplied: [f64; 2],
    final_reserves: [f64; 2],
    powered_seconds: f64,
    visits: [u32; 2],
    relay_stops: Vec<(f64, usize)>,
}

fn tick(app: &mut App, dt: f32, keys: &[KeyCode]) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    for &key in keys {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }
    *app.world_mut()
        .resource_mut::<bevy::time::TimeUpdateStrategy>() =
        bevy::time::TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(dt));
    app.update();
}

fn count_warnings(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<SpawnWarning>>()
        .iter(app.world())
        .count()
}

fn authored_bursts(balance: Balance) -> Vec<(f64, usize)> {
    if balance != Balance::Baseline {
        return WaveConfig::default().bursts;
    }
    // Freeze the previous 375-enemy schedule independently of production defaults.
    [
        (3, 24, 10, 3),
        (30, 99, 8, 3),
        (105, 159, 8, 6),
        (165, 219, 6, 9),
        (225, 294, 4, 12),
    ]
    .into_iter()
    .flat_map(|(start, end, interval, size)| {
        (start..end)
            .step_by(interval)
            .map(move |at| (at as f64, size))
    })
    .collect()
}

fn total_earned_xp(run: &UpgradeRun) -> u64 {
    let crossed = u64::from(run.level - 1);
    25 * crossed * (crossed + 3) / 2 + u64::from(run.xp)
}

fn desired_gameplay_keys(app: &mut App, drone: Entity, tactic: Tactic) -> Vec<KeyCode> {
    let mut keys = Vec::new();
    let modules = app.world().resource::<Modules>();
    if !modules.active(ModuleKind::Overdrive) {
        keys.push(KeyCode::Digit1);
    }
    if matches!(tactic, Tactic::Camp { shield: true, .. }) && !modules.active(ModuleKind::Shield) {
        keys.push(KeyCode::Digit2);
    }
    if tactic == Tactic::Moving {
        let elapsed = app.world().resource::<Encounter>().elapsed;
        let threats: Vec<_> = app
            .world_mut()
            .query_filtered::<(&Transform, &DroneFlight), With<Enemy>>()
            .iter(app.world())
            .map(|(transform, flight)| (transform.translation, flight.velocity))
            .collect();
        keys.extend(super::super::validation::pilot_keys(
            elapsed,
            position(app, drone),
            app.world().get::<DroneFlight>(drone).unwrap(),
            &threats,
        ));
    }
    keys
}

fn choice_key(run: &UpgradeRun, priorities: &[UpgradeKind]) -> KeyCode {
    let index = priorities
        .iter()
        .find_map(|wanted| run.offer.iter().position(|offered| offered == wanted))
        .unwrap_or(0);
    [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3][index]
}

fn run_case(balance: Balance, tactic: Tactic) -> ResultRow {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<WorldGeometry>()
        .init_resource::<PlayerPath>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::ZERO,
        ))
        .add_plugins((
            bevy::time::TimePlugin,
            ArenaPlugin,
            CombatPlugin,
            UpgradePlugin,
        ));
    // Historical wave/XP diagnostics retain effectively unlimited chargers.
    // This fixture changes reserve capacity only, never player health or XP.
    if balance != Balance::Depleting {
        app.insert_resource(ChargerConfig {
            capacity: 1_000_000.,
            ..default()
        });
    }
    app.update();

    let defaults = WaveConfig::default();
    assert_eq!(
        (
            defaults.duration,
            defaults.cap,
            defaults.warning_seconds,
            defaults.clearance,
        ),
        (300., 30, 0.75, 120.)
    );
    let bursts = authored_bursts(balance);
    assert_eq!(
        bursts.iter().map(|(_, count)| count).sum::<usize>(),
        balance.expected_requests()
    );
    app.world_mut().resource_mut::<WaveConfig>().bursts = bursts;
    if balance == Balance::Baseline {
        let mut xp = app.world_mut().resource_mut::<ExperienceConfig>();
        xp.kill_xp = 7;
        xp.early_kill_xp = 10;
        xp.reduction_starts_at = 105.;
    }

    let drone = app
        .world_mut()
        .query_filtered::<Entity, With<Drone>>()
        .single(app.world())
        .unwrap();
    let staged_position = match tactic {
        Tactic::Camp { x, .. } => {
            let point = Vec3::new(x, 90., 0.);
            // The sole camping fixture mutation: stage on the authored charger once.
            app.world_mut()
                .get_mut::<Transform>(drone)
                .unwrap()
                .translation = point;
            Some(point)
        }
        Tactic::Moving | Tactic::Relay => None,
    };

    let mut relay = RelayPilot::default();
    let initial_keys = if tactic == Tactic::Relay {
        relay.keys(&mut app, drone)
    } else {
        desired_gameplay_keys(&mut app, drone, tactic)
    };
    tick(&mut app, 0., &initial_keys);
    if staged_position.is_some() {
        assert!(
            app.world().resource::<Energy>().charging.is_some(),
            "camping fixture did not begin on a real charging node"
        );
        assert!(
            app.world()
                .resource::<Modules>()
                .active(ModuleKind::Overdrive)
        );
        if matches!(tactic, Tactic::Camp { shield: true, .. }) {
            assert!(app.world().resource::<Modules>().active(ModuleKind::Shield));
        }
    }

    let mut choices = Vec::new();
    let mut released_for_choice = false;
    let mut peak_enemies = count::<Enemy>(&mut app);
    let mut peak_warnings = count_warnings(&mut app);
    let mut previous_kills = app.world().resource::<Encounter>().kills;
    let mut earned_kill_xp = 0_u64;
    let mut supplied = [0.; 2];
    let mut powered_seconds = 0.;
    let mut visits = [0; 2];
    let mut occupied_before = [false; 2];
    for _ in 0..FRAME_LIMIT {
        let phase = *app.world().resource::<GamePhase>();
        if matches!(phase, GamePhase::Dead | GamePhase::Survived) {
            break;
        }

        let mut expected_choice = None;
        let keys = if phase == GamePhase::Choosing {
            let run = app.world().resource::<UpgradeRun>();
            if run.offer.is_empty() {
                released_for_choice = false;
                Vec::new()
            } else if !released_for_choice {
                released_for_choice = true;
                Vec::new()
            } else {
                let key = choice_key(run, tactic.priorities());
                let index = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3]
                    .iter()
                    .position(|candidate| *candidate == key)
                    .unwrap();
                expected_choice = Some(ChoiceRecord {
                    at: app.world().resource::<Encounter>().elapsed,
                    level: run.level,
                    selected: run.offer[index],
                });
                released_for_choice = false;
                vec![key]
            }
        } else {
            released_for_choice = false;
            if tactic == Tactic::Relay {
                relay.keys(&mut app, drone)
            } else {
                desired_gameplay_keys(&mut app, drone, tactic)
            }
        };

        let before_selected = app.world().resource::<UpgradeRun>().selected.len();
        let reserves_before = reserve_snapshot(&mut app);
        let powered = app
            .world()
            .resource::<Modules>()
            .active(ModuleKind::Overdrive);
        tick(&mut app, DT, &keys);
        if phase == GamePhase::Playing {
            let reserves_after = reserve_snapshot(&mut app);
            for side in 0..2 {
                supplied[side] += (reserves_before[side].0 - reserves_after[side].0).max(0.);
                if reserves_after[side].1 && !occupied_before[side] {
                    visits[side] += 1;
                }
                occupied_before[side] = reserves_after[side].1;
            }
            // Frame-sampled powered time, with at most one frame error per toggle.
            if powered {
                powered_seconds += f64::from(DT);
            }
        }
        if let Some(record) = expected_choice
            && app.world().resource::<UpgradeRun>().selected.len() == before_selected + 1
        {
            choices.push(record);
        }
        let encounter = app.world().resource::<Encounter>();
        let new_kills = encounter.kills - previous_kills;
        earned_kill_xp += u64::from(new_kills)
            * u64::from(
                app.world()
                    .resource::<ExperienceConfig>()
                    .kill_xp_at(encounter.elapsed),
            );
        previous_kills = encounter.kills;
        peak_enemies = peak_enemies.max(count::<Enemy>(&mut app));
        peak_warnings = peak_warnings.max(count_warnings(&mut app));
        assert!(app.world().resource::<Encounter>().elapsed <= ACTIVE_LIMIT + f64::EPSILON);
    }

    let phase = *app.world().resource::<GamePhase>();
    let encounter = app.world().resource::<Encounter>();
    assert!(
        matches!(phase, GamePhase::Dead | GamePhase::Survived),
        "{} {} did not finish within {ACTIVE_LIMIT} active seconds",
        balance.label(),
        tactic.label()
    );
    assert!(encounter.elapsed <= ACTIVE_LIMIT);
    let spawns = encounter.spawns;
    let warning_count = count_warnings(&mut app);
    assert_eq!(
        spawns.requested,
        spawns.admitted
            + spawns.rejected_cap
            + spawns.rejected_space
            + spawns.skipped_hitch
            + spawns.skipped_terminal
    );
    assert_eq!(
        spawns.admitted,
        spawns.activated + spawns.cancelled + warning_count
    );
    let final_reserves = reserve_snapshot(&mut app).map(|(remaining, _)| remaining);
    let encounter = app.world().resource::<Encounter>();
    let run = app.world().resource::<UpgradeRun>();
    let pickup = app
        .world()
        .resource::<crate::upgrades::runtime::ExplorationPickup>();
    assert_eq!(
        total_earned_xp(run),
        earned_kill_xp + if pickup.collected { 30 } else { 0 }
    );
    assert_eq!(
        choices
            .iter()
            .map(|choice| choice.selected)
            .collect::<Vec<_>>(),
        run.selected
    );
    assert!(choices.iter().all(|choice| {
        choice.at <= encounter.elapsed && choice.level >= 2 && choice.level <= run.level
    }));
    if let Some(staged) = staged_position {
        assert!(
            position(&app, drone).distance(staged) < 0.001,
            "camping fixture moved without pilot input"
        );
        assert!(
            app.world()
                .get::<DroneFlight>(drone)
                .unwrap()
                .velocity
                .length()
                < 0.001,
            "camping fixture acquired velocity without pilot input"
        );
    }

    ResultRow {
        balance,
        tactic,
        phase,
        active_seconds: encounter.elapsed,
        hull: app.world().resource::<PlayerHealth>().current,
        kills: encounter.kills,
        choices,
        peak_enemies,
        peak_warnings,
        spawns,
        final_energy: app.world().resource::<Energy>().current,
        level: run.level,
        xp: run.xp,
        upgrade_pending: run.pending,
        pickup_collected: pickup.collected,
        earned_kill_xp,
        supplied,
        final_reserves,
        powered_seconds,
        visits,
        relay_stops: relay.stops,
    }
}

#[test]
#[ignore = "explicit 10-run, 30 Hz balance diagnostic"]
fn camping_balance_probe() {
    let tactics = [
        Tactic::Camp {
            x: -280.,
            shield: false,
        },
        Tactic::Camp {
            x: 280.,
            shield: false,
        },
        Tactic::Camp {
            x: -280.,
            shield: true,
        },
        Tactic::Camp {
            x: 280.,
            shield: true,
        },
        Tactic::Moving,
    ];
    for balance in [Balance::Baseline, Balance::Tuned] {
        for tactic in tactics {
            let row = run_case(balance, tactic);
            println!(
                "CAMPING PROBE balance={} tactic={} outcome={:?} active_seconds={:.3} hull={} kills={} earned_kill_xp={} choices={:?} peak_enemies={} peak_warnings={} spawns={:?} energy={:.3} level={} xp={} upgrade_pending={} pickup={}",
                row.balance.label(),
                row.tactic.label(),
                row.phase,
                row.active_seconds,
                row.hull,
                row.kills,
                row.earned_kill_xp,
                row.choices,
                row.peak_enemies,
                row.peak_warnings,
                row.spawns,
                row.final_energy,
                row.level,
                row.xp,
                row.upgrade_pending,
                row.pickup_collected,
            );
        }
    }
}

/// Read actual per-node reserve changes; recovery is excluded from delivered totals.
fn reserve_snapshot(app: &mut App) -> [(f64, bool); 2] {
    let mut result = [(0., false); 2];
    for (node, reserve) in app
        .world_mut()
        .query::<(&ChargingNode, &ChargerReserve)>()
        .iter(app.world())
    {
        result[usize::from(node.center.x > 0.)] = (reserve.remaining, reserve.occupied);
    }
    result
}

#[derive(Default)]
struct RelayPilot {
    waypoint: usize,
    charging_since: Option<f64>,
    stops: Vec<(f64, usize)>,
}
impl RelayPilot {
    fn keys(&mut self, app: &mut App, drone: Entity) -> Vec<KeyCode> {
        // Follow the authored empty-battery detour in both directions. Dwell only
        // to refill, leaving at 90% battery, reserve exhaustion or eight seconds.
        let route = [
            Vec3::new(-280., 150., 0.),
            Vec3::new(-280., 150., 195.),
            Vec3::new(210., 150., 195.),
            Vec3::new(280., 150., 0.),
            Vec3::new(210., 150., 195.),
            Vec3::new(-280., 150., 195.),
        ];
        let point = position(app, drone);
        let flight = app.world().get::<DroneFlight>(drone).unwrap();
        let elapsed = app.world().resource::<Encounter>().elapsed;
        let energy = app.world().resource::<Energy>();
        let capacity = app
            .world()
            .resource::<crate::energy::EnergyConfig>()
            .capacity;
        let at_stop = matches!(self.waypoint, 0 | 3);
        if point.distance(route[self.waypoint]) < 30. && flight.velocity.length() < 45. {
            if at_stop {
                if self.charging_since.is_none() {
                    self.stops.push((elapsed, usize::from(self.waypoint == 3)));
                }
                let since = *self.charging_since.get_or_insert(elapsed);
                if energy.current >= capacity * 0.9
                    || energy.charging.is_none()
                    || elapsed - since >= 8.
                {
                    self.waypoint = (self.waypoint + 1) % route.len();
                    self.charging_since = None;
                }
            } else {
                self.waypoint = (self.waypoint + 1) % route.len();
            }
        }
        let mut keys = super::super::validation::routes::keys(point, flight, route[self.waypoint]);
        let modules = app.world().resource::<Modules>();
        // Conserve while charging and below half a battery; use overdrive for
        // travel once charged. This is a repeatable policy, not a survival claim.
        let want = !matches!(self.waypoint, 0 | 3) && energy.current > capacity * 0.5;
        if modules.active(ModuleKind::Overdrive) != want {
            keys.push(KeyCode::Digit1);
        }
        keys
    }
}

#[test]
#[ignore = "explicit 10-run charger-only comparison"]
fn charger_depletion_probe() {
    let mut relay_control: Option<ResultRow> = None;
    for balance in [Balance::Tuned, Balance::Depleting] {
        for tactic in [
            Tactic::Camp {
                x: -280.,
                shield: false,
            },
            Tactic::Camp {
                x: 280.,
                shield: false,
            },
            Tactic::Camp {
                x: -280.,
                shield: true,
            },
            Tactic::Camp {
                x: 280.,
                shield: true,
            },
            Tactic::Relay,
        ] {
            let row = run_case(balance, tactic);
            println!(
                "CHARGER PROBE balance={} tactic={} outcome={:?} active={:.3} hull={} kills={} supplied={:?} overdrive_seconds={:.3} visits={:?} stops={:?} choices={:?} energy={:.3} peak={} spawns={:?}",
                row.balance.label(),
                row.tactic.label(),
                row.phase,
                row.active_seconds,
                row.hull,
                row.kills,
                row.supplied,
                row.powered_seconds,
                row.visits,
                row.relay_stops,
                row.choices,
                row.final_energy,
                row.peak_enemies,
                row.spawns
            );
            if let Tactic::Camp { x, shield } = tactic
                && balance == Balance::Depleting
            {
                let side = usize::from(x > 0.);
                assert_eq!(
                    row.phase,
                    GamePhase::Dead,
                    "finite camp must die: {tactic:?}"
                );
                assert!(row.active_seconds < WaveConfig::default().duration);
                assert_eq!(row.hull, 0);
                assert_eq!(row.final_energy, 0.);
                assert_eq!(row.final_reserves[side], 0., "occupied field must deplete");
                assert!(
                    (row.supplied[side] - 200.).abs() < 1e-5,
                    "field must deliver its full reserve"
                );
                assert_eq!(
                    row.supplied[1 - side],
                    0.,
                    "camp must not use the other field"
                );
                assert_eq!(row.final_reserves[1 - side], 200.);
                let expected_powered = if shield { 300. / 18. } else { 30. };
                assert!(
                    (row.powered_seconds - expected_powered).abs() <= f64::from(DT) * 2.,
                    "unexpected powered time: {}",
                    row.powered_seconds
                );
            } else {
                assert_eq!(
                    row.phase,
                    GamePhase::Survived,
                    "control camps and relay must survive: {balance:?} {tactic:?}"
                );
                assert!((row.active_seconds - WaveConfig::default().duration).abs() < 1e-5);
                assert!(row.hull > 0);
            }
            if tactic == Tactic::Relay {
                assert!(
                    row.visits.iter().all(|visits| *visits > 0),
                    "relay must visit both fields"
                );
                if balance == Balance::Tuned {
                    relay_control = Some(row);
                } else {
                    let control = relay_control
                        .as_ref()
                        .expect("unlimited relay must run first");
                    assert_eq!(row.phase, control.phase);
                    assert_eq!(row.hull, control.hull, "finite reserves changed relay hull");
                    assert_eq!(
                        row.kills, control.kills,
                        "finite reserves changed relay kills"
                    );
                    assert_eq!(
                        row.choices, control.choices,
                        "finite reserves changed earned choices"
                    );
                    assert_eq!(row.visits, control.visits);
                    assert_eq!(row.relay_stops, control.relay_stops);
                    assert_eq!(
                        (row.level, row.xp, row.upgrade_pending, row.pickup_collected),
                        (
                            control.level,
                            control.xp,
                            control.upgrade_pending,
                            control.pickup_collected
                        )
                    );
                    for (label, actual, expected) in [
                        ("active seconds", row.active_seconds, control.active_seconds),
                        ("battery", row.final_energy, control.final_energy),
                        (
                            "powered seconds",
                            row.powered_seconds,
                            control.powered_seconds,
                        ),
                        ("left supply", row.supplied[0], control.supplied[0]),
                        ("right supply", row.supplied[1], control.supplied[1]),
                    ] {
                        assert!(
                            (actual - expected).abs() < 1e-5,
                            "finite reserves changed relay {label}: {actual} != {expected}"
                        );
                    }
                }
            }
        }
    }
}
