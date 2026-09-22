use crate::services::announcements::Announcement;
use crate::services::cafeteria::MenuItem;
use teloxide::utils::html::escape;

pub fn format_announcement(announcement: &Announcement) -> String {
    format!(
        "<a href='{}'>{}</a>\n📅 {}",
        announcement.link,
        escape(&announcement.title),
        announcement.date
    )
}

pub fn get_food_emoji(food_name: &str) -> &'static str {
    let food_lower = food_name.to_lowercase();

    // Soups
    if food_lower.contains("çorba") {
        return "🍲";
    }

    // Rice/Pilav
    if food_lower.contains("pilav") || food_lower.contains("pirinç") {
        return "🍚";
    }

    // Pasta
    if food_lower.contains("makarna")
        || food_lower.contains("spagetti")
        || food_lower.contains("erişte")
    {
        return "🍝";
    }

    // Meat dishes
    let meats = [
        "köfte", "kebap", "döner", "tavuk", "et", "piliç", "balık", "burger", "tantuni",
    ];
    if meats.iter().any(|&w| food_lower.contains(w)) {
        return "🍖";
    }

    // Vegetables/Legumes
    let vegs = [
        "fasulye",
        "nohut",
        "mercimek",
        "patlıcan",
        "biber",
        "dolma",
        "börek",
        "ispanak",
        "karnabahar",
    ];
    if vegs.iter().any(|&w| food_lower.contains(w)) {
        return "🥗";
    }

    // Desserts
    let desserts = [
        "tatlı",
        "helva",
        "revani",
        "pasta",
        "supangle",
        "aşure",
        "höşmerim",
        "sütlaç",
    ];
    if desserts.iter().any(|&w| food_lower.contains(w)) {
        return "🍰";
    }

    // Dairy/Yogurt
    let dairies = ["yoğurt", "cacık", "ayran"];
    if dairies.iter().any(|&w| food_lower.contains(w)) {
        return "🥛";
    }

    // Salad
    let salads = ["salata", "söğüş", "yeşillik"];
    if salads.iter().any(|&w| food_lower.contains(w)) {
        return "🥬";
    }

    // Fruits
    if food_lower.contains("meyve") {
        return "🍎";
    }

    // Beverages
    if food_lower.contains("içecek") {
        return "🥤";
    }

    "•"
}

pub fn format_menu(menu_data: &MenuItem, date_str: &str) -> String {
    let date_to_show = if !menu_data.date.is_empty() {
        &menu_data.date
    } else {
        date_str
    };

    if menu_data.closed {
        let reason = if menu_data.closed_reason.is_empty() {
            "Kapalı"
        } else {
            &menu_data.closed_reason
        };

        let mut text = format!("📅 <b>{}</b>", escape(date_to_show));
        if !menu_data.day_of_week.is_empty() {
            text.push_str(&format!(" ({})", escape(&menu_data.day_of_week)));
        }
        text.push_str("\n\n🚫 <b>Yemekhane Kapalı</b>\n");
        text.push_str(&format!("📝 <i>{}</i>", escape(reason)));
        return text;
    }

    let mut text = format!("📅 <b>{}</b>", escape(date_to_show));
    if !menu_data.day_of_week.is_empty() {
        text.push_str(&format!(" ({})", escape(&menu_data.day_of_week)));
    }
    text.push_str("\n\n");

    let meal_type = menu_data.meal_type.trim().to_lowercase();
    if !meal_type.is_empty() {
        let meal_emoji = if meal_type == "kahvaltı" {
            "🌅"
        } else if meal_type == "öğle" {
            "🍽"
        } else {
            "🌙"
        };
        // Poor man's capitalize
        let mut chars = meal_type.chars();
        let cap_meal_type = match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().chain(chars).collect(),
        };
        text.push_str(&format!(
            "{} <b>{} Yemeği</b>\n\n",
            meal_emoji,
            escape(&cap_meal_type)
        ));
    }

    for food in &menu_data.foods {
        let emoji = get_food_emoji(food);
        text.push_str(&format!("{} {}\n", emoji, escape(food)));
    }

    if !menu_data.foods.is_empty() {
        text.push('\n');
    }

    if !menu_data.totalcalorie.is_empty() {
        text.push_str(&format!(
            "🔥 <b>Toplam Kalori:</b> {}\n",
            escape(&menu_data.totalcalorie)
        ));
    }

    text.trim_end().to_string()
}
