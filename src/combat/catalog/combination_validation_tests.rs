use super::*;

fn fixture_app(phase: GamePhase, active: f64, captures: usize) -> App {
    let mut app = App::new();
    app.insert_resource(Fixture {
        start: Instant::now(),
        next: 0.,
        stage: 6,
        choice: 4,
        seen: SCENARIOS.last().unwrap().kinds.to_vec(),
        captures,
        directory: None,
    })
    .insert_resource(CatalogArena {
        scenario: SCENARIOS.len() - 1,
        slots: SLOTS,
        selecting: false,
        duration: 60.,
        upgrades_page: false,
        preview: CARDS.map(Some),
    })
    .insert_resource(phase)
    .insert_resource(Encounter {
        elapsed: active,
        ..default()
    })
    .insert_resource(PlayerHealth {
        current: 100,
        invulnerable_until: 0.,
    })
    .init_resource::<ButtonInput<KeyCode>>()
    .init_resource::<UpgradeRun>()
    .init_resource::<Modules>()
    .init_resource::<Environment>()
    .add_message::<AppExit>()
    .add_systems(Update, drive);
    app.world_mut()
        .spawn((Drone, Transform::default(), DroneFlight::default()));
    app.world_mut().spawn(Window::default());
    app
}

#[test]
fn combination_fixture_rejects_premature_terminal_states_after_all_kinds_are_seen() {
    for phase in [GamePhase::Dead, GamePhase::Survived] {
        for (active, captures) in [(2., 0), (5., 1), (15., 2)] {
            let mut app = fixture_app(phase, active, captures);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.update()));
            assert!(
                result.is_err(),
                "{phase:?} at {active}s silently bypassed required captures"
            );
            assert_eq!(app.world().resource::<Fixture>().captures, captures);
            assert_eq!(app.world().resource::<Fixture>().stage, 6);
        }
    }
}

#[test]
fn combination_fixture_requires_all_checkpoints_before_lifecycle_validation() {
    let mut app = fixture_app(GamePhase::Playing, 2., 0);
    for (active, captures, stage) in [
        (2., 0, 6),
        (3., 1, 6),
        (9., 1, 6),
        (10., 2, 6),
        (19., 2, 6),
        (20., 3, 7),
    ] {
        app.world_mut().resource_mut::<Encounter>().elapsed = active;
        app.world_mut().resource_mut::<Fixture>().next = 0.;
        app.update();
        assert_eq!(app.world().resource::<Fixture>().captures, captures);
        assert_eq!(app.world().resource::<Fixture>().stage, stage);
    }
    app.world_mut().resource_mut::<Fixture>().next = 0.;
    app.update();
    assert_eq!(app.world().resource::<Fixture>().stage, 8);
    assert!(
        app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::KeyR)
    );
}

#[test]
fn combination_fixture_rejects_incomplete_capture_count_at_lifecycle_entry() {
    let mut app = fixture_app(GamePhase::Playing, 20., 2);
    app.world_mut().resource_mut::<Fixture>().stage = 7;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.update()));
    assert!(
        result.is_err(),
        "incomplete captures reached restart validation"
    );
    assert!(
        !app.world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::KeyR)
    );
}
