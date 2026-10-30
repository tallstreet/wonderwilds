"""Local evaluation selection, strict output conversion, grading and paired reports."""
import copy
import importlib.util
import json
import math
from pathlib import Path
import re
from types import SimpleNamespace

from data import encode, read_rows, sha, verify
from worker import MODEL, REVISION

HERE = Path(__file__).resolve().parent
DEFAULT_EXPERIMENT = "qa/story-eval/experiments/glimmer-modal-r1"
DEFAULT_RUN = "qa/story-eval/experiments/broader-reference-quality-2026-10-30-r2/run"


def workflow():
    spec = importlib.util.spec_from_file_location("ww_story_workflow", HERE.parent / "run.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def identifier(value):
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}", value):
        raise ValueError("Expected a run ID, not a path")
    return value


PROMPT_VARIANTS = ("original", "speakers-v1", "speakers-unique-v1")
SPEAKER_RULE = """Speaker constraints: allowedSpeakerIds lists exactly the characters physically
present in input.here.characters. Every dialogue speakerId must belong to this list.
Characters mentioned in mission participants, memory, inventory or recent events
are not allowed to speak unless also listed in allowedSpeakerIds. You may mention
absent characters, but do not give them dialogue. If allowedSpeakerIds is empty,
use narration only. Keep requiredEntries unchanged. Do not include
allowedSpeakerIds in the returned scene."""


UNIQUE_ENTRY_RULE = """Presentation constraints: Every presentation entryId must be unique
within this scene, across both narration and dialogue. A speakerId identifies a
character; an entryId identifies one entry. Never reuse an entryId for another
line, even when the same character speaks again. Preserve every requiredEntries
object exactly and include it once; reserve its entryId so no other entry uses it.
Give each additional entry a distinct entryId that does not collide with a reserved
ID. Do not repeat a presentation entry or dialogue line. Do not copy the same text
between speakers or between narration and dialogue. Each additional entry must
contribute distinct information; omit redundant entries instead of padding the
presentation array. These rules do not permit editing or omitting requiredEntries.
Before returning JSON, check that all entryIds are distinct and that no optional
entry repeats another entry's text. Return only the scene, without this checklist."""


def speaker_prompt(request, variant="speakers-v1"):
    """Derive constraints from verified input only, without touching source prompts."""
    if variant not in ("speakers-v1", "speakers-unique-v1"):
        raise ValueError("Unknown speaker prompt variant")
    result = copy.deepcopy(request)
    messages = result["messages"]
    if [m["role"] for m in messages] != ["system", "user"]:
        raise ValueError("Speaker variant expects one system and one user message")
    payload = json.loads(messages[1]["content"])
    ids = [character["id"] for character in payload["input"]["here"]["characters"]]
    if any(not isinstance(value, str) or not value for value in ids) or len(set(ids)) != len(ids):
        raise ValueError("Invalid present-character IDs")
    payload["allowedSpeakerIds"] = ids
    messages[0]["content"] += "\n\n" + SPEAKER_RULE
    messages[1]["content"] = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    if variant == "speakers-unique-v1":
        messages[0]["content"] += "\n\n" + UNIQUE_ENTRY_RULE
    result["promptVersion"] += "+" + variant
    return result


def make_suite(experiment, run, split="validation", limit=0, max_new_tokens=2048, prompt_variant="original"):
    if split not in ("validation", "test") or limit < 0 or not 1 <= max_new_tokens <= 4096:
        raise ValueError("Invalid split, limit or token budget")
    if prompt_variant not in PROMPT_VARIANTS:
        raise ValueError("Unknown prompt variant")
    bundle, export = experiment / "bundle", experiment / "export"
    manifest = verify(bundle)
    if sha((export / "manifest.json").read_bytes()) != manifest["sourceExportManifestSha256"]:
        raise ValueError("Wrong source export")
    if sha((run / "corpus.jsonl").read_bytes()) != manifest["sourceCorpusSha256"]:
        raise ValueError("Wrong source corpus")
    source_manifest, corpus, original_requests = workflow().load_run(run)
    corpus_map = {r["scenarioId"]: r for r in corpus}
    requests = {}
    for r in original_requests:
        requests.setdefault(r["scenarioId"], r)
    source_split = "train" if split == "validation" else "eval"
    source_rows = read_rows(export / f"{source_split}.jsonl")
    lineage = sorted((r for r in read_rows(export / "provenance.jsonl") if r["split"] == source_split),
                     key=lambda r: r["row"])
    if [p["row"] for p in lineage] != list(range(1, len(source_rows) + 1)):
        raise ValueError("Invalid export lineage")
    selected = [(row, p) for row, p in zip(source_rows, lineage)
                if split == "test" or p["familyId"] in manifest["validationFamilies"]]
    if [row for row, _ in selected] != read_rows(bundle / f"{split}.jsonl"):
        raise ValueError("Selected rows differ from the verified bundle")
    training_ids = {p["review"]["scenarioId"] for p in read_rows(bundle / "train-provenance.jsonl")}
    prompts, seen = [], set()
    for row, p in selected:
        sid = p["review"]["scenarioId"]
        if sid in seen or sid in training_ids:
            raise ValueError("Duplicate evaluation scenario or training overlap")
        seen.add(sid)
        original = requests[sid]
        messages = row["messages"][:-1]
        if messages != original["messages"] or corpus_map[sid]["familyId"] != p["familyId"]:
            raise ValueError("Prompt or scenario provenance changed")
        prompts.append({"scenarioId": sid, "promptVersion": original["promptVersion"],
                        "messages": messages})
    total = len(prompts)
    if limit:
        prompts = prompts[:limit]
    if not prompts:
        raise ValueError("No evaluation prompts")
    if prompt_variant != "original":
        prompts = [speaker_prompt(request, prompt_variant) for request in prompts]
    return {**({"promptVariant": prompt_variant} if prompt_variant != "original" else {}),
            "format": "wonderwilds-glimmer-eval-1", "split": split,
            "availableScenarios": total, "requests": prompts,
            "trainingBundleSha256": sha((bundle / "manifest.json").read_bytes()),
            "sourceCorpusSha256": manifest["sourceCorpusSha256"],
            "gradingCorpusSha256": source_manifest["corpusSha256"],
            "gradingRequestsSha256": source_manifest["requestsSha256"],
            "model": MODEL, "revision": REVISION,
            "settings": {"maxNewTokens": max_new_tokens, "doSample": False, "seed": 42,
                         "quantization": "nf4-double-bf16", "maxInputTokens": 4096}}


def parse_response(raw, truncated=False, error=None):
    if error or truncated:
        return f"GENERATION_FAILURE: {error or 'token_limit'}\n{raw}"
    # Preserve raw Harmony output separately; extract only a final user-directed turn.
    marker = "assistant to=user<|message|>"
    if raw.startswith(" to=user<|message|>"):
        content = raw[len(" to=user<|message|>"):]
    elif marker in raw:
        content = raw.rsplit(marker, 1)[1]
    else:
        content = raw  # Plain JSON is also accepted; no fence stripping or JSON repair.
    if "<|eot|>" in content:
        content, tail = content.split("<|eot|>", 1)
        if tail.strip():
            return raw
    try:
        result = json.loads(content)
        return result if isinstance(result, dict) else raw
    except (ValueError, TypeError):
        return raw


def candidates(suite, result):
    expected = [r["scenarioId"] for r in suite["requests"]]
    if result["suiteSha256"] != sha(encode([suite])):
        raise ValueError("Result belongs to another evaluation suite")
    if [r["scenarioId"] for r in result["responses"]] != expected:
        raise ValueError("Result must cover every requested scenario exactly once, in order")
    variant = "adapter" if result["adapterRun"] else "base"
    prompts = {r["scenarioId"]: r for r in suite["requests"]}
    return [{"scenarioId": r["scenarioId"], "candidateId": variant,
             "generator": {"provider": "modal", "model": MODEL + "@" + REVISION,
                           "promptVersion": prompts[r["scenarioId"]]["promptVersion"],
                           "adapterRun": result["adapterRun"], "suiteSha256": result["suiteSha256"]},
             "output": parse_response(r["raw"], r["truncated"], r["error"])}
            for r in result["responses"]]


def grade_result(out, run):
    suite = json.loads((out / "suite.json").read_text())
    result = json.loads((out / "result.json").read_text())
    if sha((run / "corpus.jsonl").read_bytes()) != suite["sourceCorpusSha256"]:
        raise ValueError("Grading corpus differs from the evaluation suite")
    rows = candidates(suite, result)
    with (out / "candidates.jsonl").open("xb") as target:
        target.write(encode(rows))
    workflow().grade(SimpleNamespace(run=run, candidates=out / "candidates.jsonl",
                                     judgments=None, out=out / "graded"))


def compare(base, tuned, base_graded=None, tuned_graded=None):
    suites = [json.loads((p / "suite.json").read_text()) for p in (base, tuned)]
    results = [json.loads((p / "result.json").read_text()) for p in (base, tuned)]
    if suites[0] != suites[1]:
        raise ValueError("Compare identical suites and generation settings only")
    if results[0]["adapterRun"] or not results[1]["adapterRun"]:
        raise ValueError("Expected an untuned base and a tuned adapter")
    if results[0]["runtime"] != results[1]["runtime"]:
        raise ValueError("Runtime versions, evaluator or GPU differ; rerun under the same environment")
    all_candidates = [candidates(suites[i], results[i]) for i in range(2)]
    w = workflow()
    grades, qualities = [], []
    for i, path in enumerate((base_graded or base / "graded", tuned_graded or tuned / "graded")):
        stored = read_rows(path / "candidates.jsonl")
        hard = read_rows(path / "hard-results.jsonl")
        judgments = read_rows(path / "judgments.jsonl")
        meta = json.loads((path / "manifest.json").read_text())
        if stored != all_candidates[i] or meta["validationMode"] != "current-runtime":
            raise ValueError("Grades must describe these exact candidates under the current engine")
        if (meta["corpusSha256"] != suites[i]["gradingCorpusSha256"]
                or meta["requestsSha256"] != suites[i]["gradingRequestsSha256"]):
            raise ValueError("Grades used a different source corpus or prompts")
        for values, field in ((stored, "candidatesSha256"), (hard, "hardSha256"), (judgments, "judgmentsSha256")):
            if w.digest(values) != meta[field]:
                raise ValueError("Grading artifacts changed")
        keys = {(r["scenarioId"], r["candidateId"]) for r in stored}
        if len(hard) != len(keys) or {(r["scenarioId"], r["candidateId"]) for r in hard} != keys:
            raise ValueError("Incomplete or duplicate hard grades")
        grades.append({r["scenarioId"]: r["passed"] for r in hard})
        checked = w.checked_judgments(judgments, {(r["scenarioId"], r["candidateId"]): r for r in stored})
        qualities.append({sid: row for (sid, _), row in checked.items()})
    n = len(suites[0]["requests"])
    summaries = []
    for i, result in enumerate(results):
        measured = [r for r in result["responses"] if r["error"] is None]
        latencies = sorted(r["seconds"] for r in measured)
        summaries.append({"requested": n, "hardPassed": sum(grades[i].values()),
                          "hardPassRate": sum(grades[i].values()) / n,
                          "errors": sum(r["error"] is not None for r in result["responses"]),
                          "truncated": sum(r["truncated"] for r in result["responses"]),
                          "latencyMeasured": len(measured),
                          "generationP50Seconds": latencies[math.ceil(.5 * len(latencies)) - 1] if latencies else None,
                          "generationP95Seconds": latencies[math.ceil(.95 * len(latencies)) - 1] if latencies else None,
                          "meanOutputTokens": sum(r["outputTokens"] for r in measured) / len(measured) if measured else None,
                          "qualityJudged": len(qualities[i])})
    quality = {}
    common = qualities[0].keys() & qualities[1].keys()
    for criterion in w.CRITERIA:
        quality[criterion] = {"pairedJudgments": len(common)}
        for i, label in enumerate(("base", "adapter")):
            quality[criterion][label] = {verdict: sum(qualities[i][sid]["criteria"][criterion]["verdict"] == verdict for sid in common)
                                        for verdict in ("pass", "fail", "uncertain")}
    return {"split": suites[0]["split"], "base": summaries[0], "adapter": summaries[1],
            "improvedScenarioIds": [s for s in grades[0] if not grades[0][s] and grades[1][s]],
            "regressedScenarioIds": [s for s in grades[0] if grades[0][s] and not grades[1][s]],
            "quality": quality,
            "note": "Mechanical validity is not narrative quality. Quality counts use only jointly judged cases; judge consistency requires review. Latency excludes model loading, includes generation and CUDA synchronization; first-call warmup is included. Related authored journeys do not establish unseen-mission generalization."}
