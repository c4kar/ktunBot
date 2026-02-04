# ktunBot

Konya Teknik Üniversitesi öğrencileri için Telegram bot sistemi. PythonAnywhere üzerinde host edilecek.

## Özellikler
- Duyuru takibi
- Ders programı
- Akademik takvim
- Yemekhane menüsü

## Kurulum

1. Gereksinimleri yükleyin:
   ```bash
   pip install -r requirements.txt
   ```

2. `.env` dosyasını düzenleyin ve gerekli bilgileri girin.

3. PDF dosyalarını `data/schedules/` ve `data/calendars/` klasörlerine ekleyin.(update_files.py ile kolayca güncellenebilir.)

5. Botu çalıştırın (Webhook için PythonAnywhere yapılandırması gereklidir).

### TO DO
- [ ] Öğretmen isim araması yapılarak detaylı bilgilere ulaşma.(Sayfası, email, telefon num., oda numarası)
- [ ] Otomatik calender, schedule ve menu scraper.
- [ ] Okulumuzda ne hikmetse yemek takvimi resim üzerinden paylaşıldığı için doğrudan /bugun /yarin gibi komutlarla yemek menülerini öğrenemiyoruz. Otomatik bir şekilde her ay yemek menüsünü kontrol edip paylaşılan tablo görselini bir yapay zeka ile JSONA çevirip komutlara entegre etme fikri var aklımda.
- [ ] Okulumuzda okulumuz hakkında bilgi bilen kişi sayısı bir elin parmağını geçmiyor. Keşke geçse ama en basit bir prosedür hakkında bir çalışana danışdığınızda alacağınız muhtemel tepki şu: "oNa bİz baakmıyoz yanlız istersen öyrenci işlerine sor" :/ Öğrenci işlerinin tepksi de aynı... Bu yüzden okulla ilgili yapacağınız tüm işlemler (Ders seçimi, Yatay Dikey Geçiş, Alttan Üstten Ders alma, Erasmus projeleri, Topluluklar) hakkında yayınlanan belge ve talimatların RAG sistemi ile tercihen ücretsiz bir AI servisi ile telegram botu üzerinden sohbet edilebilmesi. Aklımda böyle bişi var. Bu bahar dönem inşallah yapacağım.
- [ ] 
