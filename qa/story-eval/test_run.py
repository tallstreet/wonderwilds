import copy
import importlib.util
import json
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("story_eval", Path(__file__).with_name("run.py"))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class WorkflowTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.run = self.root / "run"
        self.corpus = [{"scenarioId": f"{family}-1", "familyId": family,
                        "input": {"context": {"family": family}}, "presentation": {"sceneId": "scene", "requiredEntries": []},
                        "reference": {"sceneId": "scene", "presentation": [], "choices": []}}
                       for family in ("bank", "workshop", "supplies")]

        def fake_rust(command, *args):
            options = dict(zip(args[::2], args[1::2]))
            if command == "export":
                runner.write_rows(options["--output"], self.corpus)
            else:
                candidates = runner.rows(options["--candidates"])
                runner.write_rows(options["--output"], [{"scenarioId": r["scenarioId"],
                    "candidateId": r["candidateId"], "passed": r["candidateId"] != "bad",
                    "failures": [] if r["candidateId"] != "bad" else ["unknown action"]}
                    for r in candidates])

        self.mock = patch.object(runner, "rust", side_effect=fake_rust)
        self.mock.start()
        self.addCleanup(self.mock.stop)
        runner.prepare(SimpleNamespace(out=self.run, candidates=2, brief="Ages 6–8; short, warm sentences."))

    def grade(self, candidates=None, name="graded", judgments=None):
        source = self.run / "reference-candidates.jsonl"
        if candidates is not None:
            source = self.root / f"{name}-candidates.jsonl"
            runner.write_rows(source, candidates)
        out = self.root / name
        runner.grade(SimpleNamespace(run=self.run, candidates=source, out=out, judgments=judgments))
        return out

    def approvals(self, graded):
        values = runner.rows(graded / "reviews-template.jsonl")
        for row in values:
            row.update(decision="approve", reviewer="test reviewer", notes="Reviewed scene and choices")
        path = self.root / "reviews.jsonl"
        runner.write_rows(path, values)
        return path

    def test_generation_requests_exclude_checkpoint_and_reference(self):
        requests = runner.rows(self.run / "generation-requests.jsonl")
        supplied = json.loads(requests[0]["messages"][1]["content"])
        self.assertEqual(set(supplied), {"input", "presentation", "outputSchema", "editorialBrief"})
        self.assertEqual(len(requests), 6)

    def test_whole_families_are_split_deterministically(self):
        extended = self.corpus + [{**self.corpus[0], "scenarioId": "bank-2"}]
        self.assertEqual(runner.split_families(self.corpus), runner.split_families(extended))
        self.assertEqual(set(runner.split_families(extended).values()), {"train", "eval"})

    def test_duplicate_prompt_is_excluded_from_training(self):
        identical = copy.deepcopy(self.corpus)
        for row in identical:
            row["input"] = {"same": "state"}
        splits = runner.split_families(identical)
        excluded = runner.training_overlap(identical, splits)
        self.assertEqual(len(excluded), 2)
        self.assertTrue(all(splits[row["familyId"]] == "train" for row in identical
                            if row["scenarioId"] in excluded))

    def test_malformed_provenance_is_cleanly_rejected(self):
        candidates = runner.rows(self.run / "reference-candidates.jsonl")
        candidates[0]["generator"] = []
        with self.assertRaisesRegex(ValueError, "generator"):
            self.grade(candidates)

    def test_human_approved_export_keeps_eval_out_of_training(self):
        graded = self.grade()
        out = self.root / "export"
        runner.export(SimpleNamespace(run=self.run, graded=graded, reviews=self.approvals(graded), out=out))
        self.assertEqual(len(runner.rows(out / "train.jsonl")), 2)
        self.assertEqual(len(runner.rows(out / "eval.jsonl")), 1)
        self.assertEqual(len(runner.rows(out / "provenance.jsonl")), 3)

    def test_pending_examples_are_not_exported(self):
        graded = self.grade()
        out = self.root / "export"
        runner.export(SimpleNamespace(run=self.run, graded=graded,
            reviews=graded / "reviews-template.jsonl", out=out))
        self.assertEqual(runner.rows(out / "train.jsonl"), [])

    def test_revisions_are_not_exported_and_edited_approvals_fail(self):
        graded = self.grade()
        reviews = runner.rows(graded / "reviews-template.jsonl")
        for row in reviews:
            row.update(decision="revise", reviewer="test", notes="Needs clearer prose", suggestedText="Replacement")
        path = self.root / "reviews.jsonl"
        runner.write_rows(path, reviews)
        runner.export(SimpleNamespace(run=self.run, graded=graded, reviews=path, out=self.root / "export"))
        self.assertEqual(runner.rows(self.root / "export/train.jsonl"), [])
        reviews[0]["decision"] = "approve"
        path.write_text("\n".join(map(json.dumps, reviews)) + "\n")
        with self.assertRaisesRegex(ValueError, "Suggested edits"):
            runner.export(SimpleNamespace(run=self.run, graded=graded, reviews=path, out=self.root / "export2"))

    def test_invalid_candidate_cannot_be_human_approved(self):
        candidates = runner.rows(self.run / "reference-candidates.jsonl")
        candidates[0]["candidateId"] = "bad"
        graded = self.grade(candidates)
        with self.assertRaisesRegex(ValueError, "structurally invalid"):
            runner.export(SimpleNamespace(run=self.run, graded=graded,
                reviews=self.approvals(graded), out=self.root / "export"))

    def test_stale_candidate_or_review_is_rejected(self):
        graded = self.grade()
        reviews = self.approvals(graded)
        changed = runner.rows(reviews)
        changed[0]["candidateSha256"] = "wrong"
        reviews.write_text("\n".join(map(json.dumps, changed)) + "\n")
        with self.assertRaisesRegex(ValueError, "stale review"):
            runner.export(SimpleNamespace(run=self.run, graded=graded, reviews=reviews, out=self.root / "export"))
        (graded / "candidates.jsonl").write_text("{}\n")
        with self.assertRaisesRegex(ValueError, "artifacts changed"):
            runner.export(SimpleNamespace(run=self.run, graded=graded, reviews=reviews, out=self.root / "export"))

    def test_review_html_renders_candidate_as_inert_text(self):
        candidates = runner.rows(self.run / "reference-candidates.jsonl")
        candidates[0]["output"]["presentation"] = [{"text": "</pre><script>alert(1)</script>"}]
        graded = self.grade(candidates)
        page = (graded / "review.html").read_text()
        self.assertNotIn("<script>", page)
        self.assertIn("&lt;script&gt;", page)
        self.assertIn("Content-Security-Policy", page)

    def test_duplicate_and_unknown_candidates_rejected(self):
        candidates = runner.rows(self.run / "reference-candidates.jsonl")
        with self.assertRaisesRegex(ValueError, "Duplicate"):
            self.grade(candidates + [copy.deepcopy(candidates[0])])
        candidates[0]["scenarioId"] = "unknown"
        with self.assertRaisesRegex(ValueError, "Unknown scenario"):
            self.grade(candidates, name="unknown")

    def test_judgment_requires_exact_candidate_and_complete_rubric(self):
        candidates = runner.rows(self.run / "reference-candidates.jsonl")
        item = candidates[0]
        key = (item["scenarioId"], item["candidateId"])
        judgment = {"scenarioId": key[0], "candidateId": key[1], "candidateSha256": runner.digest(item),
            "rubricVersion": runner.RUBRIC_VERSION, "judge": {"model": "test", "promptVersion": "test"},
            "criteria": {criterion: {"verdict": "uncertain", "evidence": "Needs human review"}
                         for criterion in runner.CRITERIA}}
        self.assertIn(key, runner.checked_judgments([judgment], {key: item}))
        del judgment["criteria"]["continuity"]
        with self.assertRaisesRegex(ValueError, "every rubric"):
            runner.checked_judgments([judgment], {key: item})

    def test_changed_requests_and_existing_output_directory_rejected(self):
        with self.assertRaises(FileExistsError):
            runner.prepare(SimpleNamespace(out=self.run, candidates=2, brief="test"))
        path = self.run / "generation-requests.jsonl"
        path.write_text("{}\n")
        with self.assertRaisesRegex(ValueError, "Run changed"):
            self.grade()


if __name__ == "__main__":
    unittest.main()
