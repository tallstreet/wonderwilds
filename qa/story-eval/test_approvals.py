import hashlib
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dataset
import run as workflow


class ApprovalTests(unittest.TestCase):
    def test_user_acceptance_is_complete_bound_and_preserves_prior_decisions(self):
        root = Path(__file__).resolve().parent / "approvals/pilot-2026-10-30-user-acceptance"
        record = workflow.read_json(root / "acceptance.json")
        reviews = workflow.indexed(workflow.rows(root / "human-reviews.jsonl"), ("scenarioId", "candidateId"))
        candidates = workflow.indexed(workflow.rows(dataset.DEFAULT / "candidates.jsonl"), ("scenarioId", "candidateId"))
        recommendations = workflow.indexed(workflow.rows(dataset.DEFAULT / "recommendations.jsonl"), ("scenarioId", "candidateId"))
        self.assertEqual(set(reviews), set(candidates))
        self.assertEqual(record["reviewsSha256"], hashlib.sha256((root / "human-reviews.jsonl").read_bytes()).hexdigest())
        self.assertEqual(record["recommendationsSha256"], hashlib.sha256((dataset.DEFAULT / "recommendations.jsonl").read_bytes()).hexdigest())
        self.assertEqual(record["datasetSha256"], workflow.digest(workflow.read_json(dataset.DEFAULT / "snapshot.json")))
        for key, review in reviews.items():
            self.assertEqual(review["candidateSha256"], workflow.digest(candidates[key]))
            self.assertEqual(review["decision"], {"keep": "approve", "revise": "revise", "reject": "reject"}[recommendations[key]["recommendation"]])
        self.assertEqual(sum(r["decision"] == "approve" for r in reviews.values()), 386)
        prior = workflow.rows(root / "prior-decisions.jsonl")
        self.assertEqual(len(prior), 17)
        for review in prior:
            self.assertEqual(review, reviews[review["scenarioId"], review["candidateId"]])


if __name__ == "__main__":
    unittest.main()
