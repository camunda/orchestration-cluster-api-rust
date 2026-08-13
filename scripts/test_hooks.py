#!/usr/bin/env python3
"""Unit tests for the post-processing hooks under `scripts/hooks/`.

Run with:

    make test-docs
    python3 -m unittest discover -s scripts -t . -p 'test_*.py'

Scope: the text transforms the hooks apply, plus a guard on the stdlib surface
the hooks are allowed to use. The `bundle` CI job runs the hooks for real
against freshly generated output, but that only proves they do not crash on
whatever interpreter the runner happens to ship -- it cannot see a transform
that has silently stopped firing, and it cannot see a too-new stdlib call
sitting on a code path that run did not take. Those are the regressions
guarded here, and each guard targets the *class* of defect rather than the
single instance that prompted it.
"""

from __future__ import annotations

import ast
import tempfile
import unittest
from pathlib import Path

from scripts.hooks import hook_11_optional_body_json
from scripts.hooks.common import Context

_REPO_ROOT = Path(__file__).resolve().parents[1]
_SCRIPTS_DIR = _REPO_ROOT / "scripts"


class GuardOptionalJsonBodyTest(unittest.TestCase):
    """`hook_11` wraps `req_builder.json(&params.x)` when `x` is an `Option`."""

    def _run_hook(self, files: dict[str, str]) -> tuple[Path, Context]:
        tmp = Path(tempfile.mkdtemp())
        apis = tmp / "src" / "apis"
        apis.mkdir(parents=True)
        for name, body in files.items():
            # Bytes, so the fixture's line endings survive regardless of platform.
            (apis / name).write_bytes(body.encode("utf-8"))
        ctx = Context(client_dir=tmp, spec={}, schemas={})
        hook_11_optional_body_json.run(ctx)
        return apis, ctx

    def test_guards_an_optional_body(self):
        apis, _ = self._run_hook(
            {
                "secret_api.rs": (
                    "pub struct ListSecretsParams {\n"
                    "    pub filter: Option<SecretFilter>,\n"
                    "}\n"
                    "    req_builder = req_builder.json(&params.filter);\n"
                )
            }
        )
        out = (apis / "secret_api.rs").read_text(encoding="utf-8")
        self.assertIn("if let Some(ref body) = params.filter {", out)
        self.assertIn("req_builder = req_builder.json(body);", out)
        self.assertNotIn("req_builder.json(&params.filter)", out)

    def test_leaves_a_required_body_unconditional(self):
        apis, _ = self._run_hook(
            {
                "deployment_api.rs": (
                    "pub struct CreateDeploymentParams {\n"
                    "    pub body: DeploymentRequest,\n"
                    "}\n"
                    "    req_builder = req_builder.json(&params.body);\n"
                )
            }
        )
        out = (apis / "deployment_api.rs").read_text(encoding="utf-8")
        self.assertIn("req_builder = req_builder.json(&params.body);", out)
        self.assertNotIn("if let Some(", out)

    def test_is_idempotent(self):
        source = (
            "pub struct ListSecretsParams {\n"
            "    pub filter: Option<SecretFilter>,\n"
            "}\n"
            "    req_builder = req_builder.json(&params.filter);\n"
        )
        apis, ctx = self._run_hook({"secret_api.rs": source})
        once = (apis / "secret_api.rs").read_bytes()
        hook_11_optional_body_json.run(ctx)
        self.assertEqual((apis / "secret_api.rs").read_bytes(), once)

    def test_preserves_lf_line_endings(self):
        """The generated tree is LF-only; rewriting must not translate newlines.

        Without explicit newline handling this passes on Linux and silently
        rewrites every line to CRLF on Windows.
        """
        apis, _ = self._run_hook(
            {
                "secret_api.rs": (
                    "pub struct ListSecretsParams {\n"
                    "    pub filter: Option<SecretFilter>,\n"
                    "}\n"
                    "    req_builder = req_builder.json(&params.filter);\n"
                )
            }
        )
        self.assertNotIn(b"\r\n", (apis / "secret_api.rs").read_bytes())


class StdlibVersionFloorTest(unittest.TestCase):
    """The hooks run on whatever interpreter the runner ships, so they may not
    use stdlib signatures newer than that.

    This is a spot guard, not a general one -- it knows about the specific
    too-new signatures that have bitten us, not every 3.13+ API. A general
    check would need a tool like `vermin` wired into CI.
    """

    # `Path.read_text(newline=...)` is 3.13+; ubuntu-latest currently ships 3.12.
    # `Path.open(newline=...)` is the portable spelling.
    TOO_NEW = {("read_text", "newline")}

    def test_no_script_uses_a_too_new_stdlib_signature(self):
        offenders = []
        for py in sorted(_SCRIPTS_DIR.rglob("*.py")):
            tree = ast.parse(py.read_text(encoding="utf-8"), filename=str(py))
            for node in ast.walk(tree):
                if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)):
                    continue
                for kw in node.keywords:
                    if (node.func.attr, kw.arg) in self.TOO_NEW:
                        rel = py.relative_to(_REPO_ROOT).as_posix()
                        offenders.append(f"{rel}:{node.lineno} {node.func.attr}({kw.arg}=...)")
        self.assertEqual(offenders, [])


if __name__ == "__main__":
    unittest.main()
