use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn main_menu() -> InlineKeyboardMarkup {
    let row1 = vec![
        InlineKeyboardButton::callback("📢 Duyurular", "announcements"),
        InlineKeyboardButton::callback("📚 Program", "schedule"),
    ];
    let row2 = vec![
        InlineKeyboardButton::callback("📅 Takvim", "calendar"),
        InlineKeyboardButton::callback("🍽 Menü", "cafeteria"),
    ];
    let row3 = vec![
        InlineKeyboardButton::callback("🎓 Bölümüm", "department"),
        InlineKeyboardButton::callback("📖 Magnum Notları", "magnum_menu"),
    ];
    let row4 = vec![
        InlineKeyboardButton::callback("🔍 Not / Sınav Ara", "search_prompt"),
        InlineKeyboardButton::callback("📤 Not Yükle (+2)", "upload_help"),
    ];
    let row5 = vec![
        InlineKeyboardButton::callback("📋 Haftalık Menü", "cafeteria_week"),
        InlineKeyboardButton::callback("🌤️ Hava Durumu", "weather"),
    ];
    let row6 = vec![
        InlineKeyboardButton::callback("🌍 Son Depremler", "earthquake"),
        InlineKeyboardButton::callback("ℹ️ Hakkında", "about"),
    ];

    InlineKeyboardMarkup::new(vec![row1, row2, row3, row4, row5, row6])
}

pub fn menu_day_row() -> InlineKeyboardMarkup {
    let row1 = vec![
        InlineKeyboardButton::callback("Dün", "cafeteria_yesterday"),
        InlineKeyboardButton::callback("Bugün", "cafeteria"),
        InlineKeyboardButton::callback("Yarın", "cafeteria_tomorrow"),
    ];
    let row2 = vec![
        InlineKeyboardButton::callback("📋 Hafta", "cafeteria_week"),
        InlineKeyboardButton::callback("🔗 Rezervasyon", "cafeteria_res"),
        InlineKeyboardButton::callback("📍 Konum", "cafeteria_loc"),
    ];
    InlineKeyboardMarkup::new(vec![row1, row2])
}
