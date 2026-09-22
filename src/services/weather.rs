use std::sync::Arc;
use std::time::Duration;
use anyhow::{Context, Result};
use moka::future::Cache;
use serde::Deserialize;

const KTUN_LAT: f64 = 38.0285;
const KTUN_LON: f64 = 32.5105;

#[derive(Debug, Clone)]
pub struct WeatherReport {
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub precipitation: f64,
    pub weather_code: u16,
    pub description: &'static str,
    pub emoji: &'static str,
}

#[derive(Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    relative_humidity_2m: u8,
    apparent_temperature: f64,
    precipitation: f64,
    weather_code: u16,
    wind_speed_10m: f64,
}

#[derive(Clone)]
pub struct WeatherService {
    client: reqwest::Client,
    cache: Cache<String, WeatherReport>,
}

impl Default for WeatherService {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(8))
                .build()
                .unwrap_or_default(),
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(900)) // 15 dakika önbellek
                .build(),
        }
    }

    pub fn wmo_code_to_meta(code: u16) -> (&'static str, &'static str) {
        match code {
            0 => ("Açık / Güneşli", "☀️"),
            1 => ("Çoğunlukla Açık", "🌤️"),
            2 => ("Parçalı Bulutlu", "⛅"),
            3 => ("Kapalı / Çok Bulutlu", "☁️"),
            45 | 48 => ("Sisli / Puslu", "🌫️"),
            51 | 53 | 55 => ("Çisenti Yağmur", "🌦️"),
            61 | 63 => ("Yağmurlu", "🌧️"),
            65 => ("Kuvvetli Yağmurlu", "🌧️"),
            71 | 73 => ("Kar Yağışlı", "❄️"),
            75 => ("Yoğun Kar Yağışlı", "❄️"),
            77 => ("Kar Taneleri", "🌨️"),
            80..=82 => ("Sağanak Yağışlı", "🌧️"),
            85 | 86 => ("Kar Sağanağı", "🌨️"),
            95 => ("Gök Gürültülü Fırtına", "⛈️"),
            96 | 99 => ("Dolu ve Fırtına", "⛈️"),
            _ => ("Bilinmiyor", "🌡️"),
        }
    }

    pub async fn get_ktun_weather(&self) -> Result<WeatherReport> {
        let cache_key = "ktun_campus".to_string();
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,wind_speed_10m&timezone=Europe%2FIstanbul",
            KTUN_LAT, KTUN_LON
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Open-Meteo API isteği başarısız oldu")?
            .json::<OpenMeteoResponse>()
            .await
            .context("Hava durumu verisi JSON ayrıştırma hatası")?;

        let (desc, emoji) = Self::wmo_code_to_meta(resp.current.weather_code);

        let report = WeatherReport {
            temperature: resp.current.temperature_2m,
            apparent_temperature: resp.current.apparent_temperature,
            humidity: resp.current.relative_humidity_2m,
            wind_speed: resp.current.wind_speed_10m,
            precipitation: resp.current.precipitation,
            weather_code: resp.current.weather_code,
            description: desc,
            emoji,
        };

        self.cache.insert(cache_key, report.clone()).await;
        Ok(report)
    }

    pub fn format_weather_message(report: &WeatherReport) -> String {
        format!(
            "{emoji} <b>KTÜN Yerleşkesi Hava Durumu</b>\n\
             ━━━━━━━━━━━━━━━━━━━━━━\n\
             🌡️ <b>Sıcaklık:</b> {temp:.1}°C <i>(Hissedilen: {feel:.1}°C)</i>\n\
             {emoji} <b>Durum:</b> {desc}\n\
             💧 <b>Nem:</b> %{hum}\n\
             💨 <b>Rüzgar:</b> {wind:.1} km/s\n\
             🌧️ <b>Yağış:</b> {prec:.1} mm\n\n\
             📍 <i>Gelişim Yerleşkesi / Selçuklu, Konya</i>",
            emoji = report.emoji,
            temp = report.temperature,
            feel = report.apparent_temperature,
            desc = report.description,
            hum = report.humidity,
            wind = report.wind_speed,
            prec = report.precipitation,
        )
    }
}

pub type SharedWeatherService = Arc<WeatherService>;
