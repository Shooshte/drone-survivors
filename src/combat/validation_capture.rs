//! Opt-in engine screenshots for native validation; never installed in ordinary play.
use super::*;
use crate::world::hazard::{HazardPhase, HazardState};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::path::PathBuf;

#[derive(Resource)]
struct Captures {
    directory: PathBuf,
    minimum: bool,
    seen: [bool; 3],
}

pub(super) fn install(app: &mut App) {
    let Some(directory) = std::env::var_os("DRONE_CAPTURE_DIR") else {
        return;
    };
    let directory = PathBuf::from(directory);
    if let Err(error) = std::fs::create_dir_all(&directory) {
        eprintln!("Cannot create validation capture directory: {error}");
        return;
    }
    app.insert_resource(Captures {
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
        seen: [false; 3],
    })
    .add_systems(Startup, resize)
    .add_systems(Update, capture.after(GameplaySet::Presentation));
}

fn resize(captures: Res<Captures>, mut window: Single<&mut Window>) {
    if captures.minimum {
        window.resolution.set(640., 480.);
    }
}

fn capture(
    mut commands: Commands,
    run: Res<Encounter>,
    hazard: Res<HazardState>,
    mut captures: ResMut<Captures>,
) {
    if run.elapsed < 1. {
        return;
    }
    let (index, label) = match hazard.phase {
        HazardPhase::Inactive => (0, "open"),
        HazardPhase::Warning => (1, "warning"),
        HazardPhase::Active => (2, "active"),
    };
    if captures.seen[index] {
        return;
    }
    captures.seen[index] = true;
    let size = if captures.minimum {
        "640x480"
    } else {
        "1120x720"
    };
    let path = captures.directory.join(format!("{size}-{label}.png"));
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
