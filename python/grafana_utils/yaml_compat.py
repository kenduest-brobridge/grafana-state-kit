"""YAML compatibility helpers backed by the required PyYAML runtime."""

from __future__ import annotations

from typing import Any

import yaml


def safe_load(text: str) -> Any:
    """Load a YAML document with safe constructors."""

    return yaml.safe_load(text)


def safe_dump(data: Any) -> str:
    """Dump a YAML document while preserving insertion order."""

    return yaml.safe_dump(data, sort_keys=False, default_flow_style=False)
