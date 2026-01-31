def format_announcement(announcement):
    """Format announcement for Telegram."""
    return f"<a href='{announcement['link']}'>{announcement['title']}</a>\n📅 {announcement['date']}"

def format_menu(menu_data, date_str):
    """Format menu for Telegram."""
    
    # Check if cafeteria is closed
    if menu_data.get('closed', False):
        day_of_week = menu_data.get('dayOfWeek', '')
        reason = menu_data.get('closedReason', 'Kapalı')
        
        text = f"📅 <b>{menu_data.get('date', date_str)}</b>"
        if day_of_week:
            text += f" ({day_of_week})"
        text += "\n\n"
        text += f"🚫 <b>Yemekhane Kapalı</b>\n"
        text += f"📝 <i>{reason}</i>"
        return text
    
    # Build header with date and day of week
    day_of_week = menu_data.get('dayOfWeek', '')
    text = f"📅 <b>{menu_data.get('date', date_str)}</b>"
    if day_of_week:
        text += f" ({day_of_week})"
    text += "\n\n"
    
    # Add meal type if available
    meal_type = menu_data.get('mealType', '')
    if meal_type:
        meal_emoji = "🌅" if meal_type == "kahvaltı" else "🍽" if meal_type == "öğle" else "🌙"
        text += f"{meal_emoji} <b>{meal_type.capitalize()} Yemeği</b>\n\n"
    
    # Add foods list - new structure uses 'foods' directly
    foods = menu_data.get('foods', [])
    if foods:
        for i, food in enumerate(foods, 1):
            # Add emoji based on food type
            emoji = get_food_emoji(food)
            text += f"{emoji} {food}\n"
        text += "\n"
    
    # Legacy support for breakfast/lunch/dinner structure
    if menu_data.get('breakfast'):
        text += "🌅 <b>Kahvaltı:</b>\n" + "\n".join(f"• {i}" for i in menu_data['breakfast']) + "\n\n"
    
    if menu_data.get('lunch'):
        text += "🍽 <b>Öğle:</b>\n" + "\n".join(f"• {i}" for i in menu_data['lunch']) + "\n\n"
        
    if menu_data.get('dinner'):
        text += "🌙 <b>Akşam:</b>\n" + "\n".join(f"• {i}" for i in menu_data['dinner']) + "\n\n"
    
    # Add calorie info
    total_calorie = menu_data.get('totalcalorie', '')
    if total_calorie:
        text += f"🔥 <b>Toplam Kalori:</b> {total_calorie}\n"
        
    return text.strip()

def get_food_emoji(food_name: str) -> str:
    """Get appropriate emoji for food item based on its name."""
    food_lower = food_name.lower()
    
    # Soups
    if 'çorba' in food_lower:
        return "🍲"
    
    # Rice/Pilav
    if 'pilav' in food_lower or 'pirinç' in food_lower:
        return "🍚"
    
    # Pasta
    if 'makarna' in food_lower or 'spagetti' in food_lower or 'erişte' in food_lower:
        return "🍝"
    
    # Meat dishes
    if any(word in food_lower for word in ['köfte', 'kebap', 'döner', 'tavuk', 'et', 'piliç', 'balık', 'burger', 'tantuni']):
        return "🍖"
    
    # Vegetables/Legumes
    if any(word in food_lower for word in ['fasulye', 'nohut', 'mercimek', 'patlıcan', 'biber', 'dolma', 'börek', 'ispanak', 'karnabahar']):
        return "🥗"
    
    # Desserts
    if any(word in food_lower for word in ['tatlı', 'helva', 'revani', 'pasta', 'supangle', 'aşure', 'höşmerim', 'sütlaç']):
        return "🍰"
    
    # Dairy/Yogurt
    if any(word in food_lower for word in ['yoğurt', 'cacık', 'ayran']):
        return "🥛"
    
    # Salad
    if 'salata' in food_lower or 'söğüş' in food_lower or 'yeşillik' in food_lower:
        return "🥬"
    
    # Fruits
    if 'meyve' in food_lower:
        return "🍎"
    
    # Beverages
    if 'içecek' in food_lower:
        return "🥤"
    
    # Default
    return "•"
