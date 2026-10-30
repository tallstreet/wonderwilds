"""Modal entrypoint. Default: two optimizer steps, checkpoint reload and generation."""
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import uuid

import modal

HERE = Path(__file__).resolve().parent
UV_VERSION = "0.13.1"

image = (
    modal.Image.from_registry("nvidia/cuda:13.0.0-devel-ubuntu24.04", add_python="3.12")
    .apt_install("git", "build-essential")
    .env({"CUDA_HOME": "/usr/local/cuda", "TORCH_CUDA_ARCH_LIST": "9.0",
          "FLASH_ATTN_CUDA_ARCHS": "90",
          "CC": "/usr/bin/gcc", "CXX": "/usr/bin/g++", "HF_HOME": "/data/huggingface",
          "WANDB_MODE": "disabled", "HF_HUB_DISABLE_TELEMETRY": "1"})
    .uv_pip_install("torch==2.12.1", "torchvision==0.27.1",
                    index_url="https://download.pytorch.org/whl/cu130", uv_version=UV_VERSION)
    # Non-isolated CUDA builds require their build tools in this environment.
    .uv_pip_install("axolotl==0.19.0", "pyyaml==6.0.3", "packaging", "ninja",
                    "setuptools", "wheel",
                    uv_version=UV_VERSION)
    .uv_pip_install("flash-attn==2.8.3", extra_options="--no-build-isolation --verbose",
                    uv_version=UV_VERSION)
    .uv_pip_install(
        "cut-cross-entropy[transformers] @ git+https://github.com/axolotl-ai-cloud/ml-cross-entropy.git@4dfa52212d0552805aec5a6a2176c0abbea277d2",
        extra_options="--no-build-isolation", uv_version=UV_VERSION,
    )
    .run_commands(
        # Modal 1.6.3 copies its pinned uv executable here, outside PATH.
        "/.uv/uv pip check --system",
        "python -c 'from transformers import MuseGlimmerForConditionalGeneration'",
    )
    .add_local_file(HERE / "data.py", "/opt/wonderwilds/data.py")
    .add_local_file(HERE / "worker.py", "/opt/wonderwilds/worker.py")
    .add_local_file(HERE / "glimmer-qlora.yaml", "/opt/wonderwilds/glimmer-qlora.yaml")
)
app = modal.App("wonderwilds-glimmer-qlora")
cache = modal.Volume.from_name("wonderwilds-glimmer-data")
runs = modal.Volume.from_name("wonderwilds-glimmer-runs")


@app.function(image=image, gpu="H100", cpu=8, memory=65536,
              timeout=3600, retries=0, max_containers=1,
              volumes={"/data": cache, "/runs": runs})
def train(payload: dict, run_id: str, full: bool):
    import hashlib
    import sys
    if not re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9_-]{0,79}", run_id):
        raise ValueError("Invalid run ID")
    expected = {"manifest.json", "train.jsonl", "validation.jsonl", "smoke.jsonl"}
    if set(payload) != expected:
        raise ValueError("Upload only the explicit training bundle files")
    manifest = json.loads(payload["manifest.json"])
    for name in expected - {"manifest.json"}:
        if hashlib.sha256(payload[name]).hexdigest() != manifest["files"][name]:
            raise ValueError("Upload hash mismatch")
    run = Path("/runs") / run_id
    run.mkdir(exist_ok=False)
    (run / "data").mkdir()
    (run / "source").mkdir()
    for name in ("worker.py", "data.py", "glimmer-qlora.yaml"):
        (run / "source" / name).write_bytes((Path("/opt/wonderwilds") / name).read_bytes())
    for name, content in payload.items():
        (run / "data" / name).write_bytes(content)
    status = {"runId": run_id, "mode": "full" if full else "smoke", "status": "running"}
    (run / "status.json").write_text(json.dumps(status, indent=2))
    runs.commit()
    try:
        command = [sys.executable, "/opt/wonderwilds/worker.py", str(run)]
        if full:
            command.append("--full")
        subprocess.run(command, check=True)
        status["status"] = "completed"
    except Exception as error:
        status.update(status="failed", errorType=type(error).__name__)
        raise
    finally:
        (run / "status.json").write_text(json.dumps(status, indent=2))
        cache.commit()
        runs.commit()
    return status


@app.local_entrypoint()
def main(bundle: str = "qa/story-eval/experiments/glimmer-modal-r1/bundle",
         full: bool = False, run_id: str = "", dry_run: bool = False):
    spec = importlib.util.spec_from_file_location("glimmer_data", HERE / "data.py")
    data = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(data)
    path = Path(bundle)
    manifest = data.verify(path)
    run_id = run_id or ("full-" if full else "smoke-") + uuid.uuid4().hex[:12]
    if not re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9_-]{0,79}", run_id):
        raise ValueError("Invalid run ID")
    print(json.dumps({"runId": run_id, "mode": "full" if full else "smoke",
                      "counts": manifest["counts"], "gpu": "H100",
                      "timeoutSeconds": 3600, "dryRun": dry_run}, indent=2))
    if dry_run:
        return
    # The final test targets and human review notes never leave the local machine.
    payload = {name: (path / name).read_bytes() for name in
               ("manifest.json", "train.jsonl", "validation.jsonl", "smoke.jsonl")}
    print(train.remote(payload, run_id, full))


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Local validation only; use modal run to launch")
    parser.add_argument("--bundle", default="qa/story-eval/experiments/glimmer-modal-r1/bundle")
    parser.add_argument("--dry-run", action="store_true", required=True)
    args = parser.parse_args()
    main(bundle=args.bundle, dry_run=True)
