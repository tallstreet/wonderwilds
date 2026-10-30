import json
from pathlib import Path
import shutil
import sys
from tempfile import TemporaryDirectory
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dataset


class DatasetTests(unittest.TestCase):
    def test_archived_pilot_and_shortlists_are_complete(self):
        summary, selected, _ = dataset.verify(dataset.DEFAULT)
        self.assertEqual(summary["candidates"], 411)
        self.assertEqual(summary["judged"], 411)
        self.assertEqual(summary["recommendations"], {"keep": 386, "revise": 25})
        self.assertEqual({k: len(v) for k, v in selected.items()}, {"train": 96, "eval": 40})
        train_ids = {r["scenarioId"] for r in selected["train"]}
        self.assertNotIn("bank-series-000", train_ids)
        self.assertTrue(train_ids.isdisjoint({r["scenarioId"] for r in selected["eval"]}))
        self.assertFalse(summary["humanApproved"])

    def test_changed_bytes_in_frozen_dataset_are_rejected(self):
        with TemporaryDirectory() as temp:
            path = Path(temp) / "copy"
            shutil.copytree(dataset.DEFAULT, path)
            with (path / "candidates.jsonl").open("a") as target:
                target.write("\n")
            with self.assertRaisesRegex(ValueError, "files changed"):
                dataset.verify(path)

    def test_changed_candidate_cannot_reuse_old_judgment(self):
        with TemporaryDirectory() as temp:
            path = Path(temp) / "copy"
            shutil.copytree(dataset.DEFAULT, path)
            rows = dataset.workflow.rows(path / "candidates.jsonl")
            rows[0]["output"]["presentation"][0]["text"] = "Different scene."
            (path / "candidates.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
            with self.assertRaisesRegex(ValueError, "stale judgment"):
                dataset.inspect(path)

    def test_duplicate_preference_is_rejected(self):
        with TemporaryDirectory() as temp:
            path = Path(temp) / "copy"
            shutil.copytree(dataset.DEFAULT, path)
            rows = dataset.workflow.rows(path / "recommendations.jsonl")
            first = rows[0]["scenarioId"]
            for row in rows:
                if row["scenarioId"] == first:
                    row["preferredForScenario"] = True
            (path / "recommendations.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
            with self.assertRaisesRegex(ValueError, "at most one"):
                dataset.inspect(path)


if __name__ == "__main__":
    unittest.main()
