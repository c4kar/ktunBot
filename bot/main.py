import logging
from telegram.ext import Application, CommandHandler, CallbackQueryHandler
from bot.handlers.start import start_command
from bot.handlers.announcements import announcements_command
from bot.handlers.schedule import schedule_command
from bot.handlers.calendar import calendar_command
from bot.handlers.cafeteria import cafeteria_today, cafeteria_yesterday, cafeteria_tomorrow
from bot.handlers.about import about_command
from bot.handlers.help import help_command
from bot.handlers.callback_handler import button_handler

logger = logging.getLogger(__name__)


def setup_handlers(application: Application):
    """Register bot handlers."""
    # Start command
    application.add_handler(CommandHandler("start", start_command))
    
    # Announcement commands
    application.add_handler(CommandHandler("duyurular", announcements_command))
    
    # Schedule and calendar commands
    application.add_handler(CommandHandler("program", schedule_command))
    application.add_handler(CommandHandler("takvim", calendar_command))
    
    # Cafeteria commands
    application.add_handler(CommandHandler("bugun", cafeteria_today))
    application.add_handler(CommandHandler("dun", cafeteria_yesterday))
    application.add_handler(CommandHandler("yarin", cafeteria_tomorrow))
    
    # About/Help commands
    application.add_handler(CommandHandler("hakkinda", about_command))
    application.add_handler(CommandHandler("yardim", help_command))
    
    # Callback query handler for inline keyboard buttons
    application.add_handler(CallbackQueryHandler(button_handler))
    
    logger.info("Bot handlers registered successfully")
