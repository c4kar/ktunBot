from datetime import datetime

def get_current_semester() -> str:
    """
    Türkiye'deki akademik takvime göre mevcut dönemi tespit eder.
    
    Dönem Takvimi:
    - Güz Dönemi: Eylül (9) - Ocak (1)
    - Bahar Dönemi: Şubat (2) - Haziran (6)
    - Yaz Dönemi: Temmuz (7) - Ağustos (8)
    
    Returns:
        str: Örnek: "2025-2026 Bahar Dönemi"
    """
    now = datetime.now()
    month = now.month
    year = now.year
    
    if month == 1:
        # Ocak: Güz Dönemi (önceki yıl - şu anki yıl)
        return f"{year - 1}-{year} Güz Dönemi"
    elif 2 <= month <= 6:
        # Şubat-Haziran: Bahar Dönemi (önceki yıl - şu anki yıl)
        return f"{year - 1}-{year} Bahar Dönemi"
    elif 7 <= month <= 8:
        # Temmuz-Ağustos: Yaz Dönemi (şu anki yıl)
        return f"{year} Yaz Dönemi"
    else:
        # Eylül-Aralık: Güz Dönemi (şu anki yıl - sonraki yıl)
        return f"{year}-{year + 1} Güz Dönemi"
