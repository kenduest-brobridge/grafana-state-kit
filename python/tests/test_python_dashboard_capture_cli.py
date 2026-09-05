import importlib
import io
import tempfile
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

dashboard_cli = importlib.import_module("grafana_utils.dashboard_cli")


class DashboardVariableCliTests(unittest.TestCase):
    def test_dashboard_capture_parse_args_supports_list_vars(self):
        args = dashboard_cli.parse_args(
            [
                "variables",
                "--dashboard-uid",
                "cpu-main",
                "--vars-query",
                "var-env=prod",
                "--output-format",
                "json",
            ]
        )

        self.assertEqual(args.command, "variables")
        self.assertEqual(args.dashboard_uid, "cpu-main")
        self.assertEqual(args.vars_query, "var-env=prod")
        self.assertEqual(args.output_format, "json")

    def test_dashboard_capture_parse_args_rejects_inspect_vars_alias(self):
        with self.assertRaises(SystemExit):
            dashboard_cli.parse_args(["inspect-vars", "--dashboard-uid", "cpu-main"])

    def test_dashboard_capture_parse_args_supports_list_vars_output_file(self):
        args = dashboard_cli.parse_args(
            [
                "variables",
                "--dashboard-uid",
                "cpu-main",
                "--output-file",
                "/tmp/variables-output.txt",
            ]
        )

        self.assertEqual(args.command, "variables")
        self.assertEqual(args.output_file, "/tmp/variables-output.txt")

    def test_dashboard_capture_list_vars_help_mentions_dashboard_inputs(self):
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            with self.assertRaises(SystemExit):
                dashboard_cli.parse_args(["variables", "-h"])

        help_text = stdout.getvalue()
        self.assertIn("--dashboard-uid", help_text)
        self.assertIn("--dashboard-url", help_text)
        self.assertIn("--vars-query", help_text)
        self.assertIn("--output-format", help_text)
        self.assertIn("variables", help_text)

    def test_dashboard_capture_main_dispatches_list_vars(self):
        with mock.patch.object(
            dashboard_cli, "list_vars_command", return_value=13
        ) as mocked:
            result = dashboard_cli.main(["variables", "--dashboard-uid", "cpu-main"])

        self.assertEqual(result, 13)
        mocked.assert_called_once()

    def test_dashboard_capture_list_vars_supports_output_file(self):
        output_dir = tempfile.TemporaryDirectory(prefix="grafana-util-variables-")
        self.addCleanup(output_dir.cleanup)
        output_file = Path(output_dir.name) / "variables.txt"
        args = dashboard_cli.parse_args(
            [
                "variables",
                "--dashboard-uid",
                "cpu-main",
                "--output-file",
                str(output_file),
                "--token",
                "secret",
            ]
        )
        with mock.patch.object(
            dashboard_cli,
            "inspect_dashboard_variables_with_client",
            return_value={
                "dashboardUid": "cpu-main",
                "dashboardTitle": "CPU Main",
                "variableCount": 1,
                "variables": [
                    {
                        "name": "env",
                        "type": "query",
                        "label": "Environment",
                        "current": "prod",
                        "datasource": "prom-main",
                        "query": "up",
                        "multi": False,
                        "includeAll": False,
                        "optionCount": 1,
                        "options": ["prod"],
                    }
                ],
            },
        ) as mocked:
            stdout = io.StringIO()
            with redirect_stdout(stdout):
                result = dashboard_cli.inspect_vars(args)

        self.assertEqual(result, 0)
        self.assertTrue(mocked.called)
        self.assertTrue(output_file.exists())
        self.assertEqual(output_file.read_text(encoding="utf-8"), stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
