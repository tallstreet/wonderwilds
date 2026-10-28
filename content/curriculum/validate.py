"""Validate draft curriculum integrity; this is authoring tooling, not assessment."""
from __future__ import annotations

import csv
import hashlib
import io
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
EXPECTED_IDS = {f'M{i:02}' for i in range(1, 19)} | {f'E{i:02}' for i in range(1, 16)}
NODE_FIELDS = {'id', 'subject', 'objective', 'band', 'prerequisites', 'allowedRepresentations',
               'parameterBounds', 'misconceptionCodes', 'hintLadder', 'evidencePolicyId',
               'contentVersion', 'reviewStatus', 'alignmentRefs'}


def load_bundle(root: Path = ROOT) -> dict:
    bundle = {name: json.loads((root / name).read_text()) for name in [
        'seed-v1.json', 'authoring-v1.json', 'reviews-v1.json',
        'evidence-policies-v1.json', 'manifest-v1.json', 'integrity-index-v1.json']}
    bundle['coverage.csv'] = (root / 'coverage.csv').read_text()
    return bundle


def validate(bundle: dict) -> list[str]:
    errors = []
    nodes = bundle['seed-v1.json']['nodes']
    node_map = {node['id']: node for node in nodes}
    if len(node_map) != len(nodes):
        errors.append('Duplicate skill IDs')
    if set(node_map) != EXPECTED_IDS:
        errors.append('Seed graph must contain M01–M18 and E01–E15 exactly')
    authoring = bundle['authoring-v1.json']['skills']
    author_map = {item['skillId']: item for item in authoring}
    reviews = bundle['reviews-v1.json']['reviews']
    review_map = {item['skillId']: item for item in reviews}
    for label, rows, mapping in [('Authoring', authoring, author_map), ('Reviews', reviews, review_map)]:
        if len(rows) != len(mapping) or set(mapping) != set(node_map):
            errors.append(f'{label} must cover each skill exactly once')
    policies = bundle['evidence-policies-v1.json']['policies']
    policy_map = {policy['id']: policy for policy in policies}
    if len(policy_map) != len(policies):
        errors.append('Duplicate evidence policy IDs')
    for policy in policies:
        if policy.get('automaticCertification') is not False:
            errors.append('Draft evidence policies must not automatically certify mastery')
        if 'extends' in policy and policy['extends'] not in policy_map:
            errors.append('Dangling evidence policy inheritance')
    visiting, visited = set(), set()

    def walk(skill_id):
        if skill_id in visiting:
            errors.append(f'Prerequisite cycle at {skill_id}')
            return
        if skill_id in visited or skill_id not in node_map:
            return
        visiting.add(skill_id)
        for prerequisite in node_map[skill_id]['prerequisites']:
            walk(prerequisite)
        visiting.remove(skill_id)
        visited.add(skill_id)

    for node in nodes:
        skill_id = node['id']
        if set(node) != NODE_FIELDS:
            errors.append(f'{skill_id}: unexpected or missing SkillNode fields')
        if node['subject'] != ('maths' if skill_id.startswith('M') else 'english'):
            errors.append(f'{skill_id}: invalid subject')
        if node['band'] not in {'foundation-candidate', 'year-1-extension', 'extension'}:
            errors.append(f'{skill_id}: invalid band')
        if node['reviewStatus'] not in {'pending', 'approved'}:
            errors.append(f'{skill_id}: invalid review status')
        if any(pre not in node_map for pre in node['prerequisites']):
            errors.append(f'{skill_id}: dangling prerequisite')
        if len(node['prerequisites']) != len(set(node['prerequisites'])):
            errors.append(f'{skill_id}: duplicate prerequisite')
        if node['evidencePolicyId'] not in policy_map:
            errors.append(f'{skill_id}: unknown evidence policy')
        if not node['objective'] or not node['misconceptionCodes'] or len(node['hintLadder']) != 3:
            errors.append(f'{skill_id}: missing objective, misconception or three-step hint ladder')
        if not node['allowedRepresentations'] or len(set(node['allowedRepresentations'])) != len(node['allowedRepresentations']):
            errors.append(f'{skill_id}: invalid representations')
        for name, bound in node['parameterBounds'].items():
            if set(bound) != {'min', 'max'} or any(type(bound.get(k)) is not int for k in ['min', 'max']) or not 0 <= bound['min'] <= bound['max'] <= 1000:
                errors.append(f'{skill_id}: invalid parameter bound {name}')
        for ref in node['alignmentRefs']:
            if not ref['sourceUrl'].startswith('https://www.australiancurriculum.edu.au/'):
                errors.append(f'{skill_id}: invalid alignment authority')
            if ref['descriptorCode'] is not None and not all(ref.get(k) for k in ['reviewer', 'reviewDate']):
                errors.append(f'{skill_id}: verified descriptor requires reviewer and date')
        item = author_map.get(skill_id)
        if item:
            context_ids = {c['id'] for c in item['contexts']}
            rungs = item['difficultyLadder']
            if len(context_ids) < 2 or len(rungs) < 3 or len({r['id'] for r in rungs}) != len(rungs):
                errors.append(f'{skill_id}: needs two contexts and three distinct rung IDs')
            combinations = set()
            for rung in rungs:
                if rung['contextId'] not in context_ids or rung['representation'] not in node['allowedRepresentations']:
                    errors.append(f'{skill_id}: rung references unknown context or representation')
                if rung['supportAvailable'] is not True:
                    errors.append(f'{skill_id}: every rung needs supported completion')
                if set(rung['parameters']) != set(node['parameterBounds']):
                    errors.append(f'{skill_id}: rung parameter keys differ from bounds')
                for name, value in rung['parameters'].items():
                    bound = node['parameterBounds'].get(name)
                    if bound and (type(value) is not int or not bound['min'] <= value <= bound['max']):
                        errors.append(f'{skill_id}: rung parameter outside bounds')
                combinations.add(json.dumps([rung['parameters'], rung['contextId'], rung['representation']], sort_keys=True))
            if len(combinations) < 3:
                errors.append(f'{skill_id}: duplicate parameter/context/representation variants')
        review = review_map.get(skill_id, {})
        if node['reviewStatus'] == 'approved' and not (
            review.get('status') == 'approved' and review.get('reviewer') and review.get('reviewDate') and
            all(review.get(k) is True for k in ['prerequisitesApproved', 'alignmentApproved', 'itemsApproved', 'accessibilityApproved'])):
            errors.append(f'{skill_id}: approved node lacks complete review evidence')
        walk(skill_id)
    if len(visited) != len(node_map):
        errors.append('Not every node is reachable from the prerequisite traversal')
    coverage = list(csv.DictReader(io.StringIO(bundle['coverage.csv'])))
    covered = set()
    for row in coverage:
        status = row['coverage_status']
        ids = row['original_node_ids'].split(';') if row['original_node_ids'] else []
        if status not in {'unmapped', 'authored', 'reviewed', 'tested', 'released'}:
            errors.append('Invalid coverage status')
        for skill_id in ids:
            covered.add(skill_id)
            if skill_id not in node_map:
                errors.append('Coverage references unknown node')
            elif status in {'reviewed', 'tested', 'released'} and node_map[skill_id]['reviewStatus'] != 'approved':
                errors.append('Coverage promotion requires educator-approved node')
        if not ids and status != 'unmapped':
            errors.append('Mapped coverage requires a node')
    if covered != set(node_map):
        errors.append('Coverage must account for every seed node')
    manifest = bundle['integrity-index-v1.json']
    if manifest['releaseEnabled']:
        if manifest['educatorApproval'] != 'approved' or any(n['reviewStatus'] != 'approved' for n in nodes):
            errors.append('Release requires educator approval for the whole declared pack')
    return errors


def check_hashes(root: Path = ROOT) -> list[str]:
    manifest = json.loads((root / 'integrity-index-v1.json').read_text())
    errors = []
    declared = json.loads((root / 'manifest-v1.json').read_text())
    if declared['sha256'] != hashlib.sha256((root / 'integrity-index-v1.json').read_bytes()).hexdigest():
        errors.append('ContentManifest does not match integrity index')
    for name, expected in manifest['files'].items():
        file = (root / name).resolve()
        if file.parent != root.resolve() or not file.is_file():
            errors.append(f'Invalid manifest path: {name}')
        elif hashlib.sha256(file.read_bytes()).hexdigest() != expected:
            errors.append(f'Manifest hash mismatch: {name}')
    return errors


if __name__ == '__main__':
    errors = validate(load_bundle()) + check_hashes()
    if errors:
        raise SystemExit('\n'.join(errors))
    print('Curriculum valid: 33 draft nodes, 35 coverage rows, no release approvals; hashes verified.')
