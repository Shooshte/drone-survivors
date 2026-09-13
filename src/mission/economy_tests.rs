use super::*;
use crate::economy::{Amounts, AttemptResources};

#[test]
fn terminal_attempts_bank_once_and_replays_pay_again() {
    let mut app = app();
    assert_eq!(
        app.world().resource::<Campaign>().wallet,
        Amounts::default()
    );
    for (won, expected) in [(true, (29, 4)), (false, (33, 4)), (true, (62, 8))] {
        launch(&mut app);
        app.world_mut().resource_mut::<AttemptResources>().collected = Amounts {
            salvage: 19,
            components: 3,
        };
        *app.world_mut().resource_mut::<GamePhase>() = if won {
            GamePhase::Survived
        } else {
            GamePhase::Dead
        };
        tick(&mut app, 0., &[]);
        let result = app
            .world()
            .resource::<MissionSession>()
            .result
            .clone()
            .unwrap();
        assert_eq!(
            result.rewards.balance,
            Amounts {
                salvage: expected.0,
                components: expected.1
            }
        );
        assert_eq!(
            app.world().resource::<Campaign>().wallet,
            result.rewards.balance
        );
        for _ in 0..3 {
            tick(&mut app, 10., &[KeyCode::KeyR]);
        }
        assert_eq!(
            app.world().resource::<MissionSession>().result.as_ref(),
            Some(&result)
        );
        assert_eq!(
            app.world().resource::<Campaign>().wallet,
            result.rewards.balance
        );
        tick(&mut app, 0., &[]);
        tick(&mut app, 0., &[KeyCode::Enter]);
    }
    assert_eq!(app.world().resource::<Campaign>().history.len(), 3);
}
#[test]
fn restart_discards_collection_without_changing_bank_or_recording_result() {
    for phase in [GamePhase::Playing, GamePhase::Choosing] {
        let mut app = app();
        launch(&mut app);
        let bank = Amounts {
            salvage: 7,
            components: 8,
        };
        app.world_mut().resource_mut::<Campaign>().wallet = bank;
        app.world_mut().resource_mut::<AttemptResources>().collected = Amounts {
            salvage: 19,
            components: 3,
        };
        *app.world_mut().resource_mut::<GamePhase>() = phase;
        tick(&mut app, 1., &[KeyCode::KeyR]);
        assert_eq!(
            app.world().resource::<AttemptResources>().collected,
            Amounts::default()
        );
        assert_eq!(app.world().resource::<Campaign>().wallet, bank);
        assert!(app.world().resource::<Campaign>().history.is_empty());
    }
}
