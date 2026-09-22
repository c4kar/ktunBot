use crate::services::{cafeteria::MenuResult, Services};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode};

pub async fn send_menu(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    date: chrono::NaiveDate,
    with_day_row: bool,
) -> ResponseResult<()> {
    let res: crate::error::Result<()> = async {
        let menu_res = crate::services::cafeteria::get_menu(&svc.http_client, &svc.config.menus_dir, date).await;

        match menu_res {
            MenuResult::Json(menu_item) => {
                let date_str = crate::tz::format_tr_date(date);
                let text = crate::formatter::format_menu(&menu_item, &date_str);

                let mut send = bot.send_message(chat_id, text).parse_mode(ParseMode::Html);
                if with_day_row {
                    send = send.reply_markup(crate::keyboards::menu_day_row());
                }
                send.await?;
            }
            MenuResult::Image(img_path) => {
                let img_data = tokio::fs::read(&img_path).await?;
                let mut send = bot
                    .send_photo(chat_id, InputFile::memory(img_data).file_name("menu.jpg"))
                    .caption(format!(
                        "🍽 <b>Menü</b> (Görsel)\n{}",
                        crate::tz::format_tr_date(date)
                    ))
                    .parse_mode(ParseMode::Html);
                if with_day_row {
                    send = send.reply_markup(crate::keyboards::menu_day_row());
                }
                send.await?;
            }
            MenuResult::NotFound => {
                bot.send_message(
                    chat_id,
                    format!(
                        "❌ <b>{}</b> için menü bulunamadı.",
                        crate::tz::format_tr_date(date)
                    ),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
        }

        Ok(())
    }
    .await;

    if let Err(e) = res {
        tracing::error!("Error in send_menu: {}", e);
        bot.send_message(chat_id, "❌ Hata oluştu.").await?;
    }

    Ok(())
}

pub async fn send_week_menu(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
) -> ResponseResult<()> {
    let today = crate::tz::today();
    let week_items = crate::services::cafeteria::get_week_menu(
        &svc.http_client,
        &svc.config.menus_dir,
        today,
    )
    .await;

    if week_items.is_empty() {
        bot.send_message(chat_id, "❌ Bu hafta için menü bilgisine ulaşılamadı.")
            .await?;
        return Ok(());
    }

    let mut text = String::from("📋 <b>Bu Haftanın Yemekhane Menüsü</b>\n\n");
    for item in week_items {
        text.push_str(&format!("🍽 <b>{} ({})</b>\n", item.date, item.day_of_week));
        if item.closed {
            text.push_str("   <i>Tatil / Yemek Hizmeti Yok</i>\n\n");
        } else {
            for food in &item.foods {
                let emoji = crate::formatter::get_food_emoji(food);
                text.push_str(&format!("   {} {}\n", emoji, food));
            }
            text.push_str(&format!("   🔥 <i>{}</i>\n\n", item.totalcalorie));
        }
    }

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn send_reservation(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

    let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "🍽 Yemekhane Rezervasyon Sistemi",
        "https://yemekhane.ktun.edu.tr/".parse().unwrap(),
    )]]);

    let text = "🔗 <b>KTÜN Yemekhane Rezervasyon</b>\n\nYemek rezervasyonlarınızı aşağıdaki bağlantıdan KTÜN hesabınızla giriş yaparak gerçekleştirebilirsiniz:";

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

pub async fn send_hours(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let text = "🕒 <b>Yemekhane Servis Saatleri</b>\n\n\
        • <b>Öğle Yemeği:</b> 11:30 - 14:00 (Hafta içi)\n\n\
        ⚠️ <i>Yemek rezervasyonlarının bir gün öncesinden yapılması gerekmektedir.</i>";

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn send_locations(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let text = "📍 <b>Yemekhane Konumları:</b>\n\n\
        1. <b>Merkezi Yemekhane (Sosyal Tesisler):</b>\n\
        👉 <a href=\"https://maps.google.com/?q=38.0250705,32.5109261\">Haritada Aç</a>\n\n\
        2. <b>Kutalmışoğlu Süleymanşah Tesisleri:</b>\n\
        👉 <a href=\"https://maps.google.com/?q=38.0178754,32.5099072\">Haritada Aç</a>";

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

