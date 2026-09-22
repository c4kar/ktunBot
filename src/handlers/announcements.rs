use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn send_announcements(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    count: usize,
) -> ResponseResult<()> {
    let res: crate::error::Result<()> = async {
        let announcements = if let Some(cached) = svc.cache.get_announcements(count).await {
            serde_json::from_slice(&cached).unwrap_or_default()
        } else if let Some(blob) = svc
            .db
            .cache_get(&format!("announcements_{}", count))
            .await?
        {
            svc.cache.set_announcements(count, blob.clone()).await;
            serde_json::from_slice(&blob).unwrap_or_default()
        } else {
            let client = crate::scraper::build_client()?;
            let url = svc.config.announcements_url.as_deref().unwrap_or("https://www.ktun.edu.tr/tr/Universite/DuyuruTum");
            let mut list = crate::services::announcements::fetch_announcements(
                &client,
                url,
            )
            .await?;

            list.truncate(count);

            let bytes = serde_json::to_vec(&list)?;
            svc.cache.set_announcements(count, bytes.clone()).await;
            let _ = svc
                .db
                .cache_set(
                    &format!("announcements_{}", count),
                    bytes,
                    std::time::Duration::from_secs(900),
                )
                .await;
            list
        };

        if announcements.is_empty() {
            bot.send_message(chat_id, "❌ Duyuru bulunamadı.").await?;
            return Ok(());
        }

        let mut msg = String::from("📢 <b>Güncel Duyurular</b>\n\n");
        for (i, ann) in announcements.iter().enumerate() {
            msg.push_str(&format!(
                "{}. {}\n\n",
                i + 1,
                crate::formatter::format_announcement(ann)
            ));
        }

        bot.send_message(chat_id, msg)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
    .await;

    if let Err(e) = res {
        tracing::error!("Error in send_announcements: {}", e);
        bot.send_message(chat_id, "❌ Hata oluştu.").await?;
    }

    Ok(())
}
