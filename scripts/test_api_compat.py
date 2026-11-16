"""Tests for scripts/api_compat.py.

The baseline-resolution and parsing tests are pure and always run. The synthetic-crate
tests build tiny baseline/current crate pairs, one per break class, and run the full
check (cargo-semver-checks + rustdoc JSON diff) on them. They need the pinned
cargo-semver-checks and the DOCS_TOOLCHAIN nightly, so they only run with
API_COMPAT_E2E=1 (`make test-api-compat`, and the CI job of the same name), where a
missing tool fails instead of skipping.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("api_compat", HERE / "api_compat.py")
api_compat = importlib.util.module_from_spec(_spec)
sys.modules["api_compat"] = api_compat
_spec.loader.exec_module(api_compat)


def _versions(*entries: tuple[str, bool]) -> list[dict]:
    return [{"num": num, "yanked": yanked} for num, yanked in entries]


class ResolveBaselineTest(unittest.TestCase):
    def test_orders_numerically_not_lexically(self):
        vs = _versions(("10.0.9", False), ("10.0.10", False), ("10.0.2", False))
        self.assertEqual(api_compat.resolve_baseline(vs, 10), "10.0.10")

    def test_minor_outranks_patch(self):
        vs = _versions(("10.0.99", False), ("10.1.0", False))
        self.assertEqual(api_compat.resolve_baseline(vs, 10), "10.1.0")

    def test_skips_prereleases(self):
        vs = _versions(("10.0.1", False), ("10.0.2-alpha.1", False), ("10.1.0-rc.1", False))
        self.assertEqual(api_compat.resolve_baseline(vs, 10), "10.0.1")

    def test_skips_yanked(self):
        vs = _versions(("10.0.1", False), ("10.0.2", True))
        self.assertEqual(api_compat.resolve_baseline(vs, 10), "10.0.1")

    def test_ignores_other_majors(self):
        vs = _versions(("9.9.9", False), ("10.0.0", False), ("11.0.0", False), ("100.0.0", False))
        self.assertEqual(api_compat.resolve_baseline(vs, 10), "10.0.0")

    def test_none_without_a_stable_release(self):
        vs = _versions(("11.0.0-alpha.1", False), ("10.0.0", False), ("11.0.0", True))
        self.assertIsNone(api_compat.resolve_baseline(vs, 11))
        self.assertIsNone(api_compat.resolve_baseline([], 10))

    def test_ignores_unparseable_versions(self):
        vs = _versions(("10.0", False), ("garbage", False), ("10.0.1", False))
        self.assertEqual(api_compat.resolve_baseline(vs, 10), "10.0.1")


class ParseSemverChecksTest(unittest.TestCase):
    OUTPUT = """\
     Checked [   0.234s] 229 checks: 227 pass, 2 fail, 0 warn, 31 skip

--- failure constructible_struct_adds_field: struct exhaustively constructible through public API adds field ---

Description:
A pub struct that could be exhaustively constructed has a new pub field.
        ref: https://doc.rust-lang.org/reference/expressions/struct-expr.html

Failed in:
  field Configuration.rest_address_exact in /src/apis/configuration.rs:17

--- failure enum_variant_added: enum variant added on exhaustive enum ---

Description:
A publicly-visible enum without #[non_exhaustive] has a new variant.

Failed in:
  variant DeleteResourceError:Status409 in /src/apis/resource_api.rs:92
  variant RestoreError:Status404 in /src/apis/recovery_api.rs:147

     Summary semver requires new major version: 2 major and 0 minor checks failed
"""

    def test_lists_every_failed_instance(self):
        self.assertEqual(
            api_compat.parse_semver_checks(self.OUTPUT),
            [
                "constructible_struct_adds_field: field Configuration.rest_address_exact in /src/apis/configuration.rs:17",
                "enum_variant_added: variant DeleteResourceError:Status409 in /src/apis/resource_api.rs:92",
                "enum_variant_added: variant RestoreError:Status404 in /src/apis/recovery_api.rs:147",
            ],
        )

    def test_unindented_line_ends_the_failed_block(self):
        out = "--- failure function_missing: pub fn removed ---\n\nFailed in:\n  function a::f in /src/lib.rs:1\nwarning: unrelated trailing output\n"
        self.assertEqual(api_compat.parse_semver_checks(out), ["function_missing: function a::f in /src/lib.rs:1"])

    def test_clean_output_has_no_breaks(self):
        self.assertEqual(api_compat.parse_semver_checks("     Summary no semver update required\n"), [])


class CliBaselineTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp)
        self.versions = self.tmp / "versions.json"
        self.versions.write_text(json.dumps({"versions": _versions(
            ("10.0.9", False), ("10.0.10", False), ("10.0.11", True), ("11.0.0-alpha.3", False),
        )}))

    def _main(self, *args: str) -> tuple[int, str]:
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
            rc = api_compat.main([*args, "--versions-json", str(self.versions)])
        return rc, out.getvalue()

    def test_print_baseline_major(self):
        self.assertEqual(self._main("--print-baseline-major", "10"), (0, "10.0.10\n"))

    def test_print_baseline_major_prints_nothing_without_release(self):
        self.assertEqual(self._main("--print-baseline-major", "11"), (0, "\n"))

    def test_no_baseline_passes_with_notice(self):
        rc, out = self._main("--baseline-major", "11")
        self.assertEqual(rc, 0)
        self.assertIn("::notice::No 11.x release", out)


# ---------------------------------------------------------------------------
# Synthetic crates: one per break class
# ---------------------------------------------------------------------------

E2E = os.environ.get("API_COMPAT_E2E") == "1"

BASELINE = """\
pub struct Record {
    pub name: String,
    pub count: u32,
}

#[non_exhaustive]
pub struct Options {
    pub retries: u32,
}

pub enum Mode {
    Fast,
    Slow,
}

#[non_exhaustive]
pub enum Status {
    Active,
    Done,
}

pub enum Event {
    Created(String),
    Changed { key: String },
}

pub trait Handler {
    fn handle(&self, input: &str) -> String;
    fn name(&self) -> String {
        String::new()
    }
}

pub fn lookup(key: String) -> Record {
    Record { name: key, count: 0 }
}

pub fn helper() {}

pub const fn answer() -> u32 {
    42
}

pub fn inner() -> nested::Inner {
    nested::Inner { value: 0 }
}

pub fn drain(v: Vec<u8>) -> std::vec::IntoIter<u8> {
    v.into_iter()
}

pub fn wrap<T: Clone>(item: T) -> Vec<T> {
    vec![item]
}

impl Record {
    pub fn label(&self) -> String {
        self.name.clone()
    }
}

pub mod nested {
    pub struct Inner {
        pub value: i64,
    }
}

mod private {
    pub struct Reexported {
        pub id: u64,
    }
}

pub use private::Reexported;
"""

PACKAGE = "api-compat-synthetic"
TAG = f"[{PACKAGE}]"


@unittest.skipUnless(E2E, "set API_COMPAT_E2E=1 (make test-api-compat) to build synthetic crates")
class SyntheticCrateTest(unittest.TestCase):
    """Each case edits the baseline source and asserts the verdict of the full check."""

    @classmethod
    def setUpClass(cls):
        # Under the repo's target/ so rust-toolchain.toml pins the stable toolchain.
        cls.root = api_compat.REPO_ROOT / "target" / "api-compat-tests"
        shutil.rmtree(cls.root, ignore_errors=True)

    def _crate(self, path: Path, source: str) -> None:
        (path / "src").mkdir(parents=True)
        (path / "Cargo.toml").write_text(
            f'[package]\nname = "{PACKAGE}"\nversion = "1.0.0"\nedition = "2021"\n\n[workspace]\n'
        )
        (path / "src" / "lib.rs").write_text(source)

    def check(self, *edits: tuple[str, str]) -> tuple[int, str]:
        source = BASELINE
        for old, new in edits:
            self.assertIn(old, source, "edit does not apply to the baseline")
            source = source.replace(old, new, 1)
        case = self.root / self.id().rsplit(".", 1)[-1]
        self._crate(case / "baseline", BASELINE)
        self._crate(case / "current", source)
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
            rc = api_compat.main([
                "--manifest-path", str(case / "current" / "Cargo.toml"),
                "--baseline-root", str(case / "baseline"),
                "--work-dir", str(case / "work"),
            ])
        return rc, out.getvalue()

    def assertBreaks(self, result: tuple[int, str], *fragments: str) -> None:
        rc, out = result
        self.assertEqual(rc, 1, out)
        for fragment in fragments:
            self.assertIn(fragment, out)

    # -- compatible --------------------------------------------------------

    def test_identical_source_is_compatible(self):
        rc, out = self.check()
        self.assertEqual(rc, 0, out)

    def test_additions_are_compatible(self):
        rc, out = self.check(
            ("pub fn helper() {}", "pub fn helper() {}\npub fn extra(x: Option<u8>) -> u8 { x.unwrap_or(0) }\npub struct Added;"),
            ("    pub retries: u32,\n", "    pub retries: u32,\n    pub timeout: Option<u64>,\n"),
            # Appended: inserting it would renumber `Done` (a discriminant break).
            ("    Done,\n}\n\npub enum Event", "    Done,\n    Paused,\n}\n\npub enum Event"),
            ("        String::new()\n    }\n", "        String::new()\n    }\n    fn close(&self) {}\n"),
            ("impl Record {\n", "impl Record {\n    pub fn new_method(&self) -> Option<u32> { None }\n"),
            ("pub mod nested {\n", "pub mod nested {\n    pub fn added() {}\n"),
        )
        self.assertEqual(rc, 0, out)

    def test_renaming_generics_and_params_is_compatible(self):
        rc, out = self.check(
            ("pub fn wrap<T: Clone>(item: T) -> Vec<T> {\n    vec![item]", "pub fn wrap<U: Clone>(value: U) -> Vec<U> {\n    vec![value]"),
            ("pub fn lookup(key: String) -> Record {\n    Record { name: key,", "pub fn lookup(id: String) -> Record {\n    Record { name: id,"),
        )
        self.assertEqual(rc, 0, out)

    def test_reexporting_a_used_type_is_compatible(self):
        rc, out = self.check(("pub use private::Reexported;", "pub use private::Reexported;\npub use nested::Inner;"))
        self.assertEqual(rc, 0, out)

    def test_const_and_relaxed_bounds_are_compatible(self):
        rc, out = self.check(
            ("pub fn helper() {}", "pub const fn helper() {}"),
            ("pub fn wrap<T: Clone>(item: T) -> Vec<T> {\n    vec![item]", "pub fn wrap<T>(item: T) -> Vec<T> {\n    vec![item]"),
        )
        self.assertEqual(rc, 0, out)

    # -- breaking ----------------------------------------------------------

    def test_dependency_type_with_same_name_in_another_module(self):
        self.assertBreaks(
            self.check((
                "-> std::vec::IntoIter<u8> {\n    v.into_iter()",
                "-> std::collections::vec_deque::IntoIter<u8> {\n    std::collections::VecDeque::from(v).into_iter()",
            )),
            f"{TAG} function signature changed: `api_compat_synthetic::drain`",
        )

    def test_dropping_const(self):
        self.assertBreaks(
            self.check(("pub const fn answer()", "pub fn answer()")),
            f"{TAG} function signature changed: `api_compat_synthetic::answer`",
        )

    def test_removed_item(self):
        self.assertBreaks(
            self.check(("pub fn helper() {}\n", "")),
            "[cargo-semver-checks] function_missing",
            f"{TAG} removed function `api_compat_synthetic::helper`",
        )

    def test_field_becoming_optional(self):
        """The nullability class cargo-semver-checks misses: only the rustdoc diff sees it."""
        rc, out = self.check(
            ("    pub count: u32,", "    pub count: Option<u32>,"),
            ("Record { name: key, count: 0 }", "Record { name: key, count: None }"),
            ("        pub value: i64,", "        pub value: Option<i64>,"),
            ("        pub id: u64,", "        pub id: Option<u64>,"),
        )
        self.assertBreaks(
            (rc, out),
            f"{TAG} field type changed (became optional): `api_compat_synthetic::Record.count`: `u32` -> `core::option::Option<u32>`",
            f"{TAG} field type changed (became optional): `api_compat_synthetic::nested::Inner.value`",
            f"{TAG} field type changed (became optional): `api_compat_synthetic::Reexported.id`",
        )

    def test_semver_checks_alone_misses_field_becoming_optional(self):
        """Documents the blind spot the rustdoc diff exists for. If cargo-semver-checks
        starts catching this, the second layer is still harmless; update this test."""
        case = self.root / "semver_checks_blind_spot"
        self._crate(case / "baseline", BASELINE)
        self._crate(case / "current", BASELINE.replace("    pub count: u32,", "    pub count: Option<u32>,").replace("count: 0 }", "count: None }"))
        breaks = api_compat.run_semver_checks(case / "current" / "Cargo.toml", None, case / "baseline")
        self.assertEqual(breaks, [])

    def test_enum_variant_payload_becoming_optional(self):
        self.assertBreaks(
            self.check(("    Created(String),", "    Created(Option<String>),")),
            f"{TAG} variant signature changed: `api_compat_synthetic::Event::Created`: `(alloc::string::String)` -> `(core::option::Option<alloc::string::String>)`",
        )

    def test_changed_function_signature(self):
        self.assertBreaks(
            self.check(("pub fn lookup(key: String) -> Record {\n    Record { name: key,", "pub fn lookup(key: &str) -> Record {\n    Record { name: key.to_string(),")),
            f"{TAG} function signature changed: `api_compat_synthetic::lookup`: `fn(alloc::string::String) -> api_compat_synthetic::Record` -> `fn(&str) -> api_compat_synthetic::Record`",
        )

    def test_return_type_becoming_optional(self):
        self.assertBreaks(
            self.check(
                ("    pub fn label(&self) -> String {\n        self.name.clone()", "    pub fn label(&self) -> Option<String> {\n        Some(self.name.clone())"),
            ),
            f"{TAG} method signature changed: `api_compat_synthetic::Record::label`: `fn(&Self) -> alloc::string::String` -> `fn(&Self) -> core::option::Option<alloc::string::String>`",
        )

    def test_tightened_generic_bound(self):
        self.assertBreaks(
            self.check(("pub fn wrap<T: Clone>(", "pub fn wrap<T: Clone + Send>(")),
            f"{TAG} function signature changed: `api_compat_synthetic::wrap`",
        )

    def test_new_variant_on_exhaustive_enum(self):
        self.assertBreaks(
            self.check(("    Fast,\n", "    Fast,\n    Turbo,\n")),
            "[cargo-semver-checks] enum_variant_added: variant Mode:Turbo",
        )

    def test_new_field_on_constructible_struct(self):
        self.assertBreaks(
            self.check(
                ("    pub count: u32,\n}", "    pub count: u32,\n    pub extra: bool,\n}"),
                ("count: 0 }", "count: 0, extra: false }"),
            ),
            "[cargo-semver-checks] constructible_struct_adds_field: field Record.extra",
        )

    def test_new_required_trait_method(self):
        self.assertBreaks(
            self.check(("    fn handle(&self, input: &str) -> String;\n", "    fn handle(&self, input: &str) -> String;\n    fn close(&self);\n")),
            "[cargo-semver-checks] trait_method_added",
        )

    def test_trait_method_losing_its_default(self):
        self.assertBreaks(
            self.check(("    fn name(&self) -> String {\n        String::new()\n    }\n", "    fn name(&self) -> String;\n")),
            f"{TAG} trait method `api_compat_synthetic::Handler::name` no longer has a default",
        )


if __name__ == "__main__":
    unittest.main()
