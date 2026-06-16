"""Hook 05 — silence lints in the generated crate.

The generated crate is never hand-edited and openapi-generator output is not lint-clean.
Prepend blanket allows to ``client/src/lib.rs`` so the generated code does not pollute the
workspace's lint output.
"""

from __future__ import annotations

from .common import Context

NUMBER = 5
NAME = "silence-lints"


def run(ctx: Context) -> None:
    lib_rs = ctx.client_dir / "src" / "lib.rs"
    lib_text = lib_rs.read_text()
    if "#![allow(clippy::all)]" not in lib_text:
        lib_rs.write_text("#![allow(clippy::all)]\n#![allow(warnings)]\n" + lib_text)
        ctx.log(NAME, "silenced lints in generated client/src/lib.rs")
