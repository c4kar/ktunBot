import os
from config import Config
from bot.utils.semester_detector import get_current_semester

class ScheduleService:
    def get_schedule_path(self):
        """Get the path to the schedule PDF."""
        # Assuming the file is named 'ders_programi.pdf'
        return os.path.join(Config.SCHEDULES_DIR, 'ders_programi.pdf')
    
    def get_semester_info(self) -> str:
        """Get the current semester information."""
        return get_current_semester()
