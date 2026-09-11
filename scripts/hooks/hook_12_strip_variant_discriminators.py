"""Hook 12 — strip the re-declared discriminator field from tagged-union variant structs.

``openapi-generator`` renders each ``oneOf`` + ``discriminator`` as a serde
*internally-tagged* enum (``#[serde(tag = "...")] pub enum X``) whose variants wrap the
member schemas' structs. The generator bug it leaves behind: it **also** keeps the
discriminator property declared as an ordinary field on every member struct, e.g.

    #[serde(tag = "contentType")]
    pub enum AgentInstanceMessageContent { Object(Box<AgentInstanceObjectContent>), ... }

    pub struct AgentInstanceObjectContent {
        #[serde(rename = "contentType")] pub content_type: String,  // <- re-declares the tag
        pub object: Option<serde_json::Value>,
    }

With serde's internally-tagged representation the tag is consumed by the enum and is
**not** forwarded to the variant, so the variant's own required ``content_type`` field
can never be populated on the way in ("missing field ``contentType``"), and on the way
out it is written twice — once by the enum and once by the struct field. Every
polymorphic type in the client is therefore broken in both directions.

This hook removes the offending field from each variant struct: its doc comment and
``#[serde(rename = "<tag>")]`` attribute, the field declaration itself, and — where the
generator threaded it through the constructor — its ``new()`` parameter and struct-literal
initializer. The enum keeps sole ownership of the tag, which is what serde expects. It also
drops the now-removed field's row from the variant's checked-in markdown doc
(``client/docs/<Variant>.md``) so the published model documentation does not advertise a
field the struct no longer exposes.

It is discovery-driven (it reads the ``#[serde(tag = ...)]`` enums straight out of the
generated model files rather than an enumerated list), so a *new* ``oneOf`` added upstream
is covered automatically. It is idempotent: once a variant no longer declares the tag
there is nothing to match, so a second run is a no-op.
"""

from __future__ import annotations

import re

from .common import Context, snake_case

NUMBER = 12
NAME = "strip-variant-discriminators"

_TAG_RE = re.compile(r'#\[serde\(tag\s*=\s*"([^"]+)"\)\]')
_VARIANT_RE = re.compile(r"\bBox<\s*models::(\w+)\s*>")
_STRUCT_RE = re.compile(r"^pub struct (\w+)\b", re.MULTILINE)

# Rust keywords ``openapi-generator``'s rust generator escapes as raw identifiers
# (``r#<kw>``) when they appear as field names — which is exactly how they are then
# rendered in the generated markdown docs (e.g. a ``type`` discriminator becomes
# ``**r#type**``). A few keywords (``crate``, ``self``, ``Self``, ``super``) cannot be
# raw identifiers and the generator suffixes those instead, but none currently occur as
# discriminator field names.
_RUST_RAW_KEYWORDS = frozenset(
    {
        "abstract", "as", "async", "await", "become", "box", "break", "const",
        "continue", "do", "dyn", "else", "enum", "extern", "false", "final", "fn",
        "for", "if", "impl", "in", "let", "loop", "macro", "match", "mod", "move",
        "mut", "override", "priv", "pub", "ref", "return", "static", "struct",
        "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use",
        "virtual", "where", "while", "yield",
    }
)


def _rust_field_ident(name: str) -> str:
    """Map a snake_case field name to the identifier the rust generator emits for it.

    Rust keywords used as field names are escaped as raw identifiers (``r#type``), which
    is how they appear in both the generated struct and its markdown doc. Non-keyword
    names are returned unchanged.
    """
    return f"r#{name}" if name in _RUST_RAW_KEYWORDS else name


def _match_delim(s: str, open_idx: int, open_ch: str, close_ch: str) -> int:
    """Return the index of the delimiter matching the one at ``open_idx``, or -1."""
    depth = 0
    for i in range(open_idx, len(s)):
        if s[i] == open_ch:
            depth += 1
        elif s[i] == close_ch:
            depth -= 1
            if depth == 0:
                return i
    return -1


def _split_top_level_commas(s: str) -> list[str]:
    """Split ``s`` on commas that sit at bracket depth zero (``() [] <> {}``)."""
    parts: list[str] = []
    depth = 0
    cur: list[str] = []
    for ch in s:
        if ch in "([<{":
            depth += 1
        elif ch in ")]>}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    tail = "".join(cur)
    if tail.strip():
        parts.append(tail)
    return parts


def _strip_new_param(text: str, ident: str) -> str:
    """Drop the ``ident: Type`` parameter from the struct's ``new(...)`` signature."""
    key = "pub fn new("
    p = text.find(key)
    if p < 0:
        return text
    open_paren = p + len(key) - 1
    close = _match_delim(text, open_paren, "(", ")")
    if close < 0:
        return text
    inner = text[open_paren + 1 : close]
    parts = _split_top_level_commas(inner)
    pat = re.compile(r"\s*" + re.escape(ident) + r"\s*:")
    kept = [pt for pt in parts if not pat.match(pt)]
    if len(kept) == len(parts):
        return text  # no such parameter (e.g. an optional field left out of the ctor)
    new_inner = ", ".join(pt.strip() for pt in kept)
    return text[: open_paren + 1] + new_inner + text[close:]


def _strip_initializer(text: str, ident: str) -> str:
    """Drop the ``ident`` field from the struct literal returned by ``new(...)``."""
    m = re.search(r"\)\s*->\s*(\w+)\s*\{", text)
    if not m:
        return text
    name = m.group(1)
    p = text.find(name + " {", m.end())
    if p < 0:
        return text
    brace = text.index("{", p)
    close = _match_delim(text, brace, "{", "}")
    if close < 0:
        return text
    inner = text[brace + 1 : close]
    parts = _split_top_level_commas(inner)
    pat = re.compile(r"\s*" + re.escape(ident) + r"\s*:")
    kept = [pt for pt in parts if pt.strip() != ident and not pat.match(pt)]
    if len(kept) == len(parts):
        return text
    body = ", ".join(pt.strip() for pt in kept)
    new_inner = f" {body} " if body else ""
    return text[: brace + 1] + new_inner + text[close:]


def _strip_discriminator(text: str, tag: str) -> str | None:
    """Remove the field whose serde rename is ``tag`` from a variant struct.

    Returns the rewritten source, or ``None`` when the struct does not declare the tag
    (nothing to do — also the idempotency guard on a second pass).
    """
    lines = text.split("\n")
    attr_re = re.compile(r'#\[serde\(rename\s*=\s*"' + re.escape(tag) + r'"')
    attr_idx = next((i for i, ln in enumerate(lines) if attr_re.search(ln)), None)
    if attr_idx is None:
        return None
    # The field declaration is the next `pub <ident>:` line after the attribute.
    field_idx = attr_idx
    while field_idx < len(lines) and not lines[field_idx].lstrip().startswith("pub "):
        field_idx += 1
    if field_idx >= len(lines):
        return None
    m = re.match(r"\s*pub\s+(r#\w+|\w+)\s*:", lines[field_idx])
    if not m:
        return None
    ident = m.group(1)
    # Include the field's leading doc-comment line(s).
    start = attr_idx
    while start - 1 >= 0 and lines[start - 1].lstrip().startswith("///"):
        start -= 1
    del lines[start : field_idx + 1]
    out = "\n".join(lines)
    out = _strip_new_param(out, ident)
    out = _strip_initializer(out, ident)
    return out


def _strip_doc_field(docs_dir, variant: str, field: str) -> bool:
    """Drop the ``**<field>**`` property row from the variant's markdown doc.

    ``client/docs/<Variant>.md`` is generated alongside the struct and lists one row per
    field; when the struct loses its re-declared discriminator the doc must lose the same
    row. Keyed off the (stable) enum tag rather than the struct's current state, so it also
    repairs docs whose struct was already stripped, and it is idempotent once the row is
    gone. Returns ``True`` when a row was removed.
    """
    path = docs_dir / f"{variant}.md"
    if not path.exists():
        return False
    text = path.read_bytes().decode("utf-8")
    lines = text.split("\n")
    row_re = re.compile(r"^\s*\*\*" + re.escape(field) + r"\*\*\s*\|")
    kept = [ln for ln in lines if not row_re.match(ln)]
    if len(kept) == len(lines):
        return False
    path.write_bytes("\n".join(kept).encode("utf-8"))
    return True


def _struct_file_index(ctx: Context) -> dict[str, "object"]:
    """Map every ``pub struct Name`` in the models tree to the file that declares it."""
    index: dict[str, object] = {}
    for path in ctx.models_dir.glob("*.rs"):
        text = path.read_bytes().decode("utf-8")
        for m in _STRUCT_RE.finditer(text):
            index[m.group(1)] = path
    return index


def run(ctx: Context) -> None:
    models_dir = ctx.models_dir
    if not models_dir.exists():
        return
    struct_files = _struct_file_index(ctx)
    docs_dir = ctx.client_dir / "docs"
    for path in sorted(models_dir.glob("*.rs")):
        text = path.read_bytes().decode("utf-8")
        tag_match = _TAG_RE.search(text)
        if not tag_match:
            continue
        tag = tag_match.group(1)
        doc_field = _rust_field_ident(snake_case(tag))
        for variant in dict.fromkeys(_VARIANT_RE.findall(text)):
            vpath = struct_files.get(variant)
            if vpath is not None:
                vtext = vpath.read_bytes().decode("utf-8")
                new_text = _strip_discriminator(vtext, tag)
                if new_text is not None and new_text != vtext:
                    vpath.write_bytes(new_text.encode("utf-8"))
                    ctx.log(
                        NAME,
                        f"stripped re-declared discriminator `{tag}` from {vpath.name}",
                    )
            if _strip_doc_field(docs_dir, variant, doc_field):
                ctx.log(
                    NAME,
                    f"stripped stale discriminator row `{doc_field}` from {variant}.md",
                )
