use super::*;

fn rich() -> Amounts {
    Amounts {
        salvage: u64::MAX,
        components: u64::MAX,
    }
}

#[test]
fn prerequisite_and_exact_price_are_atomic_and_unlock_at_first_rank() {
    let mut tree = PassiveTree::default();
    let mut wallet = Amounts {
        salvage: 10,
        components: 0,
    };
    assert_eq!(
        tree.purchase(NodeId::FireRate, &mut wallet),
        Err(PurchaseError::Locked(NodeId::Damage))
    );
    assert_eq!(wallet.salvage, 10);
    assert_eq!(tree.rank(NodeId::FireRate), 0);
    assert_eq!(tree.purchase(NodeId::Damage, &mut wallet), Ok(1));
    assert_eq!(wallet, Amounts::default());
    wallet.salvage = 10;
    assert_eq!(tree.purchase(NodeId::FireRate, &mut wallet), Ok(1));
    assert_eq!(tree.rank(NodeId::Damage), 1);
    assert_eq!(tree.rank(NodeId::Range), 0);
}

#[test]
fn each_rank_spends_once_and_cap_rejects_without_mutation() {
    let mut tree = PassiveTree::default();
    let mut wallet = rich();
    for node in NodeId::ALL {
        for expected in 1..=5 {
            let before = wallet;
            let cost = tree.next_cost(node).unwrap();
            assert_eq!(tree.purchase(node, &mut wallet), Ok(expected));
            assert_eq!(before.salvage - wallet.salvage, cost.salvage);
            assert_eq!(before.components - wallet.components, cost.components);
        }
        let before = (tree.clone(), wallet);
        assert_eq!(tree.purchase(node, &mut wallet), Err(PurchaseError::Maxed));
        assert_eq!((tree.clone(), wallet), before);
        assert_eq!(tree.next_cost(node), None);
    }
}

#[test]
fn either_missing_currency_preserves_all_ranks_and_balances() {
    let mut tree = PassiveTree::default();
    let mut wallet = rich();
    tree.purchase(NodeId::Battery, &mut wallet).unwrap();
    tree.purchase(NodeId::Battery, &mut wallet).unwrap();
    let cost = tree.next_cost(NodeId::Battery).unwrap();
    for mut wallet in [
        Amounts {
            salvage: cost.salvage - 1,
            components: cost.components,
        },
        Amounts {
            salvage: cost.salvage,
            components: cost.components - 1,
        },
    ] {
        let before = (tree.clone(), wallet);
        assert_eq!(
            tree.purchase(NodeId::Battery, &mut wallet),
            Err(PurchaseError::InsufficientFunds)
        );
        assert_eq!((tree.clone(), wallet), before);
    }
}

#[test]
fn independent_branches_and_rank_bonuses_do_not_compound() {
    let mut tree = PassiveTree::default();
    let mut wallet = rich();
    for rank in 1..=5 {
        tree.purchase(NodeId::Battery, &mut wallet).unwrap();
        assert_eq!(tree.bonus_percent(NodeId::Battery), rank * 20);
        assert_eq!(tree.rank(NodeId::Damage), 0);
        assert_eq!(tree.rank(NodeId::Hull), 0);
    }
    assert_eq!(tree.availability(NodeId::Reserve, &wallet), Ok(()));
    assert_eq!(
        tree.availability(NodeId::Charging, &wallet),
        Err(PurchaseError::Locked(NodeId::Reserve))
    );
}
