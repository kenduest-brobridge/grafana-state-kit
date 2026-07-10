import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS_DIR = REPO_ROOT / "scripts"

if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

from docgen_entrypoints import (
    HANDBOOK_COMMAND_MAPS,
    JUMP_COMMAND_ENTRIES,
    QUICK_COMMANDS,
)


class DocsEntrypointsTests(unittest.TestCase):
    def test_quick_commands_cover_both_locales(self):
        self.assertIn("en", QUICK_COMMANDS)
        self.assertIn("zh-TW", QUICK_COMMANDS)
        self.assertGreaterEqual(len(QUICK_COMMANDS["en"]), 3)
        self.assertGreaterEqual(len(QUICK_COMMANDS["zh-TW"]), 3)

    def test_jump_entries_include_profile_and_version(self):
        labels = {entry.label for entry in JUMP_COMMAND_ENTRIES["en"]}
        self.assertIn("Version", labels)
        self.assertIn("Config Profile", labels)

    def test_dashboard_handbook_map_lists_subcommands(self):
        groups = HANDBOOK_COMMAND_MAPS["dashboard"]
        commands = {link.command for group in groups for link in group.links}
        self.assertIn("grafana-util dashboard browse", commands)
        self.assertIn("grafana-util dashboard summary", commands)

    def test_handbook_alias_map_reuses_existing_command_map(self):
        self.assertEqual(
            HANDBOOK_COMMAND_MAPS["architecture"],
            HANDBOOK_COMMAND_MAPS["status-workspace"],
        )


if __name__ == "__main__":
    unittest.main()
