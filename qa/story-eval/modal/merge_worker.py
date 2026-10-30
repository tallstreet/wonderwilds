"""Merge on the original BF16 base, never on dequantized training weights."""
import argparse
import hashlib
import json
from pathlib import Path

from worker import MODEL, REVISION


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(8 * 1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def check_source(root):
    status = json.loads((root / 'status.json').read_text())
    env = json.loads((root / 'environment.json').read_text())
    if status['status'] != 'completed' or env['baseModel'] != MODEL or env['revision'] != REVISION:
        raise ValueError('Adapter must be completed and use the pinned base revision')
    adapter = root / 'adapter'
    for name in ('adapter_config.json', 'adapter_model.safetensors'):
        if not (adapter / name).is_file():
            raise ValueError('Missing adapter artifact: ' + name)
    return adapter


def check_config(value):
    if isinstance(value, dict):
        if value.get('quantization_config') is not None:
            raise ValueError('Export must not contain quantization configuration')
        for nested in value.values():
            check_config(nested)
    elif isinstance(value, list):
        for nested in value:
            check_config(nested)


def merge(destination, adapter_run):
    import torch
    from huggingface_hub import snapshot_download
    from peft import PeftModel, get_peft_model_state_dict
    from safetensors.torch import load_file
    from transformers import AutoProcessor, AutoTokenizer, MuseGlimmerForConditionalGeneration
    adapter = check_source(Path('/runs') / adapter_run)
    hashes = {name: digest(adapter / name) for name in ('adapter_config.json', 'adapter_model.safetensors')}
    base_path = snapshot_download(MODEL, revision=REVISION)
    check_config(json.loads((Path(base_path) / 'config.json').read_text()))
    base = MuseGlimmerForConditionalGeneration.from_pretrained(
        base_path, dtype=torch.bfloat16, device_map={'': 0}, attn_implementation='sdpa')
    model = PeftModel.from_pretrained(base, adapter, is_trainable=False)
    saved = load_file(str(adapter / 'adapter_model.safetensors'))
    loaded = get_peft_model_state_dict(model)
    if not saved or saved.keys() != loaded.keys() or any(
        not torch.equal(tensor, loaded[key].detach().cpu().to(tensor.dtype)) for key, tensor in saved.items()
    ):
        raise ValueError('Adapter tensor verification failed')
    del saved, loaded
    model = model.merge_and_unload(safe_merge=True).to(dtype=torch.bfloat16).eval()
    if any('lora_' in name for name, _ in model.named_parameters()):
        raise ValueError('Unmerged adapter parameters remain')
    check_config(model.config.to_dict())
    target = destination / 'model'
    target.mkdir(exist_ok=False)
    model.save_pretrained(target, safe_serialization=True, max_shard_size='5GB')
    AutoProcessor.from_pretrained(base_path).save_pretrained(target)
    AutoTokenizer.from_pretrained(base_path).save_pretrained(target)
    (destination / 'export.json').write_text(json.dumps({
        'baseModel': MODEL, 'revision': REVISION, 'adapterRun': adapter_run,
        'adapterHashes': hashes, 'dtype': 'bfloat16', 'quantization': None,
        'files': {p.name: digest(p) for p in sorted(target.iterdir()) if p.is_file()},
    }, indent=2))


def verify(destination):
    import torch
    from safetensors import safe_open
    from transformers import AutoTokenizer, MuseGlimmerForConditionalGeneration
    target = destination / 'model'
    manifest = json.loads((destination / 'export.json').read_text())
    for name, expected in manifest['files'].items():
        if digest(target / name) != expected:
            raise ValueError('Export file hash mismatch: ' + name)
    check_config(json.loads((target / 'config.json').read_text()))
    shards = list(target.glob('*.safetensors'))
    if not shards:
        raise ValueError('Missing exported weights')
    tensors = 0
    for shard in shards:
        with safe_open(shard, framework='pt', device='cpu') as handle:
            for key in handle.keys():
                dtype = handle.get_slice(key).get_dtype()
                if dtype.startswith('F') or dtype.startswith('BF'):
                    if dtype != 'BF16':
                        raise ValueError('Non-BF16 floating tensor: ' + key)
                if 'lora_' in key:
                    raise ValueError('Adapter tensor in merged checkpoint')
                tensors += 1
    model = MuseGlimmerForConditionalGeneration.from_pretrained(
        target, local_files_only=True, dtype=torch.bfloat16, device_map={'': 0}, attn_implementation='sdpa').eval()
    tokenizer = AutoTokenizer.from_pretrained(target, local_files_only=True)
    ids = tokenizer.apply_chat_template([{'role': 'user', 'content': 'Say hello.'}],
        add_generation_prompt=True, tokenize=True, return_tensors='pt', return_dict=False).to('cuda')
    with torch.inference_mode():
        output = model.generate(input_ids=ids, attention_mask=torch.ones_like(ids),
                                max_new_tokens=8, do_sample=False, pad_token_id=tokenizer.eos_token_id)
    if output.shape[1] <= ids.shape[1]:
        raise ValueError('Reloaded model produced no tokens')
    (destination / 'verification.json').write_text(json.dumps({
        'tensorsChecked': tensors, 'reload': 'passed', 'generatedTokens': output.shape[1] - ids.shape[1],
        'note': 'Load/generation smoke check only; not story quality or deployment-recipe validation.'}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--adapter-run')
    parser.add_argument('--verify', action='store_true')
    args = parser.parse_args()
    if args.verify:
        verify(args.destination)
    else:
        if not args.adapter_run or '/' in args.adapter_run or args.adapter_run in ('.', '..'):
            parser.error('Valid --adapter-run is required')
        merge(args.destination, args.adapter_run)
