def parse_config_key(key: str) -> dict:
    return {
        part.split("=")[0]: int(part.split("=")[1])
        for part in key.split(",")
    }


def make_label_from_key(key: str) -> str:
    parts = parse_config_key(key)
    # Use whatever label format you want; this matches your "1,size,slide,events" idea
    return f"{parts['windows']},{parts['size']},{parts['slide']},{parts['events']}"
