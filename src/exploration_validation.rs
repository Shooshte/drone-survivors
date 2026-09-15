//! Opt-in native exploration fixture. No SavePlugin is installed, so this never touches user saves.
use crate::{
    arena::{Drone, DroneFlight},
    combat::{Encounter, PlayerHealth, validation::routes},
    economy::{
        Amounts, AttemptResources,
        runtime::{DiscoveryNotice, DiscoverySite, EconomyConfig, Pickup},
    },
    energy::{ChargerConfig, EnergyConfig},
    game::GamePhase,
    mission::{Campaign, MissionAction, MissionSession, campaign::MissionId},
    upgrades::UpgradeRun,
    world::{WorldGeometry, navigation},
};
use bevy::{
    input::InputSystems,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use std::{path::PathBuf, time::Instant};

#[derive(Clone, Debug)]
enum Step {
    Key(KeyCode),
    Phase(GamePhase),
    Select(usize),
    Profile(usize),
    RuntimeProfile(usize),
    Capture(&'static str, &'static str),
    Feedback(&'static str),
    Battery(bool),
    Balance(u64, u64),
    Capacity(f64),
    Fund,
    CompletePrior(usize),
    FlyNorthwest,
    MoveToCache(usize),
    Cache(usize, u64),
    SkipChoices,
    Deadline,
    Restart,
    Die,
    Result(bool, usize),
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
    flight_started: Option<f64>,
    flight_origin: Option<Vec3>,
    choice_skips: u32,
}

pub(crate) fn install(app: &mut App, seconds: f64) {
    use Step::*;
    let steps = vec![
        // A locked blueprint must be visible and must refuse a real keyboard purchase.
        Key(KeyCode::KeyU),
        Phase(GamePhase::Passives),
        Capture("battery-locked", "LOCKED"),
        Key(KeyCode::KeyB),
        Feedback("Discover the Reserve battery"),
        Key(KeyCode::Backspace),
        Phase(GamePhase::Hub),
        // Act 1: all NW travel is physical through the production keyboard pilot.
        Key(KeyCode::Enter),
        Phase(GamePhase::Briefing),
        Profile(0),
        Capture("scrapyard-briefing", "Scrapyard"),
        Key(KeyCode::Enter),
        Phase(GamePhase::Playing),
        RuntimeProfile(0),
        FlyNorthwest,
        Cache(0, 1),
        SkipChoices,
        Capture("northwest-blueprint", "Reserve battery blueprint unlocked"),
        Deadline,
        Result(true, 0),
        Key(KeyCode::Enter),
        Phase(GamePhase::Hub),
        Key(KeyCode::KeyU),
        Phase(GamePhase::Passives),
        Capture("battery-unlocked-unaffordable", "Need 20 salvage"),
        Fund,
        Capture("battery-unlocked-affordable", "BUY: 20 salvage"),
        Key(KeyCode::KeyB),
        Feedback("Reserve battery purchased"),
        Battery(true),
        Balance(10, 2),
        Capture("battery-active", "ACTIVE"),
        Key(KeyCode::KeyB),
        Feedback("already active"),
        Key(KeyCode::Backspace),
        Phase(GamePhase::Hub),
        Key(KeyCode::Enter),
        Phase(GamePhase::Briefing),
        Capture("battery-briefing-active", "Reserve battery"),
        Key(KeyCode::Enter),
        Phase(GamePhase::Playing),
        Capacity(125.),
        Deadline,
        Result(true, 0),
        Battery(true),
        Capture("battery-success-retained", "Reserve battery retained"),
        Key(KeyCode::Enter),
        Phase(GamePhase::Hub),
        Key(KeyCode::Enter),
        Phase(GamePhase::Briefing),
        Key(KeyCode::Enter),
        Phase(GamePhase::Playing),
        Capacity(125.),
        Restart,
        Battery(false),
        Capacity(100.),
        Die,
        Result(false, 0),
        Battery(false),
        Capture("battery-defeat-forfeited", "Reserve battery forfeited"),
        Key(KeyCode::Enter),
        Phase(GamePhase::Hub),
        Key(KeyCode::KeyU),
        Phase(GamePhase::Passives),
        Capture("battery-repurchase", "BUY: 20 salvage"),
        Key(KeyCode::KeyB),
        Feedback("Reserve battery purchased"),
        Battery(true),
        Balance(0, 2),
        Key(KeyCode::Backspace),
        Phase(GamePhase::Hub),
        Key(KeyCode::Enter),
        Phase(GamePhase::Briefing),
        Key(KeyCode::Enter),
        Phase(GamePhase::Playing),
        Capacity(125.),
        Die,
        Result(false, 0),
        Battery(false),
        Key(KeyCode::Enter),
        Phase(GamePhase::Hub),
        // Prior act victories are synthetic setup; launch, caches and profile rules are real.
        CompletePrior(1),
        Key(KeyCode::KeyC),
        Phase(GamePhase::MissionSelect),
        Select(4),
        Key(KeyCode::Enter),
        Phase(GamePhase::Briefing),
        Profile(1),
        Capture("ruins-briefing", "Ruins"),
        Key(KeyCode::Enter),
        Phase(GamePhase::Playing),
        RuntimeProfile(1),
        MoveToCache(2),
        Cache(2, 2),
        SkipChoices,
        Capture("ruins-central-cache", "Cache recovered"),
        Die,
        Result(false, 4),
        Key(KeyCode::Enter),
        Phase(GamePhase::Hub),
        CompletePrior(2),
        Key(KeyCode::KeyC),
        Phase(GamePhase::MissionSelect),
        Select(8),
        Key(KeyCode::Enter),
        Phase(GamePhase::Briefing),
        Profile(2),
        Capture("power-station-briefing", "Power station"),
        Key(KeyCode::Enter),
        Phase(GamePhase::Playing),
        RuntimeProfile(2),
        MoveToCache(1),
        Cache(1, 1),
        Capture(
            "southeast-secret-route",
            "Secret route: mission 12 unlocked",
        ),
        MoveToCache(2),
        Cache(2, 2),
        SkipChoices,
        Capture("power-station-earned-choice", "Cache recovered"),
        Die,
        Result(false, 8),
        Key(KeyCode::Enter),
        Phase(GamePhase::Hub),
        Key(KeyCode::KeyC),
        Phase(GamePhase::MissionSelect),
        Select(11),
        Capture("secret-finale-selection", "AVAILABLE / Secret route"),
        Finish,
    ];
    let directory = std::env::var_os("DRONE_CAPTURE_DIR").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).expect("exploration capture directory");
    }
    app.add_plugins((
        crate::mission::MissionPlugin,
        crate::mission::scene::MissionScenePlugin,
        crate::mission::objective_scene::ObjectiveScenePlugin,
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
        flight_started: None,
        flight_origin: None,
        choice_skips: 0,
    })
    .add_systems(Startup, setup)
    .add_systems(PreUpdate, drive.after(InputSystems));
}

fn setup(mut fixture: ResMut<Fixture>, mut window: Single<&mut Window>) {
    fixture.start = Instant::now();
    if fixture.minimum {
        window.resolution.set(640., 480.);
    }
    println!(
        "EXPLORATION FIXTURE: no SavePlugin/user-save access. Authored waves disabled; NW blueprint reached by production keyboard flight. All actual collection, XP, UI purchase, mission transitions, capacity, region settings, and finale selection use production systems."
    );
}

fn move_to(world: &mut World, position: Vec3) {
    let mut query = world.query_filtered::<(&mut Transform, &mut DroneFlight), With<Drone>>();
    let (mut transform, mut flight) = query.single_mut(world).unwrap();
    transform.translation = position;
    *flight = default();
}

fn select(world: &mut World, index: usize) {
    assert_eq!(*world.resource::<GamePhase>(), GamePhase::MissionSelect);
    let id = MissionId::ALL[index];
    assert!(
        world.resource::<Campaign>().progress.unlocked(id),
        "fixture mission {} locked",
        index + 1
    );
    let mut found = false;
    for (action, mut interaction) in world
        .query::<(&MissionAction, &mut Interaction)>()
        .iter_mut(world)
    {
        if *action == MissionAction::SelectMission(id) {
            *interaction = Interaction::Pressed;
            found = true;
        }
    }
    assert!(
        found,
        "mission selection UI has no mission {} button",
        index + 1
    );
}

fn cache_exists(world: &mut World, site: usize) -> bool {
    world
        .query::<(&DiscoverySite, &Pickup)>()
        .iter(world)
        .any(|(discovery, _)| discovery.0 == site)
}

/// The only long flight: no Transform/velocity edits or teleport since launch.
fn fly_northwest(world: &mut World, elapsed: f64) -> bool {
    let phase = *world.resource::<GamePhase>();
    if phase == GamePhase::Choosing {
        let next_at = world.resource::<Fixture>().next_at;
        if elapsed >= next_at {
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Backspace);
            let mut fixture = world.resource_mut::<Fixture>();
            fixture.next_at = elapsed + 0.5;
            fixture.choice_skips += 1;
            println!("EXPLORATION CHOICE: keyboard Backspace Skip on physical NW flight");
        }
        return false;
    }
    assert_eq!(phase, GamePhase::Playing, "physical flight interrupted");
    if !cache_exists(world, 0) {
        let pos = world
            .query_filtered::<&Transform, With<Drone>>()
            .single(world)
            .unwrap()
            .translation;
        let started = world.resource::<Fixture>().flight_started.unwrap();
        let origin = world.resource::<Fixture>().flight_origin.unwrap();
        println!(
            "EXPLORATION FLIGHT: NW cache collected by keyboard, origin={origin:?} finish={pos:?} duration={:.2}s",
            elapsed - started
        );
        assert!(
            origin.distance(pos) > 1000.,
            "NW flight did not cover the map"
        );
        return true;
    }
    let (pos, flight) = world
        .query_filtered::<(&Transform, &DroneFlight), With<Drone>>()
        .single(world)
        .map(|(t, f)| (t.translation, *f))
        .unwrap();
    if world.resource::<Fixture>().flight_started.is_none() {
        let mut fixture = world.resource_mut::<Fixture>();
        fixture.flight_started = Some(elapsed);
        fixture.flight_origin = Some(pos);
    }
    let cache = world.resource::<EconomyConfig>().caches[0];
    let cruise = Vec3::new(cache.x, 90., cache.z);
    let target = if (pos - cache).with_y(0.).length() < 85. {
        cache + Vec3::Y * 20.
    } else {
        navigation::next_point(
            world.resource::<WorldGeometry>(),
            pos,
            cruise,
            Vec3::splat(22.),
        )
        .expect("northwest keyboard route must be traversable")
    };
    for key in routes::keys(pos, &flight, target) {
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
    }
    false
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
    let index = fixture.index;
    let step = fixture.steps[index].clone();
    assert!(
        elapsed < fixture.seconds,
        "exploration fixture timeout at step {index} {step:?}, phase={:?}, flight={:?}",
        world.resource::<GamePhase>(),
        fixture.flight_started
    );
    if matches!(step, Step::FlyNorthwest) {
        if fly_northwest(world, elapsed) {
            let mut fixture = world.resource_mut::<Fixture>();
            fixture.index += 1;
            fixture.next_at = elapsed + 0.35;
        }
        return;
    }
    if matches!(step, Step::SkipChoices) {
        if *world.resource::<GamePhase>() == GamePhase::Choosing {
            if elapsed >= world.resource::<Fixture>().next_at {
                world
                    .resource_mut::<ButtonInput<KeyCode>>()
                    .press(KeyCode::Backspace);
                let mut fixture = world.resource_mut::<Fixture>();
                fixture.choice_skips += 1;
                fixture.next_at = elapsed + 0.5;
                println!(
                    "EXPLORATION CHOICE: keyboard Backspace Skip, total XP={}",
                    world.resource::<UpgradeRun>().total_xp
                );
            }
            return;
        }
        assert_eq!(*world.resource::<GamePhase>(), GamePhase::Playing);
        assert!(
            world.resource::<UpgradeRun>().selected.is_empty(),
            "fixture choices must be skipped"
        );
    } else if elapsed < world.resource::<Fixture>().next_at {
        return;
    }
    match step {
        Step::Key(key) => world.resource_mut::<ButtonInput<KeyCode>>().press(key),
        Step::Phase(phase) => assert_eq!(*world.resource::<GamePhase>(), phase),
        Step::Select(index) => select(world, index),
        Step::Profile(act) => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Briefing);
            assert_eq!(
                world.resource::<MissionSession>().selected_mission.index() / 4,
                act
            );
            let profile =
                crate::world::regions::RegionProfile::for_mission(MissionId::ALL[act * 4]);
            assert_eq!(
                (
                    profile.salvage_chance,
                    profile.cache_components,
                    profile.charger_capacity
                ),
                [(50, 1, 200.), (25, 2, 200.), (25, 1, 300.)][act]
            );
            println!(
                "EXPLORATION PROFILE act={} briefing={}",
                act + 1,
                profile.summary().replace('\n', " / ")
            );
        }
        Step::RuntimeProfile(act) => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Playing);
            assert_eq!(
                world
                    .resource::<MissionSession>()
                    .active_mission
                    .unwrap()
                    .index()
                    / 4,
                act
            );
            let (chance, components) = {
                let economy = world.resource::<EconomyConfig>();
                (economy.chaser.chance_percent, economy.cache_components)
            };
            assert_eq!(chance, [50, 25, 25][act]);
            assert_eq!(components, [1, 2, 1][act]);
            assert_eq!(
                world.resource::<ChargerConfig>().capacity,
                [200., 200., 300.][act]
            );
            let mut sites = world.query::<(&DiscoverySite, &Pickup)>();
            assert_eq!(
                sites
                    .iter(world)
                    .filter(|(_, pickup)| pickup.radius == 50.)
                    .count(),
                3
            );
            println!(
                "EXPLORATION LAUNCH act={} chaser={}%, cache={} components, charger={:.0} energy",
                act + 1,
                chance,
                components,
                world.resource::<ChargerConfig>().capacity
            );
        }
        Step::Capture(label, expected) => capture(world, label, expected),
        Step::Feedback(expected) => assert!(
            world
                .resource::<MissionSession>()
                .purchase_feedback
                .contains(expected),
            "expected purchase feedback {expected:?}, got {:?}",
            world.resource::<MissionSession>().purchase_feedback
        ),
        Step::Battery(active) => {
            let secrets = world.resource::<Campaign>().secrets;
            assert!(secrets.blueprint);
            assert_eq!(secrets.reserve_battery, active);
            println!("EXPLORATION BATTERY blueprint=true active={active}");
        }
        Step::Balance(salvage, components) => {
            assert_eq!(
                world.resource::<Campaign>().wallet,
                Amounts {
                    salvage,
                    components
                }
            );
            println!(
                "EXPLORATION SHOP DEBIT verified balance={salvage} salvage/{components} components"
            );
        }
        Step::Capacity(capacity) => {
            assert_eq!(world.resource::<EnergyConfig>().capacity, capacity);
            println!("EXPLORATION CAPACITY actual={capacity:.0}");
        }
        Step::Fund => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Passives);
            world.resource_mut::<Campaign>().wallet = Amounts {
                salvage: 30,
                components: 3,
            };
            println!(
                "EXPLORATION OVERRIDE: synthetic wallet 30 salvage/3 components to exercise actual B shop debit (20/1); no earned-balance claim."
            );
        }
        Step::CompletePrior(act) => {
            let start = (act - 1) * 4;
            for index in start..start + 4 {
                world
                    .resource_mut::<Campaign>()
                    .progress
                    .complete(MissionId::ALL[index]);
            }
            println!(
                "EXPLORATION OVERRIDE: synthetic prior Act {act} victories unlock next introduction; campaign count={} (not earned attempts)",
                world.resource::<Campaign>().progress.count()
            );
        }
        Step::MoveToCache(site) => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Playing);
            let cache = world.resource::<EconomyConfig>().caches[site];
            move_to(world, cache + Vec3::Y * 20.);
            println!(
                "EXPLORATION OVERRIDE: synthetic position at cache {site} for later-act native proximity, LOS, XP and presentation; NW was physical."
            );
        }
        Step::Cache(site, components) => {
            assert!(!cache_exists(world, site), "cache {site} not collected");
            assert_eq!(
                world.resource::<AttemptResources>().collected.components,
                components
            );
            assert!(world.resource::<UpgradeRun>().total_xp >= 30);
            if site == 0 {
                assert!(world.resource::<Campaign>().secrets.blueprint);
            }
            if site == 1 {
                assert_eq!(
                    world.resource::<Campaign>().progress.routes(),
                    [false, false, true]
                );
                assert!(
                    world
                        .resource::<Campaign>()
                        .progress
                        .unlocked(MissionId::ALL[11])
                );
                assert!(
                    !world
                        .resource::<Campaign>()
                        .progress
                        .completed(MissionId::ALL[9])
                );
                assert!(
                    !world
                        .resource::<Campaign>()
                        .progress
                        .completed(MissionId::ALL[10])
                );
            }
            println!(
                "EXPLORATION CACHE site={site} components={components} XP={} notice={:?}",
                world.resource::<UpgradeRun>().total_xp,
                world.resource::<DiscoveryNotice>().text
            );
        }
        Step::SkipChoices => (),
        Step::Deadline => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Playing);
            world.resource_mut::<Encounter>().elapsed = 301.;
            println!(
                "EXPLORATION OVERRIDE: synthetic elapsed=301s to settle survival mission success without five-minute native wait."
            );
        }
        Step::Restart => {
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            println!(
                "EXPLORATION RESTART: real R key; purchased bonus should be forfeited before new baseline."
            );
        }
        Step::Die => {
            assert_eq!(*world.resource::<GamePhase>(), GamePhase::Playing);
            world.resource_mut::<PlayerHealth>().current = 0;
            println!(
                "EXPLORATION OVERRIDE: synthetic fatal hull=0 to settle native defeat; no combat difficulty claim."
            );
        }
        Step::Result(won, mission) => {
            let session = world.resource::<MissionSession>();
            let result = session.result.as_ref().expect("native mission result");
            assert_eq!(result.succeeded, won);
            assert_eq!(result.mission, MissionId::ALL[mission]);
            assert_eq!(result.rewards.bonus.salvage, if won { 10 } else { 0 });
            println!(
                "EXPLORATION RESULT mission={:02} success={won} banked={:?} wallet={:?}",
                mission + 1,
                result.rewards.credited,
                result.rewards.balance
            );
        }
        Step::Finish => {
            let campaign = world.resource::<Campaign>();
            assert_eq!(campaign.progress.routes(), [false, false, true]);
            assert_eq!(campaign.progress.count(), 8);
            assert_eq!(campaign.history.len(), 6);
            assert!(
                world.resource::<Fixture>().choice_skips > 0,
                "earned cache XP must reach a real Skip choice"
            );
            assert_eq!(
                world.resource::<MissionSession>().selected_mission,
                MissionId::ALL[11]
            );
            println!(
                "EXPLORATION FIXTURE PASS: three region briefings and launch profiles, keyboard NW cache, unlocked/paid/retained/forfeited reserve battery, actual B and Skip keys, central and SE caches, defeat-persistent secret finale selection; skips={}. Capture size={}. No user save access.",
                world.resource::<Fixture>().choice_skips,
                if world.resource::<Fixture>().minimum {
                    "640x480"
                } else {
                    "1120x720"
                }
            );
            world.write_message(AppExit::Success);
        }
        Step::FlyNorthwest => unreachable!(),
    }
    let mut fixture = world.resource_mut::<Fixture>();
    fixture.index += 1;
    fixture.next_at = elapsed + 0.35;
}

fn capture(world: &mut World, label: &str, expected: &str) {
    let window = world.query::<&Window>().single(world).unwrap();
    let scale = window.scale_factor();
    let viewport = Vec2::new(window.width(), window.height());
    let mut visible = 0;
    let mut matched = false;
    for (text, node, transform) in world
        .query::<(&Text, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
    {
        if node.size().min_element() < 1. {
            continue;
        }
        visible += 1;
        matched |= text.0.contains(expected);
        let center = transform.translation / scale;
        let half = node.size() / scale / 2.;
        assert!(
            (center - half).cmpge(Vec2::splat(-1.)).all()
                && (center + half).cmple(viewport + Vec2::ONE).all(),
            "EXPLORATION {label}: text outside viewport: {:?}",
            text.0
        );
    }
    assert!(
        visible > 3 && matched,
        "EXPLORATION {label}: expected visible text {expected:?}, labels={visible}"
    );
    println!(
        "EXPLORATION CAPTURE {label}: phase={:?}, expected={expected:?}, visible_text_bounds={visible}, viewport={}x{}",
        world.resource::<GamePhase>(),
        viewport.x,
        viewport.y
    );
    let fixture = world.resource::<Fixture>();
    let path = fixture.directory.as_ref().map(|directory| {
        let size = if fixture.minimum {
            "640x480"
        } else {
            "1120x720"
        };
        directory.join(format!("{size}-{label}.png"))
    });
    if let Some(path) = path {
        world
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}
