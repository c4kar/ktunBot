use crate::commands::Command;
use crate::keyboards::main_menu;
use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;

pub async fn send_start(bot: &Bot, chat_id: ChatId, _svc: &Arc<Services>) -> ResponseResult<()> {
    let welcome_msg = "Merhaba! KTÜN Bot'a hoş geldiniz.\n\nAşağıdaki menüden veya komutlardan istediğiniz bilgiye ulaşabilirsiniz.";
    bot.send_message(chat_id, welcome_msg)
        .reply_markup(main_menu())
        .await?;
    Ok(())
}

pub async fn send_help(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    use teloxide::utils::command::BotCommands;
    bot.send_message(chat_id, Command::descriptions().to_string())
        .await?;
    Ok(())
}
