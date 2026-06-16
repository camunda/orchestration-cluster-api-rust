"""Hook 03 — fix generator path bugs and placeholder types.

Two independent fixups for openapi-generator output:

* The doubled module path bug (``models::models::X`` -> ``models::X``), which the generator
  emits for some cross-referenced models.
* The bare ``Object`` placeholder the generator emits for unresolved free-form object
  schemas, which is not a real Rust type — replace it with ``serde_json::Value``.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 3
NAME = "module-paths-and-placeholders"


def run(ctx: Context) -> None:
    client_dir = ctx.client_dir

    # Fix the doubled module-path bug (`models::models::X` -> `models::X`).
    fixed_files = 0
    for rs in client_dir.glob("src/**/*.rs"):
        content = rs.read_text()
        if "models::models::" in content:
            rs.write_text(content.replace("models::models::", "models::"))
            fixed_files += 1
    if fixed_files:
        ctx.log(NAME, f"fixed doubled module path in {fixed_files} file(s)")

    # Replace the generator's bare `Object` placeholder with `serde_json::Value`.
    object_fixed = 0
    for rs in client_dir.glob("src/**/*.rs"):
        content = rs.read_text()
        new = re.sub(r"<Object>", "<serde_json::Value>", content)
        new = re.sub(r": Object\b", ": serde_json::Value", new)
        if new != content:
            rs.write_text(new)
            object_fixed += 1
    if object_fixed:
        ctx.log(NAME, f"replaced bare `Object` placeholder in {object_fixed} file(s)")
