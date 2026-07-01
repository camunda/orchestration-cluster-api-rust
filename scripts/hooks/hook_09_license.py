"""Hook 09 — set the generated crate's license to ``Apache-2.0``.

openapi-generator copies the ``license`` field of the generated ``Cargo.toml`` from the
OpenAPI spec's ``info.license.name`` (currently "Camunda License Version 1.0"). The
published SDK is Apache-2.0, so rewrite the generated client's license to match the
workspace root. Without this, every ``make generate`` would silently revert the client
crate's license and break a crates.io publish (the license must be a valid SPDX id).
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 9
NAME = "license"

SPDX = "Apache-2.0"


def run(ctx: Context) -> None:
    cargo = ctx.client_dir / "Cargo.toml"
    cargo_text = cargo.read_text()
    new_text, n = re.subn(
        r'^license\s*=\s*".*"$',
        f'license = "{SPDX}"',
        cargo_text,
        count=1,
        flags=re.MULTILINE,
    )
    if n == 0:
        # No license line at all — add one after the package name.
        new_text = re.sub(
            r'(^name\s*=\s*".*"$)',
            rf'\1\nlicense = "{SPDX}"',
            cargo_text,
            count=1,
            flags=re.MULTILINE,
        )
    if new_text != cargo_text:
        cargo.write_text(new_text)
        ctx.log(NAME, f"set client/Cargo.toml license to {SPDX}")
