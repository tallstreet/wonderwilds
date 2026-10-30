"""GPU-side training and fresh-process adapter verification."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

from data import encode, read_rows, tokenize

MODEL = "meta-models/Muse-Glimmer-30B"
REVISION = "a4e59da52a7bc87ae7251dd5545c0dd437c44b68"


def reload_adapter(run):
    import torch
    from peft import PeftModel
    from safetensors.torch import load_file
    from transformers import AutoTokenizer, BitsAndBytesConfig, MuseGlimmerForConditionalGeneration
    config = json.loads((run / "resolved.json").read_text())
    tokenizer = AutoTokenizer.from_pretrained(config["base_model"])
    quant = BitsAndBytesConfig(load_in_4bit=True, bnb_4bit_quant_type="nf4",
                              bnb_4bit_use_double_quant=True,
                              bnb_4bit_compute_dtype=torch.bfloat16,
                              bnb_4bit_quant_storage=torch.bfloat16)
    base = MuseGlimmerForConditionalGeneration.from_pretrained(
        config["base_model"], dtype=torch.bfloat16, quantization_config=quant,
        device_map={"": 0})
    adapter = run / "adapter"
    saved = load_file(str(adapter / "adapter_model.safetensors"))
    if not saved or not any("lora_B" in k and torch.count_nonzero(v).item() for k, v in saved.items()):
        raise ValueError("Adapter has no trained LoRA B weights")
    model = PeftModel.from_pretrained(base, adapter).eval()
    from peft import get_peft_model_state_dict
    loaded = get_peft_model_state_dict(model)
    if saved.keys() != loaded.keys():
        raise ValueError("Adapter reload keys differ")
    for key, tensor in saved.items():
        if not torch.equal(tensor, loaded[key].detach().cpu().to(tensor.dtype)):
            raise ValueError(f"Adapter weight did not reload: {key}")
    messages = read_rows(run / "data/smoke.jsonl")[0]["messages"][:-1]
    ids = tokenizer.apply_chat_template(messages, tokenize=True, add_generation_prompt=True,
                                        return_tensors="pt", return_dict=False).to("cuda")
    with torch.inference_mode():
        output = model.generate(input_ids=ids, attention_mask=torch.ones_like(ids),
                                max_new_tokens=128, do_sample=False,
                                pad_token_id=tokenizer.eos_token_id,
                                eos_token_id=[tokenizer.eos_token_id, tokenizer.convert_tokens_to_ids("<|eot|>")])
    text = tokenizer.decode(output[0, ids.shape[1]:], skip_special_tokens=True)
    if not text.strip():
        raise ValueError("Reloaded adapter produced no visible output")
    (run / "reload-check.json").write_text(json.dumps(
        {"adapterKeysVerified": len(saved), "generatedText": text,
         "note": "Bounded training-prompt generation checks loading, not scene quality."}, indent=2))


def execute(run, full):
    import importlib.metadata
    import yaml
    from huggingface_hub import snapshot_download
    from transformers import AutoTokenizer
    config = yaml.safe_load(Path("/opt/wonderwilds/glimmer-qlora.yaml").read_text())
    tokenizer = AutoTokenizer.from_pretrained(MODEL, revision=REVISION)
    names = ["train", "validation"] if full else ["smoke"]
    stats = {}
    for name in names:
        records = [tokenize(row, tokenizer, config["sequence_len"])
                   for row in read_rows(run / "data" / f"{name}.jsonl")]
        (run / f"{name}-tokens.jsonl").write_bytes(encode(records))
        stats[name] = {"examples": len(records), "maxTokens": max(len(r["input_ids"]) for r in records),
                       "supervisedTokens": sum(sum(v != -100 for v in r["labels"]) for r in records)}
    (run / "tokenization.json").write_text(json.dumps(stats, indent=2))
    # Fetch weights only after the template, loss mask and token bounds pass.
    model_path = snapshot_download(MODEL, revision=REVISION)
    config.update(base_model=model_path, output_dir=str(run / "adapter"),
                  dataset_prepared_path=str(run / "prepared"),
                  datasets=[{"path": str(run / f"{'train' if full else 'smoke'}-tokens.jsonl"),
                             "ds_type": "json", "type": None}])
    if full:
        config["test_datasets"] = [{"path": str(run / "validation-tokens.jsonl"),
                                    "ds_type": "json", "type": None}]
    else:
        config.update(max_steps=2, gradient_accumulation_steps=1,
                      eval_strategy="no", save_steps=2)
    (run / "resolved.json").write_text(json.dumps(config, indent=2))
    (run / "config.yaml").write_text(yaml.safe_dump(config))
    (run / "environment.json").write_text(json.dumps({
        "baseModel": MODEL, "revision": REVISION,
        "packages": {name: importlib.metadata.version(name) for name in
                     ("axolotl", "torch", "transformers", "peft", "bitsandbytes", "flash-attn")},
    }, indent=2))
    with (run / "train.log").open("w") as log:
        process = subprocess.Popen(["axolotl", "train", str(run / "config.yaml")],
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            print(line, end="", flush=True)
            log.write(line)
            log.flush()
        if process.wait():
            raise subprocess.CalledProcessError(process.returncode, process.args)
    subprocess.run([sys.executable, __file__, str(run), "--reload"], check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run", type=Path)
    parser.add_argument("--full", action="store_true")
    parser.add_argument("--reload", action="store_true")
    args = parser.parse_args()
    if args.reload:
        reload_adapter(args.run)
    else:
        execute(args.run, args.full)
