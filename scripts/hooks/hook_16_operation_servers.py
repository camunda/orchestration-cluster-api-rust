"""Hook 16 — honor path-item ``servers`` overrides (cluster-admin base path).

The spec's top-level ``servers`` entry is ``{schema}://{host}:{port}/v2``, so the
generated ``Configuration::base_path`` ends in ``/v2`` and every operation path is
appended to it. The cluster-admin path items under ``/cluster/v2/*`` override that with
their own ``servers`` entry *without* the ``/v2`` suffix — the correct request for
``GET /cluster/v2/topology`` is ``{root}/cluster/v2/topology``, not
``{root}/v2/cluster/v2/topology``.

openapi-generator ignores path-item/operation ``servers`` for the Rust generator, so
the generated call sites prefix ``/v2`` twice. This hook:

1. Derives the override set from the bundled spec: every path item that declares its
   own ``servers`` entry whose URL does **not** end in the top-level ``/v2`` suffix.
   Deriving (rather than hard-coding) keeps the hook correct when upstream adds more
   cluster-admin paths; the build fails loudly if the spec drops the override while the
   generated code still references it.
2. Adds ``Configuration::operation_base_path`` to ``client/src/apis/configuration.rs``,
   which strips the ``/v2`` suffix for exactly those paths.
3. Rewrites the ``uri_str`` construction of every generated operation on an overridden
   path to go through that helper.

The runtime facade is unaffected: ``CamundaClient::configuration()`` keeps handing the
generated client the ``/v2``-suffixed base path, and the helper strips it only for the
cluster-admin operations — including under ``CAMUNDA_REST_ADDRESS_EXACT`` gateway
setups, where ``/v2`` is part of the *operation* path, not the base.
"""

from __future__ import annotations

import re

from .common import Context

NUMBER = 16
NAME = "operation-servers"

_HELPER = '''
impl Configuration {
    /// Base path for one operation, honoring the spec's path-item `servers` overrides.
    ///
    /// The top-level server is `{schema}://{host}:{port}/v2`, but the cluster-admin
    /// path items under `/cluster/v2/*` declare their own `servers` entry without the
    /// `/v2` suffix — their paths are served from the server root, so the `/v2` base
    /// suffix must not be prefixed to them (that would produce `/v2/cluster/v2/...`).
    pub fn operation_base_path(&self, operation_path: &str) -> &str {
        const ROOT_SERVER_PATHS: &[&str] = &[
__ROOT_PATHS__
        ];
        if ROOT_SERVER_PATHS.contains(&operation_path) {
            self.base_path.strip_suffix("/v2").unwrap_or(&self.base_path)
        } else {
            &self.base_path
        }
    }
}
'''


def _top_level_suffix(spec: dict) -> str:
    """The path suffix of the top-level server URL (e.g. `/v2`), or "" when absent."""
    servers = spec.get("servers") or []
    if not servers:
        return ""
    url = servers[0].get("url", "")
    # Strip the `{schema}://{host}:{port}` variable prefix; what remains is the suffix.
    m = re.search(r"\{port\}(/[^\"]*)?$", url)
    if not m:
        return ""
    return (m.group(1) or "").rstrip("/")


def _root_server_paths(ctx: Context) -> list[str]:
    """Spec paths whose path-item `servers` override drops the top-level `/v2` suffix."""
    suffix = _top_level_suffix(ctx.spec)
    out: set[str] = set()
    for path, item in (ctx.spec.get("paths") or {}).items():
        if not isinstance(item, dict):
            continue
        servers = item.get("servers")
        if not servers:
            continue
        url = servers[0].get("url", "")
        if suffix and not url.rstrip("/").endswith(suffix):
            out.add(path)
    return sorted(out)


def _rewrite_api_file(content: str, paths: list[str]) -> tuple[str, int]:
    """Rewrite `format!("{}<path>", configuration.base_path ...)` call sites.

    The generator emits one of two shapes (the second for paths with parameters):

        format!("{}/cluster/v2/topology", configuration.base_path)
        format!(
            "{}/cluster/v2/backups/runtime/{backupId}",
            configuration.base_path,
            backupId = ...
        )

    Both become `configuration.operation_base_path("<path>")` as the base argument.
    Returns the new content and the number of rewritten call sites.
    """
    rewritten = 0
    for path in paths:
        # The format string starts with "{}" followed by the literal spec path,
        # e.g. the literal: "{}/cluster/v2/topology"
        pattern = re.compile(
            r"format!\(\s*"
            + re.escape('"{}' + path + '"')
            + r"\s*,\s*configuration\.base_path\s*,?"
        )
        replacement = (
            f'format!("{{}}{path}", configuration.operation_base_path("{path}"), '
        )
        content, n = pattern.subn(replacement, content)
        rewritten += n
        # Tidy up: when base_path was the only argument, the replacement leaves
        # a trailing ", )" — collapse it.
        content = content.replace(
            f'configuration.operation_base_path("{path}"), )',
            f'configuration.operation_base_path("{path}"))',
        )
    return content, rewritten


def run(ctx: Context) -> None:
    paths = _root_server_paths(ctx)
    if not paths:
        raise RuntimeError(
            f"{NAME}: no path-item `servers` overrides found in the bundled spec. "
            "The cluster-admin paths (/cluster/v2/*) are expected to declare a "
            "root-level server; if upstream removed the override, remove this hook "
            "and the `operation_base_path` helper instead of shipping a no-op."
        )

    # 1. Add the helper to configuration.rs (idempotent).
    config_rs = ctx.client_dir / "src" / "apis" / "configuration.rs"
    text = config_rs.read_text()
    if "operation_base_path" not in text:
        rendered_paths = "\n".join(f'            "{p}",' for p in paths)
        helper = _HELPER.replace("__ROOT_PATHS__", rendered_paths)
        text = text.rstrip("\n") + "\n" + helper
        config_rs.write_text(text)
        ctx.log(
            NAME,
            f"added Configuration::operation_base_path covering {len(paths)} path(s)",
        )

    # 2. Rewrite the uri_str construction of every operation on an overridden path.
    apis_dir = ctx.client_dir / "src" / "apis"
    total = 0
    files = 0
    for api_rs in sorted(apis_dir.glob("*_api.rs")):
        content, n = _rewrite_api_file(api_rs.read_text(), paths)
        if n:
            api_rs.write_text(content)
            total += n
            files += 1
    if total:
        ctx.log(NAME, f"rewrote {total} uri_str call site(s) in {files} api file(s)")
