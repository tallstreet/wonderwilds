import json
from pathlib import Path
import tempfile
import threading
import unittest
from unittest.mock import patch
from urllib.error import HTTPError
from urllib.request import Request, urlopen
from http.server import ThreadingHTTPServer

import mission_review as review


class MissionReviewTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.source = Path(self.tmp.name) / 'missions.jsonl'
        self.progress = Path(self.tmp.name) / 'feedback.json'
        self.capture = {'format': 'mission-capture-1', 'id': 'a' * 64, 'kind': 'neighbourhood',
                        'providerValidation': 'passed', 'request': {'model': 'synthetic'},
                        'output': {'title': '<script>alert(1)</script>'}}
        self.source.write_text(json.dumps(self.capture) + '\n')
        self.state = review.ReviewState(self.source, self.progress)

    def tearDown(self):
        self.state.close()
        self.tmp.cleanup()

    def decision(self):
        return {'id': self.capture['id'], 'fingerprint': review.fingerprint(self.capture),
                'reviewer': 'Reviewer A', 'decision': 'revise', 'notes': 'Give a clearer clue',
                'scores': dict.fromkeys(review.CRITERIA, 3)}

    def test_live_append_partial_record_and_restart(self):
        self.state.save(self.decision())
        with self.source.open('a') as stream:
            stream.write('{"format":')
        workspace = self.state.workspace()
        self.assertTrue(workspace['partial'])
        self.assertEqual(len(workspace['items']), 1)
        self.state.close()
        self.state = review.ReviewState(self.source, self.progress)
        self.assertEqual(self.state.workspace()['reviews'][0]['scores']['story'], 3)

    def test_scores_notes_identity_and_changed_capture(self):
        for change in ({'scores': dict.fromkeys(review.CRITERIA, True)},
                       {'scores': dict.fromkeys(review.CRITERIA, 6)},
                       {'scores': {}}, {'notes': ''}, {'reviewer': ' '},
                       {'fingerprint': 'stale'}, {'decision': 'approved-for-release'}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.state.save(dict(self.decision(), **change))
        self.capture['output']['title'] = 'Changed'
        self.source.write_text(json.dumps(self.capture) + '\n')
        value = self.decision()
        value['fingerprint'] = 'old'
        with self.assertRaises(ValueError):
            self.state.save(value)

    def test_independent_reviewers_and_disk_failure(self):
        self.state.save(self.decision())
        self.state.save(dict(self.decision(), reviewer='Reviewer B'))
        previous = self.progress.read_bytes()
        with patch('mission_review.os.replace', side_effect=OSError('disk failure')):
            with self.assertRaises(OSError):
                self.state.save(dict(self.decision(), notes='Changed review'))
        self.assertEqual(self.progress.read_bytes(), previous)
        self.assertEqual(len(self.state.reviews), 2)
        self.state.save(dict(self.decision(), notes='New note'))
        self.assertEqual(len(self.state.reviews), 2)

    def test_invalid_log_is_visible_and_writer_is_exclusive(self):
        with self.assertRaises(ValueError):
            review.ReviewState(self.source, self.progress)
        with self.source.open('a') as stream:
            stream.write('invalid\n')
        with self.assertRaises(ValueError):
            self.state.workspace()

    def test_http_boundaries_and_export(self):
        server = ThreadingHTTPServer(('127.0.0.1', 0), review.handler(self.state))
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        url = f'http://127.0.0.1:{server.server_port}'
        try:
            for headers in ({'Origin': 'https://evil.example'}, {'Host': 'evil.example'}):
                with self.assertRaises(HTTPError) as error:
                    urlopen(Request(url + '/api/review', headers=headers))
                self.assertEqual(error.exception.code, 403)
            request = Request(url + '/api/review', data=json.dumps(self.decision()).encode(),
                              headers={'Content-Type': 'application/json'})
            with self.assertRaises(HTTPError) as error:
                urlopen(request)
            self.assertEqual(error.exception.code, 403)
            request.add_header('X-Review-Token', self.state.token)
            with urlopen(request) as response:
                self.assertEqual(response.status, 200)
            with urlopen(url + '/api/export') as response:
                rows = [json.loads(line) for line in response.read().splitlines()]
            self.assertEqual(rows[0]['capture'], self.capture)
            self.assertEqual(rows[0]['reviews'][0]['notes'], 'Give a clearer clue')
        finally:
            server.shutdown()
            server.server_close()
            thread.join()


if __name__ == '__main__':
    unittest.main()
