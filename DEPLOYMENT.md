# PythonAnywhere Deployment Guide

Bu rehber, Telegram botunu PythonAnywhere'e deploy etmek için adım adım talimatlar içerir.

## 1. Dosyaları Yükle

### Seçenek A: Git Clone
```bash
cd ~
git clone https://github.com/c4kar/TelegramBot.git
```

### Seçenek B: ZIP Upload
1. Dashboard > Files bölümüne git
2. ZIP dosyasını yükle ve çıkart

## 2. Virtual Environment Oluştur

```bash
cd ~/TelegramBot
mkvirtualenv --python=/usr/bin/python3.10 telegram-bot
pip install -r requirements.txt
```

## 3. Web App Ayarları

1. **Dashboard > Web > Add a new web app** butonuna tıkla
2. **Manual configuration** seç
3. **Python 3.10** seç
4. Ayarları yap:
   - **Source code:** `/home/USERNAME/TelegramBot`
   - **Working directory:** `/home/USERNAME/TelegramBot`
   - **Virtualenv:** `/home/USERNAME/.virtualenvs/telegram-bot`

## 4. WSGI Dosyasını Düzenle

Dashboard > Web > WSGI configuration file linkine tıkla ve içeriği şununla değiştir:

```python
import sys
import os

# Add project directory to path
path = '/home/USERNAME/TelegramBot'
if path not in sys.path:
    sys.path.insert(0, path)

# Set environment variables
os.environ['TELEGRAM_BOT_TOKEN'] = 'YOUR_BOT_TOKEN_HERE'
os.environ['WEBHOOK_URL'] = 'https://USERNAME.pythonanywhere.com/webhook'
os.environ['UNIVERSITY_ANNOUNCEMENTS_URL'] = 'https://www.ktun.edu.tr/tr/Birim/Index/?brm=YlNTVXpJWnBCMHpLQlRaMnZFTXlMdz09'
os.environ['LOG_LEVEL'] = 'WARNING'

from flask_app import application
```

> ⚠️ `USERNAME` kısmını kendi PythonAnywhere kullanıcı adınla değiştir!

## 5. Environment Variables (Alternatif)

`.env` dosyası kullanmak istersen, projenin kök dizininde oluştur:

```bash
# ~/TelegramBot/.env
TELEGRAM_BOT_TOKEN=your_bot_token_here
WEBHOOK_URL=https://USERNAME.pythonanywhere.com/webhook
UNIVERSITY_ANNOUNCEMENTS_URL=https://www.ktun.edu.tr/tr/Birim/Index/?brm=YlNTVXpJWnBCMHpLQlRaMnZFTXlMdz09
LOG_LEVEL=WARNING
```

## 6. Web App'i Yeniden Başlat

Dashboard > Web sayfasında **Reload** butonuna tıkla.

## 7. Webhook Ayarla

Tarayıcında şu URL'yi aç:
```
https://USERNAME.pythonanywhere.com/set_webhook
```

Başarılı olursa şöyle bir mesaj göreceksin:
```
Webhook set to https://USERNAME.pythonanywhere.com/webhook
```

## 8. Test Et

1. Telegram'da botuna mesaj gönder: `/start`
2. Menü butonlarını test et
3. `/bugun`, `/duyurular` gibi komutları dene

## Sorun Giderme

### Error Loglarını Kontrol Et
Dashboard > Web > Error log linkine tıkla

### Yaygın Hatalar

**"No module named 'bot'"**
- WSGI dosyasında path doğru ayarlanmamış
- Virtualenv aktif değil

**"TELEGRAM_BOT_TOKEN not set"**
- Environment variable'lar WSGI dosyasında veya .env'de tanımlı değil

**Duyurular yüklenmiyor**
- PythonAnywhere free tier'da `ktun.edu.tr` whitelist'te olmayabilir
- Paid plan'a geçmek gerekebilir veya whitelist talebi gönder

### Webhook Silme (Polling'e geçmek için)
```
https://USERNAME.pythonanywhere.com/delete_webhook
```

## Dosya Yapısı

```
~/TelegramBot/
├── bot/
│   ├── handlers/
│   ├── services/
│   ├── utils/
│   └── main.py
├── data/
│   ├── menus/
│   ├── schedules/
│   └── calendars/
├── logs/
├── .env
├── config.py
├── flask_app.py
├── requirements.txt
└── DEPLOYMENT.md
```

## Menü Dosyaları

Yemekhane menüleri `data/menus/` klasöründe JSON formatında tutulur:
- Dosya adı formatı: `YYYY-MM.json` (örn: `2025-12.json`)
- Yeni aylar için yeni dosya oluştur

## Güncellemeler

Kodu güncelledikten sonra:
1. Git pull veya dosyaları yeniden yükle
2. Dashboard > Web > **Reload** butonuna tıkla
