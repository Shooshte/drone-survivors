use super::*;
fn amounts(salvage: u64, components: u64) -> Amounts {
    Amounts {
        salvage,
        components,
    }
}

#[test]
fn success_and_failure_bank_the_agreed_example_into_existing_wallet() {
    for (success, credit, lost, bonus) in [
        (true, amounts(29, 4), amounts(0, 0), amounts(10, 1)),
        (false, amounts(4, 0), amounts(15, 3), amounts(0, 0)),
    ] {
        let mut wallet = amounts(30, 6);
        let receipt = RewardReceipt::settle(amounts(19, 3), success, &mut wallet);
        assert_eq!(receipt.collected, amounts(19, 3));
        assert_eq!(receipt.credited, credit);
        assert_eq!(receipt.lost, lost);
        assert_eq!(receipt.bonus, bonus);
        assert_eq!(wallet, amounts(30 + credit.salvage, 6 + credit.components));
        assert_eq!(receipt.balance, wallet);
        assert_eq!(receipt.error, None);
    }
}
#[test]
fn failure_rounds_each_resource_independently_at_quarter_boundaries() {
    for n in 0..12 {
        let mut wallet = Amounts::default();
        let receipt = RewardReceipt::settle(amounts(n, 11 - n), false, &mut wallet);
        assert_eq!(wallet, amounts(n / 4, (11 - n) / 4));
        assert_eq!(receipt.lost, amounts(n - n / 4, 11 - n - (11 - n) / 4));
    }
    let receipt = RewardReceipt::settle(Amounts::default(), true, &mut Amounts::default());
    assert_eq!(receipt.credited, amounts(10, 1));
}
#[test]
fn spending_checks_both_currencies_before_changing_either() {
    let original = amounts(7, 3);
    for cost in [amounts(8, 1), amounts(1, 4), amounts(u64::MAX, u64::MAX)] {
        let mut wallet = original;
        assert_eq!(
            wallet.try_spend(cost),
            Err(TransactionError::InsufficientFunds)
        );
        assert_eq!(wallet, original);
    }
    let mut wallet = original;
    assert_eq!(wallet.try_spend(original), Ok(()));
    assert_eq!(wallet, Amounts::default());
    assert_eq!(wallet.try_spend(Amounts::default()), Ok(()));
}
#[test]
fn credits_and_reward_overflow_are_atomic_and_visible() {
    for original in [amounts(u64::MAX, 4), amounts(4, u64::MAX)] {
        let mut wallet = original;
        assert_eq!(
            wallet.try_credit(amounts(1, 1)),
            Err(TransactionError::Overflow)
        );
        assert_eq!(wallet, original);
        let receipt = RewardReceipt::settle(amounts(19, 3), true, &mut wallet);
        assert_eq!(receipt.error, Some(TransactionError::Overflow));
        assert_eq!(receipt.credited, Amounts::default());
        assert_eq!(receipt.collected, amounts(19, 3));
        assert_eq!(wallet, original);
        assert_eq!(receipt.balance, original);
    }
    let mut wallet = Amounts::default();
    let receipt = RewardReceipt::settle(amounts(u64::MAX, 0), true, &mut wallet);
    assert_eq!(receipt.error, Some(TransactionError::Overflow));
    assert_eq!(wallet, Amounts::default());
    assert_eq!(wallet.try_credit(amounts(u64::MAX, u64::MAX)), Ok(()));
    assert_eq!(wallet, amounts(u64::MAX, u64::MAX));
}
