#!/usr/bin/env python3
"""Review missions captured during local synthetic play; no model calls or game writes."""
import argparse
from datetime import datetime, timezone
import fcntl
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import secrets
import tempfile
import threading

ASSETS = Path(__file__).with_name('mission-review-ui')
CRITERIA = ('story', 'clarity', 'variety', 'learning_fit', 'age_fit')


def packed(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))


def fingerprint(value):
    return hashlib.sha256(packed(value).encode()).hexdigest()


class ReviewState:
    def __init__(self, source, progress):
        self.source = source.resolve()
        self.path = progress.resolve()
        if self.source == self.path:
            raise ValueError('Capture and feedback paths must be different')
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.file_lock = self.path.with_suffix(self.path.suffix + '.lock').open('a+')
        try:
            fcntl.flock(self.file_lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError:
            self.file_lock.close()
            raise ValueError('This feedback file is already open in another server') from None
        self.lock = threading.RLock()
        self.token = secrets.token_urlsafe(32)
        try:
            self.reviews = json.loads(self.path.read_text()) if self.path.exists() else []
            if not isinstance(self.reviews, list):
                raise ValueError('Invalid feedback file')
        except Exception:
            self.close()
            raise

    def close(self):
        self.file_lock.close()

    def captures(self):
        try:
            data = self.source.read_bytes()
        except FileNotFoundError:
            return [], False
        # A writer may be in the middle of appending; expose complete records only.
        partial = bool(data and not data.endswith(b'\n'))
        lines = data.split(b'\n')[:-1]
        result, ids = [], set()
        for index, line in enumerate(lines, 1):
            try:
                record = json.loads(line)
                if (record['format'] != 'mission-capture-1' or
                        record['kind'] not in ('mission_plan', 'neighbourhood') or
                        record['providerValidation'] not in ('passed', 'failed') or
                        not isinstance(record['id'], str) or len(record['id']) != 64 or
                        record['id'] in ids or not isinstance(record['request'], dict)):
                    raise ValueError('invalid capture')
                record['output']
                ids.add(record['id'])
                result.append({'capture': record, 'fingerprint': fingerprint(record)})
            except (ValueError, KeyError, TypeError):
                raise ValueError(f'Invalid or duplicate capture on line {index}; preserve the file and inspect it') from None
        return result, partial

    def workspace(self):
        with self.lock:
            items, partial = self.captures()
            return {'items': items, 'reviews': self.reviews, 'token': self.token,
                    'partial': partial, 'criteria': CRITERIA}

    def save(self, value):
        with self.lock:
            if not isinstance(value, dict):
                raise ValueError('Expected a review object')
            items, _ = self.captures()
            item = next((item for item in items if item['capture']['id'] == value.get('id')), None)
            if not item or value.get('fingerprint') != item['fingerprint']:
                raise ValueError('Capture changed or missing; reload before grading')
            for field, limit in (('reviewer', 100), ('notes', 4000)):
                if not isinstance(value.get(field), str) or len(value[field]) > limit:
                    raise ValueError(f'Invalid {field}')
            if not value['reviewer'].strip():
                raise ValueError('Enter your reviewer name')
            if value.get('decision') not in ('accept', 'revise', 'reject'):
                raise ValueError('Choose a decision')
            if value['decision'] != 'accept' and not value['notes'].strip():
                raise ValueError('Explain what needs revision or rejection')
            scores = value.get('scores')
            if (not isinstance(scores, dict) or set(scores) != set(CRITERIA) or
                    any(type(score) is not int or not 1 <= score <= 5 for score in scores.values())):
                raise ValueError('Grade every criterion from 1 to 5')
            record = {key: value[key] for key in ('id', 'fingerprint', 'decision', 'notes', 'scores')}
            record.update(format='mission-feedback-1', reviewer=value['reviewer'].strip(),
                          reviewedAt=datetime.now(timezone.utc).isoformat())
            reviews = [r for r in self.reviews if (r['id'], r['reviewer']) != (record['id'], record['reviewer'])]
            reviews.append(record)
            # Replace atomically; failed writes leave the previous decisions intact.
            fd, temporary = tempfile.mkstemp(dir=self.path.parent, prefix='.mission-feedback-')
            try:
                with os.fdopen(fd, 'w') as stream:
                    stream.write(packed(reviews) + '\n')
                    stream.flush()
                    os.fsync(stream.fileno())
                os.replace(temporary, self.path)
            finally:
                if os.path.exists(temporary):
                    os.unlink(temporary)
            self.reviews = reviews
            return record


def handler(state):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def send(self, status, body, kind='application/json; charset=utf-8'):
            if not isinstance(body, bytes):
                body = packed(body).encode()
            self.send_response(status)
            self.send_header('Content-Type', kind)
            self.send_header('Content-Length', str(len(body)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.send_header('Content-Security-Policy', "default-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")
            self.end_headers()
            self.wfile.write(body)

        def allowed(self):
            host = f'127.0.0.1:{self.server.server_port}'
            if self.headers.get('Host') != host or self.headers.get('Origin') not in (None, f'http://{host}'):
                self.send(403, {'error': 'Local review origin required'})
                return False
            return True

        def do_GET(self):
            if not self.allowed():
                return
            try:
                if self.path == '/api/review':
                    self.send(200, state.workspace())
                elif self.path == '/api/export':
                    workspace = state.workspace()
                    rows = [dict(item, reviews=[r for r in workspace['reviews']
                            if r['id'] == item['capture']['id'] and r['fingerprint'] == item['fingerprint']])
                            for item in workspace['items']]
                    self.send(200, ''.join(packed(row) + '\n' for row in rows).encode(), 'application/x-ndjson')
                elif self.path in ('/', '/app.js', '/style.css'):
                    name, kind = {'/': ('index.html', 'text/html'), '/app.js': ('app.js', 'text/javascript'),
                                  '/style.css': ('style.css', 'text/css')}[self.path]
                    self.send(200, (ASSETS / name).read_bytes(), kind + '; charset=utf-8')
                else:
                    self.send(404, {'error': 'Not found'})
            except (ValueError, OSError):
                self.send(500, {'error': 'Could not read captures. Check the capture file for invalid records.'})

        def do_POST(self):
            if not self.allowed():
                return
            if self.path != '/api/review':
                return self.send(404, {'error': 'Not found'})
            if not secrets.compare_digest(self.headers.get('X-Review-Token', ''), state.token):
                return self.send(403, {'error': 'Review session expired; reload'})
            try:
                length = int(self.headers.get('Content-Length', '0'))
                if not 0 < length <= 32000 or self.headers.get_content_type() != 'application/json':
                    raise ValueError('Expected a bounded JSON review')
                self.send(200, state.save(json.loads(self.rfile.read(length))))
            except (ValueError, TypeError, KeyError) as error:
                self.send(400, {'error': str(error)})
            except OSError:
                self.send(500, {'error': 'Could not save feedback. Previous feedback is preserved.'})
    return Handler


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--log', type=Path, default=Path('.local-demo/missions.jsonl'))
    parser.add_argument('--feedback', type=Path, default=Path('.local-demo/mission-feedback.json'))
    parser.add_argument('--port', type=int, default=5184)
    args = parser.parse_args()
    state = ReviewState(args.log, args.feedback)
    try:
        with ThreadingHTTPServer(('127.0.0.1', args.port), handler(state)) as server:
            print(f'Mission review: http://127.0.0.1:{server.server_port}/', flush=True)
            try:
                server.serve_forever()
            except KeyboardInterrupt:
                pass
    finally:
        state.close()


if __name__ == '__main__':
    main()
