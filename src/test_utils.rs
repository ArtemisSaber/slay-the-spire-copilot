use serde_json::Value;

pub fn load_fixture(name: &str) -> Value {
    let path = format!("tests/fixtures/{name}");
    let content = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&content).unwrap()
}

pub fn card(id: &str, name: &str, cost: i64, card_type: &str) -> crate::state::CardInfo {
    crate::state::CardInfo {
        id: id.into(),
        name: name.into(),
        cost,
        card_type: card_type.into(),
        upgraded: false,
        uuid: None,
        description: String::new(),
    }
}
