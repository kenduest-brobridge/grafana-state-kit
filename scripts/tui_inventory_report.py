#!/usr/bin/env python3
"""Print a read-only inventory of current TUI and interactive surfaces."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCAN_ROOTS = (
    Path("rust/src"),
    Path("docs/commands/en"),
    Path("docs/user-guide/en"),
    Path("docs/internal"),
)
SKIP_PARTS = {"target", "html", "archive", "__pycache__"}
SKIP_FILES = {
    Path("docs/commands/en/completion.md"),
    Path("docs/internal/ai-changes.md"),
    Path("docs/internal/ai-learnings.md"),
    Path("docs/internal/ai-status.md"),
}
TUI_RE = re.compile(
    r"ratatui|crossterm|tui_shell|TerminalSession|feature = \"tui\"|"
    r"--interactive|output-format interactive|browse interactively|"
    r"interactive terminal|terminal UI|Terminal UI|TUI",
)
HELPER_DRIFT_RE = re.compile(
    r"^\s*fn\s+"
    r"(?P<helper>detail_line|fact_line|build_info_lines|build_review_lines|"
    r"control_line|key_chip|plain|muted|plain_boxed|boxed)\s*\("
)


@dataclass(frozen=True)
class InventoryItem:
    path: str
    kind: str
    signals: tuple[str, ...]


@dataclass(frozen=True)
class HelperDriftItem:
    path: str
    helper: str
    line: int
    signal: str


def iter_scan_files() -> list[Path]:
    files: list[Path] = []
    for root in SCAN_ROOTS:
        absolute_root = REPO_ROOT / root
        if not absolute_root.exists():
            continue
        for path in absolute_root.rglob("*"):
            if not path.is_file():
                continue
            relative = path.relative_to(REPO_ROOT)
            if any(part in SKIP_PARTS for part in relative.parts):
                continue
            if relative in SKIP_FILES:
                continue
            if path.suffix == ".rs" and (
                "tests" in relative.parts
                or path.name.endswith("_rust_tests.rs")
                or path.name.endswith("_tests.rs")
            ):
                continue
            if path.suffix not in {".rs", ".md"}:
                continue
            files.append(relative)
    return sorted(files)


def classify(path: Path, text: str, signals: tuple[str, ...]) -> str:
    path_text = path.as_posix()
    if path.suffix == ".md":
        return "docs"
    if "common/tui" in path_text or "common/browser/session.rs" in path_text:
        return "shared"
    if "browse" in path_text:
        return "browse"
    if "workbench" in path_text or "review_tui" in path_text or "audit_tui" in path_text:
        return "workbench"
    if "interactive" in path_text or "--interactive" in text:
        return "interactive"
    if any("feature = \"tui\"" in signal for signal in signals):
        return "feature-gated"
    return "other"


def collect_signals(text: str) -> tuple[str, ...]:
    signals: list[str] = []
    for line in text.splitlines():
        match = TUI_RE.search(line)
        if match:
            snippet = line.strip()
            if len(snippet) > 120:
                snippet = snippet[:117].rstrip() + "..."
            signals.append(snippet)
        if len(signals) >= 3:
            break
    return tuple(signals)


def build_inventory() -> list[InventoryItem]:
    items: list[InventoryItem] = []
    for relative in iter_scan_files():
        text = (REPO_ROOT / relative).read_text(encoding="utf-8")
        signals = collect_signals(text)
        if not signals:
            continue
        items.append(
            InventoryItem(
                path=relative.as_posix(),
                kind=classify(relative, text, signals),
                signals=signals,
            )
        )
    return items


def collect_helper_drift(path: Path, text: str) -> list[HelperDriftItem]:
    path_text = path.as_posix()
    if "common/tui" in path_text or "common/browser/session.rs" in path_text:
        return []
    if "tests" in path.parts or path.name.endswith("_tests.rs"):
        return []
    items: list[HelperDriftItem] = []
    lines = text.splitlines()
    for index, line in enumerate(lines, start=1):
        match = HELPER_DRIFT_RE.search(line)
        if not match:
            continue
        body = "\n".join(lines[index : min(index + 6, len(lines))])
        body_signal = next(
            (
                body_line.strip()
                for body_line in body.splitlines()
                if "tui_shell::" in body_line or "browser_" in body_line or "format!" in body_line
            ),
            "",
        )
        if body_signal:
            signal = f"{line.strip()} -> {body_signal}"
            items.append(
                HelperDriftItem(
                    path=path_text,
                    helper=match.group("helper"),
                    line=index,
                    signal=signal,
                )
            )
    return items


def build_helper_drift() -> list[HelperDriftItem]:
    items: list[HelperDriftItem] = []
    for relative in iter_scan_files():
        if relative.suffix != ".rs":
            continue
        text = (REPO_ROOT / relative).read_text(encoding="utf-8")
        items.extend(collect_helper_drift(relative, text))
    return sorted(items, key=lambda item: (item.path, item.line, item.helper))


def print_text_report(items: list[InventoryItem], helper_drift: list[HelperDriftItem]) -> None:
    by_kind: dict[str, list[InventoryItem]] = {}
    for item in items:
        by_kind.setdefault(item.kind, []).append(item)

    print("TUI inventory report")
    print("====================")
    print(f"Scanned roots: {', '.join(root.as_posix() for root in SCAN_ROOTS)}")
    print(f"Matched files: {len(items)}")
    print()
    for kind in sorted(by_kind):
        grouped = by_kind[kind]
        print(f"{kind} ({len(grouped)})")
        for item in grouped:
            print(f"  - {item.path}")
            print(f"    signal: {item.signals[0]}")
        print()

    print(f"helper-drift candidates ({len(helper_drift)})")
    for item in helper_drift:
        print(f"  - {item.path}:{item.line} {item.helper}")
        print(f"    signal: {item.signal}")
    print()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit machine-readable inventory")
    args = parser.parse_args()

    items = build_inventory()
    helper_drift = build_helper_drift()
    if args.json:
        print(
            json.dumps(
                {
                    "items": [asdict(item) for item in items],
                    "helperDrift": [asdict(item) for item in helper_drift],
                },
                indent=2,
                sort_keys=True,
            )
        )
    else:
        print_text_report(items, helper_drift)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
