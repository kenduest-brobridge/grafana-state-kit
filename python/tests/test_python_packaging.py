import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_ROOT = REPO_ROOT / "python"
MAKEFILE_PATH = REPO_ROOT / "Makefile"
CI_WORKFLOW_PATH = REPO_ROOT / ".github" / "workflows" / "ci.yml"
GITLAB_CI_PATH = REPO_ROOT / ".gitlab-ci.yml"
PYPROJECT_PATH = PYTHON_ROOT / "pyproject.toml"
MODULE_ENTRYPOINT_PATH = REPO_ROOT / "python" / "grafana_utils" / "__main__.py"
POETRY_LOCK_PATH = PYTHON_ROOT / "poetry.lock"
SET_VERSION_SCRIPT_PATH = REPO_ROOT / "scripts" / "set-version.sh"
BUILD_RUST_MACOS_ARM64_SCRIPT_PATH = REPO_ROOT / "scripts" / "build-rust-macos-arm64.sh"
VERSION_PATH = REPO_ROOT / "VERSION"
DEVELOPER_GUIDE_PATH = REPO_ROOT / "docs" / "DEVELOPER.md"
RUST_LIVE_SMOKE_PATH = REPO_ROOT / "scripts" / "test-rust-live-grafana.sh"
PYTHON_DATASOURCE_LIVE_SMOKE_PATH = REPO_ROOT / "scripts" / "test-python-datasource-live-grafana.sh"


class PackagingTests(unittest.TestCase):
    def test_packaging_pyproject_exists(self):
        self.assertTrue(PYPROJECT_PATH.is_file())

    def test_packaging_repo_commits_makefile_version_sources(self):
        self.assertTrue(MAKEFILE_PATH.is_file())
        self.assertTrue(SET_VERSION_SCRIPT_PATH.is_file())
        self.assertTrue(VERSION_PATH.is_file())

    def test_packaging_ci_runs_python_quality_and_test_gate(self):
        content = CI_WORKFLOW_PATH.read_text(encoding="utf-8")

        self.assertIn("python-quality:", content)
        self.assertIn("make test-python quality-python", content)

    def test_packaging_gitlab_ci_uses_current_layout_and_binary_surface(self):
        content = GITLAB_CI_PATH.read_text(encoding="utf-8")

        self.assertIn("python/pyproject.toml", content)
        self.assertNotIn('(root / "pyproject.toml")', content)
        self.assertIn("cd python", content)
        self.assertNotIn("grafana-access-utils", content)
        self.assertIn("grafana-util", content)
        self.assertNotIn("    - make test-python", content)
        self.assertIn(
            "PYTHONPATH=python python3 -m unittest discover -s python/tests -v",
            content,
        )

    def test_packaging_pyproject_declares_console_scripts(self):
        content = PYPROJECT_PATH.read_text(encoding="utf-8")

        self.assertRegex(content, r"(?m)^\[project\.scripts\]$")
        self.assertRegex(
            content, r'(?m)^grafana-util = "grafana_utils\.unified_cli:main"$'
        )
        self.assertNotRegex(content, r"(?m)^grafana-access-utils = ")

    def test_packaging_pyproject_declares_base_requests_dependency(self):
        content = PYPROJECT_PATH.read_text(encoding="utf-8")

        self.assertIn("requests>=2.27,<3", content)

    def test_packaging_pyproject_requires_python39_or_newer(self):
        content = PYPROJECT_PATH.read_text(encoding="utf-8")

        self.assertIn('requires-python = ">=3.9"', content)

    def test_packaging_pyproject_finds_package_submodules(self):
        content = PYPROJECT_PATH.read_text(encoding="utf-8")

        self.assertIn('{ include = "grafana_utils", from = "." }', content)
        self.assertIn('package-dir = {"" = "."}', content)
        self.assertIn('where = ["."]', content)
        self.assertIn('include = ["grafana_utils", "grafana_utils.*"]', content)

    def test_packaging_pyproject_declares_poetry_dev_group(self):
        content = PYPROJECT_PATH.read_text(encoding="utf-8")

        self.assertRegex(content, r"(?m)^\[tool\.poetry\]$")
        self.assertRegex(content, r'(?m)^requires-poetry = ">=2\.1"$')
        self.assertRegex(content, r"(?m)^\[tool\.poetry\.group\.dev\.dependencies\]$")
        self.assertIn('black = ">=24,<26"', content)
        self.assertIn('build = ">=1.2,<2"', content)
        self.assertIn('mypy = ">=1.10,<2"', content)
        self.assertIn('ruff = ">=0.11,<1"', content)
        self.assertIn('setuptools = ">=59"', content)
        self.assertIn('wheel = ">=0.45,<1"', content)

    def test_packaging_package_declares_module_entrypoint(self):
        self.assertTrue(MODULE_ENTRYPOINT_PATH.is_file())

    def test_packaging_repo_commits_poetry_lock(self):
        self.assertTrue(POETRY_LOCK_PATH.is_file())

    def test_packaging_makefile_declares_version_targets(self):
        content = MAKEFILE_PATH.read_text(encoding="utf-8")

        self.assertIn("print-version:", content)
        self.assertIn("sync-version:", content)
        self.assertIn("set-release-version:", content)
        self.assertIn("set-dev-version:", content)

    def test_packaging_macos_arm64_build_script_resigns_copied_binary(self):
        content = BUILD_RUST_MACOS_ARM64_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn(
            'cp "${RUST_DIR}/target/release/grafana-util" "${OUTPUT_DIR}/grafana-util"',
            content,
        )
        self.assertIn('codesign --force --sign - "${OUTPUT_DIR}/grafana-util"', content)

    def test_compatibility_matrix_uses_pinned_released_grafana_images(self):
        makefile = MAKEFILE_PATH.read_text(encoding="utf-8")
        workflow = CI_WORKFLOW_PATH.read_text(encoding="utf-8")

        self.assertIn("GRAFANA_12_FLOOR_IMAGE ?= grafana/grafana:12.0.10", makefile)
        self.assertIn("GRAFANA_12_CURRENT_IMAGE ?= grafana/grafana:12.4.9", makefile)
        self.assertIn("GRAFANA_13_CURRENT_IMAGE ?= grafana/grafana:13.2.0", makefile)
        self.assertIn("test-rust-live-12-floor:", makefile)
        self.assertIn("test-rust-live-12-current:", makefile)
        self.assertIn("test-rust-live-13-current:", makefile)
        self.assertIn("test-python-datasource-live-12-floor:", makefile)
        self.assertIn("test-python-datasource-live-13-current:", makefile)
        self.assertIn("bash ./scripts/test-python-datasource-live-grafana.sh", makefile)
        self.assertIn("poetry run python -m grafana_utils datasource", PYTHON_DATASOURCE_LIVE_SMOKE_PATH.read_text(encoding="utf-8"))
        self.assertIn("rust-live-compatibility:", workflow)
        self.assertIn("python-datasource-live-compatibility:", workflow)
        self.assertIn("bash ./scripts/test-python-datasource-live-grafana.sh", workflow)
        self.assertIn("grafana/grafana:12.0.10", workflow)
        self.assertIn("grafana/grafana:12.4.9", workflow)
        self.assertIn("grafana/grafana:13.2.0", workflow)

    def test_developer_guide_declares_grafana_12_compatibility_floor(self):
        content = DEVELOPER_GUIDE_PATH.read_text(encoding="utf-8")

        self.assertIn("Grafana 12.0+", content)
        self.assertIn("404", content)
        self.assertIn("capability", content.lower())

    def test_rust_feature_matrix_probes_no_default_features_as_a_verification_surface(self):
        content = (REPO_ROOT / "scripts" / "check_rust_feature_matrix.py").read_text(
            encoding="utf-8"
        )

        self.assertIn("no-default-features verification check: ok", content)
        self.assertNotIn("probe unexpectedly succeeded", content)

    def test_grafana_13_org_delete_regression_is_scoped_to_missing_org_matrix_cases(self):
        content = RUST_LIVE_SMOKE_PATH.read_text(encoding="utf-8")

        self.assertIn("ORG_DELETE_UNAVAILABLE", content)
        self.assertIn("Grafana 13 organization deletion regression", content)
        self.assertIn("skip access org delete/replay smoke", content)
        self.assertIn("skip missing-org datasource matrix", content)
        self.assertIn("skip missing-org dashboard matrix", content)
        self.assertIn('if [[ "${ORG_DELETE_UNAVAILABLE}" -eq 1 ]]; then', content)
        self.assertNotIn(
            'if [[ "${GRAFANA_VERSION}" == 13.* ]]; then\n'
            "    printf '%s\\n' 'Grafana 13 organization deletion regression: skip access org delete/replay smoke.'",
            content,
        )
        python_content = PYTHON_DATASOURCE_LIVE_SMOKE_PATH.read_text(encoding="utf-8")
        self.assertIn("ORG_DELETE_UNAVAILABLE", python_content)
        self.assertIn("skip missing-org datasource matrix", python_content)


if __name__ == "__main__":
    unittest.main()
