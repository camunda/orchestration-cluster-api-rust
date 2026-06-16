"""Hook 06 — remove generator scaffolding noise.

openapi-generator emits repo-management scaffolding (a ``git_push.sh`` and an
``.openapi-generator-ignore``) that is irrelevant to this workspace. Remove it.
"""

from __future__ import annotations

from .common import Context

NUMBER = 6
NAME = "cleanup-scaffolding"


def run(ctx: Context) -> None:
    removed = 0
    for noise in ("git_push.sh", ".openapi-generator-ignore"):
        f = ctx.client_dir / noise
        if f.exists():
            f.unlink()
            removed += 1
    if removed:
        ctx.log(NAME, f"removed {removed} generator scaffolding file(s)")
