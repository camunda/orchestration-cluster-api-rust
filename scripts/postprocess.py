#!/usr/bin/env python3
"""Orchestrate the numbered post-processing hooks over the generated client crate.

openapi-generator produces a low-level Rust client that is correct in shape but missing the
Camunda Domain Type System and carrying a few generator quirks. Rather than one monolith,
each concern lives in its own hook under ``scripts/hooks/`` exposing ``NUMBER``, ``NAME`` and
``run(ctx)``. This orchestrator builds the shared :class:`Context` once (parsing the spec and
discovering the domain keys) and runs every hook in ``NUMBER`` order.

Hooks (in order):
  01 domain-type-system          emit camunda_keys.rs + wire models/mod.rs
  02 semantic-field-types        rewrite collapsed `String` fields to semantic newtypes
  03 module-paths-and-placeholders fix `models::models::` + bare `Object`
  04 regex-dependency            ensure client/Cargo.toml depends on regex
  05 silence-lints               blanket-allow lints in generated lib.rs
  06 cleanup-scaffolding         drop generator scaffolding files

Usage:
    python3 scripts/postprocess.py --client-dir client \\
        --spec external-spec/bundled/rest-api.bundle.json \\
        --metadata external-spec/bundled/spec-metadata.json
"""

from __future__ import annotations

import argparse
import importlib
import pkgutil
import sys
from pathlib import Path

# Allow running as a plain script (`python3 scripts/postprocess.py`) by making the repo's
# `scripts` directory importable as a package root.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.hooks.common import Context  # noqa: E402
from scripts import hooks as hooks_pkg  # noqa: E402


def _discover_hooks() -> list:
    """Import every ``hook_*`` module and return them sorted by ``NUMBER``."""
    found = []
    for mod in pkgutil.iter_modules(hooks_pkg.__path__):
        if not mod.name.startswith("hook_"):
            continue
        module = importlib.import_module(f"scripts.hooks.{mod.name}")
        if not all(hasattr(module, a) for a in ("NUMBER", "NAME", "run")):
            print(f"[postprocess] WARNING: {mod.name} is not a valid hook (skipped)", file=sys.stderr)
            continue
        found.append(module)
    found.sort(key=lambda m: m.NUMBER)
    return found


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--client-dir", required=True)
    ap.add_argument("--spec", required=True)
    ap.add_argument("--metadata", required=False)
    args = ap.parse_args()

    ctx = Context.build(
        client_dir=Path(args.client_dir),
        spec_path=Path(args.spec),
        metadata_path=Path(args.metadata) if args.metadata else None,
    )

    hooks = _discover_hooks()
    if not hooks:
        print("[postprocess] ERROR: no hooks discovered", file=sys.stderr)
        return 1

    print(f"[postprocess] running {len(hooks)} hook(s): "
          + ", ".join(f"{h.NUMBER:02d}:{h.NAME}" for h in hooks))
    for h in hooks:
        h.run(ctx)
    print("[postprocess] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
