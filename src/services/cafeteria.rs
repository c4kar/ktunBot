use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MenuItem {
    pub date: String,
    #[serde(rename = "dayOfWeek")]
    pub day_of_week: String,
    #[serde(rename = "mealType")]
    pub meal_type: String,
    pub foods: Vec<String>,
    pub totalcalorie: String,
    #[serde(default)]
    pub closed: bool,
    #[serde(rename = "closedReason", default)]
    pub closed_reason: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MenuData {
    pub year: u32,
    pub month: String,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    pub menu: Vec<MenuItem>,
    pub confidence: Option<String>,
}

pub enum MenuResult {
    Json(MenuItem),
    Image(PathBuf),
    NotFound,
}

pub async fn get_menu(
    client: &reqwest::Client,
    menus_dir: &Path,
    date: chrono::NaiveDate,
) -> MenuResult {
    let year = date.year();
    let month = date.month();
    let json_filename = format!("{:04}-{:02}.json", year, month);
    let json_path = menus_dir.join(&json_filename);
    let target_date_str = crate::tz::format_tr_date(date);

    // 1. Try reading existing JSON
    if let Ok(content) = tokio::fs::read_to_string(&json_path).await {
        if let Ok(data) = serde_json::from_str::<MenuData>(&content) {
            if let Some(item) = data.menu.into_iter().find(|m| m.date == target_date_str) {
                return MenuResult::Json(item);
            }
        }
    }

    // 2. If not found in JSON, attempt live scrape if requested month is current month
    let now = crate::tz::now_istanbul();
    if year == now.year() && month == now.month() {
        if let Ok(scraped) = super::cafeteria_scraper::scrape_and_save_menu(client, menus_dir).await {
            if let Some(item) = scraped.menu.into_iter().find(|m| m.date == target_date_str) {
                return MenuResult::Json(item);
            }
        }
    }

    // 3. Fallback to image
    let img_exts = ["jpg", "jpeg", "png"];
    for ext in img_exts {
        let img_name = format!("menu_{:04}-{:02}.{}", year, month, ext);
        let img_path = menus_dir.join(&img_name);
        if tokio::fs::try_exists(&img_path).await.unwrap_or(false) {
            return MenuResult::Image(img_path);
        }
    }

    MenuResult::NotFound
}

/// Retrieve the 5 weekdays of the current week (Monday to Friday)
pub async fn get_week_menu(
    client: &reqwest::Client,
    menus_dir: &Path,
    date: chrono::NaiveDate,
) -> Vec<MenuItem> {
    let days_from_monday = date.weekday().num_days_from_monday();
    let monday = date - chrono::Duration::days(days_from_monday as i64);

    let mut week_items = Vec::new();
    for day_offset in 0..5 {
        let target_date = monday + chrono::Duration::days(day_offset);
        if let MenuResult::Json(item) = get_menu(client, menus_dir, target_date).await {
            week_items.push(item);
        }
    }
    week_items
}

