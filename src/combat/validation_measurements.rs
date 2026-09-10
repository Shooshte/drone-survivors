use super::super::waves::SpawnCounts;
use super::*;
use crate::{
    energy::{Energy, EnergyConfig},
    upgrades::UpgradeRun,
};

#[derive(Debug)]
pub(super) struct Summary {
    pub(super) median: f64,
    pub(super) p95: f64,
    pub(super) p99: f64,
    pub(super) hitches: usize,
}

pub(super) fn summarize(samples: &[f64]) -> Option<Summary> {
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

pub(crate) fn build_summary(
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

pub(super) fn spawn_summary(spawns: SpawnCounts) -> String {
    let pending = spawns
        .admitted
        .saturating_sub(spawns.activated + spawns.cancelled);
    format!(
        "requested={} admitted={} rejected_cap={} rejected_space={} skipped_hitch={} skipped_terminal={} activated={} cancelled={} pending={pending}",
        spawns.requested,
        spawns.admitted,
        spawns.rejected_cap,
        spawns.rejected_space,
        spawns.skipped_hitch,
        spawns.skipped_terminal,
        spawns.activated,
        spawns.cancelled,
    )
}

#[derive(Resource, Default)]
pub(super) struct Measurements {
    pub(super) first: Option<Instant>,
    last: Option<Instant>,
    previous_boundary_playing: bool,
    pub(super) terminal: Option<Instant>,
    pub(super) samples: Vec<f64>,
    min_enemies: Option<usize>,
    max_enemies: usize,
    max_projectiles: usize,
    hits: usize,
    kills: usize,
    damage: usize,
    progress: u64,
    pub(super) done: bool,
    stages: Vec<StagePopulation>,
}

impl Measurements {
    pub(super) fn observe_frame_boundary(&mut self, now: Instant, phase: GamePhase) -> bool {
        let start = *self.first.get_or_insert(now);
        let previous = self.last.replace(now);
        let playing = phase == GamePhase::Playing;
        let previous_playing = std::mem::replace(&mut self.previous_boundary_playing, playing);
        let Some(previous) = previous else {
            return false;
        };
        if playing && previous_playing && previous.duration_since(start).as_secs_f64() >= 5. {
            self.samples
                .push(now.duration_since(previous).as_secs_f64() * 1000.);
            true
        } else {
            false
        }
    }

    pub(super) fn observe_population(&mut self, label: &'static str, live: usize, reserved: usize) {
        let stage = if let Some(stage) = self.stages.iter_mut().find(|stage| stage.label == label) {
            stage
        } else {
            self.stages.push(StagePopulation {
                label,
                min_live: live,
                max_live: live,
                max_active: live + reserved,
            });
            self.stages.last_mut().unwrap()
        };
        stage.min_live = stage.min_live.min(live);
        stage.max_live = stage.max_live.max(live);
        stage.max_active = stage.max_active.max(live + reserved);
    }

    pub(super) fn stage_population_summary(&self) -> String {
        if self.stages.is_empty() {
            return "none".into();
        }
        self.stages
            .iter()
            .map(|stage| {
                format!(
                    "{}:live={}..{} active_peak={}",
                    stage.label, stage.min_live, stage.max_live, stage.max_active
                )
            })
            .collect::<Vec<_>>()
            .join("|")
    }
}

struct StagePopulation {
    label: &'static str,
    min_live: usize,
    max_live: usize,
    max_active: usize,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure(
    config: Res<ValidationConfig>,
    mut data: ResMut<Measurements>,
    phase: Res<GamePhase>,
    run: Res<Encounter>,
    health: Res<PlayerHealth>,
    enemies: Query<&Enemy>,
    projectiles: Query<(), With<Projectile>>,
    warnings: Query<(), With<SpawnWarning>>,
    windows: Query<&Window>,
    outcomes: Res<CombatOutcomes>,
    upgrades: Option<Res<UpgradeRun>>,
    energy: Option<Res<Energy>>,
    energy_config: Option<Res<EnergyConfig>>,
    waves: Res<WaveConfig>,
    observations: Res<ManualObservations>,
    mut exit: MessageWriter<AppExit>,
) {
    if data.done {
        return;
    }
    let now = Instant::now();
    let start = *data.first.get_or_insert(now);
    let elapsed = now.duration_since(start).as_secs_f64();
    let live = enemies.iter().filter(|e| e.health > 0).count();
    let reserved = warnings.iter().count();
    let stage = if config.mode == ValidationMode::Stress {
        Some("STRESS OVERRIDE")
    } else {
        waves.phase_at(run.elapsed).map(|stage| stage.label)
    };
    if *phase == GamePhase::Playing
        && let Some(label) = stage
    {
        data.observe_population(label, live, reserved);
    }
    if data.observe_frame_boundary(now, *phase) {
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
        let spawns = spawn_summary(run.spawns);
        println!(
            "VALIDATION progress wall={elapsed:.1}s run={:.1}s phase={:?} stage={} wave_status={:?} hull={} kills={} enemies_live={live} warnings_pending={reserved} active_population={} cap={} spawns[{spawns}]",
            run.elapsed,
            *phase,
            stage.unwrap_or("TERMINAL"),
            waves.status_at(run.elapsed),
            health.current,
            run.kills,
            live + reserved,
            waves.cap,
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
    let spawns = spawn_summary(run.spawns);
    let stages = data.stage_population_summary();
    let observations = observations.summary();
    if let Some(s) = summarize(&data.samples) {
        println!(
            "VALIDATION RESULT mode={:?} sample_seconds={:.3} frames={} frame_ms_median={:.3} p95={:.3} p99={:.3} hitches_gt_33_3={} enemies_min={} enemies_max={} warnings_pending={reserved} active_population={} cap={} projectiles_max={} hits={} kills={} damage={} phase={:?} stage={} wave_status={:?} hull={} run_seconds={:.3} total_kills={} physical_resolution={resolution:?} stages=[{stages}] spawns[{spawns}] observations[{observations}] {build}",
            config.mode,
            data.samples.iter().sum::<f64>() / 1000.,
            data.samples.len(),
            s.median,
            s.p95,
            s.p99,
            s.hitches,
            data.min_enemies.unwrap_or(0),
            data.max_enemies,
            live + reserved,
            waves.cap,
            data.max_projectiles,
            data.hits,
            data.kills,
            data.damage,
            *phase,
            stage.unwrap_or("TERMINAL"),
            waves.status_at(run.elapsed),
            health.current,
            run.elapsed,
            run.kills
        );
    } else {
        println!(
            "VALIDATION RESULT no post-warmup samples; mode={:?} phase={:?} stage={} wave_status={:?} enemies_live={live} warnings_pending={reserved} active_population={} cap={} run_seconds={:.3} total_kills={} stages=[{stages}] spawns[{spawns}] observations[{observations}] {build}",
            config.mode,
            *phase,
            stage.unwrap_or("TERMINAL"),
            waves.status_at(run.elapsed),
            live + reserved,
            waves.cap,
            run.elapsed,
            run.kills,
        );
    }
    exit.write(AppExit::Success);
}
