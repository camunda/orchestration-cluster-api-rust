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

# (file under src/models, serde `rename` wire name of the field)
#
# All entries below sit on the job-activation hot path (`ActivatedJobResult`).
# Upstream `main` marks them required, but released and alpha servers do not all
# emit them yet, so without `#[serde(default)]` a single missing field makes the
# whole activate-jobs response fail to deserialize and silently stalls workers.
# Adding `default` only affects the absent-key case (absent -> None/zero); fields
# carrying `deserialize_with = "Option::deserialize"` keep that attribute.
#
# Entries key on the wire name alone. Matching the whole attribute line, or the
# Rust field declaration, couples this hook to the field's Rust type — which the
# domain-type hooks rewrite — and to the exact serde argument order.
_DEFAULTABLE = [
    ("activated_job_result.rs", "priority"),
    ("activated_job_result.rs", "physicalTenantId"),
    ("activated_job_result.rs", "businessId"),
    ("activated_job_result.rs", "jobLeaseToken"),
]


def _serde_attr(wire_name: str) -> re.Pattern:
    return re.compile(
        r"#\[serde\((?P<args>[^)\]]*\brename = \"%s\"[^)\]]*)\)\]" % re.escape(wire_name)
    )


def run(ctx: Context) -> None:
    for filename, wire_name in _DEFAULTABLE:
        path = ctx.models_dir / filename
        if not path.exists():
            raise RuntimeError(
                f"{NAME}: {filename} does not exist, so the version-skew default for "
                f"{wire_name!r} was not applied. The model was renamed or removed "
                f"upstream — update _DEFAULTABLE in {__name__}."
            )
        text = path.read_text()
        match = _serde_attr(wire_name).search(text)
        if match is None:
            raise RuntimeError(
                f"{NAME}: no serde field renamed {wire_name!r} in {filename}, so it "
                f"would deserialize without #[serde(default)] and a server omitting "
                f"the field would fail the whole response. Either the bundled spec "
                f"predates the field (run `make bundle`), or upstream renamed it and "
                f"_DEFAULTABLE in {__name__} needs updating."
            )
        args = match.group("args")
        if re.search(r"\bdefault\b", args):
            continue
        text = text[: match.start()] + f"#[serde(default, {args})]" + text[match.end() :]
        path.write_text(text)
        ctx.log(NAME, f"made {filename} {wire_name!r} serde(default)")
