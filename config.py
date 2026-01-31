import os
from functools import lru_cache
from dotenv import load_dotenv

load_dotenv()

class Config:
    # Bot settings
    TELEGRAM_BOT_TOKEN = os.getenv('TELEGRAM_BOT_TOKEN')
    WEBHOOK_URL = os.getenv('WEBHOOK_URL')  # https://username.pythonanywhere.com/webhook
    UNIVERSITY_ANNOUNCEMENTS_URL = os.getenv('UNIVERSITY_ANNOUNCEMENTS_URL')
    
    # Paths
    BASE_DIR = os.path.dirname(os.path.abspath(__file__))
    DATA_DIR = os.path.join(BASE_DIR, 'data')
    SCHEDULES_DIR = os.path.join(DATA_DIR, 'schedules')
    CALENDARS_DIR = os.path.join(DATA_DIR, 'calendars')
    MENUS_DIR = os.path.join(DATA_DIR, 'menus')
    LOGS_DIR = os.path.join(BASE_DIR, 'logs')
    
    # Performance settings (PythonAnywhere optimized)
    CACHE_TTL_SECONDS = 3600      # 1 saat
    CACHE_MAX_SIZE = 30           # Düşük bellek için sınırlı
    HTTP_TIMEOUT = 15             # PythonAnywhere için daha uzun
    HTTP_MAX_RETRIES = 2
    
    # Logging (Production)
    LOG_LEVEL = os.getenv('LOG_LEVEL', 'WARNING')


@lru_cache(maxsize=1)
def get_config() -> Config:
    """Get singleton Config instance."""
    return Config()
