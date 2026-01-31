import logging
import os
from logging.handlers import RotatingFileHandler
from config import Config


def setup_logger():
    """Configure and return logger optimized for PythonAnywhere."""
    log_file = os.path.join(Config.LOGS_DIR, 'bot.log')
    
    # Create logs directory if it doesn't exist
    os.makedirs(Config.LOGS_DIR, exist_ok=True)
    
    # Production: WARNING level (less disk I/O on PythonAnywhere)
    log_level = getattr(logging, Config.LOG_LEVEL.upper(), logging.WARNING)
    
    logging.basicConfig(
        level=log_level,
        format='%(asctime)s - %(levelname)s - %(message)s',  # Shorter format
        handlers=[
            RotatingFileHandler(
                log_file, 
                maxBytes=256*1024,  # 256KB (PythonAnywhere disk quota friendly)
                backupCount=2       # Fewer backups to save space
            ),
            logging.StreamHandler()
        ]
    )
    
    # Suppress noisy loggers
    logging.getLogger('httpx').setLevel(logging.WARNING)
    logging.getLogger('telegram').setLevel(logging.WARNING)
    logging.getLogger('urllib3').setLevel(logging.WARNING)
    
    return logging.getLogger(__name__)
