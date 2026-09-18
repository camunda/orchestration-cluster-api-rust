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

from scripts.hooks import hook_08_version_skew_tolerance
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


# An enum tagged on a Rust keyword (`type`), whose variant struct was *already*
# stripped of the discriminator in a prior generation — only the markdown doc remains
# stale. Exercises the doc-cleanup branch independently of struct stripping (so a
# regression that skips it when the struct is already clean is caught), and the
# raw-identifier (`r#type`) mapping the generator applies to keyword field names.
_KEYWORD_ENUM = (
    "use crate::models;\n"
    "use serde::{Deserialize, Serialize};\n\n"
    "#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]\n"
    '#[serde(tag = "type")]\n'
    "pub enum JobResult {\n"
    '    #[serde(rename = "userTask")]\n'
    "    UserTask(Box<models::JobResultUserTask>),\n"
    "}\n"
)

# The variant struct as it stands *after* a prior generation stripped the tag: it no
# longer declares the discriminator field, so `_strip_discriminator` is a no-op on it.
_STRIPPED_VARIANT = (
    "use crate::models;\n"
    "use serde::{Deserialize, Serialize};\n\n"
    "#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]\n"
    "pub struct JobResultUserTask {\n"
    "    /// Whether the task was denied.\n"
    '    #[serde(rename = "denied")]\n'
    "    pub denied: bool,\n"
    "}\n\n"
    "impl JobResultUserTask {\n"
    "    pub fn new(denied: bool) -> JobResultUserTask {\n"
    "        JobResultUserTask { denied }\n"
    "    }\n"
    "}\n"
)


# A discriminator the generator rendered as a *multiline* `#[serde(...)]` attribute —
# exactly how it formats any field carrying more than one directive (here an optional
# discriminator with `skip_serializing_if`). A line-local `#[serde(rename = ...)]` match
# misses this shape entirely, leaving the tag re-declared and the variant un-decodable.
_MULTILINE_VARIANT = (
    "use crate::models;\n"
    "use serde::{Deserialize, Serialize};\n\n"
    "#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]\n"
    "pub struct ObjectContent {\n"
    "    /// The content type discriminator.\n"
    "    #[serde(\n"
    '        rename = "contentType",\n'
    '        skip_serializing_if = "Option::is_none"\n'
    "    )]\n"
    "    pub content_type: Option<String>,\n"
    "    /// The object payload.\n"
    '    #[serde(rename = "object")]\n'
    "    pub object: Option<serde_json::Value>,\n"
    "}\n\n"
    "impl ObjectContent {\n"
    "    pub fn new(object: Option<serde_json::Value>) -> ObjectContent {\n"
    "        ObjectContent {\n"
    "            content_type: None,\n"
    "            object,\n"
    "        }\n"
    "    }\n"
    "}\n"
)


class StripVariantDiscriminatorsTest(unittest.TestCase):
    """`hook_12` removes the re-declared discriminator field from every variant struct
    of a `#[serde(tag = ...)]` enum, along with its `new()` param and initializer."""

    def test_strips_multiline_serde_discriminator(self):
        models, ctx = _make_models(
            {
                "content.rs": _ENUM_FIXTURE,
                "text_content.rs": _TEXT_VARIANT,
                "object_content.rs": _MULTILINE_VARIANT,
            }
        )
        hook_12_strip_variant_discriminators.run(ctx)

        obj = (models / "object_content.rs").read_text(encoding="utf-8")
        # The whole multiline attribute + field + its doc line are gone...
        self.assertNotIn('rename = "contentType"', obj)
        self.assertNotIn("content_type", obj)
        self.assertNotIn("skip_serializing_if", obj)
        self.assertNotIn("The content type discriminator.", obj)
        # ...while the sibling non-discriminator field is untouched.
        self.assertIn("pub object: Option<serde_json::Value>", obj)
        self.assertIn(
            "pub fn new(object: Option<serde_json::Value>) -> ObjectContent", obj
        )
        self.assertNotIn("content_type: None", obj)

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
        """The variant's checked-in markdown doc must lose the discriminator row even
        when the struct was already stripped in a prior generation, and even when the
        discriminator is a Rust keyword the generator escapes as a raw identifier
        (`r#type`) in Markdown."""
        models, ctx = _make_models(
            {
                "job_result.rs": _KEYWORD_ENUM,
                "job_result_user_task.rs": _STRIPPED_VARIANT,
            }
        )
        docs = ctx.client_dir / "docs"
        docs.mkdir(parents=True)
        doc = (
            "# JobResultUserTask\n\n## Properties\n\n"
            "Name | Type | Description | Notes\n"
            "------------ | ------------- | ------------- | -------------\n"
            "**r#type** | Option<**String**> | The result type discriminator. | [optional]\n"
            "**denied** | Option<**bool**> | Whether the task was denied. | [optional]\n"
        )
        (docs / "JobResultUserTask.md").write_bytes(doc.encode("utf-8"))

        # The struct is already stripped, so its source must be left untouched...
        before = (models / "job_result_user_task.rs").read_bytes()
        hook_12_strip_variant_discriminators.run(ctx)
        self.assertEqual((models / "job_result_user_task.rs").read_bytes(), before)

        # ...but the stale raw-identifier `r#type` row must still be removed from the doc.
        out = (docs / "JobResultUserTask.md").read_text(encoding="utf-8")
        self.assertNotIn("**r#type**", out)
        self.assertIn("**denied**", out)
        # Idempotent: a second run leaves the already-repaired doc untouched.
        once = (docs / "JobResultUserTask.md").read_bytes()
        hook_12_strip_variant_discriminators.run(ctx)
        self.assertEqual((docs / "JobResultUserTask.md").read_bytes(), once)

    def test_maps_rust_keyword_discriminators_to_raw_identifiers(self):
        """The generator escapes a keyword discriminator field as a raw identifier
        (`type` -> `r#type`, `try` -> `r#try`), so the doc-field name the hook strips
        must be raw-mapped too. Non-keyword names pass through unchanged. Guards the
        `try` gap (a reserved keyword the table previously omitted)."""
        rust_ident = hook_12_strip_variant_discriminators._rust_field_ident
        self.assertEqual(rust_ident("type"), "r#type")
        self.assertEqual(rust_ident("try"), "r#try")
        self.assertEqual(rust_ident("match"), "r#match")
        # A non-keyword field name is returned verbatim.
        self.assertEqual(rust_ident("content_type"), "content_type")

    def test_strips_stale_try_keyword_discriminator_row_from_doc(self):
        """A tagged union whose discriminator is the reserved keyword `try` is spelled
        `r#try` by the generator; the stale doc row must still be stripped (regression
        for the keyword-table `try` omission)."""
        enum = (
            "use crate::models;\n"
            "use serde::{Deserialize, Serialize};\n\n"
            "#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]\n"
            '#[serde(tag = "try")]\n'
            "pub enum Attempt {\n"
            '    #[serde(rename = "first")]\n'
            "    First(Box<models::AttemptFirst>),\n"
            "}\n"
        )
        variant = (
            "use serde::{Deserialize, Serialize};\n\n"
            "#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]\n"
            "pub struct AttemptFirst {\n"
            '    #[serde(rename = "label")]\n'
            "    pub label: String,\n"
            "}\n"
        )
        models, ctx = _make_models(
            {"attempt.rs": enum, "attempt_first.rs": variant}
        )
        docs = ctx.client_dir / "docs"
        docs.mkdir(parents=True)
        doc = (
            "# AttemptFirst\n\n## Properties\n\n"
            "Name | Type | Description | Notes\n"
            "------------ | ------------- | ------------- | -------------\n"
            "**r#try** | Option<**String**> | The attempt discriminator. | [optional]\n"
            "**label** | Option<**String**> | A label. | [optional]\n"
        )
        (docs / "AttemptFirst.md").write_bytes(doc.encode("utf-8"))

        hook_12_strip_variant_discriminators.run(ctx)

        out = (docs / "AttemptFirst.md").read_text(encoding="utf-8")
        self.assertNotIn("**r#try**", out)
        self.assertIn("**label**", out)

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
                if hook_12_strip_variant_discriminators._rename_attr_re(tag).search(
                    vpath.read_text(encoding="utf-8")
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


class VersionSkewToleranceTest(unittest.TestCase):
    """`hook_08` adds `#[serde(default)]` to fields older servers may omit.

    A miss here is invisible at generation time and only shows up in
    production, as an activate-jobs response from an older server failing to
    deserialize and stalling the worker. So the hook must refuse to finish
    quietly when an entry no longer matches.
    """

    def _run_hook(self, model_src: str) -> tuple[Path, Context]:
        tmp = Path(tempfile.mkdtemp())
        models = tmp / "src" / "models"
        models.mkdir(parents=True)
        (models / "activated_job_result.rs").write_bytes(model_src.encode("utf-8"))
        ctx = Context(client_dir=tmp, spec={}, schemas={})
        hook_08_version_skew_tolerance.run(ctx)
        return models, ctx

    @staticmethod
    def _model(*fields: str) -> str:
        return "pub struct ActivatedJobResult {\n" + "\n".join(fields) + "\n}\n"

    # Every wire name the hook claims to defaultable, rendered the way the
    # generator emits it, so the fixture moves whenever _DEFAULTABLE does.
    @classmethod
    def _all_fields(cls) -> list[str]:
        return [
            '    #[serde(rename = "%s", deserialize_with = "Option::deserialize")]\n'
            "    pub f_%d: Option<String>," % (wire, i)
            for i, (_, wire) in enumerate(hook_08_version_skew_tolerance._DEFAULTABLE)
        ]

    def test_adds_default_to_every_declared_field(self):
        models, _ = self._run_hook(self._model(*self._all_fields()))
        out = (models / "activated_job_result.rs").read_text()
        for _, wire in hook_08_version_skew_tolerance._DEFAULTABLE:
            self.assertIn(f'#[serde(default, rename = "{wire}"', out)

    def test_is_idempotent(self):
        models, ctx = self._run_hook(self._model(*self._all_fields()))
        once = (models / "activated_job_result.rs").read_text()
        hook_08_version_skew_tolerance.run(ctx)
        self.assertEqual(once, (models / "activated_job_result.rs").read_text())

    def test_raises_when_a_declared_field_is_missing(self):
        """The defect class: an entry upstream renamed must not skip silently."""
        for _, wire in hook_08_version_skew_tolerance._DEFAULTABLE:
            surviving = [f for f in self._all_fields() if f'rename = "{wire}"' not in f]
            with self.subTest(renamed_away=wire):
                with self.assertRaises(RuntimeError) as caught:
                    self._run_hook(self._model(*surviving))
                self.assertIn(wire, str(caught.exception))

    def test_raises_when_the_model_file_is_missing(self):
        tmp = Path(tempfile.mkdtemp())
        (tmp / "src" / "models").mkdir(parents=True)
        ctx = Context(client_dir=tmp, spec={}, schemas={})
        with self.assertRaises(RuntimeError):
            hook_08_version_skew_tolerance.run(ctx)

    def test_tolerates_reordered_and_retyped_serde_arguments(self):
        """The domain-type hooks retype these fields; that must not break the match."""
        src = self._model(
            '    #[serde(deserialize_with = "Option::deserialize", rename = "jobLeaseToken")]\n'
            "    pub job_lease_token: Option<models::JobLeaseToken>,",
            *[f for f in self._all_fields() if 'rename = "jobLeaseToken"' not in f],
        )
        models, _ = self._run_hook(src)
        out = (models / "activated_job_result.rs").read_text()
        self.assertIn("#[serde(default, deserialize_with", out)

    # The hook runs on unformatted generator output, where a serde attribute is
    # one line. Anything re-run over committed code sees the cargo-fmt'd form,
    # which wraps once there are three arguments.
    _WRAPPED = (
        "    #[serde(\n"
        "        {default}rename = \"jobLeaseToken\",\n"
        '        deserialize_with = "Option::deserialize"\n'
        "    )]\n"
        "    pub job_lease_token: Option<models::JobLeaseToken>,"
    )

    def _wrapped_model(self, *, patched: bool) -> str:
        field = self._WRAPPED.format(default="default,\n        " if patched else "")
        return self._model(
            field, *[f for f in self._all_fields() if 'rename = "jobLeaseToken"' not in f]
        )

    def _patched_args(self, out: str, wire_name: str) -> str:
        """The serde arguments of `wire_name` alone, so a sibling field's rewrite
        cannot stand in for the one under test."""
        attr = hook_08_version_skew_tolerance._serde_attr(wire_name).search(out)
        self.assertIsNotNone(attr, f"no serde attribute for {wire_name} in output")
        return attr.group("args")

    def test_adds_default_to_a_wrapped_serde_attribute(self):
        models, _ = self._run_hook(self._wrapped_model(patched=False))
        out = (models / "activated_job_result.rs").read_text()
        self.assertIn("default", self._patched_args(out, "jobLeaseToken"))
        self.assertEqual(out.count('rename = "jobLeaseToken"'), 1)

    def test_leaves_an_already_patched_wrapped_attribute_alone(self):
        patched_attr = self._WRAPPED.format(default="default,\n        ")
        models, _ = self._run_hook(self._wrapped_model(patched=True))
        out = (models / "activated_job_result.rs").read_text()
        # The other fields are unpatched and legitimately change; this one must not.
        self.assertIn(patched_attr, out)
        self.assertEqual(out.count("default"), len(self._all_fields()))


if __name__ == "__main__":
    unittest.main()
