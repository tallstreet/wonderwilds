import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dataset
from review_server import ReviewState


class ReviewTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "progress.json"
        self.state = ReviewState(dataset.DEFAULT, self.path)
        self.addCleanup(lambda: self.state.close())
        self.item = self.state.view()["items"][0]

    def decision(self, choice="approve"):
        return {k: self.item[k] for k in ("scenarioId", "candidateId", "candidateSha256")} | {
            "decision": choice, "reviewer": "Test reviewer", "notes": "Clear and accurate",
            "suggestedText": ""}

    def test_grades_are_not_sent_or_available_before_decision(self):
        payload = self.state.view()
        self.assertNotIn("judgment", json.dumps(payload))
        self.assertNotIn("recommendation", json.dumps(payload))
        key = (self.item["scenarioId"], self.item["candidateId"])
        with self.assertRaises(PermissionError):
            self.state.grade(key)
        self.state.save(self.decision())
        self.assertIn("judgment", self.state.grade(key))

    def test_decisions_survive_restart_and_record_first_decision(self):
        self.state.save(self.decision("reject"))
        last = self.state.save(self.decision("approve"))
        self.assertEqual(last["firstDecision"], "reject")
        self.state.close()
        self.state = ReviewState(dataset.DEFAULT, self.path)
        saved = self.state.view()["items"][0]["review"]
        self.assertEqual(saved["decision"], "approve")
        self.assertEqual(saved["firstDecision"], "reject")

    def test_disk_failure_does_not_claim_success_or_reveal_grades(self):
        key = (self.item["scenarioId"], self.item["candidateId"])
        with patch.object(self.state, "persist", side_effect=OSError("disk full")):
            with self.assertRaises(OSError):
                self.state.save(self.decision())
        with self.assertRaises(PermissionError):
            self.state.grade(key)
        self.assertFalse(self.path.exists())

    def test_stale_hash_and_edited_approval_are_rejected(self):
        value = self.decision()
        value["candidateSha256"] = "stale"
        with self.assertRaisesRegex(ValueError, "Candidate changed"):
            self.state.save(value)
        value = self.decision()
        value["suggestedText"] = "Changed prose"
        with self.assertRaisesRegex(ValueError, "Suggested edits"):
            self.state.save(value)

    def test_revision_notes_are_required_and_preserved(self):
        value = self.decision("revise")
        value["notes"] = ""
        with self.assertRaises(ValueError):
            self.state.save(value)
        value.update(notes="Use concrete language", suggestedText="The bridge is ready.")
        self.assertEqual(self.state.save(value)["suggestedText"], "The bridge is ready.")

    def test_another_server_cannot_write_same_file(self):
        with self.assertRaisesRegex(ValueError, "already open"):
            ReviewState(dataset.DEFAULT, self.path)

    def test_progress_cannot_move_silently_to_changed_dataset(self):
        self.state.save(self.decision())
        self.state.close()
        value = json.loads(self.path.read_text())
        value["datasetSha256"] = "different"
        self.path.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, "different dataset"):
            ReviewState(dataset.DEFAULT, self.path)
