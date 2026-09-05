import importlib
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

unified_cli = importlib.import_module("grafana_utils.unified_cli")


class UnifiedDashboardCommandsCliTests(unittest.TestCase):
    def test_unified_cli_dashboard_group_help_mentions_dashboard_commands(
        self,
    ):
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            with self.assertRaises(SystemExit):
                unified_cli.parse_args(["dashboard"])

        help_text = stdout.getvalue()
        self.assertIn("raw-to-prompt", help_text)
        self.assertIn("variables", help_text)
        self.assertNotIn("screenshot", help_text)
        self.assertIn("summary --input-format provisioning", help_text)

    def test_unified_cli_dashboard_parse_args_supports_dashboard_raw_to_prompt_namespace(
        self,
    ):
        args = unified_cli.parse_args(
            [
                "dashboard",
                "raw-to-prompt",
                "--input-file",
                "./dashboards/raw/cpu-main.json",
            ]
        )

        self.assertEqual(args.entrypoint, "dashboard")
        self.assertEqual(
            args.forwarded_argv,
            ["raw-to-prompt", "--input-file", "./dashboards/raw/cpu-main.json"],
        )

    def test_unified_cli_dashboard_parse_args_supports_dashboard_list_vars_namespace(
        self,
    ):
        args = unified_cli.parse_args(
            ["dashboard", "variables", "--dashboard-uid", "cpu-main"]
        )

        self.assertEqual(args.entrypoint, "dashboard")
        self.assertEqual(
            args.forwarded_argv,
            ["variables", "--dashboard-uid", "cpu-main"],
        )

    def test_unified_cli_dashboard_parse_args_rejects_dashboard_inspect_vars_alias(
        self,
    ):
        with self.assertRaises(SystemExit):
            unified_cli.parse_args(
                ["dashboard", "inspect-vars", "--dashboard-uid", "cpu-main"]
            )

    def test_unified_cli_dashboard_commands_reject_retired_screenshot_namespace(self):
        with self.assertRaises(SystemExit):
            unified_cli.parse_args(["dashboard", "screenshot"])

    def test_unified_cli_dashboard_parse_args_supports_dashboard_alias(self):
        args = unified_cli.parse_args(["db", "list", "--json"])

        self.assertEqual(args.entrypoint, "dashboard")
        self.assertEqual(args.forwarded_argv, ["list", "--json"])

    def test_unified_cli_dashboard_parse_args_supports_sync_alias(self):
        args = unified_cli.parse_args(
            ["sy", "plan", "--desired-file", "./desired.json"]
        )

        self.assertEqual(args.entrypoint, "sync")
        self.assertEqual(
            args.forwarded_argv, ["plan", "--desired-file", "./desired.json"]
        )


if __name__ == "__main__":
    unittest.main()
