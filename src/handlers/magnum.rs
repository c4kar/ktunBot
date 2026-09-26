use crate::handlers::department::dept_name_tr;
use crate::services::magnum::MagnumService;
use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile, ParseMode};
use teloxide::utils::html::escape;
use tracing::{info, warn};

pub fn magnum_menu_keyboard(courses: &[(&'static str, &'static str)]) -> InlineKeyboardMarkup {
    let mut rows: Vec<Vec<InlineKeyboardButton>> = Vec::new();
    let mut current_row: Vec<InlineKeyboardButton> = Vec::new();

    for (label, course_id) in courses {
        current_row.push(InlineKeyboardButton::callback(
            *label,
            format!("magnum_get:{}", course_id),
        ));
        if current_row.len() == 2 {
            rows.push(current_row);
            current_row = Vec::new();
        }
    }
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    // Utility buttons row
    rows.push(vec![
        InlineKeyboardButton::callback("📤 Not Yükle (+Kredi)", "upload_help"),
        InlineKeyboardButton::callback("🎓 Bölüm Değiştir", "department"),
    ]);
    rows.push(vec![InlineKeyboardButton::callback("🔙 Ana Menü", "main_menu")]);

    InlineKeyboardMarkup::new(rows)
}

pub async fn send_magnum_menu(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
) -> ResponseResult<()> {
    let profile = svc.db.get_student_profile(chat_id.0).await.ok().flatten();

    let (dept, grade, title_info) = match &profile {
        Some(p) => (
            p.department.as_str(),
            p.grade,
            format!(
                "🎓 <b>Profiliniz:</b> {} ({}. Sınıf)\n\n",
                dept_name_tr(&p.department),
                p.grade
            ),
        ),
        None => (
            "GENEL",
            1,
            "💡 <i>Bölümünüze özel dersleri görmek için /bolum komutuyla profilinizi kaydedebilirsiniz.</i>\n\n".to_string(),
        ),
    };

    let courses = svc.magnum.get_catalog_courses(dept, grade);
    let keyboard = magnum_menu_keyboard(&courses);

    let text = format!(
        "📖 <b>KTÜN STEM — MAGNUM OPUS DERS NOTLARI</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━\n\
        {}\
        Öğrenci notları, sınav tuzakları ve çıkmış soruların sentezlendiği <b>Magnum Opus</b> çalışma kılavuzunuzu seçin:\n\n\
        <i>💡 Farklı bir ders aramak için: <code>/magnum [ders adı]</code> yazabilirsiniz.</i>",
        title_info
    );

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

pub async fn handle_magnum_delivery(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    course_query: &str,
) -> ResponseResult<()> {
    let query_clean = course_query.trim();
    if query_clean.is_empty() {
        return send_magnum_menu(bot, chat_id, svc).await;
    }

    let normalized = MagnumService::normalize_course_id(query_clean);
    let caption = MagnumService::format_caption(&normalized);

    // 1. Lightning fast path: Telegram file_id cache hit (0 byte upload, ~100ms)
    if let Some(cached_file_id) = svc.magnum.get_cached_telegram_file_id(&normalized).await {
        info!("Serving '{}' ({}) from telegram_file_cache", query_clean, normalized);
        let doc = InputFile::file_id(teloxide::types::FileId(cached_file_id));
        bot.send_document(chat_id, doc)
            .caption(caption)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    // 2. Fast path: PDF exists on local disk storage
    if let Some(disk_path) = svc.magnum.find_existing_pdf(&normalized) {
        info!("Serving '{}' ({}) from local disk: {:?}", query_clean, normalized, disk_path);
        let doc = InputFile::file(disk_path);
        let sent = bot.send_document(chat_id, doc)
            .caption(caption)
            .parse_mode(ParseMode::Html)
            .await?;

        if let Some(doc_info) = sent.document() {
            let _ = svc.magnum.save_cached_telegram_file_id(&normalized, &doc_info.file.id.0).await;
        }
        return Ok(());
    }

    // 3. On-demand compilation path: Compile via ktunMagnum pipeline
    let wait_msg = bot
        .send_message(
            chat_id,
            format!(
                "⏳ <b>{}</b> için Magnum Opus kılavuzu derleniyor...\n\
                <i>Sistemdeki dağınık öğrenci notları ve çıkmış sorular taranıyor, lütfen bekleyin (10-15 sn).</i>",
                escape(&query_clean.to_uppercase())
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;

    match svc.magnum.compile_course(query_clean).await {
        Ok(compiled_path) => {
            // Delete wait message to keep chat clean
            let _ = bot.delete_message(chat_id, wait_msg.id).await;

            let doc = InputFile::file(compiled_path);
            let sent = bot.send_document(chat_id, doc)
                .caption(caption)
                .parse_mode(ParseMode::Html)
                .await?;

            if let Some(doc_info) = sent.document() {
                let _ = svc.magnum.save_cached_telegram_file_id(&normalized, &doc_info.file.id.0).await;
            }
        }
        Err(e) => {
            warn!("Failed to compile course '{}': {}", query_clean, e);
            // Update wait message to inform the user
            let help_text = format!(
                "❌ <b>Ders Notu Bulunamadı</b>\n\n\
                '<b>{}</b>' dersi için henüz sisteme yüklenmiş yeterli ders notu bulunmuyor veya derleme başarısız oldu.\n\n\
                💡 Mevcut hazır paketleri görmek için aşağıdaki butonu kullanabilirsiniz:",
                escape(query_clean)
            );

            let retry_kb = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("📖 Magnum Menüsü", "magnum_menu"),
                InlineKeyboardButton::callback("🎓 Bölüm Seç", "department"),
            ]]);

            bot.edit_message_text(chat_id, wait_msg.id, help_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(retry_kb)
                .await?;
        }
    }

    Ok(())
}
