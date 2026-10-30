"""Offline regression coverage for prompt-only evaluation and paired accounting."""
import importlib.util
import json
from pathlib import Path
import sys
import subprocess
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest.mock import patch

MODAL_DIR = Path(__file__).parent / "modal"
sys.path.insert(0, str(MODAL_DIR))
import eval_io
from data import encode, prepare, sha


class EvaluationTests(unittest.TestCase):
    def test_remote_entrypoint_imports_without_repository_helpers(self):
        # Reproduce Modal's /root layout: evaluate.py alone, no train/eval_io.
        with TemporaryDirectory() as temporary:
            entrypoint = Path(temporary) / "evaluate.py"
            entrypoint.write_bytes((MODAL_DIR / "evaluate.py").read_bytes())
            script = '''
import runpy, sys
from types import SimpleNamespace
class App:
    def __init__(self, *args): pass
    def function(self, **kwargs): return lambda f: f
    def local_entrypoint(self): return lambda f: f
sys.modules['modal'] = SimpleNamespace(
    is_local=lambda: False, App=App,
    Volume=SimpleNamespace(from_name=lambda name: SimpleNamespace(commit=lambda: None)))
module = runpy.run_path(sys.argv[1], run_name='remote_evaluate')
assert callable(module['infer'])
assert module['image'] is None
assert not {'train', 'eval_io', 'data', 'worker'} & sys.modules.keys()
'''
            subprocess.run([sys.executable, "-I", "-c", script, str(entrypoint)],
                           cwd=temporary, check=True, capture_output=True, text=True)

    def test_speaker_prompt_uses_only_present_characters_and_preserves_source(self):
        for present in ([], [{"id": "pip"}]):
            payload = {"input": {"here": {"characters": present},
                                 "mission": {"participantIds": ["pip", "woodcutter"]},
                                 "memory": {"facts": ["Bramble visited."]}},
                       "presentation": {"requiredEntries": [{"text": "Keep me"}]}}
            request = {"scenarioId": "case", "promptVersion": "v1", "messages": [
                {"role": "system", "content": "Original rules"},
                {"role": "user", "content": json.dumps(payload)}]}
            before = json.dumps(request)
            changed = eval_io.speaker_prompt(request)
            self.assertEqual(json.dumps(request), before)
            transformed = json.loads(changed['messages'][1]['content'])
            self.assertEqual(transformed.pop('allowedSpeakerIds'), [p['id'] for p in present])
            self.assertEqual(transformed, payload)
            self.assertEqual(changed['promptVersion'], 'v1+speakers-v1')
            self.assertIn('use narration only', changed['messages'][0]['content'])
            self.assertEqual(changed, eval_io.speaker_prompt(request))

    def test_unique_variant_retains_speakers_and_required_entries(self):
        request = {"scenarioId": "case", "promptVersion": "v1", "messages": [
            {"role": "system", "content": "Original rules"},
            {"role": "user", "content": json.dumps({
                "input": {"here": {"characters": [{"id": "pip"}]}},
                "presentation": {"requiredEntries": [
                    {"entryId": "intro", "type": "dialogue", "speakerId": "pip", "text": "Hello"}]}})}]}
        original = json.dumps(request)
        speakers = eval_io.speaker_prompt(request)
        unique = eval_io.speaker_prompt(request, 'speakers-unique-v1')
        self.assertEqual(json.dumps(request), original)
        self.assertEqual(unique['messages'][1], speakers['messages'][1])
        self.assertEqual(unique['messages'][0]['content'],
                         speakers['messages'][0]['content'] + '\n\n' + eval_io.UNIQUE_ENTRY_RULE)
        self.assertEqual(unique['promptVersion'], 'v1+speakers-unique-v1')
        self.assertEqual(unique, eval_io.speaker_prompt(request, 'speakers-unique-v1'))
        with self.assertRaises(ValueError):
            eval_io.speaker_prompt(request, 'typo')

    def test_harmony_final_response_and_strict_failure_handling(self):
        scene = '{"sceneId":"s","choices":[],"presentation":[]}'
        self.assertEqual(eval_io.parse_response(' to=user<|message|>' + scene + '<|eot|>'), json.loads(scene))
        raw = ' to=self<|message|>Thinking<|eom|><|start|>assistant to=user<|message|>' + scene + '<|eot|>'
        self.assertEqual(eval_io.parse_response(raw), json.loads(scene))
        for raw in ('not json', '```json\n' + scene + '\n```', '[1,2]', scene + ' trailing'):
            self.assertIsInstance(eval_io.parse_response(raw), str)
        self.assertIsInstance(eval_io.parse_response(scene, truncated=True), str)
        self.assertIsInstance(eval_io.parse_response(scene, error='OOM'), str)

    def suite(self):
        return {"requests": [{"scenarioId": s, "promptVersion": "v1"} for s in ("a", "b", "c")],
                "settings": {"maxNewTokens": 2048}, "split": "validation",
                "gradingCorpusSha256": "corpus", "gradingRequestsSha256": "requests"}

    def result(self, suite, adapter=""):
        return {"suiteSha256": sha(encode([suite])), "adapterRun": adapter,
                "runtime": {"gpu": "H100"}, "responses": [
                    {"scenarioId": r["scenarioId"], "raw": '{}', "error": None,
                     "truncated": False, "seconds": 1., "outputTokens": 10} for r in suite["requests"]]}

    def test_missing_duplicate_and_mismatched_results_fail(self):
        suite = self.suite()
        for mutation in (lambda r: r['responses'].pop(),
                         lambda r: r['responses'].append(r['responses'][0]),
                         lambda r: r.update(suiteSha256='changed')):
            result = self.result(suite)
            mutation(result)
            with self.assertRaises(ValueError):
                eval_io.candidates(suite, result)

    def test_run_ids_cannot_escape_volume(self):
        for name in ('../adapter', '/tmp/adapter', '', 'a/b'):
            with self.assertRaises(ValueError):
                eval_io.identifier(name)
        self.assertEqual(eval_io.identifier('full-123'), 'full-123')

    def test_comparison_keeps_failures_in_denominator_and_rejects_drift(self):
        suite = self.suite()
        w = eval_io.workflow()
        with TemporaryDirectory() as temporary:
            paths = [Path(temporary) / n for n in ('base', 'adapter')]
            for i, path in enumerate(paths):
                (path / 'graded').mkdir(parents=True)
                result = self.result(suite, 'full-123' if i else '')
                result['responses'][2].update(error='OOM', seconds=None)
                rows = eval_io.candidates(suite, result)
                hard = [{"scenarioId": r["scenarioId"], "candidateId": r["candidateId"],
                         "passed": (r["scenarioId"] == ('a' if i == 0 else 'b'))} for r in rows]
                (path / 'suite.json').write_text(json.dumps(suite))
                (path / 'result.json').write_text(json.dumps(result))
                for name, values in (('candidates', rows), ('hard-results', hard), ('judgments', [])):
                    (path / 'graded' / f'{name}.jsonl').write_bytes(encode(values))
                (path / 'graded/manifest.json').write_text(json.dumps({
                    'validationMode': 'current-runtime', 'candidatesSha256': w.digest(rows),
                    'corpusSha256': 'corpus', 'requestsSha256': 'requests',
                    'hardSha256': w.digest(hard), 'judgmentsSha256': w.digest([])}))
            report = eval_io.compare(*paths)
            self.assertEqual(report['base']['hardPassRate'], 1 / 3)
            self.assertEqual(report['base']['errors'], 1)
            self.assertEqual(report['improvedScenarioIds'], ['b'])
            self.assertEqual(report['regressedScenarioIds'], ['a'])
            self.assertEqual(report['quality']['continuity']['pairedJudgments'], 0)
            changed = json.loads((paths[1] / 'suite.json').read_text())
            changed['settings']['maxNewTokens'] = 100
            (paths[1] / 'suite.json').write_text(json.dumps(changed))
            with self.assertRaisesRegex(ValueError, 'identical suites'):
                eval_io.compare(*paths)
            (paths[1] / 'suite.json').write_text(json.dumps(suite))
            result['runtime']['gpu'] = 'other'
            (paths[1] / 'result.json').write_text(json.dumps(result))
            with self.assertRaisesRegex(ValueError, 'Runtime'):
                eval_io.compare(*paths)

    def test_suite_uses_exact_held_out_prompts_without_answers(self):
        w = eval_io.workflow()
        corpus = [{"scenarioId": f"{family}-1", "familyId": family,
                   "input": {"family": family, "here": {"characters": [{"id": "pip"}]}}, "presentation": {"sceneId": family, "requiredEntries": []},
                   "reference": {"sceneId": family, "presentation": [], "choices": []}}
                  for family in ('bank', 'workshop', 'supplied', 'repaired')]
        def fake_rust(command, *args):
            options = dict(zip(args[::2], args[1::2]))
            if command == 'export':
                w.write_rows(options['--output'], corpus)
            else:
                rows = w.rows(options['--candidates'])
                w.write_rows(options['--output'], [{"scenarioId": r['scenarioId'], 'candidateId': r['candidateId'],
                                                   'passed': True} for r in rows])
        with TemporaryDirectory() as temporary, patch.object(w, 'rust', side_effect=fake_rust), patch.object(eval_io, 'workflow', return_value=w):
            root = Path(temporary)
            run = root / 'run'
            w.prepare(SimpleNamespace(out=run, candidates=1, brief='Synthetic test'))
            w.grade(SimpleNamespace(run=run, candidates=run / 'reference-candidates.jsonl', judgments=None, out=root / 'graded'))
            reviews = w.rows(root / 'graded/reviews-template.jsonl')
            for row in reviews:
                row.update(decision='approve', reviewer='test', notes='synthetic test')
            w.write_rows(root / 'reviews.jsonl', reviews)
            w.export(SimpleNamespace(run=run, graded=root / 'graded', reviews=root / 'reviews.jsonl', out=root / 'export'))
            prepare(root / 'export', run / 'corpus.jsonl', root / 'bundle')
            for split in ('validation', 'test'):
                suite = eval_io.make_suite(root, run, split)
                self.assertEqual(len(suite['requests']), 1)
                self.assertEqual([m['role'] for m in suite['requests'][0]['messages']], ['system', 'user'])
                self.assertEqual(set(suite['requests'][0]), {'scenarioId', 'messages', 'promptVersion'})
            original = eval_io.make_suite(root, run)
            variant = eval_io.make_suite(root, run, prompt_variant='speakers-v1')
            self.assertNotIn('promptVariant', original)
            self.assertEqual(variant['promptVariant'], 'speakers-v1')
            self.assertEqual(variant['requests'], [eval_io.speaker_prompt(r) for r in original['requests']])
            self.assertEqual(variant['trainingBundleSha256'], original['trainingBundleSha256'])
            self.assertNotEqual(sha(encode([original])), sha(encode([variant])))
            unique = eval_io.make_suite(root, run, prompt_variant='speakers-unique-v1')
            self.assertEqual(unique['requests'], [eval_io.speaker_prompt(r, 'speakers-unique-v1')
                                                  for r in original['requests']])
            self.assertEqual(unique['promptVariant'], 'speakers-unique-v1')
            self.assertNotEqual(sha(encode([unique])), sha(encode([variant])))
            with self.assertRaisesRegex(ValueError, 'Unknown prompt variant'):
                eval_io.make_suite(root, run, prompt_variant='typo')
            (run / 'corpus.jsonl').write_text('{}\n')
            with self.assertRaisesRegex(ValueError, 'Wrong source corpus'):
                eval_io.make_suite(root, run)


if __name__ == '__main__':
    unittest.main()
