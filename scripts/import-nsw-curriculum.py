#!/usr/bin/env python3
"""Compile the supplied NSW authoring graph into private tasks and bounded public assets.

Unknown media/assessment specifications fail closed to explicit supported workshops.
The original source remains intact for authoring; it is never imported by the browser.
"""
import collections
import hashlib
import json
import pathlib
import re
import sys
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape

ROOT = pathlib.Path(__file__).resolve().parents[1]
PACK = ROOT / 'content/curriculum/nsw'
ASSETS = ROOT / 'frontend/public/assets/learning/nsw'
CHECK = '--check' in sys.argv

class Unsupported(ValueError):
    pass


def write(path, data):
    text = json.dumps(data, ensure_ascii=False, indent=2) + '\n'
    if CHECK:
        assert path.read_text() == text, f'Generated curriculum drift: {path}'
    else:
        path.write_text(text)


def asset(svg):
    if '<!' in svg or '<?' in svg:
        raise Unsupported('Unsupported SVG declaration')
    tree = ET.fromstring(svg)
    tags = {'svg', 'g', 'path', 'rect', 'circle', 'ellipse', 'line', 'polygon', 'polyline', 'text'}
    attrs = {'width', 'height', 'viewBox', 'x', 'y', 'x1', 'y1', 'x2', 'y2', 'cx', 'cy', 'r', 'rx', 'ry', 'points', 'd', 'fill', 'stroke', 'stroke-width', 'stroke-dasharray', 'transform', 'font-family', 'font-size', 'font-weight', 'text-anchor'}
    for node in tree.iter():
        if node.tag.split('}')[-1] not in tags or not set(node.attrib) <= attrs:
            raise Unsupported('Unsafe SVG element or attribute')
        if any(re.search(r'url\s*\(|javascript:|https?:|data:', v, re.I) for v in node.attrib.values()):
            raise Unsupported('External SVG reference')
    digest = hashlib.sha256(svg.encode()).hexdigest()
    ASSETS.mkdir(parents=True, exist_ok=True)
    path = ASSETS / f'{digest}.svg'
    if CHECK:
        assert path.read_text() == svg, f'Generated asset drift: {path}'
    else:
        path.write_text(svg)
    return {'kind': 'diagram', 'assetId': digest, 'description': 'Question diagram. Ask for an accessible supported activity if needed.'}


def icon_svg(name):
    icons = json.loads((PACK/'icons-v1.json').read_text())
    icon = icons.get(name)
    if not icon: raise Unsupported('Unverified icon: '+name)
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 '+str(icon['width'])+' '+str(icon['height'])+'">'+''.join('<path fill="#35684b" d="'+escape(p)+'"/>' for p in icon['paths'])+'</svg>'


def clock_coordinate(value):
    """Keep libm's insignificant last bits out of content-addressed SVG hashes."""
    # A millionth of a view-box unit is far below display precision. Strip both
    # trailing zeroes and negative zero so cardinal hands have stable spelling.
    text = f'{value:.6f}'.rstrip('0').rstrip('.')
    return '0' if text == '-0' else text


def visual(v):
    kind = v['kind']
    if kind == 'svg':
        return asset(v['svg'])
    body = ''
    view = v.get('view_box', [0, 0, 100, 100])
    if kind == 'object_set':
        if v.get('object') in ('shell', 'shells'):
            # A neutral shell silhouette; count and positions come only from the source.
            body = ''.join(f'<g transform="translate({o["x"]} {o["y"]})"><path d="M-5 0 Q-7-6-3-6 Q0-9 3-6 Q7-6 5 0 L1 5 L-1 5 Z" fill="#b98c62" stroke="#614a34" stroke-width="0.5"/><path d="M0 4V-6 M0 4L-4-4 M0 4L4-4" fill="none" stroke="#614a34" stroke-width="0.5"/></g>' for o in v['objects'])
        elif v.get('object') == 'duck':
            body = ''.join(f'<g transform="translate({o["x"]} {o["y"]})"><path d="M-6 1 Q-6 7 1 6 Q7 5 5 0 L5-2 Q8-7 3-7 Q-1-7 0-2 L-2 1Z" fill="#e4b946" stroke="#735822" stroke-width="0.5"/><path d="M5-5L9-3L5-2Z" fill="#b56b31"/><circle cx="4" cy="-5" r="0.5"/></g>' for o in v['objects'])
        elif v.get('object') == 'blocks':
            body = ''.join(f'<rect x="{o["x"]-5}" y="{o["y"]-5}" width="10" height="10" fill="#548dab" stroke="#254b3a"/>' for o in v['objects'])
        elif v.get('object') == 'buttons':
            body = ''.join(f'<g transform="translate({o["x"]} {o["y"]})"><circle r="5" fill="#548dab" stroke="#254b3a"/><circle cx="-1.5" r="0.8" fill="white"/><circle cx="1.5" r="0.8" fill="white"/></g>' for o in v['objects'])
        else:
            body = ''.join(f'<circle cx="{o["x"]}" cy="{o["y"]}" r="5" fill="#35684b"/>' for o in v['objects'])
    elif kind == 'polygon':
        body = '<polygon points="' + ' '.join(f'{x},{y}' for x,y in v['vertices']) + '" fill="#a8d2c0" stroke="#254b3a" stroke-width="2"/>'
    elif kind == 'circle':
        body = f'<circle cx="{v["centre"][0]}" cy="{v["centre"][1]}" r="{v["radius"]}" fill="#a8d2c0" stroke="#254b3a"/>'
    elif kind == 'analogue_clock':
        # Angles are the supplied geometry; labels never disclose the digital answer.
        import math
        body = '<circle cx="50" cy="50" r="46" fill="white" stroke="#254b3a"/>'
        for n in v['dial_labels']:
            a = n * math.pi / 6
            body += f'<text x="{clock_coordinate(50+38*math.sin(a))}" y="{clock_coordinate(53-38*math.cos(a))}" font-size="8" text-anchor="middle">{n}</text>'
        for key, length in [('hour',26),('minute',38)]:
            a = v[f'{key}_angle_degrees']*math.pi/180
            body += f'<line x1="50" y1="50" x2="{clock_coordinate(50+length*math.sin(a))}" y2="{clock_coordinate(50-length*math.cos(a))}" stroke="#254b3a" stroke-width="2"/>'
    elif kind in ('ten_frame', 'ten_frames', 'hundred_grid', 'array'):
        if kind == 'array':
            rows, cols = v['rows'], v['columns']; filled = set(range(rows*cols))
        elif kind == 'ten_frames':
            rows, cols = len(v['frames'])*3, 5
            filled = {f*15+c for f, frame in enumerate(v['frames']) for c in frame['occupied_cells']}
        else:
            rows, cols = v['rows'], v['columns']; filled = set(v.get('occupied_cells', v.get('shaded_cells', [])))
        view = [0,0,cols*12,rows*12]
        for i in range(rows*cols):
            x,y = (i%cols)*12, (i//cols)*12
            if kind != 'array': body += f'<rect x="{x}" y="{y}" width="12" height="12" fill="white" stroke="#254b3a" stroke-width="0.5"/>'
            if i in filled: body += f'<circle cx="{x+6}" cy="{y+6}" r="4" fill="#35684b"/>'
    elif kind in ('fraction_strip', 'partitioned_strip'):
        divisions = v.get('division_positions') or list(range(v['equal_parts']+1))
        total = divisions[-1]
        shaded = v['shaded_parts']; shaded = set(range(shaded)) if isinstance(shaded,int) else set(shaded)
        for n,(a,b) in enumerate(zip(divisions,divisions[1:])):
            body += f'<rect x="{a/total*96+2}" y="30" width="{(b-a)/total*96}" height="30" fill="{ "#35684b" if n in shaded else "white"}" stroke="#254b3a" stroke-width="0.3"/>'
    else:
        raise Unsupported(f'Renderer needed: {kind}')
    return asset('<svg xmlns="http://www.w3.org/2000/svg" viewBox="'+' '.join(map(str,view))+'">'+body+'</svg>')


def option(o, index):
    if o.get('audio') or o.get('text_display') is False or str(o.get('text_display','')).startswith('hidden'):
        raise Unsupported('Reviewed option audio needed')
    result = {'id': o['id'], 'label': o.get('text') or f'Picture {index+1}'}
    if 'visual_spec' in o:
        result['diagram'] = visual(o['visual_spec'])['assetId']
    if len(result['label']) > 120:
        raise Unsupported('Long option needs expanded control')
    return result


def convert(skill, item):
    score = item['scoring']; typ = score['type']; stimuli = []
    source = item.get('stimulus', {})
    forbidden = set(source) - {'text','visual_spec','table','materials_note','materials','read_target_aloud','option_audio_allowed','preserve_line_breaks','context_note'}
    if forbidden:
        raise Unsupported('Media/control needed: ' + ', '.join(sorted(forbidden)))
    for key in ['text', 'context_note', 'materials_note']:
        if source.get(key):
            if not isinstance(source[key],str): raise Unsupported('Structured text needs renderer')
            stimuli.append({'kind':'passage' if key=='text' else 'text','text':source[key]})
    if 'visual_spec' in source:
        diagram = visual(source['visual_spec'])
        if source['visual_spec'].get('brief_exposure_ms'):
            diagram['exposureMs'] = source['visual_spec']['brief_exposure_ms']
        stimuli.append(diagram)
    if 'table' in source:
        t = source['table']
        if not isinstance(t,dict) or 'headers' not in t: raise Unsupported('Structured table needs renderer')
        stimuli.append({'kind':'table','headings':list(map(str,t['headers'])),'rows':[list(map(str,r)) for r in t['rows']]})
    if typ in ('exact_option','exact_set'):
        options = [option(o,i) for i,o in enumerate(item['options'])]
        interaction = {'kind':'single_choice' if typ=='exact_option' else 'multi_choice','options':options}
        answer = {'kind':'choice','choiceId':score['correct_option_id']} if typ=='exact_option' else {'kind':'choices','choiceIds':score['correct_option_ids']}
    elif typ == 'numeric':
        if score.get('tolerance',0) != 0: raise Unsupported('Numeric tolerance needs assessor')
        formats = item.get('input',{}).get('formats',['integer','decimal','fraction'])
        fmt = 'fraction' if 'fraction' in formats else 'decimal' if 'decimal' in formats else 'integer'
        interaction = {'kind':'number_entry','fields':[{'id':'value','label':'Your answer','format':fmt,'unit':str(item.get('input',{}).get('unit') or '')}]}
        answer = {'kind':'numbers','values':[{'fieldId':'value','value':str(score['value'])}], 'sourceScoring':score, 'sourceInput':item.get('input',{})}
    elif typ == 'exact':
        interaction = {'kind':'text_entry','label':'Your answer','maxLength':120}
        answer = {'kind':'text','value':score['accepted'][0], 'sourceScoring':score}
    elif typ == 'sequence':
        interaction = {'kind':'order','items':[option(o,i) for i,o in enumerate(item['items'])]}
        answer = {'kind':'order','itemIds':score['accepted_sequences'][0], 'acceptedSequences':score['accepted_sequences']}
    elif typ in ('mapping','constraints'):
        if 'items' not in item or 'categories' not in item: raise Unsupported('Matching control needs renderer')
        interaction = {'kind':'assign','items':[option(o,i) for i,o in enumerate(item['items'])], 'targets':[{'id':o['id'],'label':o['text']} for o in item['categories']], 'capacity':min(20,len(item['items']))}
        if typ == 'constraints':
            answer = {'rule':'counts','groupCounts':score['group_counts']}
        else:
            answer = {'kind':'assign','assignments':[{'objectId':k,'targetId':v} for k,v in score['mapping'].items()], 'acceptedMappings':score.get('accepted_mappings',[score['mapping']])}
    else:
        raise Unsupported('Human observation required' if typ=='rubric' else 'Geometry interaction/assessor needed')
    suggestion = item.get('visual_enhancement', {})
    if suggestion.get('recommended_mode') == 'optional_context_icon' and suggestion.get('icon_names'):
        name = suggestion['icon_names'][0]
        if name in json.loads((PACK/'icons-v1.json').read_text()):
            stimuli.append(asset(icon_svg(name)))
    return interaction, stimuli, answer


def build():
    raw = (PACK/'source-v1.json').read_bytes(); data = json.loads(raw)
    nodes=[]; skills=[]; issues=[]; ids={s['id'] for s in data['skills']}
    assert len(ids)==len(data['skills'])==736
    for s in data['skills']:
        assert len(s['practice_items'])>=10
        assert set(s['prerequisites']) <= ids
        nodes.append({'id':s['id'],'subject':s['subject'],'objective':s['name'],'band':'Foundation' if s['year']=='K' else 'Year '+s['year'], 'prerequisites':s['prerequisites'], 'allowedRepresentations':s['representations'],'reviewStatus':'pending', 'learningResources':[{'title':r['title'],'url':r['url'],'provider':r['provider'],'audience':r['audience']} for r in s['learning_resources']]})
        exercises=[]
        for i in s['practice_items']:
            try:
                interaction,stimuli,answer = convert(s,i)
                reason=None
            except Unsupported as error:
                reason=str(error); issues.append({'skillId':s['id'],'itemId':i['id'],'reason':reason})
                interaction={'kind':'workshop','scaffold':'This activity needs a grown-up or materials that are not available on screen yet. Explore the skill together; finishing this activity does not assess ability.', 'rubric':['Discuss the question and choose accessible materials together.','Use support to explore the idea; this is not an independent assessment.']}
                if i['scoring']['type']=='rubric':
                    interaction['rubric']=[c['description'] for c in i['scoring']['criteria']][:6]
                stimuli=[]; answer=None
            exercises.append({'id':i['id'],'facet':i.get('assessment_facet',s['focus_area']) if isinstance(i.get('assessment_facet',s['focus_area']),str) else s['focus_area'], 'difficultyRungs':[i.get('difficulty','core')], 'assessmentMode':'automatic' if answer is not None else 'human_review', 'limitations':[] if reason is None else [reason], 'items':[{'title':s['name'],'prompt':i['question'],'representation':i['type'],'stimuli':stimuli,'interaction':interaction,'answer':answer,'hints':['Read the question again. What information do you have?','Try a smaller example or explore the idea with a grown-up.'],'explanation':i.get('explanation') or 'Explore this task with a grown-up.', 'semanticId':i['item_fingerprint']}]})
        skills.append({'skillId':s['id'],'exercises':exercises})
    write(PACK/'graph-v1.json',{'nodes':nodes,'reviewStatus':'pending','releaseEnabled':False})
    write(PACK/'exercises-v1.json',{'schemaVersion':'1.0','reviewStatus':'pending','releaseEnabled':False,'skills':skills})
    write(PACK/'delivery-report-v1.json',{'sourceSha256':hashlib.sha256(raw).hexdigest(),'skills':len(nodes),'items':sum(len(s['exercises']) for s in skills),'automaticItems':sum(e['assessmentMode']=='automatic' for s in skills for e in s['exercises']),'supportedItems':len(issues),'limitations':issues})
    print(f'{len(nodes)} skills; {sum(len(s["exercises"]) for s in skills)} items; {len(issues)} explicit supported routes')

if __name__=='__main__': build()
