from __future__ import annotations

import unittest
from pathlib import Path

from scripts.tui_inventory_report import collect_helper_drift


class TuiInventoryReportTest(unittest.TestCase):
    def test_collect_helper_drift_reports_local_tui_delegate_helpers(self) -> None:
        text = """
use crate::tui_shell;

fn control_line(items: &[(&str, Color, &str)]) -> Line<'static> {
    tui_shell::fixed_body_control_line(items, 14)
}
"""

        drift = collect_helper_drift(Path("rust/src/commands/datasource/browse/render.rs"), text)

        self.assertEqual(len(drift), 1)
        self.assertEqual(drift[0].helper, "control_line")
        self.assertEqual(drift[0].line, 4)
        self.assertIn("tui_shell", drift[0].signal)

    def test_collect_helper_drift_ignores_shared_tui_shell_helpers(self) -> None:
        text = """
pub(crate) fn control_line(items: &[(&str, Color, &str)]) -> Line<'static> {
    build_control_line(items, &[])
}
"""

        drift = collect_helper_drift(Path("rust/src/common/tui/shell.rs"), text)

        self.assertEqual(drift, [])


if __name__ == "__main__":
    unittest.main()
