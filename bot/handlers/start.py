from telegram import Update
from telegram.ext import ContextTypes
from bot.keyboards.inline_keyboards import get_main_menu_keyboard

async def start_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    """Handle the /start command."""
    user = update.effective_user
    welcome_text = f"""
Merhaba {user.mention_html()}! 👋

Ben ktünBot

Komutlar için /yardim
    """
    
    await update.message.reply_html(welcome_text, reply_markup=get_main_menu_keyboard())
