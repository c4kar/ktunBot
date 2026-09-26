use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

/// Base URL of the ktunot web portal
pub const KTUNOT_WEB_URL: &str = "https://ktunot.net.tr";
pub const KTUNOT_SEARCH_URL: &str = "https://ktunot.net.tr/ara/";

/// Handles the `/ara [sorgu]` command by bridging students to ktunot web archive & Magnum synthesis
pub async fn handle_search(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    query: &str,
) -> ResponseResult<()> {
    let clean_query = query.trim();
    if clean_query.is_empty() {
        return send_search_prompt(bot, chat_id, svc).await;
    }

    // Retrieve user's department to personalize info
    let dept_label = match svc.db.get_student_profile(chat_id.0).await {
        Ok(Some(profile)) => format!(" (Bölümün: <code>{}</code>)", html_escape(&profile.department)),
        _ => String::new(),
    };

    let text = format!(
        "🔍 <b>ktünNot Web Arşivi & Arama</b>\n\n\
        Aranan ders/konu: <b>\"{}\"</b>{}\n\n\
        🌐 Konya Teknik Üniversitesi ders notları, çıkmış sınavlar, kavram haritaları ve dökümanlar \
        <b><a href=\"{}\">ktünNot</a></b> web portalında eksiksiz olarak yayınlanmaktadır.\n\n\
        ✨ <i>Web sitemizde akıl haritası (graph view), konu filtreleri ve anında önizleme ile kolayca arama yapabilirsiniz.</i>\n\n\
        📖 <b>Hızlı Magnum Kılavuzu:</b>\n\
        Dağınık notları tek tek aramak yerine bu derse ait tüm kaynakların sentezlendiği çalışma kılavuzunu \
        hemen indirmek için aşağıdaki butonu kullanabilirsiniz.",
        html_escape(clean_query),
        dept_label,
        KTUNOT_WEB_URL
    );

    let search_button_url = reqwest::Url::parse(KTUNOT_SEARCH_URL)
        .unwrap_or_else(|_| reqwest::Url::parse("https://ktunot.net.tr").unwrap());

    let safe_cb_query = if clean_query.len() > 50 {
        let mut end = 50;
        while end > 0 && !clean_query.is_char_boundary(end) {
            end -= 1;
        }
        &clean_query[..end]
    } else {
        clean_query
    };

    let btn_title = if clean_query.chars().count() > 25 {
        let short: String = clean_query.chars().take(22).collect();
        format!("📖 \"{}...\" Magnum İndir", short)
    } else {
        format!("📖 \"{}\" Magnum Kılavuzu İndir", clean_query)
    };

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::url(
            "🌐 ktünNot'ta Notları Gör",
            search_button_url,
        )],
        vec![InlineKeyboardButton::callback(
            btn_title,
            format!("magnum_get:{}", safe_cb_query),
        )],
        vec![
            InlineKeyboardButton::callback("📤 Not Yükle (+2)", "upload_help"),
            InlineKeyboardButton::callback("📱 Ana Menü", "main_menu"),
        ],
    ]);

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

/// Prompts user with usage guide when `/ara` is invoked without arguments
pub async fn send_search_prompt(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
) -> ResponseResult<()> {
    let dept_info = match svc.db.get_student_profile(chat_id.0).await {
        Ok(Some(p)) => format!("Kayıtlı Bölümün: <b>{}</b> (Sınıf: {})\n\n", p.department, p.grade),
        _ => String::new(),
    };

    let text = format!(
        "🔍 <b>ktünNot Web Not & Sınav Arşivi</b>\n\n\
        {}\
        Mühendislik fakültemizin tüm ders notları, çıkmış sınav soruları ve ders dökümanları \
        <b><a href=\"{}\">ktunot.net.tr</a></b> üzerinde akıl haritası (graph view) ve tam metin arama ile sunulmaktadır.\n\n\
        <b>Nasıl Not Bulabilirim?</b>\n\
        • Doğrudan web arşivimizi ziyaret etmek için aşağıdaki butona dokunun.\n\
        • Veya aramak istediğiniz dersi yazın: <code>/ara Fizik 1</code>, <code>/ara Devre Teorisi</code>\n\n\
        📖 <i>İpucu: Sınava hazırlanırken tek tek not aramak yerine <code>/magnum [ders]</code> ile yapay zekanın derlediği özet kitapçığı indirebilirsiniz!</i>",
        dept_info,
        KTUNOT_WEB_URL
    );

    let search_button_url = reqwest::Url::parse(KTUNOT_SEARCH_URL)
        .unwrap_or_else(|_| reqwest::Url::parse("https://ktunot.net.tr").unwrap());

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::url(
            "🌐 ktünNot Web Arşivini Aç",
            search_button_url,
        )],
        vec![
            InlineKeyboardButton::callback("📖 Magnum Menüsü", "magnum_menu"),
            InlineKeyboardButton::callback("📤 Not Yükle (+2)", "upload_help"),
        ],
        vec![InlineKeyboardButton::callback("📱 Ana Menü", "main_menu")],
    ]);

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
