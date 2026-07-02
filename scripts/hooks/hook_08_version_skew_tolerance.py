"""Hook 08 — tolerate server version skew on optional-by-deployment fields.

The upstream `main` spec (which this crate is generated from) runs ahead of released
Camunda servers. Some fields it marks *required* are not yet emitted by released or
alpha servers, so strict serde deserialization fails with `missing field <x>` and the
job-worker activate-jobs call errors out.

`ActivatedJobResult.priority` is the concrete case: added on `main` (the API returns 0
for jobs created before 8.10), but absent from 8.9.x and 8.10.0-alpha* responses. Make
it `#[serde(default)]` so the SDK deserializes activated jobs from any server version;
the zero default matches the documented pre-8.10 semantics.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 8
NAME = "version-skew-tolerance"

# (file under src/models, serde rename of the field, rust field declaration line)
_DEFAULTABLE = [
    (
        "activated_job_result.rs",
        '#[serde(rename = "priority")]',
        "pub priority: i32,",
    ),
]


def run(ctx: Context) -> None:
    for filename, serde_attr, field_decl in _DEFAULTABLE:
        path = ctx.models_dir / filename
        if not path.exists():
            continue
        text = path.read_text()
        if serde_attr not in text:
            continue
        # already patched?
        patched_attr = serde_attr.replace(
            '#[serde(rename = "', '#[serde(default, rename = "'
        )
        if patched_attr in text:
            continue
        new_text = text.replace(serde_attr, patched_attr, 1)
        if new_text != text:
            path.write_text(new_text)
            ctx.log(NAME, f"made {filename} {field_decl!r} serde(default)")
