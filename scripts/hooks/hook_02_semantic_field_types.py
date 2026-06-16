"""Hook 02 — apply semantic newtypes to generated fields.

openapi-generator treats the spec's plain ``type: string`` semantic scalars
(``ProcessDefinitionId``, ``ElementId``, ``TenantId``, …) as primitive aliases, so every
``$ref`` to them collapses to a bare ``String`` in field positions — unlike the ``*Key``
family, which is defined via ``allOf`` composition and survives as a typed reference.

Hook 01 already generates the newtype for every semantic key. This hook closes the gap by
rewriting the generated struct fields (and their ``new()`` constructor parameters) from
``String`` to the corresponding ``models::<SemanticType>`` newtype, bringing the ``*Id`` /
``*Name`` family up to parity with the ``*Key`` family.

Only fields whose spec property resolves to a semantic type AND whose generated type is a
plain ``String`` shape are rewritten; search-filter fields (``Option<Box<models::
StringFilterProperty>>`` etc.) are left untouched.
"""

from __future__ import annotations

import re

from .common import Context, ref_name, snake_case

NUMBER = 2
NAME = "semantic-field-types"

# The plain-String field shapes we rewrite, longest-first so the regexes don't overlap.
# `{nt}` is substituted with the target newtype path (e.g. `models::ProcessDefinitionId`).
_FIELD_SHAPES = [
    ("Option<Vec<String>>", "Option<Vec<{nt}>>"),
    ("Vec<String>", "Vec<{nt}>"),
    ("Option<String>", "Option<{nt}>"),
    ("String", "{nt}"),
]


def _semantic_type_of(schema: dict | None, semantic: set[str]) -> tuple[str | None, bool]:
    """Resolve a property's value schema to ``(semantic_type, is_array)``.

    Handles a direct ``$ref``, a single-ref ``allOf`` wrapper, an inline ``x-semantic-type``,
    and arrays whose ``items`` are any of the above. Returns ``(None, False)`` when the
    property does not reference a semantic type.
    """
    if not isinstance(schema, dict):
        return None, False

    if schema.get("type") == "array":
        inner, _ = _semantic_type_of(schema.get("items"), semantic)
        return inner, inner is not None

    ref = schema.get("$ref")
    if ref:
        name = ref_name(ref)
        return (name if name in semantic else None), False

    for sub in schema.get("allOf", []):
        if "$ref" in sub:
            name = ref_name(sub["$ref"])
            if name in semantic:
                return name, False

    st = schema.get("x-semantic-type")
    if st and st in semantic:
        return st, False

    return None, False


def _collect_properties(name: str, schemas: dict, seen: set[str] | None = None) -> dict:
    """Collect a schema's effective properties, flattening allOf composition (matching how
    openapi-generator flattens parent properties into the generated struct)."""
    if seen is None:
        seen = set()
    if name in seen:
        return {}
    seen.add(name)
    schema = schemas.get(name, {})
    props: dict = {}
    for sub in schema.get("allOf", []):
        if "$ref" in sub:
            props.update(_collect_properties(ref_name(sub["$ref"]), schemas, seen))
        elif "properties" in sub:
            props.update(sub["properties"])
    if "properties" in schema:
        props.update(schema["properties"])
    return props


def _build_field_map(ctx: Context) -> dict[str, dict[str, tuple[str, bool]]]:
    """Map ``struct_name -> {snake_field -> (semantic_type, is_array)}`` for every schema
    property that references a semantic type."""
    # Resolve only to types that hook 01 actually generated a newtype for. This excludes
    # the lone integer-typed semantic scalar (no string newtype) so we never rewrite a
    # field to a non-existent `models::<T>`.
    semantic = {k["name"] for k in ctx.domain_keys}
    field_map: dict[str, dict[str, tuple[str, bool]]] = {}
    for schema_name in ctx.schemas:
        props = _collect_properties(schema_name, ctx.schemas)
        if not props:
            continue
        fields: dict[str, tuple[str, bool]] = {}
        for prop_name, prop_schema in props.items():
            st, is_array = _semantic_type_of(prop_schema, semantic)
            if st:
                fields[snake_case(prop_name)] = (st, is_array)
        if fields:
            field_map[schema_name] = fields
    return field_map


def _rewrite_field(content: str, snake: str, newtype: str, is_array: bool) -> tuple[str, int]:
    """Rewrite a single field's struct declaration and its ``new()`` parameter."""
    count = 0

    # 1. Struct field declaration: `pub <snake>: <shape>,`
    for old, new_tmpl in _FIELD_SHAPES:
        if is_array and "Vec" not in old:
            continue
        if not is_array and "Vec" in old:
            continue
        pat = re.compile(r"pub %s: %s," % (re.escape(snake), re.escape(old)))
        content, n = pat.subn(f"pub {snake}: {new_tmpl.format(nt=newtype)},", content)
        count += n
        if n:
            break  # a field has exactly one declaration shape

    # 2. `new()` constructor parameter (no `pub`): `<snake>: <shape>`. Nullable-but-required
    #    fields appear as `Option<...>` params, so cover every shape — anchored at the param
    #    name so the shapes stay mutually exclusive.
    for old, new_tmpl in _FIELD_SHAPES:
        if is_array and "Vec" not in old:
            continue
        if not is_array and "Vec" in old:
            continue
        pat = re.compile(r"(?<!pub )(?<![\w])%s: %s(?=[,)\s])" % (re.escape(snake), re.escape(old)))
        content, n = pat.subn(f"{snake}: {new_tmpl.format(nt=newtype)}", content)
        count += n
        if n:
            break

    return content, count


def run(ctx: Context) -> None:
    field_map = _build_field_map(ctx)
    if not field_map:
        ctx.log(NAME, "no semantic field references found (nothing to rewrite)")
        return

    models_dir = ctx.models_dir
    struct_re = re.compile(r"pub struct (\w+)")

    files_changed = 0
    fields_rewritten = 0
    struct_re_pat = struct_re
    for rs in sorted(models_dir.glob("*.rs")):
        if rs.name in ("mod.rs", "camunda_keys.rs"):
            continue
        content = rs.read_text()
        m = struct_re_pat.search(content)
        if not m:
            continue
        struct_name = m.group(1)
        fields = field_map.get(struct_name)
        if not fields:
            continue

        file_count = 0
        for snake, (semtype, is_array) in fields.items():
            newtype = f"models::{semtype}"
            content, n = _rewrite_field(content, snake, newtype, is_array)
            file_count += n

        if file_count:
            rs.write_text(content)
            files_changed += 1
            fields_rewritten += file_count

    ctx.log(
        NAME,
        f"rewrote {fields_rewritten} field/param site(s) across {files_changed} model file(s) "
        f"to use semantic newtypes",
    )
