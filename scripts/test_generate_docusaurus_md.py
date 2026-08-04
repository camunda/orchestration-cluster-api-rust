#!/usr/bin/env python3
"""Unit tests for `scripts/generate-docusaurus-md.py`.

Run with:

    make test-docs
    python3 -m unittest discover -s scripts -t . -p 'test_*.py'

Scope: the pure text transforms (doc normalisation, link rewriting, README
splitting), the rustdoc-JSON collectors, and the example/operation-map wiring.
Everything here runs against hand-built fixtures or files already in the repo,
so no nightly toolchain and no generated rustdoc JSON are required and the
whole suite finishes in milliseconds.

Why this exists: the `docs` CI job proves the generator *runs* and that every
link resolves. It cannot see output that is still valid, well-linked Markdown
but silently wrong -- a dropped heading, a swallowed code fence, a private item
leaking into the published reference. Those are the regressions these tests
guard, and each guard is written against the *class* of defect rather than the
single instance that prompted it.
"""

from __future__ import annotations

import importlib.util
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

# The module under test has hyphens in its filename, so it cannot be imported
# by name. It must be registered in `sys.modules` before it is executed --
# `@dataclass` resolves annotations through `sys.modules[cls.__module__]`.
_SCRIPT_PATH = Path(__file__).resolve().parent / "generate-docusaurus-md.py"
_spec = importlib.util.spec_from_file_location("generate_docusaurus_md", _SCRIPT_PATH)
assert _spec is not None and _spec.loader is not None
gen = importlib.util.module_from_spec(_spec)
sys.modules[_spec.name] = gen
_spec.loader.exec_module(gen)


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------


def _crate(index: dict, paths: dict | None = None, root: str = "0") -> "gen.Crate":
    return gen.Crate(
        name="testcrate",
        version="0.0.0",
        index=index,
        paths=paths or {},
        root=root,
    )


def _struct_item(name: str, visibility: str, fields: list | None = None) -> dict:
    return {
        "name": name,
        "visibility": visibility,
        "docs": f"Docs for {name}.",
        "inner": {"struct": {"kind": {"plain": {"fields": fields or []}}, "impls": []}},
    }


def _fn_item(name: str, visibility: str) -> dict:
    return {
        "name": name,
        "visibility": visibility,
        "docs": f"Docs for {name}.",
        "inner": {"function": {"header": {}, "sig": {"inputs": [], "output": None}}},
    }


def _field_item(name: str, visibility: str) -> dict:
    return {
        "name": name,
        "visibility": visibility,
        "docs": "",
        "inner": {"struct_field": {"primitive": "u8"}},
    }


def _fence_lines(text: str) -> list[str]:
    return [line for line in text.split("\n") if line.startswith("```")]


# Visibility values rustdoc emits that must never reach the published docs.
NON_PUBLIC_VISIBILITIES = ["default", {"restricted": {"parent": "1", "path": "::foo"}}]


# ---------------------------------------------------------------------------
# _normalize_docs: headings and code fences
# ---------------------------------------------------------------------------


class NormalizeDocsTest(unittest.TestCase):
    """`_normalize_docs` has to tell two meanings of `#` apart.

    Inside a code fence a leading `#` marks a line rustdoc hides from the
    rendered doctest. Outside one it is an ordinary Markdown heading. Treating
    every `^# ` as the former deletes real headings; treating every one as the
    latter dumps doctest scaffolding into the docs.
    """

    def test_keeps_headings_outside_code_fences(self):
        docs = "# Quick start\n\nSome prose."
        self.assertIn("# Quick start", gen._normalize_docs(docs, 3))

    def test_shifts_headings_by_heading_base(self):
        out = gen._normalize_docs("# Quick start", 3, heading_base=2)
        self.assertEqual(out, "### Quick start")

    def test_shifts_every_heading_level(self):
        docs = "# One\n\n## Two\n\n### Three"
        out = gen._normalize_docs(docs, 3, heading_base=1)
        self.assertIn("## One", out)
        self.assertIn("### Two", out)
        self.assertIn("#### Three", out)

    def test_caps_shifted_heading_at_h6(self):
        # Markdown has no h7; over-shifting must clamp rather than emit `#######`.
        out = gen._normalize_docs("##### Deep", 3, heading_base=4)
        self.assertEqual(out, "###### Deep")
        self.assertNotIn("#######", out)

    def test_drops_hidden_doctest_lines_inside_fences(self):
        docs = "```rust\n# use foo::Bar;\nlet x = 1;\n#\n```"
        out = gen._normalize_docs(docs, 3)
        self.assertNotIn("use foo::Bar", out)
        self.assertIn("let x = 1;", out)

    def test_keeps_headings_that_follow_a_closed_fence(self):
        # Regression: fence state must be tracked, not assumed.
        docs = "```rust\n# hidden\nlet x = 1;\n```\n\n# Real heading"
        out = gen._normalize_docs(docs, 3)
        self.assertNotIn("hidden", out)
        self.assertIn("# Real heading", out)

    def test_normalises_opening_fence_info_strings(self):
        for opener in ["```", "```rust", "```no_run", "```rust,ignore", "```should_panic"]:
            with self.subTest(opener=opener):
                out = gen._normalize_docs(f"{opener}\nlet x = 1;\n```", 3)
                self.assertEqual(_fence_lines(out)[0], "```rust")

    def test_never_writes_an_info_string_onto_a_closing_fence(self):
        """A closing fence carrying an info string is not a valid closer.

        Markdown then treats it as a *second opener* and the rest of the page is
        swallowed into a code block. Guard the whole class, not one instance.
        """
        cases = [
            "```no_run\nlet x = 1;\n```",
            "```\nlet x = 1;\n```\n\nprose\n\n```rust\nlet y = 2;\n```",
            "```rust,ignore\nlet x = 1;\n```\n\n# Heading\n\n```\nlet y = 2;\n```",
        ]
        for docs in cases:
            with self.subTest(docs=docs):
                fences = _fence_lines(gen._normalize_docs(docs, 3))
                self.assertEqual(len(fences) % 2, 0, "unbalanced fences")
                for closer in fences[1::2]:
                    self.assertEqual(closer, "```")

    def test_preserves_fence_count(self):
        docs = "```no_run\n# hidden\nlet x = 1;\n```\n\ntext\n\n```rust\nlet y = 2;\n```"
        self.assertEqual(len(_fence_lines(gen._normalize_docs(docs, 3))), 4)

    def test_does_not_treat_fenced_hash_as_a_heading_to_shift(self):
        docs = "```rust\n# use foo::Bar;\n```"
        out = gen._normalize_docs(docs, 3, heading_base=2)
        self.assertNotIn("###", out)

    def test_empty_docs_round_trip(self):
        self.assertEqual(gen._normalize_docs("", 3), "")


# ---------------------------------------------------------------------------
# Visibility guards
# ---------------------------------------------------------------------------


class VisibilityGuardTest(unittest.TestCase):
    """Every collector must agree on what "public" means.

    rustdoc records `"default"` for items written without a visibility keyword,
    which for structs, enums, fields and inherent-impl methods means *private*.
    Those only reach `crate.index` under `--document-private-items`, so this is
    a latent guard -- but a collector that disagrees with its siblings is how it
    stops being latent.
    """

    def test_is_public_only_accepts_explicit_pub(self):
        self.assertTrue(gen._is_public({"visibility": "public"}))
        self.assertFalse(gen._is_public({}))
        for visibility in NON_PUBLIC_VISIBILITIES:
            with self.subTest(visibility=visibility):
                self.assertFalse(gen._is_public({"visibility": visibility}))

    def test_collect_types_skips_non_public_types(self):
        for visibility in NON_PUBLIC_VISIBILITIES:
            with self.subTest(visibility=visibility):
                crate = _crate(
                    {
                        "1": _struct_item("Public", "public"),
                        "2": _struct_item("Hidden", visibility),
                    }
                )
                self.assertEqual(sorted(gen.collect_types(crate)), ["Public"])

    def test_collect_fields_skips_non_public_fields(self):
        for visibility in NON_PUBLIC_VISIBILITIES:
            with self.subTest(visibility=visibility):
                crate = _crate(
                    {
                        "10": _field_item("shown", "public"),
                        "11": _field_item("hidden", visibility),
                    }
                )
                body = {"kind": {"plain": {"fields": ["10", "11"]}}}
                names = [f.name for f in gen._collect_fields(crate, body)]
                self.assertEqual(names, ["shown"])

    def test_collect_methods_skips_non_public_methods(self):
        for visibility in NON_PUBLIC_VISIBILITIES:
            with self.subTest(visibility=visibility):
                crate = _crate(
                    {
                        "5": {"inner": {"impl": {"trait": None, "items": ["20", "21"]}}},
                        "20": _fn_item("shown", "public"),
                        "21": _fn_item("hidden", visibility),
                    }
                )
                names = [m.name for m in gen._collect_methods(crate, ["5"])]
                self.assertEqual(names, ["shown"])

    def test_collect_methods_skips_trait_impls(self):
        crate = _crate(
            {
                "5": {"inner": {"impl": {"trait": {"path": "Debug"}, "items": ["20"]}}},
                "20": _fn_item("fmt", "public"),
            }
        )
        self.assertEqual(gen._collect_methods(crate, ["5"]), [])


# ---------------------------------------------------------------------------
# Link rewriting
# ---------------------------------------------------------------------------


class LinkRewritingTest(unittest.TestCase):
    def test_absolute_docs_links_become_site_relative(self):
        out = gen._rewrite_docs_links("[x](https://docs.camunda.io/docs/components/concepts/a/)", 3)
        self.assertEqual(out, "[x](../../../components/concepts/a.md)")

    def test_next_prefix_is_stripped(self):
        out = gen._rewrite_docs_links("[x](https://docs.camunda.io/docs/next/reference/b)", 1)
        self.assertEqual(out, "[x](../reference/b.md)")

    def test_url_path_overrides_are_applied(self):
        out = gen._rewrite_docs_links("[x](https://docs.camunda.io/docs/apis-tools/camunda-api-rest/c)", 1)
        self.assertIn("orchestration-cluster-api-rest", out)
        self.assertNotIn("camunda-api-rest", out)

    def test_depth_controls_the_relative_prefix(self):
        for depth in (1, 2, 3):
            with self.subTest(depth=depth):
                out = gen._rewrite_docs_links("[x](https://docs.camunda.io/docs/a/b)", depth)
                self.assertEqual(out, f"[x]({'../' * depth}a/b.md)")

    def test_repo_links_point_at_github(self):
        out = gen.rewrite_repo_links("[LICENSE](LICENSE)")
        self.assertEqual(out, f"[LICENSE]({gen.GITHUB_BLOB}/LICENSE)")

    def test_sibling_generated_pages_are_left_alone(self):
        self.assertEqual(gen.rewrite_repo_links("[Other](other.md)"), "[Other](other.md)")

    def test_external_and_anchor_links_are_left_alone(self):
        for link in ["[x](https://example.com)", "[x](#anchor)", "[x](../up.md)"]:
            with self.subTest(link=link):
                self.assertEqual(gen.rewrite_repo_links(link), link)


# ---------------------------------------------------------------------------
# README pipeline
# ---------------------------------------------------------------------------


class ReadmePipelineTest(unittest.TestCase):
    def test_strip_cut_sections(self):
        content = "keep\n<!-- docs:cut:start -->\ndrop\n<!-- docs:cut:end -->\nkeep2\n"
        out = gen._strip_cut_sections(content)
        self.assertNotIn("drop", out)
        self.assertIn("keep", out)
        self.assertIn("keep2", out)

    def test_strip_cut_sections_handles_multiple_pairs(self):
        content = (
            "a\n<!-- docs:cut:start -->\nx\n<!-- docs:cut:end -->\n"
            "b\n<!-- docs:cut:start -->\ny\n<!-- docs:cut:end -->\nc\n"
        )
        out = gen._strip_cut_sections(content)
        for dropped in ("x", "y"):
            self.assertNotIn(dropped, out)
        for kept in ("a", "b", "c"):
            self.assertIn(kept, out)

    def test_strip_snippet_markers(self):
        content = "<!-- snippet-source: examples/readme.rs | regions: A -->\n```rust\n```\n"
        self.assertNotIn("snippet-source", gen._strip_snippet_markers(content))

    def test_promote_headings_lifts_everything_below_h1(self):
        out = gen._promote_headings("## Two\n### Three\n")
        self.assertIn("# Two", out)
        self.assertIn("## Three", out)

    def test_promote_headings_leaves_h1_alone(self):
        self.assertEqual(gen._promote_headings("# One\n"), "# One\n")

    def test_split_by_h2(self):
        preamble, sections = gen._split_by_h2("intro\n\n## One\nbody\n\n## Two\nbody2\n")
        self.assertIn("intro", preamble)
        self.assertEqual([title for title, _ in sections], ["One", "Two"])
        for title, body in sections:
            self.assertTrue(body.startswith(f"## {title}"))

    def test_slugify(self):
        self.assertEqual(gen._slugify("Job Workers"), "job-workers")
        self.assertEqual(gen._slugify("The Camunda Domain Type System"), "the-camunda-domain-type-system")
        self.assertEqual(gen._slugify("Reliability & Convenience"), "reliability-convenience")

    def test_tech_preview_banner_lands_after_the_first_h1(self):
        out = gen.inject_tech_preview_banner("# Title\n\nprose\n")
        self.assertLess(out.index("# Title"), out.index("Technical Preview"))
        self.assertLess(out.index("Technical Preview"), out.index("prose"))


# ---------------------------------------------------------------------------
# rustdoc JSON format contract
# ---------------------------------------------------------------------------


class RustdocFormatGuardTest(unittest.TestCase):
    """rustdoc JSON is unstable; the version pin must fail loudly, not silently."""

    def _write(self, payload: dict) -> Path:
        tmp = Path(tempfile.mkdtemp()) / "crate.json"
        tmp.write_text(json.dumps(payload), encoding="utf-8")
        return tmp

    def test_supported_format_version_loads(self):
        version = sorted(gen.SUPPORTED_FORMAT_VERSIONS)[0]
        path = self._write(
            {
                "format_version": version,
                "root": "0",
                "index": {"0": {"name": "mycrate"}},
                "paths": {},
                "crate_version": "1.2.3",
            }
        )
        crate = gen.load_rustdoc_json(path)
        self.assertEqual(crate.name, "mycrate")
        self.assertEqual(crate.version, "1.2.3")

    def test_unsupported_format_version_raises(self):
        version = max(gen.SUPPORTED_FORMAT_VERSIONS) + 1
        path = self._write({"format_version": version, "root": "0", "index": {"0": {}}})
        with self.assertRaises(gen.RustdocFormatError) as ctx:
            gen.load_rustdoc_json(path)
        self.assertIn(str(version), str(ctx.exception))

    def test_missing_file_raises_with_an_actionable_hint(self):
        with self.assertRaises(FileNotFoundError) as ctx:
            gen.load_rustdoc_json(Path(tempfile.mkdtemp()) / "absent.json")
        self.assertIn("make docs-json", str(ctx.exception))


# ---------------------------------------------------------------------------
# Examples and the operation map
# ---------------------------------------------------------------------------


class OperationMapTest(unittest.TestCase):
    """The operation map drives the Rust code tabs on the REST API reference.

    `load_examples()` only *warns* on a broken entry, so a typo'd region name
    silently drops a tab from the published docs. These assertions turn that
    into a build failure.
    """

    @classmethod
    def setUpClass(cls):
        cls.op_map = json.loads(gen.OPERATION_MAP_PATH.read_text(encoding="utf-8"))

    def test_entries_are_well_formed(self):
        for operation_id, entries in self.op_map.items():
            with self.subTest(operation_id=operation_id):
                self.assertIsInstance(entries, list)
                self.assertGreaterEqual(len(entries), 1)
                for entry in entries:
                    self.assertEqual(set(entry), {"file", "region", "label"})
                    self.assertTrue(entry["file"].endswith(".rs"))
                    self.assertNotIn("/", entry["file"], "paths are relative to examples/")

    def test_every_referenced_file_exists(self):
        missing = sorted(
            {
                entry["file"]
                for entries in self.op_map.values()
                for entry in entries
                if not (gen.EXAMPLES_DIR / entry["file"]).is_file()
            }
        )
        self.assertEqual(missing, [], f"operation-map references missing files: {missing}")

    def test_every_mapped_region_resolves(self):
        cache: dict[str, str] = {}
        unresolved: list[str] = []
        for operation_id, entries in self.op_map.items():
            for entry in entries:
                src = gen.EXAMPLES_DIR / entry["file"]
                if not src.is_file():
                    continue
                if entry["file"] not in cache:
                    cache[entry["file"]] = src.read_text(encoding="utf-8")
                pattern = gen._REGION_RE_TEMPLATE.format(name=re.escape(entry["region"]))
                if not re.search(pattern, cache[entry["file"]], re.MULTILINE | re.DOTALL):
                    unresolved.append(f"{operation_id} -> {entry['file']}#{entry['region']}")
        self.assertEqual(unresolved, [], f"unresolved example regions: {unresolved}")

    def test_load_examples_returns_a_block_for_every_operation(self):
        self.assertEqual(len(gen.load_examples()), len(self.op_map))

    def test_operation_ids_are_camel_case(self):
        for operation_id in self.op_map:
            with self.subTest(operation_id=operation_id):
                self.assertRegex(operation_id, r"^[a-z][A-Za-z0-9]*$")


class SnakeToCamelTest(unittest.TestCase):
    def test_round_trips_plain_snake_case(self):
        self.assertEqual(gen._snake_to_camel("create_process_instance"), "createProcessInstance")
        self.assertEqual(gen._snake_to_camel("search"), "search")

    def test_applies_initialism_overrides(self):
        for method, operation_id in gen._METHOD_OPERATION_ID.items():
            with self.subTest(method=method):
                self.assertEqual(gen._snake_to_camel(method), operation_id)

    def test_overrides_are_needed(self):
        """Each override must actually differ from the naive conversion.

        A stale override silently masks a real casing mismatch, so drop any
        entry that the plain rule already handles.
        """
        for method, operation_id in gen._METHOD_OPERATION_ID.items():
            head, *tail = method.split("_")
            naive = head + "".join(p.title() for p in tail)
            with self.subTest(method=method):
                self.assertNotEqual(naive, operation_id, f"override for {method} is redundant")

    def test_overrides_point_at_real_operations(self):
        op_map = json.loads(gen.OPERATION_MAP_PATH.read_text(encoding="utf-8"))
        for method, operation_id in gen._METHOD_OPERATION_ID.items():
            with self.subTest(method=method):
                self.assertIn(operation_id, op_map)


if __name__ == "__main__":
    unittest.main()
