import aiohttp
from bs4 import BeautifulSoup
import logging
from typing import Optional

logger = logging.getLogger(__name__)

# Singleton session
_session: Optional[aiohttp.ClientSession] = None


async def get_session() -> aiohttp.ClientSession:
    """Get or create a singleton aiohttp session."""
    global _session
    if _session is None or _session.closed:
        connector = aiohttp.TCPConnector(limit=10)
        _session = aiohttp.ClientSession(
            connector=connector,
            headers={'User-Agent': 'KTUN-Bot/1.0'},
            timeout=aiohttp.ClientTimeout(total=15)
        )
    return _session


async def close_session():
    """Close the aiohttp session."""
    global _session
    if _session and not _session.closed:
        await _session.close()
        _session = None


async def fetch_page(url: str, timeout: int = 15) -> Optional[str]:
    """Fetch page content asynchronously."""
    try:
        session = await get_session()
        async with session.get(url, timeout=aiohttp.ClientTimeout(total=timeout)) as response:
            response.raise_for_status()
            return await response.text()
    except Exception as e:
        logger.warning(f"Error fetching {url}: {e}")
        return None


def parse_html(html_content: str) -> Optional[BeautifulSoup]:
    """Parse HTML with lxml for better performance."""
    if not html_content:
        return None
    try:
        return BeautifulSoup(html_content, 'lxml')
    except Exception:
        # Fallback to html.parser if lxml fails
        return BeautifulSoup(html_content, 'html.parser')
