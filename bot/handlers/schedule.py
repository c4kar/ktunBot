from telegram import Update
from telegram.ext import ContextTypes
from bot.services.schedule_service import ScheduleService
import os
import aiofiles

async def schedule_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle the /program command."""
    try:
        service = ScheduleService()
        pdf_path = service.get_schedule_path()
        
        if not os.path.exists(pdf_path):
            await update.message.reply_text("❌ Dosya bulunamadı.")
            return
        
        # Otomatik dönem bilgisini al
        semester_info = service.get_semester_info()
        
        async with aiofiles.open(pdf_path, 'rb') as pdf_file:
            content = await pdf_file.read()
            await update.message.reply_document(
                document=content,
                filename="Ders_Programi.pdf",
                caption=f"📚 <b>{semester_info} Ders Programı</b>",
                parse_mode='HTML'
            )
    except Exception as e:
        await update.message.reply_text("❌ Gönderim hatası.")
