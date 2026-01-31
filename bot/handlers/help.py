from telegram import Update
from telegram.ext import ContextTypes

async def help_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /yardim command."""
    text = """
🦾<b>ktünBot</b>🤖

Öğrencilerden öğrenciler için geliştirilmiştir.

<b>• Komutlar •</b>
/duyurular - Güncel duyuruları listeler
/program - Ders programını gönderir
/takvim - Akademik takvimi gönderir
/menu - Yemekhane menüsünü gönderir
/hakkinda - Bot hakkında bilgi

v1.2
    """
    await update.message.reply_html(text)
