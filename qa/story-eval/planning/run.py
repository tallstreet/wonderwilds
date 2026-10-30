#!/usr/bin/env python3
"""Offline mission planning experiment; stdlib only, explicit inference, no training."""
import argparse
import collections
import hashlib
import html
import json
import os
from pathlib import Path
import shlex
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
VERSION = "mission-plan-eval-1"
ENDPOINT = "https://garytafe--ep-wonderwilds-server.us-west.modal.direct/v1/chat/completions"
SYSTEM = """You plan short, kind adventures for Wonderwilds. Return only a JSON object matching outputSchema. Choose an offered opportunity and plan its supported outcome using only the supplied characters, capabilities and condition IDs. Provide a clear goal, a persistent narrative brief (premise, participantIds, payoff), and an ordered list of meaningful milestones. Conditions must be achievable in that order; respect consumed resources, travel gates, prerequisites and existing progress. Already completed conditions need not be repeated. Do not ask for unavailable work (for example gathering wood when sufficient planks already exist or repairing an already repaired bridge). Include the conditions needed to finish the outcome. Do not invent mechanics, items, characters, condition IDs, rewards, educational tasks or academic gates. Do not claim that future events have already occurred. Ages 6–8: short, warm, concrete wording; no frightening or manipulative stakes. This is a planning request, not scene dialogue: do not return presentation or choices."""


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def rows(path):
    path = Path(path)
    if path.stat().st_size > 32 * 1024 * 1024:
        raise ValueError("Input exceeds 32 MiB")
    return [json.loads(line) for line in path.read_text().splitlines()]


def write_json(path, value):
    with Path(path).open("x") as out:
        json.dump(value, out, indent=2, ensure_ascii=False)
        out.write("\n")


def write_rows(path, values):
    with Path(path).open("x") as out:
        for value in values:
            out.write(json.dumps(value, ensure_ascii=False) + "\n")


def rust(*args):
    subprocess.run(["cargo", "run", "--quiet", "-p", "wonderwilds-director", "--locked",
                    "--example", "story_plan_eval", "--", *map(str, args)], cwd=ROOT, check=True)


def output_schema():
    schema = read_json(ROOT / "contracts/schemas/v1.json")
    # Resolve the transitive closure, never send unrelated schemas or reference answers.
    defs = {}
    def visit(value):
        if isinstance(value, dict):
            if "$ref" in value:
                name = value["$ref"].removeprefix("#/$defs/")
                if name not in defs:
                    defs[name] = schema["$defs"][name]
                    visit(defs[name])
            for child in value.values():
                visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)
    result = dict(schema["$defs"]["StoryMissionPlan"])
    visit(result)
    return {**result, "$defs": defs}


def request_for(scenario, condition_order="authored"):
    if condition_order not in {"authored", "reversed"}:
        raise ValueError("Unknown condition order")
    model_input = json.loads(json.dumps(scenario["input"]))
    if condition_order == "reversed":
        for opportunity in model_input["opportunities"]:
            opportunity["conditions"].reverse()
    supplied = {"input": model_input, "completedConditionIds": scenario["completedConditionIds"],
                "outputSchema": output_schema()}
    return {"scenarioId": scenario["source"]["scenarioId"], "candidateId": "current-model-1",
            "promptVersion": VERSION + ("-reversed-conditions" if condition_order == "reversed" else ""),
            "messages": [{"role": "system", "content": SYSTEM},
                         {"role": "user", "content": json.dumps(supplied, separators=(",", ":"))}]}


def prepare(args):
    args.out.mkdir(parents=True, exist_ok=False)
    rust("export", "--output", args.out / "scenarios.jsonl")
    scenarios = rows(args.out / "scenarios.jsonl")
    requests = [request_for(s, args.condition_order) for s in scenarios]
    write_rows(args.out / "requests.jsonl", requests)
    write_rows(args.out / "references.jsonl", [{"scenarioId": s["source"]["scenarioId"],
               "candidateId": "reference", "output": s["reference"]} for s in scenarios])
    write_json(args.out / "manifest.json", {"version": VERSION, "scenarioCount": len(scenarios),
               "scenarioSha256": digest(scenarios), "requestSha256": digest(requests),
               "conditionOrder": args.condition_order,
               "gitCommit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
               "sourceSha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in [Path(__file__).resolve(), ROOT / "backend/crates/director/src/story/planning_evaluation.rs",
                             ROOT / "backend/crates/director/examples/story_plan_eval.rs"]},
               "scope": "Authored opportunities only; related to scene training families; no unseen-mission claim."})
    print(f"Prepared {len(scenarios)} cases in {args.out}", flush=True)


def verified_run(path):
    scenarios, requests = rows(path / "scenarios.jsonl"), rows(path / "requests.jsonl")
    manifest = read_json(path / "manifest.json")
    if manifest["version"] != VERSION or digest(scenarios) != manifest["scenarioSha256"] or digest(requests) != manifest["requestSha256"]:
        raise ValueError("Run manifest mismatch")
    if requests != [request_for(s, manifest.get("conditionOrder", "authored")) for s in scenarios]:
        raise ValueError("Requests differ from current prompt/schema; prepare a new run")
    return scenarios, requests, manifest


def credentials(env_file):
    values = dict(os.environ)
    if env_file:
        # Parse assignments as data; never execute shell commands from an env file.
        for line in env_file.read_text().splitlines():
            line = line.strip().removeprefix("export ")
            if "=" not in line or line.startswith("#"):
                continue
            key, value = line.split("=", 1)
            if key.strip() in {"MODAL_PROXY_TOKEN_ID", "MODAL_PROXY_TOKEN_SECRET"}:
                parts = shlex.split(value, comments=True)
                if len(parts) != 1:
                    raise ValueError("Invalid proxy credential assignment")
                values[key.strip()] = parts[0]
    if not all(values.get(k) for k in ("MODAL_PROXY_TOKEN_ID", "MODAL_PROXY_TOKEN_SECRET")):
        raise ValueError("Missing Modal proxy credentials")
    return "Bearer " + values["MODAL_PROXY_TOKEN_ID"] + "." + values["MODAL_PROXY_TOKEN_SECRET"]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def parse_response(envelope):
    if envelope.get("error"):
        raise ValueError("provider_error")
    choice = envelope["choices"][0]
    if choice["finish_reason"] != "stop":
        raise ValueError("finish_reason_" + str(choice["finish_reason"]))
    output = json.loads(choice["message"]["content"])
    if not isinstance(output, dict):
        raise ValueError("non_object_output")
    return output


def generate(args):
    _, requests, manifest = verified_run(args.run)
    authorization = credentials(args.env_file)
    args.out.mkdir(parents=True, exist_ok=False)
    opener = urllib.request.build_opener(NoRedirect)
    metadata = {"version": VERSION, "endpoint": ENDPOINT, "model": "wonderwilds",
                "modelRevision": "Endpoint alias; immutable deployed weight revision not exposed",
                "suiteSha256": manifest["requestSha256"], "temperature": 0, "maxTokens": 2048,
                "reasoningStrength": "low", "timeoutSeconds": 60, "automaticRetries": 0}
    write_json(args.out / "generation.json", metadata)
    # Flush each record: interruptions keep completed outputs; missing cases never count as passes.
    with (args.out / "candidates.jsonl").open("x") as candidates, (args.out / "responses.jsonl").open("x") as raw:
        for index, request in enumerate(requests):
            body = {"model": "wonderwilds", "messages": request["messages"], "temperature": 0,
                    "max_tokens": 2048, "stream": False, "chat_template_kwargs": {"reasoning_strength": "low"}}
            record = {k: request[k] for k in ("scenarioId", "candidateId")}
            record["generator"] = {**metadata, "promptVersion": request["promptVersion"]}
            evidence = {"scenarioId": request["scenarioId"], "requestSha256": digest(body)}
            start = time.monotonic()
            try:
                req = urllib.request.Request(ENDPOINT, data=json.dumps(body).encode(),
                    headers={"Authorization": authorization, "Content-Type": "application/json"})
                with opener.open(req, timeout=60) as response:
                    data = response.read(64 * 1024 + 1)
                if len(data) > 64 * 1024:
                    raise ValueError("oversized_response")
                envelope = json.loads(data)
                evidence["response"] = envelope
                record["output"] = parse_response(envelope)
            except (ValueError, KeyError, IndexError, TypeError, OSError) as exc:
                # Never serialize exception messages/headers which could contain credentials.
                code = f"http_{exc.code}" if isinstance(exc, urllib.error.HTTPError) else type(exc).__name__
                evidence["error"] = code
                record["generator"]["generationError"] = code
                record["output"] = "generation_failed:" + code
            evidence["seconds"] = round(time.monotonic() - start, 3)
            candidates.write(json.dumps(record, ensure_ascii=False) + "\n"); candidates.flush()
            raw.write(json.dumps(evidence, ensure_ascii=False) + "\n"); raw.flush()
            print(f"{index + 1}/{len(requests)} {request['scenarioId']}: {evidence.get('error', 'received')} ({evidence['seconds']}s)", flush=True)


def grade(args):
    scenarios, _, manifest = verified_run(args.run)
    candidates = rows(args.candidates)
    known = {s["source"]["scenarioId"]: s for s in scenarios}
    seen = set()
    for c in candidates:
        if c["scenarioId"] not in known or c["scenarioId"] in seen:
            raise ValueError("Unknown or duplicate scenario; grade one candidate per case per run")
        seen.add(c["scenarioId"])
    args.out.mkdir(parents=True, exist_ok=False)
    rust("grade", "--corpus", args.run / "scenarios.jsonl", "--candidates", args.candidates,
         "--output", args.out / "grades.jsonl")
    grades = rows(args.out / "grades.jsonl")
    counts = collections.Counter(f.split(":", 1)[0] for g in grades for f in g["failures"])
    by_mission = {}
    for sid, scenario in known.items():
        mission = scenario["input"]["opportunities"][0]["id"]
        bucket = by_mission.setdefault(mission, {"total": 0, "received": 0, "passed": 0})
        bucket["total"] += 1
        matching = [g for g in grades if g["scenarioId"] == sid]
        bucket["received"] += len(matching)
        bucket["passed"] += sum(g["passed"] for g in matching)
    summary = {"version": VERSION, "suiteSha256": manifest["requestSha256"],
               "candidateSha256": digest(candidates), "total": len(scenarios), "received": len(grades),
               "missing": len(scenarios) - len(grades), "passed": sum(g["passed"] for g in grades),
               "failures": dict(counts), "byMission": by_mission,
               "generationFailures": sum(isinstance(c["output"], str) and c["output"].startswith("generation_failed:") for c in candidates),
               "note": "Transport failures are availability results, not evidence of planning ability.",
               "quality": "Unscored: review prose fidelity, clarity, motivation, age suitability, variety and education separation.",
               "scope": manifest["scope"]}
    write_json(args.out / "summary.json", summary)
    review = ["<!doctype html><meta charset='utf-8'><title>Mission planning evaluation</title><style>body{font:16px system-ui;max-width:1000px;margin:32px auto;padding:16px}pre{white-space:pre-wrap;background:#f3f3f3;padding:16px}article{border-top:2px solid #ddd;margin-top:30px}</style><h1>Mission planning evaluation</h1>",
              "<p>Mechanical witnesses cover existing authored missions only. They do not establish prose truth, child suitability or novel mission generation. No training or approval is performed.</p>",
              "<pre>" + html.escape(json.dumps(summary, indent=2)) + "</pre>"]
    for c, g in zip(candidates, grades, strict=True):
        s = known[c["scenarioId"]]
        review.append("<article><h2>" + html.escape(c["scenarioId"]) + "</h2>")
        for title, value in [("Input", s["input"]), ("Candidate", c["output"]), ("Mechanical grade and executable witness", g)]:
            review.append("<h3>" + title + "</h3><pre>" + html.escape(json.dumps(value, ensure_ascii=False, indent=2)) + "</pre>")
        review.append("<p>Review: Does the premise fit existing facts? Do labels match conditions? Is the payoff supported? Are steps clear and kind? Is anything academic required? Is this meaningfully different or just reworded?</p></article>")
    (args.out / "review.html").write_text("\n".join(review))
    print(json.dumps(summary, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("prepare"); p.add_argument("--out", type=Path, required=True)
    p.add_argument("--condition-order", choices=("authored", "reversed"), default="authored")
    p = sub.add_parser("generate"); p.add_argument("--run", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True); p.add_argument("--env-file", type=Path)
    p = sub.add_parser("grade"); p.add_argument("--run", type=Path, required=True)
    p.add_argument("--candidates", type=Path, required=True); p.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    {"prepare": prepare, "generate": generate, "grade": grade}[args.command](args)


if __name__ == "__main__":
    main()
