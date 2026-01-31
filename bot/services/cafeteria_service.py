import json
from datetime import datetime
from pathlib import Path
import logging
import aiofiles
from bot.utils.cache import get_menu_cache

logger = logging.getLogger(__name__)

# Turkish month names mapping
MONTHS_TR = {
    1: 'Ocak', 2: 'Şubat', 3: 'Mart', 4: 'Nisan', 5: 'Mayıs', 6: 'Haziran',
    7: 'Temmuz', 8: 'Ağustos', 9: 'Eylül', 10: 'Ekim', 11: 'Kasım', 12: 'Aralık'
}

# Singleton instance
_cafeteria_service = None


def get_cafeteria_service() -> 'CafeteriaService':
    """Get the singleton CafeteriaService instance."""
    global _cafeteria_service
    if _cafeteria_service is None:
        _cafeteria_service = CafeteriaService()
    return _cafeteria_service


class CafeteriaService:
    """Service for fetching cafeteria menus from JSON files with caching."""
    
    def __init__(self):
        self.base_dir = Path(__file__).parent.parent.parent
        self.menus_dir = self.base_dir / "data" / "menus"
        self._cache = get_menu_cache()

    def _get_menu_file_path(self, date: datetime) -> Path:
        """Get the JSON file path for a specific month."""
        filename = f"{date.year}-{date.month:02d}.json"
        return self.menus_dir / filename
    
    def _get_menu_image_path(self, date: datetime) -> Path | None:
        """Get the image file path for a specific month."""
        for ext in ['jpg', 'jpeg', 'png']:
            filename = f"menu_{date.year}-{date.month:02d}.{ext}"
            path = self.menus_dir / filename
            if path.exists():
                return path
        return None

    def _convert_date_format(self, date: datetime) -> str:
        """Convert datetime to the format used in JSON (e.g., '01 Aralık')."""
        day = f"{date.day:02d}"
        month = MONTHS_TR[date.month]
        return f"{day} {month}"

    async def _load_menu_from_json(self, date: datetime) -> list:
        """Load menu data from JSON file for the given month with caching."""
        month_key = f"menu_{date.year}-{date.month:02d}"
        
        # Check cache first
        cached = self._cache.get(month_key)
        if cached is not None:
            return cached
        
        file_path = self._get_menu_file_path(date)
        
        if not file_path.exists():
            logger.warning(f"Menu file not found: {file_path}")
            return []
        
        try:
            async with aiofiles.open(file_path, 'r', encoding='utf-8') as f:
                content = await f.read()
                data = json.loads(content)
                menu_list = data.get('menu', [])
                # Cache for 2 hours
                self._cache.set(month_key, menu_list, ttl=7200)
                logger.info(f"Loaded {len(menu_list)} menu items from {file_path}")
                return menu_list
        except (json.JSONDecodeError, IOError) as e:
            logger.error(f"Error loading menu from {file_path}: {e}")
            return []

    async def get_menu_for_date(self, date: datetime):
        """Fetch menu for a specific date. Returns image path if available, otherwise JSON data."""
        # First, check if there's an image file for this month
        image_path = self._get_menu_image_path(date)
        if image_path:
            return {
                "type": "image",
                "path": str(image_path),
                "date": date.strftime('%d.%m.%Y')
            }
        
        # If no image, fall back to JSON
        menu_list = await self._load_menu_from_json(date)
        
        if not menu_list:
            return None
        
        target_date_str = self._convert_date_format(date)
        
        for menu_item in menu_list:
            if menu_item.get('date') == target_date_str:
                # Check if cafeteria is closed
                if menu_item.get('closed', False):
                    return {
                        "type": "json",
                        "date": menu_item['date'],
                        "dayOfWeek": menu_item.get('dayOfWeek', ''),
                        "closed": True,
                        "closedReason": menu_item.get('closedReason', 'Kapalı'),
                        "foods": [],
                        "totalcalorie": "",
                    }
                
                return {
                    "type": "json",
                    "date": menu_item['date'],
                    "dayOfWeek": menu_item.get('dayOfWeek', ''),
                    "closed": False,
                    "foods": menu_item.get('foods', []),
                    "totalcalorie": menu_item.get('totalcalorie', ''),
                    "mealType": menu_item.get('mealType', ''),
                }
        
        return None

    def get_available_months(self) -> list:
        """List all available menu months."""
        if not self.menus_dir.exists():
            return []
        
        months = []
        for file in self.menus_dir.glob("*.json"):
            try:
                parts = file.stem.split('-')
                if len(parts) == 2:
                    year, month = int(parts[0]), int(parts[1])
                    months.append({"year": year, "month": month})
            except ValueError:
                continue
        
        return sorted(months, key=lambda x: (x['year'], x['month']), reverse=True)
