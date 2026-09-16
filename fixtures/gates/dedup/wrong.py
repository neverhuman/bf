"""Injected defect: still always accepts. Used by AT-032."""


def accept(delivery_id: str, seen: list[str]) -> bool:
    seen.append(delivery_id)
    return True
