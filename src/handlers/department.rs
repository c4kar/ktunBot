use crate::services::Services;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub fn dept_name_tr(code: &str) -> &'static str {
    match code {
        "CENG" => "💻 Bilgisayar Mühendisliği",
        "EEM" => "⚡ Elektrik-Elektronik Mühendisliği",
        "MECH" => "⚙️ Makine Mühendisliği",
        "MKT" => "🤖 Mekatronik Mühendisliği",
        "CIVIL" => "🏗️ İnşaat Mühendisliği",
        "CHEM" => "🧪 Kimya Mühendisliği",
        "IE" => "🏭 Endüstri Mühendisliği",
        "GEO" => "🗺️ Harita Mühendisliği",
        _ => "🎓 Genel STEM",
    }
}

pub fn department_keyboard() -> InlineKeyboardMarkup {
    let row1 = vec![
        InlineKeyboardButton::callback("💻 Bilgisayar", "dept:CENG"),
        InlineKeyboardButton::callback("⚡ Elektrik-Elk.", "dept:EEM"),
    ];
    let row2 = vec![
        InlineKeyboardButton::callback("⚙️ Makine", "dept:MECH"),
        InlineKeyboardButton::callback("🤖 Mekatronik", "dept:MKT"),
    ];
    let row3 = vec![
        InlineKeyboardButton::callback("🏗️ İnşaat", "dept:CIVIL"),
        InlineKeyboardButton::callback("🧪 Kimya", "dept:CHEM"),
    ];
    let row4 = vec![
        InlineKeyboardButton::callback("🏭 Endüstri", "dept:IE"),
        InlineKeyboardButton::callback("🗺️ Harita", "dept:GEO"),
    ];
    InlineKeyboardMarkup::new(vec![row1, row2, row3, row4])
}

pub fn grade_keyboard(dept: &str) -> InlineKeyboardMarkup {
    let row1 = vec![
        InlineKeyboardButton::callback("1. Sınıf", format!("grade:{}:1", dept)),
        InlineKeyboardButton::callback("2. Sınıf", format!("grade:{}:2", dept)),
    ];
    let row2 = vec![
        InlineKeyboardButton::callback("3. Sınıf", format!("grade:{}:3", dept)),
        InlineKeyboardButton::callback("4. Sınıf", format!("grade:{}:4", dept)),
    ];
    InlineKeyboardMarkup::new(vec![row1, row2])
}

pub async fn send_department_picker(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
) -> ResponseResult<()> {
    let profile = svc.db.get_student_profile(chat_id.0).await.ok().flatten();

    let text = match profile {
        Some(p) => format!(
            "🎓 <b>Mevcut Profiliniz:</b>\n• <b>Bölüm:</b> {}\n• <b>Sınıf:</b> {}. Sınıf\n• <b>Magnum Krediniz:</b> {}\n\nDeğiştirmek için aşağıdan bölümünüzü seçin:",
            dept_name_tr(&p.department),
            p.grade,
            p.magnum_credits
        ),
        None => "🎓 <b>STEM Bölümünüzü Seçin:</b>\n\nSize özel ders programı ve notları sunabilmemiz için lütfen bölümünüzü seçin:".to_string(),
    };

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(department_keyboard())
        .await?;

    Ok(())
}

pub async fn handle_department_select(
    bot: &Bot,
    chat_id: ChatId,
    dept: &str,
) -> ResponseResult<()> {
    let name = dept_name_tr(dept);
    let text = format!(
        "Seçilen Bölüm: <b>{}</b>\n\nLütfen kaçıncı sınıf olduğunuzu seçin:",
        name
    );

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(grade_keyboard(dept))
        .await?;

    Ok(())
}

pub async fn handle_grade_select(
    bot: &Bot,
    chat_id: ChatId,
    svc: &Arc<Services>,
    dept: &str,
    grade: u8,
) -> ResponseResult<()> {
    let term = "Guz";
    if let Err(e) = svc.db.upsert_student_profile(chat_id.0, dept, grade, term).await {
        tracing::error!("Failed to save student profile: {}", e);
        bot.send_message(chat_id, "❌ Profil kaydedilirken hata oluştu.").await?;
        return Ok(());
    }

    let text = format!(
        "✅ <b>Profiliniz Başarıyla Kaydedildi!</b>\n\n• <b>Bölüm:</b> {}\n• <b>Sınıf:</b> {}. Sınıf\n\nArtık ders programınız ve not arama sonuçlarınız bölümünüze özel olarak listelenecektir.",
        dept_name_tr(dept),
        grade
    );

    bot.send_message(chat_id, text)
        .parse_mode(ParseMode::Html)
        .reply_markup(crate::keyboards::main_menu())
        .await?;

    Ok(())
}
