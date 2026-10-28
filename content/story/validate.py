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
    return errors


if __name__ == '__main__':
    errors = validate()
    if errors:
        raise SystemExit('\n'.join(errors))
    print('Story content valid: 3 chapters, 18 beats, 12 friends; release remains disabled.')
