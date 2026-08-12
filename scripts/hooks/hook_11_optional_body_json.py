"""Hook 11 — guard optional JSON request bodies.

For an operation whose OpenAPI ``requestBody`` is ``required: false``, openapi-generator
still emits an unconditional ``req_builder = req_builder.json(&params.<field>);`` call
even when the params field is ``Option<T>``. When the caller passes ``None`` this sends
the literal JSON body `null` with a `Content-Type: application/json` header, which is a
different wire request than omitting the body entirely and can cause strict servers to
reject it (e.g. `POST /secrets/list` with no filters).

This hook finds every such unconditional ``req_builder.json(&params.<field>)`` call whose
params struct declares ``pub <field>: Option<...>`` and rewrites it to only attach the
body when the option is `Some`, leaving the request bodyless otherwise:

    if let Some(ref body) = params.<field> {
        req_builder = req_builder.json(body);
    }

It is idempotent: once a call is rewritten, the unconditional
``req_builder.json(&params.<field>)`` pattern this hook matches on no longer exists in the
file, so a second run finds nothing left to guard.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 11
NAME = "guard-optional-json-body"

CALL_RE = re.compile(
    r"(?P<indent>[ \t]*)req_builder = req_builder\.json\(&params\.(?P<field>\w+)\);\n"
)


def run(ctx: Context) -> None:
    fixed_files = 0
    fixed_calls = 0
    for rs in (ctx.client_dir / "src" / "apis").glob("*.rs"):
        # Read/write with an explicit `\n` newline so this hook is a no-op on line
        # endings — the generated tree is LF-only; letting Python translate `\n` to
        # the platform newline (CRLF on Windows) would otherwise rewrite every line.
        content = rs.read_text(newline="\n")
        struct_optional_fields = set(
            m.group(1) for m in re.finditer(r"pub (\w+): Option<", content)
        )
        if not struct_optional_fields:
            continue

        guarded_here = 0

        def replace(m: re.Match) -> str:
            nonlocal guarded_here
            field = m.group("field")
            if field not in struct_optional_fields:
                return m.group(0)
            guarded_here += 1
            indent = m.group("indent")
            return (
                f"{indent}if let Some(ref body) = params.{field} {{\n"
                f"{indent}    req_builder = req_builder.json(body);\n"
                f"{indent}}}\n"
            )

        new_content = CALL_RE.sub(replace, content)
        if guarded_here and new_content != content:
            rs.write_text(new_content, newline="\n")
            fixed_files += 1
            fixed_calls += guarded_here
    if fixed_calls:
        ctx.log(NAME, f"guarded {fixed_calls} optional JSON body call(s) in {fixed_files} file(s)")
