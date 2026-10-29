"""Offline content validation/reference simulation; Rust remains gameplay authority."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[2]
VERSION = 'wonderwilds-story-1.0.0'


def load(relative):
    return json.loads((ROOT / relative).read_text())


def discover(seed, region, owned, pending=None, last=None):
    """Reference algorithm. Persist seed/pending; declining never consumes a draw."""
    roster = [c['id'] for c in load('content/creatures/friends.v1.json')['creatures']
              if c['regionId'] == region]
    if not roster:
        raise ValueError('unknown region')
    if pending is not None:
        if pending not in roster or pending in owned:
            raise ValueError('invalid pending discovery')
        return seed, pending
    missing = [friend for friend in roster if friend not in owned]
    if region == 'meadow' and 'pip' in missing:
        return seed, 'pip'
    candidates = missing or [friend for friend in roster if friend != last]
    seed = (1664525 * seed + 1013904223) % (2 ** 32)
    return seed, candidates[seed % len(candidates)]


def validate_development(development=None, friends=None, adventure=None):
    """Check narrative sidecar without granting gameplay rewards or evidence."""
    development = development if development is not None else load('content/creatures/development.v1.json')
    friends = friends if friends is not None else load('content/creatures/friends.v1.json')
    adventure = adventure if adventure is not None else load('content/story/adventure.v1.json')
    errors = []
    def require(condition, message):
        if not condition:
            errors.append(message)
    require(development.get('schemaVersion') == '1.0', 'unsupported development schema')
    require(development.get('contentVersion') == VERSION, 'development content version mismatch')
    require(development.get('reviewStatus') == 'pending', 'development approval requires external review')
    require(development.get('activation') == 'disabled_until_integration_and_review', 'unreviewed development activated')
    characters = development.get('characters', [])
    roster = {c['id']: c for c in friends['creatures']}
    ids = [c.get('creatureId') for c in characters]
    require(len(ids) == len(set(ids)) and set(ids) == set(roster), 'development roster mismatch')
    flags = {flag for beat in adventure['beats'] for flag in beat['completionFlags']}
    maximum = load('content/story/progression.v1.json')['friendship']['maximum']
    moment_ids = []
    for character in characters:
        for field in ['motivation', 'growthOpportunity', 'voice', 'returnGreeting', 'supportedCompletionDialogue']:
            value = character.get(field)
            require(isinstance(value, str) and bool(value.strip()), 'missing character ' + field)
        moments = character.get('friendshipMoments', [])
        require(len(moments) == 3, 'expected three friendship moments')
        thresholds = []
        for moment in moments:
            moment_ids.append(moment.get('id'))
            threshold = moment.get('minimumFriendship')
            require(type(threshold) is int and 1 <= threshold <= maximum, 'invalid friendship threshold')
            thresholds.append(threshold)
            gates = moment.get('requiredStoryFlags', [])
            require(isinstance(gates, list) and all(flag in flags for flag in gates), 'unknown character story gate')
            dialogue = moment.get('dialogue')
            require(isinstance(dialogue, str) and bool(dialogue.strip()), 'missing character dialogue')
            require(set(moment) == {'id', 'minimumFriendship', 'requiredStoryFlags', 'dialogue'}, 'narrative moment contains gameplay fields')
        require(thresholds == [1, 2, 3], 'friendship moments must follow distinct helping milestones')
        creature = roster.get(character.get('creatureId'))
        if creature and moments:
            require(moments[-1].get('requiredStoryFlags') == [creature['transformation']['requiredStoryFlag']], 'reflection must follow regional finale')
    require(all(isinstance(i, str) and bool(i) for i in moment_ids) and len(moment_ids) == len(set(moment_ids)), 'invalid or duplicate character moment ID')
    for field in ['motivation', 'growthOpportunity', 'voice']:
        values = [c.get(field) for c in characters]
        require(len(values) == len(set(values)), 'character ' + field + ' must be individual')
    return errors


def validate_training(progression=None):
    progression = progression if progression is not None else load('content/story/progression.v1.json')
    policy = progression.get('characterProgression', {})
    errors = []
    def require(condition, message):
        if not condition:
            errors.append(message)
    levels = policy.get('levels', [])
    require(levels == [0, 10, 20, 30, 40], 'invalid character thresholds')
    require(policy.get('rulesVersion') == 'character-training-1.0.0', 'unknown training rules')
    require(policy.get('supportedCompletionMultiplier') == 1, 'training support penalty')
    require(policy.get('repeatMilestoneXp') == 0, 'training XP farming')
    training = policy.get('training', [])
    ids = [t.get('id') for t in training]
    require(len(ids) == len(set(ids)), 'duplicate training ID')
    mappings = {'pip': ('M06', 'missing_quantity'), 'tansy': ('M05', 'pattern_sort'),
                'nib': ('M08', 'equal_sharing'), 'flutter': ('M09', 'pattern_sort')}
    for friend, mapping in mappings.items():
        milestones = sorted([t for t in training if t.get('creatureId') == friend], key=lambda t: t.get('requiredLevel', 0))
        require([t.get('requiredLevel') for t in milestones] == [1, 2, 3, 4], 'missing training milestone')
        xp = 0
        for t in milestones:
            require((t.get('skillId'), t.get('templateId')) == mapping, 'unsupported training skill/template')
            require(type(t.get('xp')) is int and t.get('xp') == 10, 'invalid training XP')
            require(isinstance(t.get('reactionId'), str) and bool(t.get('reactionId')), 'missing training reaction')
            xp += t.get('xp', 0) if type(t.get('xp')) is int else 0
        require(xp >= 40, 'unreachable character level five')
    require(len(training) == 16 and all(t.get('creatureId') in mappings for t in training), 'invalid training roster')
    return errors


def validate(adventure=None, friends=None, assets=None):
    adventure = adventure or load('content/story/adventure.v1.json')
    friends = friends or load('content/creatures/friends.v1.json')
    assets = assets or load('content/manifests/story-assets.json')
    errors = []
    def require(condition, message):
        if not condition:
            errors.append(message)
    for data in [adventure, friends, assets]:
        require(data.get('schemaVersion') == '1.0', 'unsupported schema version')
        require(data.get('contentVersion') == VERSION, 'content version mismatch')
    beats = {b['id']: b for b in adventure['beats']}
    require(len(beats) == len(adventure['beats']), 'duplicate beat ID')
    asset_index = {a['id']: a for a in assets['assets']}
    require(len(asset_index) == len(assets['assets']), 'duplicate asset ID')
    require(len(friends['creatures']) == 12, 'expected twelve friends')
    ids = [c['id'] for c in friends['creatures']]
    require(len(ids) == len(set(ids)), 'duplicate creature ID')
    require(friends['preservedAvatarIds'] == ['rose','sky','gold','poppy','wren','clover','mochi'], 'legacy avatar IDs changed')
    reward_ids = []
    quests = load('content/story/help-quests.v1.json')['quests']
    quest_ids = {q['id'] for q in quests}
    require(len(quest_ids) == len(quests) == 12, 'invalid friendship quest IDs')
    for quest in quests:
        require(quest['creatureId'] in ids, 'unknown quest creature')
        require(all(b in beats for b in quest['requiresBeatIds']), 'unknown quest story prerequisite')
        require(quest['supportAvailable'] and quest['supportedCompletionBefriends'], 'unsupported friendship route')
        require(quest['repeatRewards'] == [], 'repeat friendship rewards forbidden')
        reward_ids += [r['id'] for r in quest['rewards']]
    reached = set()
    while True:
        available = {bid for bid,b in beats.items() if set(b['requiresBeatIds']) <= reached}
        if available <= reached:
            break
        reached |= available
    require(reached == set(beats), 'story has cycle or unreachable beat')
    require(adventure['endingBeatId'] in reached, 'ending unreachable')
    for b in beats.values():
        require(all(p in beats for p in b['requiresBeatIds']), 'dangling beat dependency')
        require(not b['requiredCreatureIds'], 'essential story gated by creature')
        require(not b['requiredMasteryIds'], 'story gated by academic mastery')
        require(b['supportedCompletion']['allowed'] and b['supportedCompletion']['grantsSameStoryRewards'], 'supported story route missing')
        require(not b['supportedCompletion']['independentEvidence'], 'support falsely independent')
        require(b['fallbackDialogue'] == b['dialogue'] and bool(b['dialogue']), 'authored fallback absent')
        require(all(a in asset_index for a in b['assetIds']), 'unknown story asset')
        require(all(step in b['requiresBeatIds'] or step in reached for step in b['finaleSteps']), 'unknown finale step')
        reward_ids += [r['id'] for r in b['rewards']]
    require(len(reward_ids) == len(set(reward_ids)), 'duplicate reward definition')
    for chapter in adventure['chapters']:
        require(sum(c['regionId'] == chapter['id'] for c in friends['creatures']) == 4, 'expected four regional friends')
        require(chapter['entryBeatId'] in beats and chapter['finaleBeatId'] in beats, 'unknown chapter anchor')
        require(set(chapter['beatIds']) == {b['id'] for b in beats.values() if b['chapterId'] == chapter['id']}, 'chapter membership mismatch')
        require(beats[chapter['finaleBeatId']]['finaleSteps'] != [], 'finale must combine prior tasks')
    flags = {flag for b in beats.values() for flag in b['completionFlags']}
    require(set(adventure['ending']['requiredFlags']) <= flags, 'ending flags unattainable')
    for c in friends['creatures']:
        require(c['helpQuestId'] in quest_ids, 'unknown help quest')
        require(c['discovery']['eligibleAfterBeat'] in beats, 'unknown discovery gate')
        require(c['assetId'] in asset_index and c['transformation']['assetId'] in asset_index, 'unknown creature asset')
        require(c['transformation']['requiredStoryFlag'] in flags, 'unknown transformation gate')
        require(c['assistance']['optionalOnly'] and c['assistance']['marksSupport'] and bool(c['assistance']['alternative']), 'unsafe assistance gate')
        require(c['transformation']['reversible'] and c['transformation']['cosmeticOnly'], 'transformation bypasses story')
    for asset in assets['assets']:
        if asset['path'] is not None:
            path = (ROOT / asset['path']).resolve()
            require(path.is_relative_to(ROOT), 'asset path escapes repository')
            require(path.is_file(), 'missing declared asset file')
            if path.is_file():
                require(hashlib.sha256(path.read_bytes()).hexdigest() == asset['sha256'], 'asset hash mismatch')
        else:
            require(asset['status'] == 'missing' and not asset['approved'], 'absent art claimed approved')
    require(not assets['releaseReady'], 'unreviewed assets cannot release')
    require(adventure['activation'] == 'disabled_until_integration_and_review', 'unreviewed story activated')
    manifest = load('content/story/manifest.v1.json')
    paths = ['content/story/adventure.v1.json', 'content/story/progression.v1.json',
             'content/story/help-quests.v1.json', 'content/creatures/friends.v1.json',
             'content/manifests/story-assets.json']
    payload = b''.join(p.encode() + b'\0' + (ROOT / p).read_bytes() for p in paths)
    require(manifest['sha256'] == hashlib.sha256(payload).hexdigest(), 'pack hash mismatch')
    require(set(manifest['assetIds']) == set(asset_index), 'manifest assets mismatch')
    require(manifest['educatorApproval']['status'] == 'pending', 'approval requires external review')
    progression = load('content/story/progression.v1.json')
    require(progression['explorerLevels'] == sorted(set(progression['explorerLevels'])), 'levels must strictly increase')
    require(progression['xpPolicy']['repeatQuestXp'] == 0, 'repeated XP farming')
    require(progression['xpPolicy']['supportedCompletionMultiplier'] == 1, 'support penalises story rewards')
    require(progression['friendship']['inactivityLoss'] == 0, 'inactivity penalty forbidden')
    errors.extend(validate_training())
    errors.extend(validate_development(friends=friends, adventure=adventure))
    return errors


if __name__ == '__main__':
    errors = validate()
    if errors:
        raise SystemExit('\n'.join(errors))
    print('Story content valid: 3 chapters, 18 beats, 12 friends; release remains disabled.')
