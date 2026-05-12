import unittest
from unittest import mock
from grafana_utils import project_status_live
from grafana_utils.project_status import (
    build_project_status,
    ProjectStatusFreshness,
    ProjectDomainStatus,
    PROJECT_STATUS_READY,
    PROJECT_STATUS_PARTIAL,
)
from grafana_utils.status_model import StatusReading


class TestProjectStatus(unittest.TestCase):
    def test_build_project_status_ready(self):
        domain = StatusReading(
            id="test",
            scope="live",
            mode="test-mode",
            status=PROJECT_STATUS_READY,
            reason_code="ready",
            primary_count=1,
            source_kinds=["test-source"],
            signal_keys=["test.count"],
            blockers=[],
            warnings=[],
            next_actions=["test-action"],
        ).into_project_domain_status()

        status = build_project_status(
            scope="live",
            domain_count=1,
            freshness=ProjectStatusFreshness(status="ready", source_count=1),
            domains=[domain],
        )

        self.assertEqual(status.overall.status, PROJECT_STATUS_READY)
        self.assertEqual(status.overall.domain_count, 1)
        self.assertEqual(status.overall.present_count, 1)
        self.assertEqual(status.overall.blocked_count, 0)

    def test_build_project_status_partial(self):
        status = build_project_status(
            scope="live",
            domain_count=2,
            freshness=ProjectStatusFreshness(status="ready", source_count=0),
            domains=[],
        )

        self.assertEqual(status.overall.status, PROJECT_STATUS_PARTIAL)
        self.assertEqual(status.overall.domain_count, 2)
        self.assertEqual(status.overall.present_count, 0)

    def test_live_read_failed_domain_is_blocked(self):
        domain = project_status_live._read_failed_domain(
            "dashboard",
            source="live.dashboard",
            error=RuntimeError("boom"),
            next_action="restore dashboard access",
        )

        self.assertEqual(domain.id, "dashboard")
        self.assertEqual(domain.status, "blocked")
        self.assertEqual(domain.reason_code, "live-read-failed")
        self.assertEqual(domain.blocker_count, 1)
        self.assertIn("restore dashboard access", domain.next_actions)

    def test_merge_live_domain_statuses_adds_all_org_mode_suffix(self):
        first = StatusReading(
            id="dashboard",
            scope="live",
            mode="live-list-surfaces",
            status=PROJECT_STATUS_READY,
            reason_code="ready",
            primary_count=2,
            source_kinds=["live-dashboard-search"],
            signal_keys=["live.dashboards.count"],
            blockers=[],
            warnings=[],
            next_actions=["review dashboards"],
            freshness=ProjectStatusFreshness(status="current", source_count=1),
        ).into_project_domain_status()
        second = StatusReading(
            id="dashboard",
            scope="live",
            mode="live-list-surfaces",
            status=PROJECT_STATUS_READY,
            reason_code="ready",
            primary_count=3,
            source_kinds=["live-dashboard-search"],
            signal_keys=["live.dashboards.count"],
            blockers=[],
            warnings=[],
            next_actions=["review dashboards"],
            freshness=ProjectStatusFreshness(status="current", source_count=1),
        ).into_project_domain_status()

        merged = project_status_live.merge_live_domain_statuses([first, second])

        self.assertEqual(merged.primary_count, 5)
        self.assertEqual(merged.mode, "live-list-surfaces-all-orgs")
        self.assertEqual(merged.freshness.source_count, 2)

    def test_build_live_project_status_health_request_uses_path_as_first_arg(self) -> None:
        dashboard_client = mock.Mock()
        datasource_client = mock.Mock()
        alert_client = mock.Mock()
        access_client = mock.Mock()

        dashboard_client.request_json.return_value = {"database": "ok"}

        with mock.patch.object(
            project_status_live,
            "_build_live_grafana_domains_for_scope",
            return_value=[],
        ):
            status = project_status_live.build_live_project_status(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )

        dashboard_client.request_json.assert_called_once_with(
            "/api/health",
            method="GET",
        )
        self.assertIsNotNone(status.discovery)
        self.assertEqual(status.discovery["instance"]["source"], "api-health")
        self.assertEqual(status.discovery["instance"]["status"], "available")

    def test_build_live_project_status_health_request_errors_mark_discovery_unavailable(self) -> None:
        dashboard_client = mock.Mock()
        datasource_client = mock.Mock()
        alert_client = mock.Mock()
        access_client = mock.Mock()

        dashboard_client.request_json.side_effect = RuntimeError("boom")

        with mock.patch.object(
            project_status_live,
            "_build_live_grafana_domains_for_scope",
            return_value=[],
        ):
            status = project_status_live.build_live_project_status(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )

        self.assertIsNotNone(status.discovery)
        self.assertEqual(status.discovery["instance"]["source"], "api-health")
        self.assertEqual(status.discovery["instance"]["status"], "unavailable")
        self.assertIn("error", status.discovery["instance"])
        self.assertIn("boom", status.discovery["instance"]["error"])

    def test_build_live_project_status_health_non_dict_marks_discovery_unavailable(self) -> None:
        dashboard_client = mock.Mock()
        datasource_client = mock.Mock()
        alert_client = mock.Mock()
        access_client = mock.Mock()

        dashboard_client.request_json.return_value = []

        with mock.patch.object(
            project_status_live,
            "_build_live_grafana_domains_for_scope",
            return_value=[],
        ):
            status = project_status_live.build_live_project_status(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )

        self.assertEqual(status.discovery["instance"]["status"], "unavailable")
        self.assertIn("instead of an object", status.discovery["instance"]["error"])

    def test_build_live_project_status_health_bool_marks_discovery_unavailable_with_boolean_message(self) -> None:
        dashboard_client = mock.Mock()
        datasource_client = mock.Mock()
        alert_client = mock.Mock()
        access_client = mock.Mock()

        dashboard_client.request_json.return_value = False

        with mock.patch.object(
            project_status_live,
            "_build_live_grafana_domains_for_scope",
            return_value=[],
        ):
            status = project_status_live.build_live_project_status(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )

        self.assertEqual(status.discovery["instance"]["status"], "unavailable")
        self.assertIn(
            "Grafana /api/health returned a boolean instead of an object.",
            status.discovery["instance"]["error"],
        )

    def test_build_live_project_status_health_float_marks_discovery_unavailable_with_number_message(self) -> None:
        dashboard_client = mock.Mock()
        datasource_client = mock.Mock()
        alert_client = mock.Mock()
        access_client = mock.Mock()

        dashboard_client.request_json.return_value = 1.2

        with mock.patch.object(
            project_status_live,
            "_build_live_grafana_domains_for_scope",
            return_value=[],
        ):
            status = project_status_live.build_live_project_status(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )

        self.assertEqual(status.discovery["instance"]["status"], "unavailable")
        self.assertIn(
            "Grafana /api/health returned a number instead of an object.",
            status.discovery["instance"]["error"],
        )

    def test_build_live_project_status_health_none_marks_discovery_unavailable(self) -> None:
        dashboard_client = mock.Mock()
        datasource_client = mock.Mock()
        alert_client = mock.Mock()
        access_client = mock.Mock()

        dashboard_client.request_json.return_value = None

        with mock.patch.object(
            project_status_live,
            "_build_live_grafana_domains_for_scope",
            return_value=[],
        ):
            status = project_status_live.build_live_project_status(
                dashboard_client,
                datasource_client,
                alert_client,
                access_client,
            )

        self.assertEqual(status.discovery["instance"]["status"], "unavailable")
        self.assertEqual(
            status.discovery["instance"]["error"],
            "Grafana /api/health returned no body."
        )


if __name__ == "__main__":
    unittest.main()
