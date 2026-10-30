"""Check real Glimmer token lengths and masks locally without weights or a GPU."""
import argparse
import json
from pathlib import Path

from data import read_rows, tokenize, verify
from worker import MODEL, REVISION


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    verify(args.bundle)
    if args.out.exists():
        raise FileExistsError(args.out)
    from transformers import AutoTokenizer
    tokenizer = AutoTokenizer.from_pretrained(MODEL, revision=REVISION)
    report = {"model": MODEL, "revision": REVISION, "splits": {}}
    for split in ("train", "validation", "test", "smoke"):
        rows = read_rows(args.bundle / f"{split}.jsonl")
        lengths = [len(tokenize(row, tokenizer, 4096)["input_ids"]) for row in rows]
        report["splits"][split] = {"examples": len(rows), "minTokens": min(lengths),
                                    "maxTokens": max(lengths)}
    args.out.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
