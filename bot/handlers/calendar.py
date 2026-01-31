from telegram import Update
from telegram.ext import ContextTypes
from bot.services.calendar_service import CalendarService
from bot.keyboards.inline_keyboards import get_calendar_keyboard
import os
import aiofiles

async def calendar_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle the /takvim command."""
    # For now, just sending the PDF directly or showing options
    # If we want to support term selection via buttons, we need a CallbackQueryHandler
    # Here we implement the basic command to send the PDF
    
    try:
        service = CalendarService()
        pdf_path = service.get_calendar_path()
        
        if not os.path.exists(pdf_path):
            await update.message.reply_text("❌ Dosya bulunamadı.")
            return
            
        async with aiofiles.open(pdf_path, 'rb') as pdf_file:
            content = await pdf_file.read()
            await update.message.reply_document(
                document=content,
                filename="Akademik_Takvim.pdf",
                caption="📅 <b>Akademik Takvim</b>",
                parse_mode='HTML'
            )
    except Exception as e:
        await update.message.reply_text("❌ Gönderim hatası.")
