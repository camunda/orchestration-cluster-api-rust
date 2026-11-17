#!/usr/bin/env python3
"""Public API compatibility check against the last published release of a major.

A stable branch (`stable/N`) regenerates from a moving upstream spec, so drift can
change the public API of an `N.x` patch release without anyone noticing. This check
compares the workspace crates with the newest published, non-yanked, non-prerelease
`N.x.y` on crates.io and fails if customer code compiled against that release could
stop compiling.

Two layers, because neither covers the whole surface on its own:

1. cargo-semver-checks (pinned) — removals, new variants on exhaustive enums, new
   fields on constructible structs, new required trait methods, lost trait impls, ...
2. A rustdoc-JSON signature diff — every public field, function and method signature,
   type alias, const, static and trait impl is rendered as written, with canonical
   paths, and compared verbatim. This covers the class cargo-semver-checks does not:
   a field, param or return type changing (`T` -> `Option<T>` is the nullability
   drift the SDKs have shipped before). It does not judge equivalence: a respelling
   such as a renamed generic is reported, and needs the approval label.

Exit codes: 0 compatible (or no baseline), 1 breaking changes (each one listed),
2 tool/setup error.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.error
import urllib.request
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
SDK_CRATE = "camunda-orchestration-sdk"
SEMVER_CHECKS_VERSION = "0.51.0"
CRATES_IO_API = "https://crates.io/api/v1/crates"
CRATES_IO_DOWNLOAD = "https://static.crates.io/crates"
USER_AGENT = "camunda-orchestration-api-rust api-compat (https://github.com/camunda/orchestration-cluster-api-rust)"
# Both sides are rendered by the same toolchain, so only the walker below has to
# understand the format. Bump together with DOCS_TOOLCHAIN after checking the
# rustdoc-types changelog.
SUPPORTED_FORMAT_VERSIONS = {61}


class SetupError(RuntimeError):
    """A tool or environment problem: exit code 2, never a verdict on the API."""


# ---------------------------------------------------------------------------
# Baseline resolution
# ---------------------------------------------------------------------------

_SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$")


def parse_version(num: str) -> tuple[int, int, int, str | None] | None:
    m = _SEMVER_RE.match(num)
    if not m:
        return None
    return int(m.group(1)), int(m.group(2)), int(m.group(3)), m.group(4)


def resolve_baseline(versions: list[dict], major: int) -> str | None:
    """Newest non-yanked, non-prerelease `major.x.y` in a crates.io versions list."""
    best: tuple[tuple[int, int, int], str] | None = None
    for v in versions:
        if v.get("yanked"):
            continue
        num = str(v.get("num", ""))
        parsed = parse_version(num)
        if parsed is None or parsed[3] is not None or parsed[0] != major:
            continue
        key = parsed[:3]
        if best is None or key > best[0]:
            best = (key, num)
    return best[1] if best else None


def _http_get(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            return resp.read()
    except (urllib.error.URLError, TimeoutError) as e:
        raise SetupError(f"GET {url} failed: {e}") from e


def fetch_versions(crate: str, versions_json: Path | None = None) -> list[dict]:
    """All published versions of `crate` (follows crates.io pagination)."""
    if versions_json is not None:
        return json.loads(versions_json.read_text(encoding="utf-8"))["versions"]
    out: list[dict] = []
    url: str | None = f"{CRATES_IO_API}/{crate}/versions?per_page=100"
    while url:
        try:
            data = json.loads(_http_get(url))
        except json.JSONDecodeError as e:
            raise SetupError(f"crates.io returned invalid JSON for {crate}: {e}") from e
        if "versions" not in data:
            raise SetupError(f"crates.io response for {crate} has no versions: {data}")
        out.extend(data["versions"])
        next_page = (data.get("meta") or {}).get("next_page")
        url = f"{CRATES_IO_API}/{crate}/versions{next_page}" if next_page else None
    return out


# ---------------------------------------------------------------------------
# Workspace + baseline sources
# ---------------------------------------------------------------------------


def _run(cmd: list[str], cwd: Path | None = None, env: dict | None = None) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True)
    except FileNotFoundError as e:
        raise SetupError(f"{cmd[0]} not found: {e}") from e


def workspace_packages(manifest: Path) -> dict[str, Path]:
    """Library packages of the workspace at `manifest`: name -> Cargo.toml."""
    r = _run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", str(manifest)])
    if r.returncode != 0:
        raise SetupError(f"cargo metadata failed for {manifest}:\n{r.stderr}")
    meta = json.loads(r.stdout)
    members = set(meta["workspace_members"])
    out: dict[str, Path] = {}
    for p in meta["packages"]:
        if p["id"] in members and any("lib" in t["kind"] for t in p["targets"]):
            out[p["name"]] = Path(p["manifest_path"])
    return out


def package_version(manifest: Path, name: str) -> str:
    r = _run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", str(manifest)])
    if r.returncode != 0:
        raise SetupError(f"cargo metadata failed for {manifest}:\n{r.stderr}")
    for p in json.loads(r.stdout)["packages"]:
        if p["name"] == name:
            return p["version"]
    raise SetupError(f"package {name} not found in {manifest}")


def download_crate(name: str, version: str, dest: Path, versions_json: Path | None) -> Path:
    """Extract the published `name-version` source and return its Cargo.toml."""
    root = dest / f"{name}-{version}"
    manifest = root / "Cargo.toml"
    if manifest.is_file():
        return manifest
    entry = next((v for v in fetch_versions(name, versions_json) if v.get("num") == version), None)
    if entry is None:
        raise SetupError(f"{name} {version} is not published on crates.io")
    blob = _http_get(f"{CRATES_IO_DOWNLOAD}/{name}/{name}-{version}.crate")
    checksum = entry.get("checksum")
    if checksum and hashlib.sha256(blob).hexdigest() != checksum:
        raise SetupError(f"checksum mismatch for {name}-{version}.crate")
    dest.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(blob), mode="r:gz") as tar:
        base = dest.resolve()
        for member in tar.getmembers():
            target = (base / member.name).resolve()
            if not (member.isfile() or member.isdir()) or base not in target.parents:
                raise SetupError(f"refusing to extract {member.name!r} from {name}-{version}.crate")
        tar.extractall(dest)
    return manifest


def pin_baseline(src: Path, manifests: dict[str, Path]) -> Path:
    """Make the extracted crates one workspace whose members depend on each other by path.

    A published SDK requires its client as `^N.x.y`, so on its own it can resolve a newer
    client than the one it was released with (one with new exhaustive enum variants, say)
    and stop compiling. Returns the workspace root, used by both checker layers."""
    members = sorted(m.parent.name for m in manifests.values())
    (src / "Cargo.toml").write_text(
        "[workspace]\nresolver = \"2\"\nmembers = [" + ", ".join(f'"{m}"' for m in members) + "]\n",
        encoding="utf-8",
    )
    for manifest in manifests.values():
        text = manifest.read_text(encoding="utf-8")
        # Earlier versions of this script made each extracted crate its own workspace root.
        text = re.sub(r"\n\[workspace\]\n*\Z", "\n", text)
        for name, dep in manifests.items():
            if dep == manifest:
                continue
            rel = f"../{dep.parent.name}"
            text = re.sub(
                rf"^(\[(?:target\.[^\]\n]+\.)?(?:dev-|build-)?dependencies\.{re.escape(name)}\]\n)(?!path = )",
                lambda m: f'{m.group(1)}path = "{rel}"\n',
                text,
                flags=re.M,
            )
        manifest.write_text(text, encoding="utf-8")
    return src


# ---------------------------------------------------------------------------
# Layer 1: cargo-semver-checks
# ---------------------------------------------------------------------------


def check_semver_tool() -> None:
    r = _run(["cargo", "semver-checks", "--version"])
    if r.returncode != 0:
        raise SetupError(
            f"cargo-semver-checks is not installed. Install the pinned version:\n"
            f"  cargo install --locked cargo-semver-checks@{SEMVER_CHECKS_VERSION}"
        )
    found = r.stdout.strip().split()[-1] if r.stdout.strip() else ""
    if found != SEMVER_CHECKS_VERSION:
        raise SetupError(
            f"cargo-semver-checks {found or '?'} found, {SEMVER_CHECKS_VERSION} required:\n"
            f"  cargo install --locked cargo-semver-checks@{SEMVER_CHECKS_VERSION}"
        )


_FAILURE_RE = re.compile(r"^--- failure ([a-z0-9_]+): (.*) ---$")


def parse_semver_checks(output: str) -> list[str]:
    """One line per failed instance: `<lint>: <location>`."""
    breaks: list[str] = []
    lint = None
    in_failed = False
    for line in output.splitlines():
        m = _FAILURE_RE.match(line.strip())
        if m:
            lint, in_failed = m.group(1), False
            continue
        if lint and line.strip() == "Failed in:":
            in_failed = True
            continue
        if in_failed:
            # Instances are indented; anything else (blank line, summary) ends the block.
            if not line.startswith(" ") or not line.strip():
                lint, in_failed = None, False
                continue
            breaks.append(f"{lint}: {line.strip()}")
    return breaks


def run_semver_checks(manifest: Path, baseline_root: Path) -> list[str]:
    cmd = [
        "cargo", "semver-checks", "check-release",
        "--manifest-path", str(manifest), "--workspace",
        # The checked version is a patch/minor of the baseline by definition (same
        # major), so only changes that require a new major fail.
        "--release-type", "minor",
        "--color", "never",
        # The pinned baseline workspace, not `--baseline-version`: that resolves the
        # published SDK's client requirement afresh.
        "--baseline-root", str(baseline_root),
    ]
    r = _run(cmd)
    output = r.stdout + r.stderr
    # Lint reports go to stdout, progress to stderr.
    breaks = parse_semver_checks(r.stdout)
    # A crate that fails to build is unchecked even when another crate reported lints, and
    # an approval must not cover that. A lint-only failure prints no `error:` line.
    if re.search(r"^error(\[|:)", r.stderr, re.M):
        raise SetupError(f"cargo-semver-checks could not check every crate:\n{output}")
    if r.returncode != 0 and not breaks:
        raise SetupError(f"cargo-semver-checks failed without reporting a lint:\n{output}")
    if r.returncode == 0 and breaks:
        raise SetupError(f"cargo-semver-checks passed but reported failures:\n{output}")
    return breaks


# ---------------------------------------------------------------------------
# Layer 2: rustdoc JSON signature diff
# ---------------------------------------------------------------------------


def docs_toolchain() -> str:
    env = os.environ.get("API_COMPAT_TOOLCHAIN")
    if env:
        return env
    makefile = (REPO_ROOT / "Makefile").read_text(encoding="utf-8")
    m = re.search(r"^DOCS_TOOLCHAIN \?= *(\S+)", makefile, re.M)
    if not m:
        raise SetupError("DOCS_TOOLCHAIN not found in Makefile")
    return m.group(1)


def build_rustdoc_json(manifest: Path, package: str, target_dir: Path) -> tuple[Path, dict[str, str]]:
    """The rustdoc JSON of `package`, and the semver-compatible version (`1`, `0.12`) of
    every compiled dependency, keyed by artifact file name."""
    toolchain = docs_toolchain()
    cmd = [
        "cargo", f"+{toolchain}", "rustdoc", "--lib", "-p", package,
        "--manifest-path", str(manifest), "--message-format", "json",
        "--", "-Z", "unstable-options", "--output-format", "json",
    ]
    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir))
    r = _run(cmd, env=env)
    if r.returncode != 0:
        raise SetupError(
            f"rustdoc JSON build failed for {package} ({manifest}):\n{r.stderr}\n{r.stdout[-4000:]}\n"
            f"Install the toolchain with: rustup toolchain install {toolchain} --profile minimal"
        )
    out = target_dir / "doc" / f"{package.replace('-', '_')}.json"
    if not out.is_file():
        raise SetupError(f"rustdoc did not produce {out}")
    versions: dict[str, str] = {}
    for line in r.stdout.splitlines():
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-artifact":
            continue
        bucket = _compat_bucket(msg.get("package_id", ""))
        for f in msg.get("filenames", []):
            versions[Path(f).name] = bucket
    return out, versions


def _compat_bucket(package_id: str) -> str:
    """`registry+...#reqwest@0.12.5` -> `0.12`: versions within one bucket are compatible."""
    version = package_id.rsplit("#", 1)[-1].rsplit("@", 1)[-1]
    parsed = parse_version(version)
    if parsed is None:
        return version
    major, minor, patch = parsed[:3]
    if major:
        return str(major)
    return f"0.{minor}" if minor else f"0.0.{patch}"


@dataclass(frozen=True)
class Entry:
    """One public API element. `sig` is compared verbatim; `kind` drives the message."""

    kind: str
    sig: str
    # Trait methods/assoc items: implementors must provide it.
    required: bool = False
    # Key of the item this is a member of (fields, variants, methods, trait items).
    owner: str | None = None
    # The same element under its owner's shortest public path: an item re-exported at
    # several paths is one change, not one per path.
    canonical: str = ""


# Local types render as a placeholder holding their item id; `Surface.resolver` names
# them with a public path that exists on both sides, so a new re-export is not a change.
_LOCAL = re.compile("\x1f([^\x1f]*)\x1f")


class Surface:
    """Public API of one crate, keyed by public path."""

    def __init__(self, doc: dict, dep_versions: dict[str, str] | None = None):
        version = doc.get("format_version")
        if version not in SUPPORTED_FORMAT_VERSIONS:
            raise SetupError(
                f"unsupported rustdoc JSON format_version {version} "
                f"(supported: {sorted(SUPPORTED_FORMAT_VERSIONS)})"
            )
        self.index: dict[str, dict] = doc["index"]
        self.paths: dict[str, dict] = doc.get("paths", {})
        self.crates: dict[str, dict] = doc.get("external_crates", {})
        # Crate id -> compatible version of the dependency it was built from (std has none).
        self.crate_versions: dict[str, str] = {}
        for crate_id, crate in self.crates.items():
            bucket = (dep_versions or {}).get(Path(crate.get("path") or "").name)
            if bucket:
                self.crate_versions[str(crate_id)] = bucket
        self.root = str(doc["root"])
        self.crate_name = self.index[self.root]["name"]
        self.public_path: dict[str, str] = {}
        self.all_paths: dict[str, set[str]] = {}
        self.entries: dict[str, Entry] = {}
        self._collect_public_paths()
        for item_id, path in sorted(self._reachable, key=lambda x: x[1]):
            self._record(item_id, path)

    # -- reachability ------------------------------------------------------

    def _collect_public_paths(self) -> None:
        """BFS from the crate root: every (item, path) a downstream crate can name."""
        self._reachable: set[tuple[str, str]] = set()
        seen_modules: set[tuple[str, str]] = set()
        queue: list[tuple[str, str]] = [(self.root, self.crate_name)]
        while queue:
            mod_id, prefix = queue.pop(0)
            if (mod_id, prefix) in seen_modules or prefix.count("::") > 32:
                continue
            seen_modules.add((mod_id, prefix))
            module = self.index[mod_id]["inner"]["module"]
            for child_id in module.get("items", []):
                child = self.index.get(str(child_id))
                if child is None or child.get("visibility") != "public":
                    continue
                inner = child.get("inner", {})
                if "use" in inner:
                    use = inner["use"]
                    target_id = str(use.get("id")) if use.get("id") is not None else None
                    target = self.index.get(target_id) if target_id else None
                    if target is None:
                        # Re-export of another crate: that crate is checked on its own.
                        continue
                    if use.get("is_glob"):
                        if "module" in target.get("inner", {}):
                            queue.append((target_id, prefix))
                        continue
                    self._reach(target_id, f"{prefix}::{use['name']}", queue)
                    continue
                name = child.get("name")
                if name:
                    self._reach(str(child_id), f"{prefix}::{name}", queue)

    def _reach(self, item_id: str, path: str, queue: list) -> None:
        item = self.index[item_id]
        if "module" in item.get("inner", {}):
            queue.append((item_id, path))
        self._reachable.add((item_id, path))
        self.all_paths.setdefault(item_id, set()).add(path)
        current = self.public_path.get(item_id)
        if current is None or (path.count("::"), path) < (current.count("::"), current):
            self.public_path[item_id] = path

    # -- rendering ---------------------------------------------------------

    def resolver(self, other: "Surface"):
        """Name each local type by its shortest public path that `other` also has."""
        def name(m: re.Match) -> str:
            paths = self.all_paths[m.group(1)]
            shared = [p for p in paths if p in other.path_set] or list(paths)
            return min(shared, key=lambda p: (p.count("::"), p))
        return lambda s: _LOCAL.sub(name, s)

    @property
    def path_set(self) -> set[str]:
        if not hasattr(self, "_path_set"):
            self._path_set = {p for ps in self.all_paths.values() for p in ps}
        return self._path_set

    def _type_path(self, rp: dict) -> str:
        item_id = str(rp.get("id")) if rp.get("id") is not None else None
        if item_id in self.all_paths:
            return f"\x1f{item_id}\x1f"
        entry = self.paths.get(item_id) if item_id else None
        if entry and entry.get("path"):
            # Full defining path: `mpsc::Receiver` and `oneshot::Receiver` must differ. The
            # dependency's compatible version too: `reqwest@0.12::Client` is not 0.13's.
            path = list(entry["path"])
            bucket = self.crate_versions.get(str(entry.get("crate_id")))
            if bucket:
                path[0] = f"{path[0]}@{bucket}"
            return "::".join(path)
        return rp.get("path", "_")

    def render(self, t: Any) -> str:
        if not isinstance(t, dict):
            return "_"
        if "primitive" in t:
            return t["primitive"]
        if "generic" in t:
            return t["generic"]
        if "resolved_path" in t:
            rp = t["resolved_path"]
            return self._type_path(rp) + self._args(rp.get("args"))
        if "borrowed_ref" in t:
            br = t["borrowed_ref"]
            lifetime = f"{br['lifetime']} " if br.get("lifetime") else ""
            return f"&{lifetime}{'mut ' if br.get('is_mutable') else ''}{self.render(br.get('type'))}"
        if "raw_pointer" in t:
            rp = t["raw_pointer"]
            return f"*{'mut' if rp.get('is_mutable') else 'const'} {self.render(rp.get('type'))}"
        if "tuple" in t:
            return "(" + ", ".join(self.render(x) for x in t["tuple"]) + ")"
        if "slice" in t:
            return f"[{self.render(t['slice'])}]"
        if "array" in t:
            a = t["array"]
            return f"[{self.render(a.get('type'))}; {a.get('len')}]"
        if "impl_trait" in t:
            return "impl " + self._bounds(t["impl_trait"])
        if "dyn_trait" in t:
            dt = t["dyn_trait"]
            parts = [self._poly_trait(pt) for pt in dt.get("traits", [])]
            if dt.get("lifetime"):
                parts.append(dt["lifetime"])
            return "dyn " + " + ".join(parts)
        if "qualified_path" in t:
            qp = t["qualified_path"]
            trait = qp.get("trait")
            as_trait = f" as {self._type_path(trait)}{self._args(trait.get('args'))}" if trait else ""
            return f"<{self.render(qp.get('self_type'))}{as_trait}>::{qp.get('name')}{self._args(qp.get('args'))}"
        if "function_pointer" in t:
            fp = t["function_pointer"]
            header = fp.get("header") or {}
            safety = "unsafe " if header.get("is_unsafe") else ""
            abi = header.get("abi", "Rust")
            convention = "" if abi == "Rust" else f'extern "{abi}" '
            return self._binder(fp.get("generic_params")) + convention + safety + "fn" + self._sig(fp.get("sig", {}))
        if "pat" in t:
            return self.render(t["pat"].get("type"))
        return "_"

    def _args(self, args: Any) -> str:
        if not isinstance(args, dict):
            return ""
        if "angle_bracketed" in args:
            ab = args["angle_bracketed"]
            parts: list[str] = []
            for a in ab.get("args", []):
                if isinstance(a, dict) and "type" in a:
                    parts.append(self.render(a["type"]))
                elif isinstance(a, dict) and "lifetime" in a:
                    parts.append(a["lifetime"])
                elif isinstance(a, dict) and "const" in a:
                    parts.append(str(a["const"].get("expr", "")))
                elif a == "infer":
                    parts.append("_")
            for c in ab.get("constraints", []):
                binding = c.get("binding") or {}
                name = c.get("name", "") + self._args(c.get("args"))
                if "equality" in binding:
                    term = binding["equality"]
                    value = self.render(term["type"]) if "type" in term else str(term.get("constant", {}).get("expr", ""))
                    parts.append(f"{name} = {value}")
                elif "constraint" in binding:
                    parts.append(f"{name}: {self._bounds(binding['constraint'])}")
            return f"<{', '.join(parts)}>" if parts else ""
        if "parenthesized" in args:
            p = args["parenthesized"]
            out = p.get("output")
            ret = f" -> {self.render(out)}" if out else ""
            return "(" + ", ".join(self.render(x) for x in p.get("inputs", [])) + ")" + ret
        if "return_type_notation" in args:
            return "(..)"
        return ""

    def _binder(self, params: list | None) -> str:
        """`for<'a> ` of a higher-ranked bound or fn pointer, as written."""
        rendered = self._params(params)
        return f"for{rendered} " if rendered else ""

    def _poly_trait(self, pt: dict) -> str:
        tr = pt.get("trait") or {}
        return self._binder(pt.get("generic_params")) + self._type_path(tr) + self._args(tr.get("args"))

    def _bound_list(self, bounds: list) -> list[str]:
        out: list[str] = []
        for b in bounds or []:
            if isinstance(b, dict) and "trait_bound" in b:
                tb = b["trait_bound"]
                mod = {"maybe": "?", "maybe_const": "~const "}.get(tb.get("modifier"), "")
                out.append(mod + self._poly_trait(tb))
            elif isinstance(b, dict) and "outlives" in b:
                out.append(b["outlives"])
        return sorted(out)

    def _bounds(self, bounds: list) -> str:
        return " + ".join(self._bound_list(bounds))

    def _params(self, params: list | None) -> str:
        """A generic parameter list as written: names, bounds and defaults."""
        rendered = []
        for p in params or []:
            kind = p.get("kind") or {}
            if "lifetime" in kind:
                outlives = kind["lifetime"].get("outlives", [])
                rendered.append(p["name"] + (f": {' + '.join(outlives)}" if outlives else ""))
            elif "type" in kind:
                if kind["type"].get("is_synthetic"):
                    continue  # `impl Trait` argument: rendered at the parameter instead
                bounds = self._bounds(kind["type"].get("bounds"))
                s = p["name"] + (f": {bounds}" if bounds else "")
                if kind["type"].get("default") is not None:
                    s += f" = {self.render(kind['type']['default'])}"
                rendered.append(s)
            elif "const" in kind:
                s = f"const {p['name']}: {self.render(kind['const'].get('type'))}"
                if kind["const"].get("default") is not None:
                    s += f" = {kind['const']['default']}"
                rendered.append(s)
        return f"<{', '.join(rendered)}>" if rendered else ""

    def _where(self, generics: dict | None) -> str:
        preds = []
        for wp in (generics or {}).get("where_predicates", []):
            if "bound_predicate" in wp:
                bp = wp["bound_predicate"]
                preds.append(
                    f"{self._binder(bp.get('generic_params'))}{self.render(bp.get('type'))}: {self._bounds(bp.get('bounds'))}"
                )
            elif "lifetime_predicate" in wp:
                lp = wp["lifetime_predicate"]
                preds.append(f"{lp.get('lifetime')}: {' + '.join(lp.get('outlives', []))}")
            elif "eq_predicate" in wp:
                ep = wp["eq_predicate"]
                rhs = ep.get("rhs") or {}
                rhs_s = self.render(rhs["type"]) if "type" in rhs else str(rhs)
                preds.append(f"{self.render(ep.get('lhs'))} == {rhs_s}")
        return " where " + ", ".join(sorted(preds)) if preds else ""

    def _generics(self, generics: dict | None) -> str:
        return self._params((generics or {}).get("params")) + self._where(generics)

    def _sig(self, sig: dict) -> str:
        params = [self.render(ty) for _, ty in sig.get("inputs", [])]
        if sig.get("is_c_variadic"):
            params.append("...")
        out = sig.get("output")
        return "(" + ", ".join(params) + ")" + (f" -> {self.render(out)}" if out else "")

    def _fn(self, fn: dict) -> str:
        header = fn.get("header") or {}
        prefix = "".join(
            word for flag, word in (("is_const", "const "), ("is_async", "async "), ("is_unsafe", "unsafe "))
            if header.get(flag)
        )
        generics = fn.get("generics")
        # Param names are not part of the contract; their types and order are.
        return f"{prefix}fn{self._params((generics or {}).get('params'))}{self._sig(fn.get('sig') or {})}{self._where(generics)}"

    # -- recording ---------------------------------------------------------

    def _add(self, path: str, entry: Entry) -> None:
        self.entries.setdefault(path, entry)

    def _member(self, owner_id: str, owner_path: str, sep: str, name: str, entry: Entry) -> None:
        canonical = f"{self.public_path[owner_id]}{sep}{name}"
        self._add(
            f"{owner_path}{sep}{name}",
            replace(entry, owner=owner_path, canonical=canonical),
        )

    def _record(self, item_id: str, path: str) -> None:
        item = self.index[item_id]
        inner = item.get("inner", {})
        kind = next(iter(inner), "")
        body = inner.get(kind) or {}
        canonical = self.public_path[item_id]
        if kind in ("struct", "union"):
            self._add(path, Entry(kind, f"{kind}{self._generics(body.get('generics'))}", canonical=canonical))
            if kind == "union":
                fields = body.get("fields", [])
            else:
                skind = body.get("kind")
                fields = []
                if isinstance(skind, dict) and "plain" in skind:
                    fields = skind["plain"].get("fields", [])
                elif isinstance(skind, dict) and "tuple" in skind:
                    fields = skind["tuple"]
            self._record_fields(path, canonical, fields, public_only=True)
            self._record_impls(item_id, path, body.get("impls", []))
        elif kind == "enum":
            self._add(path, Entry("enum", f"enum{self._generics(body.get('generics'))}", canonical=canonical))
            for vid in body.get("variants", []):
                v = self.index.get(str(vid))
                if v is None:
                    continue
                vkind = v["inner"]["variant"].get("kind")
                shape = next(iter(vkind)) if isinstance(vkind, dict) else "unit"
                self._member(item_id, path, "::", v["name"], Entry("variant", shape))
                fields = vkind["tuple"] if shape == "tuple" else vkind["struct"].get("fields", []) if shape == "struct" else []
                # Field by field: an added field is cargo-semver-checks' call (it knows `#[non_exhaustive]`).
                self._record_fields(f"{path}::{v['name']}", f"{canonical}::{v['name']}", fields, public_only=False)
            self._record_impls(item_id, path, body.get("impls", []))
        elif kind == "trait":
            supertraits = self._bounds(body.get("bounds"))
            self._add(path, Entry(
                "trait", f"trait{self._generics(body.get('generics'))}" + (f": {supertraits}" if supertraits else ""),
                canonical=canonical,
            ))
            for mid in body.get("items", []):
                m = self.index.get(str(mid))
                if m is None or not m.get("name"):
                    continue
                minner = m.get("inner", {})
                if "function" in minner:
                    fn = minner["function"]
                    entry = Entry("trait method", self._fn(fn), required=not fn.get("has_body"))
                elif "assoc_type" in minner:
                    at = minner["assoc_type"]
                    bounds = self._bounds(at.get("bounds"))
                    entry = Entry(
                        "associated type", f"type{self._generics(at.get('generics'))}" + (f": {bounds}" if bounds else ""),
                        required=at.get("type") is None,
                    )
                elif "assoc_const" in minner:
                    ac = minner["assoc_const"]
                    entry = Entry("associated const", f"const: {self.render(ac.get('type'))}", required=ac.get("value") is None)
                else:
                    continue
                self._member(item_id, path, "::", m["name"], entry)
        elif kind == "function":
            self._add(path, Entry("function", self._fn(body), canonical=canonical))
        elif kind == "type_alias":
            self._add(path, Entry(
                "type alias", f"type{self._generics(body.get('generics'))} = {self.render(body.get('type'))}",
                canonical=canonical,
            ))
        elif kind == "constant":
            self._add(path, Entry("const", f"const: {self.render(body.get('type'))}", canonical=canonical))
        elif kind == "static":
            mut = "mut " if body.get("is_mutable") else ""
            self._add(path, Entry("static", f"static {mut}{self.render(body.get('type'))}", canonical=canonical))
        elif kind in ("macro", "proc_macro", "module", "trait_alias"):
            self._add(path, Entry(kind, kind, canonical=canonical))

    def _record_fields(self, owner_path: str, owner_canonical: str, field_ids: list, public_only: bool) -> None:
        for pos, fid in enumerate(field_ids):
            f = self.index.get(str(fid)) if fid is not None else None
            # Variant fields are as visible as their enum; rustdoc marks them `default`.
            if f is None or (public_only and f.get("visibility") != "public"):
                continue
            name = f.get("name") or str(pos)
            self._add(f"{owner_path}.{name}", Entry(
                "field", self.render(f["inner"]["struct_field"]),
                owner=owner_path, canonical=f"{owner_canonical}.{name}",
            ))

    def _record_impls(self, owner_id: str, path: str, impl_ids: list) -> None:
        # Methods of one name across inherent impls (`impl Spec<u8>` and `impl Spec<bool>`
        # can both define `value`) are one entry: each signature with its impl header.
        members: dict[str, tuple[str, list[str]]] = {}
        for impl_id in impl_ids:
            impl_item = self.index.get(str(impl_id))
            if impl_item is None:
                continue
            impl = impl_item["inner"].get("impl") or {}
            # Auto-trait and blanket impls follow from other items; those are checked instead.
            if impl.get("is_synthetic") or impl.get("blanket_impl"):
                continue
            if impl.get("trait") is not None:
                self._record_trait_impl(owner_id, path, impl)
                continue
            generics = impl.get("generics")
            header = f"impl{self._params((generics or {}).get('params'))} {self.render(impl.get('for'))}{self._where(generics)}"
            for mid in impl.get("items", []):
                m = self.index.get(str(mid))
                if m is None or m.get("visibility") != "public" or not m.get("name"):
                    continue
                minner = m.get("inner", {})
                if "function" in minner:
                    kind, sig = "method", self._fn(minner["function"])
                elif "assoc_const" in minner:
                    kind, sig = "associated const", f"const: {self.render(minner['assoc_const'].get('type'))}"
                else:
                    continue
                members.setdefault(m["name"], (kind, []))[1].append(f"{header} {{ {sig} }}")
        for name, (kind, sigs) in members.items():
            self._member(owner_id, path, "::", name, Entry(kind, _ALT.join(sigs)))

    def _record_trait_impl(self, owner_id: str, path: str, impl: dict) -> None:
        """The impl and its associated types; method signatures are the trait's."""
        tr = impl["trait"]
        trait_id = str(tr.get("id")) if tr.get("id") is not None else None
        known = self.paths.get(trait_id) if trait_id else None
        if trait_id not in self.all_paths and (known is None or known.get("crate_id") == 0):
            return  # a private trait: downstream cannot name it
        neg = "!" if impl.get("is_negative") else ""
        name = f"<impl {neg}{self._type_path(tr)}{self._args(tr.get('args'))} for {self.render(impl.get('for'))}>"
        self._member(owner_id, path, "::", name, Entry("trait impl", f"impl{self._generics(impl.get('generics'))}"))
        impl_key = f"{path}::{name}"
        for mid in impl.get("items", []):
            m = self.index.get(str(mid))
            at = (m or {}).get("inner", {}).get("assoc_type")
            if at is None or not m.get("name"):
                continue
            self._add(f"{impl_key}::{m['name']}", Entry(
                "associated type", f"type = {self.render(at.get('type'))}",
                owner=impl_key, canonical=f"{self.public_path[owner_id]}::{name}::{m['name']}",
            ))


def _became_optional(old: str, new: str) -> bool:
    return new in (f"core::option::Option<{old}>", f"core::option::Option<alloc::boxed::Box<{old}>>")


# Separates one method's signatures across inherent impls. They are ordered only after local
# item ids resolve to paths: ids are per build, so sorting earlier can flip the order.
_ALT = "\x1d"


def _sig_text(resolve, sig: str) -> str:
    return " | ".join(sorted(resolve(s) for s in sig.split(_ALT)))


def diff_surfaces(old: Surface, new: Surface) -> list[str]:
    """Breaking differences in `new` relative to `old`. Additions are compatible; an
    equivalent respelling is not recognised (see the module docstring)."""
    breaks: list[str] = []
    reported: set[tuple[str, str]] = set()
    ro, rn = old.resolver(new), new.resolver(old)

    def report(key: tuple[str, str], message: str) -> None:
        if key not in reported:
            reported.add(key)
            breaks.append(message)

    # Trait-impl keys name local types by item id; resolve them to compare across builds.
    new_entries = {rn(p): e for p, e in new.entries.items()}
    for raw_path, before in sorted(old.entries.items()):
        path = ro(raw_path)
        after = new_entries.get(path)
        if after is None:
            if before.owner is not None and ro(before.owner) not in new_entries:
                continue  # the owner's own removal is the break
            report((ro(before.canonical), "removed"), f"removed {before.kind} `{path}`")
            continue
        if before.kind != after.kind:
            report((before.canonical, "kind"), f"`{path}` changed from {before.kind} to {after.kind}")
            continue
        b_sig, a_sig = _sig_text(ro, before.sig), _sig_text(rn, after.sig)
        if b_sig != a_sig:
            label = f"{before.kind} type" if before.kind == "field" else f"{before.kind} signature"
            note = " (became optional)" if _became_optional(b_sig, a_sig) else ""
            report(
                (before.canonical, "sig"),
                f"{label} changed{note}: `{rn(after.canonical)}`: `{b_sig}` -> `{a_sig}`",
            )
            continue
        if after.required and not before.required:
            report(
                (before.canonical, "required"),
                f"{before.kind} `{rn(after.canonical)}` no longer has a default: implementors must now provide it",
            )
    return breaks


def load_surface(built: tuple[Path, dict[str, str]]) -> Surface:
    path, dep_versions = built
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        raise SetupError(f"cannot read rustdoc JSON {path}: {e}") from e
    return Surface(doc, dep_versions)


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------


def check(manifest: Path, baseline_version: str | None, baseline_root: Path | None,
          versions_json: Path | None, work_dir: Path) -> list[str]:
    check_semver_tool()
    packages = workspace_packages(manifest)

    if baseline_root is None:
        src = work_dir / "baseline-src"
        downloaded = {
            name: download_crate(name, str(baseline_version), src, versions_json if name == SDK_CRATE else None)
            for name in packages
        }
        baseline_root = pin_baseline(src, downloaded)
        # Same third-party versions on both sides where the requirements allow; a dependency
        # bumped across a compatibility boundary shows as a changed type path.
        lock = manifest.parent / "Cargo.lock"
        if lock.is_file():
            shutil.copyfile(lock, src / "Cargo.lock")
    breaks = [f"[cargo-semver-checks] {b}" for b in run_semver_checks(manifest, baseline_root)]

    baseline_manifests = workspace_packages(baseline_root / "Cargo.toml")
    for name in sorted(packages):
        if name not in baseline_manifests:
            continue  # a crate new since the baseline has nothing to break
        old = load_surface(build_rustdoc_json(baseline_manifests[name], name, work_dir / "baseline-target"))
        new = load_surface(build_rustdoc_json(manifest, name, work_dir / "current-target"))
        breaks += [f"[{name}] {b}" for b in diff_surfaces(old, new)]
    return breaks


def _summary(lines: list[str]) -> None:
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if path:
        with open(path, "a", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--baseline-major", type=int, help="compare with the newest published N.x.y (default: major of the SDK crate)")
    mode.add_argument("--baseline-version", help="compare with this published version")
    mode.add_argument("--baseline-root", type=Path, help="compare with a local workspace/crate (tests)")
    mode.add_argument("--print-baseline-major", type=int, metavar="N", help="print the newest published N.x.y (empty if none) and exit")
    ap.add_argument("--manifest-path", type=Path, default=REPO_ROOT / "Cargo.toml")
    ap.add_argument("--versions-json", type=Path, help="read the crates.io versions response from a file (offline/tests)")
    ap.add_argument("--work-dir", type=Path, help="scratch dir for baseline sources and rustdoc output (default: target/api-compat)")
    args = ap.parse_args(argv)

    try:
        if args.print_baseline_major is not None:
            print(resolve_baseline(fetch_versions(SDK_CRATE, args.versions_json), args.print_baseline_major) or "")
            return 0

        manifest = args.manifest_path.resolve()
        baseline_version = args.baseline_version
        label = str(args.baseline_root) if args.baseline_root else None
        if args.baseline_root is None and baseline_version is None:
            major = args.baseline_major
            if major is None:
                parsed = parse_version(package_version(manifest, SDK_CRATE))
                if parsed is None:
                    raise SetupError("cannot parse the SDK crate version")
                major = parsed[0]
            baseline_version = resolve_baseline(fetch_versions(SDK_CRATE, args.versions_json), major)
            if baseline_version is None:
                print(f"::notice::No {major}.x release of {SDK_CRATE} on crates.io; nothing to compare against.")
                return 0
        label = label or f"{SDK_CRATE} {baseline_version}"
        print(f"API baseline: {label}")

        work_dir = (args.work_dir or manifest.parent / "target" / "api-compat").resolve()
        breaks = check(manifest, baseline_version, args.baseline_root, args.versions_json, work_dir)
    except SetupError as e:
        print(f"::error::api-compat: {e}", file=sys.stderr)
        return 2
    except Exception as e:  # exit 1 means "breaks found"; a crash must not read as that
        print(f"::error::api-compat crashed: {type(e).__name__}: {e}", file=sys.stderr)
        return 2

    if not breaks:
        print(f"No breaking public API changes against {label}.")
        _summary([f"### API compatibility\n\nNo breaking public API changes against {label}."])
        return 0
    print(f"{len(breaks)} breaking public API change(s) against {label}:")
    for b in breaks:
        print(f"  - {b}")
    _summary([f"### API compatibility\n\n{len(breaks)} breaking public API change(s) against {label}:\n"]
             + [f"- {b}" for b in breaks])
    return 1


if __name__ == "__main__":
    sys.exit(main())
