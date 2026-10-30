"""Release policy regression checks. Run with Python 3 and PyYAML 6.0.3."""

from pathlib import Path
import os
import subprocess
import tempfile
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[1]


class ReleasePolicy(unittest.TestCase):
    def setUp(self):
        # BaseLoader preserves GitHub's `on` key instead of YAML 1.1 booleans.
        self.workflow = yaml.load(
            (ROOT / ".github/workflows/release.yml").read_text(),
            Loader=yaml.BaseLoader,
        )
        self.job = self.workflow["jobs"]["publish"]
        self.steps = self.job["steps"]

    def test_release_events_and_validation_gate(self):
        self.assertEqual(
            self.workflow["on"]["release"]["types"], ["published"]
        )
        self.assertIn("workflow_dispatch", self.workflow["on"])
        self.assertEqual(self.job["needs"], "checks")
        self.assertEqual(
            self.workflow["jobs"]["checks"]["uses"], "./.github/workflows/ci.yml"
        )
        self.assertIn("refs/heads/main", self.job["if"])
        self.assertIn("github.event.release.target_commitish == 'main'", self.job["if"])

    def test_oidc_precedes_publish_and_can_fall_back(self):
        auth = next(s for s in self.steps if s.get("id") == "crates_auth")
        publish = next(s for s in self.steps if s.get("name") == "Publish")
        self.assertLess(self.steps.index(auth), self.steps.index(publish))
        self.assertEqual(auth["continue-on-error"], "true")
        action, revision = auth["uses"].split("@")
        self.assertEqual(action, "rust-lang/crates-io-auth-action")
        self.assertRegex(revision, r"^[0-9a-f]{40}$")
        self.assertEqual(self.job["permissions"]["id-token"], "write")
        self.assertEqual(
            publish["env"]["CARGO_REGISTRY_TOKEN"],
            "${{ steps.crates_auth.outputs.token || secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}",
        )
        self.assertIn("cargo publish --locked", publish["run"])
        self.assertNotIn("--no-verify", publish["run"])

    def test_credentials_only_available_to_publish_job(self):
        self.assertEqual(self.workflow["permissions"], {"contents": "read"})
        self.assertNotIn("pull_request", self.workflow["on"])
        self.assertNotIn("pull_request_target", self.workflow["on"])
        self.assertEqual(self.job["concurrency"]["cancel-in-progress"], "false")

    def test_publish_stops_without_credentials(self):
        result, invoked = self.run_publish(token="")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Bootstrap the crate", result.stdout)
        self.assertFalse(invoked)

    def test_publish_uses_cargo_and_propagates_failures(self):
        for cargo_exit in (0, 42):
            with self.subTest(cargo_exit=cargo_exit):
                result, invoked = self.run_publish("test-only-token", cargo_exit)
                self.assertEqual(result.returncode, cargo_exit)
                self.assertEqual(invoked, "publish --locked\n")
                self.assertNotIn("test-only-token", result.stdout + result.stderr)

    def run_publish(self, token, cargo_exit=0):
        publish = next(s for s in self.steps if s.get("name") == "Publish")
        with tempfile.TemporaryDirectory() as directory:
            cargo = Path(directory) / "cargo"
            cargo.write_text(
                '#!/bin/sh\nprintf "%s\\n" "$*" > "$MOCK_CARGO_LOG"\n'
                'exit "$MOCK_CARGO_EXIT"\n'
            )
            cargo.chmod(0o755)
            log = Path(directory) / "cargo.log"
            result = subprocess.run(
                ["bash", "-e", "-c", publish["run"]],
                env={
                    **os.environ,
                    "PATH": directory + os.pathsep + os.environ["PATH"],
                    "CARGO_REGISTRY_TOKEN": token,
                    "MOCK_CARGO_LOG": str(log),
                    "MOCK_CARGO_EXIT": str(cargo_exit),
                },
                capture_output=True,
                text=True,
                timeout=10,
            )
            return result, log.read_text() if log.exists() else ""


if __name__ == "__main__":
    unittest.main()
