"""Validate campaign art audit without editing legacy manifests or approving art."""
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[3]


def validate():
    manifest = json.loads((ROOT / 'content/campaign/art/manifest.v1.json').read_text())
    campaign = json.loads((ROOT / 'content/campaign/missing-moonlight.v2.json').read_text())
    errors = []
    seen = set()
    for asset in manifest['assets']:
        name = asset['id']
        if name in seen:
            errors.append(f'duplicate ID: {name}')
        seen.add(name)
        if asset['reviewStatus'] != 'pending':
            errors.append(f'unreviewed audit must retain pending: {name}')
        for key in ['targetDimensions', 'worldFootprint']:
            if len(asset[key]) != 2 or any(not isinstance(n, int) or n < 0 for n in asset[key]):
                errors.append(f'invalid {key}: {name}')
        if asset['source'] is None:
            if asset['status'] != 'missing' or asset['frame'] is not None:
                errors.append(f'missing artwork presented as available: {name}')
            continue
        path = (ROOT / asset['source']).resolve()
        if not path.is_relative_to(ROOT / 'frontend/public/assets') or not path.is_file():
            errors.append(f'invalid asset path: {name}')
            continue
        raw = path.read_bytes()
        if raw[:8] != b'\x89PNG\r\n\x1a\n':
            errors.append(f'expected PNG: {name}')
            continue
        dims = list(struct.unpack('>II', raw[16:24]))
        if dims != asset['sourceDimensions'] or hashlib.sha256(raw).hexdigest() != asset['sha256']:
            errors.append(f'source dimensions/hash drift: {name}')
        x, y, width, height = asset['frame']
        if min(x, y) < 0 or min(width, height) <= 0 or x + width > dims[0] or y + height > dims[1]:
            errors.append(f'frame outside source: {name}')
        if not (ROOT / asset['provenance']).is_file():
            errors.append(f'missing provenance: {name}')
        if asset.get('frameMetadata'):
            metadata = json.loads((ROOT / asset['frameMetadata']).read_text())
            for entry in metadata['frames']:
                x, y, width, height = entry['rect']
                if min(x, y) < 0 or min(width, height) <= 0 or x + width > dims[0] or y + height > dims[1]:
                    errors.append(f'animation frame outside source: {name}/{entry["id"]}')
                foot_x, foot_y = entry['foot']
                if not 0 <= foot_x <= width or not 0 <= foot_y <= height:
                    errors.append(f'animation foot anchor outside frame: {name}/{entry["id"]}')
    for friend in campaign['companions']:
        for name in ['companion-' + friend['id'], friend['id'] + '-wayfinder']:
            if name not in seen:
                errors.append(f'missing companion inventory: {name}')
    affinities = {friend['affinity'] for friend in campaign['companions']}
    if {name[9:] for name in seen if name.startswith('affinity-')} != affinities:
        errors.append('affinity inventory differs from campaign')
    motion_registry = json.loads((ROOT / 'content/campaign/art/motion-registry.v1.json').read_text())
    expected_motion = {friend['id'] + suffix for friend in campaign['companions']
                       for suffix in ['', '-wayfinder']}
    registered = [sheet['character'] for sheet in motion_registry['sheets']]
    if len(registered) != 24 or set(registered) != expected_motion:
        errors.append('motion registry must cover twelve base and twelve Wayfinder sets')
    for sheet in motion_registry['sheets']:
        metadata = json.loads((ROOT / f'content/campaign/art/{sheet["id"]}.frames.json').read_text())
        if metadata != sheet or len(sheet['frames']) != 20:
            errors.append(f'motion registry differs from measured twenty-pose metadata: {sheet["id"]}')
    for sheet in ['cast', 'props', 'wayfinders-meadow', 'finale', 'orchard',
                  'wayfinders-orchard', 'wayfinders-cove', 'wayfinders-sail', 'seals',
                  'affinities-a', 'affinities-b', 'landmarks', 'effects']:
        provenance = json.loads((ROOT / f'content/campaign/art/{sheet}-v1.provenance.json').read_text())
        raw = (ROOT / provenance['rawSource']).read_bytes()
        processed = (ROOT / provenance['processedAsset']).read_bytes()
        if raw != processed or hashlib.sha256(raw).hexdigest() != provenance['sha256']:
            errors.append(f'generated {sheet} raw/processed provenance mismatch')
        if not provenance.get('prompt') or provenance['reviewStatus'] != 'pending':
            errors.append(f'generated {sheet} prompt/review provenance missing')
    return errors


if __name__ == '__main__':
    errors = validate()
    if errors:
        raise SystemExit('\n'.join(errors))
    print('Campaign art inventory valid; technical evidence recorded separately; human review pending.')
