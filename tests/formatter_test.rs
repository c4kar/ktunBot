use ktunbot::formatter::{format_announcement, format_menu, get_food_emoji};
use ktunbot::services::announcements::Announcement;
use ktunbot::services::cafeteria::MenuItem;

#[test]
fn test_get_food_emoji() {
    assert_eq!(get_food_emoji("Mercimek Çorbası"), "🍲");
    assert_eq!(get_food_emoji("Köfte Ekmek"), "🍖");
    assert_eq!(get_food_emoji("Et Döner"), "🍖");
    assert_eq!(get_food_emoji("Pirinç Pilavı"), "🍚");
    assert_eq!(get_food_emoji("Bilinmeyen"), "•");
}

#[test]
fn test_format_announcement_escape() {
    let ann = Announcement {
        title: "Test <Title> & More".to_string(),
        date: "01.01.2026".to_string(),
        link: "http://example.com/test".to_string(),
    };
    let formatted = format_announcement(&ann);
    assert!(formatted.contains("Test &lt;Title&gt; &amp; More"));
}

#[test]
fn test_format_menu_closed() {
    let item = MenuItem {
        date: "06 Aralık".to_string(),
        day_of_week: "Cumartesi".to_string(),
        meal_type: "".to_string(),
        foods: vec![],
        totalcalorie: "".to_string(),
        closed: true,
        closed_reason: "Hafta sonu".to_string(),
    };
    let formatted = format_menu(&item, "06 Aralık");
    assert!(formatted.contains("Yemekhane Kapalı"));
    assert!(formatted.contains("Hafta sonu"));
}

#[test]
fn test_format_menu_open() {
    let item = MenuItem {
        date: "01 Aralık".to_string(),
        day_of_week: "Pazartesi".to_string(),
        meal_type: "öğle".to_string(),
        foods: vec!["Mercimek Çorbası".to_string(), "Et Döner".to_string()],
        totalcalorie: "1100 KKAL".to_string(),
        closed: false,
        closed_reason: "".to_string(),
    };
    let formatted = format_menu(&item, "01 Aralık");
    assert!(formatted.contains("Öğle Yemeği"));
    assert!(formatted.contains("🍲 Mercimek Çorbası"));
    assert!(formatted.contains("🍖 Et Döner"));
    assert!(formatted.contains("1100 KKAL"));
}
