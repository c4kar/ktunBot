#!/usr/bin/env python3
"""
Manuel Dosya Güncelleme Script'i
Bu script ile kolayca güncel dosyaları indirebilirsiniz.
"""

import os
import sys
import requests
from pathlib import Path
from datetime import datetime
from urllib.parse import urlparse

class FileUpdater:
    def __init__(self):
        self.base_dir = Path(__file__).parent
        self.data_dir = self.base_dir / "data"
        self.schedules_dir = self.data_dir / "schedules"
        self.calendars_dir = self.data_dir / "calendars"
        self.menus_dir = self.data_dir / "menus"
        
        # Klasörlerin var olduğundan emin ol
        self.schedules_dir.mkdir(parents=True, exist_ok=True)
        self.calendars_dir.mkdir(parents=True, exist_ok=True)
        self.menus_dir.mkdir(parents=True, exist_ok=True)
    
    def download_file(self, url, save_path):
        """URL'den dosya indir ve kaydet."""
        try:
            print(f"📥 İndiriliyor: {url}")
            response = requests.get(url, timeout=30)
            response.raise_for_status()
            
            with open(save_path, 'wb') as f:
                f.write(response.content)
            
            # Dosya boyutunu göster
            size_mb = len(response.content) / (1024 * 1024)
            print(f"✅ İndirildi: {save_path.name} ({size_mb:.2f} MB)")
            return True
        except Exception as e:
            print(f"❌ Hata: {e}")
            return False
    
    def get_file_extension(self, url):
        """URL'den dosya uzantısını çıkar."""
        parsed = urlparse(url)
        path = parsed.path
        _, ext = os.path.splitext(path)
        return ext.lower()
    
    def update_menu_image(self, url):
        """Yemekhane menüsü resmini güncelle."""
        print("\n🍽️  Yemekhane Menüsü İndiriliyor...")
        
        # Dosya uzantısını belirle
        ext = self.get_file_extension(url)
        if not ext or ext not in ['.jpg', '.jpeg', '.png', '.pdf']:
            # Varsayılan olarak .jpg kullan
            ext = '.jpg'
        
        # Güncel ay için dosya adı
        current_month = datetime.now().strftime("%Y-%m")
        filename = f"menu_{current_month}{ext}"
        save_path = self.menus_dir / filename
        
        return self.download_file(url, save_path)
    
    def update_schedule_pdf(self, url):
        """EEM ders programını güncelle."""
        print("\n📚 Ders Programı İndiriliyor...")
        
        # Dosya adı
        filename = "ders_programi.pdf"
        save_path = self.schedules_dir / filename
        
        return self.download_file(url, save_path)
    
    def update_calendar_pdf(self, url):
        """Akademik takvimi güncelle."""
        print("\n📅 Akademik Takvim İndiriliyor...")
        
        # Dosya adı
        filename = "akademik_takvim.pdf"
        save_path = self.calendars_dir / filename
        
        return self.download_file(url, save_path)
    
    def run(self):
        """Ana güncelleme işlemi."""
        print("=" * 60)
        print("📁 KTUN BOT - Dosya Güncelleme Aracı")
        print("=" * 60)
        print()
        
        success_count = 0
        total_count = 0
        
        # 1. Yemekhane Menüsü
        print("1️⃣  Güncel ayın yemekhane listesi resmi")
        menu_url = input("   Link girin (boş bırakırsanız atlanır): ").strip()
        
        if menu_url:
            total_count += 1
            if self.update_menu_image(menu_url):
                success_count += 1
        else:
            print("   ⏭️  Atlandı")
        
        # 2. EEM Ders Programı
        print("\n2️⃣  EEM Güncel dönem ders programı PDF'i")
        schedule_url = input("   Link girin (boş bırakırsanız atlanır): ").strip()
        
        if schedule_url:
            total_count += 1
            if self.update_schedule_pdf(schedule_url):
                success_count += 1
        else:
            print("   ⏭️  Atlandı")
        
        # 3. Akademik Takvim
        print("\n3️⃣  Akademik Takvim PDF'i")
        calendar_url = input("   Link girin (boş bırakırsanız atlanır): ").strip()
        
        if calendar_url:
            total_count += 1
            if self.update_calendar_pdf(calendar_url):
                success_count += 1
        else:
            print("   ⏭️  Atlandı")
        
        # Özet
        print("\n" + "=" * 60)
        if total_count > 0:
            print(f"✨ Tamamlandı: {success_count}/{total_count} dosya başarıyla indirildi!")
        else:
            print("ℹ️  Hiçbir dosya indirilmedi.")
        print("=" * 60)
        
        if success_count > 0:
            print("\n📂 İndirilen dosyalar:")
            print(f"   • Menüler: {self.menus_dir}")
            print(f"   • Ders Programları: {self.schedules_dir}")
            print(f"   • Akademik Takvimler: {self.calendars_dir}")


def main():
    try:
        updater = FileUpdater()
        updater.run()
    except KeyboardInterrupt:
        print("\n\n❌ İşlem kullanıcı tarafından iptal edildi.")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Beklenmeyen hata: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
