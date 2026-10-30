"""Run base or adapter inference on Modal, then grade all responses locally."""
import json
from pathlib import Path
import subprocess
import sys
import uuid

import modal

HERE = Path(__file__).resolve().parent
# The Modal CLI also loads this file as a module rather than a Python script.
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
DEFAULT_EXPERIMENT = "qa/story-eval/experiments/glimmer-modal-r1"
DEFAULT_RUN = "qa/story-eval/experiments/broader-reference-quality-2026-10-30-r2/run"

# Only the local client constructs the image. Modal imports this entrypoint alone
# at /root/evaluate.py remotely; repository helper modules are not beside it.
image = None
if modal.is_local():
    import train as training
    image = training.image.add_local_file(HERE / "eval_worker.py", "/opt/wonderwilds/eval_worker.py")
app = modal.App("wonderwilds-glimmer-eval")
cache = modal.Volume.from_name("wonderwilds-glimmer-data")
runs = modal.Volume.from_name("wonderwilds-glimmer-runs")


@app.function(image=image, gpu="H100", cpu=8, memory=65536, timeout=3600,
              retries=0, max_containers=1,
              volumes={"/data": cache, "/runs": runs})
def infer(suite: dict, evaluation_id: str, adapter_run: str):
    # IDs are checked again inside the remote function before building paths.
    import re
    for value in (evaluation_id, adapter_run):
        if value and not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}", value):
            raise ValueError("Invalid evaluation/adapter run ID")
    directory = Path("/runs/evaluations") / evaluation_id
    directory.mkdir(parents=True, exist_ok=False)
    (directory / "suite.json").write_text(json.dumps(suite, indent=2))
    status = {"status": "running", "adapterRun": adapter_run}
    (directory / "status.json").write_text(json.dumps(status))
    runs.commit()
    try:
        command = [sys.executable, "/opt/wonderwilds/eval_worker.py", str(directory)]
        if adapter_run:
            command += ["--adapter-run", adapter_run]
        subprocess.run(command, check=True)
        status["status"] = "completed"
    except Exception:
        status["status"] = "failed"
        raise
    finally:
        (directory / "status.json").write_text(json.dumps(status))
        cache.commit()
        runs.commit()
    return json.loads((directory / "result.json").read_text())


@app.local_entrypoint()
def main(out: str, experiment: str = DEFAULT_EXPERIMENT, run: str = DEFAULT_RUN,
         split: str = "validation", adapter_run: str = "", limit: int = 0,
         max_new_tokens: int = 2048, dry_run: bool = False, prompt_variant: str = "original"):
    # Dataset selection and Rust grading run only on the local client.
    from eval_io import grade_result, identifier, make_suite
    if adapter_run:
        identifier(adapter_run)
    suite = make_suite(Path(experiment), Path(run), split, limit, max_new_tokens, prompt_variant)
    target = Path(out)
    if target.exists():
        raise FileExistsError("Choose a new output directory")
    print(json.dumps({"variant": "adapter" if adapter_run else "base", "split": split,
                      "scenarios": len(suite["requests"]), "settings": suite["settings"],
                      "promptVariant": prompt_variant, "dryRun": dry_run}, indent=2))
    if dry_run:
        return
    target.mkdir(parents=True)
    (target / "suite.json").write_text(json.dumps(suite, indent=2))
    evaluation_id = "eval-" + uuid.uuid4().hex[:12]
    (target / "remote.json").write_text(json.dumps({"evaluationId": evaluation_id,
                                                  "volumePath": f"evaluations/{evaluation_id}"}, indent=2))
    result = infer.remote(suite, evaluation_id, adapter_run)
    (target / "result.json").write_text(json.dumps(result, indent=2))
    grade_result(target, Path(run))
    print(f"Graded results: {target / 'graded/review.html'}")


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Local-only evaluation preflight")
    parser.add_argument("--experiment", default=DEFAULT_EXPERIMENT)
    parser.add_argument("--run", default=DEFAULT_RUN)
    parser.add_argument("--split", choices=("validation", "test"), default="validation")
    parser.add_argument("--prompt-variant", choices=("original", "speakers-v1", "speakers-unique-v1"), default="original")
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--max-new-tokens", type=int, default=2048)
    parser.add_argument("--dry-run", action="store_true", required=True)
    args = parser.parse_args()
    main(out="/tmp/wonderwilds-eval-unused-dry-run", **vars(args))
