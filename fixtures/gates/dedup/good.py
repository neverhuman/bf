"""Known-good delivery-id gate. First accept, repeat reject."""


def accept(delivery_id: str, seen: list[str]) -> bool:
    if delivery_id in seen:
        return False
    seen.append(delivery_id)
    return True
