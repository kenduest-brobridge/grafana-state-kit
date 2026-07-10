from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path
import sys


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "rust_maintainability_report.py"


def load_module():
    spec = importlib.util.spec_from_file_location(
        "rust_maintainability_report", SCRIPT_PATH
    )
    module = importlib.util.module_from_spec(spec)
    sys.modules.setdefault("rust_maintainability_report", module)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class RustMaintainabilityReportTests(unittest.TestCase):
    def test_discover_rust_maintainability_findings_reports_oversized_files(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            source = root / "src"
            source.mkdir()
            path = source / "large.rs"
            path.write_text("\n".join(["fn x() {}"] * 9), encoding="utf-8")

            findings = module.discover_rust_maintainability_findings(
                source,
                source_line_limit=5,
                test_line_limit=5,
                reexport_line_limit=99,
            )

            self.assertEqual(len(findings), 1)
            self.assertEqual(findings[0].category, "oversized-file")
            self.assertEqual(findings[0].path.resolve(), path.resolve())

    def test_discover_rust_maintainability_findings_reports_reexport_counts(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            source = root / "src"
            source.mkdir()
            path = source / "mod.rs"
            path.write_text(
                "\n".join(
                    [
                        "pub use crate::alpha::A;",
                        "pub use crate::beta::B;",
                        "pub use crate::gamma::C;",
                    ]
                ),
                encoding="utf-8",
            )

            findings = module.discover_rust_maintainability_findings(
                source,
                source_line_limit=99,
                test_line_limit=99,
                reexport_line_limit=2,
            )

            self.assertEqual(len(findings), 1)
            self.assertEqual(findings[0].category, "reexport-heavy")
            self.assertEqual(findings[0].detail, "3 pub use lines")
            self.assertEqual(findings[0].path.resolve(), path.resolve())

    def test_discover_rust_maintainability_directory_summaries_reports_domain_totals(
        self,
    ):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            sync_dir = root / "sync"
            nested_dir = sync_dir / "nested"
            nested_dir.mkdir(parents=True)

            first = sync_dir / "bundle_preflight.rs"
            first.write_text("\n".join(["fn alpha() {}"] * 6), encoding="utf-8")
            second = sync_dir / "promotion_preflight.rs"
            second.write_text("\n".join(["fn beta() {}"] * 4), encoding="utf-8")
            third = nested_dir / "workspace_discovery.rs"
            third.write_text("\n".join(["fn gamma() {}"] * 8), encoding="utf-8")

            summaries = module.discover_rust_maintainability_directory_summaries(
                [sync_dir], hotspot_limit=2
            )

            self.assertEqual(len(summaries), 1)
            summary = summaries[0]
            self.assertEqual(summary.path.resolve(), sync_dir.resolve())
            self.assertEqual(summary.file_count, 3)
            self.assertEqual(summary.line_count, 18)
            self.assertEqual(
                [item.path.resolve() for item in summary.hotspots],
                [third.resolve(), first.resolve()],
            )
            self.assertEqual([item.line_count for item in summary.hotspots], [8, 6])

    def test_render_rust_maintainability_report_places_directory_summaries_first(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            sync_dir = root / "sync"
            sync_dir.mkdir()
            (sync_dir / "bundle_inputs.rs").write_text(
                "fn alpha() {}\nfn beta() {}\n", encoding="utf-8"
            )
            (sync_dir / "workspace_discovery.rs").write_text(
                "fn gamma() {}\n", encoding="utf-8"
            )

            lines = module.render_rust_maintainability_report(
                sync_dir,
                summary_roots=[sync_dir],
                source_line_limit=99,
                test_line_limit=99,
                reexport_line_limit=99,
                summary_hotspot_limit=2,
            )

            self.assertGreaterEqual(len(lines), 1)
            self.assertTrue(
                lines[0].startswith("directory-summary\t2 files, 3 lines; hotspots: ")
            )
            self.assertIn("bundle_inputs.rs (2 lines)", lines[0])
            self.assertIn("workspace_discovery.rs (1 lines)", lines[0])


if __name__ == "__main__":
    unittest.main()
