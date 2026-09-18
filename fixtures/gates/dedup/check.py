"""Protected deterministic corpus oracle; live untrusted execution is separately gated."""
import importlib.util
import json
import sys
from pathlib import Path


def main():
    src = Path(sys.argv[1]) / "src" / "dedup.py"
    spec = importlib.util.spec_from_file_location("candidate", src)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    tests = []
    seen = []
    cases = [("first_delivery", "a", True), ("repeated_delivery", "a", False),
             ("distinct_delivery", "b", True)]
    for name, item, expected in cases:
        result = {"name": name, "executed": True, "assertions": 0,
                  "skipped": False, "passed": False}
        try:
            actual = module.accept(item, seen)
            result["assertions"] += 1
            result["passed"] = actual is expected
        except BaseException as error:
            result["error"] = type(error).__name__
        tests.append(result)
    print(json.dumps({"tests": tests, "complete": True}))
    return 0 if all(t["passed"] for t in tests) else 1


if __name__ == "__main__":
    raise SystemExit(main())
