//! Explicit UI fixture: real selection/launch/finalization with synthetic outcomes.
use super::{Campaign, MissionAction, MissionSession, campaign::MissionId};
use crate::{
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
    Click(MissionAction),
    Win,
    Fail,
    Phase(GamePhase),
    Selected(MissionId),
    Progress(usize),
    Result(MissionId, bool),
    Capture(&'static str),
    Finish,
}
#[derive(Resource)]
struct Fixture {
    start: Instant,
    steps: Vec<Step>,
    index: usize,
    next_at: f64,
    release_frame: bool,
    seconds: f64,
    directory: Option<PathBuf>,
    minimum: bool,
    clicked: Option<MissionAction>,
}

pub(crate) fn install(app: &mut App, seconds: f64) {
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(path) = &directory {
        std::fs::create_dir_all(path).expect("campaign capture directory");
    }
    let mut steps = vec![
        Step::Capture("hub"),
        Step::Key(KeyCode::KeyC),
        Step::Phase(GamePhase::MissionSelect),
        Step::Capture("initial"),
        Step::Click(MissionAction::SelectMission(MissionId::ALL[11])),
        Step::Selected(MissionId::ALL[0]),
        Step::Capture("locked"),
        Step::Key(KeyCode::Backspace),
    ];
    let reverse = std::env::var_os("DRONE_CAMPAIGN_REVERSE").is_some();
    for act in 0..3 {
        for offset in if reverse { [0, 2, 1, 3] } else { [0, 1, 2, 3] } {
            let id = MissionId::ALL[act * 4 + offset];
            steps.extend([
                Step::Key(KeyCode::KeyC),
                Step::Phase(GamePhase::MissionSelect),
                Step::Click(MissionAction::SelectMission(id)),
                Step::Selected(id),
                Step::Key(KeyCode::Enter),
                Step::Phase(GamePhase::Briefing),
            ]);
            if offset == 0 {
                steps.push(Step::Capture(match act {
                    0 => "briefing",
                    1 => "act2-briefing",
                    _ => "act3-briefing",
                }));
            }
            steps.extend([Step::Key(KeyCode::Enter), Step::Phase(GamePhase::Playing)]);
            if id == MissionId::ALL[0] {
                steps.extend([Step::Key(KeyCode::KeyR), Step::Phase(GamePhase::Playing)]);
            }
            steps.extend([Step::Win, Step::Result(id, true)]);
            if id == MissionId::ALL[0] {
                steps.push(Step::Capture("first-win-result"));
            }
            steps.extend([Step::Key(KeyCode::Enter), Step::Phase(GamePhase::Hub)]);
            if id == MissionId::ALL[0] {
                steps.extend([
                    Step::Capture("first-win-hub"),
                    Step::Key(KeyCode::KeyM),
                    Step::Phase(GamePhase::ModuleShop),
                    Step::Capture("first-win-shop"),
                    Step::Key(KeyCode::Backspace),
                ]);
            }
            if offset == 0 {
                steps.extend([
                    Step::Key(KeyCode::KeyC),
                    Step::Capture(match act {
                        0 => "branches",
                        1 => "act2-branches",
                        _ => "act3-branches",
                    }),
                    Step::Key(KeyCode::Backspace),
                ]);
            }
        }
    }
    // Every unlocked mission stays replayable after campaign completion.
    steps.extend([
        Step::Progress(12),
        Step::Capture("complete-hub"),
        Step::Key(KeyCode::KeyC),
        Step::Capture("complete-selection"),
        Step::Click(MissionAction::SelectMission(MissionId::ALL[1])),
        Step::Selected(MissionId::ALL[1]),
        Step::Key(KeyCode::Enter),
        Step::Key(KeyCode::Enter),
        Step::Phase(GamePhase::Playing),
        Step::Fail,
        Step::Result(MissionId::ALL[1], false),
        Step::Capture("replay-failure"),
        Step::Progress(12),
        Step::Key(KeyCode::Enter),
        Step::Key(KeyCode::Enter),
        Step::Key(KeyCode::Enter),
        Step::Phase(GamePhase::Playing),
        Step::Win,
        Step::Result(MissionId::ALL[1], true),
        Step::Capture("replay-success"),
        Step::Key(KeyCode::Enter),
        Step::Progress(12),
        Step::Finish,
    ]);
    println!(
        "CAMPAIGN FIXTURE: synthetic success timers/objective progress and fatal hull; real menu/launch/reward rules. reverse={reverse}. Not combat balance evidence."
    );
    app.add_plugins((
        super::MissionPlugin,
        super::objective_scene::ObjectiveScenePlugin,
        super::scene::MissionScenePlugin,
        crate::economy::scene::EconomyScenePlugin,
    ))
    .insert_resource(Fixture {
        start: Instant::now(),
        steps,
        index: 0,
        next_at: 0.3,
        release_frame: false,
        seconds,
        directory,
        minimum: std::env::var_os("DRONE_CAPTURE_MINIMUM").is_some(),
        clicked: None,
    })
    .add_systems(Startup, resize)
    .add_systems(PreUpdate, drive.after(InputSystems));
}
fn resize(mut fixture: ResMut<Fixture>, mut window: Single<&mut Window>) {
    fixture.start = Instant::now();
    if fixture.minimum {
        window.resolution.set(640., 480.);
    }
}
#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    mut fixture: ResMut<Fixture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    phase: Res<GamePhase>,
    campaign: Res<Campaign>,
    session: Res<MissionSession>,
    mut encounter: ResMut<Encounter>,
    mut health: ResMut<PlayerHealth>,
    mut objective: ResMut<super::objectives::ObjectiveRun>,
    targets: Res<super::objectives::ObjectiveConfig>,
    mut drone: Single<&mut Transform, With<crate::arena::Drone>>,
    mut buttons: Query<(&MissionAction, &mut Interaction)>,
    text_nodes: Query<(&Text, &ComputedNode, &UiGlobalTransform)>,
    control_labels: Query<(&Text, &ComputedNode), With<super::selection_scene::ControlText>>,
    window: Single<&Window>,
    mut exit: MessageWriter<AppExit>,
) {
    for key in [
        KeyCode::KeyC,
        KeyCode::KeyM,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::KeyR,
    ] {
        keys.reset(key);
    }
    if let Some(clicked) = fixture.clicked.take() {
        for (action, mut interaction) in &mut buttons {
            if *action == clicked {
                *interaction = Interaction::None;
            }
        }
    }
    let elapsed = fixture.start.elapsed().as_secs_f64();
    assert!(
        elapsed < fixture.seconds,
        "campaign fixture timeout at {}",
        fixture.index
    );
    if fixture.release_frame {
        fixture.release_frame = false;
        return;
    }
    if elapsed < fixture.next_at {
        return;
    }
    let Some(step) = fixture.steps.get(fixture.index).cloned() else {
        return;
    };
    match step {
        Step::Key(key) => keys.press(key),
        Step::Click(target) => {
            assert_eq!(*phase, GamePhase::MissionSelect);
            let mut found = false;
            for (action, mut interaction) in &mut buttons {
                if *action == target {
                    *interaction = Interaction::Pressed;
                    found = true;
                }
            }
            assert!(found, "missing mission button");
            fixture.clicked = Some(target);
        }
        Step::Win => {
            assert_eq!(*phase, GamePhase::Playing);
            if objective.survival() {
                encounter.elapsed = 300.;
            } else {
                objective.visited = [true; 3];
                drone.translation = targets.extraction;
            }
        }
        Step::Fail => {
            assert_eq!(*phase, GamePhase::Playing);
            health.current = 0;
        }
        Step::Phase(expected) => assert_eq!(*phase, expected, "campaign step {}", fixture.index),
        Step::Selected(expected) => assert_eq!(session.selected_mission, expected),
        Step::Progress(count) => assert_eq!(campaign.progress.count(), count),
        Step::Result(id, succeeded) => {
            let result = session.result.as_ref().expect("completed result");
            assert_eq!((result.mission, result.succeeded), (id, succeeded));
            if campaign.history.len() == 1 {
                assert_eq!(
                    result.attempt, 2,
                    "R must create a new attempt without recording the interrupted one"
                );
            }
        }
        Step::Capture(label) => {
            if *phase == GamePhase::MissionSelect {
                assert_eq!(control_labels.iter().count(), 14);
                for (text, node) in &control_labels {
                    assert!(
                        !text.0.is_empty() && node.size().x > 1. && node.size().y > 1.,
                        "collapsed campaign control: {:?} {:?}",
                        text.0,
                        node.size()
                    );
                }
            }
            let mut visible = 0;
            for (text, node, transform) in &text_nodes {
                if node.size().x < 1. || node.size().y < 1. {
                    continue;
                }
                visible += 1;
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
            assert!(visible > 5);
            println!(
                "CAMPAIGN FIXTURE {label}: phase={phase:?} complete={} history={} selected={:?} visible_labels={visible}",
                campaign.progress.count(),
                campaign.history.len(),
                session.selected_mission
            );
            if let Some(path) = &fixture.directory {
                let size = if fixture.minimum {
                    "640x480"
                } else {
                    "1120x720"
                };
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path.join(format!("{size}-{label}.png"))));
            }
        }
        Step::Finish => {
            assert!(campaign.progress.finished());
            assert_eq!(campaign.history.len(), 14);
            assert_eq!(
                campaign.wallet,
                crate::economy::Amounts {
                    salvage: 130,
                    components: 13
                }
            );
            assert_eq!(*phase, GamePhase::Hub);
            println!(
                "CAMPAIGN FIXTURE PASS: 12 distinct missions, all acts, locked rejection, restart, complete campaign, failure and successful replays; 14 results, 130 salvage/13 components."
            );
            exit.write(AppExit::Success);
        }
    }
    fixture.index += 1;
    fixture.next_at = elapsed + 0.3;
    fixture.release_frame = true;
}
