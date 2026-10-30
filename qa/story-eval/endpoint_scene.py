#!/usr/bin/env python3
"""Evaluate the live scene request against a representative synthetic state matrix."""
import argparse
import collections
import importlib.util
import json
from pathlib import Path

import run as scene

SPEC = importlib.util.spec_from_file_location("planning", Path(__file__).parent / "planning/run.py")
transport = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(transport)
BRIEF = "Ages 6–8; short, warm sentences; concrete next steps; no frightening or manipulative content."


def select(corpus):
    # First reachable example per mission/status/location/introduction combination.
    # Select before generation, never based on model outputs or reference quality.
    selected = {}
    for row in corpus:
        mission = row["input"]["mission"] or {}
        key = (mission.get("missionId"), mission.get("status"), row["input"]["context"]["locationId"],
               bool(row["presentation"]["requiredEntries"]))
        selected.setdefault(key, row)
    return list(selected.values())


def request_for(row):
    schema = scene.read_json(scene.ROOT / "contracts/schemas/v1.json")["$defs"]["StoryScene"]
    supplied = {"input": row["input"], "presentation": row["presentation"],
                "allowedSpeakerIds": [c["id"] for c in row["input"]["here"]["characters"]],
                "outputSchema": schema, "editorialBrief": BRIEF}
    return {"scenarioId": row["scenarioId"], "candidateId": "current-model-schema-1",
            "promptVersion": "live-scene-json-schema-1",
            "messages": [{"role": "system", "content": (scene.ROOT / "backend/crates/api/src/story/modal_prompt.txt").read_text()},
                         {"role": "user", "content": json.dumps(supplied, separators=(",", ":"))}],
            "response_format": transport.response_format("StoryScene", schema)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--env-file", type=Path)
    args = parser.parse_args()
    authorization = transport.credentials(args.env_file)
    args.out.mkdir(parents=True, exist_ok=False)
    scene.rust("export", "--output", args.out / "full-corpus.jsonl")
    corpus = select(scene.rows(args.out / "full-corpus.jsonl"))
    requests = [request_for(row) for row in corpus]
    scene.write_rows(args.out / "corpus.jsonl", corpus)
    scene.write_rows(args.out / "requests.jsonl", requests)
    metadata = {"provider": "modal-endpoint", "endpoint": transport.ENDPOINT, "model": "wonderwilds",
                "modelRevision": "Endpoint alias; immutable deployed weight revision not exposed",
                "corpusSha256": scene.digest(corpus), "suiteSha256": scene.digest(requests),
                "temperature": 0, "maxTokens": 2048, "reasoningStrength": "low",
                "timeoutSeconds": 60, "automaticRetries": 0, "responseMode": "json-schema",
                "selection": "First mission/status/location/required-introduction combination in runtime corpus"}
    transport.generate_requests(requests, args.out / "live", authorization, metadata)
    scene.rust("grade", "--corpus", args.out / "corpus.jsonl", "--candidates",
               args.out / "live/candidates.jsonl", "--output", args.out / "grades.jsonl")
    grades = scene.rows(args.out / "grades.jsonl")
    candidates = scene.rows(args.out / "live/candidates.jsonl")
    summary = {"total": len(corpus), "received": len(candidates),
               "passed": sum(g["passed"] for g in grades),
               "generationFailures": sum("generationError" in c["generator"] for c in candidates),
               "failures": dict(collections.Counter(f for g in grades for f in g["failures"])),
               "missions": sorted({(r["input"]["mission"] or {}).get("missionId", "no_active_mission") for r in corpus}),
               "note": "Mechanical integration sample; no prose-quality or unseen-mission claim. No fallback or repair."}
    scene.write_json(args.out / "summary.json", summary)
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
