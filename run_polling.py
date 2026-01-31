import os
import logging
import asyncio
from dotenv import load_dotenv
from telegram.ext import Application
from bot.main import setup_handlers
from bot.utils.logger import setup_logger

# Load .env
load_dotenv()

# Setup logger
logger = setup_logger()

def main():
    """Run the bot in polling mode for local development."""
    token = os.getenv('TELEGRAM_BOT_TOKEN')
    if not token:
        logger.error("TELEGRAM_BOT_TOKEN not found in environment variables.")
        return

    # Fix for Python 3.14+ event loop issue
    try:
        asyncio.get_event_loop()
    except RuntimeError:
        asyncio.set_event_loop(asyncio.new_event_loop())

    # Build the application
    application = Application.builder().token(token).build()
    
    # Setup handlers using the function from bot.main
    setup_handlers(application)
    
    logger.info("Starting bot in polling mode...")
    # Run the bot until the user presses Ctrl-C
    application.run_polling()

if __name__ == "__main__":
    main()
