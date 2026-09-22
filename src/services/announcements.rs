use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Announcement {
    pub title: String,
    pub date: String,
    pub link: String,
}

pub async fn fetch_announcements(
    client: &Client,
    url: &str,
) -> crate::error::Result<Vec<Announcement>> {
    let html = client.get(url).send().await?.text().await?;
    let document = Html::parse_document(&html);

    let tr_sel = Selector::parse("tbody tr").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let date_sel = Selector::parse("td:last-child").unwrap();

    let mut results = Vec::new();

    for tr in document.select(&tr_sel) {
        if let Some(a) = tr.select(&a_sel).next() {
            let title = a.text().collect::<String>().trim().to_string();
            let mut href = a.value().attr("href").unwrap_or("").to_string();
            if href.starts_with('/') {
                href = format!("https://www.ktun.edu.tr{}", href);
            }

            let date = if let Some(td) = tr.select(&date_sel).next() {
                td.text().collect::<String>().trim().to_string()
            } else {
                "".to_string()
            };

            results.push(Announcement {
                title,
                date,
                link: href,
            });
        }
    }

    Ok(results)
}
