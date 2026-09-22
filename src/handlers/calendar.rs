use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::InputFile;

pub async fn send_calendar(bot: &Bot, chat_id: ChatId, svc: &Arc<Services>) -> ResponseResult<()> {
    let res: crate::error::Result<()> = async {
        let path = crate::services::calendar::calendar_path(&svc.config.calendars_dir);

        let pdf_data = tokio::fs::read(&path).await?;
        bot.send_document(
            chat_id,
            InputFile::memory(pdf_data).file_name("akademik_takvim.pdf"),
        )
        .caption("📅 <b>Akademik Takvim</b>")
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

        Ok(())
    }
    .await;

    if let Err(e) = res {
        tracing::error!("Error in send_calendar: {}", e);
        bot.send_message(chat_id, "❌ Hata oluştu.").await?;
    }

    Ok(())
}
