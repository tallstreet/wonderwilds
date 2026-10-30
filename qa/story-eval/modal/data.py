"""Dependency-free preparation and verification of approved training bundles."""
import argparse
import hashlib
import json
from pathlib import Path


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def encode(rows):
    return ("".join(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n" for row in rows)).encode()


def prepare(export, corpus_path, out):
    train, test = read_rows(export / "train.jsonl"), read_rows(export / "eval.jsonl")
    provenance = read_rows(export / "provenance.jsonl")
    corpus = {r["scenarioId"]: r for r in read_rows(corpus_path)}
    by_split = {split: sorted((r for r in provenance if r["split"] == split),
                             key=lambda r: r["row"]) for split in ("train", "eval")}
    for split, rows in (("train", train), ("eval", test)):
        if [r["row"] for r in by_split[split]] != list(range(1, len(rows) + 1)):
            raise ValueError("Export provenance does not cover every row")
        for record in by_split[split]:
            review = record["review"]
            if review["decision"] != "approve" or review.get("suggestedText"):
                raise ValueError("Only unchanged approved candidates may be prepared")
            if corpus[review["scenarioId"]]["familyId"] != record["familyId"]:
                raise ValueError("Provenance family mismatch")
    families = sorted({r["familyId"] for r in by_split["train"]}, key=lambda x: sha(x.encode()))
    if len(families) < 2:
        raise ValueError("Need at least two training journey families")
    validation_families = set(families[:max(1, len(families) // 5)])
    validation = [row for row, p in zip(train, by_split["train"])
                  if p["familyId"] in validation_families]
    # Use corpus keys rather than prompts containing unique scene identifiers.
    def key(p):
        row = corpus[p["review"]["scenarioId"]]
        return sha(json.dumps({"input": row["input"], "presentation": row["presentation"]},
                              sort_keys=True).encode())
    blocked = {key(p) for p in by_split["eval"]}
    blocked.update(key(p) for p in by_split["train"] if p["familyId"] in validation_families)
    selected, lineage, seen, excluded = [], [], set(), 0
    for row, p in zip(train, by_split["train"]):
        if p["familyId"] in validation_families:
            continue
        fingerprint = sha(encode([row]))
        if key(p) in blocked or fingerprint in seen:
            excluded += 1
            continue
        selected.append(row)
        lineage.append(p)
        seen.add(fingerprint)
    if not selected or not validation or not test:
        raise ValueError("Train, validation and test must all remain nonempty")
    contents = {"train.jsonl": encode(selected), "validation.jsonl": encode(validation),
                "smoke.jsonl": encode(sorted(selected, key=lambda r: len(encode([r])))[:8]),
                "test.jsonl": encode(test), "train-provenance.jsonl": encode(lineage)}
    manifest = {"format": "wonderwilds-glimmer-bundle-1",
                "counts": {"train": len(selected), "validation": len(validation), "test": len(test)},
                "validationFamilies": sorted(validation_families),
                "excludedForValidationOverlapOrDuplicate": excluded,
                "sourceExportManifestSha256": sha((export / "manifest.json").read_bytes()),
                "sourceCorpusSha256": sha(corpus_path.read_bytes()),
                "files": {name: sha(data) for name, data in contents.items()},
                "note": "Synthetic authored scenes; batch acceptance is not blind human grading. Test stays local."}
    out.mkdir(parents=True, exist_ok=False)
    for name, data in contents.items():
        (out / name).write_bytes(data)
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    verify(out)
    return manifest


def verify(path):
    manifest = json.loads((path / "manifest.json").read_text())
    expected = {"train.jsonl", "validation.jsonl", "smoke.jsonl", "test.jsonl", "train-provenance.jsonl"}
    if manifest.get("format") != "wonderwilds-glimmer-bundle-1" or set(manifest["files"]) != expected:
        raise ValueError("Unknown bundle layout")
    for name, digest in manifest["files"].items():
        if sha((path / name).read_bytes()) != digest:
            raise ValueError(f"Bundle file changed: {name}")
    for name in ("train", "validation", "test", "smoke"):
        rows = read_rows(path / f"{name}.jsonl")
        if not rows:
            raise ValueError("Empty dataset")
        for row in rows:
            messages = row.get("messages", [])
            if [m.get("role") for m in messages] != ["system", "user", "assistant"]:
                raise ValueError("Expected system/user/assistant messages")
            if any(not isinstance(m.get("content"), str) or not m["content"] for m in messages):
                raise ValueError("Empty or non-text message")
            json.loads(messages[-1]["content"])
        if name != "smoke" and len(rows) != manifest["counts"][name]:
            raise ValueError("Bundle count mismatch")
    train = {sha(encode([r])) for r in read_rows(path / "train.jsonl")}
    if not {sha(encode([r])) for r in read_rows(path / "smoke.jsonl")} <= train:
        raise ValueError("Smoke data must be a subset of training")
    return manifest


def tokenize(row, tokenizer, limit):
    messages = row["messages"]
    # Mask complete system/user turns. Glimmer's open generation header can
    # retokenize when its recipient is appended; train the whole assistant turn.
    prompt = tokenizer.apply_chat_template(messages[:-1], tokenize=True, add_generation_prompt=False, return_dict=False)
    full = tokenizer.apply_chat_template(messages, tokenize=True, add_generation_prompt=False, return_dict=False)
    if full[:len(prompt)] != prompt or len(full) <= len(prompt):
        raise ValueError("Chat template is not prefix-stable; inspect assistant masking before training")
    if len(full) > limit:
        raise ValueError(f"Example has {len(full)} tokens, exceeds {limit}; no silent truncation")
    return {"input_ids": full, "attention_mask": [1] * len(full),
            "labels": [-100] * len(prompt) + full[len(prompt):]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--export", type=Path)
    parser.add_argument("--corpus", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--verify", type=Path)
    args = parser.parse_args()
    if args.verify:
        result = verify(args.verify)
    else:
        if not all((args.export, args.corpus, args.out)):
            parser.error("--export, --corpus and --out are required")
        result = prepare(args.export, args.corpus, args.out)
    print(json.dumps(result, indent=2))
