"""Register an intentionally completed native-motion atlas; never approve artwork."""
import argparse
import hashlib
import json
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[3]


def register(provenance_path):
    path = (ROOT / provenance_path).resolve()
    if not path.is_relative_to(ROOT / 'content/campaign/art'):
        raise ValueError('provenance must be inside campaign art resources')
    provenance = json.loads(path.read_text())
    metadata_path = ROOT / provenance['frameMetadata']
    metadata = json.loads(metadata_path.read_text())
    raw = (ROOT / provenance['rawSource']).read_bytes()
    processed = (ROOT / provenance['processedAsset']).read_bytes()
    digest = hashlib.sha256(processed).hexdigest()
    dimensions = list(struct.unpack('>II', processed[16:24]))
    if raw != processed or digest != provenance['sha256']:
        raise ValueError('raw/public bytes and declared hash must agree before registration')
    if dimensions != provenance['actualDimensions'] or dimensions != metadata['dimensions']:
        raise ValueError('declared source dimensions must agree')
    for frame in metadata['frames']:
        x, y, width, height = frame['rect']
        if min(x, y) < 0 or min(width, height) <= 0 or x + width > dimensions[0] or y + height > dimensions[1]:
            raise ValueError(f'frame outside source: {frame["id"]}')
        if not 0 <= frame['foot'][0] <= width or not 0 <= frame['foot'][1] <= height:
            raise ValueError(f'foot anchor outside source: {frame["id"]}')
    manifest_path = ROOT / 'content/campaign/art/manifest.v1.json'
    manifest = json.loads(manifest_path.read_text())
    asset_id = provenance['assetId']
    entry = dict(id=asset_id, kind='animation-development', source=provenance['processedAsset'],
                 url=metadata['url'], frame=metadata['frames'][0]['rect'],
                 sourceDimensions=dimensions, sha256=digest,
                 provenance=str(path.relative_to(ROOT)), frameMetadata=provenance['frameMetadata'],
                 availablePoses=provenance['availablePoses'],
                 desiredPoses=['directional-idle', 'directional-walk', 'signature', 'cheer', 'rest'],
                 targetDimensions=[96, 96], worldFootprint=[32, 24], milestone='M5',
                 status='generated-development', reviewStatus='pending',
                 visualInspection=provenance.get('technicalInspection', 'Actual source and renderer inspection pending.'))
    manifest['assets'] = [asset for asset in manifest['assets'] if asset['id'] != asset_id]
    manifest['assets'].append(entry)
    manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Registered {asset_id}; human review remains pending.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('provenance', help='Repository-relative provenance JSON for intentional registration')
    register(parser.parse_args().provenance)
