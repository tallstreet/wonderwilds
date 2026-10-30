#!/usr/bin/env python3
"""Loopback-only human story review with durable, candidate-bound decisions."""
import argparse
import copy
from datetime import datetime, timezone
import fcntl
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import secrets
import tempfile
import threading
from urllib.parse import parse_qs, urlsplit

import dataset
import run as workflow

ASSETS = Path(__file__).resolve().parent / "review-ui"


class ReviewState:
    def __init__(self, source, progress):
        source = source.resolve()
        _, selected, recommendations = dataset.verify(source)
        manifest, corpus, _ = workflow.load_run(source / "run")
        self.fingerprint = workflow.digest(workflow.read_json(source / "snapshot.json"))
        self.dataset_id = source.name
        self.brief = manifest["editorialBrief"]
        self.token = secrets.token_urlsafe(32)
        self.path = progress.resolve()
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.file_lock = self.path.with_suffix(self.path.suffix + ".lock").open("a+")
        try:
            fcntl.flock(self.file_lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError:
            self.file_lock.close()
            raise ValueError("This review progress file is already open in another server") from None
        self.lock = threading.RLock()
        self.candidates = workflow.indexed(workflow.rows(source / "candidates.jsonl"), ("scenarioId", "candidateId"))
        self.judgments = workflow.indexed(workflow.rows(source / "judgments.jsonl"), ("scenarioId", "candidateId"))
        self.recommendations = recommendations
        hard = workflow.indexed(workflow.rows(source / "hard-results.jsonl"), ("scenarioId", "candidateId"))
        contexts = {r["scenarioId"]: r for r in corpus}
        short = {(r["scenarioId"], r["candidateId"]) for rows in selected.values() for r in rows}
        overlap = workflow.training_overlap(corpus, manifest["splits"])
        self.items = {}
        for key, candidate in self.candidates.items():
            context = contexts[key[0]]
            self.items[key] = {"scenarioId": key[0], "candidateId": key[1],
                "candidateSha256": workflow.digest(candidate), "input": context["input"],
                "output": candidate["output"], "split": manifest["splits"][context["familyId"]],
                "shortlisted": key in short, "excludedForEvalOverlap": key[0] in overlap,
                "hardPassed": hard[key]["passed"]}
        self.reviews = {}
        self.reviewer = ""
        try:
            if self.path.exists():
                saved = workflow.read_json(self.path)
                if saved.get("format") != "human-story-review-1" or saved.get("datasetSha256") != self.fingerprint:
                    raise ValueError("Progress belongs to a different dataset version; choose a new --progress path")
                for record in saved["reviews"]:
                    key = self.validate(record)
                    if key in self.reviews:
                        raise ValueError("Duplicate saved decision")
                    self.reviews[key] = record
                self.reviewer = saved["reviewer"]
        except Exception:
            self.close()
            raise

    def close(self):
        self.file_lock.close()

    def validate(self, value):
        if not isinstance(value, dict):
            raise ValueError("Decision must be an object")
        key = (value.get("scenarioId"), value.get("candidateId"))
        if not all(isinstance(v, str) for v in key) or key not in self.items:
            raise ValueError("Unknown candidate")
        if value.get("candidateSha256") != self.items[key]["candidateSha256"]:
            raise ValueError("Candidate changed; reload before deciding")
        decision = value.get("decision")
        if decision not in ("approve", "revise", "reject"):
            raise ValueError("Choose approve, revise or reject")
        for field, limit in (("reviewer", 100), ("notes", 2000), ("suggestedText", 6000)):
            if not isinstance(value.get(field, ""), str) or len(value.get(field, "")) > limit:
                raise ValueError(f"Invalid or overlong {field}")
        if not value.get("reviewer", "").strip():
            raise ValueError("Enter your reviewer name")
        if decision != "approve" and not value.get("notes", "").strip():
            raise ValueError("Explain what needs revision or why you reject this scene")
        if decision != "revise" and value.get("suggestedText", "").strip():
            raise ValueError("Suggested edits need revision and regrading; approve only the unchanged original")
        if decision == "approve" and not self.items[key]["hardPassed"]:
            raise ValueError("A mechanically invalid candidate cannot be approved")
        return key

    def persist(self, reviews, reviewer):
        content = {"format": "human-story-review-1", "datasetSha256": self.fingerprint,
                   "reviewer": reviewer, "reviews": list(reviews.values())}
        name = None
        try:
            with tempfile.NamedTemporaryFile(mode="w", dir=self.path.parent, delete=False, encoding="utf-8") as target:
                name = target.name
                json.dump(content, target, ensure_ascii=False, indent=2)
                target.write("\n")
                target.flush()
                os.fsync(target.fileno())
            os.replace(name, self.path)
        finally:
            if name and os.path.exists(name):
                os.unlink(name)

    def save(self, value):
        with self.lock:
            key = self.validate(value)
            previous = self.reviews.get(key)
            record = {k: value.get(k, "").strip() for k in (
                "scenarioId", "candidateId", "candidateSha256", "decision", "reviewer", "notes", "suggestedText")}
            if not record["notes"]:
                record["notes"] = "Approved unchanged after reviewing the game state, scene and choices."
            record["updatedAt"] = datetime.now(timezone.utc).isoformat()
            record["firstDecision"] = previous.get("firstDecision", previous["decision"]) if previous else record["decision"]
            record["firstDecisionAt"] = previous.get("firstDecisionAt", previous["updatedAt"]) if previous else record["updatedAt"]
            next_reviews = {**self.reviews, key: record}
            self.persist(next_reviews, record["reviewer"])
            self.reviews = next_reviews
            self.reviewer = record["reviewer"]
            return copy.deepcopy(record)

    def view(self):
        with self.lock:
            # Do not send model verdicts/reasons to the browser before a decision.
            return {"datasetId": self.dataset_id, "datasetSha256": self.fingerprint,
                    "editorialBrief": self.brief, "token": self.token, "reviewer": self.reviewer,
                    "items": [{**copy.deepcopy(item), "review": copy.deepcopy(self.reviews.get(key))}
                              for key, item in self.items.items()]}

    def grade(self, key):
        with self.lock:
            if key not in self.reviews:
                raise PermissionError("Make and save your decision before viewing model grades")
            return {"judgment": self.judgments[key], "recommendation": self.recommendations[key]}


def handler(state):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def send(self, status, value, content_type="application/json; charset=utf-8", download=False):
            data = value if isinstance(value, bytes) else json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy", "default-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'")
            if download:
                self.send_header("Content-Disposition", 'attachment; filename="human-reviews.jsonl"')
            self.end_headers()
            self.wfile.write(data)

        def allowed(self):
            host = f"127.0.0.1:{self.server.server_port}"
            if (self.headers.get("Host") != host
                    or self.headers.get("Origin") not in (None, f"http://{host}")
                    or self.headers.get("Sec-Fetch-Site") not in (None, "none", "same-origin")):
                self.send(403, {"error": "Open the local 127.0.0.1 review address"})
                return False
            return True

        def do_GET(self):
            if not self.allowed():
                return
            route = urlsplit(self.path)
            if route.path == "/api/review":
                self.send(200, state.view())
            elif route.path == "/api/grade":
                params = parse_qs(route.query)
                key = (params.get("scenarioId", [""])[0], params.get("candidateId", [""])[0])
                try:
                    self.send(200, state.grade(key))
                except PermissionError as error:
                    self.send(403, {"error": str(error)})
            elif route.path == "/api/export":
                with state.lock:
                    body = "".join(workflow.packed(r) + "\n" for r in state.reviews.values()).encode()
                self.send(200, body, "application/x-ndjson; charset=utf-8", download=True)
            elif route.path in ("/", "/app.js", "/style.css"):
                name, kind = {"/": ("index.html", "text/html"), "/app.js": ("app.js", "text/javascript"),
                              "/style.css": ("style.css", "text/css")}[route.path]
                self.send(200, (ASSETS / name).read_bytes(), kind + "; charset=utf-8")
            else:
                self.send(404, {"error": "Not found"})

        def do_POST(self):
            if not self.allowed():
                return
            if self.path != "/api/decision":
                return self.send(404, {"error": "Not found"})
            if not secrets.compare_digest(self.headers.get("X-Review-Token", ""), state.token):
                return self.send(403, {"error": "Review session expired; reload this page"})
            try:
                size = int(self.headers.get("Content-Length", "0"))
                if not 0 < size <= 32000 or self.headers.get_content_type() != "application/json":
                    raise ValueError("Expected a bounded JSON decision")
                value = json.loads(self.rfile.read(size))
                self.send(200, {"review": state.save(value)})
            except (ValueError, TypeError, KeyError) as error:
                self.send(409 if "Candidate changed" in str(error) else 400, {"error": str(error)})
            except OSError:
                self.send(500, {"error": "Decision could not be saved to disk. Your previous progress is unchanged."})
    return Handler


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", type=Path, default=dataset.DEFAULT)
    parser.add_argument("--progress", type=Path)
    parser.add_argument("--port", type=int, default=5182)
    args = parser.parse_args()
    progress = args.progress or Path(__file__).resolve().parent / "experiments/human-review" / args.dataset.name / "progress.json"
    state = ReviewState(args.dataset, progress)
    try:
        server = ThreadingHTTPServer(("127.0.0.1", args.port), handler(state))
        print(f"Human review: http://127.0.0.1:{server.server_port}/\nProgress: {state.path}", flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            server.server_close()
    finally:
        state.close()


if __name__ == "__main__":
    main()
