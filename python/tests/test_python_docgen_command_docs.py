import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"

if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from docgen_command_docs import command_doc_cli_path  # noqa: E402


class CommandDocCliPathTests(unittest.TestCase):
    def test_reads_backticked_canonical_path(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "resource-describe.md"
            path.write_text(
                "# `grafana-util status resource describe`\n", encoding="utf-8"
            )
            self.assertEqual(
                command_doc_cli_path(path), "grafana-util status resource describe"
            )

    def test_reads_bare_command_heading(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "dashboard-browse.md"
            path.write_text("# dashboard browse\n", encoding="utf-8")
            self.assertEqual(
                command_doc_cli_path(path), "grafana-util dashboard browse"
            )

    def test_reads_removed_root_heading(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "overview.md"
            path.write_text(
                "# 已移除的 root path：`grafana-util overview`\n", encoding="utf-8"
            )
            self.assertEqual(command_doc_cli_path(path), "grafana-util overview")


if __name__ == "__main__":
    unittest.main()
