def parse_config_key(key: str) -> dict:
    return {
        part.split("=")[0]: int(part.split("=")[1])
        for part in key.split(",")
    }


def make_label_from_key(key: str) -> str:
    parts = parse_config_key(key)
    if "windows" in parts:
        return f"{parts['windows']},{parts['size']},{parts['slide']},{parts['events']},{parts['spread']},{parts['event_offset']}"
    return f"{parts['size']},{parts['slide']},{parts['events']},{parts['spread']},{parts['event_offset']}"
