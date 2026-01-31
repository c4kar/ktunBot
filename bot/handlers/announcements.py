from telegram import Update
from telegram.ext import ContextTypes
from bot.services.announcement_service import get_announcement_service
from bot.utils.validators import validate_count
from bot.utils.formatter import format_announcement
import logging

logger = logging.getLogger(__name__)


async def announcements_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle the /duyurular command."""
    try:
        count = 10
        if context.args:
            count = validate_count(context.args[0])
        
        loading_msg = await update.message.reply_text("🔄 Duyurular yükleniyor...")
        
        service = get_announcement_service()
        announcements = await service.get_announcements(count)
        
        if not announcements:
            await loading_msg.edit_text("❌ Duyuru bulunamadı.")
            return
        
        text = "📢 <b>Güncel Duyurular</b>\n\n"
        for i, ann in enumerate(announcements, 1):
            text += f"{i}. {format_announcement(ann)}\n\n"
        
        await loading_msg.edit_text(text, parse_mode='HTML', disable_web_page_preview=True)
        
    except Exception as e:
        logger.error(f"Error in announcements: {e}")
        await update.message.reply_text("❌ Hata oluştu.")
