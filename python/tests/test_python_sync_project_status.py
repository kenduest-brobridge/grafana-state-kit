import unittest

from grafana_utils.sync import project_status


class SyncProjectStatusTests(unittest.TestCase):
    def test_summary_without_preflight_reports_missing_surfaces(self) -> None:
        status = project_status.build_live_sync_domain_status(
            {"summary": {"resourceCount": 1}}
        )

        self.assertIsNotNone(status)
        assert status is not None
        self.assertEqual(
            status.reason_code,
            project_status.SYNC_REASON_PARTIAL_MISSING_SURFACES,
        )


if __name__ == "__main__":
    unittest.main()
