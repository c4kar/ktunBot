from telegram import Update
from telegram.ext import ContextTypes

async def about_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle /hakkinda command."""
    text = """

Buraya daha yazı yazmadım. Bilahare yazarım.

<b>"Uzmanlık böcekler içindir."</b>
    """
    await update.message.reply_html(text)
