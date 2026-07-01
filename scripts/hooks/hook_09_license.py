"""Hook 09 — set the generated crate's package metadata for crates.io.

openapi-generator copies the ``license`` field of the generated ``Cargo.toml`` from the
OpenAPI spec's ``info.license.name`` (currently "Camunda License Version 1.0") and emits
no ``repository``/``homepage``/``documentation``. The published SDK is Apache-2.0, so
rewrite the license to a valid SPDX id and add the standard crates.io metadata so the
generated client publishes cleanly and links back to this repo.

Without this, every ``make generate`` would silently revert the client crate's license
(breaking a crates.io publish) and drop the crate-linkage metadata.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 9
NAME = "crate-metadata"

LICENSE = "Apache-2.0"
REPOSITORY = "https://github.com/camunda/orchestration-cluster-api-rust"
HOMEPAGE = REPOSITORY
DOCUMENTATION = "https://docs.rs/camunda-orchestration-api-client"


def _set_field(text: str, key: str, value: str) -> str:
    """Replace ``key = "..."`` if present, else insert it after the ``license`` line."""
    pattern = rf'^{re.escape(key)}\s*=\s*".*"$'
    line = f'{key} = "{value}"'
    if re.search(pattern, text, flags=re.MULTILINE):
        return re.sub(pattern, line, text, count=1, flags=re.MULTILINE)
    return re.sub(
        r'(^license\s*=\s*".*"$)',
        rf'\1\n{line}',
        text,
        count=1,
        flags=re.MULTILINE,
    )


def run(ctx: Context) -> None:
    cargo = ctx.client_dir / "Cargo.toml"
    text = cargo.read_text()
    original = text

    # License first (also anchors where the other fields are inserted).
    if re.search(r'^license\s*=', text, flags=re.MULTILINE):
        text = re.sub(
            r'^license\s*=\s*".*"$',
            f'license = "{LICENSE}"',
            text,
            count=1,
            flags=re.MULTILINE,
        )
    else:
        text = re.sub(
            r'(^name\s*=\s*".*"$)',
            rf'\1\nlicense = "{LICENSE}"',
            text,
            count=1,
            flags=re.MULTILINE,
        )

    text = _set_field(text, "repository", REPOSITORY)
    text = _set_field(text, "homepage", HOMEPAGE)
    text = _set_field(text, "documentation", DOCUMENTATION)

    if text != original:
        cargo.write_text(text)
        ctx.log(NAME, f"set client/Cargo.toml license ({LICENSE}) + repository/homepage/documentation")
