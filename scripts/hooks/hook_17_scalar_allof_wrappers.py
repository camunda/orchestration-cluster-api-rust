"""Hook 15 — transparent wrappers for scalar ``allOf`` single-ref schemas.

A schema shaped ``{type: string, allOf: [<single $ref>]}`` with no own ``properties`` is
an *exact-match* wrapper: it means "a value of the referenced type". openapi-generator
collapses it to an **empty struct** (``pub struct FooExactMatch {}``) that serializes as
``{}`` and refuses the referenced value — so the ``*ExactMatch`` branch of every
``*FilterProperty`` union is unusable (it serializes nothing and, as the first untagged
branch, greedily swallows the sibling advanced-filter object).

The Domain Type System hook (01) already rewrites the wrappers whose inner type is a
*string scalar* semantic key (``JobKeyExactMatch`` etc.). This hook covers the remaining
class: wrappers whose inner type is a **string enum** (``ProtectionModeEnum``,
``JobStateEnum``, …). Each such empty struct is replaced with a ``#[serde(transparent)]``
newtype over the referenced enum, so it serializes as the bare enum value (``"REDACT"``)
and round-trips correctly.
"""

from __future__ import annotations

import re

from .common import Context, ref_name, snake_case

NUMBER = 17
NAME = "scalar-allof-wrappers"


def _empty_struct_re(name: str) -> re.Pattern[str]:
    """Match the generator's empty-struct form for ``name`` tolerant of any whitespace
    between the braces. The hooks run on *unformatted* generator output (post-processing
    precedes ``cargo fmt`` in ``scripts/generate.sh``), where the generator emits
    ``pub struct Foo {\\n}`` — the collapsed ``pub struct Foo {}`` only appears after
    formatting, so a literal match on the formatted form silently skips every wrapper on
    a normal regeneration and lets the broken empty models survive."""
    return re.compile(rf"pub struct {re.escape(name)}\s*\{{\s*\}}")


def _is_string_enum(schema: dict | None) -> bool:
    return bool(schema) and schema.get("type") == "string" and "enum" in schema


def _single_allof_ref(schema: dict) -> str | None:
    """Return the single inner $ref name for an ``allOf: [{$ref}]`` wrapper that has no
    own ``properties``, else None."""
    if not isinstance(schema, dict) or "properties" in schema:
        return None
    allof = schema.get("allOf")
    if not isinstance(allof, list) or len(allof) != 1:
        return None
    ref = allof[0].get("$ref") if isinstance(allof[0], dict) else None
    return ref_name(ref) if ref else None


HEADER = """\
/*
 * Orchestration Cluster API
 *
 * @generated transparent wrapper (scripts/postprocess.py hook 17) — DO NOT EDIT.
 */

use crate::models;
use serde::{Deserialize, Serialize};
"""


def _render(name: str, inner: str, schema: dict) -> str:
    desc = (schema.get("description") or schema.get("title") or "").strip()
    desc = " ".join(desc.split())
    lines = [HEADER, ""]
    if desc:
        lines.append(f"/// {name} : {desc}")
    lines.append("#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]")
    lines.append("#[serde(transparent)]")
    lines.append(f"pub struct {name}(pub models::{inner});")
    lines.append("")
    lines.append(f"impl {name} {{")
    if desc:
        lines.append(f"    /// {desc}")
    lines.append(f"    pub fn new(value: models::{inner}) -> {name} {{")
    lines.append(f"        {name}(value)")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def run(ctx: Context) -> None:
    models_dir = ctx.models_dir
    rewritten = 0
    for name, schema in ctx.schemas.items():
        if _is_string_enum(schema):
            continue
        inner = _single_allof_ref(schema)
        if not inner or not _is_string_enum(ctx.schemas.get(inner)):
            continue
        f = models_dir / f"{snake_case(name)}.rs"
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8")
        # Only touch the broken empty-struct form the generator emits; stay idempotent.
        # Whitespace-tolerant so it fires on unformatted generator output, not just the
        # post-`cargo fmt` collapsed form.
        if not _empty_struct_re(name).search(text):
            continue
        f.write_text(_render(name, inner, schema), encoding="utf-8")
        rewritten += 1
    ctx.log(NAME, f"rewrote {rewritten} enum-backed exact-match wrapper(s) as transparent newtypes")
