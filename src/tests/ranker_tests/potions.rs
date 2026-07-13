use super::*;

#[test]
fn potion_slot_preserves_raw_index_with_gaps() {
    let state = comm_mod_state("comm-gapped-potions.json");
    let potions = &state.potions;
    assert_eq!(potions.len(), 2, "should have 2 usable potions");

    let energy = potions.iter().find(|p| p.name == "能量药水").unwrap();
    assert_eq!(energy.slot, 1, "能量药水 should be at raw index 1");
    let block = potions.iter().find(|p| p.name == "格挡药水").unwrap();
    assert_eq!(block.slot, 2, "格挡药水 should be at raw index 2");
}

#[test]
fn potion_slot_not_zero_when_first_slot_empty() {
    let state = comm_mod_state("comm-gapped-potions.json");
    let energy = state.potions.iter().find(|p| p.name == "能量药水").unwrap();
    assert_eq!(
        energy.slot, 1,
        "energy potion slot should be 1, not 0 (index 0 is empty Potion Slot)"
    );
    let block = state.potions.iter().find(|p| p.name == "格挡药水").unwrap();
    assert_eq!(block.slot, 2, "block potion slot should be 2");
}

#[test]
fn empty_potion_slots_counted_correctly() {
    let state = comm_mod_state("comm-gapped-potions.json");
    assert_eq!(
        state.empty_potion_slots, 1,
        "one slot (index 0) should be counted as empty"
    );
}
