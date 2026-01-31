import os
from config import Config

class CalendarService:
    def get_calendar_path(self):
        """Get the path to the academic calendar PDF."""
        # Assuming the file is named 'akademik_takvim.pdf'
        return os.path.join(Config.CALENDARS_DIR, 'akademik_takvim.pdf')
