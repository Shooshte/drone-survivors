use super::{price, ModuleInventory, ShopError};
use crate::{economy::Amounts, modules::ModuleKind};

fn amounts(salvage: u64, components: u64) -> Amounts {
    Amounts {
        salvage,
        components,
    }
}

#[test]
fn catalog_prices_match_the_provisional_balance() {
    assert_eq!(price(ModuleKind::Overdrive), amounts(10, 0));
    assert_eq!(price(ModuleKind::Shield), amounts(15, 0));
    assert_eq!(price(ModuleKind::Mobility), amounts(15, 0));
    assert_eq!(price(ModuleKind::Rocket), amounts(25, 1));
}

#[test]
fn new_inventory_has_no_owned_modules_and_four_empty_slots() {
    let inventory = ModuleInventory::default();

    assert!(ModuleKind::ALL
        .into_iter()
        .all(|kind| !inventory.owns(kind)));
    assert_eq!(inventory.loadout().slots(), &[None, None, None, None]);
    assert_eq!(inventory.validate(), Ok(()));
}

#[test]
fn purchase_at_exact_funds_unlocks_without_equipping() {
    let mut inventory = ModuleInventory::default();
    let mut bank = amounts(10, 0);

    assert_eq!(inventory.purchase(ModuleKind::Overdrive, &mut bank), Ok(()));
    assert!(inventory.owns(ModuleKind::Overdrive));
    assert_eq!(bank, amounts(0, 0));
    assert_eq!(inventory.loadout().slots(), &[None, None, None, None]);
}

#[test]
fn unaffordable_purchase_is_atomic() {
    let mut inventory = ModuleInventory::default();
    let mut bank = amounts(25, 0);

    assert_eq!(
        inventory.purchase(ModuleKind::Rocket, &mut bank),
        Err(ShopError::InsufficientFunds)
    );
    assert!(!inventory.owns(ModuleKind::Rocket));
    assert_eq!(bank, amounts(25, 0));
}

#[test]
fn duplicate_purchase_preserves_balance() {
    let mut inventory = ModuleInventory::default();
    let mut bank = amounts(20, 0);
    inventory
        .purchase(ModuleKind::Overdrive, &mut bank)
        .unwrap();

    assert_eq!(
        inventory.purchase(ModuleKind::Overdrive, &mut bank),
        Err(ShopError::AlreadyOwned(ModuleKind::Overdrive))
    );
    assert_eq!(bank, amounts(10, 0));
}

#[test]
fn assignment_rejects_unowned_modules_and_invalid_slots() {
    let mut inventory = ModuleInventory::default();

    assert_eq!(
        inventory.assign(0, Some(ModuleKind::Shield)),
        Err(ShopError::NotOwned(ModuleKind::Shield))
    );
    assert_eq!(inventory.assign(4, None), Err(ShopError::InvalidSlot(4)));
}

#[test]
fn assigning_a_module_moves_it_and_replaces_the_destination() {
    let mut inventory = ModuleInventory::default();
    let mut bank = amounts(25, 0);
    inventory
        .purchase(ModuleKind::Overdrive, &mut bank)
        .unwrap();
    inventory.purchase(ModuleKind::Shield, &mut bank).unwrap();

    inventory.assign(0, Some(ModuleKind::Overdrive)).unwrap();
    inventory.assign(1, Some(ModuleKind::Shield)).unwrap();
    inventory.assign(1, Some(ModuleKind::Overdrive)).unwrap();

    assert_eq!(
        inventory.loadout().slots(),
        &[None, Some(ModuleKind::Overdrive), None, None]
    );
    assert!(inventory.owns(ModuleKind::Overdrive));
    assert!(inventory.owns(ModuleKind::Shield));
    assert_eq!(bank, amounts(0, 0));
    assert_eq!(inventory.validate(), Ok(()));
}

#[test]
fn removing_an_equipped_module_preserves_ownership() {
    let mut inventory = ModuleInventory::default();
    let mut bank = amounts(15, 0);
    inventory.purchase(ModuleKind::Shield, &mut bank).unwrap();
    inventory.assign(3, Some(ModuleKind::Shield)).unwrap();

    assert_eq!(inventory.assign(3, None), Ok(()));
    assert_eq!(inventory.loadout().slots(), &[None, None, None, None]);
    assert!(inventory.owns(ModuleKind::Shield));
}
