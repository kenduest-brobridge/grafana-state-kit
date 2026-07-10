import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"

if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from docgen_handbook import (  # noqa: E402
    HANDBOOK_NAV_GROUPS,
    HANDBOOK_NAV_TITLES,
    HANDBOOK_ORDER,
)


class DocgenHandbookTests(unittest.TestCase):
    def test_handbook_nav_contract_covers_order_once(self):
        grouped = [
            filename for group in HANDBOOK_NAV_GROUPS for filename in group.files
        ]
        self.assertEqual(set(grouped), set(HANDBOOK_ORDER))
        self.assertEqual(len(grouped), len(HANDBOOK_ORDER))

    def test_handbook_nav_contract_exposes_short_nav_titles(self):
        self.assertEqual(HANDBOOK_NAV_TITLES["zh-TW"]["dashboard"], "Dashboard")
        self.assertEqual(
            HANDBOOK_NAV_TITLES["zh-TW"]["status-workspace"], "Status / Workspace"
        )
        self.assertEqual(HANDBOOK_NAV_TITLES["en"]["architecture"], "Design Principles")


if __name__ == "__main__":
    unittest.main()
