"""Hook 16 — order untagged enum variants by specificity.

openapi-generator emits ``oneOf`` polymorphic schemas as ``#[serde(untagged)]`` Rust
enums, trying each variant in **spec declaration order**. serde's untagged matching stops
at the first variant that deserializes, and the generated variant structs *ignore* unknown
fields (they must, to tolerate forward-compatible server additions — see hook 08). So when
a *less-specific* variant is declared before a *more-specific* one whose required fields
are a superset, a payload meant for the specific variant silently decodes as the general
one and loses the extra fields.

Concretely, ``ProcessInstanceCreationInstruction`` lists ``ById`` (requires
``processDefinitionId``) before ``ByVersionTag`` (requires ``processDefinitionId`` **and**
``processDefinitionVersionTag``): ``{"processDefinitionId":"x","processDefinitionVersionTag":"v2"}``
decodes as ``ById`` and drops the tag.

This hook stably reorders the variants of every generated untagged enum by *descending*
count of required properties, so a superset variant is always tried before its subset. The
``Default`` impl names its variant explicitly and is left untouched.
"""

from __future__ import annotations

import re

from .common import Context, snake_case

NUMBER = 18
NAME = "untagged-variant-ordering"

_VARIANT_RE = re.compile(r"(\w+)\(\s*Box<models::(\w+)>\s*,?\s*\)")


def _required_count(name: str, schemas: dict, seen: set[str] | None = None) -> int:
    """Count the distinct required properties of a model, merging ``allOf`` bases."""
    if seen is None:
        seen = set()
    schema = schemas.get(name)
    if not isinstance(schema, dict) or name in seen:
        return 0
    seen.add(name)
    required: set[str] = set(schema.get("required", []) or [])
    for sub in schema.get("allOf", []) or []:
        if isinstance(sub, dict) and sub.get("$ref"):
            base = sub["$ref"].split("/")[-1]
            b = schemas.get(base)
            if isinstance(b, dict):
                required |= set(b.get("required", []) or [])
    return len(required)


def _reorder_enum(text: str, schemas: dict) -> str | None:
    m = re.search(r"#\[serde\(untagged\)\]\s*pub enum (\w+) \{(.*?)\n\}", text, re.DOTALL)
    if not m:
        return None
    body = m.group(2)
    variants = _VARIANT_RE.findall(body)
    if len(variants) < 2:
        return None
    # Bail out unless every variant is the simple `Name(Box<models::Inner>)` shape, so we
    # never mangle an enum carrying a variant we did not parse.
    if len(variants) != len(re.findall(r"\b\w+\(", body)):
        return None
    ordered = sorted(
        variants,
        key=lambda v: _required_count(v[1], schemas),
        reverse=True,
    )
    if ordered == variants:
        return None
    rebuilt = "\n".join(f"    {name}(Box<models::{inner}>)," for name, inner in ordered)
    new_block = f"#[serde(untagged)]\npub enum {m.group(1)} {{\n{rebuilt}\n}}"
    return text[: m.start()] + new_block + text[m.end():]


def run(ctx: Context) -> None:
    models_dir = ctx.models_dir
    reordered = 0
    for f in sorted(models_dir.glob("*.rs")):
        text = f.read_text(encoding="utf-8")
        if "#[serde(untagged)]" not in text:
            continue
        new = _reorder_enum(text, ctx.schemas)
        if new and new != text:
            f.write_text(new, encoding="utf-8")
            reordered += 1
    ctx.log(NAME, f"reordered variants of {reordered} untagged enum(s) by specificity")
