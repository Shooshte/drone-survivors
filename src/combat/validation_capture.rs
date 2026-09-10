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
    charger_seen: [bool; 4],
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
        charger_seen: [false; 4],
    })
    .add_systems(Startup, resize)
    .add_systems(
        Update,
        (capture, capture_chargers)
            .chain()
            .after(GameplaySet::Presentation),
    );
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

fn capture_chargers(
    mut commands: Commands,
    validation: Res<ValidationConfig>,
    run: Res<Encounter>,
    config: Res<crate::energy::ChargerConfig>,
    nodes: Query<&crate::energy::ChargerReserve>,
    mut captures: ResMut<Captures>,
) {
    if validation.mode != ValidationMode::Chargers || run.elapsed < 1. {
        return;
    }
    for reserve in &nodes {
        let state = if reserve.occupied && reserve.remaining == 0. {
            Some((1, "charger-depleted"))
        } else if !reserve.occupied && reserve.remaining < config.capacity {
            if reserve.away_seconds < config.recovery_delay {
                Some((2, "charger-delay"))
            } else {
                Some((3, "charger-recovering"))
            }
        } else if reserve.occupied && reserve.remaining < config.capacity * 0.5 {
            Some((0, "charger-in-use"))
        } else {
            None
        };
        if let Some((index, label)) = state
            && !captures.charger_seen[index]
        {
            captures.charger_seen[index] = true;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::{ChargerConfig, ChargerReserve};

    #[test]
    fn charger_screenshots_are_exclusive_to_charger_fixture() {
        for mode in [
            ValidationMode::Manual,
            ValidationMode::Survival,
            ValidationMode::Stress,
            ValidationMode::Idle,
            ValidationMode::Routes,
            ValidationMode::Mobile,
            ValidationMode::Armored,
            ValidationMode::Choices,
            ValidationMode::Chargers,
        ] {
            let mut app = App::new();
            app.insert_resource(ValidationConfig {
                mode,
                seconds: 60.,
                enemies: 150,
            })
            .insert_resource(Encounter {
                elapsed: 2.,
                ..default()
            })
            .init_resource::<HazardState>()
            .init_resource::<ChargerConfig>()
            .insert_resource(Captures {
                directory: PathBuf::from("unused-capture-test-directory"),
                minimum: false,
                seen: [false; 3],
                charger_seen: [false; 4],
            })
            .add_systems(Update, (capture, capture_chargers).chain());
            // Exercise all four charger triggers without rendering or filesystem writes.
            for (remaining, away_seconds, occupied) in [
                (50., 0., true),
                (0., 0., true),
                (0., 2., false),
                (50., 10., false),
            ] {
                app.world_mut().spawn(ChargerReserve {
                    remaining,
                    away_seconds,
                    occupied,
                });
            }
            app.update();
            let dedicated = mode == ValidationMode::Chargers;
            assert_eq!(
                app.world().resource::<Captures>().charger_seen,
                [dedicated; 4],
                "{mode:?}"
            );
            let expected = if dedicated { 5 } else { 1 }; // Hazard capture remains available in every mode.
            let requests = |app: &mut App| {
                app.world_mut()
                    .query_filtered::<Entity, With<Screenshot>>()
                    .iter(app.world())
                    .count()
            };
            assert_eq!(requests(&mut app), expected, "{mode:?}");
            app.update();
            assert_eq!(
                requests(&mut app),
                expected,
                "duplicate capture in {mode:?}"
            );
        }
    }
}
