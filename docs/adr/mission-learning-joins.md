# Inline learning joins for generated missions

Status: development implementation, 31 October 2026. Content and release reviews
remain pending.

Mission generation supplies narrative context rather than academic tasks. Each
GeneratedNeighbourhood step may include a bounded learningBridge invitation to
organise discovery markers for that investigation. The field is optional so old
saved generated stories, checkpoints and receipts retain their meaning. It is
presentation text only. The generator is told not to invent task quantities,
answers, difficulty, successful assessment or world inventory. Missing bridges
use authored copy selected from the legal action's mechanic.

The story world routes inspect, collect, exchange and assemble choices through
begin_step. The server resolves the action handle and exact story revision under
the shared profile lock. A frozen binding contains mission identity (including
neighbourhood generation), location, action, revision and joining text. Supported
prerequisite-ready skills reuse existing task parameters, representation,
assessment key and canonical evidence fingerprint with discovery-marker labels.
The activity is persisted before presentation. Existing standalone begin commands
and saved activities remain readable for compatibility; the world no longer
advertises a separate practice launch.

A correct answer executes the bound story action inside the learning transaction.
The transaction saves the story checkpoint and its ordinary replayable domain
receipt, learning checkpoint, evidence and original-response learning receipt.
No new database table or applied-migration change is required. Learning-command
receipt lookup precedes revision checks, so uncertain retries cannot duplicate
story effects. Any failure rolls both streams back. Story changes after issuance
make completion conflict; the child may put the stale activity aside and reopen
the current action. Bindings are not silently moved onto a new generated story.

An explicit finish_together command reveals a server-authored solution and runs
the same bound action. Its event is worked_example / not_assessed with support
recorded, never an independent answer success. Abandon closes the task without
story advancement. Closing the dialog preserves it for resume. The browser
refreshes story state on completion and retry; failed story reads preserve the
confirmed learning receipt and require recovery before world actions.

This is a presentation integration, not an academic authorization rule on the
original story API. The inspector and direct story actions stay compatible, and
unsupported learning frontiers use the original action. No readiness or mastery
is required to enjoy the story. The initial reusable joining prop is discovery
markers, not arbitrary AI-created teaching mechanics. Semantic fit, repetitive
placement and pacing still require human playtesting. Only the existing five
maths skills have activity coverage; prose generation does not expand it.

Forward compatibility preserves old data. Older binaries may not read newly
saved optional step fields or inline activity bindings; rollback must retain a
compatible reader rather than delete evidence or reset saves.
