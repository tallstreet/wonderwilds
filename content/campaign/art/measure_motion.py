#!/usr/bin/env python3
"""Preserve generated pixels; measure an inspected 20-pose atlas and publish metadata.

This is source/frame processing, not raster editing or visual approval. Review every
output and actual renderer preview before changing pending review states.
"""
import fcntl
import hashlib
import json
from pathlib import Path
import shutil
import sys
from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
ART = ROOT / 'content/campaign/art'
NAMES = ['idle-down','walk-down-a','walk-down-b','idle-right',
         'walk-right-a','walk-right-b','idle-up','walk-up-a',
         'walk-up-b','idle-left','walk-left-a','walk-left-b',
         'signature-anticipate','signature-release','signature-settle','cheer-prepare',
         'cheer-lift','cheer-settle','rest-settle','rest-asleep']

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def measure(request_path):
    request = json.loads(request_path.read_text())
    identity = request['id']
    source = Path(request['path'])
    assert source.is_file(), source
    original = Path(request.get('originalPath', source))
    raw = ART / 'sources' / (identity + '.png')
    if original != source:
        shutil.copyfile(original, ART / 'sources' / (identity + '-original.png'))
    shutil.copyfile(source, raw)
    public = ROOT / 'frontend/public/assets/campaign' / (identity + '.png')
    shutil.copyfile(raw, public)
    im = Image.open(raw).convert('RGBA')
    width, height = im.size
    alpha = im.getchannel('A')
    assert alpha.getextrema()[0] == 0, 'Not true transparent: use imagegen cleanup, never silently key pixels'
    pixels = alpha.load()
    cols = [round(n * width / 4) for n in range(5)]
    rows, boundary_counts = [0], []
    for n in range(1, 5):
        expected = round(n * height / 5)
        radius = round(height / 40)
        counts = [(sum(pixels[x,y] > 220 for x in range(width)), y)
                  for y in range(expected-radius, expected+radius+1)]
        minimum = min(count for count, y in counts)
        assert minimum <= width / 100, f'No clear row gutter {n}: {minimum} opaque pixels; inspect/regenerate'
        candidates = [y for count, y in counts if count == minimum]
        # Prefer the middle of the zero-density gutter nearest the expected grid.
        runs, run = [], []
        for y in candidates:
            if run and y != run[-1] + 1:
                runs.append(run); run = []
            run.append(y)
        runs.append(run)
        chosen = min(runs, key=lambda values: abs((values[0]+values[-1])/2-expected))
        rows.append(round((chosen[0]+chosen[-1])/2))
        boundary_counts.append(minimum)
    rows.append(height)
    frames = []
    for i, name in enumerate(NAMES):
        row, col = divmod(i, 4)
        x,y = cols[col], rows[row]
        w,h = cols[col+1]-x, rows[row+1]-y
        occupied = [(xx,yy) for yy in range(h) for xx in range(w) if pixels[x+xx,y+yy] > 220]
        assert len(occupied) > 100, f'Empty pose {name}'
        ground = max(yy for xx,yy in occupied)
        support = [xx for xx,yy in occupied if yy >= ground-8]
        foot = [round((min(support)+max(support))/2,1), ground]
        frames.append({'id':name,'rect':[x,y,w,h],'foot':foot})
    meta = {'id':identity,'character':request['character'],'url':'/assets/campaign/'+identity+'.png',
            'texture':'campaign-'+identity,'dimensions':[width,height],'nominalCell':width/4,'frames':frames}
    metadata = ART / (identity+'.frames.json')
    metadata.write_text(json.dumps(meta,indent=2)+'\n')
    prompt = ART / (identity+'.prompt.txt')
    prompt.write_text(request['prompt']+'\n')
    reference = Path(request['reference'])
    try: reference = reference.relative_to(ROOT)
    except ValueError: pass
    provenance = {'schemaVersion':'1.0','assetId':identity,'generatedAt':'2026-11-14',
        'generator':'image_gen.imagegen','reviewStatus':'pending','promptFile':str(prompt.relative_to(ROOT)),
        'rawSource':str(raw.relative_to(ROOT)),'processedAsset':str(public.relative_to(ROOT)),
        'sha256':digest(raw),'actualDimensions':[width,height],'requestedDimensions':[1536,1920],
        'frameMetadata':str(metadata.relative_to(ROOT)),
        'sourceReferences':[{'path':str(reference),'sha256':digest(ROOT/reference),'use':request.get('identity','Specified original companion identity, excluding other atlas characters.')}],
        'processing':'No raster editing: generated source copied byte-for-byte; high-alpha gutters/ground support measured for runtime frame rectangles and anchors.',
        'availablePoses':['idle-down','idle-right','idle-up','idle-left','two-stride-walk-down','two-stride-walk-right','two-stride-walk-up','two-stride-walk-left','three-frame-signature','three-frame-cheer','two-frame-rest'],
        'technicalInspection':'Twenty source cells measured; source visual and actual Phaser review pending.',
        'measurement':{'rowCuts':rows,'columnCuts':cols,'rowBoundaryOpaqueCounts':boundary_counts},
        'limitations':['Directional idle is one still per heading, not a breathing/blink cycle.','Two-stride walking is a minimal cycle; frame interpolation and four/eight-stride refinement remain review questions.','Human art/accessibility approval pending.']}
    if original != source:
        provenance['originalGenerationSource']=str((ART/'sources'/(identity+'-original.png')).relative_to(ROOT))
        cleanup = ART / (identity+'.cleanup-prompt.txt')
        cleanup.write_text(request['cleanupPrompt']+'\n')
        provenance['cleanupPromptFile']=str(cleanup.relative_to(ROOT))
        provenance['processing']='Imagegen background-cleanup edit; original generation and edited source preserved. Public PNG copied byte-for-byte from edited source, no programmatic pixel edits. Runtime frame rectangles/ground anchors measured separately.'
    (ART/(identity+'.provenance.json')).write_text(json.dumps(provenance,indent=2)+'\n')
    with open('/tmp/wonderwilds-motion-registry.lock','w') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX)
        sheets = [json.loads(path.read_text()) for path in sorted(ART.glob('*-motion-v1.frames.json'))]
        registry = ART/'motion-registry.v1.json'
        temporary = ART/'motion-registry.v1.tmp'
        temporary.write_text(json.dumps({'sheets':sheets},indent=2)+'\n')
        temporary.replace(registry)
    print(identity, width, height, '20 measured frames', 'rows',rows)

if __name__ == '__main__':
    for arg in sys.argv[1:]: measure(Path(arg))
