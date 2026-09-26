//! Explicit native presentation check. No save access; not installed in ordinary play.
use crate::{
    arena::{Drone, DroneFlight},
    game::{GamePhase, GameplaySet},
    world::mission01 as map,
};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};
#[derive(Resource)]
struct Check {
    started: Instant,
    step: usize,
    directory: PathBuf,
    minimum: bool,
    pending: Option<&'static str>,
    frame_ms: Vec<f64>,
}
pub(crate) fn install(app: &mut App) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/drone-mission01-check"));
    std::fs::create_dir_all(&directory).expect("capture directory must be writable");
    app.insert_resource(Check {
        started: Instant::now(),
        step: 0,
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
        pending: None,
        frame_ms: Vec::new(),
    })
    .add_systems(Startup, resize)
    .add_systems(Update, drive.before(GameplaySet::Transition))
    .add_systems(Update, capture.after(GameplaySet::Presentation));
}
fn resize(check: Res<Check>, mut window: Single<&mut Window>) {
    if check.minimum {
        window.resolution.set(640., 480.);
    }
}
fn drive(world: &mut World) {
    world.resource_scope(|world, mut check: Mut<Check>| {
        if *world.resource::<GamePhase>() == GamePhase::Choosing {
            let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
            if keys.pressed(KeyCode::Backspace) {
                keys.reset_all();
            } else {
                keys.press(KeyCode::Backspace);
            }
            return;
        }
        if check.step >= 4
            && *world.resource::<GamePhase>() == GamePhase::Playing
            && check.frame_ms.len() < 10000
            && let Some(time) = world.get_resource::<Time<Real>>()
        {
            check.frame_ms.push(time.delta_secs_f64() * 1000.);
        }
        const TIMES: [f64; 17] = [
            2., 3., 4., 5., 6., 8., 10., 11., 12., 13., 14., 16., 18., 48., 50., 52., 55.,
        ];
        let elapsed = check.started.elapsed().as_secs_f64();
        if check.step >= TIMES.len() || elapsed < TIMES[check.step] {
            return;
        }
        let step = check.step;
        check.step += 1;
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        match step {
            0 | 2 => world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Enter),
            4 => {
                check.pending = Some("launch");
                world.resource_mut::<crate::combat::CombatConfig>().contact_damage = 0;
            }
            5 => teleport(world, map::pickup()),
            6 => check.pending = Some("payload"),
            7 => teleport(world, map::point(565., 650.)),
            8 => check.pending = Some("ridge"),
            9 => teleport(world, map::fields()[1].bounds.center.with_y(90.)),
            10 => check.pending = Some("directional-field"),
            11 => teleport(world, map::challenge()),
            12 => check.pending = Some("holdout"),
            13 => check.pending = Some("holdout-complete"),
            14 => teleport(world, map::delivery()),
            15 => check.pending = Some("result"),
            16 => {
                let success = *world.resource::<GamePhase>() == GamePhase::Survived;
                let result = world.resource::<super::MissionSession>().result.as_ref();
                check.frame_ms.sort_by(f64::total_cmp);
                let frames = &check.frame_ms;
                if !frames.is_empty() {
                    eprintln!(
                        "Native frame intervals: n={} median={:.2}ms p95={:.2}ms max={:.2}ms",
                        frames.len(), frames[frames.len() / 2],
                        frames[frames.len() * 95 / 100], frames[frames.len() - 1]
                    );
                }
                eprintln!("Mission01 native check: success={success}, result={result:?}; scripted poses, contact damage disabled, no save access");
                world.write_message(if success { AppExit::Success } else { AppExit::error() });
            }
            _ => {}
        }
    });
}
fn teleport(world: &mut World, position: Vec3) {
    let (mut transform, mut flight) = world
        .query_filtered::<(&mut Transform, &mut DroneFlight), With<Drone>>()
        .single_mut(world)
        .unwrap();
    transform.translation = position;
    *flight = default();
}
fn capture(mut commands: Commands, mut check: ResMut<Check>) {
    let Some(label) = check.pending.take() else {
        return;
    };
    let size = if check.minimum { "640x480" } else { "1120x720" };
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(
            check.directory.join(format!("dro-42-{size}-{label}.png")),
        ));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_check_releases_and_skips_upgrade_choices() {
        let mut world = World::new();
        world.insert_resource(Check {
            started: Instant::now(),
            step: 10,
            directory: PathBuf::new(),
            minimum: false,
            pending: None,
            frame_ms: Vec::new(),
        });
        world.insert_resource(GamePhase::Choosing);
        world.init_resource::<ButtonInput<KeyCode>>();
        drive(&mut world);
        assert!(
            world
                .resource::<ButtonInput<KeyCode>>()
                .pressed(KeyCode::Backspace)
        );
        drive(&mut world);
        assert!(
            !world
                .resource::<ButtonInput<KeyCode>>()
                .pressed(KeyCode::Backspace)
        );
        assert_eq!(world.resource::<Check>().step, 10);
    }
}
