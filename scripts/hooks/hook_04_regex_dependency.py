"""Hook 04 — ensure the generated crate depends on ``regex``.

The Domain Type System (hook 01) validates key values against the spec's ``pattern``
constraints using ``regex``. openapi-generator does not declare that dependency, so add it
to the generated crate's ``Cargo.toml`` if missing.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 4
NAME = "regex-dependency"


def run(ctx: Context) -> None:
    cargo = ctx.client_dir / "Cargo.toml"
    cargo_text = cargo.read_text()
    if re.search(r"^regex\s*=", cargo_text, re.MULTILINE) is None:
        cargo_text = re.sub(
            r"(\[dependencies\]\n)",
            r'\1regex = "^1"\n',
            cargo_text,
            count=1,
        )
        cargo.write_text(cargo_text)
        ctx.log(NAME, "added regex dependency to client/Cargo.toml")
