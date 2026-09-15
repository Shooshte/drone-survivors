//! Opt-in native UI fixture. Teleports and fatal hull are synthetic; objective rules are real.
use super::{
    Campaign, MissionAction, MissionSession,
    campaign::MissionId,
    objectives::{ObjectiveConfig, ObjectiveRun},
};
use crate::{
    arena::{Drone, DroneFlight},
    combat::{Encounter, PlayerHealth},
    game::GamePhase,
};
use bevy::{
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Clone)]
enum Step {
    Key(KeyCode),
    Select(usize),
    Move(Vec3),
    Count(usize),
    Phase(GamePhase),
    Deadline,
    Die,
    Result(bool),
    Capture(&'static str),
    Finish,
}
#[derive(Resource)]
struct Fixture {
    start: Instant,
    next_at: f64,
    seconds: f64,
    index: usize,
    steps: Vec<Step>,
    directory: Option<PathBuf>,
    minimum: bool,
}
pub(crate) fn install(app: &mut App, seconds: f64) {
    let config = ObjectiveConfig::default();
    let mut steps = Vec::new();
    for index in [1, 2] {
        steps.extend([
            Step::Key(KeyCode::KeyC),
            Step::Select(index),
            Step::Key(KeyCode::Enter),
            Step::Phase(GamePhase::Briefing),
            Step::Capture(if index == 1 {
                "scan-briefing"
            } else {
                "cargo-briefing"
            }),
            Step::Key(KeyCode::Enter),
            Step::Phase(GamePhase::Playing),
            Step::Move(config.extraction + Vec3::Z * 200.),
            Step::Capture(if index == 1 {
                "scan-locked-exit"
            } else {
                "cargo-locked-exit"
            }),
            Step::Move(config.extraction),
            Step::Count(0),
            Step::Deadline,
            Step::Phase(GamePhase::Playing),
            Step::Move(config.sites[0] + Vec3::Z * 200.),
            Step::Capture(if index == 1 {
                "scan-beacon"
            } else {
                "cargo-beacon"
            }),
            Step::Move(config.sites[0]),
            Step::Count(1),
            Step::Move(config.sites[0]),
            Step::Count(1),
            Step::Key(KeyCode::KeyR),
            Step::Count(0),
        ]);
        for point in config.sites {
            steps.push(Step::Move(point));
        }
        steps.extend([
            Step::Count(3),
            Step::Move(config.extraction + Vec3::Z * 200.),
            Step::Capture(if index == 1 {
                "scan-ready-exit"
            } else {
                "cargo-ready-exit"
            }),
            Step::Die,
            Step::Result(false),
            Step::Capture(if index == 1 {
                "scan-failure"
            } else {
                "cargo-failure"
            }),
            Step::Key(KeyCode::Enter),
            Step::Phase(GamePhase::Hub),
            Step::Key(KeyCode::Enter),
            Step::Key(KeyCode::Enter),
            Step::Count(0),
        ]);
        for point in config.sites {
            steps.push(Step::Move(point));
        }
        steps.extend([
            Step::Count(3),
            Step::Move(config.extraction),
            Step::Result(true),
            Step::Capture(if index == 1 {
                "scan-success"
            } else {
                "cargo-success"
            }),
            Step::Key(KeyCode::Enter),
            Step::Phase(GamePhase::Hub),
        ]);
    }
    steps.push(Step::Finish);
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).expect("objective capture directory");
    }
    app.add_plugins((
        super::MissionPlugin,
        super::scene::MissionScenePlugin,
        super::objective_scene::ObjectiveScenePlugin,
        crate::economy::scene::EconomyScenePlugin,
    ))
    .insert_resource(Fixture {
        start: Instant::now(),
        next_at: 0.5,
        seconds,
        index: 0,
        steps,
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
    })
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, drive.after(InputSystems));
}
fn setup(
    mut fixture: ResMut<Fixture>,
    mut window: Single<&mut Window>,
    mut campaign: ResMut<Campaign>,
) {
    fixture.start = Instant::now();
    if fixture.minimum {
        window.resolution.set(640., 480.);
    }
    campaign.progress.complete(MissionId::ALL[0]);
    println!(
        "OBJECTIVES FIXTURE: synthetic positions/fatal hull, mission 01 pre-unlocked, spawn bursts disabled. Real objective/lifecycle/reward rules; no save or balance claim."
    );
}
fn move_to(world: &mut World, position: Vec3) {
    let mut query = world.query_filtered::<(&mut Transform, &mut DroneFlight), With<Drone>>();
    let (mut transform, mut flight) = query.single_mut(world).unwrap();
    transform.translation = position;
    *flight = default();
}
fn drive(world: &mut World) {
    world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
    for (_, mut interaction) in world
        .query::<(&MissionAction, &mut Interaction)>()
        .iter_mut(world)
    {
        *interaction = Interaction::None;
    }
    let fixture = world.resource::<Fixture>();
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < fixture.seconds,
        "objective fixture timeout at {}",
        fixture.index
    );
    if elapsed < fixture.next_at {
        return;
    }
    let Some(step) = fixture.steps.get(fixture.index).cloned() else {
        return;
    };
    match step {
        Step::Key(key) => world.resource_mut::<ButtonInput<KeyCode>>().press(key),
        Step::Select(index) => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::MissionSelect);
            let mut found = false;
            for (action, mut interaction) in world
                .query::<(&MissionAction, &mut Interaction)>()
                .iter_mut(world)
            {
                if *action == MissionAction::SelectMission(MissionId::ALL[index]) {
                    *interaction = Interaction::Pressed;
                    found = true;
                }
            }
            assert!(found);
        }
        Step::Move(position) => move_to(world, position),
        Step::Count(count) => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Playing);
            assert_eq!(world.resource::<ObjectiveRun>().count(), count);
        }
        Step::Phase(phase) => assert_eq!(*world.resource::<GamePhase>(), phase),
        Step::Deadline => world.resource_mut::<Encounter>().elapsed = 301.,
        Step::Die => {
            move_to(world, world.resource::<ObjectiveConfig>().extraction);
            world.resource_mut::<PlayerHealth>().current = 0;
        }
        Step::Result(won) => {
            let session = world.resource::<MissionSession>();
            let result = session.result.as_ref().expect("objective result");
            assert_eq!(result.succeeded, won);
            assert_eq!(result.mission, session.selected_mission);
            assert_eq!(result.rewards.credited.salvage, if won { 10 } else { 0 });
            assert_eq!(result.rewards.credited.components, if won { 1 } else { 0 });
        }
        Step::Capture(label) => capture(world, label),
        Step::Finish => {
            let campaign = world.resource::<Campaign>();
            assert_eq!(campaign.history.len(), 4);
            assert_eq!(campaign.progress.count(), 3);
            assert_eq!(campaign.wallet.salvage, 20);
            assert_eq!(campaign.wallet.components, 2);
            println!(
                "OBJECTIVES FIXTURE PASS: both types, premature exit, unique triggers, no deadline victory, clean restart, death at ready exit, success, 4 results and 20 salvage/2 components."
            );
            world.write_message(AppExit::Success);
        }
    }
    let mut fixture = world.resource_mut::<Fixture>();
    fixture.index += 1;
    fixture.next_at = elapsed + 0.35;
}
fn capture(world: &mut World, label: &str) {
    let window = world.query::<&Window>().single(world).unwrap();
    let scale = window.scale_factor();
    let size = Vec2::new(window.width(), window.height());
    let mut visible = 0;
    for (text, node, transform) in world
        .query::<(&Text, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
    {
        if node.size().min_element() < 1. {
            continue;
        }
        visible += 1;
        let center = transform.translation / scale;
        let half = node.size() / scale / 2.;
        assert!(
            (center - half).cmpge(Vec2::splat(-1.)).all()
                && (center + half).cmple(size + Vec2::ONE).all(),
            "text outside viewport: {}",
            text.0
        );
    }
    if label.contains("beacon") || label.contains("exit") {
        assert!(world.query_filtered::<(&Text,&ComputedNode),With<super::objective_scene::MarkerLabel>>()
            .iter(world).any(|(text,node)| !text.0.is_empty() && node.size().min_element()>1.),
            "objective beacon label must be visible at {label}");
    }
    assert!(visible > 3);
    println!(
        "OBJECTIVES {label}: phase={:?}, progress={}, visible_labels={visible}",
        world.resource::<GamePhase>(),
        world.resource::<ObjectiveRun>().count()
    );
    let fixture = world.resource::<Fixture>();
    if let Some(directory) = &fixture.directory {
        let size = if fixture.minimum {
            "640x480"
        } else {
            "1120x720"
        };
        let path = directory.join(format!("{size}-{label}.png"));
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}
