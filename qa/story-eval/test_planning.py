"""Planning runner boundaries; Rust tests exercise actual completion search."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("mission_planning", Path(__file__).parent / "planning/run.py")
planning = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(planning)


class PlanningTests(unittest.TestCase):
    def test_strict_response_and_no_repair_or_fallback(self):
        def envelope(content, finish="stop"):
            return {"choices": [{"finish_reason": finish, "message": {"content": content}}]}
        self.assertEqual(planning.parse_response(envelope('{"goal":"hello"}')), {"goal": "hello"})
        for response in [envelope("```json\n{}\n```"), envelope("[]"), envelope("{}", "length"),
                         envelope("{broken}"), {"error": "failed"}]:
            with self.assertRaises(ValueError):
                planning.parse_response(response)

    def test_requests_exclude_checkpoint_reference_and_scene_targets(self):
        scenario = {"source": {"scenarioId": "synthetic", "checkpoint": {"secret": "NEVER_SEND"}},
                    "input": {"mode": "plan_mission"}, "reference": "REFERENCE_ANSWER",
                    "completedConditionIds": ["already_done"]}
        request = planning.request_for(scenario)
        serialized = json.dumps(request)
        self.assertNotIn("NEVER_SEND", serialized)
        self.assertNotIn("REFERENCE_ANSWER", serialized)
        supplied = json.loads(request["messages"][1]["content"])
        self.assertEqual(supplied["completedConditionIds"], ["already_done"])
        self.assertEqual(set(supplied["outputSchema"]["$defs"]), {"StoryNarrative"})

    def test_reversed_vocabulary_changes_request_without_changing_runtime_scenario(self):
        scenario = {"source": {"scenarioId": "case"}, "input": {"opportunities": [
            {"conditions": [{"id": "first"}, {"id": "second"}]}]}, "completedConditionIds": []}
        normal = planning.request_for(scenario)
        reversed_request = planning.request_for(scenario, "reversed")
        self.assertNotEqual(planning.digest(normal), planning.digest(reversed_request))
        self.assertNotEqual(normal["promptVersion"], reversed_request["promptVersion"])
        supplied = json.loads(reversed_request["messages"][1]["content"])
        self.assertEqual(supplied["input"]["opportunities"][0]["conditions"][0]["id"], "second")
        self.assertEqual(scenario["input"]["opportunities"][0]["conditions"][0]["id"], "first")

    def test_env_parsed_as_data_not_executed_and_not_returned_in_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "model.env"
            path.write_text("export MODAL_PROXY_TOKEN_ID='synthetic-id'\nMODAL_PROXY_TOKEN_SECRET='synthetic-secret' # comment\nUNRELATED=$(exit 2)\n")
            with patch.dict(planning.os.environ, {}, clear=True):
                self.assertEqual(planning.credentials(path), "Bearer synthetic-id.synthetic-secret")
                with self.assertRaises(ValueError):
                    planning.credentials(None)

    def test_manifest_drift_rejected_and_existing_files_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            planning.write_rows(path / "scenarios.jsonl", [])
            planning.write_rows(path / "requests.jsonl", [])
            manifest = {"version": planning.VERSION, "scenarioSha256": planning.digest([]),
                        "requestSha256": planning.digest([])}
            planning.write_json(path / "manifest.json", manifest)
            self.assertEqual(planning.verified_run(path)[0], [])
            with self.assertRaises(FileExistsError):
                planning.write_json(path / "manifest.json", {})
            (path / "scenarios.jsonl").write_text('{"modified":true}\n')
            with self.assertRaises(ValueError):
                planning.verified_run(path)

    def make_run(self, path):
        path.mkdir()
        scenarios = [{"source": {"scenarioId": sid}, "input": {"opportunities": [{"id": "test"}]},
                      "completedConditionIds": [], "reference": {}} for sid in ("a", "b")]
        requests = [planning.request_for(s) for s in scenarios]
        planning.write_rows(path / "scenarios.jsonl", scenarios)
        planning.write_rows(path / "requests.jsonl", requests)
        planning.write_json(path / "manifest.json", {"version": planning.VERSION,
            "scenarioSha256": planning.digest(scenarios), "requestSha256": planning.digest(requests),
            "scope": "synthetic fixture"})

    def test_transport_failures_are_preserved_without_secret_exception_text(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            self.make_run(path / "run")
            with patch.object(planning, "credentials", return_value="SENSITIVE"), \
                 patch.object(planning.urllib.request, "build_opener") as opener:
                opener.return_value.open.side_effect = OSError("SENSITIVE")
                planning.generate(SimpleNamespace(run=path / "run", out=path / "out", env_file=None))
            self.assertEqual(len(planning.rows(path / "out/candidates.jsonl")), 2)
            for p in (path / "out").iterdir():
                self.assertNotIn("SENSITIVE", p.read_text())
            self.assertEqual(planning.rows(path / "out/candidates.jsonl")[0]["output"], "generation_failed:OSError")

    def test_partial_coverage_and_html_escaping(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            self.make_run(path / "run")
            planning.write_rows(path / "candidates.jsonl", [{"scenarioId": "a", "candidateId": "test", "output": "<script>bad</script>"}])
            def fake_rust(*args):
                planning.write_rows(args[-1], [{"scenarioId": "a", "candidateId": "test", "passed": False,
                    "failures": ["schema: wrong type"], "witness": [], "exploredStates": 0}])
            with patch.object(planning, "rust", side_effect=fake_rust):
                planning.grade(SimpleNamespace(run=path / "run", candidates=path / "candidates.jsonl", out=path / "out"))
            summary = planning.read_json(path / "out/summary.json")
            self.assertEqual((summary["total"], summary["received"], summary["missing"], summary["passed"]), (2, 1, 1, 0))
            review = (path / "out/review.html").read_text()
            self.assertNotIn("<script>bad</script>", review)
            self.assertIn("&lt;script&gt;bad&lt;/script&gt;", review)

    def test_no_credential_forwarding_on_redirect(self):
        self.assertIsNone(planning.NoRedirect().redirect_request(None, None, 302, "", {}, "https://other.invalid"))


if __name__ == "__main__":
    unittest.main()
