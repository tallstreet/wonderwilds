# Character development drafts

`friends.v1.json` remains the gameplay roster: IDs, discovery, assistance and
transformation rules are unchanged. `development.v1.json` is an additive authored
narrative sidecar for those same twelve friends. It supplies a personal motivation,
a gentle growth opportunity, a distinct voice and three friendship moments.

Moments require authoritative friendship levels 1, 2 and 3 respectively. The third
also requires the character's regional lantern flag, matching the existing
transformation gate. These are presentation conditions, not commands: showing a
line must never award XP, friendship, a transformation or learning evidence.
Supported help earns the same story appreciation; leaving causes no friendship
loss. Return greetings carry no time or streak condition.

The sidecar remains pending review, with release activation disabled. The local
development journal previews dialogue by selecting the highest moment eligible
from confirmed server ownership, friendship points and required story flags.
It does not infer points from completed quests or browser-local learning history.
When authoritative friendship is absent, no earned moment is shown. Static profile
motivations can still appear without implying a moment was earned.

The dialogue describes character preferences, invitations and reflections, rather
than claiming the player performed a particular action. Friendship points confirm
helping, but do not prove a paw print was spotted, a letter sent or a cart built.
Showing a line does not itself perform any of those actions.

The development preview does not approve or activate a release pack. Release
activation requires content review and coordinated manifest/version and consumer
updates. The sidecar is not yet part of the signed story-pack manifest. Existing
hashed pack bytes and the sidecar activation value remain unchanged.

Validate from the repository root:

```sh
python content/story/validate.py
python -m unittest discover -s content/story -p 'test_*.py'
```

These checks verify roster coverage, distinct profiles, threshold/story references,
unique moment IDs and the absence of reward fields in moments. They do not certify
reading level, educator approval or child comprehension.
