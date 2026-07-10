#!/usr/bin/env python3
"""Repo-local grafana-util profile management CLI."""

from __future__ import annotations

import argparse
import base64
import getpass
import json
import sys
from pathlib import Path
from typing import Optional

from . import profile_config
from .cli_shared import OUTPUT_FORMAT_CHOICES, build_connection_details, dump_document

STORE_SECRET_CHOICES = ("file", "os", "encrypted-file")


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    """Build the profile CLI parser."""

    parser = argparse.ArgumentParser(
        prog=prog or "grafana-util profile",
        description="Manage repo-local grafana-util profiles.",
        epilog=(
            "Examples:\n\n"
            "  grafana-util profile list\n"
            "  grafana-util profile show --profile prod --output-format yaml\n"
            "  grafana-util profile add prod --url https://grafana.example.com --basic-user admin --prompt-password --store-secret encrypted-file\n"
            "  grafana-util profile example --mode full\n"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--config",
        default=profile_config.DEFAULT_CONFIG_FILENAME,
        help="Repo-local profile config file (default: grafana-util.yaml).",
    )
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    list_parser = subparsers.add_parser("list", help="List profile names.")
    list_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the result as text, table, json, yaml, or interactive.",
    )

    show_parser = subparsers.add_parser("show", help="Show one resolved profile.")
    show_parser.add_argument(
        "--profile",
        default=None,
        help="Profile name to resolve. Falls back to the default profile.",
    )
    show_parser.add_argument(
        "--show-secrets",
        action="store_true",
        help="Reveal stored secret values in the rendered summary.",
    )
    show_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the result as text, table, json, yaml, or interactive.",
    )

    current_parser = subparsers.add_parser(
        "current", help="Show the currently selected profile."
    )
    current_parser.add_argument(
        "--show-secrets",
        action="store_true",
        help="Reveal stored secret values in the rendered summary.",
    )
    current_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the result as text, table, json, yaml, or interactive.",
    )

    validate_parser = subparsers.add_parser(
        "validate", help="Validate the resolved profile connection settings."
    )
    validate_parser.add_argument(
        "--profile",
        default=None,
        help="Profile name to validate. Falls back to the default profile.",
    )
    validate_parser.add_argument(
        "--url",
        default=None,
        help="Override Grafana base URL for validation.",
    )
    validate_parser.add_argument(
        "--token",
        "--api-token",
        dest="api_token",
        default=None,
        help="Override token auth for validation.",
    )
    validate_parser.add_argument(
        "--prompt-token",
        action="store_true",
        help="Prompt for the token while validating.",
    )
    validate_parser.add_argument(
        "--basic-user",
        dest="username",
        default=None,
        help="Override Basic auth username for validation.",
    )
    validate_parser.add_argument(
        "--basic-password",
        dest="password",
        default=None,
        help="Override Basic auth password for validation.",
    )
    validate_parser.add_argument(
        "--prompt-password",
        action="store_true",
        help="Prompt for the Basic auth password while validating.",
    )
    validate_parser.add_argument(
        "--timeout",
        type=int,
        default=None,
        help="Override HTTP timeout for validation.",
    )
    validate_parser.add_argument(
        "--verify-ssl",
        action="store_true",
        help="Enable TLS certificate verification.",
    )
    validate_parser.add_argument(
        "--insecure",
        action="store_true",
        help="Skip TLS certificate verification.",
    )
    validate_parser.add_argument(
        "--ca-cert",
        default=None,
        help="CA certificate path for TLS verification.",
    )
    validate_parser.add_argument(
        "--org-id",
        default=None,
        help="Override organization ID for validation.",
    )
    validate_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="text",
        help="Render the result as text, table, json, yaml, or interactive.",
    )

    add_parser = subparsers.add_parser(
        "add", help="Create or replace one named profile."
    )
    add_parser.add_argument("name", help="Profile name to create or replace.")
    add_parser.add_argument(
        "--url",
        default=None,
        help="Grafana base URL to store in the profile.",
    )
    add_parser.add_argument(
        "--basic-user",
        dest="username",
        default=None,
        help="Basic auth username to store in the profile.",
    )
    add_parser.add_argument(
        "--basic-password",
        dest="password",
        default=None,
        help="Basic auth password to store in the profile.",
    )
    add_parser.add_argument(
        "--prompt-password",
        action="store_true",
        help="Prompt for the Basic auth password before writing the profile.",
    )
    add_parser.add_argument(
        "--token",
        "--api-token",
        dest="api_token",
        default=None,
        help="API token to store in the profile.",
    )
    add_parser.add_argument(
        "--prompt-token",
        action="store_true",
        help="Prompt for the API token before writing the profile.",
    )
    add_parser.add_argument(
        "--token-env",
        default=None,
        help="Environment variable name that supplies the API token.",
    )
    add_parser.add_argument(
        "--password-env",
        default=None,
        help="Environment variable name that supplies the Basic auth password.",
    )
    add_parser.add_argument(
        "--org-id",
        default=None,
        help="Organization ID to store in the profile.",
    )
    add_parser.add_argument(
        "--timeout",
        type=int,
        default=None,
        help="HTTP timeout to store in the profile.",
    )
    add_parser.add_argument(
        "--verify-ssl",
        action="store_true",
        help="Store TLS certificate verification as enabled.",
    )
    add_parser.add_argument(
        "--insecure",
        action="store_true",
        help="Store TLS certificate verification as disabled.",
    )
    add_parser.add_argument(
        "--ca-cert",
        default=None,
        help="CA certificate path to store in the profile.",
    )
    add_parser.add_argument(
        "--store-secret",
        choices=STORE_SECRET_CHOICES,
        default="file",
        help="Secret storage mode to record in the profile.",
    )
    add_parser.add_argument(
        "--secret-passphrase-env",
        default=None,
        help="Environment variable name for the encrypted-file passphrase.",
    )
    add_parser.add_argument(
        "--replace-existing",
        action="store_true",
        help="Allow overwriting an existing profile with the same name.",
    )
    add_parser.add_argument(
        "--set-default",
        action="store_true",
        help="Make the added profile the default selection.",
    )

    init_parser = subparsers.add_parser(
        "init", help="Initialize grafana-util.yaml in the current checkout."
    )
    init_parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite an existing grafana-util.yaml.",
    )
    init_parser.add_argument(
        "--mode",
        choices=("basic", "full"),
        default="full",
        help="Example document mode to write into grafana-util.yaml.",
    )

    example_parser = subparsers.add_parser(
        "example", help="Print a comment-rich reference profile document."
    )
    example_parser.add_argument(
        "--mode",
        choices=("basic", "full"),
        default="full",
        help="Example document mode to render.",
    )
    example_parser.add_argument(
        "--output-format",
        choices=OUTPUT_FORMAT_CHOICES,
        default="yaml",
        help="Render the example as text, table, json, yaml, or interactive.",
    )
    example_parser.add_argument(
        "--output-file",
        default=None,
        help="Optional file to write the example document to.",
    )

    return parser


def _load_document(config_path: str | Path | None) -> dict[str, object]:
    return profile_config.load_profile_document(config_path)


def _resolve_selected_profile(
    args: argparse.Namespace,
) -> tuple[str, dict[str, object]]:
    document = _load_document(args.config)
    return profile_config.select_profile(document, getattr(args, "profile", None))


def _profile_summary_document(
    args: argparse.Namespace,
    *,
    show_secrets: bool = False,
) -> dict[str, object]:
    name, profile = _resolve_selected_profile(args)
    return profile_config.build_profile_summary(
        name, profile, show_secrets=show_secrets
    )


def _as_profile_document(details) -> dict[str, object]:
    headers = dict(details.headers)
    auth_mode = "none"
    if str(headers.get("Authorization", "")).startswith("Bearer "):
        auth_mode = "token"
    elif str(headers.get("Authorization", "")).startswith("Basic "):
        auth_mode = "basic"
    return {
        "profile": details.profile_name,
        "url": details.url,
        "timeout": details.timeout,
        "verifySsl": details.verify_ssl,
        "insecure": details.insecure,
        "caCert": details.ca_cert,
        "orgId": details.org_id,
        "authMode": auth_mode,
        "authorizationHeader": "***" if headers.get("Authorization") else "",
    }


def list_profiles(args: argparse.Namespace) -> int:
    document = _load_document(args.config)
    names = profile_config.list_profile_names(document)
    dump_document({"profiles": names}, getattr(args, "output_format", "text"))
    return 0


def show_profile(args: argparse.Namespace) -> int:
    summary = _profile_summary_document(args, show_secrets=bool(args.show_secrets))
    dump_document(summary, getattr(args, "output_format", "text"))
    return 0


def current_profile(args: argparse.Namespace) -> int:
    summary = _profile_summary_document(args, show_secrets=bool(args.show_secrets))
    dump_document(summary, getattr(args, "output_format", "text"))
    return 0


def validate_profile(args: argparse.Namespace) -> int:
    details = build_connection_details(args, config_path=args.config)
    dump_document(_as_profile_document(details), getattr(args, "output_format", "text"))
    return 0


def _maybe_prompt_secret(
    value: Optional[str], prompt: str, enabled: bool
) -> Optional[str]:
    if value:
        return value
    if not enabled:
        return None
    secret = getpass.getpass(prompt)
    return secret or None


def _encode_secret(value: str) -> str:
    return base64.b64encode(value.encode("utf-8")).decode("ascii")


def add_profile(args: argparse.Namespace) -> int:
    document = _load_document(args.config)
    profiles_value = document.get("profiles")
    profiles = dict(profiles_value) if isinstance(profiles_value, dict) else {}
    if args.name in profiles and not bool(args.replace_existing):
        raise ValueError("Profile already exists: %s" % args.name)

    auth: dict[str, object] = {}
    mode = "none"

    token = _maybe_prompt_secret(
        getattr(args, "api_token", None),
        "Grafana API token: ",
        bool(getattr(args, "prompt_token", False)),
    )
    username = getattr(args, "username", None)
    password = _maybe_prompt_secret(
        getattr(args, "password", None),
        "Grafana Basic auth password: ",
        bool(getattr(args, "prompt_password", False)),
    )
    if token and (username or password):
        raise ValueError("Choose either token auth or Basic auth, not both.")
    if token:
        mode = "token"
        auth["apiTokenEncoded"] = _encode_secret(token)
    elif username or password:
        if not username or not password:
            raise ValueError(
                "Basic auth requires both --basic-user and --basic-password or --prompt-password."
            )
        mode = "basic"
        auth["basicUser"] = username
        auth["basicPasswordEncoded"] = _encode_secret(password)

    token_env = getattr(args, "token_env", None)
    password_env = getattr(args, "password_env", None)
    if token_env:
        auth["tokenEnv"] = token_env
        mode = "token"
    if password_env:
        auth["passwordEnv"] = password_env
        mode = "basic"
    if getattr(args, "secret_passphrase_env", None):
        auth["secretPassphraseEnv"] = args.secret_passphrase_env

    auth["mode"] = mode
    auth["secretMode"] = getattr(args, "store_secret", "file")

    profile: dict[str, object] = {
        "url": args.url or "",
        "orgId": args.org_id or "",
        "timeout": args.timeout if args.timeout is not None else 30,
        "verifySsl": bool(args.verify_ssl),
        "insecure": bool(args.insecure),
        "caCert": args.ca_cert or "",
        "auth": auth,
    }

    profiles[args.name] = profile
    document["profiles"] = profiles
    if bool(getattr(args, "set_default", False)):
        document["defaultProfile"] = args.name
    profile_config.save_profile_document(document, args.config)
    print("Saved profile %s to %s" % (args.name, args.config))
    return 0


def init_profile(args: argparse.Namespace) -> int:
    path = Path(args.config)
    if path.exists() and not bool(args.overwrite):
        raise ValueError("Profile config already exists: %s" % path)
    document = profile_config.build_profile_example_document(args.mode)
    profile_config.save_profile_document(document, args.config)
    print("Initialized %s" % path)
    return 0


def example_profile(args: argparse.Namespace) -> int:
    document = profile_config.build_profile_example_document(args.mode)
    if args.output_file:
        Path(args.output_file).write_text(
            json.dumps(document, indent=2, sort_keys=False),
            encoding="utf-8",
        )
    dump_document(document, args.output_format)
    return 0


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    return build_parser().parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    try:
        args = parse_args(argv)
        if args.command == "list":
            return list_profiles(args)
        if args.command == "show":
            return show_profile(args)
        if args.command == "current":
            return current_profile(args)
        if args.command == "validate":
            return validate_profile(args)
        if args.command == "add":
            return add_profile(args)
        if args.command == "init":
            return init_profile(args)
        if args.command == "example":
            return example_profile(args)
        raise RuntimeError("Unsupported profile command.")
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
