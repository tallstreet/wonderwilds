"""Validate development artwork inventory without changing images or release packs."""
from pathlib import Path
import hashlib
import json
import struct

ROOT = Path(__file__).resolve().parents[3]


def validate():
    errors = []
    inventory = json.loads((ROOT / 'content/creatures/art/library.v1.json').read_text())
    roster = json.loads((ROOT / 'content/creatures/friends.v1.json').read_text())['creatures']
    ids = {friend['id'] for friend in roster}
    if inventory.get('schemaVersion') != '1.0' or inventory.get('reviewStatus') != 'pending':
        errors.append('invalid inventory version or review status')
    seen = []
    for sheet in inventory['sheets']:
        path = (ROOT / sheet['path']).resolve()
        if not path.is_relative_to(ROOT / 'frontend/public/assets/characters') or not path.is_file():
            errors.append(f'missing/invalid artwork path: {sheet["path"]}')
            continue
        raw = path.read_bytes()
        if raw[:8] != b'\x89PNG\r\n\x1a\n':
            errors.append('artwork must be PNG')
            continue
        width, height = struct.unpack('>II', raw[16:24])
        if [width, height] != sheet['dimensions'] or width < 2 or height < 2:
            errors.append('sheet dimensions must match inventory')
        if raw[25] not in (4, 6):
            errors.append('artwork must include an alpha channel')
        if hashlib.sha256(raw).hexdigest() != sheet['sha256']:
            errors.append(f'artwork hash mismatch: {sheet["path"]}')
        if len(sheet['characters']) != 4:
            errors.append('each regional sheet must contain four friends')
        for index, entry in enumerate(sheet['characters']):
            seen.append(entry['creatureId'])
            friend = next((f for f in roster if f['id'] == entry['creatureId']), None)
            if not friend or friend['regionId'] != sheet['regionId']:
                errors.append('unknown or wrong-region character')
            if entry['cell'] != index:
                errors.append('invalid row-major cell order')
            x, y, w, h = entry['frame']
            if min(x, y) < 0 or min(w, h) <= 0 or x + w > width or y + h > height:
                errors.append('artwork frame exceeds source bounds')
        if not sheet.get('provenancePath') or not (ROOT / sheet['provenancePath']).is_file():
            errors.append('generation provenance missing')
    if len(seen) != len(set(seen)) or set(seen) != ids:
        errors.append('artwork must cover the twelve friends exactly once')
    return errors


if __name__ == '__main__':
    errors = validate()
    if errors:
        raise SystemExit('\n'.join(errors))
    print('Character library artwork valid: twelve friends; human review pending.')
