use crate::services::Services;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn send_weather(bot: &Bot, chat_id: ChatId, svc: &Services) -> ResponseResult<()> {
    match svc.weather.get_ktun_weather().await {
        Ok(report) => {
            let text = crate::services::weather::WeatherService::format_weather_message(&report);
            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            tracing::error!("Hava durumu alinamadi: {e}");
            bot.send_message(
                chat_id,
                "⚠️ Hava durumu bilgisi alınamadı. Lütfen daha sonra tekrar deneyin.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}

pub async fn send_earthquakes(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Services,
    query: &str,
) -> ResponseResult<()> {
    let is_konya = query.trim().to_uppercase().contains("KONYA");

    let result = if is_konya {
        svc.earthquake.get_konya_recent(5).await
    } else {
        svc.earthquake.get_recent(5).await
    };

    match result {
        Ok(earthquakes) => {
            let title = if is_konya {
                "Konya ve Çevresi Son Depremler"
            } else {
                "Türkiye Geneli Son Depremler"
            };

            let text = crate::services::earthquake::EarthquakeService::format_list(&earthquakes, title);
            bot.send_message(chat_id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            tracing::error!("Deprem bilgisi alinamadi: {e}");
            bot.send_message(
                chat_id,
                "⚠️ Son deprem verileri Kandilli'den alınamadı. Lütfen daha sonra tekrar deneyin.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }
    Ok(())
}
