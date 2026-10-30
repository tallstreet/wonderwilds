"""Compare matched base and adapter results; requires current-engine hard grades."""
import argparse
import json
from pathlib import Path

from eval_io import compare

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", type=Path, required=True)
    parser.add_argument("--adapter", type=Path, required=True)
    parser.add_argument("--base-graded", type=Path, help="Optional regrade including narrative judgments")
    parser.add_argument("--adapter-graded", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    report = compare(args.base, args.adapter, args.base_graded, args.adapter_graded)
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = ["# Glimmer evaluation comparison", "", f"Split: {report['split']}", "",
             "| Metric | Base | Adapter |", "| --- | ---: | ---: |"]
    for key in report["base"]:
        lines.append(f"| {key} | {report['base'][key]} | {report['adapter'][key]} |")
    lines += ["", "Improved scenario IDs: " + ", ".join(report["improvedScenarioIds"]),
              "Regressed scenario IDs: " + ", ".join(report["regressedScenarioIds"]),
              "", report["note"], "", "Per-criterion paired narrative counts are in comparison.json."]
    (args.out / "comparison.md").write_text("\n".join(lines) + "\n")
    print(json.dumps(report, indent=2))
