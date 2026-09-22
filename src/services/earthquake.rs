use std::sync::Arc;
use std::time::Duration;
use anyhow::{Context, Result};
use moka::future::Cache;

const KANDILLI_URL: &str = "http://www.koeri.boun.edu.tr/scripts/lst0.asp";

#[derive(Debug, Clone)]
pub struct Earthquake {
    pub date: String,
    pub time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub depth_km: f64,
    pub magnitude: f64,
    pub location: String,
}

impl Earthquake {
    pub fn is_konya(&self) -> bool {
        self.location.to_uppercase().contains("KONYA")
    }

    pub fn severity_emoji(&self) -> &'static str {
        if self.magnitude >= 5.0 {
            "🔴"
        } else if self.magnitude >= 4.0 {
            "🟡"
        } else if self.magnitude >= 3.0 {
            "🟠"
        } else {
            "🟢"
        }
    }
}

#[derive(Clone)]
pub struct EarthquakeService {
    client: reqwest::Client,
    cache: Cache<String, Vec<Earthquake>>,
}

impl Default for EarthquakeService {
    fn default() -> Self {
        Self::new()
    }
}

impl EarthquakeService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap_or_default(),
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(60)) // 1 dakika önbellek
                .build(),
        }
    }

    pub fn parse_kandilli_raw(raw_bytes: &[u8]) -> Vec<Earthquake> {
        let (decoded_text, _, _) = encoding_rs::WINDOWS_1254.decode(raw_bytes);
        let mut results = Vec::new();

        let in_pre = if let Some(start) = decoded_text.find("<pre>") {
            &decoded_text[start + 5..]
        } else {
            &decoded_text
        };

        let content = if let Some(end) = in_pre.find("</pre>") {
            &in_pre[..end]
        } else {
            in_pre
        };

        for line in content.lines() {
            let line = line.trim();
            // Data lines start with date e.g. "2026.09.21"
            if line.len() >= 70 && line.chars().take(4).all(|c| c.is_ascii_digit()) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 8 {
                    let date = parts[0].to_string();
                    let time = parts[1].to_string();
                    let lat = parts[2].parse::<f64>().unwrap_or(0.0);
                    let lon = parts[3].parse::<f64>().unwrap_or(0.0);
                    let depth = parts[4].parse::<f64>().unwrap_or(0.0);

                    // Magnitude can be in column 5 (MD) or 6 (ML). We prefer ML, fallback to MD
                    let mag = parts[6]
                        .parse::<f64>()
                        .or_else(|_| parts[5].parse::<f64>())
                        .unwrap_or(0.0);

                    // Location is from part 8 onwards until "İlksel" or "REVIZE"
                    let loc_parts: Vec<&str> = parts[8..]
                        .iter()
                        .take_while(|&&p| {
                            let p_upper = p.to_uppercase();
                            !p_upper.starts_with("ILKSEL")
                                && !p_upper.starts_with("İLKS")
                                && !p_upper.starts_with("REVIZE")
                        })
                        .copied()
                        .collect();
                    let location = loc_parts.join(" ");

                    if !location.is_empty() && mag > 0.0 {
                        results.push(Earthquake {
                            date,
                            time,
                            latitude: lat,
                            longitude: lon,
                            depth_km: depth,
                            magnitude: mag,
                            location,
                        });
                    }
                }
            }
        }

        results
    }

    pub async fn fetch_all(&self) -> Result<Vec<Earthquake>> {
        let cache_key = "kandilli_recent".to_string();
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        let bytes = self
            .client
            .get(KANDILLI_URL)
            .send()
            .await
            .context("Kandilli Rasathanesi bağlantısı kurulamadı")?
            .bytes()
            .await
            .context("Kandilli verisi okunamadı")?;

        let earthquakes = Self::parse_kandilli_raw(&bytes);
        self.cache.insert(cache_key, earthquakes.clone()).await;
        Ok(earthquakes)
    }

    pub async fn get_recent(&self, limit: usize) -> Result<Vec<Earthquake>> {
        let all = self.fetch_all().await?;
        Ok(all.into_iter().take(limit).collect())
    }

    pub async fn get_konya_recent(&self, limit: usize) -> Result<Vec<Earthquake>> {
        let all = self.fetch_all().await?;
        Ok(all.into_iter().filter(|e| e.is_konya()).take(limit).collect())
    }

    pub fn format_list(items: &[Earthquake], title: &str) -> String {
        if items.is_empty() {
            return format!("ℹ️ <b>{}</b>\n\nSon kaydedilen sarsıntı bulunamadı.", teloxide::utils::html::escape(title));
        }

        let mut out = format!("🌍 <b>{}</b>\n━━━━━━━━━━━━━━━━━━━━━━\n", teloxide::utils::html::escape(title));

        for eq in items {
            out.push_str(&format!(
                "\n{} <b>M {:.1}</b> | <code>{}</code>\n📍 {}\n📏 Derinlik: {:.1} km\n",
                eq.severity_emoji(),
                eq.magnitude,
                teloxide::utils::html::escape(&eq.time),
                teloxide::utils::html::escape(&eq.location),
                eq.depth_km
            ));
        }

        out.push_str("\n📡 <i>Kaynak: Boğaziçi Üniv. Kandilli Rasathanesi (KRDAE)</i>");
        out
    }
}

pub type SharedEarthquakeService = Arc<EarthquakeService>;
