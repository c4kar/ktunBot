use crate::handlers::{about, announcements, cafeteria, calendar, department, schedule};
use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;

pub async fn handle(bot: Bot, q: CallbackQuery, svc: Arc<Services>) -> ResponseResult<()> {
    // Acknowledge callback query immediately to prevent client loading spinner from hanging
    let _ = bot.answer_callback_query(q.id.clone()).await;

    if let Some(data) = &q.data {
        let chat_id = q.message.as_ref().map(|m| m.chat().id);

        if let Some(chat_id) = chat_id {
            if !svc.cache.check_rate_limit(chat_id.0, 10).await {
                // Rate limited callback - silent drop to prevent spam
                return Ok(());
            }

            if data == "announcements" {
                announcements::send_announcements(&bot, chat_id, &svc, 10).await?;
            } else if data == "schedule" {
                schedule::send_schedule(&bot, chat_id, &svc).await?;
            } else if data == "calendar" {
                calendar::send_calendar(&bot, chat_id, &svc).await?;
            } else if data == "cafeteria" {
                cafeteria::send_menu(&bot, chat_id, &svc, crate::tz::today(), true).await?;
            } else if data == "cafeteria_yesterday" {
                cafeteria::send_menu(&bot, chat_id, &svc, crate::tz::yesterday(), false).await?;
            } else if data == "cafeteria_tomorrow" {
                cafeteria::send_menu(&bot, chat_id, &svc, crate::tz::tomorrow(), false).await?;
            } else if data == "cafeteria_week" {
                cafeteria::send_week_menu(&bot, chat_id, &svc).await?;
            } else if data == "cafeteria_res" {
                cafeteria::send_reservation(&bot, chat_id).await?;
            } else if data == "cafeteria_loc" {
                cafeteria::send_locations(&bot, chat_id).await?;
            } else if data == "department" {
                department::send_department_picker(&bot, chat_id, &svc).await?;
            } else if data == "weather" {
                crate::handlers::utility::send_weather(&bot, chat_id, &svc).await?;
            } else if data == "earthquake" {
                crate::handlers::utility::send_earthquakes(&bot, chat_id, &svc, "").await?;
            } else if data == "about" {
                about::send_about(&bot, chat_id, &svc).await?;
            } else if data == "magnum_menu" {
                crate::handlers::magnum::send_magnum_menu(&bot, chat_id, &svc).await?;
            } else if data == "upload_help" {
                crate::handlers::intake::handle_upload_help(&bot, chat_id).await?;
            } else if data == "search_prompt" {
                crate::handlers::search::send_search_prompt(&bot, chat_id, &svc).await?;
            } else if data == "main_menu" {
                bot.send_message(chat_id, "📱 <b>Ana Menü:</b>")
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .reply_markup(crate::keyboards::main_menu())
                    .await?;
            } else if let Some(course) = data.strip_prefix("magnum_get:") {
                crate::handlers::magnum::handle_magnum_delivery(&bot, chat_id, &svc, course).await?;
            } else if let Some(dept) = data.strip_prefix("dept:") {
                department::handle_department_select(&bot, chat_id, dept).await?;
            } else if let Some(grade_info) = data.strip_prefix("grade:") {
                let parts: Vec<&str> = grade_info.split(':').collect();
                if parts.len() == 2 {
                    let dept = parts[0];
                    if let Ok(grade_num) = parts[1].parse::<u8>() {
                        department::handle_grade_select(&bot, chat_id, &svc, dept, grade_num).await?;
                    }
                }
            }
        }
    }

    Ok(())
}
