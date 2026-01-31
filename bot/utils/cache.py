import time
from collections import OrderedDict
from threading import Lock
from typing import Any, Optional


class LRUCache:
    """Lightweight LRU cache with TTL support for PythonAnywhere."""
    
    def __init__(self, max_size: int = 30, default_ttl: int = 3600):
        self._cache: OrderedDict[str, tuple[Any, float]] = OrderedDict()
        self._max_size = max_size
        self._default_ttl = default_ttl
        self._lock = Lock()
    
    def get(self, key: str) -> Optional[Any]:
        """Get value from cache if exists and not expired."""
        with self._lock:
            if key not in self._cache:
                return None
            
            value, expiry = self._cache[key]
            if time.time() > expiry:
                del self._cache[key]
                return None
            
            # Move to end (most recently used)
            self._cache.move_to_end(key)
            return value
    
    def set(self, key: str, value: Any, ttl: Optional[int] = None) -> None:
        """Set value in cache with optional TTL."""
        with self._lock:
            expiry = time.time() + (ttl or self._default_ttl)
            
            if key in self._cache:
                del self._cache[key]
            elif len(self._cache) >= self._max_size:
                # Remove oldest (least recently used)
                self._cache.popitem(last=False)
            
            self._cache[key] = (value, expiry)
    
    def clear(self) -> None:
        """Clear all cache entries."""
        with self._lock:
            self._cache.clear()
    
    @property
    def size(self) -> int:
        """Get current cache size."""
        return len(self._cache)


# Global cache instances (memory-efficient sizes for PythonAnywhere)
_menu_cache = LRUCache(max_size=15, default_ttl=7200)      # 2 saat
_announcement_cache = LRUCache(max_size=5, default_ttl=900)  # 15 dk


def get_menu_cache() -> LRUCache:
    """Get the global menu cache instance."""
    return _menu_cache


def get_announcement_cache() -> LRUCache:
    """Get the global announcement cache instance."""
    return _announcement_cache
