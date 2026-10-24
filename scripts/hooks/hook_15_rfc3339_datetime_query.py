"""Hook 15 — serialize date-time query parameters as RFC 3339.

The spec declares a handful of query parameters with ``format: date-time`` — e.g.
``getGlobalJobStatistics`` (``from``/``to``) and ``getUsageMetrics``
(``startTime``/``endTime``). openapi-generator types these params as
``chrono::DateTime<chrono::FixedOffset>`` but serializes them into the query string with
``.to_string()``, which emits chrono's ``Display`` form::

    ?from=2025-01-15 12:00:00 +00:00

That is not RFC 3339: the server (which parses ``format: date-time`` strictly) rejects it.
Every sibling SDK (JS/Python/C#/Go/Java/PHP) sends the RFC 3339 form::

    ?from=2025-01-15T12:00:00+00:00

This hook finds every query call that serializes a ``chrono::DateTime`` params field and
rewrites its ``.to_string()`` to ``.to_rfc3339()``. It covers both the required shape
(``&params.<field>.to_string()``) and the optional shape openapi-generator emits inside an
``if let Some(ref param_value) = params.<field>`` block (``&param_value.to_string()``).

It is idempotent: once a call serializes with ``.to_rfc3339()`` the ``.to_string()`` pattern
this hook matches on no longer exists, so a second run finds nothing left to rewrite.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 15
NAME = "rfc3339-datetime-query"

# Params fields typed as a chrono date-time (optionally Option-wrapped).
DATETIME_FIELD_RE = re.compile(r"pub (\w+): (?:Option<)?chrono::DateTime")

# Required shape: query(&[("key", &params.<field>.to_string())]).
REQUIRED_RE = re.compile(
    r'(?P<head>req_builder = req_builder\.query\(&\[\("[^"]+", &params\.(?P<field>\w+))\.to_string\(\)'
)

# Optional shape: an `if let Some(ref param_value) = params.<field>` guard whose body
# serializes `param_value` into a query call.
OPTIONAL_RE = re.compile(
    r"(?P<head>if let Some\(ref param_value\) = params\.(?P<field>\w+) \{"
    r'(?P<between>.*?req_builder = req_builder\.query\(&\[\("[^"]+", &param_value))\.to_string\(\)',
    re.DOTALL,
)


def run(ctx: Context) -> None:
    fixed_files = 0
    fixed_calls = 0
    for rs in (ctx.client_dir / "src" / "apis").glob("*.rs"):
        # LF-only generated tree: read and write untranslated so this hook never
        # rewrites line endings. `Path.open` because `read_text(newline=…)` is 3.13+.
        with rs.open("r", encoding="utf-8", newline="") as fh:
            content = fh.read()
        datetime_fields = set(DATETIME_FIELD_RE.findall(content))
        if not datetime_fields:
            continue

        rewritten_here = 0

        def replace(m: re.Match) -> str:
            nonlocal rewritten_here
            if m.group("field") not in datetime_fields:
                return m.group(0)
            rewritten_here += 1
            return f"{m.group('head')}.to_rfc3339()"

        new_content = REQUIRED_RE.sub(replace, content)
        new_content = OPTIONAL_RE.sub(replace, new_content)

        if rewritten_here and new_content != content:
            with rs.open("w", encoding="utf-8", newline="") as fh:
                fh.write(new_content)
            fixed_files += 1
            fixed_calls += rewritten_here
    if fixed_calls:
        ctx.log(NAME, f"rewrote {fixed_calls} date-time query param(s) to RFC 3339 in {fixed_files} file(s)")
