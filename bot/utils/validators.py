def validate_count(count_arg):
    """Validate announcement count argument."""
    try:
        count = int(count_arg)
        return max(1, min(count, 50))
    except (ValueError, TypeError):
        return 10
