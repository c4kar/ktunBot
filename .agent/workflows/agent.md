---
description: Üniversite öğrencileri için Telegram bot sistemi. PythonAnywhere üzerinde host edilecek. Özellikler: duyuru takibi, ders programı, akademik takvim, yemekhane menüsü.
---

## Sistem Mimarisi
- **Platform**: pythonanywhere.com
- **Python**: 3.10+
- **Framework**: Flask (webhook)
- **Library**: python-telegram-bot (v20.x)
- **Yapı**: Tek bot (tüm özellikler birleşik)

## Gereksinimler

### Python Kütüphaneleri (requirements.txt)
```txt
python-telegram-bot==20.7
requests==2.31.0
beautifulsoup4==4.12.2
python-dotenv==1.0.0
flask==3.0.0
pytz==2023.3
```

### Çevre Değişkenleri (.env)
```env
TELEGRAM_BOT_TOKEN=your_bot_token_here
WEBHOOK_URL=https://yourusername.pythonanywhere.com/webhook
UNIVERSITY_ANNOUNCEMENTS_URL=https://university.edu.tr/duyurular
CAFETERIA_API_URL=https://cafeteria-api-url.com
```

## Dosya Yapısı

```
telegram-bot/
├── bot/
│   ├── __init__.py
│   ├── main.py                 # Ana bot logic
│   ├── handlers/
│   │   ├── __init__.py
│   │   ├── start.py           # /start komutu
│   │   ├── announcements.py   # Duyuru işlemleri
│   │   ├── schedule.py        # Program işlemleri
│   │   ├── calendar.py        # Takvim işlemleri
│   │   ├── cafeteria.py       # Yemekhane menüsü
│   │   └── about.py           # Bot hakkında
│   ├── utils/
│   │   ├── __init__.py
│   │   ├── scraper.py         # Web scraping fonksiyonları
│   │   ├── formatter.py       # Mesaj formatlama
│   │   ├── validators.py      # Input validasyonu
│   │   └── logger.py          # Logging yapılandırması
│   ├── services/
│   │   ├── __init__.py
│   │   ├── announcement_service.py
│   │   ├── schedule_service.py
│   │   ├── calendar_service.py
│   │   └── cafeteria_service.py
│   └── keyboards/
│       ├── __init__.py
│       └── inline_keyboards.py
├── data/
│   ├── schedules/
│   │   └── ders_programi.pdf
│   └── calendars/
│       └── akademik_takvim.pdf
├── logs/
│   └── bot.log
├── flask_app.py              # PythonAnywhere WSGI app
├── requirements.txt
├── .env
├── .gitignore
├── config.py                 # Konfigürasyon yönetimi
└── README.md
```

### 1. Duyuru Sistemi

**Komutlar**: `/duyurular [sayı]` - Varsayılan 10, max 50

**İmplementasyon**:
- requests + BeautifulSoup ile web scraping
- 5 dakika cache (bellekte)
- Rate limiting: Kullanıcı başına 1 dakikada 3 istek
- Hata durumunda kullanıcı dostu mesaj

**Scraping Stratejisi**:
```python
# 1. HTML çek, BeautifulSoup ile parse et
# 2. Duyuru elementlerini bul (CSS selector)
# 3. Her duyuru için: başlık, tarih, link çıkar
# 4. Telegram HTML formatında düzenle
```

### 2. Ders Programı

**Komutlar**: `/program`

**İmplementasyon**:
- data/schedules/ klasöründe PDF sakla
- send_document() ile gönder
- Max 20MB dosya boyutu
- PDF bulunamazsa hata mesajı

### 3. Akademik Takvim

**Komutlar**: `/takvim [guz|bahar]`

**İmplementasyon**:
- data/calendars/ klasöründe PDF sakla
- Dönem algılama: Eylül-Ocak=Güz, Şubat-Haziran=Bahar
- Inline keyboard ile dönem seçimi

### 4. Yemekhane Menüsü

**Komutlar**: `/dun`, `/bugun`, `/yarin`, `/hafta`, `/ay`, `/komutlar`, `/hakkinda`

**İmplementasyon**:
- API/scraping ile menü verisi çek
- Tarih bazlı filtreleme
- Formatlama: Kahvaltı, öğle, akşam öğünleri
```python
# Veri yapısı örneği:
{
    "date": "2024-12-03",
    "meals": {
        "breakfast": ["Peynir", "Zeytin"],
        "lunch": ["Çorba", "Ana yemek"],
        "dinner": ["Akşam menüsü"]
    }
}
```

### 5. Bot Hakkında

**Komutlar**: `/hakkinda`, `/yardim`

**İçerik**: Bot açıklaması, geliştirici bilgileri, GitHub linki, versiyon

## PythonAnywhere Deployment

### 1. Kurulum
```bash
# Virtual environment oluştur
mkvirtualenv --python=/usr/bin/python3.10 telegram-bot

# Projeyi yükle
git clone https://github.com/yourusername/telegram-bot.git
cd telegram-bot

# Dependencies
pip install -r requirements.txt
```

### 2. Flask App (flask_app.py)
```python
import sys
import os
from pathlib import Path

project_home = '/home/yourusername/telegram-bot'
if project_home not in sys.path:
    sys.path.insert(0, project_home)

from dotenv import load_dotenv
load_dotenv(os.path.join(project_home, '.env'))

from bot.main import create_app
application = create_app()
```

### 3. Webhook Setup (bot/main.py)
```python
from flask import Flask, request
from telegram import Update
from telegram.ext import Application

def create_app():
    app = Flask(__name__)
    bot_app = Application.builder().token(os.getenv('TELEGRAM_BOT_TOKEN')).build()
    setup_handlers(bot_app)
    
    @app.route('/webhook', methods=['POST'])
    async def webhook():
        update = Update.de_json(request.get_json(force=True), bot_app.bot)
        await bot_app.process_update(update)
        return 'OK'
    
    @app.route('/set_webhook', methods=['GET'])
    async def set_webhook():
        webhook_url = os.getenv('WEBHOOK_URL')
        await bot_app.bot.set_webhook(webhook_url)
        return f'Webhook set to {webhook_url}', 200
    
    return app
```

### 4. Web App Config
- Source code: /home/yourusername/telegram-bot
- WSGI file: /var/www/yourusername_pythonanywhere_com_wsgi.py
- Virtualenv: /home/yourusername/.virtualenvs/telegram-bot

### 5. Webhook Aktivasyonu
```bash
# Tarayıcıda:
https://yourusername.pythonanywhere.com/set_webhook
```

### Optimizasyon (Ücretsiz Plan İçin)
```python
# Cache ile CPU tasarrufu
from functools import lru_cache
import time

@lru_cache(maxsize=100)
def get_announcements_cached(timestamp):
    return fetch_announcements()

def get_announcements():
    cache_timestamp = int(time.time() / 300)  # 5 dakika
    return get_announcements_cached(cache_timestamp)

# Rate limiting
from collections import defaultdict
from datetime import datetime, timedelta

user_requests = defaultdict(list)

def check_rate_limit(user_id, max_requests=10, window_minutes=1):
    now = datetime.now()
    cutoff = now - timedelta(minutes=window_minutes)
    user_requests[user_id] = [t for t in user_requests[user_id] if t > cutoff]
    
    if len(user_requests[user_id]) >= max_requests:
        return False
    
    user_requests[user_id].append(now)
    return True
```

## Handler Kod Örnekleri

### Start Handler (bot/handlers/start.py)
```python
from telegram import Update, InlineKeyboardButton, InlineKeyboardMarkup
from telegram.ext import ContextTypes

async def start_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    user = update.effective_user
    welcome_text = f"""
Merhaba {user.mention_html()}! 👋

Ben üniversite asistan botuyum.

📢 Duyurular - 📚 Ders Programı
📅 Akademik Takvim - 🍽 Yemekhane Menüsü

Komutlar için /yardim
    """
    
    keyboard = [
        [InlineKeyboardButton("📢 Duyurular", callback_data="announcements"),
         InlineKeyboardButton("📚 Program", callback_data="schedule")],
        [InlineKeyboardButton("📅 Takvim", callback_data="calendar"),
         InlineKeyboardButton("🍽 Menü", callback_data="cafeteria")],
        [InlineKeyboardButton("ℹ️ Hakkında", callback_data="about")]
    ]
    reply_markup = InlineKeyboardMarkup(keyboard)
    await update.message.reply_html(welcome_text, reply_markup=reply_markup)
```

### Announcements Handler (bot/handlers/announcements.py)
```python
from telegram import Update
from telegram.ext import ContextTypes
from bot.services.announcement_service import AnnouncementService

async def announcements_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        count = 10
        if context.args and context.args[0].isdigit():
            count = min(int(context.args[0]), 50)
        
        if not check_rate_limit(update.effective_user.id):
            await update.message.reply_text("⏳ Çok fazla istek. 1 dakika bekleyin.")
            return
        
        loading_msg = await update.message.reply_text("🔄 Duyurular yükleniyor...")
        
        service = AnnouncementService()
        announcements = await service.get_announcements(count)
        
        if not announcements:
            await loading_msg.edit_text("❌ Duyuru bulunamadı.")
            return
        
        text = "📢 <b>Güncel Duyurular</b>\n\n"
        for i, ann in enumerate(announcements, 1):
            text += f"{i}. <a href='{ann['link']}'>{ann['title']}</a>\n📅 {ann['date']}\n\n"
        
        await loading_msg.edit_text(text, parse_mode='HTML', disable_web_page_preview=True)
        
    except Exception as e:
        await update.message.reply_text("❌ Hata oluştu.")
```

### Schedule Handler (bot/handlers/schedule.py)
```python
from telegram import Update
from telegram.ext import ContextTypes
from pathlib import Path

async def schedule_command(update: Update, context: ContextTypes.DEFAULT_TYPE):
    try:
        pdf_path = Path("data/schedules/ders_programi.pdf")
        
        if not pdf_path.exists():
            await update.message.reply_text("❌ Dosya bulunamadı.")
            return
        
        with open(pdf_path, 'rb') as pdf_file:
            await update.message.reply_document(
                document=pdf_file,
                filename="Ders_Programi.pdf",
                caption="📚 <b>2024-2025 Güz Dönemi Ders Programı</b>",
                parse_mode='HTML'
            )
    except Exception as e:
        await update.message.reply_text("❌ Gönderim hatası.")
```

### Cafeteria Handler (bot/handlers/cafeteria.py)
```python
from telegram import Update
from telegram.ext import ContextTypes
from bot.services.cafeteria_service import CafeteriaService
from datetime import datetime, timedelta

async def cafeteria_today(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await send_menu(update, context, datetime.now())

async def cafeteria_yesterday(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await send_menu(update, context, datetime.now() - timedelta(days=1))

async def cafeteria_tomorrow(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await send_menu(update, context, datetime.now() + timedelta(days=1))

async def send_menu(update: Update, context: ContextTypes.DEFAULT_TYPE, date: datetime):
    try:
        loading_msg = await update.message.reply_text("🔄 Menü yükleniyor...")
        service = CafeteriaService()
        menu = await service.get_menu_for_date(date)
        
        if not menu:
            await loading_msg.edit_text(f"❌ Menü bulunamadı.")
            return
        
        text = f"📅 <b>{date.strftime('%d.%m.%Y')}</b>\n\n"
        if menu.get('breakfast'):
            text += "🌅 <b>Kahvaltı:</b>\n" + "\n".join(f"• {i}" for i in menu['breakfast']) + "\n\n"
        if menu.get('lunch'):
            text += "🍽 <b>Öğle:</b>\n" + "\n".join(f"• {i}" for i in menu['lunch']) + "\n\n"
        if menu.get('dinner'):
            text += "🌙 <b>Akşam:</b>\n" + "\n".join(f"• {i}" for i in menu['dinner'])
        
        await loading_msg.edit_text(text, parse_mode='HTML')
    except Exception as e:
        await update.message.reply_text("❌ Hata oluştu.")
```