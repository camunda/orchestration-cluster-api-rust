"""Shared context and helpers for the numbered post-processing hooks.

Each hook is a module under ``scripts/hooks/`` exposing ``NUMBER``, ``NAME`` and a
``run(ctx: Context) -> None`` entry point. The orchestrator (``scripts/postprocess.py``)
builds a single :class:`Context`, then runs the hooks in ``NUMBER`` order. Hooks share
this context so the spec is parsed and the domain keys discovered exactly once.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from pathlib import Path


def snake_case(name: str) -> str:
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    s = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", s)
    return s.lower()


def load_json(path: Path):
    with path.open() as f:
        return json.load(f)


def ref_name(ref: str) -> str:
    return ref.split("/")[-1]


def is_string_scalar(schema: dict | None) -> bool:
    return bool(schema) and schema.get("type") == "string" and "enum" not in schema


def resolve_constraints(name: str, schemas: dict, seen: set[str] | None = None) -> dict:
    """Walk the allOf chain to resolve pattern/minLength/maxLength for a semantic key."""
    if seen is None:
        seen = set()
    schema = schemas.get(name)
    if not schema or name in seen:
        return {}
    seen.add(name)
    out = {
        "pattern": schema.get("pattern"),
        "minLength": schema.get("minLength"),
        "maxLength": schema.get("maxLength"),
        "description": schema.get("description") or schema.get("title"),
        "example": schema.get("example"),
    }
    for sub in schema.get("allOf", []):
        ref = sub.get("$ref")
        if not ref:
            continue
        inherited = resolve_constraints(ref_name(ref), schemas, seen)
        for k in ("pattern", "minLength", "maxLength"):
            if out.get(k) is None:
                out[k] = inherited.get(k)
        if not out.get("description"):
            out["description"] = inherited.get("description")
    return out


def discover_domain_keys(schemas: dict) -> list[dict]:
    """Find all semantic-key schemas: string scalars with x-semantic-type, or
    single-ref allOf wrappers over another string scalar (e.g. *ExactMatch)."""
    keys: list[dict] = []
    for name, schema in schemas.items():
        if not is_string_scalar(schema):
            continue
        is_key = False
        if schema.get("x-semantic-type"):
            is_key = True
        else:
            refs = [ref_name(s["$ref"]) for s in schema.get("allOf", []) if s.get("$ref")]
            if len(refs) == 1 and is_string_scalar(schemas.get(refs[0])):
                is_key = True
        if is_key:
            c = resolve_constraints(name, schemas)
            c["name"] = name
            keys.append(c)
    keys.sort(key=lambda k: k["name"])
    return keys


@dataclass
class Context:
    """Shared state threaded through every post-processing hook."""

    client_dir: Path
    spec: dict
    schemas: dict
    metadata: dict | None = None
    domain_keys: list[dict] = field(default_factory=list)
    # Names of every schema carrying an x-semantic-type (the Domain Type System surface).
    semantic_type_names: set[str] = field(default_factory=set)

    @property
    def models_dir(self) -> Path:
        return self.client_dir / "src" / "models"

    @classmethod
    def build(cls, client_dir: Path, spec_path: Path, metadata_path: Path | None) -> "Context":
        spec = load_json(spec_path)
        schemas = spec.get("components", {}).get("schemas", {})
        metadata = load_json(metadata_path) if metadata_path and metadata_path.exists() else None
        domain_keys = discover_domain_keys(schemas)
        semantic_type_names = {
            n for n, sc in schemas.items() if isinstance(sc, dict) and sc.get("x-semantic-type")
        }
        return cls(
            client_dir=client_dir,
            spec=spec,
            schemas=schemas,
            metadata=metadata,
            domain_keys=domain_keys,
            semantic_type_names=semantic_type_names,
        )

    def log(self, hook: str, message: str) -> None:
        print(f"[postprocess:{hook}] {message}")
