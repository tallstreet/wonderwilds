"""Recheck batch acceptance, current engine validity, and prepare a fresh local bundle."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

from data import prepare, sha

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "qa/story-eval/experiments/broader-reference-quality-2026-10-30-r2"
APPROVALS = ROOT / "qa/story-eval/approvals/broader-reference-r2-2026-10-30-user-acceptance"


def verify_acceptance(source, approvals):
    acceptance = json.loads((approvals / "acceptance.json").read_text())
    if sha((source / "snapshot.json").read_bytes()) != acceptance["snapshotFileSha256"]:
        raise ValueError("Acceptance belongs to a different snapshot")
    for name in ("human-reviews.jsonl", "prior-decisions.jsonl"):
        if sha((approvals / name).read_bytes()) != acceptance["files"][name]:
            raise ValueError(f"Accepted decisions changed: {name}")
    return acceptance


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True, help="New experiment directory")
    args = parser.parse_args()
    acceptance = verify_acceptance(SOURCE, APPROVALS)
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    subprocess.run([sys.executable, str(ROOT / "qa/story-eval/dataset.py"), "review",
                    "--dataset", str(SOURCE), "--out", str(out / "review")], cwd=ROOT, check=True)
    subprocess.run([sys.executable, str(ROOT / "qa/story-eval/run.py"), "export",
                    "--run", str(SOURCE / "run"), "--graded", str(out / "review"),
                    "--reviews", str(APPROVALS / "human-reviews.jsonl"),
                    "--out", str(out / "export")], cwd=ROOT, check=True)
    manifest = prepare(out / "export", SOURCE / "run/corpus.jsonl", out / "bundle")
    (out / "acceptance.json").write_text(json.dumps(acceptance, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    main()
