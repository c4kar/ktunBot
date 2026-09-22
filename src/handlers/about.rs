use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

pub async fn send_about(bot: &Bot, chat_id: ChatId, _svc: &Arc<Services>) -> ResponseResult<()> {
    let msg = "🤖 <b>ktünBot v2.0</b>\n\nKTÜN öğrencileri için resmi olmayan bir yardımcı bot.\n\n\
               Rust 🦀 ve Teloxide kullanılarak yeniden yazılmıştır.\n\
               Geliştirici: @c4kar";

    bot.send_message(chat_id, msg)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
