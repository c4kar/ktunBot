# ktunBot (Rust / Teloxide)

Konya Teknik Üniversitesi öğrencileri için duyuru, menü ve ders programı sağlayan Telegram botu. 
Python altyapısından Rust'a (Teloxide) geçirilmiştir.

## Özellikler
- 📢 Güncel duyuruları çekme (KTÜN sayfasından)
- 🍽 Yemekhane menüsü (günlük/aylık JSON veya görsel desteği)
- 📚 Ders programı (PDF)
- 📅 Akademik takvim (PDF)

## Kurulum ve Çalıştırma

### Gereksinimler
- Rust (≥ 1.85)
- Geçerli bir Telegram Bot Token (BotFather üzerinden)

### Adımlar

1. Depoyu klonlayın ve `.env.example` dosyasını `.env` olarak kopyalayıp düzenleyin.
   ```bash
   cp .env.example .env
   # .env içine TELEGRAM_BOT_TOKEN vb. bilgileri girin.
   ```

2. Veritabanı ve Data Dizinlerini ayarlayın.
   - Bot, `data/ktunbot.db` yolunda lokal bir Turso (libSQL) veritabanı kullanabilir (`TURSO_DATABASE_URL=file:./data/ktunbot.db`).
   - `data/menus/`, `data/schedules/`, `data/calendars/` dizinlerinin var olduğundan emin olun.

3. Çalıştırın
   ```bash
   cargo run
   ```

4. Testler
   ```bash
   cargo test
   ```

## Dağıtım
Canlı ortama almak (VPS vb.) için `deploy/README.deploy.md` rehberine bakabilirsiniz.

## Dosyaları Güncelleme
Eski python sisteminde olduğu gibi PDF ve Menü verilerini güncellemek için `update_files.py` betiğini kullanabilirsiniz:
```bash
python3 update_files.py
```
