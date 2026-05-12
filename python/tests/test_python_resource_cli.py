import argparse
import sys
import unittest
from unittest import mock

from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

import grafana_utils.resource_cli as resource_cli  # noqa: E402


class FakeListClient:
    def __init__(self):
        self.calls = []

    def request_json(self, *args, **kwargs):
        self.calls.append((args, kwargs))
        return []


class FakeGetClient:
    def __init__(self):
        self.calls = []

    def request_json(self, *args, **kwargs):
        self.calls.append((args, kwargs))
        return {"uid": kwargs.get("path") or (args[0] if args else None)}


class ResourceCliTests(unittest.TestCase):
    def test_list_folders_uses_path_as_first_arg(self) -> None:
        dashboard_client = FakeListClient()
        datasource_client = FakeListClient()
        alert_client = mock.Mock()
        access_client = FakeListClient()

        args = argparse.Namespace(
            command="list",
            kind="folders",
            output_format="json",
            verify_ssl=False,
            url="http://127.0.0.1:3000",
        )

        with mock.patch.object(
            resource_cli,
            "build_live_clients",
            return_value=(None, dashboard_client, datasource_client, alert_client, access_client),
        ), mock.patch.object(resource_cli, "dump_document"):
            resource_cli.list_command(args)

        self.assertEqual(len(dashboard_client.calls), 1)
        args0, kwargs0 = dashboard_client.calls[0]
        self.assertEqual(len(args0), 1)
        self.assertEqual(args0[0], "/api/folders")
        self.assertEqual(kwargs0.get("method"), "GET")

    def test_get_folder_uses_path_as_first_arg(self) -> None:
        dashboard_client = FakeGetClient()
        datasource_client = FakeGetClient()
        alert_client = mock.Mock()
        access_client = FakeGetClient()

        args = argparse.Namespace(
            command="get",
            selector="folders/folder-1",
            output_format="json",
            verify_ssl=False,
            url="http://127.0.0.1:3000",
        )

        with mock.patch.object(
            resource_cli,
            "build_live_clients",
            return_value=(None, dashboard_client, datasource_client, alert_client, access_client),
        ), mock.patch.object(resource_cli, "dump_document"):
            resource_cli.get_command(args)

        self.assertEqual(len(dashboard_client.calls), 1)
        args0, kwargs0 = dashboard_client.calls[0]
        self.assertEqual(len(args0), 1)
        self.assertEqual(args0[0], "/api/folders/folder-1")
        self.assertEqual(kwargs0.get("method"), "GET")

    def test_list_orgs_uses_path_as_first_arg(self) -> None:
        dashboard_client = FakeListClient()
        datasource_client = FakeListClient()
        alert_client = mock.Mock()
        access_client = FakeListClient()

        args = argparse.Namespace(
            command="list",
            kind="orgs",
            output_format="json",
            verify_ssl=False,
            url="http://127.0.0.1:3000",
        )

        with mock.patch.object(
            resource_cli,
            "build_live_clients",
            return_value=(None, dashboard_client, datasource_client, alert_client, access_client),
        ), mock.patch.object(resource_cli, "dump_document"):
            resource_cli.list_command(args)

        self.assertEqual(len(access_client.calls), 1)
        args0, kwargs0 = access_client.calls[0]
        self.assertEqual(len(args0), 1)
        self.assertEqual(args0[0], "/api/orgs")
        self.assertEqual(kwargs0.get("method"), "GET")

    def test_get_org_uses_path_as_first_arg(self) -> None:
        dashboard_client = FakeGetClient()
        datasource_client = FakeGetClient()
        alert_client = mock.Mock()
        access_client = FakeGetClient()

        args = argparse.Namespace(
            command="get",
            selector="orgs/99",
            output_format="json",
            verify_ssl=False,
            url="http://127.0.0.1:3000",
        )

        with mock.patch.object(
            resource_cli,
            "build_live_clients",
            return_value=(None, dashboard_client, datasource_client, alert_client, access_client),
        ), mock.patch.object(resource_cli, "dump_document"):
            resource_cli.get_command(args)

        self.assertEqual(len(access_client.calls), 1)
        args0, kwargs0 = access_client.calls[0]
        self.assertEqual(len(args0), 1)
        self.assertEqual(args0[0], "/api/orgs/99")
        self.assertEqual(kwargs0.get("method"), "GET")

    def test_get_datasource_uses_path_as_first_arg(self) -> None:
        dashboard_client = FakeGetClient()
        datasource_client = FakeGetClient()
        alert_client = mock.Mock()
        access_client = FakeGetClient()

        args = argparse.Namespace(
            command="get",
            selector="datasources/datasource-1",
            output_format="json",
            verify_ssl=False,
            url="http://127.0.0.1:3000",
        )

        with mock.patch.object(
            resource_cli,
            "build_live_clients",
            return_value=(None, dashboard_client, datasource_client, alert_client, access_client),
        ), mock.patch.object(resource_cli, "dump_document"):
            resource_cli.get_command(args)

        self.assertEqual(len(datasource_client.calls), 1)
        args0, kwargs0 = datasource_client.calls[0]
        self.assertEqual(len(args0), 1)
        self.assertEqual(args0[0], "/api/datasources/uid/datasource-1")
        self.assertEqual(kwargs0.get("method"), "GET")
