"""Buggy delivery-id gate. Always accepts, including repeats."""


def accept(delivery_id: str, seen: list[str]) -> bool:
    seen.append(delivery_id)
    return True
