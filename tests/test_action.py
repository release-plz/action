"""Exercise the action's Bash with a fake release-plz, without forge API calls."""

import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

import yaml


ACTION = yaml.safe_load((Path(__file__).resolve().parents[1] / "action.yml").read_text())
SCRIPT = next(step["run"] for step in ACTION["runs"]["steps"] if step.get("id") == "release-plz")


class ActionTests(unittest.TestCase):
    def run_action(self, inputs, server_url):
        values = {name: spec.get("default", "") for name, spec in ACTION["inputs"].items()}
        values.update(inputs)
        script = re.sub(r"\$\{\{ inputs\.(\w+) \}\}", lambda match: values[match[1]], SCRIPT)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "release-plz"
            executable.write_text(
                "#!/usr/bin/env python3\n"
                "import json, os, sys\n"
                "with open(os.environ['TEST_CALLS'], 'a') as calls:\n"
                "    calls.write(json.dumps(sys.argv[1:]) + '\\n')\n"
                "print(json.dumps({'prs': [], 'releases': []}))\n"
            )
            executable.chmod(0o755)
            env = {
                **os.environ,
                "PATH": f"{root}:{os.environ['PATH']}",
                "GITHUB_TOKEN": "test-token",
                "GITHUB_REPOSITORY": "owner/project",
                "GITHUB_OUTPUT": str(root / "output"),
                "TEST_CALLS": str(root / "calls"),
            }
            env.pop("GITHUB_SERVER_URL", None)
            if server_url is not None:
                env["GITHUB_SERVER_URL"] = server_url
            result = subprocess.run(
                ["bash", "--noprofile", "--norc", "-e", "-o", "pipefail", "-c", script],
                env=env, cwd=root, text=True, capture_output=True,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()]
            outputs = dict(line.split("=", 1) for line in (root / "output").read_text().splitlines())
            return calls, outputs

    def test_forge_selection_and_repository_url(self):
        cases = [
            ({}, "https://github.com", "github"),
            ({}, None, "github"),
            ({"forge": "gitea"}, "https://git.example.com", "gitea"),
            ({"backend": "gitea"}, "https://git.example.com:3000/gitea", "gitea"),
            ({"forge": "github", "backend": "gitea"}, "https://github.com", "github"),
        ]
        for inputs, server_url, forge in cases:
            with self.subTest(inputs=inputs, server_url=server_url):
                calls, outputs = self.run_action(inputs, server_url)
                self.assertEqual([call[0] for call in calls], ["release-pr", "release"])
                for call in calls:
                    self.assertEqual(call[call.index("--forge") + 1], forge)
                    self.assertEqual(call[call.index("--git-token") + 1], "test-token")
                self.assertEqual(
                    calls[0][calls[0].index("--repo-url") + 1],
                    f"{server_url or 'https://github.com'}/owner/project",
                )
                self.assertEqual(outputs, {
                    "prs": "[]", "pr": "{}", "prs_created": "false",
                    "releases": "[]", "releases_created": "false",
                })

    def test_individual_commands(self):
        for command in ("release-pr", "release"):
            with self.subTest(command=command):
                calls, _ = self.run_action({"forge": "gitea", "command": command}, "https://git.example.com")
                self.assertEqual([call[0] for call in calls], [command])


if __name__ == "__main__":
    unittest.main()
