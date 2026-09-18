use super::*;
use crate::upgrades::UpgradeKind;

fn contains_text(app: &mut App, needle: &str) -> bool {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .any(|text| text.0.contains(needle))
}

#[test]
fn catalog_preview_uses_four_normal_choices_and_replays_after_restart() {
    let mut app = app();
    step(&mut app, &[KeyCode::KeyU]);
    assert!(contains_text(&mut app, "UPGRADE PREVIEW"));
    for key in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ] {
        step(&mut app, &[key]);
    }
    assert!(contains_text(&mut app, "Interceptor"));
    assert!(contains_text(&mut app, "Agile frame"));
    step(&mut app, &[KeyCode::Enter]);
    step(&mut app, &[]);
    for (i, expected) in [
        UpgradeKind::Interceptor,
        UpgradeKind::AgileFrame,
        UpgradeKind::HeavyArmor,
        UpgradeKind::HeavyRounds,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Choosing);
        assert_eq!(app.world().resource::<UpgradeRun>().offer, vec![expected]);
        step(&mut app, &[]);
        step(
            &mut app,
            &[if i == 1 {
                KeyCode::Backspace
            } else {
                KeyCode::Digit1
            }],
        );
    }
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().resolved, 4);
    assert_eq!(app.world().resource::<UpgradeRun>().selected.len(), 3);
    assert!(app.world().resource::<UpgradeRun>().exhausted);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
    app.world_mut().resource_mut::<UpgradeRun>().award(u32::MAX);
    step(&mut app, &[]);
    assert!(app.world().resource::<UpgradeRun>().offer.is_empty());
    step(&mut app, &[KeyCode::KeyR]);
    step(&mut app, &[]);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
    assert_eq!(
        app.world().resource::<UpgradeRun>().offer,
        vec![UpgradeKind::Interceptor]
    );
    assert_eq!(
        app.world()
            .resource::<crate::arena::FlightConfig>()
            .max_horizontal_speed,
        crate::arena::FlightConfig::default().max_horizontal_speed
    );
    step(&mut app, &[KeyCode::Tab]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    assert!(app.world().resource::<UpgradeRun>().selected.is_empty());
    step(&mut app, &[KeyCode::Enter]);
    step(&mut app, &[]);
    assert_eq!(
        app.world().resource::<UpgradeRun>().offer,
        vec![UpgradeKind::Interceptor]
    );
}

#[test]
fn empty_catalog_preview_grants_no_xp_and_starts_playing() {
    let mut app = app();
    step(&mut app, &[KeyCode::KeyU]);
    step(&mut app, &[KeyCode::Enter]);
    step(&mut app, &[]);
    let run = app.world().resource::<UpgradeRun>();
    assert_eq!(run.total_xp, 0);
    assert!(run.selected.is_empty());
    assert!(run.offer.is_empty());
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Playing);
}

fn click(app: &mut App, action: catalog::CatalogAction) {
    let world = app.world_mut();
    let mut query = world.query::<(&catalog::CatalogAction, &mut Interaction)>();
    let mut found = false;
    for (candidate, mut interaction) in query.iter_mut(world) {
        *interaction = if *candidate == action {
            found = true;
            Interaction::Pressed
        } else {
            Interaction::None
        };
    }
    assert!(found, "missing action {action:?}");
    step(app, &[]);
}

#[test]
fn preview_clicks_and_keyboard_edit_the_same_fixed_slot() {
    let mut app = app();
    click(&mut app, catalog::CatalogAction::ToggleUpgrades);
    click(&mut app, catalog::CatalogAction::CycleUpgrade(3));
    assert!(contains_text(&mut app, "4   Interceptor\n"));
    assert!(contains_text(&mut app, "1   EMPTY — earn"));
    step(&mut app, &[KeyCode::Digit4]);
    assert!(contains_text(&mut app, "4   Agile frame\n"));
    click(&mut app, catalog::CatalogAction::Launch);
    step(&mut app, &[]);
    assert_eq!(
        app.world().resource::<UpgradeRun>().offer,
        vec![UpgradeKind::AgileFrame]
    );
    assert_eq!(app.world().resource::<UpgradeRun>().pending, 1);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 50);
}

#[test]
fn preview_only_cycles_eligible_cards_and_module_changes_remove_invalid_choices() {
    let mut app = app();
    for _ in 0..6 {
        step(&mut app, &[KeyCode::Digit1]);
    } // Repair
    for _ in 0..5 {
        step(&mut app, &[KeyCode::Digit2]);
    } // Repulsor
    step(&mut app, &[KeyCode::Digit3]); // Overdrive
    step(&mut app, &[KeyCode::Digit4]); // Shield (skip owned Overdrive)
    step(&mut app, &[KeyCode::KeyU]);
    let mut seen = Vec::new();
    for _ in 0..12 {
        step(&mut app, &[KeyCode::Digit1]);
        let text = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .find(|text| {
                text.0.starts_with("1   ")
                    && (text.0.contains("Benefit:") || text.0.contains("earn this"))
            })
            .unwrap()
            .0
            .clone();
        seen.push(text);
    }
    for name in [
        "Efficient coils",
        "Reserve battery",
        "Long-range rounds",
        "Hot overdrive",
        "Rapid repair",
        "Wide repulsor",
    ] {
        assert!(
            seen.iter().any(|text| text.contains(name)),
            "missing {name} in {seen:?}"
        );
    }
    assert!(!seen.iter().any(|text| text.contains("Wide-area rockets")));
    for _ in 0..12 {
        if contains_text(&mut app, "1   Rapid repair\n") {
            break;
        }
        step(&mut app, &[KeyCode::Digit1]);
    }
    assert!(contains_text(&mut app, "1   Rapid repair\n"));
    step(&mut app, &[KeyCode::KeyU]);
    step(&mut app, &[KeyCode::Digit1]); // Repair -> Empty
    step(&mut app, &[KeyCode::KeyU]);
    assert!(contains_text(&mut app, "1   EMPTY — earn"));
    step(&mut app, &[KeyCode::Enter]);
    step(&mut app, &[]);
    assert_eq!(app.world().resource::<UpgradeRun>().total_xp, 0);
}
