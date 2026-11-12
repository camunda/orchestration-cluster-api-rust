#!/usr/bin/env python3
"""Prepare the bundled spec for openapi-generator.

The committed bundle (``external-spec/bundled/rest-api.bundle.json``) stays a faithful
copy of upstream. A few shapes in it make openapi-generator emit unusable Rust, and are
easier to fix in the generator *input* than in its output. This writes a derived spec
(gitignored) that ``openapi-generator-config.yaml`` reads instead:

* ``x-clearable`` date-times (``ClearableDateTime``) are rewritten to nullable
  ``date-time`` properties and the clearable component schemas dropped, so no empty
  ``ClearableDateTime`` struct is generated. Hook 19 then makes the fields clearable.

Usage:
    python3 scripts/preprocess.py \\
        --spec external-spec/bundled/rest-api.bundle.json \\
        --out external-spec/generator/rest-api.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.hooks.hook_19_clearable_date_time import prepare_spec  # noqa: E402


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--spec", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    spec = json.loads(Path(args.spec).read_text(encoding="utf-8"))
    prepared, fields = prepare_spec(spec)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(prepared, indent=2) + "\n", encoding="utf-8")
    listed = ", ".join(f"{m}.{p}" for m, ps in sorted(fields.items()) for p in ps) or "none"
    print(f"[preprocess] clearable date-time fields: {listed}")
    print(f"[preprocess] wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
