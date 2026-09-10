use super::*;
use crate::modules::{Loadout, ModuleKind};

fn choice_ready() -> UpgradeRun {
    let mut run = UpgradeRun::default();
    run.award(50);
    run
}

#[test]
fn thresholds_carry_excess_and_queue_every_crossed_level() {
    let mut run = UpgradeRun::default();
    assert_eq!(run.threshold(), 50);

    run.award(49);
    assert_eq!((run.level, run.xp, run.pending), (1, 49, 0));
    run.award(1);
    assert_eq!((run.level, run.xp, run.pending), (2, 0, 1));
    assert_eq!(run.threshold(), 75);

    run.award(140);
    assert_eq!((run.level, run.xp, run.pending), (3, 65, 2));
}

#[test]
fn one_large_award_crosses_multiple_thresholds() {
    let mut run = UpgradeRun::default();
    run.award(140);
    assert_eq!((run.level, run.xp, run.pending), (3, 15, 2));
}

#[test]
fn maximum_awards_use_bounded_arithmetic_and_keep_valid_remainders() {
    let mut run = UpgradeRun::default();
    run.award(u32::MAX);

    assert_eq!(run.level, 18_535);
    assert_eq!(run.pending, 18_534);
    assert_eq!(run.xp, 407_820);
    assert!(run.xp < run.threshold());
}

#[test]
fn a_nonempty_offer_is_stable_and_contains_distinct_eligible_choices() {
    let mut run = choice_ready();
    let loadout = Loadout::new([Some(ModuleKind::Overdrive), None, None, None]).unwrap();
    run.prepare_offer(&loadout);
    let first = run.offer.clone();

    assert_eq!(first.len(), 3);
    assert!(!first.contains(&UpgradeKind::RapidShield));
    assert!(!first.contains(&UpgradeKind::WideAreaRockets));
    assert!(
        first
            .iter()
            .enumerate()
            .all(|(index, kind)| !first[..index].contains(kind))
    );

    run.prepare_offer(&Loadout::default());
    assert_eq!(run.offer, first);
}

#[test]
fn prerequisites_depend_on_equipment_not_module_position_or_power() {
    let rocket = Loadout::new([None, Some(ModuleKind::Rocket), None, None]).unwrap();
    let mut run = choice_ready();
    run.selected = vec![
        UpgradeKind::Interceptor,
        UpgradeKind::AgileFrame,
        UpgradeKind::HeavyArmor,
        UpgradeKind::HeavyRounds,
    ];
    run.prepare_offer(&rocket);

    assert_eq!(run.offer, vec![UpgradeKind::WideAreaRockets]);
}

#[test]
fn selecting_or_skipping_consumes_exactly_one_pending_choice() {
    let mut selected = UpgradeRun::default();
    selected.award(140);
    selected.prepare_offer(&Loadout::default());
    let picked = selected.offer[1];
    assert!(selected.resolve(Some(1)));
    assert_eq!(selected.selected, vec![picked]);
    assert_eq!(selected.pending, 1);
    assert!(selected.offer.is_empty());

    selected.prepare_offer(&Loadout::default());
    let skipped_offer = selected.offer.clone();
    assert!(selected.resolve(None));
    assert_eq!(selected.selected, vec![picked]);
    assert_eq!(selected.pending, 0);
    assert!(selected.offer.is_empty());
    assert!(
        skipped_offer
            .iter()
            .all(|kind| !selected.selected.contains(kind) || *kind == picked)
    );
}

#[test]
fn invalid_selection_preserves_the_current_offer_and_pending_choice() {
    let mut run = choice_ready();
    run.prepare_offer(&Loadout::default());
    let offer = run.offer.clone();

    assert!(!run.resolve(Some(offer.len())));
    assert_eq!(run.offer, offer);
    assert_eq!(run.pending, 1);
    assert!(run.selected.is_empty());
}

#[test]
fn depleted_pools_clear_queued_dialogs_but_future_xp_still_levels() {
    let mut run = UpgradeRun::default();
    run.award(275);
    run.selected = UpgradeKind::ALL.to_vec();
    run.prepare_offer(&Loadout::default());

    assert!(run.exhausted);
    assert_eq!(run.pending, 0);
    assert!(run.offer.is_empty());
    let before_level = run.level;
    let before_xp = run.xp;
    run.award(run.threshold());
    assert_eq!(run.level, before_level + 1);
    assert_eq!(run.xp, before_xp);
    assert_eq!(run.pending, 0);
}

#[test]
fn fixed_seed_sampling_repeats_the_same_offer_sequence() {
    let mut left = UpgradeRun::default();
    let mut right = UpgradeRun::default();
    left.award(140);
    right.award(140);

    for _ in 0..2 {
        left.prepare_offer(&Loadout::default());
        right.prepare_offer(&Loadout::default());
        assert_eq!(left.offer, right.offer);
        assert!(left.resolve(None));
        assert!(right.resolve(None));
    }
}

#[test]
fn modifiers_compose_from_baselines_independent_of_selection_order() {
    let selected = [
        UpgradeKind::Interceptor,
        UpgradeKind::AgileFrame,
        UpgradeKind::HeavyArmor,
        UpgradeKind::HeavyRounds,
        UpgradeKind::RapidShield,
        UpgradeKind::WideAreaRockets,
    ];
    let forward = UpgradeModifiers::from_selected(&selected);
    let reverse = UpgradeModifiers::from_selected(&selected.into_iter().rev().collect::<Vec<_>>());

    assert_eq!(forward, reverse);
    assert_eq!(forward.horizontal_acceleration, 1.3);
    assert_eq!(forward.speed, 1.3);
    assert_eq!(forward.handling, 1.4);
    assert!((forward.acceleration - 0.675).abs() < f32::EPSILON);
    assert_eq!(forward.hull_delta, 30);
    assert_eq!(forward.capacity, 0.75);
    assert_eq!(forward.shot_damage, 2);
    assert_eq!(forward.shield_recharge, 0.5);
    assert_eq!(forward.shield_drain, 1.5);
    assert_eq!(forward.rocket_radius, 1.5);
    assert_eq!(forward.rocket_interval, 1.5);
}

#[test]
fn final_selection_marks_build_complete_without_another_level() {
    let mut run = UpgradeRun {
        selected: UpgradeKind::ALL[..5].to_vec(),
        ..default()
    };
    run.award(50);
    run.prepare_offer(&Loadout::default());
    assert_eq!(run.offer, vec![UpgradeKind::WideAreaRockets]);
    assert!(run.resolve(Some(0)));
    assert_eq!(run.pending, 0);
    run.prepare_offer(&Loadout::default());
    assert!(run.exhausted);
    assert!(run.offer.is_empty());
    run.award(run.threshold());
    assert_eq!(run.pending, 0);
}
