#!/usr/bin/env python3
"""Version, verify and review a repository story-evaluation dataset."""
import argparse
import collections
import hashlib
import html
import json
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace

import run as workflow

DEFAULT = Path(__file__).resolve().parent / "datasets/pilot-2026-10-30"
FILES = (
    "run/corpus.jsonl", "run/generation-requests.jsonl", "run/manifest.json",
    "run/reference-candidates.jsonl", "candidates.jsonl", "judgments.jsonl",
    "recommendations.jsonl", "hard-results.jsonl", "assessments.tsv", "rubric.json",
    "quality-review.md", "generation-provenance.json",
    "recommended-train-candidates.jsonl", "recommended-eval-candidates.jsonl",
)
INPUT_FILES = ("run/corpus.jsonl", "run/generation-requests.jsonl", "run/manifest.json",
               "run/reference-candidates.jsonl", "reference-hard-results.jsonl")


def verify_inputs(path):
    """Verify a sealed input-only pack; no generation, judging or new engine proof."""
    pack = workflow.read_json(path / "input-pack.json")
    hashes = {name: hashlib.sha256((path / name).read_bytes()).hexdigest() for name in INPUT_FILES}
    if pack.get("format") != "story-input-pack-1" or pack.get("files") != hashes:
        raise ValueError("Input pack changed; create a new version rather than edit a sealed pack")
    manifest, corpus, requests = workflow.load_run(path / "run")
    references = workflow.rows(path / "run/reference-candidates.jsonl")
    ref_map = workflow.candidate_map(references, corpus)
    hard = workflow.indexed(workflow.rows(path / "reference-hard-results.jsonl"),
                            ("scenarioId", "candidateId"))
    if set(ref_map) != set(hard) or len(ref_map) != len(corpus):
        raise ValueError("Reference checks must cover every input exactly once")
    by_scenario = {row["scenarioId"]: row for row in corpus}
    for key, row in ref_map.items():
        if row["output"] != by_scenario[key[0]]["reference"] or hard[key].get("passed") is not True:
            raise ValueError("Input pack references must match captured authored outputs and pass archived hard checks")
    counts = {"scenarios": len(corpus), "generationRequests": len(requests),
              "referenceHardPasses": len(hard), "modelCandidates": 0,
              "qualityJudgments": 0, "humanApprovals": 0}
    if pack.get("counts") != counts or pack.get("parentDataset") != manifest.get("parentDataset"):
        raise ValueError("Input pack counts or parent linkage differ from contents")
    outcomes = set()
    for row in corpus:
        state = row["checkpoint"]["state"]
        if len(state["completedMissions"]) == 7:
            broad = state["broader"]
            outcomes.add((broad["instrumentHome"], broad["cornerStyle"], broad["gatheringPlace"]))
    coverage = {
        "missionStates": dict(collections.Counter((row["input"].get("mission") or {}).get("missionId", "not_accepted") for row in corpus)),
        "journeyFamilies": sorted({row["familyId"] for row in corpus}),
        "lastingChoiceCombinations": len(outcomes),
    }
    if pack.get("coverage") != coverage:
        raise ValueError("Input pack coverage differs from its captured states")
    return counts


def checksums(dataset):
    return {name: hashlib.sha256((dataset / name).read_bytes()).hexdigest() for name in FILES}


def inspect(dataset):
    manifest, corpus, _ = workflow.load_run(dataset / "run")
    candidates = workflow.rows(dataset / "candidates.jsonl")
    by_key = workflow.candidate_map(candidates, corpus)
    judgments = workflow.checked_judgments(workflow.rows(dataset / "judgments.jsonl"), by_key)
    recommendations = workflow.indexed(workflow.rows(dataset / "recommendations.jsonl"),
                                       ("scenarioId", "candidateId"))
    hard = workflow.indexed(workflow.rows(dataset / "hard-results.jsonl"), ("scenarioId", "candidateId"))
    if not by_key or not (set(by_key) == set(judgments) == set(recommendations) == set(hard)):
        raise ValueError("Candidates, judgments, recommendations and hard checks must cover identical IDs")
    preferred = set()
    for key, row in recommendations.items():
        if row.get("candidateSha256") != workflow.digest(by_key[key]):
            raise ValueError(f"Stale recommendation: {key}")
        if row.get("recommendation") not in ("keep", "revise", "reject"):
            raise ValueError("Unknown recommendation")
        if type(row.get("preferredForScenario")) is not bool:
            raise ValueError("Preferred flag must be boolean")
        if not isinstance(row.get("reason"), str) or not row["reason"].strip():
            raise ValueError("Recommendation requires a reason")
        if type(hard[key].get("passed")) is not bool:
            raise ValueError("Hard pass flag must be boolean")
        if row["recommendation"] == "keep" and (
            not hard[key]["passed"] or any(v["verdict"] != "pass" for v in judgments[key]["criteria"].values())
        ):
            raise ValueError(f"Keep requires all hard and quality criteria to pass: {key}")
        if row["preferredForScenario"]:
            if key[0] in preferred or row["recommendation"] != "keep":
                raise ValueError("Each scenario may prefer at most one keep")
            preferred.add(key[0])
    overlap = workflow.training_overlap(corpus, manifest["splits"])
    families = {row["scenarioId"]: row["familyId"] for row in corpus}
    selected = {"train": [], "eval": []}
    excluded = []
    for key, row in recommendations.items():
        if not row["preferredForScenario"]:
            continue
        if key[0] in overlap:
            excluded.append({"scenarioId": key[0], "candidateId": key[1]})
        else:
            selected[manifest["splits"][families[key[0]]]].append(by_key[key])
    summary = {
        "scenarios": len(corpus), "candidates": len(candidates), "judged": len(judgments),
        "recommendations": dict(collections.Counter(r["recommendation"] for r in recommendations.values())),
        "preferred": len(preferred), "shortlist": {k: len(v) for k, v in selected.items()},
        "excludedForEvalOverlap": excluded, "humanApproved": False, "trainingPerformed": False,
    }
    return summary, selected, recommendations


def verify(dataset):
    snapshot = workflow.read_json(dataset / "snapshot.json")
    if snapshot.get("format") != "story-dataset-1" or snapshot.get("files") != checksums(dataset):
        raise ValueError("Snapshot files changed. Preserve this version; create a new version for edits")
    summary, selected, recommendations = inspect(dataset)
    for split, values in selected.items():
        if workflow.rows(dataset / f"recommended-{split}-candidates.jsonl") != values:
            raise ValueError("Archived shortlist differs from recommendations and split policy")
    if snapshot.get("summary") != summary:
        raise ValueError("Snapshot summary differs from its data")
    return summary, selected, recommendations


def snapshot(args):
    if (args.dataset / "snapshot.json").exists():
        raise FileExistsError("Snapshot already exists; create a new dataset version")
    if not args.notes.strip():
        raise ValueError("Snapshot needs a change description")
    summary, selected, _ = inspect(args.dataset)
    commit = subprocess.run(["git", "rev-parse", "HEAD"], cwd=workflow.ROOT,
                            check=True, capture_output=True, text=True).stdout.strip()
    # Derived shortlists are refreshed only before sealing a new, unfrozen version.
    for split, values in selected.items():
        (args.dataset / f"recommended-{split}-candidates.jsonl").write_text(
            "".join(workflow.packed(value) + "\n" for value in values))
    workflow.write_json(args.dataset / "snapshot.json", {
        "format": "story-dataset-1", "datasetId": args.dataset.name,
        "parentDataset": args.parent, "changeDescription": args.notes,
        "archiveBaseCommit": commit,
        "generationCommit": None,
        "provenanceNote": "Generation-time commit and exact session model IDs were not recorded for the original pilot. See per-row provenance; archiveBaseCommit is not proof of generation version.",
        "files": checksums(args.dataset), "summary": summary,
    })
    print(json.dumps(summary, indent=2))


def review(args):
    summary, selected, recommendations = verify(args.dataset)
    workflow.grade(SimpleNamespace(run=args.dataset / "run", candidates=args.dataset / "candidates.jsonl",
                                   judgments=args.dataset / "judgments.jsonl", out=args.out,
                                   archived_hard=(args.dataset / "hard-results.jsonl")
                                   if getattr(args, "archived", False) else None))
    # Current checks or explicitly archived evidence must retain recommendation consistency.
    current = workflow.indexed(workflow.rows(args.out / "hard-results.jsonl"), ("scenarioId", "candidateId"))
    for values in selected.values():
        if any(not current[(r["scenarioId"], r["candidateId"])]["passed"] for r in values):
            raise ValueError("A preferred candidate fails the current engine; inspect hard-results before proceeding")
    for split, values in selected.items():
        workflow.write_rows(args.out / f"recommended-{split}-candidates.jsonl", values)
    workflow.write_json(args.out / "selection-summary.json", summary)
    page_path = args.out / "review.html"
    page = page_path.read_text()
    for key, recommendation in recommendations.items():
        label = html.escape(" / ".join(key))
        old = "<details><summary>" + label
        new = "<details><summary>" + label + " — " + recommendation["recommendation"].upper()
        if recommendation["preferredForScenario"]:
            new += " · PREFERRED"
        page = page.replace(old, new)
        # Reasons and criterion evidence remain data, never HTML instructions.
        end = page.index("</summary>", page.index(new)) + len("</summary>")
        page = page[:end] + "<p>" + html.escape(recommendation["reason"]) + "</p>" + page[end:]
    intro = ("<section><h2>Dataset quality recommendations</h2><pre>" +
             html.escape(json.dumps(summary, indent=2)) +
             "</pre><p>Model recommendations only. Human review remains pending. "
             "The train/eval candidate shortlists are not approved training exports.</p>"
             '<p><a href="recommended-train-candidates.jsonl">Training shortlist</a> · '
             '<a href="recommended-eval-candidates.jsonl">Held-out shortlist</a></p></section>')
    page_path.write_text(page.replace("<h1>Story evaluation</h1>", "<h1>Story evaluation</h1>" + intro))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("verify", "verify-inputs", "snapshot", "review"):
        command = commands.add_parser(name)
        command.add_argument("--dataset", type=Path, default=DEFAULT)
        if name == "snapshot":
            command.add_argument("--parent")
            command.add_argument("--notes", required=True)
        if name == "review":
            command.add_argument("--out", required=True, type=Path)
            command.add_argument("--archived", action="store_true",
                                 help="Show checksum-verified historical checks without rerunning the current engine")
    args = parser.parse_args()
    try:
        if args.command == "verify":
            print(json.dumps(verify(args.dataset)[0], indent=2))
        elif args.command == "verify-inputs":
            print(json.dumps(verify_inputs(args.dataset), indent=2))
        elif args.command == "snapshot":
            snapshot(args)
        else:
            review(args)
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        print(f"story-dataset: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
