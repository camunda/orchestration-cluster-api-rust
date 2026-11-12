"""Hook 19 — clearable date-time fields.

The API clears some date attributes with an **empty string** while ``null`` or omitting
the property preserves the persisted value: ``dueDate`` / ``followUpDate`` on
``JobResultCorrections`` (user task listener corrections) and on the user-task update
``Changeset``. A ``format: date-time`` field cannot carry ``""``, so the generated client
could not express a clear at all.

The spec marks these fields in one of two ways, and both are detected:

* **``x-clearable``** (camunda/camunda#68710): the property is (or ``$ref``s, directly or
  through a single-element ``allOf``) a schema such as ``ClearableDateTime`` —
  ``anyOf: [date-time, maxLength: 0]`` with ``x-clearable: true``. openapi-generator turns
  that schema into an empty ``pub struct ClearableDateTime {}``, which makes the fields
  unusable. ``scripts/preprocess.py`` therefore rewrites every such property to a plain
  nullable ``date-time`` (keeping ``x-clearable: true`` on it) and drops the then-unused
  clearable component schemas *before* generation; see :func:`prepare_spec`.
* **The older shape**: a nullable ``format: date-time`` property whose schema description
  carries a bullet "`` `<field>` - clear by providing an empty String``" (or ``reset``), or
  whose own description says "Reset by providing an empty String".

Either way the generator emits the field as ``Option<Option<chrono::DateTime<..>>>`` with
``serde_with::rust::double_option``. The server treats ``null`` exactly like omission, so
the ``Some(None)`` state carried no meaning of its own; this hook gives it the clear:

* ``None`` — omitted (preserve), ``Some(Some(date))`` — RFC 3339 (set),
  ``Some(None)`` — ``""`` (clear). Reading back, ``""`` is a clear and ``null`` preserve.
* It emits ``client/src/clearable.rs`` (the serde module) and wires it into ``lib.rs``,
  points each field's ``with =`` at it, and adds ``clear_<field>()`` /
  ``is_<field>_cleared()`` to the model.

The field type does not change, so existing code keeps compiling. A detected field whose
generated code does not have the expected shape fails the run rather than being skipped.
"""

from __future__ import annotations

import copy
import re

from .common import Context, ref_name, snake_case

NUMBER = 19
NAME = "clearable-date-time"

_SCHEMA_BULLET = re.compile(
    r"`(?P<field>\w+)`\s*-\s*(?:clear|reset) by providing an empty string", re.IGNORECASE
)
_PROPERTY_NOTE = re.compile(r"\b(?:clear|reset) by providing an empty string", re.IGNORECASE)

DATE_TIME_TYPE = "Option<Option<chrono::DateTime<chrono::FixedOffset>>>"
DOUBLE_OPTION = "::serde_with::rust::double_option"
CLEARABLE_WITH = "crate::clearable::date_time"
ACCESSOR_MARKER = "clearable date-time accessors (scripts/postprocess.py hook 19)"


def _fail(message: str) -> None:
    raise SystemExit(f"[postprocess:{NAME}] ERROR: {message}")


def _ref_target(node: dict) -> str | None:
    """The schema name ``node`` points at: a ``$ref``, or a single-element ``allOf``
    holding one. Anything else (including a multi-element ``allOf``) is not an alias."""
    if not isinstance(node, dict):
        return None
    if node.get("$ref"):
        return ref_name(node["$ref"])
    allof = node.get("allOf")
    if isinstance(allof, list) and len(allof) == 1 and isinstance(allof[0], dict):
        if allof[0].get("$ref"):
            return ref_name(allof[0]["$ref"])
    return None


def _clearable_schema(node: dict, schemas: dict) -> dict | None:
    """The ``x-clearable`` schema ``node`` is or aliases (following ``$ref`` chains), or None."""
    seen: set[str] = set()
    while isinstance(node, dict):
        if node.get("x-clearable") is True:
            return node
        target = _ref_target(node)
        if target is None or target in seen:
            return None
        seen.add(target)
        node = schemas.get(target)
    return None


def _is_date_time(schema: dict) -> bool:
    if schema.get("format") == "date-time":
        return True
    return any(
        isinstance(b, dict) and b.get("format") == "date-time"
        for key in ("anyOf", "oneOf")
        for b in schema.get(key) or []
    )


def clearable_date_fields(schemas: dict) -> dict[str, list[str]]:
    """Map schema name -> sorted wire names of its clearable date-time properties."""
    found: dict[str, list[str]] = {}
    for name, schema in schemas.items():
        if not isinstance(schema, dict) or schema.get("x-clearable") is True:
            continue
        properties = schema.get("properties") or {}
        bullets = {m.group("field") for m in _SCHEMA_BULLET.finditer(schema.get("description") or "")}
        fields = []
        for prop, node in properties.items():
            if not isinstance(node, dict):
                continue
            clearable = _clearable_schema(node, schemas)
            if clearable is not None:
                if not _is_date_time(clearable):
                    _fail(
                        f"{name}.{prop} is x-clearable but not a date-time; this hook only "
                        "knows how to clear date-times — extend it for the new kind"
                    )
                fields.append(prop)
            elif node.get("format") == "date-time" and (
                prop in bullets or _PROPERTY_NOTE.search(node.get("description") or "")
            ):
                fields.append(prop)
        if fields:
            found[name] = sorted(fields)
    return found


_ALIAS_KEYS = {"$ref", "allOf", "anyOf", "oneOf", "type", "format", "nullable", "maxLength"}


def _iter_refs(node, path: str = "#"):
    if isinstance(node, dict):
        for key, value in node.items():
            if key == "$ref" and isinstance(value, str):
                yield path, value
            else:
                yield from _iter_refs(value, f"{path}/{key}")
    elif isinstance(node, list):
        for i, value in enumerate(node):
            yield from _iter_refs(value, f"{path}/{i}")


def prepare_spec(spec: dict) -> tuple[dict, dict[str, list[str]]]:
    """Return a copy of ``spec`` openapi-generator can type, plus the clearable fields.

    Every clearable property becomes ``{type: string, format: date-time, nullable: true,
    x-clearable: true}`` (keeping its description and other annotations), so it generates
    as a chrono date in the ``double_option`` shape :func:`run` patches. The ``x-clearable``
    component schemas (and components aliasing them) are then dropped; one still referenced
    elsewhere fails the run, as it would generate as an empty struct.
    """
    out = copy.deepcopy(spec)
    schemas = out.get("components", {}).get("schemas", {})
    fields = clearable_date_fields(schemas)
    for name, props in fields.items():
        properties = schemas[name]["properties"]
        for prop in props:
            node = properties[prop]
            target = _clearable_schema(node, schemas) or {}
            rewritten = {k: v for k, v in node.items() if k not in _ALIAS_KEYS}
            rewritten.update(type="string", format="date-time", nullable=True)
            rewritten["x-clearable"] = True
            if not rewritten.get("description") and target.get("description"):
                rewritten["description"] = target["description"]
            properties[prop] = rewritten

    # The x-clearable schemas and any component aliasing one.
    clearable_names = {
        n for n, s in schemas.items()
        if isinstance(s, dict) and _clearable_schema(s, schemas) is not None
    }
    for name in sorted(clearable_names):
        del schemas[name]
    dangling = [
        f"{path} -> {ref}" for path, ref in _iter_refs(out)
        if ref.startswith("#/components/schemas/") and ref_name(ref) in clearable_names
    ]
    if dangling:
        _fail(
            "x-clearable schema still referenced outside a component-schema property; it "
            "would generate as an empty struct:\n  " + "\n  ".join(dangling)
        )
    return out, fields


MODULE = """\
/*
 * Orchestration Cluster API
 *
 * @generated clearable fields (scripts/postprocess.py hook 19) — DO NOT EDIT.
 */

//! Serde support for fields the API clears with an empty string.

/// Serde for a clearable date-time field typed
/// `Option<Option<chrono::DateTime<chrono::FixedOffset>>>`:
///
/// | Value | JSON | Meaning |
/// | --- | --- | --- |
/// | `None` | omitted | preserve the persisted value |
/// | `Some(Some(date))` | RFC 3339 string | set the value |
/// | `Some(None)` | `""` | clear the persisted value |
///
/// Deserializing maps `""` to `Some(None)` and `null` (which the API also treats as
/// "preserve") to `None`.
pub mod date_time {
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        value: &Option<Option<DateTime<FixedOffset>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(Some(date)) => date.serialize(serializer),
            Some(None) => serializer.serialize_str(""),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Option<DateTime<FixedOffset>>>, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(None),
            Some(s) if s.is_empty() => Ok(Some(None)),
            Some(s) => s
                .parse::<DateTime<FixedOffset>>()
                .map(|date| Some(Some(date)))
                .map_err(serde::de::Error::custom),
        }
    }
}
"""


def _field_re(wire: str, with_path: str) -> re.Pattern[str]:
    """The generator's serde attribute + field for ``wire``, tolerant of whitespace so it
    matches both the unformatted generator output and the `cargo fmt`-ed form."""
    return re.compile(
        rf"(?P<indent>[ \t]*)#\[serde\(\s*rename\s*=\s*\"{re.escape(wire)}\",\s*default,\s*"
        rf"with\s*=\s*\"(?P<with>{re.escape(with_path)})\",\s*"
        rf"skip_serializing_if\s*=\s*\"Option::is_none\",?\s*\)\]\s*"
        rf"pub (?P<field>\w+): (?P<ty>[^,\n]+),"
    )


def patch_model(text: str, model: str, wires: list[str], where: str) -> str:
    """Patch ``model``'s clearable ``wires`` in the model file ``text``; idempotent."""
    accessors = []
    for wire in wires:
        m = _field_re(wire, DOUBLE_OPTION).search(text)
        if m is None:
            done = _field_re(wire, CLEARABLE_WITH).search(text)
            if done is None:
                _fail(
                    f"{model}.{wire} in {where}: expected a `double_option` serde field "
                    f"typed {DATE_TIME_TYPE}; the generator output changed — update this hook"
                )
            m = done
        if re.sub(r"\s+", "", m.group("ty")) != re.sub(r"\s+", "", DATE_TIME_TYPE):
            _fail(f"{model}.{wire} in {where}: expected type {DATE_TIME_TYPE}, got {m.group('ty')}")
        field = m.group("field")
        if m.group("with") == DOUBLE_OPTION:
            indent = m.group("indent")
            doc = (
                f"{indent}///\n"
                f"{indent}/// `None` preserves the persisted value, `Some(Some(date))` sets it and "
                f"`Some(None)` clears it (sent as `\"\"`; see [`Self::clear_{field}`]).\n"
            )
            attr = m.group(0).replace(DOUBLE_OPTION, CLEARABLE_WITH, 1)
            text = text[: m.start()] + doc + attr + text[m.end():]
        accessors.append((wire, field))

    if ACCESSOR_MARKER in text:
        return text
    lines = ["", f"// @generated {ACCESSOR_MARKER}.", f"impl {model} {{"]
    for wire, field in accessors:
        lines += [
            f"    /// Clear `{wire}`: it is sent as `\"\"`, which removes the persisted value.",
            f"    /// Omitting it (`None`) or sending `null` preserves the value instead.",
            f"    pub fn clear_{field}(&mut self) {{",
            f"        self.{field} = Some(None);",
            "    }",
            "",
            f"    /// Whether `{wire}` is to be cleared (see [`Self::clear_{field}`]).",
            f"    pub fn is_{field}_cleared(&self) -> bool {{",
            f"        matches!(self.{field}, Some(None))",
            "    }",
            "",
        ]
    lines[-1] = "}"
    return text.rstrip("\n") + "\n" + "\n".join(lines) + "\n"


def _wire_module(lib_text: str) -> str:
    if re.search(r"^pub mod clearable;", lib_text, re.MULTILINE):
        return lib_text
    anchor = re.search(r"^pub mod models;\n", lib_text, re.MULTILINE)
    if anchor is None:
        _fail("`pub mod models;` not found in client/src/lib.rs")
    return lib_text[: anchor.end()] + "pub mod clearable;\n" + lib_text[anchor.end():]


def run(ctx: Context) -> None:
    for f in sorted(ctx.models_dir.glob("*.rs")):
        if re.search(r"pub struct ClearableDateTime\b", f.read_text(encoding="utf-8")):
            _fail(
                f"{f.name} defines ClearableDateTime: the generator was fed the raw spec. "
                "Run scripts/preprocess.py first (scripts/generate.sh does)."
            )
    fields = clearable_date_fields(ctx.schemas)
    if not fields:
        ctx.log(NAME, "no clearable date-time fields in the spec")
        return
    patched = 0
    for model, wires in sorted(fields.items()):
        f = ctx.models_dir / f"{snake_case(model)}.rs"
        if not f.exists():
            _fail(f"no generated model {f.name} for schema {model}")
        text = f.read_text(encoding="utf-8")
        f.write_text(patch_model(text, model, wires, f.name), encoding="utf-8")
        patched += len(wires)
    (ctx.client_dir / "src" / "clearable.rs").write_text(MODULE, encoding="utf-8")
    lib_rs = ctx.client_dir / "src" / "lib.rs"
    lib_rs.write_text(_wire_module(lib_rs.read_text(encoding="utf-8")), encoding="utf-8")
    ctx.log(NAME, f"made {patched} date-time field(s) clearable in {len(fields)} model(s)")
