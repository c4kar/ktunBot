use crate::db::IntakeQueueItem;
use crate::handlers::department::dept_name_tr;
use crate::services::Services;
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{Document, ParseMode, PhotoSize};
use tracing::{error, info};

pub async fn handle_upload_help(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let text = "📤 <b>Ders Notu ve Çıkmış Soru Yükleme</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━\n\
        KTÜN STEM Not Havuzu'na katkıda bulunarak ekosistemin büyümesine yardımcı olabilirsiniz!\n\n\
        📌 <b>Nasıl Yüklenir?</b>\n\
        • PDF, Word, PPTX veya ders notu fotoğraflarınızı doğrudan bu sohbete dosya olarak gönderin.\n\
        • İsteğe bağlı olarak dosya açıklamasına (caption) dersin adını yazabilirsiniz (Örn: <code>Fizik 1 Vize Soruları 2024</code>).\n\n\
        🤖 <b>Değerlendirme Süreci:</b>\n\
        Yüklediğiniz materyal Docling & Qdrant yapay zeka hattı tarafından taranır; mükerrer olmayan ve okunabilir notlar kabul edildiğinde hesabınıza <b>Magnum Kredisi</b> yüklenir!";

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn handle_document_upload(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    doc: &Document,
    caption: Option<&str>,
) -> ResponseResult<()> {
    let file_name = doc
        .file_name
        .clone()
        .unwrap_or_else(|| "unnamed_document.pdf".to_string());
    let file_size = doc.file.size as i64;
    let file_id = &doc.file.id;

    process_file_intake(bot, chat_id, svc, &file_id.0, &file_name, file_size, caption).await
}

pub async fn handle_photo_upload(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    photos: &[PhotoSize],
    caption: Option<&str>,
) -> ResponseResult<()> {
    // Pick the largest photo available
    let best_photo = match photos.iter().max_by_key(|p| p.width * p.height) {
        Some(p) => p,
        None => return Ok(()),
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let file_name = format!("photo_note_{}.jpg", timestamp);
    let file_size = best_photo.file.size as i64;
    let file_id = &best_photo.file.id;

    process_file_intake(bot, chat_id, svc, &file_id.0, &file_name, file_size, caption).await
}

async fn process_file_intake(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    telegram_file_id: &str,
    file_name: &str,
    file_size: i64,
    caption: Option<&str>,
) -> ResponseResult<()> {
    // Basic file extension filter
    let lower_name = file_name.to_lowercase();
    let is_valid_ext = [".pdf", ".docx", ".pptx", ".jpg", ".jpeg", ".png", ".zip", ".rar"]
        .iter()
        .any(|ext| lower_name.ends_with(ext));

    if !is_valid_ext {
        bot.send_message(
            chat_id,
            "⚠️ <b>Desteklenmeyen Dosya Biçimi</b>\n\n\
            Lütfen yalnızca PDF, Word (.docx), PowerPoint (.pptx) veya resim (.jpg, .png) formatında ders materyali yükleyin.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    // Generate unique ID and destination path
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let item_id = format!("intake_{}_{}", chat_id.0, nanos);
    let sanitized_filename = file_name
        .replace(['/', '\\', ' '], "_")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
        .collect::<String>();

    let dest_filename = format!("{}_{}", item_id, sanitized_filename);
    let dest_path = svc.config.incoming_dir.join(&dest_filename);

    // Ensure incoming dir exists
    let _ = tokio::fs::create_dir_all(&svc.config.incoming_dir).await;

    // Download from Telegram
    let wait_msg = bot
        .send_message(chat_id, "📥 <i>Dosyanız indiriliyor ve kuyruğa alınıyor...</i>")
        .parse_mode(ParseMode::Html)
        .await?;

    let download_result: Result<(), crate::error::BotError> = async {
        let tg_file = bot.get_file(teloxide::types::FileId(telegram_file_id.to_string())).await?;
        let mut out_file = tokio::fs::File::create(&dest_path).await?;
        bot.download_file(&tg_file.path, &mut out_file)
            .await
            .map_err(|e| crate::error::BotError::Other(e.to_string()))?;
        Ok(())
    }
    .await;

    if let Err(e) = download_result {
        error!("Failed to download telegram file {}: {}", telegram_file_id, e);
        let _ = bot.delete_message(chat_id, wait_msg.id).await;
        bot.send_message(
            chat_id,
            "❌ <b>Dosya İndirilemedi</b>\n\nTelegram sunucularından dosya alınırken bir hata oluştu. Lütfen tekrar deneyin.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    // Get student's department if profile exists
    let profile = svc.db.get_student_profile(chat_id.0).await.ok().flatten();
    let dept_str = profile.as_ref().map(|p| p.department.clone());
    let dept_display = match &dept_str {
        Some(d) => dept_name_tr(d),
        None => "Genel / Belirtilmemiş (AI tespit edecek)",
    };

    // Enqueue in Turso DB
    let queue_item = IntakeQueueItem {
        id: item_id,
        chat_id: chat_id.0,
        file_name: file_name.to_string(),
        file_size,
        storage_path: dest_path.to_string_lossy().to_string(),
        department: dept_str,
        course_hint: caption.map(|s| s.trim().to_string()),
        status: "QUEUED".to_string(),
        quality_score: None,
        rejection_reason: None,
        created_at: None,
        processed_at: None,
    };

    if let Err(e) = svc.db.enqueue_intake(&queue_item).await {
        error!("Failed to enqueue intake item into SQLite: {}", e);
        let _ = bot.delete_message(chat_id, wait_msg.id).await;
        bot.send_message(chat_id, "❌ Dosya kaydedilirken veritabanı hatası oluştu.").await?;
        return Ok(());
    }

    info!("Enqueued student upload: {} ({:.1} KB) for chat_id: {}", file_name, file_size as f64 / 1024.0, chat_id.0);

    let _ = bot.delete_message(chat_id, wait_msg.id).await;

    let confirmation_text = format!(
        "✅ <b>Materyaliniz İnceleme Sırasına Alındı!</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━\n\
        📄 <b>Dosya:</b> <code>{}</code>\n\
        📊 <b>Boyut:</b> {:.1} KB\n\
        🎓 <b>Bölüm:</b> {}\n\
        {}\n\
        🤖 <b>İşlem Adımları:</b>\n\
        • 🔍 OCR & Metin Çıkarımı (Docling)\n\
        • 🗂️ Vektörel Benzerlik & Kopya Filtresi (Qdrant)\n\
        • ⭐️ Akademik Kalite ve Okunabilirlik Puanlaması\n\n\
        <i>Notunuz kabul edildiğinde Magnum Opus derleme havuzuna dahil edilecek ve hesabınıza ek <b>Magnum Kredisi</b> yüklenecektir. KTÜN öğrencileri adına teşekkür ederiz!</i>",
        file_name,
        file_size as f64 / 1024.0,
        dept_display,
        caption.map(|c| format!("📝 <b>Not Açıklaması:</b> <i>{}</i>\n", c)).unwrap_or_default()
    );

    bot.send_message(chat_id, confirmation_text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}
