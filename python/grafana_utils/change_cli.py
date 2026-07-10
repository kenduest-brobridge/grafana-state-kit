#!/usr/bin/env python3
"""Review-first change workflow facade for the Python CLI."""

from __future__ import annotations

import argparse
import sys
from typing import Optional

from .cli_shared import (
    OUTPUT_FORMAT_CHOICES,
    add_live_connection_args,
    dump_document,
    summarize_path,
)
from . import sync_cli

CHANGE_COMMAND_HELP = {
    "inspect": "Inspect the staged package and summarize what it contains.",
    "check": "Check whether the staged package is safe to continue.",
    "preview": "Preview what would change from staged or live inputs.",
    "apply": "Apply a reviewed preview with explicit approval.",
    "advanced": "Expose the lower-level staged contracts used by sync.",
}

CHANGE_TO_SYNC_COMMAND = {
    "inspect": "summary",
    "check": "preflight",
    "preview": "plan",
    "apply": "apply",
}


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    """Build the task-first change parser."""

    parser = argparse.ArgumentParser(
        prog=prog or "grafana-util change",
        description=(
            "Task-first staged change workflow with inspect, check, preview, "
            "and apply steps."
        ),
        epilog=(
            "Examples:\n\n"
            "  grafana-util change inspect --workspace .\n"
            "  grafana-util change check --workspace . --fetch-live --output-format json\n"
            "  grafana-util change preview --workspace . --profile prod\n"
            "  grafana-util change apply --preview-file ./change-preview.json --approve --execute-live\n"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True
    for command, help_text in CHANGE_COMMAND_HELP.items():
        command_parser = subparsers.add_parser(command, help=help_text, add_help=False)
        if command != "advanced":
            add_live_connection_args(command_parser)
            command_parser.add_argument(
                "--workspace",
                default=None,
                help="Optional workspace directory to inspect before planning or apply.",
            )
            command_parser.add_argument(
                "--output-format",
                choices=OUTPUT_FORMAT_CHOICES,
                default="text",
                help="Render the result as text, table, json, yaml, or interactive.",
            )
    return parser


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    """Parse change arguments and keep the sync translation stable."""

    parser = build_parser()
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        parser.print_help()
        raise SystemExit(0)
    if argv in (["-h"], ["--help"]):
        parser.print_help()
        raise SystemExit(0)
    if argv[0] not in CHANGE_COMMAND_HELP:
        parser.parse_args(argv)
        raise AssertionError(
            "argparse should have exited for unsupported change command"
        )
    if len(argv) == 1 or argv[1] in ("-h", "--help"):
        if argv[0] == "advanced":
            sync_cli.build_parser(prog="grafana-util change advanced").print_help()
        else:
            build_parser(prog="grafana-util change").print_help()
        raise SystemExit(0)
    if argv[0] == "advanced":
        return argparse.Namespace(entrypoint="change", forwarded_argv=argv[1:])
    if "--workspace" in argv[2:]:
        return argparse.Namespace(
            entrypoint="change",
            command=argv[0],
            workspace=(
                argv[argv.index("--workspace") + 1]
                if argv.index("--workspace") + 1 < len(argv)
                else None
            ),
            output_format=(
                argv[argv.index("--output-format") + 1]
                if "--output-format" in argv
                and argv.index("--output-format") + 1 < len(argv)
                else "text"
            ),
            forwarded_argv=[],
        )
    translated = CHANGE_TO_SYNC_COMMAND[argv[0]]
    return argparse.Namespace(
        entrypoint="change",
        forwarded_argv=[translated] + argv[2:],
    )


def _workspace_summary(args: argparse.Namespace) -> dict[str, object]:
    workspace = summarize_path(args.workspace)
    return {
        "kind": f"change-{args.command}",
        "workspace": workspace,
        "status": "ok",
    }


def main(argv: Optional[list[str]] = None) -> int:
    """Dispatch the change surface to the existing sync workflow engine."""

    args = parse_args(argv)
    if getattr(args, "workspace", None):
        dump_document(_workspace_summary(args), getattr(args, "output_format", "text"))
        return 0
    return sync_cli.main(args.forwarded_argv)


if __name__ == "__main__":
    sys.exit(main())
