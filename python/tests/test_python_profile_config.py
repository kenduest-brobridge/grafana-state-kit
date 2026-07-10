import tempfile
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

from grafana_utils import profile_config


class ProfileConfigTests(unittest.TestCase):
    def test_profile_document_roundtrip_preserves_profiles(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "grafana-util.yaml"
            document = {
                "version": 1,
                "defaultProfile": "prod",
                "profiles": {
                    "prod": {
                        "url": "https://grafana.example.com",
                        "orgId": 1,
                        "auth": {
                            "mode": "basic",
                            "basicUser": "admin",
                            "basicPasswordEncoded": "YWRtaW4=",
                            "secretMode": "file",
                        },
                    }
                },
            }
            profile_config.save_profile_document(document, path)
            loaded = profile_config.load_profile_document(path)

        self.assertEqual(loaded["version"], 1)
        self.assertEqual(loaded["defaultProfile"], "prod")
        self.assertIn("prod", loaded["profiles"])
        self.assertEqual(
            loaded["profiles"]["prod"]["url"], "https://grafana.example.com"
        )

    def test_profile_example_document_includes_multiple_profiles(self):
        document = profile_config.build_profile_example_document("full")
        self.assertIn("prod", document["profiles"])
        self.assertIn("ci", document["profiles"])


if __name__ == "__main__":
    unittest.main()
