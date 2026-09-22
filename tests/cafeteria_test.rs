use ktunbot::services::cafeteria::MenuData;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_cafeteria_parse() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("data");
    path.push("menus");
    path.push("2025-12.json");

    let content = fs::read_to_string(&path).expect("failed to read 2025-12.json");
    let data: MenuData = serde_json::from_str(&content).expect("failed to parse JSON");

    assert_eq!(data.year, 2025);
    assert_eq!(data.month, "Aralık");

    let item_01 = data.menu.iter().find(|m| m.date == "01 Aralık").unwrap();
    assert!(!item_01.closed);
    assert!(item_01.foods.contains(&"Mercimek Çorbası".to_string()));

    let item_06 = data.menu.iter().find(|m| m.date == "06 Aralık").unwrap();
    assert!(item_06.closed);
    assert_eq!(item_06.closed_reason, "Hafta sonu");
}
