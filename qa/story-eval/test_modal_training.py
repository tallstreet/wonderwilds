"""Local regressions for training data boundaries; no Modal account or GPU."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("glimmer_data", Path(__file__).parent / "modal/data.py")
data = importlib.util.module_from_spec(spec)
spec.loader.exec_module(data)


class Tokenizer:
    def apply_chat_template(self, messages, tokenize, add_generation_prompt, return_dict=False):
        return [1, 2] if len(messages) == 2 else [1, 2, 3, 4]


class TrainingTests(unittest.TestCase):
    def test_loss_masks_prompt_and_keeps_answer_terminator(self):
        result = data.tokenize({"messages": [{}, {}, {}]}, Tokenizer(), 4)
        self.assertEqual(result["labels"], [-100, -100, 3, 4])
        self.assertEqual(result["attention_mask"], [1, 1, 1, 1])

    def test_overlong_targets_fail_instead_of_truncating(self):
        with self.assertRaisesRegex(ValueError, "no silent truncation"):
            data.tokenize({"messages": [{}, {}, {}]}, Tokenizer(), 3)

    def test_unstable_template_fails(self):
        class Broken(Tokenizer):
            def apply_chat_template(self, messages, tokenize, add_generation_prompt, return_dict=False):
                return [9] if len(messages) == 2 else [1, 2]
        with self.assertRaisesRegex(ValueError, "prefix-stable"):
            data.tokenize({"messages": [{}, {}, {}]}, Broken(), 5)

    def test_journey_split_overlap_removal_and_tamper_detection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            export = root / "export"
            export.mkdir()
            families = sorted(["a", "b", "c"], key=lambda x: data.sha(x.encode()))
            corpus, provenance, train, test = [], [], [], []
            for i, (family, situation) in enumerate([
                (families[0], "shared"), (families[1], "shared"),
                (families[1], "unique"), (families[2], "other"), ("held-out", "test"),
            ]):
                split = "eval" if family == "held-out" else "train"
                output = test if split == "eval" else train
                output.append({"messages": [
                    {"role": "system", "content": "scene"},
                    {"role": "user", "content": str(i)},
                    {"role": "assistant", "content": '{}'},
                ]})
                corpus.append({"scenarioId": str(i), "familyId": family,
                               "input": situation, "presentation": []})
                provenance.append({"split": split, "row": len(output), "familyId": family,
                                   "review": {"scenarioId": str(i), "decision": "approve"}})
            for name, rows in (("train", train), ("eval", test), ("provenance", provenance)):
                (export / f"{name}.jsonl").write_bytes(data.encode(rows))
            (export / "manifest.json").write_text('{}')
            (root / "corpus.jsonl").write_bytes(data.encode(corpus))
            result = data.prepare(export, root / "corpus.jsonl", root / "bundle")
            self.assertEqual(result["counts"], {"train": 2, "validation": 1, "test": 1})
            self.assertEqual(result["excludedForValidationOverlapOrDuplicate"], 1)
            self.assertEqual(result["validationFamilies"], [families[0]])
            data.verify(root / "bundle")
            with self.assertRaises(FileExistsError):
                data.prepare(export, root / "corpus.jsonl", root / "bundle")
            (root / "bundle/train.jsonl").write_text('{}\n')
            with self.assertRaisesRegex(ValueError, "changed"):
                data.verify(root / "bundle")


if __name__ == "__main__":
    unittest.main()
