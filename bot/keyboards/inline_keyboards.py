from telegram import InlineKeyboardButton, InlineKeyboardMarkup

def get_main_menu_keyboard():
    """Return the main menu inline keyboard."""
    keyboard = [
        [InlineKeyboardButton("📢 Duyurular", callback_data="announcements"),
         InlineKeyboardButton("📚 Program", callback_data="schedule")],
        [InlineKeyboardButton("📅 Takvim", callback_data="calendar"),
         InlineKeyboardButton("🍽 Menü", callback_data="cafeteria")],
        [InlineKeyboardButton("ℹ️ Hakkında", callback_data="about")]
    ]
    return InlineKeyboardMarkup(keyboard)

def get_calendar_keyboard():
    """Return the calendar term selection keyboard."""
    keyboard = [
        [InlineKeyboardButton("🍂 Güz Dönemi", callback_data="calendar_fall"),
         InlineKeyboardButton("🌸 Bahar Dönemi", callback_data="calendar_spring")]
    ]
    return InlineKeyboardMarkup(keyboard)
