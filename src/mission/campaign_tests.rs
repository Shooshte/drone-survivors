use super::{
    campaign::MissionId,
    tests::{app, launch, tick},
    *,
};

fn win(app: &mut App) {
    if !app
        .world()
        .resource::<super::objectives::ObjectiveRun>()
        .survival()
    {
        super::objective_tests::finish_objective(app);
        return;
    }
    app.world_mut().resource_mut::<Encounter>().elapsed = 300.;
    tick(app, 0., &[]);
}
fn hub(app: &mut App) {
    tick(app, 0., &[]);
    tick(app, 0., &[KeyCode::Enter]);
}

#[test]
fn campaign_unlocks_both_branch_orders_and_requires_all_twelve_distinct_wins() {
    for reverse in [false, true] {
        let mut app = app();
        for act in 0..3 {
            let base = act * 4;
            for offset in if reverse { [0, 2, 1, 3] } else { [0, 1, 2, 3] } {
                let id = MissionId::ALL[base + offset];
                let campaign = app.world().resource::<Campaign>();
                assert!(campaign.progress.unlocked(id));
                if offset == 0 {
                    assert!(!campaign.progress.unlocked(MissionId::ALL[base + 1]));
                    assert!(!campaign.progress.unlocked(MissionId::ALL[base + 2]));
                }
                if offset != 3 {
                    assert!(!campaign.progress.unlocked(MissionId::ALL[base + 3]));
                    if act < 2 {
                        assert!(!campaign.progress.unlocked(MissionId::ALL[base + 4]));
                    }
                }
                app.world_mut()
                    .resource_mut::<MissionSession>()
                    .selected_mission = id;
                launch(&mut app);
                win(&mut app);
                assert_eq!(
                    app.world()
                        .resource::<MissionSession>()
                        .result
                        .as_ref()
                        .unwrap()
                        .mission,
                    id
                );
                assert!(app.world().resource::<Campaign>().progress.completed(id));
                hub(&mut app);
            }
        }
        let campaign = app.world().resource::<Campaign>();
        assert_eq!(campaign.progress.count(), 12);
        assert!(campaign.progress.finished());
        assert_eq!(campaign.history.len(), 12);
        assert_eq!(
            campaign.wallet,
            crate::economy::Amounts {
                salvage: 120,
                components: 12
            }
        );
        for id in MissionId::ALL {
            assert!(campaign.progress.unlocked(id));
        }
    }
}

#[test]
fn locked_briefing_and_launch_reject_without_consuming_attempt_or_loadout() {
    let mut app = app();
    app.world_mut()
        .resource_mut::<MissionSession>()
        .selected_mission = MissionId::ALL[3];
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Briefing;
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    let session = app.world().resource::<MissionSession>();
    assert_eq!(session.next_attempt, 0);
    assert!(session.active_loadout.is_none());
    assert!(session.active_mission.is_none());
}

#[test]
fn restart_and_results_use_active_identity_and_replays_keep_progress_and_rewards() {
    let mut app = app();
    launch(&mut app);
    app.world_mut()
        .resource_mut::<MissionSession>()
        .selected_mission = MissionId::ALL[11];
    tick(&mut app, 0., &[KeyCode::KeyR]);
    win(&mut app);
    let campaign = app.world().resource::<Campaign>();
    assert_eq!(campaign.history.len(), 1);
    assert_eq!(campaign.history[0].mission, MissionId::ALL[0]);
    assert_eq!(campaign.history[0].attempt, 2);
    assert_eq!(campaign.progress.count(), 1);
    hub(&mut app);
    app.world_mut()
        .resource_mut::<MissionSession>()
        .selected_mission = MissionId::ALL[0];
    launch(&mut app);
    win(&mut app);
    tick(&mut app, 20., &[]);
    assert_eq!(app.world().resource::<Campaign>().progress.count(), 1);
    assert_eq!(app.world().resource::<Campaign>().wallet.salvage, 20);
    hub(&mut app);
    app.world_mut()
        .resource_mut::<MissionSession>()
        .selected_mission = MissionId::ALL[1];
    launch(&mut app);
    *app.world_mut().resource_mut::<GamePhase>() = GamePhase::Dead;
    tick(&mut app, 0., &[]);
    let campaign = app.world().resource::<Campaign>();
    assert_eq!(campaign.progress.count(), 1);
    assert!(!campaign.progress.completed(MissionId::ALL[1]));
    assert!(!campaign.progress.unlocked(MissionId::ALL[3]));
    assert_eq!(campaign.history[2].mission, MissionId::ALL[1]);
    assert_eq!(campaign.wallet.salvage, 20);
}

#[test]
fn selection_navigation_skips_locks_releases_input_and_freezes_gameplay() {
    let mut app = app();
    tick(&mut app, 0., &[KeyCode::KeyC]);
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::MissionSelect
    );
    tick(&mut app, 100., &[KeyCode::KeyC, KeyCode::Enter]);
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::MissionSelect
    );
    assert_eq!(app.world().resource::<Encounter>().elapsed, 0.);
    assert_eq!(app.world().resource::<Encounter>().next_burst, 0);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::ArrowRight]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[0]
    );
    app.world_mut()
        .resource_mut::<Campaign>()
        .progress
        .complete(MissionId::ALL[0]);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[1]
    );
    tick(&mut app, 0., &[KeyCode::ArrowDown]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[1]
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::ArrowRight]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[2]
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::ArrowRight]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[0]
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::ArrowLeft]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[2]
    );
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Backspace]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Hub);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[2]
    );
}

#[test]
fn locked_click_does_not_select_and_multiple_actions_choose_only_one() {
    let mut app = app();
    tick(&mut app, 0., &[KeyCode::KeyC]);
    tick(&mut app, 0., &[]);
    let button = app
        .world_mut()
        .spawn((
            MissionAction::SelectMission(MissionId::ALL[3]),
            Interaction::Pressed,
        ))
        .id();
    tick(&mut app, 0., &[]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[0]
    );
    assert!(
        app.world()
            .resource::<MissionSession>()
            .purchase_feedback
            .contains("02 + 03")
    );
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
    tick(&mut app, 0., &[]);
    app.world_mut()
        .resource_mut::<Campaign>()
        .progress
        .complete(MissionId::ALL[0]);
    *app.world_mut().get_mut::<MissionAction>(button).unwrap() =
        MissionAction::SelectMission(MissionId::ALL[2]);
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
    tick(&mut app, 0., &[]);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[2]
    );
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::MissionSelect
    );
    tick(&mut app, 0., &[KeyCode::Enter]);
    assert_eq!(
        *app.world().resource::<GamePhase>(),
        GamePhase::MissionSelect
    );
    *app.world_mut().get_mut::<Interaction>(button).unwrap() = Interaction::None;
    tick(&mut app, 0., &[]);
    tick(&mut app, 0., &[KeyCode::Enter, KeyCode::ArrowLeft]);
    assert_eq!(*app.world().resource::<GamePhase>(), GamePhase::Briefing);
    assert_eq!(
        app.world().resource::<MissionSession>().selected_mission,
        MissionId::ALL[2]
    );
}
