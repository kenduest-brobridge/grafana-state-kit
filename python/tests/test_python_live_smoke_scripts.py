"""Exercise live-smoke shell behavior without starting Docker or Grafana."""

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_SCRIPT = REPO_ROOT / "scripts/test-rust-live-grafana.sh"
PYTHON_SCRIPT = REPO_ROOT / "scripts/test-python-datasource-live-grafana.sh"


@unittest.skipUnless(
    shutil.which("bash") and shutil.which("jq"), "requires bash and jq"
)
class LiveSmokeScriptTests(unittest.TestCase):
    def run_shell(self, path, body, **variables):
        # Load the real shell functions, replacing only the external dependencies.
        source = path.read_text(encoding="utf-8").rsplit('\nmain "$@"', 1)[0]
        source = source.replace("${BASH_SOURCE[0]}", "${TEST_SCRIPT_PATH}")
        with tempfile.TemporaryDirectory() as temporary_dir:
            return subprocess.run(
                ["bash", "-c", source + "\n" + body, str(path)],
                env={
                    **os.environ,
                    "TMPDIR": temporary_dir,
                    "TEST_SCRIPT_PATH": str(path),
                    **variables,
                },
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
            )

    def test_rust_smoke_uses_the_binary_built_in_the_selected_target_directory(self):
        result = self.run_shell(
            RUST_SCRIPT,
            """
cargo() {
  printf '%s\n' '{"reason":"compiler-artifact","target":{"name":"grafana-util"},"executable":"/tmp/review-target/debug/grafana-util"}'
}
build_rust_bins >/dev/null
dashboard_bin; alert_bin; datasource_bin; access_bin; sync_bin
""",
            CARGO_TARGET_DIR="/tmp/review-target",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            result.stdout.splitlines(), ["/tmp/review-target/debug/grafana-util"] * 5
        )

    def test_rust_smoke_cleans_its_container_and_workdir_on_success_and_failure(self):
        for exit_code in (0, 7):
            with self.subTest(exit_code=exit_code):
                result = self.run_shell(
                    RUST_SCRIPT,
                    """
docker() { printf '%s\n' "$*" > "${TMPDIR}/docker-cleanup"; }
build_rust_bins() { :; }
start_grafana() { printf 'review-created-id\n' > "${CONTAINER_ID_FILE}"; }
seed_contact_point() { :; }
create_api_token() { :; }
run_alert_artifact_smoke() { return "$TEST_EXIT"; }
cleanup_original="$(declare -f cleanup)"
eval "${cleanup_original/cleanup ()/original_cleanup ()}"
cleanup() {
  original_cleanup
  [[ ! -d "$WORK_DIR" ]] && printf 'workdir removed\n'
  [[ "$(<"${TMPDIR}/docker-cleanup")" == "rm -f review-created-id" ]] && printf 'container removed\n'
}
main
""",
                    TEST_EXIT=str(exit_code),
                    RUST_LIVE_SCOPE="alert-artifact",
                )
                self.assertEqual(result.returncode, exit_code, result.stderr)
                self.assertIn("workdir removed", result.stdout)
                self.assertIn("container removed", result.stdout)

    def test_smoke_cleans_a_created_container_when_docker_start_fails(self):
        for path in (RUST_SCRIPT, PYTHON_SCRIPT):
            with self.subTest(script=path.name):
                result = self.run_shell(
                    path,
                    """
docker() {
  if [[ "$1" == rm ]]; then
    printf '%s\n' "$*" > "${TMPDIR}/docker-cleanup"
    return
  fi
  while [[ $# -gt 0 ]]; do
    if [[ "$1" == --cidfile ]]; then
      printf 'review-created-id\n' > "$2"
      shift
    fi
    shift
  done
  return 125
}
build_rust_bins() { :; }
check_requested_grafana_port() { :; }
cleanup_original="$(declare -f cleanup)"
eval "${cleanup_original/cleanup ()/original_cleanup ()}"
cleanup() {
  original_cleanup
  printf 'cleanup=%s\n' "$(<"${TMPDIR}/docker-cleanup")"
  [[ ! -d "$WORK_DIR" ]] && printf 'workdir removed\n'
}
main
""",
                    PYTHON_BIN="bash",
                )
                self.assertEqual(result.returncode, 125, result.stderr)
                self.assertIn("cleanup=rm -f review-created-id", result.stdout)
                self.assertIn("workdir removed", result.stdout)

    def test_org_delete_skips_only_the_recorded_server_failure(self):
        known_body = '{"message":"Failed to delete organization"}'
        cases = [
            ("13.2.0", "500", known_body, 0, True),
            (
                "13.2.0",
                "500",
                '{ "message": "Failed to delete organization" }',
                0,
                True,
            ),
            ("13.2.0", "200", '{"message":"Organization deleted"}', 0, False),
            ("13.2.0", "401", known_body, 1, False),
            ("13.2.0", "403", known_body, 1, False),
            ("13.2.0", "503", known_body, 1, False),
            ("13.2.0", "500", '{"message":"Other failure"}', 1, False),
            ("13.2.0", "500", known_body + "garbage", 1, False),
            ("12.0.10", "500", known_body, 1, False),
            ("13.3.0", "500", known_body, 1, False),
        ]
        for path in (RUST_SCRIPT, PYTHON_SCRIPT):
            for version, status, body, exit_code, skipped in cases:
                with self.subTest(
                    script=path.name, version=version, status=status, body=body
                ):
                    result = self.run_shell(
                        path,
                        """
curl() {
  local output="" write_out=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --output|-o) output="$2"; shift ;;
      --write-out|-w) write_out="$2"; shift ;;
    esac
    shift
  done
  if [[ -n "$output" ]]; then
    printf '%s' "$TEST_BODY" > "$output"
  else
    printf '%s' "$TEST_BODY"
  fi
  [[ -z "$write_out" ]] || printf '%s' "$TEST_STATUS"
  [[ "$TEST_STATUS" == 2* ]] || return 22
}
GRAFANA_VERSION="$TEST_VERSION"
delete_org 2
printf 'skip=%s\n' "$ORG_DELETE_UNAVAILABLE"
""",
                        TEST_VERSION=version,
                        TEST_STATUS=status,
                        TEST_BODY=body,
                    )
                    self.assertEqual(result.returncode, exit_code, result.stderr)
                    if exit_code == 0:
                        self.assertIn("skip=%s" % int(skipped), result.stdout)
