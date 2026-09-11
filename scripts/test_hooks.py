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
import re
import tempfile
import unittest
from pathlib import Path

from scripts.hooks import hook_11_optional_body_json
from scripts.hooks import hook_12_strip_variant_discriminators
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


_MODELS_DIR = _REPO_ROOT / "client" / "src" / "models"

_TAG_RE = hook_12_strip_variant_discriminators._TAG_RE
_VARIANT_RE = hook_12_strip_variant_discriminators._VARIANT_RE


def _make_models(files: dict[str, str]) -> tuple[Path, Context]:
    tmp = Path(tempfile.mkdtemp())
    models = tmp / "src" / "models"
    models.mkdir(parents=True)
    for name, body in files.items():
        (models / name).write_bytes(body.encode("utf-8"))
    ctx = Context(client_dir=tmp, spec={}, schemas={})
    return models, ctx


_ENUM_FIXTURE = (
    "use crate::models;\n"
    "use serde::{Deserialize, Serialize};\n\n"
    "#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]\n"
    '#[serde(tag = "contentType")]\n'
    "pub enum Content {\n"
    '    #[serde(rename = "TEXT")]\n'
    "    Text(Box<models::TextContent>),\n"
    '    #[serde(rename = "OBJECT")]\n'
    "    Object(Box<models::ObjectContent>),\n"
    "}\n"
)

# A required discriminator threaded through `new()` (both a param and an initializer).
_TEXT_VARIANT = (
    "use crate::models;\n"
    "use serde::{Deserialize, Serialize};\n\n"
    "#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]\n"
    "pub struct TextContent {\n"
    "    /// The content type discriminator.\n"
    '    #[serde(rename = "contentType")]\n'
    "    pub content_type: String,\n"
    "    /// The text content.\n"
    '    #[serde(rename = "text")]\n'
    "    pub text: String,\n"
    "}\n\n"
    "impl TextContent {\n"
    "    pub fn new(content_type: String, text: String) -> TextContent {\n"
    "        TextContent { content_type, text }\n"
    "    }\n"
    "}\n"
)

# A discriminator that is the struct's only field — stripping empties the struct.
_OBJECT_VARIANT = (
    "use crate::models;\n"
    "use serde::{Deserialize, Serialize};\n\n"
    "#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]\n"
    "pub struct ObjectContent {\n"
    "    /// The content type discriminator.\n"
    '    #[serde(rename = "contentType")]\n'
    "    pub content_type: String,\n"
    "}\n\n"
    "impl ObjectContent {\n"
    "    pub fn new(content_type: String) -> ObjectContent {\n"
    "        ObjectContent {\n"
    "            content_type,\n"
    "        }\n"
    "    }\n"
    "}\n"
)


class StripVariantDiscriminatorsTest(unittest.TestCase):
    """`hook_12` removes the re-declared discriminator field from every variant struct
    of a `#[serde(tag = ...)]` enum, along with its `new()` param and initializer."""

    def test_strips_field_param_and_initializer(self):
        models, ctx = _make_models(
            {
                "content.rs": _ENUM_FIXTURE,
                "text_content.rs": _TEXT_VARIANT,
                "object_content.rs": _OBJECT_VARIANT,
            }
        )
        hook_12_strip_variant_discriminators.run(ctx)

        text = (models / "text_content.rs").read_text(encoding="utf-8")
        self.assertNotIn('rename = "contentType"', text)
        self.assertNotIn("content_type", text)
        self.assertIn("pub text: String", text)
        # `new` keeps the non-discriminator param.
        self.assertIn("pub fn new(text: String)", text)
        self.assertIn("TextContent { text }", text)

        obj = (models / "object_content.rs").read_text(encoding="utf-8")
        self.assertNotIn("content_type", obj)
        self.assertIn("pub fn new() -> ObjectContent", obj)

    def test_is_idempotent(self):
        models, ctx = _make_models(
            {
                "content.rs": _ENUM_FIXTURE,
                "text_content.rs": _TEXT_VARIANT,
                "object_content.rs": _OBJECT_VARIANT,
            }
        )
        hook_12_strip_variant_discriminators.run(ctx)
        once = (models / "text_content.rs").read_bytes()
        hook_12_strip_variant_discriminators.run(ctx)
        self.assertEqual((models / "text_content.rs").read_bytes(), once)

    def test_preserves_lf_line_endings(self):
        models, ctx = _make_models(
            {
                "content.rs": _ENUM_FIXTURE,
                "text_content.rs": _TEXT_VARIANT,
                "object_content.rs": _OBJECT_VARIANT,
            }
        )
        hook_12_strip_variant_discriminators.run(ctx)
        self.assertNotIn(b"\r\n", (models / "text_content.rs").read_bytes())

    def test_strips_stale_discriminator_row_from_markdown_doc(self):
        """The variant's checked-in markdown doc must lose the discriminator row too,
        even when the struct field was already stripped in a prior generation."""
        models, ctx = _make_models(
            {
                "content.rs": _ENUM_FIXTURE,
                "text_content.rs": _TEXT_VARIANT,
                "object_content.rs": _OBJECT_VARIANT,
            }
        )
        docs = ctx.client_dir / "docs"
        docs.mkdir(parents=True)
        doc = (
            "# TextContent\n\n## Properties\n\n"
            "Name | Type | Description | Notes\n"
            "------------ | ------------- | ------------- | -------------\n"
            "**content_type** | **String** | The content type discriminator. | \n"
            "**text** | **String** | The text content. | \n"
        )
        (docs / "TextContent.md").write_bytes(doc.encode("utf-8"))
        hook_12_strip_variant_discriminators.run(ctx)

        out = (docs / "TextContent.md").read_text(encoding="utf-8")
        self.assertNotIn("**content_type**", out)
        self.assertIn("**text**", out)
        # Idempotent: a second run leaves the already-repaired doc untouched.
        once = (docs / "TextContent.md").read_bytes()
        hook_12_strip_variant_discriminators.run(ctx)
        self.assertEqual((docs / "TextContent.md").read_bytes(), once)

    def test_leaves_a_non_discriminator_field_alone(self):
        """A struct that is not a tagged-union variant must be untouched."""
        plain = (
            "pub struct Plain {\n"
            '    #[serde(rename = "contentType")]\n'
            "    pub content_type: String,\n"
            "}\n"
        )
        models, ctx = _make_models({"plain.rs": plain})
        hook_12_strip_variant_discriminators.run(ctx)
        self.assertEqual(
            (models / "plain.rs").read_text(encoding="utf-8"), plain
        )


class NoVariantRedeclaresItsTagTest(unittest.TestCase):
    """Class-scoped regression guard over the *real* generated client: no variant
    struct of any `#[serde(tag = ...)]` enum may re-declare its enum's tag.

    This is the invariant issue #41 restores. It is deliberately not pinned to the
    six known enums — a new `oneOf` added upstream, regenerated with the same
    generator bug, would re-introduce exactly this defect and must be caught here.
    """

    def _struct_files(self) -> dict[str, Path]:
        index: dict[str, Path] = {}
        for path in _MODELS_DIR.glob("*.rs"):
            for m in hook_12_strip_variant_discriminators._STRUCT_RE.finditer(
                path.read_text(encoding="utf-8")
            ):
                index[m.group(1)] = path
        return index

    def test_no_tagged_union_variant_redeclares_the_tag(self):
        if not _MODELS_DIR.exists():
            self.skipTest("generated client models are not present")
        struct_files = self._struct_files()
        offenders = []
        unresolved = []
        tagged_seen = False
        for path in _MODELS_DIR.glob("*.rs"):
            text = path.read_text(encoding="utf-8")
            tag_match = _TAG_RE.search(text)
            if not tag_match:
                continue
            tagged_seen = True
            tag = tag_match.group(1)
            for variant in dict.fromkeys(_VARIANT_RE.findall(text)):
                vpath = struct_files.get(variant)
                if vpath is None:
                    # A variant that resolves to no struct file is a coverage hole: the
                    # guard cannot prove that variant does not re-declare its tag, so the
                    # defect could slip back in unseen. Fail instead of skipping silently.
                    unresolved.append(f"{path.name}: variant `{variant}` has no struct file")
                    continue
                if re.search(
                    r'#\[serde\(rename\s*=\s*"' + re.escape(tag) + r'"',
                    vpath.read_text(encoding="utf-8"),
                ):
                    offenders.append(f"{vpath.name} re-declares tag `{tag}`")
        self.assertTrue(tagged_seen, "expected at least one #[serde(tag = ...)] enum")
        self.assertEqual(unresolved, [], "tagged-union variant did not resolve to a struct")
        self.assertEqual(offenders, [])


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
