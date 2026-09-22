use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::InputFile;

pub async fn send_schedule(bot: &Bot, chat_id: ChatId, svc: &Arc<Services>) -> ResponseResult<()> {
    let res: crate::error::Result<()> = async {
        let path = crate::services::schedule::schedule_path(&svc.config.schedules_dir);
        let semester = crate::services::semester::current_semester(crate::tz::today());

        let pdf_data = tokio::fs::read(&path).await?;
        bot.send_document(
            chat_id,
            InputFile::memory(pdf_data).file_name("ders_programi.pdf"),
        )
        .caption(format!("📚 <b>{} Dönemi Ders Programı</b>", semester))
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

        Ok(())
    }
    .await;

    if let Err(e) = res {
        tracing::error!("Error in send_schedule: {}", e);
        bot.send_message(chat_id, "❌ Hata oluştu.").await?;
    }

    Ok(())
}
