import os
import asyncio
import logging
from flask import Flask, request
from dotenv import load_dotenv
from telegram import Update
from telegram.ext import Application
from bot.main import setup_handlers
from bot.utils.logger import setup_logger

load_dotenv()
logger = setup_logger()

# Global bot application (singleton for memory efficiency)
_bot_app = None


def get_bot_app():
    """Get or create the singleton bot application."""
    global _bot_app
    if _bot_app is None:
        token = os.getenv('TELEGRAM_BOT_TOKEN')
        if not token:
            raise ValueError("TELEGRAM_BOT_TOKEN not set")
        
        _bot_app = (
            Application.builder()
            .token(token)
            .build()
        )
        setup_handlers(_bot_app)
    return _bot_app


# Flask app for PythonAnywhere WSGI
app = Flask(__name__)


@app.route('/webhook', methods=['POST'])
def webhook():
    """Handle incoming Telegram updates via webhook."""
    try:
        bot_app = get_bot_app()
        update = Update.de_json(request.get_json(force=True), bot_app.bot)
        
        # Run async handler in sync context
        asyncio.run(bot_app.process_update(update))
        
        return 'OK', 200
    except Exception as e:
        logger.error(f"Webhook error: {e}")
        return 'Error', 500


@app.route('/set_webhook', methods=['GET'])
def set_webhook():
    """Setup webhook - call once after deployment."""
    webhook_url = os.getenv('WEBHOOK_URL')
    if not webhook_url:
        return 'WEBHOOK_URL not set', 400
    
    try:
        bot_app = get_bot_app()
        asyncio.run(bot_app.bot.set_webhook(webhook_url))
        return f'Webhook set to {webhook_url}', 200
    except Exception as e:
        logger.error(f"Set webhook error: {e}")
        return f'Error: {e}', 500


@app.route('/delete_webhook', methods=['GET'])
def delete_webhook():
    """Delete webhook - useful for switching to polling mode."""
    try:
        bot_app = get_bot_app()
        asyncio.run(bot_app.bot.delete_webhook())
        return 'Webhook deleted', 200
    except Exception as e:
        logger.error(f"Delete webhook error: {e}")
        return f'Error: {e}', 500


@app.route('/')
def index():
    """Health check endpoint."""
    return 'KTUN Bot is running!'


# PythonAnywhere expects 'app' or 'application'
application = app
