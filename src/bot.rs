use crate::callbacks;
use crate::commands::Command;
use crate::handlers::{
    about, announcements, cafeteria, calendar, department, intake, magnum, schedule, search, start,
    utility,
};
use crate::services::Services;
use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::prelude::*;

async fn commands_router(
    bot: Bot,
    msg: Message,
    cmd: Command,
    svc: Arc<Services>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    if !svc.cache.check_rate_limit(chat_id.0, 5).await {
        bot.send_message(
            chat_id,
            "⚠️ Çok hızlı işlem yapıyorsunuz. Lütfen birkaç saniye bekleyin.",
        )
        .await?;
        return Ok(());
    }

    match cmd {
        Command::Start => start::send_start(&bot, chat_id, &svc).await?,
        Command::Bolum => department::send_department_picker(&bot, chat_id, &svc).await?,
        Command::Duyurular(count) => {
            announcements::send_announcements(&bot, chat_id, &svc, Command::parse_count(count))
                .await?
        }
        Command::Program => schedule::send_schedule(&bot, chat_id, &svc).await?,
        Command::Takvim => calendar::send_calendar(&bot, chat_id, &svc).await?,
        Command::Bugun => {
            cafeteria::send_menu(&bot, chat_id, &svc, crate::tz::today(), true).await?
        }
        Command::Dun => {
            cafeteria::send_menu(&bot, chat_id, &svc, crate::tz::yesterday(), false).await?
        }
        Command::Yarin => {
            cafeteria::send_menu(&bot, chat_id, &svc, crate::tz::tomorrow(), false).await?
        }
        Command::Hafta => cafeteria::send_week_menu(&bot, chat_id, &svc).await?,
        Command::Rezervasyon => cafeteria::send_reservation(&bot, chat_id).await?,
        Command::Vakit => cafeteria::send_hours(&bot, chat_id).await?,
        Command::Konum => cafeteria::send_locations(&bot, chat_id).await?,
        Command::Hava => utility::send_weather(&bot, chat_id, &svc).await?,
        Command::Deprem(query) => utility::send_earthquakes(&bot, chat_id, &svc, &query).await?,
        Command::Magnum(course) => magnum::handle_magnum_delivery(&bot, chat_id, &svc, &course).await?,
        Command::Yukle => intake::handle_upload_help(&bot, chat_id).await?,
        Command::Ara(query) => search::handle_search(&bot, chat_id, &svc, &query).await?,
        Command::Hakkinda => about::send_about(&bot, chat_id, &svc).await?,
        Command::Yardim => start::send_help(&bot, chat_id).await?,
    }
    Ok(())
}

async fn messages_router(
    bot: Bot,
    msg: Message,
    svc: Arc<Services>,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    if !svc.cache.check_rate_limit(chat_id.0, 5).await {
        bot.send_message(
            chat_id,
            "⚠️ Çok hızlı işlem yapıyorsunuz. Lütfen birkaç saniye bekleyin.",
        )
        .await?;
        return Ok(());
    }

    if let Some(doc) = msg.document() {
        intake::handle_document_upload(&bot, chat_id, &svc, doc, msg.caption()).await?;
    } else if let Some(photos) = msg.photo() {
        intake::handle_photo_upload(&bot, chat_id, &svc, photos, msg.caption()).await?;
    }

    Ok(())
}

pub fn build_dispatcher(
    bot: Bot,
    svc: Services,
) -> Dispatcher<Bot, teloxide::RequestError, teloxide::dispatching::DefaultKey> {
    let dependencies = dptree::deps![Arc::new(svc)];

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(commands_router),
        )
        .branch(
            Update::filter_message()
                .endpoint(messages_router),
        )
        .branch(Update::filter_callback_query().endpoint(callbacks::handle));

    Dispatcher::builder(bot, handler)
        .dependencies(dependencies)
        .enable_ctrlc_handler()
        .build()
}
