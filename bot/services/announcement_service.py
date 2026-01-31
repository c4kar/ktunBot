from bot.utils.scraper import fetch_page, parse_html
from bot.utils.cache import get_announcement_cache
from config import Config
import logging

logger = logging.getLogger(__name__)

# Singleton instance
_announcement_service = None


def get_announcement_service() -> 'AnnouncementService':
    """Get the singleton AnnouncementService instance."""
    global _announcement_service
    if _announcement_service is None:
        _announcement_service = AnnouncementService()
    return _announcement_service


class AnnouncementService:
    """Service for fetching university announcements with caching."""
    
    def __init__(self):
        self.url = Config.UNIVERSITY_ANNOUNCEMENTS_URL
        self._cache = get_announcement_cache()

    async def get_announcements(self, count=10):
        """Fetch announcements from the university website with caching."""
        cache_key = f"announcements_{count}"
        
        # Check cache first
        cached = self._cache.get(cache_key)
        if cached is not None:
            logger.debug("Returning cached announcements")
            return cached
        
        if not self.url:
            logger.error("UNIVERSITY_ANNOUNCEMENTS_URL not set.")
            return []

        html = await fetch_page(self.url)
        if not html:
            return []

        soup = parse_html(html)
        if not soup:
            return []

        announcements = []
        # KTUN website uses a table structure for announcements
        rows = soup.select('tbody tr')[:count]
        
        for row in rows:
            try:
                cells = row.select('td')
                if len(cells) >= 2:
                    title_cell = cells[0]
                    link_elem = title_cell.select_one('a')
                    date_cell = cells[1]
                    
                    if link_elem:
                        title = link_elem.text.strip()
                        href = link_elem.get('href', '')
                        
                        # Build full URL if relative
                        if href and not href.startswith('http'):
                            full_link = f"https://www.ktun.edu.tr{href}"
                        else:
                            full_link = href
                        
                        announcements.append({
                            'title': title,
                            'date': date_cell.text.strip() if date_cell else '',
                            'link': full_link
                        })
            except Exception as e:
                logger.error(f"Error parsing announcement: {e}")
                continue
        
        # Cache results for 15 minutes
        if announcements:
            self._cache.set(cache_key, announcements, ttl=900)
        
        return announcements
