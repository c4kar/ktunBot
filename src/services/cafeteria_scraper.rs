//! Cafeteria live scraper for Selçuk/KTÜN menu (ported from legacy).
//!
//! Scrapes https://yemek.selcuk.edu.tr/Menu/MenuGetir and serializes
//! the full month menu to `data/menus/{YYYY-MM}.json`.

use crate::error::Result;
use crate::services::cafeteria::{MenuData, MenuItem};
use chrono::Datelike;
use scraper::{Html, Selector};
use std::path::Path;

const MENU_URL: &str = "https://yemek.selcuk.edu.tr/Menu/MenuGetir";

use std::sync::LazyLock;

static TABLE_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("table.menu-calendar").expect("valid table selector"));
static TR_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("tr").expect("valid tr selector"));
static TD_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("td").expect("valid td selector"));
static H4_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("h4").expect("valid h4 selector"));
static LI_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("ul li").expect("valid li selector"));
static CAL_SEL: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".calorie-value").expect("valid calorie selector"));

/// Parses raw cafeteria HTML into MenuData in a pure synchronous function.
pub fn parse_menu_html(
    html_text: &str,
    year: u32,
    month_num: u32,
    generated_at: &str,
) -> MenuData {
    let month_name = crate::tz::tr_month_name(month_num);
    let document = Html::parse_document(html_text);

    let days_of_week = ["Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma"];
    let mut menu_items = Vec::new();

    if let Some(table) = document.select(&TABLE_SEL).next() {
        for tr in table.select(&TR_SEL) {
            for (col_idx, td) in tr.select(&TD_SEL).enumerate() {
                let day_of_week = days_of_week.get(col_idx).unwrap_or(&"Bilinmeyen").to_string();

                // Extract date from <h4> e.g. "1 Eylül"
                let date_text = td
                    .select(&H4_SEL)
                    .next()
                    .map(|h| h.text().collect::<String>().trim().to_string());

                if let Some(date_raw) = date_text {
                    if date_raw.is_empty() {
                        continue;
                    }

                    // Normalize date format "1 Eylül" -> "01 Eylül"
                    let parts: Vec<&str> = date_raw.split_whitespace().collect();
                    let normalized_date = if parts.len() >= 2 {
                        if let Ok(day_num) = parts[0].parse::<u32>() {
                            format!("{:02} {}", day_num, parts[1])
                        } else {
                            date_raw.clone()
                        }
                    } else {
                        date_raw.clone()
                    };

                    // Extract foods
                    let foods: Vec<String> = td
                        .select(&LI_SEL)
                        .map(|li| li.text().collect::<String>().trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    // Extract calories
                    let cal_text = td
                        .select(&CAL_SEL)
                        .next()
                        .map(|c| c.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                    let totalcalorie = if !cal_text.is_empty() {
                        format!("{} Kcal", cal_text)
                    } else {
                        "Belirtilmedi".to_string()
                    };

                    let is_closed = foods.is_empty();
                    let closed_reason = if is_closed {
                        "Resmi Tatil / Yemek Hizmeti Yok".to_string()
                    } else {
                        "".to_string()
                    };

                    menu_items.push(MenuItem {
                        date: normalized_date,
                        day_of_week,
                        meal_type: "öğle".to_string(),
                        foods,
                        totalcalorie,
                        closed: is_closed,
                        closed_reason,
                    });
                }
            }
        }
    }

    MenuData {
        year,
        month: month_name.to_string(),
        generated_at: generated_at.to_string(),
        menu: menu_items,
        confidence: Some("live_scraped".to_string()),
    }
}

/// Scrapes the live menu from Selçuk/KTÜN refectory and saves it to `menus_dir`.
pub async fn scrape_and_save_menu(client: &reqwest::Client, menus_dir: &Path) -> Result<MenuData> {
    let now = crate::tz::now_istanbul();
    let year = now.year() as u32;
    let month_num = now.month();

    let res = client
        .get(MENU_URL)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    let html_text = res.text().await?;
    let menu_data = parse_menu_html(&html_text, year, month_num, &now.to_rfc3339());

    // Save to disk
    tokio::fs::create_dir_all(menus_dir).await?;
    let json_filename = format!("{:04}-{:02}.json", year, month_num);
    let json_path = menus_dir.join(&json_filename);
    let serialized = serde_json::to_string_pretty(&menu_data)?;
    tokio::fs::write(&json_path, serialized).await?;
    tracing::info!(path = ?json_path, count = menu_data.menu.len(), "saved live cafeteria menu");

    Ok(menu_data)
}
