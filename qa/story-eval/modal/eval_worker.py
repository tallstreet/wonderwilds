"""GPU inference for a frozen prompt suite; no approved answers are accepted."""
import argparse
import importlib.metadata
import json
from pathlib import Path
import time

from data import encode, sha
from worker import MODEL, REVISION


def evaluate(directory, adapter_run):
    import torch
    from huggingface_hub import snapshot_download
    from transformers import AutoTokenizer, BitsAndBytesConfig, MuseGlimmerForConditionalGeneration
    suite = json.loads((directory / "suite.json").read_text())
    if suite["model"] != MODEL or suite["revision"] != REVISION:
        raise ValueError("Unsupported base checkpoint")
    settings = suite["settings"]
    if settings["doSample"] or settings["quantization"] != "nf4-double-bf16":
        raise ValueError("Unsupported generation settings")
    for request in suite["requests"]:
        if set(request) != {"scenarioId", "promptVersion", "messages"} or [m["role"] for m in request["messages"]] != ["system", "user"]:
            raise ValueError("Only input prompts may reach evaluation")
    if adapter_run:
        root = Path("/runs") / adapter_run
        status = json.loads((root / "status.json").read_text())
        environment = json.loads((root / "environment.json").read_text())
        if status["status"] != "completed" or environment["revision"] != REVISION or environment["baseModel"] != MODEL:
            raise ValueError("Adapter run must be completed and use the pinned base")
        if sha((root / "data/manifest.json").read_bytes()) != suite["trainingBundleSha256"]:
            raise ValueError("Adapter was trained with a different bundle")
        adapter = root / "adapter"
    torch.manual_seed(settings["seed"])
    model_path = snapshot_download(MODEL, revision=REVISION)
    tokenizer = AutoTokenizer.from_pretrained(model_path)
    quant = BitsAndBytesConfig(load_in_4bit=True, bnb_4bit_quant_type="nf4",
                              bnb_4bit_use_double_quant=True,
                              bnb_4bit_compute_dtype=torch.bfloat16,
                              bnb_4bit_quant_storage=torch.bfloat16)
    model = MuseGlimmerForConditionalGeneration.from_pretrained(
        model_path, dtype=torch.bfloat16, quantization_config=quant, device_map={"": 0})
    adapter_hashes = None
    if adapter_run:
        from peft import PeftModel, get_peft_model_state_dict
        from safetensors.torch import load_file
        saved = load_file(str(adapter / "adapter_model.safetensors"))
        model = PeftModel.from_pretrained(model, adapter)
        loaded = get_peft_model_state_dict(model)
        if not saved or saved.keys() != loaded.keys() or any(
            not torch.equal(tensor, loaded[key].detach().cpu().to(tensor.dtype)) for key, tensor in saved.items()
        ):
            raise ValueError("Adapter weights failed reload verification")
        adapter_hashes = {name: sha((adapter / name).read_bytes()) for name in
                          ("adapter_config.json", "adapter_model.safetensors")}
        del saved, loaded
    model.eval()
    stop_ids = list({tokenizer.eos_token_id, tokenizer.convert_tokens_to_ids("<|eot|>")})
    result = {"suiteSha256": sha(encode([suite])), "adapterRun": adapter_run,
              "adapterHashes": adapter_hashes,
              "runtime": {"gpu": torch.cuda.get_device_name(0),
                          "packages": {name: importlib.metadata.version(name) for name in
                                       ("torch", "transformers", "peft", "bitsandbytes")},
                          "evaluatorSha256": sha(Path(__file__).read_bytes())},
              "responses": []}
    for request in suite["requests"]:
        row = {"scenarioId": request["scenarioId"], "raw": "", "error": None,
               "truncated": False, "outputTokens": 0, "seconds": None}
        ids = output = None
        try:
            ids = tokenizer.apply_chat_template(request["messages"], tokenize=True,
                                                add_generation_prompt=True, return_tensors="pt",
                                                return_dict=False).to("cuda")
            if ids.shape[1] > settings["maxInputTokens"]:
                raise ValueError("Input exceeds token budget; refusing truncation")
            torch.cuda.synchronize()
            started = time.perf_counter()
            with torch.inference_mode():
                output = model.generate(input_ids=ids, attention_mask=torch.ones_like(ids),
                                        max_new_tokens=settings["maxNewTokens"], do_sample=False,
                                        pad_token_id=tokenizer.eos_token_id, eos_token_id=stop_ids)
            torch.cuda.synchronize()
            row["seconds"] = time.perf_counter() - started
            generated = output[0, ids.shape[1]:].tolist()
            row["outputTokens"] = len(generated)
            row["truncated"] = bool(generated and len(generated) >= settings["maxNewTokens"] and generated[-1] not in stop_ids)
            row["raw"] = tokenizer.decode(generated, skip_special_tokens=False)
        except Exception as error:
            row["error"] = type(error).__name__ + ": " + str(error)
        finally:
            del ids, output
            if row["error"]:
                torch.cuda.empty_cache()
        result["responses"].append(row)
        temporary = directory / "result.tmp"
        temporary.write_text(json.dumps(result, indent=2))
        temporary.replace(directory / "result.json")
        print(f"{len(result['responses'])}/{len(suite['requests'])} {row['scenarioId']} error={row['error'] is not None} truncated={row['truncated']}", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--adapter-run", default="")
    args = parser.parse_args()
    evaluate(args.directory, args.adapter_run)
