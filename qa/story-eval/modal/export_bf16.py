"""Export a trained adapter as a standalone, unquantized BF16 Modal checkpoint."""
import json
from pathlib import Path
import re
import subprocess
import sys

import modal

HERE = Path(__file__).resolve().parent
image = None
if modal.is_local():
    sys.path.insert(0, str(HERE))
    import train
    image = train.image.add_local_file(HERE / 'merge_worker.py', '/opt/wonderwilds/merge_worker.py')

app = modal.App('wonderwilds-glimmer-export')
cache = modal.Volume.from_name('wonderwilds-glimmer-data')
runs = modal.Volume.from_name('wonderwilds-glimmer-runs')


def checked_id(value):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]{0,79}', value):
        raise ValueError('Expected a run/export ID, not a path')
    return value


@app.function(image=image, gpu='H100', cpu=8, memory=196608, timeout=7200,
              retries=0, max_containers=1, volumes={'/data': cache, '/runs': runs})
def export(adapter_run: str, export_id: str):
    checked_id(adapter_run)
    checked_id(export_id)
    destination = Path('/runs/exports') / export_id
    destination.mkdir(parents=True, exist_ok=False)
    status = {'status': 'running', 'adapterRun': adapter_run, 'modelPath': str(destination / 'model')}
    def save_status():
        (destination / 'status.json').write_text(json.dumps(status, indent=2))
        runs.commit()
    save_status()
    try:
        worker = '/opt/wonderwilds/merge_worker.py'
        subprocess.run([sys.executable, worker, str(destination), '--adapter-run', adapter_run], check=True)
        # A fresh process proves loading requires only the exported checkpoint.
        subprocess.run([sys.executable, worker, str(destination), '--verify'], check=True)
        status['status'] = 'completed'
    except Exception as error:
        status.update(status='failed', errorType=type(error).__name__)
        raise
    finally:
        cache.commit()
        save_status()
    return status


@app.local_entrypoint()
def main(adapter_run: str = 'full-6584b280eaaf', export_id: str = '', dry_run: bool = False):
    checked_id(adapter_run)
    export_id = checked_id(export_id or adapter_run + '-bf16')
    print(json.dumps({'volume': 'wonderwilds-glimmer-runs',
                      'volumeRelativePath': f'exports/{export_id}/model',
                      'adapterRun': adapter_run, 'dryRun': dry_run}, indent=2))
    if not dry_run:
        print(json.dumps(export.remote(adapter_run, export_id), indent=2))
