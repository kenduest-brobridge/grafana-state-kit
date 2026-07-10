import unittest

from grafana_utils import yaml_compat


class YamlCompatTests(unittest.TestCase):
    def test_safe_dump_and_load_use_yaml_syntax(self) -> None:
        document = {"root": {"lane": "dashboards"}}

        rendered = yaml_compat.safe_dump(document)

        self.assertIn("root:\n", rendered)
        self.assertIn("  lane: dashboards", rendered)
        self.assertEqual(yaml_compat.safe_load(rendered), document)


if __name__ == "__main__":
    unittest.main()
