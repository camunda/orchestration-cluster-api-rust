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
import subprocess
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


class ReleaseGateOrderTest(unittest.TestCase):
    """A manual publish creates no tag: the gate may only need it to search for approvals."""

    def test_baseline_tag_is_only_required_for_the_approval_search(self):
        workflow = (api_compat.REPO_ROOT / ".github" / "workflows" / "release.yml").read_text()
        gate = workflow[workflow.index("- name: API compatibility gate"):]
        tag_check = gate.index('git rev-parse -q --verify "refs/tags/${tag}"')
        self.assertLess(gate.index('scripts/api_compat.py --baseline-version'), tag_check)
        self.assertLess(gate.index('if [ "$DISPATCH_APPROVED" = "true" ]'), tag_check)
        self.assertLess(tag_check, gate.index('scripts/api-break-approvals.sh "$tag" "$branch"'))


@unittest.skipUnless(shutil.which("jq") and shutil.which("git"), "needs jq and git")
class ApprovalSearchTest(unittest.TestCase):
    """Only a labelled PR merged into the release line after the baseline approves a break."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        repo = self.tmp / "repo"
        repo.mkdir()
        git = lambda *a: subprocess.run(["git", *a], cwd=repo, check=True, capture_output=True, text=True).stdout.strip()
        git("init", "-q")
        git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "before")
        self.before = git("rev-parse", "HEAD")
        git("tag", "v10.0.0")
        git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "--allow-empty", "-m", "after")
        self.after = git("rev-parse", "HEAD")
        self.repo = repo
        self.fixtures = self.tmp / "pulls"
        self.fixtures.mkdir()
        bin_dir = self.tmp / "bin"
        bin_dir.mkdir()
        # `gh api repos/<r>/commits/<sha>/pulls --jq <filter>`, answered from a fixture file.
        (bin_dir / "gh").write_text(
            '#!/usr/bin/env bash\nset -euo pipefail\nsha="${2##*/commits/}"; sha="${sha%/pulls}"\n'
            'f="$FIXTURES/$sha.json"; [ -f "$f" ] || f="$FIXTURES/none.json"\njq -r "$4" "$f"\n'
        )
        (bin_dir / "gh").chmod(0o755)
        (self.fixtures / "none.json").write_text("[]")
        self.env = dict(os.environ, PATH=f"{bin_dir}{os.pathsep}{os.environ['PATH']}",
                        FIXTURES=str(self.fixtures), GITHUB_REPOSITORY="o/r")

    def pr(self, number: int, base: str, merge_sha: str | None, labelled: bool = True) -> dict:
        return {
            "number": number,
            "merged_at": "2026-01-01T00:00:00Z" if merge_sha else None,
            "merge_commit_sha": merge_sha,
            "base": {"ref": base},
            "labels": [{"name": "api-break-approved"}] if labelled else [],
        }

    def approvals(self, *prs: dict) -> list[str]:
        (self.fixtures / f"{self.after}.json").write_text(json.dumps(list(prs)))
        r = subprocess.run(
            [str(api_compat.REPO_ROOT / "scripts" / "api-break-approvals.sh"), "v10.0.0", "stable/10"],
            cwd=self.repo, env=self.env, capture_output=True, text=True,
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        return r.stdout.split()

    def test_labelled_pr_merged_into_the_release_line_approves(self):
        self.assertEqual(self.approvals(self.pr(1, "stable/10", self.after)), ["#1"])

    def test_pr_merged_into_another_branch_does_not_approve(self):
        self.assertEqual(self.approvals(self.pr(2, "main", self.after)), [])

    def test_pr_merged_before_the_baseline_does_not_approve(self):
        self.assertEqual(self.approvals(self.pr(3, "stable/10", self.before)), [])

    def test_unlabelled_or_unmerged_pr_does_not_approve(self):
        self.assertEqual(
            self.approvals(self.pr(4, "stable/10", self.after, labelled=False), self.pr(5, "stable/10", None)),
            [],
        )

    def test_lookup_failure_is_an_error(self):
        (self.fixtures / f"{self.after}.json").write_text("not json")
        r = subprocess.run(
            [str(api_compat.REPO_ROOT / "scripts" / "api-break-approvals.sh"), "v10.0.0", "stable/10"],
            cwd=self.repo, env=self.env, capture_output=True, text=True,
        )
        self.assertNotEqual(r.returncode, 0)


@unittest.skipUnless(shutil.which("cargo"), "needs cargo")
class PinBaselineTest(unittest.TestCase):
    """The baseline SDK must build against its own published client, not a newer one."""

    def test_sdk_is_pinned_to_the_extracted_client(self):
        src = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, src, ignore_errors=True)
        client = src / "camunda-orchestration-api-client-10.0.0"
        sdk = src / "camunda-orchestration-sdk-10.0.0"
        for crate, deps in ((client, ""), (sdk, '[dependencies.camunda-orchestration-api-client]\nversion = "10.0.0"\ndefault-features = false\n')):
            (crate / "src").mkdir(parents=True)
            (crate / "src" / "lib.rs").write_text("")
            name = crate.name.rsplit("-", 1)[0]
            (crate / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion = "10.0.0"\nedition = "2021"\n\n{deps}')
        root = api_compat.pin_baseline(src, {"camunda-orchestration-sdk": sdk / "Cargo.toml", "camunda-orchestration-api-client": client / "Cargo.toml"})
        # Offline: resolving only succeeds if the client comes from the extracted path.
        r = subprocess.run(
            ["cargo", "metadata", "--offline", "--format-version", "1", "--manifest-path", str(root / "Cargo.toml")],
            capture_output=True, text=True,
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        meta = json.loads(r.stdout)
        resolved = next(p for p in meta["packages"] if p["name"] == "camunda-orchestration-api-client")
        self.assertIsNone(resolved["source"])
        self.assertEqual(Path(resolved["manifest_path"]).resolve(), (client / "Cargo.toml").resolve())
        self.assertEqual(len(meta["workspace_members"]), 2)


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
    pub callback: fn(u8),
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

    pub fn with_count(self, count: u32) -> Record {
        Record { count, ..self }
    }
}

pub struct Holder<T>(pub T);
pub struct Bounded<T: Clone>(pub T);
pub struct Relaxed<T>(pub T);
pub type Borrowed<'a> = &'a str;
pub type CloneVec<T: Clone> = Vec<T>;

impl<T> Holder<T> {
    pub fn choose<U>(&self, keep: T, _other: U) -> T {
        keep
    }
}

impl<T> Holder<T> {
    pub fn wrap(value: T) -> Holder<T> {
        Holder(value)
    }
}

impl<T: Clone> Relaxed<T> {
    pub fn clone_value(value: T) -> T {
        value.clone()
    }
}

impl<T: Clone> From<T> for Holder<T> {
    fn from(value: T) -> Self {
        Holder(value)
    }
}

pub struct DynHolder<'a> {
    pub handler: Box<dyn Fn() + 'a>,
    pub marker: std::marker::PhantomData<&'a ()>,
}

pub struct DynDefault {
    pub handler: Box<dyn Fn()>,
}

pub trait Lifetimes {
    fn pair<'a, 'b>(&self, first: &'a str, second: &'b str);
}

pub fn elided(value: &str) -> &str {
    value
}

pub struct Spec<T>(pub T);

impl Spec<u8> {
    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn width() -> u8 {
        8
    }
}

pub trait Lending<'a, 'b> {
    fn get(&self) -> &'a str;
}

pub struct Tag<const C: char>;

pub fn tag(_tag: Tag<'a'>) {}

pub struct Buffer<const N: usize>;

impl<const N: usize> Buffer<N> {
    pub fn capacity(&self) -> usize {
        N
    }
}

impl Spec<bool> {
    pub fn value(&self) -> bool {
        self.0
    }
}

pub struct Lent<'a>(pub &'a str);

impl<'a> Lent<'a> {
    pub fn accepts(_x: &str) {}
}

pub struct Pair<T, U>(pub T, pub U);

impl<T, U> Pair<T, U> {
    pub fn make() -> u8 {
        0
    }
}

pub fn inspect<T: ?Sized>(_value: &T) {}
pub fn inspect_compat<T>(_value: &T) {}
pub fn accepts_borrow<'a>(_value: &'a str) {}
pub fn lifetime_relation<'a, 'b>(_first: &'a str, _second: &'b str) {}
pub fn where_lifetime<'a, 'b>(_first: &'a str, _second: &'b str) {}
pub fn bounded_where_lifetime_relax<'a, 'b>(_first: &'a str, _second: &'b str) where 'a: 'b {}

impl std::str::FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "fast" { Mode::Fast } else { Mode::Slow })
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
            ("pub fn inspect", "impl std::fmt::Display for Mode {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(\"mode\") }\n}\n\npub fn inspect"),
        )
        self.assertEqual(rc, 0, out)

    def test_renaming_impl_generic_is_compatible(self):
        rc, out = self.check(("impl<T> Holder<T> {\n    pub fn choose<U>(&self, keep: T, _other: U) -> T", "impl<V> Holder<V> {\n    pub fn choose<U>(&self, keep: V, _other: U) -> V"))
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

    def test_self_and_the_owner_type_are_the_same(self):
        rc, out = self.check(
            ("-> Record {\n        Record { count", "-> Self {\n        Self { count"),
            ("pub fn wrap(value: T) -> Holder<T>", "pub fn wrap(value: T) -> Self"),
        )
        self.assertEqual(rc, 0, out)

    def test_relaxed_generic_type_and_trait_impl_bounds_are_compatible(self):
        rc, out = self.check(
            ("pub struct Bounded<T: Clone>(pub T);", "pub struct Bounded<T>(pub T);"),
            ("impl<T: Clone> From<T> for Holder<T>", "impl<T> From<T> for Holder<T>"),
        )
        self.assertEqual(rc, 0, out)

    def test_adding_maybe_sized_bound_is_compatible(self):
        rc, out = self.check(("pub fn inspect_compat<T>", "pub fn inspect_compat<T: ?Sized>"))
        self.assertEqual(rc, 0, out)

    def test_relaxed_inherent_impl_bound_is_compatible(self):
        rc, out = self.check(
            (
                "impl<T: Clone> Relaxed<T> {\n    pub fn clone_value",
                "impl<T> Relaxed<T> {\n    pub fn clone_value",
            )
        )
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

    def test_tightened_inherent_impl_bound(self):
        self.assertBreaks(
            self.check(
                ("impl<T> Holder<T> {\n    pub fn choose", "impl<T: Clone> Holder<T> {\n    pub fn choose")
            ),
            f"{TAG} method signature changed: `api_compat_synthetic::Holder::choose`",
        )

    def test_unsafe_function_pointer_field_is_breaking(self):
        self.assertBreaks(
            self.check(("pub callback: fn(u8)", "pub callback: unsafe fn(u8)")),
            f"{TAG} field type changed: `api_compat_synthetic::Options.callback`",
        )

    def test_function_pointer_abi_change_is_breaking(self):
        self.assertBreaks(
            self.check(("pub callback: fn(u8)", 'pub callback: extern "C" fn(u8)')),
            f"{TAG} field type changed: `api_compat_synthetic::Options.callback`",
        )

    def test_borrowed_input_lifetime_narrowing_is_breaking(self):
        self.assertBreaks(
            self.check(("accepts_borrow<'a>(_value: &'a str)", "accepts_borrow(_value: &'static str)")),
            f"{TAG} function signature changed: `api_compat_synthetic::accepts_borrow`",
        )

    def test_enclosing_lifetime_change_in_trait_method_is_breaking(self):
        self.assertBreaks(
            self.check(("fn get(&self) -> &'a str;", "fn get(&self) -> &'b str;")),
            f"{TAG} trait method signature changed: `api_compat_synthetic::Lending::get`",
        )

    def test_char_const_argument_change_is_breaking(self):
        self.assertBreaks(
            self.check(("pub fn tag(_tag: Tag<'a'>) {}", "pub fn tag(_tag: Tag<'b'>) {}")),
            f"{TAG} function signature changed: `api_compat_synthetic::tag`",
        )

    def test_impl_lifetime_in_method_is_breaking(self):
        # `Lent::<'static>::accepts(&local)` stops compiling.
        self.assertBreaks(
            self.check(("pub fn accepts(_x: &str) {}", "pub fn accepts(_x: &'a str) {}")),
            f"{TAG} method signature changed: `api_compat_synthetic::Lent::accepts`",
        )

    def test_impl_restricted_by_repeated_parameter_is_breaking(self):
        # `Pair::<u8, bool>::make()` no longer exists.
        self.assertBreaks(
            self.check(("impl<T, U> Pair<T, U> {", "impl<T> Pair<T, T> {")),
            f"{TAG} removed method `api_compat_synthetic::Pair::make`",
        )

    def _dependency_case(self, old_version: str, new_version: str) -> tuple[int, str]:
        """Baseline and current expose `dep::Thing`; only the resolved `dep` version differs."""
        case = self.root / self.id().rsplit(".", 1)[-1]
        for side, version in (("baseline", old_version), ("current", new_version)):
            dep = case / f"dep-{side}"
            (dep / "src").mkdir(parents=True)
            (dep / "Cargo.toml").write_text(
                f'[package]\nname = "dep"\nversion = "{version}"\nedition = "2021"\n'
            )
            (dep / "src" / "lib.rs").write_text("pub struct Thing;\n")
            crate = case / side
            (crate / "src").mkdir(parents=True)
            (crate / "Cargo.toml").write_text(
                f'[package]\nname = "{PACKAGE}"\nversion = "1.0.0"\nedition = "2021"\n\n'
                f'[dependencies]\ndep = {{ path = "../dep-{side}" }}\n\n[workspace]\n'
            )
            (crate / "src" / "lib.rs").write_text("pub fn thing() -> dep::Thing {\n    dep::Thing\n}\n")
        out = io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
            rc = api_compat.main([
                "--manifest-path", str(case / "current" / "Cargo.toml"),
                "--baseline-root", str(case / "baseline"),
                "--work-dir", str(case / "work"),
            ])
        return rc, out.getvalue()

    def test_dependency_major_upgrade_is_breaking(self):
        # Same path, `dep::Thing`, but a 1.x value is not a 2.x value.
        self.assertBreaks(
            self._dependency_case("1.0.0", "2.0.0"),
            f"{TAG} function signature changed: `api_compat_synthetic::thing`",
        )

    def test_dependency_compatible_upgrade_is_compatible(self):
        rc, out = self._dependency_case("1.0.0", "1.4.0")
        self.assertEqual(rc, 0, out)

    def test_moving_method_to_generic_impl_is_compatible(self):
        rc, out = self.check(
            ("\n    pub fn width() -> u8 {\n        8\n    }\n", "\n"),
            ("pub trait Lending", "impl<T> Spec<T> {\n    pub fn width() -> u8 {\n        8\n    }\n}\n\npub trait Lending"),
        )
        self.assertEqual(rc, 0, out)

    def test_renaming_const_impl_parameter_is_compatible(self):
        rc, out = self.check(("impl<const N: usize> Buffer<N> {\n    pub fn capacity(&self) -> usize {\n        N", "impl<const M: usize> Buffer<M> {\n    pub fn capacity(&self) -> usize {\n        M"))
        self.assertEqual(rc, 0, out)

    def test_explicit_and_elided_lifetimes_are_compatible(self):
        rc, out = self.check(
            ("pub fn elided(value: &str) -> &str", "pub fn elided<'a>(value: &'a str) -> &'a str")
        )
        self.assertEqual(rc, 0, out)

    def test_default_trait_object_lifetime_is_compatible(self):
        rc, out = self.check(("pub handler: Box<dyn Fn()>,", "pub handler: Box<dyn Fn() + 'static>,"))
        self.assertEqual(rc, 0, out)

    def test_trait_object_lifetime_narrowing_is_breaking(self):
        self.assertBreaks(
            self.check(("pub handler: Box<dyn Fn() + 'a>,", "pub handler: Box<dyn Fn() + 'static>,")),
            f"{TAG} field type changed: `api_compat_synthetic::DynHolder.handler`",
        )

    def test_static_bound_on_type_parameter_is_breaking(self):
        self.assertBreaks(
            self.check(("pub fn wrap<T: Clone>(", "pub fn wrap<T: Clone + 'static>(")),
            f"{TAG} function signature changed: `api_compat_synthetic::wrap`",
        )

    def test_trait_method_lifetime_bound_is_breaking(self):
        self.assertBreaks(
            self.check(("fn pair<'a, 'b>(&self", "fn pair<'a: 'b, 'b>(&self")),
            f"{TAG} trait method signature changed: `api_compat_synthetic::Lifetimes::pair`",
        )

    def test_change_in_first_specialized_impl_is_breaking(self):
        self.assertBreaks(
            self.check(("pub fn value(&self) -> u8 {\n        self.0", "pub fn value(&self) -> Option<u8> {\n        Some(self.0)")),
            f"{TAG} method signature changed: `api_compat_synthetic::Spec::<u8>::value`",
        )

    def test_change_in_second_specialized_impl_is_breaking(self):
        self.assertBreaks(
            self.check(("pub fn value(&self) -> bool {\n        self.0", "pub fn value(&self) -> Option<bool> {\n        Some(self.0)")),
            f"{TAG} method signature changed: `api_compat_synthetic::Spec::<bool>::value`",
        )

    def test_renaming_lifetime_is_compatible(self):
        rc, out = self.check(
            ("accepts_borrow<'a>(_value: &'a str)", "accepts_borrow<'b>(_value: &'b str)")
        )
        self.assertEqual(rc, 0, out)

    def test_tightening_lifetime_outlives_bound_is_breaking(self):
        self.assertBreaks(
            self.check(
                ("lifetime_relation<'a, 'b>", "lifetime_relation<'a: 'b, 'b>")
            ),
            f"{TAG} function signature changed: `api_compat_synthetic::lifetime_relation`",
        )

    def test_tightening_where_lifetime_bound_is_breaking(self):
        self.assertBreaks(
            self.check(
                (
                    "where_lifetime<'a, 'b>(_first: &'a str, _second: &'b str) {}",
                    "where_lifetime<'a, 'b>(_first: &'a str, _second: &'b str) where 'a: 'b {}",
                )
            ),
            f"{TAG} function signature changed: `api_compat_synthetic::where_lifetime`",
        )

    def test_relaxing_where_lifetime_bound_is_compatible(self):
        rc, out = self.check(
            (
                "bounded_where_lifetime_relax<'a, 'b>(_first: &'a str, _second: &'b str) where 'a: 'b {}",
                "bounded_where_lifetime_relax<'a, 'b>(_first: &'a str, _second: &'b str) {}",
            )
        )
        self.assertEqual(rc, 0, out)

    def test_type_alias_lifetime_narrowing_is_breaking(self):
        self.assertBreaks(
            self.check(("pub type Borrowed<'a> = &'a str;", "pub type Borrowed<'a> = &'static str;")),
            f"{TAG} type alias signature changed: `api_compat_synthetic::Borrowed`",
        )

    def test_renaming_type_alias_lifetime_is_compatible(self):
        rc, out = self.check(("pub type Borrowed<'a> = &'a str;", "pub type Borrowed<'b> = &'b str;"))
        self.assertEqual(rc, 0, out)

    def test_relaxing_type_alias_bound_is_compatible(self):
        rc, out = self.check(("pub type CloneVec<T: Clone> = Vec<T>;", "pub type CloneVec<T> = Vec<T>;"))
        self.assertEqual(rc, 0, out)

    def test_implicit_and_explicit_unit_returns_are_compatible(self):
        rc, out = self.check(("pub fn helper() {}", "pub fn helper() -> () {}"))
        self.assertEqual(rc, 0, out)

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
            ("nested::Inner { value: 0 }", "nested::Inner { value: None }"),
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
        breaks = api_compat.run_semver_checks(case / "current" / "Cargo.toml", case / "baseline")
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
            f"{TAG} method signature changed: `api_compat_synthetic::Record::label`: `fn(&api_compat_synthetic::Record) -> alloc::string::String` -> `fn(&api_compat_synthetic::Record) -> core::option::Option<alloc::string::String>`",
        )

    def test_tightened_generic_bound(self):
        self.assertBreaks(
            self.check(("pub fn wrap<T: Clone>(", "pub fn wrap<T: Clone + Send>(")),
            f"{TAG} function signature changed: `api_compat_synthetic::wrap`",
        )

    def test_generic_method_return_type_change(self):
        """Impl and method generics must get distinct ids, or `-> T` and `-> U` render alike."""
        self.assertBreaks(
            self.check(("_other: U) -> T {\n        keep", "_other: U) -> U {\n        _other")),
            f"{TAG} method signature changed: `api_compat_synthetic::Holder::choose`",
        )

    def test_removing_maybe_sized_bound(self):
        self.assertBreaks(
            self.check(("pub fn inspect<T: ?Sized>", "pub fn inspect<T>")),
            f"{TAG} function signature changed: `api_compat_synthetic::inspect`",
        )

    def test_removed_manual_trait_impl(self):
        """`\"fast\".parse::<Mode>()` stops compiling; the pinned cargo-semver-checks misses it."""
        self.assertBreaks(
            self.check((
                "impl std::str::FromStr for Mode {\n    type Err = String;\n    fn from_str(s: &str) -> Result<Self, Self::Err> {",
                "impl Mode {\n    pub fn from_str(s: &str) -> Result<Self, String> {",
            )),
            f"{TAG} removed trait impl `api_compat_synthetic::Mode::<impl core::str::traits::FromStr for api_compat_synthetic::Mode>`",
        )

    def test_changed_associated_type_of_trait_impl(self):
        self.assertBreaks(
            self.check(
                ("    type Err = String;", "    type Err = std::num::ParseIntError;"),
                ("Ok(if s == \"fast\"", "s.parse::<u8>()?;\n        Ok(if s == \"fast\""),
            ),
            f"{TAG} associated type signature changed: `api_compat_synthetic::Mode::<impl core::str::traits::FromStr for api_compat_synthetic::Mode>::Err`",
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
