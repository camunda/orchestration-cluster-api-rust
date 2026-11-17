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
2. A rustdoc-JSON signature diff — every public struct/variant field, function and
   method signature, type alias, const and static is rendered with canonical paths
   and compared. This covers the class cargo-semver-checks does not: a field, param
   or return type changing (`T` -> `Option<T>` is the nullability drift the SDKs have
   shipped before).

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
    # The extracted crate sits under this workspace's target/; make it its own root.
    text = manifest.read_text(encoding="utf-8")
    if not re.search(r"^\[workspace\]", text, re.M):
        manifest.write_text(text + "\n[workspace]\n", encoding="utf-8")
    return manifest


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


def run_semver_checks(manifest: Path, baseline_version: str | None, baseline_root: Path | None) -> list[str]:
    cmd = [
        "cargo", "semver-checks", "check-release",
        "--manifest-path", str(manifest), "--workspace",
        # The checked version is a patch/minor of the baseline by definition (same
        # major), so only changes that require a new major fail.
        "--release-type", "minor",
        "--color", "never",
    ]
    if baseline_root is not None:
        cmd += ["--baseline-root", str(baseline_root)]
    else:
        cmd += ["--baseline-version", str(baseline_version)]
    r = _run(cmd)
    output = r.stdout + r.stderr
    # Lint reports go to stdout, progress to stderr.
    breaks = parse_semver_checks(r.stdout)
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


def build_rustdoc_json(manifest: Path, package: str, target_dir: Path) -> Path:
    toolchain = docs_toolchain()
    cmd = [
        "cargo", f"+{toolchain}", "rustdoc", "--lib", "-p", package,
        "--manifest-path", str(manifest),
        "--", "-Z", "unstable-options", "--output-format", "json",
    ]
    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir))
    r = _run(cmd, env=env)
    if r.returncode != 0:
        raise SetupError(
            f"rustdoc JSON build failed for {package} ({manifest}):\n{r.stderr}\n"
            f"Install the toolchain with: rustup toolchain install {toolchain} --profile minimal"
        )
    out = target_dir / "doc" / f"{package.replace('-', '_')}.json"
    if not out.is_file():
        raise SetupError(f"rustdoc did not produce {out}")
    return out


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
    # Functions/methods: (signature without `const` and bounds, bound atoms, is_const),
    # so adding `const` or relaxing a bound is not reported.
    relax: tuple[str, frozenset[str], bool] | None = None


# Local types render as a placeholder holding their item id; `Surface.resolver` names
# them with a public path that exists on both sides, so a new re-export is not a change.
_LOCAL = re.compile("\x1f([^\x1f]*)\x1f")


class Surface:
    """Public API of one crate, keyed by public path."""

    def __init__(self, doc: dict):
        version = doc.get("format_version")
        if version not in SUPPORTED_FORMAT_VERSIONS:
            raise SetupError(
                f"unsupported rustdoc JSON format_version {version} "
                f"(supported: {sorted(SUPPORTED_FORMAT_VERSIONS)})"
            )
        self.index: dict[str, dict] = doc["index"]
        self.paths: dict[str, dict] = doc.get("paths", {})
        self.crates: dict[str, dict] = doc.get("external_crates", {})
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
            # Full defining path: `mpsc::Receiver` and `oneshot::Receiver` must differ.
            # Baselines build against this workspace's Cargo.lock, so it is stable.
            return "::".join(entry["path"])
        return rp.get("path", "_")

    def render(self, t: Any, g: dict[str, str]) -> str:
        if not isinstance(t, dict):
            return "_"
        if "primitive" in t:
            return t["primitive"]
        if "generic" in t:
            return g.get(t["generic"], t["generic"])
        if "resolved_path" in t:
            rp = t["resolved_path"]
            return self._type_path(rp) + self._args(rp.get("args"), g)
        if "borrowed_ref" in t:
            br = t["borrowed_ref"]
            lifetime = br.get("lifetime")
            lifetime = f"{g.get(lifetime, lifetime)} " if lifetime else ""
            return f"&{lifetime}{'mut ' if br.get('is_mutable') else ''}{self.render(br.get('type'), g)}"
        if "raw_pointer" in t:
            rp = t["raw_pointer"]
            return f"*{'mut' if rp.get('is_mutable') else 'const'} {self.render(rp.get('type'), g)}"
        if "tuple" in t:
            return "(" + ", ".join(self.render(x, g) for x in t["tuple"]) + ")"
        if "slice" in t:
            return f"[{self.render(t['slice'], g)}]"
        if "array" in t:
            a = t["array"]
            return f"[{self.render(a.get('type'), g)}; {a.get('len')}]"
        if "impl_trait" in t:
            return "impl " + self._bounds(t["impl_trait"], g)
        if "dyn_trait" in t:
            traits = [self._poly_trait(pt, g) for pt in t["dyn_trait"].get("traits", [])]
            return "dyn " + " + ".join(traits)
        if "qualified_path" in t:
            qp = t["qualified_path"]
            trait = qp.get("trait")
            as_trait = f" as {self._type_path(trait)}{self._args(trait.get('args'), g)}" if trait else ""
            return f"<{self.render(qp.get('self_type'), g)}{as_trait}>::{qp.get('name')}{self._args(qp.get('args'), g)}"
        if "function_pointer" in t:
            fp = t["function_pointer"]
            header = fp.get("header") or {}
            safety = "unsafe " if header.get("is_unsafe") else ""
            abi = header.get("abi", "Rust")
            convention = "" if abi == "Rust" else f'extern "{abi}" '
            fg, _ = self._generics({"params": fp.get("generic_params", [])}, g)
            return convention + safety + "fn" + self._sig(fp.get("sig", {}), fg)
        if "pat" in t:
            return self.render(t["pat"].get("type"), g)
        return "_"

    def _args(self, args: Any, g: dict[str, str]) -> str:
        if not isinstance(args, dict):
            return ""
        if "angle_bracketed" in args:
            ab = args["angle_bracketed"]
            parts: list[str] = []
            for a in ab.get("args", []):
                if isinstance(a, dict) and "type" in a:
                    parts.append(self.render(a["type"], g))
                elif isinstance(a, dict) and "lifetime" in a:
                    lifetime = a["lifetime"]
                    parts.append(g.get(lifetime, lifetime))
                elif isinstance(a, dict) and "const" in a:
                    parts.append(str(a["const"].get("expr", "")))
                elif a == "infer":
                    parts.append("_")
            for c in ab.get("constraints", []):
                binding = c.get("binding") or {}
                name = c.get("name", "") + self._args(c.get("args"), g)
                if "equality" in binding:
                    term = binding["equality"]
                    value = self.render(term["type"], g) if "type" in term else str(term.get("constant", {}).get("expr", ""))
                    parts.append(f"{name} = {value}")
                elif "constraint" in binding:
                    parts.append(f"{name}: {self._bounds(binding['constraint'], g)}")
            return f"<{', '.join(parts)}>" if parts else ""
        if "parenthesized" in args:
            p = args["parenthesized"]
            out = p.get("output")
            ret = f" -> {self.render(out, g)}" if out else ""
            return "(" + ", ".join(self.render(x, g) for x in p.get("inputs", [])) + ")" + ret
        if "return_type_notation" in args:
            return "(..)"
        return ""

    def _poly_trait(self, pt: dict, g: dict[str, str]) -> str:
        tr = pt.get("trait") or {}
        return self._type_path(tr) + self._args(tr.get("args"), g)

    def _bound_list(self, bounds: list, g: dict[str, str]) -> list[str]:
        out: list[str] = []
        for b in bounds or []:
            if isinstance(b, dict) and "trait_bound" in b:
                tb = b["trait_bound"]
                mod = {"maybe": "?", "maybe_const": "~const "}.get(tb.get("modifier"), "")
                out.append(mod + self._poly_trait(tb, g))
        # Lifetime bounds are elided throughout: they are not part of what this layer checks.
        return sorted(out)

    def _bounds(self, bounds: list, g: dict[str, str]) -> str:
        return " + ".join(self._bound_list(bounds, g))

    def _generics(self, generics: dict | None, g: dict[str, str]) -> tuple[dict[str, str], str]:
        g, params, where = self._generics_parts(generics, g)
        return g, params + where

    def _generics_parts(
        self, generics: dict | None, g: dict[str, str], atoms: set[str] | None = None
    ) -> tuple[dict[str, str], str, str]:
        """Extend `g` with positional names for this item's params (renaming `T` to `U`
        is not a break); render the param list and the where clause. With `atoms`, bounds
        and predicates go there one per bound instead of into the rendering."""
        g = dict(g)
        for p in (generics or {}).get("params", []):
            kind = p.get("kind") or {}
            if "lifetime" in kind:
                g[p["name"]] = _fresh_lifetime(g)
                continue
            if "type" in kind and kind["type"].get("is_synthetic"):
                continue  # `impl Trait` argument: rendered at the parameter instead
            g[p["name"]] = _fresh_generic(g)
        if atoms is not None:
            for p in (generics or {}).get("params", []):
                kind = p.get("kind") or {}
                if "lifetime" in kind:
                    lifetime = g[p["name"]]
                    atoms.update(
                        f"{lifetime}: {g.get(parent, parent)}"
                        for parent in kind["lifetime"].get("outlives", [])
                    )
        rendered = []
        for p in (generics or {}).get("params", []):
            kind = p.get("kind") or {}
            if p["name"] not in g or "lifetime" in kind:
                continue
            if "type" in kind and kind["type"].get("is_synthetic"):
                continue
            if "type" in kind:
                bounds = self._bounds(kind["type"].get("bounds"), g)
                default = kind["type"].get("default")
                if atoms is not None:
                    atoms.update(f"{g[p['name']]}: {b}" for b in self._bound_list(kind["type"].get("bounds"), g))
                    bounds = ""
                s = g[p["name"]] + (f": {bounds}" if bounds else "")
                if default is not None:
                    s += f" = {self.render(default, g)}"
                rendered.append(s)
            elif "const" in kind:
                rendered.append(f"const {g[p['name']]}: {self.render(kind['const'].get('type'), g)}")
        preds = []
        for wp in (generics or {}).get("where_predicates", []):
            if "bound_predicate" in wp:
                bp = wp["bound_predicate"]
                bounds = self._bounds(bp.get("bounds"), g)
                if bounds:
                    preds.append(f"{self.render(bp.get('type'), g)}: {bounds}")
            elif "eq_predicate" in wp:
                ep = wp["eq_predicate"]
                rhs = ep.get("rhs") or {}
                rhs_s = self.render(rhs["type"], g) if "type" in rhs else str(rhs)
                preds.append(f"{self.render(ep.get('lhs'), g)} == {rhs_s}")
        if atoms is not None:
            for wp in (generics or {}).get("where_predicates", []):
                if "bound_predicate" in wp:
                    bp = wp["bound_predicate"]
                    ty = self.render(bp.get("type"), g)
                    atoms.update(f"{ty}: {b}" for b in self._bound_list(bp.get("bounds"), g))
            atoms.update(p for p in preds if " == " in p)
            preds = []
        params = f"<{', '.join(rendered)}>" if rendered else ""
        where = " where " + ", ".join(sorted(preds)) if preds else ""
        return g, params, where

    def _sig(self, sig: dict, g: dict[str, str]) -> str:
        params = [self.render(ty, g) for _, ty in sig.get("inputs", [])]
        if sig.get("is_c_variadic"):
            params.append("...")
        out = sig.get("output")
        return "(" + ", ".join(params) + ")" + (f" -> {self.render(out, g)}" if out else "")

    def _fn(self, fn: dict, g: dict[str, str]) -> str:
        header = fn.get("header") or {}
        prefix = "".join(
            word for flag, word in (("is_const", "const "), ("is_async", "async "), ("is_unsafe", "unsafe "))
            if header.get(flag)
        )
        g2, params, where = self._generics_parts(fn.get("generics"), g)
        # Param names are not part of the contract; their types and order are.
        return f"{prefix}fn{params}{self._sig(fn.get('sig') or {}, g2)}{where}"

    def _fn_relax(self, fn: dict, g: dict[str, str]) -> tuple[str, frozenset[str], bool]:
        header = fn.get("header") or {}
        prefix = "".join(
            word for flag, word in (("is_async", "async "), ("is_unsafe", "unsafe ")) if header.get(flag)
        )
        atoms: set[str] = set()
        g, params, _ = self._generics_parts(fn.get("generics"), g, atoms)
        return f"{prefix}fn{params}{self._sig(fn.get('sig') or {}, g)}", frozenset(atoms), bool(header.get("is_const"))

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
            g, generics = self._generics(body.get("generics"), {})
            atoms: set[str] = set()
            _, relaxed_generics, _ = self._generics_parts(body.get("generics"), {}, atoms)
            self._add(
                path,
                Entry(
                    kind,
                    f"{kind}{generics}",
                    canonical=canonical,
                    relax=(f"{kind}{relaxed_generics}", frozenset(atoms), True),
                ),
            )
            if kind == "union":
                fields = body.get("fields", [])
            else:
                skind = body.get("kind")
                fields = []
                if isinstance(skind, dict) and "plain" in skind:
                    fields = skind["plain"].get("fields", [])
                elif isinstance(skind, dict) and "tuple" in skind:
                    fields = skind["tuple"]
            for pos, fid in enumerate(fields):
                f = self.index.get(str(fid)) if fid is not None else None
                if f is None or f.get("visibility") != "public":
                    continue
                name = f.get("name") or str(pos)
                self._member(item_id, path, ".", name, Entry("field", self.render(f["inner"]["struct_field"], g)))
            self._record_impls(item_id, path, body.get("impls", []), g)
        elif kind == "enum":
            g, generics = self._generics(body.get("generics"), {})
            atoms: set[str] = set()
            _, relaxed_generics, _ = self._generics_parts(body.get("generics"), {}, atoms)
            self._add(
                path,
                Entry(
                    "enum",
                    f"enum{generics}",
                    canonical=canonical,
                    relax=(f"enum{relaxed_generics}", frozenset(atoms), True),
                ),
            )
            for vid in body.get("variants", []):
                v = self.index.get(str(vid))
                if v is None:
                    continue
                self._member(item_id, path, "::", v["name"], Entry("variant", self._variant(v["inner"]["variant"], g)))
            self._record_impls(item_id, path, body.get("impls", []), g)
        elif kind == "trait":
            g, generics = self._generics(body.get("generics"), {"Self": "Self"})
            supertraits = self._bounds(body.get("bounds"), g)
            self._add(path, Entry("trait", f"trait{generics}" + (f": {supertraits}" if supertraits else ""), canonical=canonical))
            for mid in body.get("items", []):
                m = self.index.get(str(mid))
                if m is None or not m.get("name"):
                    continue
                minner = m.get("inner", {})
                if "function" in minner:
                    fn = minner["function"]
                    entry = Entry("trait method", self._fn(fn, g), required=not fn.get("has_body"))
                elif "assoc_type" in minner:
                    at = minner["assoc_type"]
                    bounds = self._bounds(at.get("bounds"), g)
                    entry = Entry("associated type", "type" + (f": {bounds}" if bounds else ""), required=at.get("type") is None)
                elif "assoc_const" in minner:
                    ac = minner["assoc_const"]
                    entry = Entry("associated const", f"const: {self.render(ac.get('type'), g)}", required=ac.get("value") is None)
                else:
                    continue
                self._member(item_id, path, "::", m["name"], entry)
        elif kind == "function":
            self._add(path, Entry("function", self._fn(body, {}), canonical=canonical, relax=self._fn_relax(body, {})))
        elif kind == "type_alias":
            g, generics = self._generics(body.get("generics"), {})
            atoms: set[str] = set()
            _, relaxed_generics, _ = self._generics_parts(body.get("generics"), {}, atoms)
            self._add(
                path,
                Entry(
                    "type alias",
                    f"type{generics} = {self.render(body.get('type'), g)}",
                    canonical=canonical,
                    relax=(
                        f"type{relaxed_generics} = {self.render(body.get('type'), g)}",
                        frozenset(atoms),
                        True,
                    ),
                ),
            )
        elif kind == "constant":
            self._add(path, Entry("const", f"const: {self.render(body.get('type'), {})}", canonical=canonical))
        elif kind == "static":
            mut = "mut " if body.get("is_mutable") else ""
            self._add(path, Entry("static", f"static {mut}{self.render(body.get('type'), {})}", canonical=canonical))
        elif kind in ("macro", "proc_macro", "module", "trait_alias"):
            self._add(path, Entry(kind, kind, canonical=canonical))

    def _variant(self, variant: dict, g: dict[str, str]) -> str:
        vkind = variant.get("kind")
        if not isinstance(vkind, dict):
            return "unit"
        if "tuple" in vkind:
            parts = []
            for fid in vkind["tuple"]:
                f = self.index.get(str(fid)) if fid is not None else None
                parts.append(self.render(f["inner"]["struct_field"], g) if f else "_")
            return f"({', '.join(parts)})"
        if "struct" in vkind:
            parts = []
            for fid in vkind["struct"].get("fields", []):
                f = self.index.get(str(fid))
                if f:
                    parts.append(f"{f['name']}: {self.render(f['inner']['struct_field'], g)}")
            return "{ " + ", ".join(sorted(parts)) + " }"
        return "unit"

    def _record_impls(self, owner_id: str, path: str, impl_ids: list, g: dict[str, str]) -> None:
        for impl_id in impl_ids:
            impl_item = self.index.get(str(impl_id))
            if impl_item is None:
                continue
            impl = impl_item["inner"].get("impl") or {}
            # Auto-trait and blanket impls follow from other items; those are checked instead.
            if impl.get("is_synthetic") or impl.get("blanket_impl"):
                continue
            if impl.get("trait") is not None:
                self._record_trait_impl(owner_id, path, impl, g)
                continue
            impl_atoms: set[str] = set()
            ig, _, _ = self._generics_parts(impl.get("generics"), g, impl_atoms)
            # `Self` in an inherent impl is the owner type; writing either spelling is the same API.
            ig["Self"] = self.render(impl.get("for"), ig)
            for mid in impl.get("items", []):
                m = self.index.get(str(mid))
                if m is None or m.get("visibility") != "public" or not m.get("name"):
                    continue
                minner = m.get("inner", {})
                if "function" in minner:
                    fn = minner["function"]
                    sig, atoms, const = self._fn_relax(fn, ig)
                    self._member(
                        owner_id,
                        path,
                        "::",
                        m["name"],
                        Entry(
                            "method",
                            self._fn(fn, ig),
                            relax=(sig, atoms | frozenset(impl_atoms), const),
                        ),
                    )
                elif "assoc_const" in minner:
                    self._member(owner_id, path, "::", m["name"], Entry("associated const", f"const: {self.render(minner['assoc_const'].get('type'), ig)}"))

    def _record_trait_impl(self, owner_id: str, path: str, impl: dict, g: dict[str, str]) -> None:
        """The impl and its associated types; method signatures are the trait's."""
        tr = impl["trait"]
        trait_id = str(tr.get("id")) if tr.get("id") is not None else None
        known = self.paths.get(trait_id) if trait_id else None
        if trait_id not in self.all_paths and (known is None or known.get("crate_id") == 0):
            return  # a private trait: downstream cannot name it
        atoms: set[str] = set()
        ig, params, where = self._generics_parts(impl.get("generics"), g)
        _, relaxed_params, _ = self._generics_parts(impl.get("generics"), g, atoms)
        neg = "!" if impl.get("is_negative") else ""
        name = f"<impl {neg}{self._type_path(tr)}{self._args(tr.get('args'), ig)} for {self.render(impl.get('for'), ig)}>"
        self._member(
            owner_id,
            path,
            "::",
            name,
            Entry(
                "trait impl",
                f"impl{params}{where}",
                relax=(f"impl{relaxed_params}", frozenset(atoms), True),
            ),
        )
        impl_key = f"{path}::{name}"
        for mid in impl.get("items", []):
            m = self.index.get(str(mid))
            at = (m or {}).get("inner", {}).get("assoc_type")
            if at is None or not m.get("name"):
                continue
            self._add(f"{impl_key}::{m['name']}", Entry(
                "associated type", f"type = {self.render(at.get('type'), ig)}",
                owner=impl_key, canonical=f"{self.public_path[owner_id]}::{name}::{m['name']}",
            ))


def _fresh_generic(g: dict[str, str]) -> str:
    # Not `len(g)`: a shadowing param (`impl<T>` on `Holder<T>`) replaces a key without growing g.
    used = [int(v[1:]) for v in g.values() if v[:1] == "G" and v[1:].isdigit()]
    return f"G{max(used, default=-1) + 1}"


def _fresh_lifetime(g: dict[str, str]) -> str:
    used = [int(v[2:]) for v in g.values() if v.startswith("'L") and v[2:].isdigit()]
    return f"'L{max(used, default=-1) + 1}"


def _became_optional(old: str, new: str) -> bool:
    return new in (f"core::option::Option<{old}>", f"core::option::Option<alloc::boxed::Box<{old}>>")


def _relaxed_only(before: Entry, after: Entry, ro, rn) -> bool:
    """Callers still compile: same shape, no new bound, no `?Sized` dropped, `const` kept."""
    if before.relax is None or after.relax is None:
        return False
    (b_sig, b_atoms, b_const), (a_sig, a_atoms, a_const) = before.relax, after.relax
    a_set, b_set = {rn(a) for a in a_atoms}, {ro(a) for a in b_atoms}
    return (
        ro(b_sig) == rn(a_sig)
        and {a for a in a_set if ": ?" not in a} <= b_set
        # Dropping `?Sized` restores the implicit `Sized` bound.
        and {a for a in b_set if ": ?" in a} <= a_set
        and (a_const or not b_const)
    )


def diff_surfaces(old: Surface, new: Surface) -> list[str]:
    """Breaking differences in `new` relative to `old`. Additions are compatible."""
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
        b_sig, a_sig = ro(before.sig), rn(after.sig)
        if (
            b_sig != a_sig or before.relax != after.relax
        ) and not _relaxed_only(before, after, ro, rn):
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


def load_surface(path: Path) -> Surface:
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        raise SetupError(f"cannot read rustdoc JSON {path}: {e}") from e
    return Surface(doc)


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------


def check(manifest: Path, baseline_version: str | None, baseline_root: Path | None,
          versions_json: Path | None, work_dir: Path) -> list[str]:
    check_semver_tool()
    packages = workspace_packages(manifest)
    breaks = [f"[cargo-semver-checks] {b}" for b in run_semver_checks(manifest, baseline_version, baseline_root)]

    if baseline_root is not None:
        baseline_manifests = workspace_packages(baseline_root / "Cargo.toml")
    else:
        src = work_dir / "baseline-src"
        baseline_manifests = {
            name: download_crate(name, str(baseline_version), src, versions_json if name == SDK_CRATE else None)
            for name in packages
        }
        # Same dependency versions on both sides, so dependency type paths compare exactly.
        lock = manifest.parent / "Cargo.lock"
        if lock.is_file():
            for baseline_manifest in baseline_manifests.values():
                shutil.copyfile(lock, baseline_manifest.parent / "Cargo.lock")
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
