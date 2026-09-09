//! Explicit development validation only; normal launches install no harness systems.
use super::*;
use crate::arena::{Drone, DroneFlight};
use crate::{
    energy::{Energy, EnergyConfig},
    upgrades::{UpgradeKind, UpgradeRun},
};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::time::Instant;
#[path = "validation_capture.rs"]
mod capture;
#[path = "route_validation.rs"]
pub(super) mod routes;

const CHOICE_KEYS: [KeyCode; 4] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Backspace,
];
const MOBILE_PRIORITIES: [UpgradeKind; 3] = [
    UpgradeKind::Interceptor,
    UpgradeKind::AgileFrame,
    UpgradeKind::RapidShield,
];
const ARMORED_PRIORITIES: [UpgradeKind; 3] = [
    UpgradeKind::HeavyArmor,
    UpgradeKind::HeavyRounds,
    UpgradeKind::WideAreaRockets,
];

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub(crate) enum ValidationMode {
    Survival,
    Stress,
    Idle,
    Routes,
    Mobile,
    Armored,
    Choices,
}

#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct ValidationConfig {
    pub mode: ValidationMode,
    pub seconds: f64,
    pub enemies: usize,
}

#[derive(Clone, Debug)]
struct ChoiceSnapshot {
    pending: u32,
    selected: usize,
    run_seconds: f64,
    level: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ValidationChoiceEntry {
    pub(super) run_seconds: f64,
    pub(super) level: u32,
    pub(super) action: Option<UpgradeKind>,
}

#[derive(Resource, Default)]
pub(super) struct ValidationChoices {
    pub(super) entries: Vec<ValidationChoiceEntry>,
    signature: Option<(Vec<UpgradeKind>, u32)>,
    released: bool,
    before: Option<ChoiceSnapshot>,
    pub(super) preview_started: Option<Instant>,
    pub(super) preview_requested: bool,
}

#[derive(Debug)]
struct Summary {
    median: f64,
    p95: f64,
    p99: f64,
    hitches: usize,
}
fn summarize(samples: &[f64]) -> Option<Summary> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| sorted[((p * sorted.len() as f64).ceil() as usize).saturating_sub(1)];
    Some(Summary {
        median: percentile(0.5),
        p95: percentile(0.95),
        p99: percentile(0.99),
        hitches: samples.iter().filter(|&&ms| ms > 33.3).count(),
    })
}

pub(super) fn build_summary(
    run: Option<&UpgradeRun>,
    energy: Option<&Energy>,
    config: Option<&EnergyConfig>,
) -> String {
    let names = run
        .map(|run| {
            run.selected
                .iter()
                .map(|kind| kind.name())
                .collect::<Vec<_>>()
                .join("|")
        })
        .filter(|names| !names.is_empty())
        .unwrap_or_else(|| "none".into());
    let energy = energy.map_or(f64::NAN, |energy| energy.current);
    let capacity = config.map_or(f64::NAN, |config| config.capacity);
    format!("acquired=[{names}] energy={energy:.3} capacity={capacity:.3}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restarting_clears_the_previous_terminal_exit_deadline() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<AppExit>()
            .add_plugins((crate::arena::ArenaPlugin, CombatPlugin));
        install(
            &mut app,
            ValidationConfig {
                mode: ValidationMode::Survival,
                seconds: 185.,
                enemies: 150,
            },
        );
        app.update();
        *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
        app.update();
        app.world_mut().resource_mut::<Measurements>().terminal =
            Some(Instant::now() - std::time::Duration::from_secs(3));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyR);
        app.update();
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
        let data = app.world().resource::<Measurements>();
        assert!(!data.done);
        assert!(data.terminal.is_none());
    }

    #[test]
    fn percentiles_include_slow_tail_and_handle_empty_samples() {
        assert!(summarize(&[]).is_none());
        let mut samples = vec![10.; 98];
        samples.extend([40., 50.]);
        let summary = summarize(&samples).unwrap();
        assert_eq!(
            (summary.median, summary.p95, summary.p99, summary.hitches),
            (10., 10., 40., 2)
        );
        let one = summarize(&[20.]).unwrap();
        assert_eq!(
            (one.median, one.p95, one.p99, one.hitches),
            (20., 20., 20., 0)
        );
    }
}

impl ValidationConfig {
    pub(crate) fn parse(args: impl IntoIterator<Item = String>) -> Result<Option<Self>, String> {
        let mut args = args.into_iter();
        let mut mode = None;
        let mut seconds = None;
        let mut enemies = None;
        while let Some(flag) = args.next() {
            let value = args.next().ok_or(
                "Expected --validate survival|stress|idle|routes|mobile|armored|choices \
                 [--seconds 1..600] [--enemies 1..500 (stress only)]",
            )?;
            match flag.as_str() {
                "--validate" if mode.is_none() => {
                    mode = Some(match value.as_str() {
                        "survival" => ValidationMode::Survival,
                        "stress" => ValidationMode::Stress,
                        "idle" => ValidationMode::Idle,
                        "routes" => ValidationMode::Routes,
                        "mobile" => ValidationMode::Mobile,
                        "armored" => ValidationMode::Armored,
                        "choices" => ValidationMode::Choices,
                        _ => {
                            return Err(
                                "Validation mode must be survival, stress, idle, routes, mobile, armored, or choices"
                                    .into(),
                            );
                        }
                    })
                }
                "--seconds" if seconds.is_none() => {
                    let n: f64 = value.parse().map_err(|_| "Seconds must be a number")?;
                    if !n.is_finite() || !(1. ..=600.).contains(&n) {
                        return Err("Seconds must be finite and between 1 and 600".into());
                    }
                    seconds = Some(n);
                }
                "--enemies" if enemies.is_none() => {
                    let n: usize = value.parse().map_err(|_| "Enemies must be an integer")?;
                    if !(1..=500).contains(&n) {
                        return Err("Enemies must be between 1 and 500".into());
                    }
                    enemies = Some(n);
                }
                _ => return Err(format!("Unknown or duplicate argument: {flag}")),
            }
        }
        let Some(mode) = mode else {
            return if seconds.is_some() || enemies.is_some() {
                Err("Validation options require --validate".into())
            } else {
                Ok(None)
            };
        };
        if enemies.is_some() && mode != ValidationMode::Stress {
            return Err("--enemies requires stress mode".into());
        }
        Ok(Some(Self {
            mode,
            seconds: seconds.unwrap_or(match mode {
                ValidationMode::Stress => 30.,
                ValidationMode::Choices => 60.,
                _ => 185.,
            }),
            enemies: enemies.unwrap_or(150),
        }))
    }
}

pub(crate) fn install(app: &mut App, config: ValidationConfig) {
    capture::install(app);
    println!(
        "VALIDATION {:?}: keyboard pilot={}, stress overrides={}, warmup=5s, sample limit={}s, stress target={}, synthetic_xp={}",
        config.mode,
        config.mode != ValidationMode::Idle,
        config.mode == ValidationMode::Stress,
        config.seconds,
        config.enemies,
        if config.mode == ValidationMode::Choices {
            140
        } else {
            0
        }
    );
    if config.mode == ValidationMode::Routes {
        app.init_resource::<routes::RouteProbe>().add_systems(
            Update,
            routes::input
                .in_set(GameplaySet::Reset)
                .after(lifecycle::restart),
        );
    }
    app.insert_resource(config)
        .init_resource::<Measurements>()
        .init_resource::<ValidationChoices>()
        .add_systems(Startup, configure)
        .add_systems(
            Update,
            pilot_input
                .in_set(GameplaySet::Reset)
                .after(lifecycle::restart),
        )
        .add_systems(
            Update,
            validation_choice_input
                .in_set(GameplaySet::Reset)
                .after(pilot_input),
        )
        .add_systems(Update, record_choice.in_set(GameplaySet::Movement))
        .add_systems(
            Update,
            replenish
                .after(GameplaySet::Combat)
                .before(GameplaySet::Presentation),
        )
        .add_systems(
            Update,
            choice_preview_screenshot.in_set(GameplaySet::Presentation),
        )
        .add_systems(Update, measure.after(GameplaySet::Presentation));
}

fn configure(
    config: Res<ValidationConfig>,
    mut waves: ResMut<WaveConfig>,
    mut windows: Query<&mut Window>,
    run: Option<ResMut<UpgradeRun>>,
) {
    if config.mode == ValidationMode::Routes {
        waves.bursts.clear();
    }
    if config.mode == ValidationMode::Stress {
        waves.bursts.clear();
        waves.cap = config.enemies;
        waves.duration = config.seconds + 30.;
    }
    if config.mode == ValidationMode::Choices {
        for mut window in &mut windows {
            window.resolution.set(640., 480.);
        }
        if let Some(mut run) = run {
            run.award(140);
        }
    }
}

fn validation_choice_input(
    config: Res<ValidationConfig>,
    phase: Res<GamePhase>,
    encounter: Res<Encounter>,
    run: Option<Res<UpgradeRun>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut choices: ResMut<ValidationChoices>,
) {
    let Some(run) = run else {
        return;
    };
    if *phase == GamePhase::Choosing && !run.offer.is_empty() {
        choices.before = Some(ChoiceSnapshot {
            pending: run.pending,
            selected: run.selected.len(),
            run_seconds: encounter.elapsed,
            level: run.level,
        });
    } else {
        choices.before = None;
    }

    if config.mode == ValidationMode::Choices {
        return;
    }
    for key in CHOICE_KEYS {
        keys.reset(key);
    }
    if *phase != GamePhase::Choosing || run.offer.is_empty() {
        choices.signature = None;
        choices.released = false;
        return;
    }

    let signature = (run.offer.clone(), run.pending);
    if choices.signature.as_ref() != Some(&signature) {
        choices.signature = Some(signature);
        choices.released = false;
    }
    if !choices.released {
        choices.released = true;
        return;
    }

    let priorities = match config.mode {
        ValidationMode::Mobile => &MOBILE_PRIORITIES[..],
        ValidationMode::Armored => &ARMORED_PRIORITIES[..],
        ValidationMode::Survival
        | ValidationMode::Stress
        | ValidationMode::Idle
        | ValidationMode::Routes => &[],
        ValidationMode::Choices => unreachable!(),
    };
    let key = priorities
        .iter()
        .find_map(|kind| run.offer.iter().position(|offered| offered == kind))
        .and_then(|index| CHOICE_KEYS.get(index).copied())
        .unwrap_or(KeyCode::Backspace);
    keys.press(key);
}

fn record_choice(run: Option<Res<UpgradeRun>>, mut choices: ResMut<ValidationChoices>) {
    let Some(run) = run else {
        return;
    };
    let Some(before) = choices.before.take() else {
        return;
    };
    if run.pending >= before.pending {
        return;
    }
    let action = (run.selected.len() > before.selected)
        .then(|| run.selected.last().copied())
        .flatten();
    let label = action.map_or("Skip", UpgradeKind::name);
    println!(
        "VALIDATION CHOICE run_seconds={:.3} level={} action={label}",
        before.run_seconds, before.level
    );
    choices.entries.push(ValidationChoiceEntry {
        run_seconds: before.run_seconds,
        level: before.level,
        action,
    });
}

fn choice_preview_screenshot(
    mut commands: Commands,
    config: Res<ValidationConfig>,
    phase: Res<GamePhase>,
    mut choices: ResMut<ValidationChoices>,
) {
    if config.mode != ValidationMode::Choices || *phase != GamePhase::Choosing {
        choices.preview_started = None;
        return;
    }
    let now = Instant::now();
    let started = *choices.preview_started.get_or_insert(now);
    if choices.preview_requested || now.duration_since(started).as_secs_f64() < 1. {
        return;
    }
    choices.preview_requested = true;
    println!("VALIDATION screenshot=/tmp/dro10-choices.png");
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk("/tmp/dro10-choices.png"));
}

/// Repeatable keyboard inputs only: never writes scout transforms, velocities or HP.
/// A moving ellipse exercises turning thrust and altitude compensation.
fn pilot_input(
    config: Res<ValidationConfig>,
    run: Res<Encounter>,
    drone: Single<(&Transform, &DroneFlight), With<Drone>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    enemies: Query<(&Transform, &DroneFlight), With<Enemy>>,
) {
    if matches!(config.mode, ValidationMode::Idle | ValidationMode::Routes)
        || keys.just_pressed(KeyCode::KeyR)
    {
        return;
    }
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::KeyA,
        KeyCode::KeyD,
        KeyCode::KeyQ,
        KeyCode::KeyE,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::Space,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ] {
        keys.reset(key);
    }
    let threats: Vec<_> = enemies
        .iter()
        .map(|(t, f)| (t.translation, f.velocity))
        .collect();
    for key in pilot_keys(run.elapsed, drone.0.translation, drone.1, &threats) {
        keys.press(key);
    }
}

pub(super) fn pilot_keys(
    elapsed: f64,
    position: Vec3,
    flight: &DroneFlight,
    threats: &[(Vec3, Vec3)],
) -> Vec<KeyCode> {
    let theta = elapsed as f32 * 0.8;
    let target = Vec3::new(
        300. * theta.cos(),
        145. + 45. * (theta * 0.5).sin(),
        165. * theta.sin(),
    );
    let tangent = Vec3::new(
        -240. * theta.sin(),
        18. * (theta * 0.5).cos(),
        132. * theta.cos(),
    );
    let desired = tangent + (target - position) * 1.5;
    let acceleration = (desired - flight.velocity) * 3. + flight.velocity * 0.25;
    steering_keys(acceleration, flight, position, threats)
}

fn steering_keys(
    mut acceleration: Vec3,
    flight: &DroneFlight,
    position: Vec3,
    threats: &[(Vec3, Vec3)],
) -> Vec<KeyCode> {
    for &(point, velocity) in threats {
        let offset = position - point;
        let relative = flight.velocity - velocity;
        let closest = (-offset.dot(relative) / relative.length_squared().max(1.)).clamp(0., 0.8);
        let miss = offset + relative * closest;
        let distance = miss.length();
        if distance < 170. {
            acceleration += miss.normalize_or_zero() * (1. - distance / 170.) * 650.;
        }
    }
    let local = Quat::from_rotation_y(-flight.heading) * acceleration;
    let mut keys = Vec::with_capacity(3);
    if local.x.abs() > 15. {
        keys.push(if local.x > 0. {
            KeyCode::KeyE
        } else {
            KeyCode::KeyQ
        });
    }
    if local.z.abs() > 15. {
        keys.push(if local.z > 0. {
            KeyCode::KeyS
        } else {
            KeyCode::KeyW
        });
    }
    let vertical = acceleration.y + 360. * (1. - flight.tilt.length().cos());
    if vertical > 10. {
        keys.push(KeyCode::Space);
    } else if vertical < -10. {
        keys.push(KeyCode::ShiftLeft);
    }
    keys
}

/// Stress-only: maintain the requested rendered population by replacing kills.
/// Invulnerability prevents death from stopping the measured workload. Real hits,
/// projectiles, enemy health, pursuit and feedback still run without modification.
#[allow(clippy::too_many_arguments)]
fn replenish(
    mut commands: Commands,
    config: Res<ValidationConfig>,
    combat: Res<CombatConfig>,
    phase: Res<GamePhase>,
    mut health: ResMut<PlayerHealth>,
    drone: Single<&Transform, With<Drone>>,
    enemies: Query<&Enemy>,
    mut cursor: Local<usize>,
) {
    if config.mode != ValidationMode::Stress || *phase != GamePhase::Playing {
        return;
    }
    health.invulnerable_until = f64::INFINITY;
    let live = enemies.iter().filter(|e| e.health > 0).count();
    for _ in live..config.enemies {
        let i = *cursor % 150;
        *cursor += 1;
        let position = Vec3::new(
            -405. + (i % 10) as f32 * 90.,
            60. + (i / 50) as f32 * 90.,
            -180. + ((i / 10) % 5) as f32 * 90.,
        );
        enemies::spawn_enemy(&mut commands, &combat, position, drone.translation);
    }
}

#[derive(Resource, Default)]
struct Measurements {
    first: Option<Instant>,
    last: Option<Instant>,
    terminal: Option<Instant>,
    samples: Vec<f64>,
    min_enemies: Option<usize>,
    max_enemies: usize,
    max_projectiles: usize,
    hits: usize,
    kills: usize,
    damage: usize,
    progress: u64,
    done: bool,
}

#[allow(clippy::too_many_arguments)]
fn measure(
    config: Res<ValidationConfig>,
    mut data: ResMut<Measurements>,
    phase: Res<GamePhase>,
    run: Res<Encounter>,
    health: Res<PlayerHealth>,
    enemies: Query<&Enemy>,
    projectiles: Query<(), With<Projectile>>,
    windows: Query<&Window>,
    outcomes: Res<CombatOutcomes>,
    upgrades: Option<Res<UpgradeRun>>,
    energy: Option<Res<Energy>>,
    energy_config: Option<Res<EnergyConfig>>,
    mut exit: MessageWriter<AppExit>,
) {
    if data.done {
        return;
    }
    let now = Instant::now();
    let start = *data.first.get_or_insert(now);
    let elapsed = now.duration_since(start).as_secs_f64();
    let previous = data.last.replace(now);
    let live = enemies.iter().filter(|e| e.health > 0).count();
    if let Some(previous) = previous
        .filter(|t| t.duration_since(start).as_secs_f64() >= 5. && *phase == GamePhase::Playing)
    {
        data.samples
            .push(now.duration_since(previous).as_secs_f64() * 1000.);
        data.min_enemies = Some(data.min_enemies.map_or(live, |n| n.min(live)));
        data.max_enemies = data.max_enemies.max(live);
        data.max_projectiles = data.max_projectiles.max(projectiles.iter().count());
        for event in &outcomes.0 {
            match event {
                CombatOutcome::Hit { killed, .. } => {
                    data.hits += 1;
                    data.kills += usize::from(*killed);
                }
                CombatOutcome::PlayerDamaged => data.damage += 1,
                CombatOutcome::RocketExplosion { .. } => {}
            }
        }
    }
    let progress = (elapsed / 10.) as u64;
    if progress > data.progress {
        data.progress = progress;
        println!(
            "VALIDATION progress wall={elapsed:.1}s run={:.1}s phase={:?} hull={} kills={} enemies={live}",
            run.elapsed, *phase, health.current, run.kills
        );
    }
    if matches!(*phase, GamePhase::Dead | GamePhase::Survived) {
        data.terminal.get_or_insert(now);
    } else {
        data.terminal = None;
    }
    let terminal_done = data
        .terminal
        .is_some_and(|at| now.duration_since(at).as_secs_f64() >= 2.);
    if elapsed < config.seconds + 5. && !terminal_done {
        return;
    }
    data.done = true;
    let resolution = windows
        .iter()
        .next()
        .map(|w| (w.physical_width(), w.physical_height()));
    let build = build_summary(
        upgrades.as_deref(),
        energy.as_deref(),
        energy_config.as_deref(),
    );
    if let Some(s) = summarize(&data.samples) {
        println!(
            "VALIDATION RESULT mode={:?} sample_seconds={:.3} frames={} frame_ms_median={:.3} p95={:.3} p99={:.3} hitches_gt_33_3={} enemies_min={} enemies_max={} projectiles_max={} hits={} kills={} damage={} phase={:?} hull={} run_seconds={:.3} total_kills={} physical_resolution={resolution:?} {build}",
            config.mode,
            data.samples.iter().sum::<f64>() / 1000.,
            data.samples.len(),
            s.median,
            s.p95,
            s.p99,
            s.hitches,
            data.min_enemies.unwrap_or(0),
            data.max_enemies,
            data.max_projectiles,
            data.hits,
            data.kills,
            data.damage,
            *phase,
            health.current,
            run.elapsed,
            run.kills
        );
    } else {
        println!(
            "VALIDATION RESULT no post-warmup samples; phase={:?} {build}",
            *phase,
        );
    }
    exit.write(AppExit::Success);
}
