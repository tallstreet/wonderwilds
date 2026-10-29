# Continuity between authored episodes

Status: implemented and validated; actual verification belongs in
`../IMPLEMENTATION_STATUS.md`.

The quest document remains a bounded choice/activity/handoff graph. A separate
trusted catalog connects episodes using continuation tags and permitted source
quests. This allows an ongoing adventure to reuse a small authored bank while
the document format is refined for a later model.

Continuity memory records actual choices and completed outcomes, including their
quest, document version and episode. Threads retain the source node/choice,
original summary and an open/resolved status. Working memory is bounded to recent
records; durable command/event history remains separate. This does not claim
unlimited unique stories or unlimited model context.

A follow-up becomes available only when its trusted source policy matches a real
thread and ordinary ownership, region and character-level requirements hold.
Its acknowledgement and source are pinned when the episode starts. Editing the
authored bank later must not rewrite the meaning of an earlier choice.

Completing the follow-up resolves the exact pinned source thread. Abandoning an
activity or episode leaves that source open. A newer thread with the same tag
must not be resolved merely because an older follow-up completes. Subsequent
handoffs can connect the Pip follow-ups to one another; individual graphs remain
acyclic while the sequence of episodes can continue.

Player-selected interests come from the catalog's bounded vocabulary and affect
offer ordering. They never change ownership, academic correctness, supported
completion, reward policy or essential access. Eligible episodes remain a free
choice. Thread matches, interests, freshness and recent repetition provide a
deterministic ranking rather than a model call.

The future director context is assembled from authoritative world state,
bounded narrative memory, selected interests and server-selected learning needs.
Conversation capture and model inference remain deferred. A structural context
or graph check does not establish semantic prose quality or human content approval.

Continuity-bearing compact saves use envelope version three so older writers
cannot silently drop memory or preferences. Version-one/two saves remain
readable by the current runtime. Earlier version-two handoffs are preserved;
unknown historical continuity must not be invented to unlock a follow-up.

Acceptance: choose the woodland or night-sky hook, receive the matching authored
follow-up, see the earlier choice acknowledged, and reload with the same state.
Also cover a second follow-up, support, abandonment, exact retries, one-time
rewards, interest ranking, bounded memory and older-save compatibility.
