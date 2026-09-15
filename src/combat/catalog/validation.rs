//! Explicit native presentation fixture; never installed in ordinary arena play.
use super::*;
use bevy::{
    input::InputSystems,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Resource)]
struct Fixture {
    start: Instant,
    step: usize,
    directory: Option<PathBuf>,
}

pub(super) fn install(app: &mut App) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).expect("capture directory");
    }
    app.insert_resource(Fixture {
        start: Instant::now(),
        step: 0,
        directory,
    })
    .add_systems(PreUpdate, drive.after(InputSystems));
    println!(
        "CATALOG UI FIXTURE: synthetic keyboard input and XP=50 for choice preview; no campaign/save plugins. Not human or balance evidence."
    );
}

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    phase: Res<GamePhase>,
    modules: Res<Modules>,
    health: Res<super::super::PlayerHealth>,
    encounter: Res<super::super::Encounter>,
    mut upgrades: ResMut<crate::upgrades::UpgradeRun>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    for key in [
        KeyCode::Enter,
        KeyCode::ArrowRight,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Tab,
        KeyCode::KeyR,
    ] {
        keys.reset(key);
    }
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < 35.,
        "catalog fixture timed out at step {}",
        fixture.step
    );
    if elapsed < (fixture.step + 1) as f64 {
        return;
    }
    fixture.step += 1;
    let label = match fixture.step {
        1 => {
            assert_eq!(*phase, GamePhase::Hub);
            Some("selector")
        }
        2 => {
            keys.press(KeyCode::ArrowRight);
            None
        }
        3 => {
            keys.press(KeyCode::Digit1);
            None
        }
        4 => {
            keys.press(KeyCode::Digit2);
            None
        }
        5 => Some("loadout"),
        6 => {
            keys.press(KeyCode::Enter);
            None
        }
        7 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert_eq!(
                modules.loadout.slots(),
                &[
                    Some(ModuleKind::Overdrive),
                    Some(ModuleKind::Shield),
                    None,
                    None
                ]
            );
            assert_eq!(modules.enabled, [false; 4]);
            Some("round")
        }
        8 => {
            upgrades.award(50);
            None
        }
        9 => {
            assert_eq!(*phase, GamePhase::Choosing);
            Some("choice")
        }
        10 => {
            keys.press(KeyCode::Tab);
            None
        }
        11 => {
            assert_eq!(*phase, GamePhase::Hub);
            assert_eq!(health.current, 100);
            assert!(upgrades.offer.is_empty());
            assert_eq!(encounter.elapsed, 0.);
            Some("returned")
        }
        12 => {
            keys.press(KeyCode::Enter);
            None
        }
        13 => {
            keys.press(KeyCode::KeyR);
            None
        }
        14 => {
            assert_eq!(*phase, GamePhase::Playing);
            assert!(encounter.elapsed < 1.5);
            println!(
                "CATALOG UI FIXTURE PASS: selector, slot edits, launch, paused choice return and restart at {}x{}",
                window.width(),
                window.height()
            );
            exit.write(AppExit::Success);
            None
        }
        _ => None,
    };
    if let Some(label) = label {
        for (text, node, transform) in &text_nodes {
            if node.size().x < 1. || node.size().y < 1. {
                continue;
            }
            let center = transform.translation / window.scale_factor();
            let half = node.size() / window.scale_factor() / 2.;
            assert!(
                center.x - half.x >= -1.
                    && center.y - half.y >= -1.
                    && center.x + half.x <= window.width() + 1.
                    && center.y + half.y <= window.height() + 1.,
                "text outside viewport: {:?} center={center:?} half={half:?}",
                text.0
            );
        }
        println!(
            "CATALOG UI FIXTURE {label}: phase={phase:?} hull={} time={:.2}",
            health.current, encounter.elapsed
        );
        if let Some(directory) = &fixture.directory {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(directory.join(format!(
                    "{}x{}-{label}.png",
                    window.width(),
                    window.height()
                ))));
        }
    }
}
