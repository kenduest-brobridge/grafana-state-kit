"""Shared CLI helpers for the Python grafana-util command surfaces."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping, Optional, Sequence

from .clients.access_client import GrafanaAccessClient
from .clients.alert_client import GrafanaAlertClient
from .clients.dashboard_client import GrafanaClient
from .clients.datasource_client import GrafanaDatasourceClient
from .profile_config import ProfileConnectionDetails, resolve_connection_details
from . import yaml_compat as yaml

OUTPUT_FORMAT_CHOICES = ("text", "table", "json", "yaml", "interactive")


def add_live_connection_args(
    parser,
    *,
    include_org_id: bool = True,
    include_profile: bool = True,
    include_config: bool = True,
) -> None:
    """Add shared live connection and auth arguments."""

    connection_group = parser.add_argument_group("Connection Options")
    auth_group = parser.add_argument_group("Auth Options")
    if include_config:
        connection_group.add_argument(
            "--config",
            default=None,
            help="Repo-local profile config path (default: grafana-util.yaml).",
        )
    if include_profile:
        connection_group.add_argument(
            "--profile",
            default=None,
            help="Load connection defaults from one named repo-local profile.",
        )
    connection_group.add_argument(
        "--url",
        default=None,
        help="Grafana base URL. Falls back to the selected profile or localhost.",
    )
    auth_group.add_argument(
        "--token",
        "--api-token",
        dest="api_token",
        default=None,
        help="Grafana API token.",
    )
    auth_group.add_argument(
        "--prompt-token",
        action="store_true",
        help="Prompt for the Grafana API token without echo.",
    )
    auth_group.add_argument(
        "--basic-user",
        dest="username",
        default=None,
        help="Grafana Basic auth username.",
    )
    auth_group.add_argument(
        "--basic-password",
        dest="password",
        default=None,
        help="Grafana Basic auth password.",
    )
    auth_group.add_argument(
        "--prompt-password",
        action="store_true",
        help="Prompt for the Grafana Basic auth password without echo.",
    )
    connection_group.add_argument(
        "--timeout",
        type=int,
        default=None,
        help="HTTP timeout in seconds.",
    )
    connection_group.add_argument(
        "--verify-ssl",
        action="store_true",
        help="Enable TLS certificate verification.",
    )
    connection_group.add_argument(
        "--insecure",
        action="store_true",
        help="Skip TLS certificate verification.",
    )
    connection_group.add_argument(
        "--ca-cert",
        default=None,
        help="CA certificate path for TLS verification.",
    )
    if include_org_id:
        connection_group.add_argument(
            "--org-id",
            default=None,
            help="Optional Grafana organization ID.",
        )


def build_connection_details(
    args: Any,
    config_path: Optional[Path | str] = None,
) -> ProfileConnectionDetails:
    """Resolve live connection details with repo-local profile defaults."""

    return resolve_connection_details(args, config_path=config_path)


def build_live_clients(
    args: Any,
    config_path: Optional[Path | str] = None,
) -> tuple[
    ProfileConnectionDetails,
    GrafanaClient,
    GrafanaDatasourceClient,
    GrafanaAlertClient,
    GrafanaAccessClient,
]:
    """Build the common live Grafana clients from resolved connection details."""

    details = build_connection_details(args, config_path=config_path)
    verify_ssl = bool(details.verify_ssl or details.ca_cert)
    dashboard_client = GrafanaClient(
        base_url=details.url,
        headers=details.headers,
        timeout=details.timeout,
        verify_ssl=verify_ssl,
        ca_cert=details.ca_cert,
    )
    datasource_client = GrafanaDatasourceClient(
        base_url=details.url,
        headers=details.headers,
        timeout=details.timeout,
        verify_ssl=verify_ssl,
        ca_cert=details.ca_cert,
    )
    alert_client = GrafanaAlertClient(
        base_url=details.url,
        headers=details.headers,
        timeout=details.timeout,
        verify_ssl=verify_ssl,
        ca_cert=details.ca_cert,
    )
    access_client = GrafanaAccessClient(
        base_url=details.url,
        headers=details.headers,
        timeout=details.timeout,
        verify_ssl=verify_ssl,
        ca_cert=details.ca_cert,
    )
    return details, dashboard_client, datasource_client, alert_client, access_client


def _document_lines(document: Any, indent: int = 0) -> list[str]:
    """Render a generic Python document as human-readable lines."""

    prefix = "  " * indent
    if document is None:
        return [f"{prefix}null"]
    if isinstance(document, Mapping):
        lines: list[str] = []
        for key, value in document.items():
            if isinstance(value, (Mapping, list, tuple)):
                lines.append(f"{prefix}{key}:")
                lines.extend(_document_lines(value, indent + 1))
            else:
                lines.append(f"{prefix}{key}: {value}")
        return lines
    if isinstance(document, (list, tuple)):
        lines = []
        for item in document:
            if isinstance(item, (Mapping, list, tuple)):
                lines.append(f"{prefix}-")
                lines.extend(_document_lines(item, indent + 1))
            else:
                lines.append(f"{prefix}- {item}")
        return lines
    return [f"{prefix}{document}"]


def dump_document(
    document: Any,
    output_format: str = "text",
    *,
    text_lines: Optional[Sequence[str]] = None,
) -> None:
    """Render one JSON/YAML/text document to stdout."""

    normalized = str(output_format or "text").strip().lower()
    if normalized == "json":
        print(json.dumps(document, indent=2, sort_keys=False))
        return
    if normalized == "yaml":
        print(yaml.safe_dump(document).rstrip())
        return
    lines = list(text_lines) if text_lines is not None else _document_lines(document)
    print("\n".join(lines))


def summarize_path(path: Optional[Path | str]) -> Optional[dict[str, Any]]:
    """Return a small summary for one optional file or directory path."""

    if path is None:
        return None
    resolved = Path(path)
    if not resolved.exists():
        return {"path": str(resolved), "exists": False, "count": 0, "kind": "missing"}
    if resolved.is_file():
        return {"path": str(resolved), "exists": True, "count": 1, "kind": "file"}
    count = len([item for item in resolved.rglob("*.json") if item.is_file()])
    return {"path": str(resolved), "exists": True, "count": count, "kind": "dir"}


__all__ = [
    "OUTPUT_FORMAT_CHOICES",
    "add_live_connection_args",
    "build_connection_details",
    "build_live_clients",
    "dump_document",
    "summarize_path",
]
