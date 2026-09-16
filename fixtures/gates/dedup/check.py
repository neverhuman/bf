#!/usr/bin/env python3
"""Trusted oracle for the dedup gate. Lives outside the candidate."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def load(path: Path):
    spec = importlib.util.spec_from_file_location("dedup", path)
    if spec is None or spec.loader is None:
        raise SystemExit("incomplete")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check.py <workspace>", file=sys.stderr)
        return 2
    workspace = Path(sys.argv[1])
    src = workspace / "src" / "dedup.py"
    if not src.is_file():
        print("FAIL missing src/dedup.py")
        return 1
    mod = load(src)
    if not hasattr(mod, "accept"):
        print("FAIL missing accept")
        return 1
    seen: list[str] = []
    first = mod.accept("a", seen)
    second = mod.accept("a", seen)
    if first is True and second is False:
        print("PASS")
        return 0
    print(f"FAIL first={first!r} second={second!r}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
