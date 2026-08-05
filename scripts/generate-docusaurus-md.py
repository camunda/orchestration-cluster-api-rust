#!/usr/bin/env python3
"""
Convert rustdoc JSON metadata into Docusaurus-compatible markdown pages for the
Rust SDK documentation published at https://docs.camunda.io.

Usage:
    python3 scripts/generate-docusaurus-md.py [--validate-links] [--readme-only]

Input:
    README.md                                   – guide content (split by H2)
    target/doc/camunda_orchestration_sdk.json   – rustdoc JSON for the SDK crate
    target/doc/camunda_orchestration_api_client.json
                                                – rustdoc JSON for the generated
                                                  client crate (domain key types)
    examples/operation-map.json                 – operationId → example region map
    examples/*.rs                               – compilable examples with region tags

Output:
    docs-md/rust-sdk.md                 – landing page (sibling of section directory)
    docs-md/rust-sdk/*.md               – per-section guide pages (from README H2s)
    docs-md/rust-sdk/api-reference/     – API reference pages + _category_.json

The rustdoc JSON files are produced by `make docs-json`, which runs:

    cargo +<nightly> rustdoc -p <crate> --lib -- -Z unstable-options --output-format json

rustdoc JSON is an unstable format. This script pins the `format_version` values
it understands and fails loudly on anything else — see SUPPORTED_FORMAT_VERSIONS.
"""

from __future__ import annotations

import json
import re
import sys
import textwrap
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parent.parent
README_PATH = REPO_ROOT / "README.md"
EXAMPLES_DIR = REPO_ROOT / "examples"
OPERATION_MAP_PATH = EXAMPLES_DIR / "operation-map.json"
TARGET_DOC_DIR = REPO_ROOT / "target" / "doc"
SDK_JSON_PATH = TARGET_DOC_DIR / "camunda_orchestration_sdk.json"
CLIENT_JSON_PATH = TARGET_DOC_DIR / "camunda_orchestration_api_client.json"

DOCS_MD_DIR = REPO_ROOT / "docs-md"
SECTION_DIR = DOCS_MD_DIR / "rust-sdk"
OUTPUT_DIR = SECTION_DIR / "api-reference"

# rustdoc JSON format versions this script has been validated against.
# Bump deliberately after re-reading the rustdoc-types changelog.
SUPPORTED_FORMAT_VERSIONS = {61}

DOCS_RS_SDK = "https://docs.rs/camunda-orchestration-sdk/latest/camunda_orchestration_sdk"
DOCS_RS_CLIENT = (
    "https://docs.rs/camunda-orchestration-api-client/latest/camunda_orchestration_api_client"
)
GITHUB_BLOB = "https://github.com/camunda/orchestration-cluster-api-rust/blob/main"

# ---------------------------------------------------------------------------
# Frontmatter helpers
# ---------------------------------------------------------------------------

FRONTMATTER_TEMPLATE = textwrap.dedent("""\
    ---
    title: "{title}"
    sidebar_label: "{label}"
    mdx:
      format: md
    ---
""")


def frontmatter(title: str, label: str | None = None) -> str:
    return FRONTMATTER_TEMPLATE.format(
        title=_escape_yaml(title),
        label=_escape_yaml(label or title),
    )


def _escape_yaml(s: str) -> str:
    return s.replace('"', '\\"')


# ---------------------------------------------------------------------------
# Technical Preview banner (injected after the first H1 on every page)
# ---------------------------------------------------------------------------

TECH_PREVIEW_BANNER = (
    "\n:::caution Technical Preview\n"
    "The Rust SDK is a **technical preview**. Its API surface may still evolve and "
    "changes may not follow semantic versioning. Pin an exact version if you need "
    "stability.\n"
    ":::\n"
)


def inject_tech_preview_banner(content: str) -> str:
    """Insert the Technical Preview banner after the first H1 heading."""
    m = re.search(r"^#\s+.+$", content, re.MULTILINE)
    if m:
        pos = m.end()
        return content[:pos] + "\n" + TECH_PREVIEW_BANNER + content[pos:]
    return content


# ---------------------------------------------------------------------------
# Link rewriting
# ---------------------------------------------------------------------------

# Depth for the landing page: apis-tools/rust-sdk.md
_LANDING_PAGE_DEPTH = 1
# Depth for section pages: apis-tools/rust-sdk/<slug>.md
_SECTION_PAGE_DEPTH = 2
# Depth for api-reference pages: apis-tools/rust-sdk/api-reference/<slug>.md
_API_REFERENCE_DEPTH = 3
# sidebar_position for the API Reference category (always last)
_API_REFERENCE_POSITION = 100

_URL_PATH_OVERRIDES: dict[str, str] = {
    "camunda-api-rest": "orchestration-cluster-api-rest",
}

_DOCS_LINK_RE = re.compile(r"\[([^\]]*)\]\(https?://docs\.camunda\.io/docs/(?:next/)?(.*?)\)")


def _rewrite_docs_links(content: str, depth: int) -> str:
    """Rewrite absolute docs.camunda.io links to site-relative links."""
    prefix = "../" * depth

    def _replace(m: re.Match) -> str:  # type: ignore[type-arg]
        text = m.group(1)
        url_path = m.group(2).rstrip("/")
        for old, new in _URL_PATH_OVERRIDES.items():
            url_path = url_path.replace(old, new)
        return f"[{text}]({prefix}{url_path}.md)"

    return _DOCS_LINK_RE.sub(_replace, content)


def rewrite_camunda_docs_links(content: str) -> str:
    return _rewrite_docs_links(content, _API_REFERENCE_DEPTH)


# Repo-relative markdown links (e.g. `[LICENSE](LICENSE)`) are meaningless once
# the page is copied into camunda-docs. Point them at GitHub instead.
_REPO_LINK_RE = re.compile(r"\[([^\]]+)\]\((?!https?://|#|mailto:|\.\./)([^)\s]+)\)")


def rewrite_repo_links(content: str) -> str:
    def _replace(m: re.Match) -> str:  # type: ignore[type-arg]
        text, target = m.group(1), m.group(2)
        if target.endswith(".md") and "/" not in target:
            # Sibling generated page — leave alone.
            return m.group(0)
        return f"[{text}]({GITHUB_BLOB}/{target.lstrip('./')})"

    return _REPO_LINK_RE.sub(_replace, content)


# ---------------------------------------------------------------------------
# rustdoc JSON loading
# ---------------------------------------------------------------------------


class RustdocFormatError(RuntimeError):
    pass


@dataclass
class Crate:
    """A parsed rustdoc JSON document."""

    name: str
    version: str | None
    index: dict[str, dict]
    paths: dict[str, dict]
    root: str

    def get(self, item_id: Any) -> dict | None:
        return self.index.get(str(item_id))

    def path_of(self, item_id: Any) -> list[str]:
        entry = self.paths.get(str(item_id))
        return entry.get("path", []) if entry else []


def load_rustdoc_json(path: Path) -> Crate:
    if not path.is_file():
        raise FileNotFoundError(
            f"rustdoc JSON not found: {path}\n"
            f"Run `make docs-json` first (requires a nightly toolchain)."
        )
    data = json.loads(path.read_text(encoding="utf-8"))
    version = data.get("format_version")
    if version not in SUPPORTED_FORMAT_VERSIONS:
        raise RustdocFormatError(
            f"{path.name}: unsupported rustdoc JSON format_version {version} "
            f"(this script supports {sorted(SUPPORTED_FORMAT_VERSIONS)}).\n"
            f"rustdoc JSON is an unstable format. Either pin the nightly toolchain "
            f"in the Makefile (DOCS_TOOLCHAIN) back to a supported one, or review the "
            f"rustdoc-types changelog and add {version} to SUPPORTED_FORMAT_VERSIONS "
            f"after updating this script."
        )
    root = str(data["root"])
    index = data["index"]
    root_item = index[root]
    return Crate(
        name=root_item.get("name") or path.stem,
        version=data.get("crate_version"),
        index=index,
        paths=data.get("paths", {}),
        root=root,
    )


# ---------------------------------------------------------------------------
# Rust type rendering
# ---------------------------------------------------------------------------


def render_type(t: Any) -> str:
    """Render a rustdoc JSON `Type` node back into readable Rust source."""
    if t is None:
        return "_"
    if isinstance(t, str):
        return t
    if not isinstance(t, dict):
        return str(t)

    if "primitive" in t:
        return t["primitive"]
    if "generic" in t:
        return t["generic"]
    if "resolved_path" in t:
        rp = t["resolved_path"]
        return _short_path(rp.get("path", "")) + _render_generic_args(rp.get("args"))
    if "borrowed_ref" in t:
        br = t["borrowed_ref"]
        lt = f"{br['lifetime']} " if br.get("lifetime") else ""
        mut = "mut " if br.get("is_mutable") else ""
        return f"&{lt}{mut}{render_type(br.get('type'))}"
    if "raw_pointer" in t:
        rp = t["raw_pointer"]
        mut = "mut" if rp.get("is_mutable") else "const"
        return f"*{mut} {render_type(rp.get('type'))}"
    if "tuple" in t:
        inner = ", ".join(render_type(x) for x in t["tuple"])
        return f"({inner})"
    if "slice" in t:
        return f"[{render_type(t['slice'])}]"
    if "array" in t:
        a = t["array"]
        return f"[{render_type(a.get('type'))}; {a.get('len')}]"
    if "impl_trait" in t:
        return "impl " + _render_bounds(t["impl_trait"])
    if "dyn_trait" in t:
        dt = t["dyn_trait"]
        traits = " + ".join(
            _short_path(tr["trait"].get("path", "")) + _render_generic_args(tr["trait"].get("args"))
            for tr in dt.get("traits", [])
        )
        if dt.get("lifetime"):
            traits = f"{traits} + {dt['lifetime']}"
        return f"dyn {traits}"
    if "qualified_path" in t:
        qp = t["qualified_path"]
        return f"<{render_type(qp.get('self_type'))}>::{qp.get('name')}"
    if "function_pointer" in t:
        sig = t["function_pointer"].get("sig", {})
        args = ", ".join(render_type(ty) for _, ty in sig.get("inputs", []))
        out = sig.get("output")
        ret = f" -> {render_type(out)}" if out else ""
        return f"fn({args}){ret}"
    if "infer" in t:
        return "_"
    return "_"


def _short_path(path: str) -> str:
    """Trim fully-qualified paths to their last one or two segments."""
    if not path:
        return "_"
    parts = path.split("::")
    if len(parts) >= 2 and parts[-2] in ("models", "apis", "client"):
        return "::".join(parts[-2:])
    return parts[-1]


def _render_generic_args(args: Any) -> str:
    if not args or not isinstance(args, dict):
        return ""
    if "angle_bracketed" in args:
        ab = args["angle_bracketed"]
        rendered: list[str] = []
        for a in ab.get("args", []):
            if not isinstance(a, dict):
                continue
            if "type" in a:
                rendered.append(render_type(a["type"]))
            elif "lifetime" in a:
                rendered.append(a["lifetime"])
            elif "const" in a:
                rendered.append(str(a["const"].get("expr", "")))
        for c in ab.get("constraints", []):
            rendered.append(c.get("name", ""))
        return f"<{', '.join(x for x in rendered if x)}>" if rendered else ""
    if "parenthesized" in args:
        p = args["parenthesized"]
        inputs = ", ".join(render_type(x) for x in p.get("inputs", []))
        out = p.get("output")
        ret = f" -> {render_type(out)}" if out else ""
        return f"({inputs}){ret}"
    return ""


def _render_bounds(bounds: list) -> str:
    out: list[str] = []
    for b in bounds or []:
        if not isinstance(b, dict):
            continue
        if "trait_bound" in b:
            tb = b["trait_bound"]["trait"]
            out.append(_short_path(tb.get("path", "")) + _render_generic_args(tb.get("args")))
        elif "outlives" in b:
            out.append(b["outlives"])
    return " + ".join(out) if out else "Sized"


def _render_generics(generics: dict | None) -> str:
    """Render the `<...>` parameter list of a generic item (lifetimes elided)."""
    if not generics:
        return ""
    params = [
        p.get("name")
        for p in generics.get("params", [])
        if isinstance(p, dict)
        and p.get("name")
        and "lifetime" not in (p.get("kind") or {})
        and not str(p.get("name", "")).startswith("impl ")
    ]
    return f"<{', '.join(params)}>" if params else ""


def render_fn_signature(name: str, fn: dict) -> str:
    """Render a full `fn` signature from a rustdoc JSON function item."""
    header = fn.get("header") or {}
    sig = fn.get("sig") or {}
    prefix = "pub "
    if header.get("is_const"):
        prefix += "const "
    if header.get("is_async"):
        prefix += "async "
    if header.get("is_unsafe"):
        prefix += "unsafe "

    params: list[str] = []
    for pname, ptype in sig.get("inputs", []):
        if pname == "self":
            params.append(_render_self(ptype))
        else:
            params.append(f"{pname}: {render_type(ptype)}")
    if sig.get("is_c_variadic"):
        params.append("...")

    out = sig.get("output")
    ret = f" -> {render_type(out)}" if out else ""
    return f"{prefix}fn {name}{_render_generics(fn.get('generics'))}({', '.join(params)}){ret}"


def _render_self(ptype: Any) -> str:
    if isinstance(ptype, dict) and "borrowed_ref" in ptype:
        br = ptype["borrowed_ref"]
        return "&mut self" if br.get("is_mutable") else "&self"
    return "self"


# ---------------------------------------------------------------------------
# Item model
# ---------------------------------------------------------------------------


@dataclass
class Method:
    name: str
    signature: str
    docs: str
    is_async: bool


@dataclass
class Member:
    name: str
    type_str: str
    docs: str


@dataclass
class TypeItem:
    name: str
    kind: str  # struct | enum | type_alias | trait
    docs: str
    module_path: list[str]
    crate_name: str
    fields: list[Member] = field(default_factory=list)
    variants: list[Member] = field(default_factory=list)
    methods: list[Method] = field(default_factory=list)
    alias_target: str = ""

    @property
    def summary(self) -> str:
        return _first_line(self.docs)


def _first_line(docs: str) -> str:
    if not docs:
        return ""
    for para in docs.split("\n\n"):
        text = " ".join(line.strip() for line in para.strip().splitlines() if line.strip())
        if text and not text.startswith("```"):
            return text
    return ""


def _docs(item: dict) -> str:
    return _strip_intra_doc_links((item.get("docs") or "").strip())


# rustdoc intra-doc links (`[`Foo::bar`]`, `[text](Foo::bar)`) have no meaning
# outside rustdoc's own HTML output, so reduce them to plain inline code.
_INTRA_DOC_SHORTCUT_RE = re.compile(r"\[(`[^`\]]+`)\](?!\(|\[|:)")
_INTRA_DOC_INLINE_RE = re.compile(r"\[([^\]]+)\]\((?!https?://|#|\.\./)[A-Za-z_][\w:]*(?:\(\))?\)")
_INTRA_DOC_REFDEF_RE = re.compile(r"^\[[^\]]+\]:\s*[A-Za-z_][\w:]*\s*$", re.MULTILINE)


def _strip_intra_doc_links(docs: str) -> str:
    if not docs:
        return ""
    docs = _INTRA_DOC_REFDEF_RE.sub("", docs)
    docs = _INTRA_DOC_INLINE_RE.sub(r"\1", docs)
    docs = _INTRA_DOC_SHORTCUT_RE.sub(r"\1", docs)
    return docs.strip()


def _is_public(item: dict) -> bool:
    """True only for items rustdoc explicitly marks `pub`.

    rustdoc records `"default"` for items written without a visibility keyword,
    which for structs, enums, type aliases and inherent-impl methods means
    *private*. Those only reach `crate.index` when rustdoc is invoked with
    `--document-private-items`, so this guard keeps that flag (or a future
    rustdoc default change) from leaking internals into the published reference.
    """
    return item.get("visibility") == "public"


def collect_types(crate: Crate) -> dict[str, TypeItem]:
    """Extract every public struct / enum / type alias declared in this crate."""
    types: dict[str, TypeItem] = {}
    for item_id, item in crate.index.items():
        inner = item.get("inner")
        if not isinstance(inner, dict):
            continue
        kind = next(iter(inner), "")
        if kind not in ("struct", "enum", "type_alias"):
            continue
        if not _is_public(item):
            continue
        name = item.get("name")
        if not name:
            continue
        path = crate.path_of(item_id)
        ti = TypeItem(
            name=name,
            kind=kind,
            docs=_docs(item),
            module_path=path[:-1] if path else [],
            crate_name=crate.name,
        )
        body = inner[kind]
        if kind == "struct":
            ti.fields = _collect_fields(crate, body)
            ti.methods = _collect_methods(crate, body.get("impls", []))
        elif kind == "enum":
            ti.variants = _collect_variants(crate, body.get("variants", []))
            ti.methods = _collect_methods(crate, body.get("impls", []))
        elif kind == "type_alias":
            ti.alias_target = render_type(body.get("type"))
        types[name] = ti
    return types


def _collect_fields(crate: Crate, struct_body: dict) -> list[Member]:
    kind = struct_body.get("kind")
    if not isinstance(kind, dict) or "plain" not in kind:
        return []
    out: list[Member] = []
    for fid in kind["plain"].get("fields", []):
        f = crate.get(fid)
        if not f or not _is_public(f):
            continue
        inner = f.get("inner", {})
        out.append(
            Member(
                name=f.get("name", ""),
                type_str=render_type(inner.get("struct_field")),
                docs=_docs(f),
            )
        )
    return out


def _collect_variants(crate: Crate, variant_ids: list) -> list[Member]:
    out: list[Member] = []
    for vid in variant_ids:
        v = crate.get(vid)
        if not v:
            continue
        inner = v.get("inner", {}).get("variant", {})
        out.append(
            Member(
                name=v.get("name", ""),
                type_str=_render_variant_payload(crate, inner),
                docs=_docs(v),
            )
        )
    return out


def _render_variant_payload(crate: Crate, variant: dict) -> str:
    kind = variant.get("kind")
    if kind == "plain" or not isinstance(kind, dict):
        return ""
    if "tuple" in kind:
        parts = []
        for fid in kind["tuple"]:
            f = crate.get(fid) if fid is not None else None
            parts.append(render_type(f.get("inner", {}).get("struct_field")) if f else "_")
        return f"({', '.join(parts)})"
    if "struct" in kind:
        parts = []
        for fid in kind["struct"].get("fields", []):
            f = crate.get(fid)
            if not f:
                continue
            parts.append(
                f"{f.get('name')}: {render_type(f.get('inner', {}).get('struct_field'))}"
            )
        return "{ " + ", ".join(parts) + " }"
    return ""


def _collect_methods(crate: Crate, impl_ids: list) -> list[Method]:
    """Collect public methods from inherent (non-trait) impls."""
    out: list[Method] = []
    seen: set[str] = set()
    for impl_id in impl_ids:
        impl_item = crate.get(impl_id)
        if not impl_item:
            continue
        impl = impl_item.get("inner", {}).get("impl")
        if not impl or impl.get("trait") is not None:
            continue  # trait impls are noise in a user-facing reference
        for mid in impl.get("items", []):
            m = crate.get(mid)
            if not m:
                continue
            fn = m.get("inner", {}).get("function")
            if not fn:
                continue
            if not _is_public(m):
                continue
            name = m.get("name", "")
            if not name or name.startswith("_") or name in seen:
                continue
            seen.add(name)
            out.append(
                Method(
                    name=name,
                    signature=render_fn_signature(name, fn),
                    docs=_docs(m),
                    is_async=bool((fn.get("header") or {}).get("is_async")),
                )
            )
    out.sort(key=lambda x: x.name)
    return out


# ---------------------------------------------------------------------------
# Example inlining (operation-map.json + `// region` tags)
# ---------------------------------------------------------------------------

_REGION_RE_TEMPLATE = r"^[ \t]*//\s*region\s+{name}\s*$(.*?)^[ \t]*//\s*endregion\s+{name}\s*$"


# Rust method name → operationId, for the handful of ids whose casing does not
# round-trip through snake_case (initialisms, and curated method renames).
_METHOD_OPERATION_ID = {
    "topology": "getTopology",
    "get_process_definition_xml": "getProcessDefinitionXML",
    "get_decision_definition_xml": "getDecisionDefinitionXML",
    "get_decision_requirements_xml": "getDecisionRequirementsXML",
}


def _snake_to_camel(name: str) -> str:
    if name in _METHOD_OPERATION_ID:
        return _METHOD_OPERATION_ID[name]
    head, *tail = name.split("_")
    return head + "".join(p.title() for p in tail)


def load_examples() -> dict[str, str]:
    """Map operationId → rendered example code block (first mapped region wins)."""
    if not OPERATION_MAP_PATH.is_file():
        print(f"  (no {OPERATION_MAP_PATH.name}; skipping example inlining)")
        return {}
    op_map = json.loads(OPERATION_MAP_PATH.read_text(encoding="utf-8"))
    out: dict[str, str] = {}
    cache: dict[Path, str] = {}
    for operation_id, entries in op_map.items():
        if not entries:
            continue
        entry = entries[0]
        src = EXAMPLES_DIR / entry["file"]
        if src not in cache:
            if not src.is_file():
                print(f"  WARNING: operation-map references missing file {entry['file']}")
                cache[src] = ""
            else:
                cache[src] = src.read_text(encoding="utf-8")
        body = cache[src]
        if not body:
            continue
        pattern = _REGION_RE_TEMPLATE.format(name=re.escape(entry["region"]))
        m = re.search(pattern, body, re.MULTILINE | re.DOTALL)
        if not m:
            print(f"  WARNING: region '{entry['region']}' not found in {entry['file']}")
            continue
        out[operation_id] = textwrap.dedent(m.group(1)).strip("\n")
    return out


# ---------------------------------------------------------------------------
# Markdown helpers
# ---------------------------------------------------------------------------


def _md_escape_cell(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ").strip()


def _code(text: str) -> str:
    return f"`{_md_escape_cell(text)}`" if text else ""


def _md_signature(sig: str) -> str:
    return f"```rust\n{sig}\n```\n\n"


def _md_table(headers: list[str], rows: list[list[str]]) -> str:
    if not rows:
        return ""
    out = "| " + " | ".join(headers) + " |\n"
    out += "| " + " | ".join("---" for _ in headers) + " |\n"
    for row in rows:
        out += "| " + " | ".join(row) + " |\n"
    return out + "\n"


def _md_fields_table(fields: list[Member]) -> str:
    return _md_table(
        ["Field", "Type", "Description"],
        [[_code(f.name), _code(f.type_str), _md_escape_cell(_first_line(f.docs))] for f in fields],
    )


def _md_variants_table(variants: list[Member]) -> str:
    return _md_table(
        ["Variant", "Payload", "Description"],
        [
            [_code(v.name), _code(v.type_str) if v.type_str else "—", _md_escape_cell(_first_line(v.docs))]
            for v in variants
        ],
    )


_MD_HEADING_RE = re.compile(r"^(#{1,6}) (.+)$")
# rustdoc fences default to Rust and often carry doctest attributes
# (```no_run, ```rust,ignore, ```should_panic, ...).
_RUST_FENCE_INFO_RE = re.compile(r"^```(?:rust)?[,\w]*$")


def _normalize_docs(docs: str, depth: int, heading_base: int = 0) -> str:
    """Prepare rustdoc prose for Docusaurus markdown.

    A leading `#` only means "hide this line from the rendered doctest" *inside*
    a code fence. Outside one it is an ordinary Markdown heading and must be
    kept -- shifted down by ``heading_base`` so it nests under the section that
    hosts the prose rather than colliding with it.
    """
    if not docs:
        return ""
    docs = _rewrite_docs_links(docs, depth)
    out: list[str] = []
    in_fence = False
    for line in docs.split("\n"):
        if line.startswith("```"):
            # Only the opening fence carries an info string; a closing fence
            # with one is not a valid closer and swallows the rest of the page.
            if not in_fence:
                line = _RUST_FENCE_INFO_RE.sub("```rust", line)
            in_fence = not in_fence
            out.append(line)
            continue
        if in_fence:
            if line == "#" or line.startswith("# "):
                continue  # hidden doctest line
            out.append(line)
            continue
        heading = _MD_HEADING_RE.match(line)
        if heading and heading_base:
            level = min(len(heading.group(1)) + heading_base, 6)
            line = f"{'#' * level} {heading.group(2)}"
        out.append(line)
    return "\n".join(out).strip()


def _render_type_section(t: TypeItem, level: int, examples: dict[str, str]) -> str:
    h = "#" * level
    out = f"\n{h} {t.name}\n\n"
    body = _normalize_docs(t.docs, _API_REFERENCE_DEPTH, level)
    if body:
        out += body + "\n\n"
    if t.kind == "type_alias":
        out += _md_signature(f"pub type {t.name} = {t.alias_target};")
    if t.fields:
        out += f"{h}# Fields\n\n" + _md_fields_table(t.fields)
    if t.variants:
        out += f"{h}# Variants\n\n" + _md_variants_table(t.variants)
    if t.methods:
        out += f"{h}# Methods\n\n"
        out += _md_table(
            ["Method", "Description"],
            [[_code(m.name), _md_escape_cell(_first_line(m.docs))] for m in t.methods],
        )
    return out


def _render_methods_detail(
    methods: list[Method], level: int, examples: dict[str, str]
) -> str:
    h = "#" * level
    out = ""
    for m in methods:
        out += f"\n{h} {m.name}\n\n"
        out += _md_signature(m.signature)
        body = _normalize_docs(m.docs, _API_REFERENCE_DEPTH, level)
        if body:
            out += body + "\n\n"
        example = examples.get(_snake_to_camel(m.name))
        if example:
            out += f"**Example**\n\n```rust\n{example}\n```\n\n"
    return out


# ---------------------------------------------------------------------------
# API reference page generators
# ---------------------------------------------------------------------------

# Bucket definition: (slug, title, sidebar_position, member type names)
CLIENT_TYPES = ["CamundaClient", "CamundaOptions"]
CONFIGURATION_TYPES = [
    "CamundaConfig",
    "Authentication",
    "AuthStrategy",
    "TlsConfig",
    "RetryConfig",
    "WorkerDefaults",
    "ConsistencyOptions",
    "LogLevel",
]
WORKER_TYPES = [
    "JobWorker",
    "JobWorkerConfig",
    "JobWorkerHandle",
    "Job",
    "JobAction",
    "JobHandler",
    "ReadyCallback",
]
RUNTIME_TYPES = [
    "CamundaError",
    "Result",
    "BackpressureManager",
    "BackpressureProfile",
    "BackpressureSeverity",
    "BackpressureState",
]


def classify_types(types: dict[str, TypeItem]) -> dict[str, list[TypeItem]]:
    """Assign every SDK type to exactly one API reference page.

    Fails loudly on an unclassified type so that a new public type cannot be
    silently dropped from the published reference.
    """
    buckets: dict[str, list[str]] = {
        "camunda-client": CLIENT_TYPES,
        "configuration": CONFIGURATION_TYPES,
        "job-workers": WORKER_TYPES,
        "runtime": RUNTIME_TYPES,
    }
    assigned: dict[str, list[TypeItem]] = {}
    claimed: set[str] = set()
    for slug, names in buckets.items():
        items = []
        for n in names:
            t = types.get(n)
            if t is None:
                print(f"  WARNING: expected type '{n}' not found in rustdoc JSON")
                continue
            items.append(t)
            claimed.add(n)
        assigned[slug] = items

    unclassified = sorted(set(types) - claimed)
    if unclassified:
        raise SystemExit(
            "Unclassified public types found in the SDK crate:\n"
            + "\n".join(f"  - {n}" for n in unclassified)
            + "\n\nAdd each one to a bucket in classify_types() so it appears in the "
            "published API reference (or confirm it should not be `pub`)."
        )
    return assigned


def generate_camunda_client(types: list[TypeItem], examples: dict[str, str]) -> str:
    client = next((t for t in types if t.name == "CamundaClient"), None)
    out = frontmatter("CamundaClient", "CamundaClient")
    out += "\n# CamundaClient\n\n"
    if client:
        body = _normalize_docs(client.docs, _API_REFERENCE_DEPTH, 1)
        if body:
            out += body + "\n\n"
        out += (
            f"`CamundaClient` exposes **{len(client.methods)}** methods covering the full "
            "Orchestration Cluster REST API surface, with authentication, retries, and "
            "backpressure applied automatically.\n\n"
        )
        out += "## Methods\n\n"
        out += _md_table(
            ["Method", "Description"],
            [
                [f"[`{m.name}`](#{_slugify(m.name)})", _md_escape_cell(_first_line(m.docs))]
                for m in client.methods
            ],
        )
        out += "## Method details\n"
        out += _render_methods_detail(client.methods, 3, examples)

    others = [t for t in types if t.name != "CamundaClient"]
    for t in others:
        out += _render_type_section(t, 2, examples)
    return out


def generate_section_page(title: str, intro: str, types: list[TypeItem], examples: dict[str, str]) -> str:
    out = frontmatter(title, title)
    out += f"\n# {title}\n\n"
    if intro:
        out += intro + "\n\n"
    for t in sorted(types, key=lambda t: t.name):
        out += _render_type_section(t, 2, examples)
    return out


def generate_domain_keys(keys: list[TypeItem]) -> str:
    out = frontmatter("Domain keys", "Domain keys")
    out += "\n# Domain keys\n\n"
    out += (
        "The Camunda Domain Type System replaces the bare `String` identifiers emitted by "
        "the OpenAPI generator with validated newtypes. Passing a `ProcessInstanceKey` where "
        "a `JobKey` is expected is a compile error, so whole classes of identifier mix-ups "
        "are caught before the request is sent.\n\n"
    )

    # Derive the shared constructor surface from a representative key rather than
    # hardcoding it, so the page cannot drift from the generated code.
    exemplar = max(keys, key=lambda k: len(k.methods), default=None)
    if exemplar and exemplar.methods:
        out += "Every key type exposes the same methods:\n\n"
        out += _md_table(
            ["Method", "Description"],
            [
                [_code(m.signature.removeprefix("pub fn ").removeprefix("pub ")),
                 _md_escape_cell(_first_line(m.docs))]
                for m in exemplar.methods
            ],
        )

    out += "## Key types\n\n"
    out += _md_table(
        ["Key type", "Description"],
        [
            [_code(k.name), _md_escape_cell(k.summary) or _fallback_key_summary(k.name)]
            for k in sorted(keys, key=lambda k: k.name)
        ],
    )
    out += (
        "Each key also has a matching `<Key>ExactMatch` wrapper used by the search filter "
        "models to express an exact-value match.\n"
    )
    return out


def _fallback_key_summary(name: str) -> str:
    """Description used when the generated key type carries no doc comment."""
    if not name.endswith("Key"):
        return "Validated identifier newtype."
    words = re.findall(r"[A-Z][a-z0-9]*|[a-z0-9]+", name.removesuffix("Key"))
    if not words:
        return "Validated identifier newtype."
    subject = " ".join(w.lower() for w in words)
    article = "an" if subject[0] in "aeiou" else "a"
    return f"Identifier for {article} {subject}."


def generate_index(counts: dict[str, int]) -> str:
    out = frontmatter("API reference", "Overview")
    out += "\n# API reference\n\n"
    out += (
        "This reference covers the hand-written ergonomic surface of the Rust SDK: the "
        "client, its configuration, the job worker, and the error and backpressure types.\n\n"
    )
    out += _md_table(
        ["Page", "Contents"],
        [
            ["[CamundaClient](camunda-client.md)", f"The client and its {counts.get('methods', 0)} API methods."],
            ["[Configuration](configuration.md)", "Client configuration, authentication, TLS, and retry policy."],
            ["[Job workers](job-workers.md)", "Job worker configuration, handlers, and lifecycle."],
            ["[Runtime](runtime.md)", "Error types and adaptive backpressure."],
            ["[Domain keys](domain-keys.md)", f"{counts.get('keys', 0)} validated identifier newtypes."],
        ],
    )
    out += (
        "The generated request and response models are not reproduced here — there are "
        f"several hundred of them. Browse them on "
        f"[docs.rs]({DOCS_RS_CLIENT}/models/index.html), or use your editor's "
        "go-to-definition on any method signature.\n"
    )
    return out


# ---------------------------------------------------------------------------
# Landing page + section page generator (from README.md)
# ---------------------------------------------------------------------------


def _strip_cut_sections(content: str) -> str:
    return re.sub(
        r"<!-- docs:cut:start -->.*?<!-- docs:cut:end -->\n?",
        "",
        content,
        flags=re.DOTALL,
    )


def _strip_snippet_markers(content: str) -> str:
    """Remove the `<!-- snippet-source: ... -->` provenance comments."""
    return re.sub(r"^<!-- snippet-source:.*?-->\n", "", content, flags=re.MULTILINE)


def _strip_contributing(content: str) -> str:
    return re.sub(r"\n## Contributing\b.*", "", content, flags=re.DOTALL)


def _clean_empty_lines(content: str) -> str:
    return re.sub(r"\n{4,}", "\n\n\n", content)


def _slugify(title: str) -> str:
    # Underscores are preserved to stay compatible with github-slugger, which
    # Docusaurus uses to derive heading anchors. Stripping them would make
    # links to snake_case method headings (e.g. `activate_jobs`) resolve to a
    # non-existent anchor.
    slug = title.lower()
    slug = re.sub(r"[^a-z0-9\s_-]", "", slug)
    slug = re.sub(r"\s+", "-", slug.strip())
    slug = re.sub(r"-+", "-", slug)
    return slug


def _build_anchor_map(sections: list[tuple[str, str]]) -> dict[str, str]:
    anchor_to_page: dict[str, str] = {}
    for title, body in sections:
        page_slug = _slugify(title)
        for m in re.finditer(r"^#{2,6}\s+(.+)$", body, re.MULTILINE):
            anchor_to_page[_slugify(m.group(1).strip())] = page_slug
    return anchor_to_page


def _rewrite_internal_anchors(
    content: str, current_slug: str, anchor_map: dict[str, str]
) -> str:
    def _replace(m: re.Match) -> str:  # type: ignore[type-arg]
        text, anchor = m.group(1), m.group(2)
        target_page = anchor_map.get(anchor)
        if target_page and target_page != current_slug:
            return f"[{text}]({target_page}.md#{anchor})"
        return m.group(0)

    return re.sub(r"\[([^\]]+)\]\(#([^)]+)\)", _replace, content)


def _promote_headings(content: str) -> str:
    def _promote(m: re.Match) -> str:  # type: ignore[type-arg]
        hashes, rest = m.group(1), m.group(2)
        if len(hashes) > 1:
            return f"{'#' * (len(hashes) - 1)} {rest}"
        return m.group(0)

    return re.sub(r"^(#{1,6}) (.+)$", _promote, content, flags=re.MULTILINE)


def _split_by_h2(content: str) -> tuple[str, list[tuple[str, str]]]:
    parts = re.split(r"(?=^## )", content, flags=re.MULTILINE)
    preamble = parts[0]
    sections: list[tuple[str, str]] = []
    for part in parts[1:]:
        h2_match = re.match(r"^## (.+)\n", part)
        if h2_match:
            sections.append((h2_match.group(1).strip(), part))
    return preamble, sections


def _make_section_frontmatter(doc_id: str, title: str, sidebar_position: int) -> str:
    return (
        f"---\n"
        f"id: {doc_id}\n"
        f'title: "{_escape_yaml(title)}"\n'
        f'sidebar_label: "{_escape_yaml(title)}"\n'
        f"sidebar_position: {sidebar_position}\n"
        f"mdx:\n"
        f"  format: md\n"
        f"---\n\n"
    )


LANDING_FRONTMATTER = textwrap.dedent("""\
    ---
    id: rust-sdk
    title: "Rust SDK (Technical Preview)"
    sidebar_label: "Rust SDK (Technical Preview)"
    sidebar_position: 1
    mdx:
      format: md
    ---

""")


def generate_readme_pages(readme_path: Path, output_dir: Path) -> None:
    content = readme_path.read_text(encoding="utf-8")
    content = _strip_cut_sections(content)
    content = _strip_contributing(content)
    content = _strip_snippet_markers(content)
    content = _clean_empty_lines(content)

    content = re.sub(
        r"^#\s+.*$",
        "# Rust SDK (Technical Preview)",
        content,
        count=1,
        flags=re.MULTILINE,
    )
    # Badge images point at shields.io / crates.io and add no value in the docs site.
    content = re.sub(r"^\[!\[.*?\)$\n?", "", content, flags=re.MULTILINE)
    content = content.strip() + "\n"

    preamble, sections = _split_by_h2(content)
    anchor_map = _build_anchor_map(sections)

    # --- Landing page: sibling of the section directory ---
    landing = _rewrite_docs_links(preamble, depth=_LANDING_PAGE_DEPTH)
    landing = rewrite_repo_links(landing)
    landing = inject_tech_preview_banner(landing)
    output_dir.mkdir(parents=True, exist_ok=True)
    landing_path = output_dir / "rust-sdk.md"
    landing_path.write_text(
        LANDING_FRONTMATTER + _clean_empty_lines(landing).strip() + "\n", encoding="utf-8"
    )
    print(f"  Wrote landing page {landing_path}")

    # --- Section pages: one per H2 ---
    section_dir = output_dir / "rust-sdk"
    section_dir.mkdir(parents=True, exist_ok=True)

    for i, (title, body) in enumerate(sections):
        slug = _slugify(title)
        position = i + 2  # landing page is 1
        section_content = _rewrite_docs_links(body, depth=_SECTION_PAGE_DEPTH)
        section_content = _rewrite_internal_anchors(section_content, slug, anchor_map)
        section_content = rewrite_repo_links(section_content)
        page_content = _promote_headings(section_content)
        page_content = inject_tech_preview_banner(page_content)
        fm = _make_section_frontmatter(slug, title, position)
        page_path = section_dir / f"{slug}.md"
        page_path.write_text(
            fm + _clean_empty_lines(page_content).strip() + "\n", encoding="utf-8"
        )
        print(f"  Wrote section {page_path}")

    # --- API Reference category metadata ---
    api_ref_dir = section_dir / "api-reference"
    if api_ref_dir.is_dir():
        category_path = api_ref_dir / "_category_.json"
        category_path.write_text(
            json.dumps({"label": "API Reference", "position": _API_REFERENCE_POSITION}, indent=2)
            + "\n",
            encoding="utf-8",
        )
        print(f"  Wrote {category_path}")


# ---------------------------------------------------------------------------
# Link validation
# ---------------------------------------------------------------------------

_RELATIVE_LINK_RE = re.compile(r"\[([^\]]*)\]\((?!https?://|#|mailto:)([^)]+)\)")

# A bracket pair that no Markdown construct follows: not `](`, not `][`.
_BARE_BRACKET_RE = re.compile(r"\[([^\[\]]*)\](?![(\[])")
# `]:` opens a reference definition only at the start of a line (Markdown allows
# up to three spaces of indent); anywhere else the colon is prose punctuation.
_REF_DEF_RE = re.compile(r"^ {0,3}(\[)[^\[\]]*\]:")
_INLINE_CODE_RE = re.compile(r"`+[^`]*`+")
_TASK_LIST_RE = re.compile(r"^\s*[-*+]\s+\[[ xX]\]")
# Conservative: a bare (non-code-span) candidate must look like a Rust path.
_RUST_PATH_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+(?:\(\))?")
_MASK = "\x00"


def _mask_inline_code(line: str) -> str:
    """Blank out inline code spans, preserving offsets so slices still line up."""
    return _INLINE_CODE_RE.sub(lambda m: _MASK * len(m.group(0)), line)


def _find_intra_doc_links(line: str) -> list[str]:
    """Find rustdoc intra-doc links, which Markdown renders as literal brackets.

    rustdoc resolves `[`Foo::bar`]` against the crate graph. Markdown has no such
    graph, so the brackets reach the reader verbatim.
    """
    if _TASK_LIST_RE.match(line):
        return []
    masked = _mask_inline_code(line)
    ref_def = _REF_DEF_RE.match(masked)
    definition_start = ref_def.start(1) if ref_def else -1
    found: list[str] = []
    for m in _BARE_BRACKET_RE.finditer(masked):
        if m.start() == definition_start:
            continue
        inner_masked = m.group(1)
        inner = line[m.start(1) : m.end(1)]
        was_code_span = bool(inner_masked) and set(inner_masked) == {_MASK}
        if was_code_span or _RUST_PATH_RE.fullmatch(inner_masked):
            found.append(f"[{inner}]")
    return found


def validate_generated_links(output_dir: Path) -> list[str]:
    """Flag links that will not resolve once copied into camunda-docs."""
    errors: list[str] = []
    for md_file in sorted(output_dir.rglob("*.md")):
        content = md_file.read_text(encoding="utf-8")
        in_fence = False
        for line_no, line in enumerate(content.splitlines(), start=1):
            if line.lstrip().startswith("```"):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            rel = md_file.relative_to(output_dir)
            for m in _RELATIVE_LINK_RE.finditer(line):
                target = m.group(2).split("#")[0]
                if not target or target.startswith("../") or "/" not in target:
                    continue
                errors.append(
                    f"  {rel}:{line_no}: repo-relative link [{m.group(1)}]({m.group(2)})"
                )
            for link in _find_intra_doc_links(line):
                errors.append(f"  {rel}:{line_no}: unresolved intra-doc link {link}")
    return errors


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def generate_api_reference() -> None:
    sdk = load_rustdoc_json(SDK_JSON_PATH)
    types = collect_types(sdk)
    buckets = classify_types(types)
    examples = load_examples()

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    keys: list[TypeItem] = []
    if CLIENT_JSON_PATH.is_file():
        client_crate = load_rustdoc_json(CLIENT_JSON_PATH)
        keys = [
            t
            for t in collect_types(client_crate).values()
            if "camunda_keys" in t.module_path and not t.name.endswith("ExactMatch")
        ]
    else:
        print(f"  (no {CLIENT_JSON_PATH.name}; skipping the domain keys page)")

    pages: list[tuple[str, str]] = [
        ("camunda-client.md", generate_camunda_client(buckets["camunda-client"], examples)),
        (
            "configuration.md",
            generate_section_page(
                "Configuration",
                "Configuration is resolved from explicit options first, then environment "
                "variables, then built-in defaults.",
                buckets["configuration"],
                examples,
            ),
        ),
        (
            "job-workers.md",
            generate_section_page(
                "Job workers",
                "Job workers poll for jobs of a given type, run a handler, and report the "
                "outcome back to the cluster.",
                buckets["job-workers"],
                examples,
            ),
        ),
        (
            "runtime.md",
            generate_section_page(
                "Runtime",
                "Error types returned by every SDK call, and the adaptive backpressure "
                "manager that paces requests when the cluster pushes back.",
                buckets["runtime"],
                examples,
            ),
        ),
    ]
    if keys:
        pages.append(("domain-keys.md", generate_domain_keys(keys)))

    client_type = types.get("CamundaClient")
    pages.insert(
        0,
        (
            "index.md",
            generate_index(
                {
                    "methods": len(client_type.methods) if client_type else 0,
                    "keys": len(keys),
                }
            ),
        ),
    )

    for filename, content in pages:
        path = OUTPUT_DIR / filename
        path.write_text(_clean_empty_lines(content).rstrip() + "\n", encoding="utf-8")
        print(f"  Wrote {path}")


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(
        description="Generate Docusaurus markdown from rustdoc JSON + README"
    )
    parser.add_argument(
        "--validate-links",
        action="store_true",
        help="After generation, validate links in the generated markdown.",
    )
    parser.add_argument(
        "--readme-only",
        action="store_true",
        help="Only generate README section pages (skip the rustdoc API reference).",
    )
    args = parser.parse_args()

    if not args.readme_only:
        print("Generating API reference from rustdoc JSON...")
        generate_api_reference()

    print("Generating landing + section pages from README...")
    generate_readme_pages(README_PATH, DOCS_MD_DIR)

    if args.validate_links:
        print("Validating generated links...")
        errors = validate_generated_links(DOCS_MD_DIR)
        if errors:
            print("\nERROR: broken links found in generated markdown:")
            print("\n".join(errors))
            sys.exit(1)
        print("  All relative links OK.")

    print("Done.")


if __name__ == "__main__":
    main()
