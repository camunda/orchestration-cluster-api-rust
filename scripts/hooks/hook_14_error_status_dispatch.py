"""Hook 14 — decode typed error responses by HTTP status, not by payload shape.

openapi-generator emits one ``#[serde(untagged)]`` error enum per operation and decodes the
response body with ``serde_json::from_str(&content).ok()``, ignoring the status it already
holds. Untagged serde picks the first variant whose payload fits, and almost every variant
wraps ``models::ProblemDetail``, so a 404 or 409 came back as ``Status400`` (issue #59).

This hook adds a ``from_response(status, content)`` constructor to every error enum that
selects the variant declared for ``status`` (falling back to ``UnknownValue`` when the status
is undeclared or the body does not fit), and rewrites each decode site to call it.

An error enum with a variant shape this hook does not understand (e.g. a ``4XX`` range or a
``default`` response) is rejected rather than skipped, so a new shape cannot silently fall
back to shape-based decoding.

Idempotent: a rewritten decode site no longer matches ``DECODE_RE``, and the constructor is
not re-emitted once present.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 14
NAME = "error-status-dispatch"

ENUM_RE = re.compile(
    r"#\[serde\(untagged\)\]\npub enum (?P<name>\w+Error) \{\n(?P<body>(?:[^\n}]*\n)*?)\}\n"
)
STATUS_VARIANT_RE = re.compile(r"^\s*Status(?P<code>\d{3})\((?P<payload>[^()]*)\),$")
UNKNOWN_VARIANT = "UnknownValue(serde_json::Value),"
DECODE_RE = re.compile(
    r"let entity: Option<(?P<name>\w+Error)> =\s*serde_json::from_str\(&content\)\.ok\(\);"
)


def parse_variants(name: str, body: str) -> list[tuple[str, bool]]:
    """Return ``(code, has_payload)`` per status variant; raise on any other shape."""
    variants: list[tuple[str, bool]] = []
    has_unknown = False
    for line in body.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped == UNKNOWN_VARIANT:
            has_unknown = True
            continue
        m = STATUS_VARIANT_RE.match(line)
        if not m:
            raise SystemExit(f"[hook {NUMBER}] {name}: unsupported error variant `{stripped}`")
        variants.append((m.group("code"), bool(m.group("payload").strip())))
    if not has_unknown:
        raise SystemExit(f"[hook {NUMBER}] {name}: missing `{UNKNOWN_VARIANT}` fallback")
    return variants


def render_impl(name: str, variants: list[tuple[str, bool]]) -> str:
    arms = "".join(
        f"            {code} => serde_json::from_str(content).ok().map(Self::Status{code}),\n"
        if has_payload
        else f"            {code} => Some(Self::Status{code}()),\n"
        for code, has_payload in variants
    )
    return (
        f"\nimpl {name} {{\n"
        "    /// Decode an error response body into the variant declared for `status`.\n"
        "    ///\n"
        "    /// The variants share payload types, so deserializing this untagged enum directly\n"
        "    /// selects the first variant that fits, whatever the status.\n"
        "    pub fn from_response(status: u16, content: &str) -> Option<Self> {\n"
        "        let declared: Option<Self> = match status {\n"
        f"{arms}"
        "            _ => None,\n"
        "        };\n"
        "        declared.or_else(|| serde_json::from_str(content).ok().map(Self::UnknownValue))\n"
        "    }\n"
        "}\n"
    )


def transform(content: str) -> tuple[str, int, int]:
    """Return ``(new_content, impls_added, sites_rewritten)``."""
    enums: set[str] = set()
    impls_added = 0

    def add_impl(m: re.Match) -> str:
        nonlocal impls_added
        name = m.group("name")
        enums.add(name)
        variants = parse_variants(name, m.group("body"))
        if f"\nimpl {name} {{\n" in content:
            return m.group(0)
        impls_added += 1
        return m.group(0) + render_impl(name, variants)

    out = ENUM_RE.sub(add_impl, content)

    sites = 0

    def rewrite(m: re.Match) -> str:
        nonlocal sites
        name = m.group("name")
        if name not in enums:
            raise SystemExit(f"[hook {NUMBER}] decode site for unknown error enum `{name}`")
        sites += 1
        return f"let entity: Option<{name}> = {name}::from_response(status.as_u16(), &content);"

    out = DECODE_RE.sub(rewrite, out)
    return out, impls_added, sites


def run(ctx: Context) -> None:
    impls = 0
    sites = 0
    for rs in sorted((ctx.client_dir / "src" / "apis").glob("*.rs")):
        # LF-only generated tree: read and write untranslated.
        with rs.open("r", encoding="utf-8", newline="") as fh:
            content = fh.read()
        new_content, added, rewritten = transform(content)
        if new_content != content:
            with rs.open("w", encoding="utf-8", newline="") as fh:
                fh.write(new_content)
        impls += added
        sites += rewritten
    if impls or sites:
        ctx.log(NAME, f"added {impls} status-dispatch decoder(s), rewrote {sites} decode site(s)")
