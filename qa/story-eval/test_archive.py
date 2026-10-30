from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
import json
import shutil
from unittest.mock import patch

import dataset
import run


class HistoricalReviewTests(unittest.TestCase):
    def test_broader_input_pack_is_separate_and_detects_tampering(self):
        source = dataset.DEFAULT.parent / "broader-2026-10-30-inputs"
        counts = dataset.verify_inputs(source)
        self.assertEqual(counts["scenarios"], 607)
        self.assertEqual(counts["generationRequests"], 1821)
        self.assertEqual(counts["humanApprovals"], 0)
        with TemporaryDirectory() as folder:
            copied = Path(folder) / "pack"
            shutil.copytree(source, copied)
            manifest = run.read_json(copied / "input-pack.json")
            manifest["coverage"]["lastingChoiceCombinations"] = 1
            (copied / "input-pack.json").write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError, "coverage"):
                dataset.verify_inputs(copied)
            (copied / "reference-hard-results.jsonl").write_text('{}\n')
            with self.assertRaisesRegex(ValueError, "changed"):
                dataset.verify_inputs(copied)

    def test_parent_link_is_verified_and_does_not_transfer_approvals(self):
        source = run.rows(dataset.DEFAULT / "run/corpus.jsonl")[:2]
        # Two families are needed for a train/eval split.
        source[1]["familyId"] = "other-family"
        def capture(*args):
            run.write_rows(args[-1], source)
        with TemporaryDirectory() as folder:
            out = Path(folder) / "run"
            with patch.object(run, "rust", side_effect=capture):
                run.prepare(SimpleNamespace(out=out, candidates=1, brief="Test brief",
                                            parent_dataset=dataset.DEFAULT))
            manifest = run.read_json(out / "manifest.json")
            self.assertEqual(manifest["parentDataset"]["datasetId"], dataset.DEFAULT.name)
            self.assertFalse(manifest["parentDataset"]["approvalsTransferred"])
            self.assertFalse((out / "human-reviews.jsonl").exists())

    def test_archived_review_keeps_original_evidence_explicit_and_does_not_call_runtime(self):
        before = dataset.checksums(dataset.DEFAULT)
        with TemporaryDirectory() as folder:
            out = Path(folder) / "review"
            with patch.object(run, "rust", side_effect=AssertionError("Historical review must not call current runtime")):
                dataset.review(SimpleNamespace(dataset=dataset.DEFAULT, out=out, archived=True))
            manifest = run.read_json(out / "manifest.json")
            self.assertEqual(manifest["validationMode"], "archived")
            self.assertEqual(manifest["candidateCount"], 411)
            self.assertIn("not rerun under the current runtime", (out / "review.html").read_text())
            self.assertEqual(len(run.rows(out / "reviews-template.jsonl")), 411)
            self.assertTrue(all(row["decision"] == "pending" for row in run.rows(out / "reviews-template.jsonl")))
        self.assertEqual(before, dataset.checksums(dataset.DEFAULT))


if __name__ == "__main__":
    unittest.main()
