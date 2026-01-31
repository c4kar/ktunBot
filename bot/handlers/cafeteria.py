from telegram import Update
from telegram.ext import ContextTypes
from bot.services.cafeteria_service import get_cafeteria_service
from bot.utils.formatter import format_menu
from datetime import datetime, timedelta
import logging
import aiofiles

logger = logging.getLogger(__name__)


async def cafeteria_today(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /bugun command."""
    await send_menu(update, context, datetime.now())


async def cafeteria_yesterday(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /dun command."""
    await send_menu(update, context, datetime.now() - timedelta(days=1))


async def cafeteria_tomorrow(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /yarin command."""
    await send_menu(update, context, datetime.now() + timedelta(days=1))


async def send_menu(update: Update, context: ContextTypes.DEFAULT_TYPE, date: datetime):
    """Helper to fetch and send menu."""
    try:
        loading_msg = await update.message.reply_text("🔄 Menü yükleniyor...")
        
        service = get_cafeteria_service()
        menu = await service.get_menu_for_date(date)
        
        if not menu:
            available_months = service.get_available_months()
            if not available_months:
                await loading_msg.edit_text(
                    "❌ Henüz menü dosyası yüklenmemiş.\n\n"
                    "📁 Menü dosyaları `data/menus/` klasörüne eklenmeli.\n"
                    "📅 Dosya formatı: `YYYY-MM.json` (örn: 2025-12.json)"
                )
            else:
                month_list = ', '.join([f"{m['year']}-{m['month']:02d}" for m in available_months[:3]])
                await loading_msg.edit_text(
                    f"❌ {date.strftime('%d.%m.%Y')} tarihi için menü bulunamadı.\n\n"
                    f"📁 Mevcut menüler: {month_list}"
                )
            return
        
        # Check if it's an image
        if menu.get('type') == 'image':
            # Delete the loading message
            await loading_msg.delete()
            # Send the image
            async with aiofiles.open(menu['path'], 'rb') as photo:
                content = await photo.read()
                await update.message.reply_photo(
                    photo=content,
                    caption=f"🍽 <b>Yemekhane Menüsü</b>\n📅 {menu['date']}",
                    parse_mode='HTML'
                )
        else:
            # Format and send as text (JSON format)
            text = format_menu(menu, date.strftime('%d.%m.%Y'))
            await loading_msg.edit_text(text, parse_mode='HTML')
        
    except Exception as e:
        logger.error(f"Error in send_menu: {e}")
        await update.message.reply_text(
            f"❌ Menü yüklenirken hata oluştu.",
            parse_mode='HTML'
        )
