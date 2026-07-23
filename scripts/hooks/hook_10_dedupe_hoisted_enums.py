"""Hook 10 — drop hoisted, unreferenced top-level items from tagged-union models.

openapi-generator renders a ``oneOf`` discriminated union as a single tagged enum
(``#[serde(tag = "...")] pub enum X``) in its own model file. When one of the union's
member schemas carries an *inline* enum property, the generator sometimes hoists that
inline enum into the union's module as an extra top-level ``pub enum`` — even though the
canonical copy already lives in the member's own model file. The hoisted copy is dead:
it is never referenced in-file and ``models/mod.rs`` re-exports each module by name
(``pub use self::wait_state_details::WaitStateDetails``), not with a glob, so it is not
reachable as ``models::<Name>`` either. It survives only because generated code is
compiled with ``#![allow(warnings)]``.

Concrete case (upstream ``main``): ``wait_state_details.rs`` (the ``WaitStateDetails``
union) gains a stray ``pub enum Events`` that duplicates the real one in
``condition_wait_state_details.rs``.

This hook removes such artifacts so they do not reappear on regen. It is deliberately
conservative: it only touches files whose primary type is a ``#[serde(tag = ...)]``
union, and within those it removes a top-level ``pub enum``/``pub struct`` (plus its
``impl`` blocks and leading doc/attribute lines) *only* when its name is referenced
nowhere else in the file. A co-located type that the union actually uses is referenced,
so it is kept.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 10
NAME = "dedupe-hoisted-enums"

_ITEM_RE = re.compile(r"^(?:pub\s+)?(enum|struct|impl)\b")
_ENUM_STRUCT_NAME_RE = re.compile(r"^(?:pub\s+)?(?:enum|struct)\s+(\w+)")
# impl [<generics>] [Trait [for]] Type ...  — capture the target type (last word before `{`/`for`).
_IMPL_TARGET_RE = re.compile(
    r"^impl(?:<[^>]*>)?\s+(?:[\w:]+(?:<[^>]*>)?\s+for\s+)?(\w+)"
)
_ATTR_OR_DOC_RE = re.compile(r"^\s*(///|//!|#\[|#!\[)")


class _Block:
    __slots__ = ("kind", "name", "start", "end")

    def __init__(self, kind: str, name: str, start: int, end: int) -> None:
        self.kind = kind  # "enum" | "struct" | "impl"
        self.name = name  # type name (for impl: the target type)
        self.start = start  # first line index (incl. leading doc/attr lines)
        self.end = end  # last line index (inclusive)


def _find_block_end(lines: list[str], header: int) -> int:
    """Return the last line index of the item starting at ``header``.

    Handles brace-delimited items (enum/struct/impl with a body) and the unit
    ``pub struct X;`` / one-line forms.
    """
    # Unit struct or one-liner terminated by `;` before any `{`.
    if "{" not in lines[header]:
        i = header
        while i < len(lines) and "{" not in lines[i]:
            if lines[i].rstrip().endswith(";"):
                return i
            i += 1
        if i >= len(lines):
            return len(lines) - 1
        header = i  # brace opens on a later line
    depth = 0
    for i in range(header, len(lines)):
        depth += lines[i].count("{") - lines[i].count("}")
        if depth <= 0 and "{" in "".join(lines[header : i + 1]):
            return i
    return len(lines) - 1


def _leading_start(lines: list[str], header: int) -> int:
    """Walk back over contiguous doc/attribute lines above ``header``."""
    start = header
    j = header - 1
    while j >= 0 and _ATTR_OR_DOC_RE.match(lines[j]):
        start = j
        j -= 1
    return start


def _parse_top_level(lines: list[str]) -> tuple[list[_Block], str | None]:
    """Parse top-level enum/struct/impl blocks; return (blocks, union_enum_name)."""
    blocks: list[_Block] = []
    union_name: str | None = None
    i = 0
    while i < len(lines):
        line = lines[i]
        if not _ITEM_RE.match(line):
            i += 1
            continue
        header = i
        end = _find_block_end(lines, header)
        start = _leading_start(lines, header)
        m_es = _ENUM_STRUCT_NAME_RE.match(line)
        if m_es:
            kind = "enum" if line.lstrip().startswith(("pub enum", "enum")) else "struct"
            name = m_es.group(1)
            blocks.append(_Block(kind, name, start, end))
            # Is this the discriminated-union enum? (a `#[serde(tag` sits in its attrs)
            if kind == "enum" and any(
                "#[serde(tag" in lines[k] for k in range(start, header)
            ):
                union_name = name
        else:
            m_impl = _IMPL_TARGET_RE.match(line)
            if m_impl:
                blocks.append(_Block("impl", m_impl.group(1), start, end))
        i = end + 1
    return blocks, union_name


def _normalize_blank_runs(text: str) -> str:
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.rstrip("\n") + "\n"


def run(ctx: Context) -> None:
    for path in sorted(ctx.models_dir.glob("*.rs")):
        text = path.read_text()
        if "#[serde(tag" not in text:
            continue
        lines = text.splitlines()
        blocks, union_name = _parse_top_level(lines)
        if union_name is None:
            continue

        # Candidate types: every top-level enum/struct that is not the union itself.
        candidate_names = {
            b.name for b in blocks if b.kind in ("enum", "struct") and b.name != union_name
        }
        if not candidate_names:
            continue

        removed: list[str] = []
        drop: set[int] = set()
        for name in sorted(candidate_names):
            owned = [b for b in blocks if b.name == name]
            owned_lines = {i for b in owned for i in range(b.start, b.end + 1)}
            remaining = "\n".join(
                lines[i] for i in range(len(lines)) if i not in owned_lines
            )
            if re.search(r"\b" + re.escape(name) + r"\b", remaining):
                continue  # referenced elsewhere — keep it
            drop |= owned_lines
            removed.append(name)

        if not drop:
            continue

        kept = [lines[i] for i in range(len(lines)) if i not in drop]
        new_text = _normalize_blank_runs("\n".join(kept))
        if new_text != text:
            path.write_text(new_text)
            ctx.log(
                NAME,
                f"removed hoisted dead type(s) {', '.join(removed)} from {path.name}",
            )
