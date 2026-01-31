from telegram import Update
from telegram.ext import ContextTypes
from bot.handlers.announcements import announcements_command
from bot.handlers.schedule import schedule_command
from bot.handlers.calendar import calendar_command
from bot.handlers.about import about_command
from bot.services.cafeteria_service import get_cafeteria_service
from bot.utils.formatter import format_menu
from datetime import datetime, timedelta
import logging

logger = logging.getLogger(__name__)


async def button_handler(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle callback queries from inline keyboard buttons."""
    query = update.callback_query
    await query.answer()  # Acknowledge the button press
    
    callback_data = query.data
    
    # Route to appropriate handler based on callback_data
    if callback_data == "announcements":
        context.args = []
        await handle_callback_as_command(update, context, announcements_command, "🔄 Duyurular yükleniyor...")
        
    elif callback_data == "schedule":
        await handle_callback_as_command(update, context, schedule_command, "📚 Ders programı gönderiliyor...")
        
    elif callback_data == "calendar":
        await handle_callback_as_command(update, context, calendar_command, "📅 Akademik takvim gönderiliyor...")
        
    elif callback_data == "cafeteria":
        await handle_cafeteria_callback(query, datetime.now())
        
    elif callback_data == "cafeteria_yesterday":
        await handle_cafeteria_callback(query, datetime.now() - timedelta(days=1))
        
    elif callback_data == "cafeteria_tomorrow":
        await handle_cafeteria_callback(query, datetime.now() + timedelta(days=1))
        
    elif callback_data == "about":
        await handle_callback_as_command(update, context, about_command, None)


async def handle_cafeteria_callback(query, date: datetime):
    """Handle cafeteria menu callback directly."""
    try:
        await query.edit_message_text("🔄 Menü yükleniyor...")
        
        service = get_cafeteria_service()
        menu = await service.get_menu_for_date(date)
        
        if not menu:
            available_months = service.get_available_months()
            if not available_months:
                await query.edit_message_text(
                    "❌ Henüz menü dosyası yüklenmemiş.\n\n"
                    "📁 Menü dosyaları `data/menus/` klasörüne eklenmeli."
                )
            else:
                month_list = ', '.join([f"{m['year']}-{m['month']:02d}" for m in available_months[:3]])
                await query.edit_message_text(
                    f"❌ {date.strftime('%d.%m.%Y')} tarihi için menü bulunamadı.\n\n"
                    f"📁 Mevcut menüler: {month_list}"
                )
            return
        
        # Check if it's an image
        if menu.get('type') == 'image':
            # Delete the loading message
            await query.message.delete()
            # Send the image
            with open(menu['path'], 'rb') as photo:
                await query.message.reply_photo(
                    photo=photo,
                    caption=f"🍽 <b>Yemekhane Menüsü</b>\n📅 {menu['date']}",
                    parse_mode='HTML'
                )
        else:
            # Format and send as text (JSON format)
            text = format_menu(menu, date.strftime('%d.%m.%Y'))
            await query.edit_message_text(text, parse_mode='HTML')
        
    except Exception as e:
        logger.error(f"Error in cafeteria callback: {e}")
        await query.edit_message_text("❌ Menü yüklenirken hata oluştu.")


async def handle_callback_as_command(update: Update, context: ContextTypes.DEFAULT_TYPE, handler_func, loading_text):
    """Helper to execute command handlers from callback queries."""
    query = update.callback_query
    chat_id = query.message.chat_id
    
    if loading_text:
        await query.edit_message_text(loading_text)
    
    # Create a wrapper that mimics message behavior
    class FakeMessage:
        def __init__(self, chat_id, query):
            self.chat_id = chat_id
            self._query = query
            
        async def reply_text(self, text, **kwargs):
            return await self._query.message.reply_text(text, **kwargs)
            
        async def reply_html(self, text, **kwargs):
            return await self._query.message.reply_text(text, parse_mode='HTML', **kwargs)
            
        async def reply_document(self, document, **kwargs):
            return await self._query.message.reply_document(document=document, **kwargs)
    
    class FakeUpdate:
        def __init__(self, message, user):
            self.message = message
            self.effective_user = user
    
    fake_message = FakeMessage(chat_id, query)
    fake_update = FakeUpdate(fake_message, query.from_user)
    
    try:
        await handler_func(fake_update, context)
    except Exception as e:
        logger.error(f"Error in callback handler: {e}")
        await query.message.reply_text("❌ Bir hata oluştu.")
