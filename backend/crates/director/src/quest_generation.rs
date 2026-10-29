//! Future quest-proposal boundary. No provider, conversation capture or commit authority.
//!
//! Passing these checks establishes a bounded graph using permitted references.
//! It does not approve generated prose for children or prove narrative coherence.
//! The current playable catalog remains authored, pending human review.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use wonderwilds_contracts::{QuestGenerationContext, QuestGenerationProposal, validate};
use wonderwilds_domain::personal_quests::{self, ActivitySlot, Catalog, QuestDocument, QuestNode};

pub const MAX_CONTEXT_BYTES: usize = 16 * 1024;
pub const MAX_PROPOSAL_BYTES: usize = 32 * 1024;
pub const RULES_VERSION: &str = "personal-quests-1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    Oversized,
    Malformed,
    Schema,
    UnsupportedVersion,
    ContextBinding,
    Stale,
    NotAllowed,
    InvalidGraph,
    NoPlayableCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    AuthoredFallback,
    GeneratedProposal,
}

/// A structurally accepted proposal, never an approved or committed game action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub proposal: QuestGenerationProposal,
    pub source: Source,
    pub rejected_proposal: Option<Rejection>,
    pub content_review_required: bool,
}

fn value<T: serde::Serialize>(input: &T) -> Result<Value, Rejection> {
    serde_json::to_value(input).map_err(|_| Rejection::Malformed)
}

fn strings(input: &Value) -> Result<BTreeSet<String>, Rejection> {
    let values = input.as_array().ok_or(Rejection::Schema)?;
    values
        .iter()
        .map(|v| v.as_str().map(str::to_owned).ok_or(Rejection::Schema))
        .collect()
}

/// SHA-256 of compact UTF-8 JSON with recursively sorted object keys, excluding
/// only contextDigest. Array order remains meaningful. serde_json's default Map
/// representation provides the canonical key ordering used by this workspace.
fn digest(input: &Value) -> Result<String, Rejection> {
    let mut input = input.clone();
    input
        .as_object_mut()
        .ok_or(Rejection::Schema)?
        .remove("contextDigest");
    Ok(format!(
        "{:x}",
        Sha256::digest(input.to_string().as_bytes())
    ))
}

/// Bind a server-built context after redaction/permission decisions. This helper
/// is not authorization for browser-supplied world facts or conversation text.
pub fn bind_context(mut input: Value) -> Result<QuestGenerationContext, Rejection> {
    if input.to_string().len() > MAX_CONTEXT_BYTES {
        return Err(Rejection::Oversized);
    }
    let binding = digest(&input)?;
    input["contextDigest"] = json!(binding);
    validate("QuestGenerationContext", &input).map_err(|_| Rejection::Schema)?;
    let context = serde_json::from_value(input).map_err(|_| Rejection::Malformed)?;
    validate_context(&context)?;
    Ok(context)
}

pub fn validate_context(context: &QuestGenerationContext) -> Result<(), Rejection> {
    let input = value(context)?;
    if input.to_string().len() > MAX_CONTEXT_BYTES {
        return Err(Rejection::Oversized);
    }
    validate("QuestGenerationContext", &input).map_err(|_| Rejection::Schema)?;
    if input["contextDigest"].as_str() != Some(&digest(&input)?) {
        return Err(Rejection::ContextBinding);
    }
    let catalog = Catalog::bundled().map_err(|_| Rejection::InvalidGraph)?;
    if input["contentVersion"] != catalog.version() || input["rulesVersion"] != RULES_VERSION {
        return Err(Rejection::UnsupportedVersion);
    }
    let allowed_companions = strings(&input["allowed"]["companionIds"])?;
    let owned = strings(&input["world"]["ownedCompanionIds"])?;
    let allowed_regions = strings(&input["allowed"]["regionIds"])?;
    let companion = input["progression"]["companionId"]
        .as_str()
        .ok_or(Rejection::Schema)?;
    let region = input["world"]["regionId"]
        .as_str()
        .ok_or(Rejection::Schema)?;
    if !allowed_companions.contains(companion)
        || !owned.contains(companion)
        || !allowed_regions.contains(region)
    {
        return Err(Rejection::NotAllowed);
    }
    let interest_tags = strings(&input["allowed"]["interestTags"])?;
    let mut seen_interests = BTreeSet::new();
    let shared = input["conversationMemory"]["permission"] == "shared_summary";
    let summary = input["conversationMemory"]["summary"]
        .as_str()
        .ok_or(Rejection::Schema)?;
    if !shared && !summary.is_empty() {
        return Err(Rejection::NotAllowed);
    }
    for interest in input["interests"].as_array().ok_or(Rejection::Schema)? {
        let tag = interest["tag"].as_str().ok_or(Rejection::Schema)?;
        if !interest_tags.contains(tag)
            || !seen_interests.insert(tag)
            || interest["source"] == "curated_conversation"
                && (!shared || summary.trim().is_empty())
        {
            return Err(Rejection::NotAllowed);
        }
    }
    let slots: Vec<ActivitySlot> =
        serde_json::from_value(input["allowed"]["activitySlots"].clone())
            .map_err(|_| Rejection::Schema)?;
    let mut ids = BTreeSet::new();
    for slot in slots {
        if !ids.insert(slot.id.clone())
            || !catalog.activity_slots().iter().any(|known| known == &slot)
        {
            return Err(Rejection::NotAllowed);
        }
    }
    Ok(())
}

fn validate_document_context(
    context: &QuestGenerationContext,
    document: &QuestDocument,
) -> Result<(), Rejection> {
    let input = value(context)?;
    if document.companion_id != input["progression"]["companionId"]
        || document.region_id != input["world"]["regionId"]
    {
        return Err(Rejection::NotAllowed);
    }
    let catalog = Catalog::bundled().map_err(|_| Rejection::InvalidGraph)?;
    // The current authored personal-quest policy unlocks at level four. A future
    // generated document cannot lower trusted catalog policy or grant ownership.
    let minimum = catalog
        .entry(&document.quest_id)
        .map_or(4, |entry| entry.minimum_character_level);
    if input["progression"]["companionLevel"]
        .as_u64()
        .is_none_or(|level| level < minimum as u64)
    {
        return Err(Rejection::NotAllowed);
    }
    let interest_tags = strings(&input["allowed"]["interestTags"])?;
    if document
        .interest_tags
        .iter()
        .any(|tag| !interest_tags.contains(tag))
    {
        return Err(Rejection::NotAllowed);
    }
    let continuation_tags = strings(&input["allowed"]["continuationTags"])?;
    let skills = strings(&input["learning"]["skillIds"])?;
    let mut slots: Vec<ActivitySlot> =
        serde_json::from_value(input["allowed"]["activitySlots"].clone())
            .map_err(|_| Rejection::Schema)?;
    // A broadly allowed renderer is not permission to change the learning goal.
    slots.retain(|slot| skills.contains(&slot.skill_id));
    for node in &document.nodes {
        if let QuestNode::Handoff {
            continuation_tags: tags,
            ..
        } = node
            && tags.iter().any(|tag| !continuation_tags.contains(tag))
        {
            return Err(Rejection::NotAllowed);
        }
    }
    personal_quests::validate_document(document, &slots).map_err(|_| Rejection::InvalidGraph)
}

/// Validate an untrusted future model response against the exact server context.
/// The caller must recheck revision under its transaction lock before any commit.
pub fn validate_proposal(
    context: &QuestGenerationContext,
    bytes: &[u8],
    current_revision: u64,
) -> Result<QuestGenerationProposal, Rejection> {
    validate_context(context)?;
    if context.revision != current_revision {
        return Err(Rejection::Stale);
    }
    if bytes.len() > MAX_PROPOSAL_BYTES {
        return Err(Rejection::Oversized);
    }
    let input: Value = serde_json::from_slice(bytes).map_err(|_| Rejection::Malformed)?;
    validate("QuestGenerationProposal", &input).map_err(|_| Rejection::Schema)?;
    if input["requestId"] != context.request_id
        || input["expectedRevision"] != context.revision
        || input["contextDigest"] != context.context_digest
    {
        return Err(Rejection::ContextBinding);
    }
    let document: QuestDocument =
        serde_json::from_value(input["document"].clone()).map_err(|_| Rejection::Schema)?;
    validate_document_context(context, &document)?;
    serde_json::from_value(input).map_err(|_| Rejection::Malformed)
}

/// Deterministic authored development fallback in the same future proposal shape.
/// The small catalog is replayable; it does not promise unlimited unique quests.
pub fn authored_fallback(
    context: &QuestGenerationContext,
    current_revision: u64,
) -> Result<QuestGenerationProposal, Rejection> {
    validate_context(context)?;
    if context.revision != current_revision {
        return Err(Rejection::Stale);
    }
    let catalog = Catalog::bundled().map_err(|_| Rejection::InvalidGraph)?;
    let document = catalog
        .documents()
        .filter(|document| validate_document_context(context, document).is_ok())
        .min_by_key(|document| &document.quest_id)
        .ok_or(Rejection::NoPlayableCandidate)?;
    let input = json!({
        "schemaVersion":"1.0",
        "requestId":context.request_id,
        "expectedRevision":context.revision,
        "contextDigest":context.context_digest,
        "document":document,
    });
    validate_proposal(
        context,
        &serde_json::to_vec(&input).map_err(|_| Rejection::Malformed)?,
        current_revision,
    )
}

/// Experimental, pure selection seam. No returned proposal is release-approved.
pub fn propose_with_fallback(
    context: &QuestGenerationContext,
    current_revision: u64,
    generated_bytes: Option<&[u8]>,
) -> Result<Selection, Rejection> {
    let rejected_proposal = if let Some(bytes) = generated_bytes {
        match validate_proposal(context, bytes, current_revision) {
            Ok(proposal) => {
                return Ok(Selection {
                    proposal,
                    source: Source::GeneratedProposal,
                    rejected_proposal: None,
                    content_review_required: true,
                });
            }
            Err(error) => Some(error),
        }
    } else {
        None
    };
    Ok(Selection {
        proposal: authored_fallback(context, current_revision)?,
        source: Source::AuthoredFallback,
        rejected_proposal,
        content_review_required: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> QuestGenerationContext {
        bind_context(
            serde_json::from_str(include_str!(
                "../../../../content/quests/generation-context.example.json"
            ))
            .unwrap(),
        )
        .unwrap()
    }
    fn proposal(context: &QuestGenerationContext) -> Value {
        value(&authored_fallback(context, context.revision).unwrap()).unwrap()
    }
    fn validate_changed(
        context: &QuestGenerationContext,
        input: &Value,
    ) -> Result<QuestGenerationProposal, Rejection> {
        validate_proposal(
            context,
            &serde_json::to_vec(input).unwrap(),
            context.revision,
        )
    }

    #[test]
    fn checked_in_examples_share_the_reference_context_digest() {
        let context: QuestGenerationContext = serde_json::from_str(include_str!(
            "../../../../content/quests/generation-context.example.json"
        ))
        .unwrap();
        validate_context(&context).unwrap();
        let proposal = validate_proposal(
            &context,
            include_bytes!("../../../../content/quests/generation-proposal.example.json"),
            context.revision,
        )
        .unwrap();
        assert_eq!(proposal.context_digest, context.context_digest);
    }

    #[test]
    fn authored_fallback_uses_same_bound_contract_and_stays_deterministic() {
        let context = context();
        let first = propose_with_fallback(&context, context.revision, None).unwrap();
        assert_eq!(
            first,
            propose_with_fallback(&context, context.revision, None).unwrap()
        );
        assert_eq!(first.source, Source::AuthoredFallback);
        assert!(first.content_review_required);
        let bytes = serde_json::to_vec(&first.proposal).unwrap();
        let candidate = propose_with_fallback(&context, context.revision, Some(&bytes)).unwrap();
        assert_eq!(candidate.source, Source::GeneratedProposal);
        assert!(candidate.content_review_required);
        assert_eq!(candidate.proposal, first.proposal);
    }

    #[test]
    fn stale_or_changed_context_cannot_reuse_a_proposal() {
        let context = context();
        let input = proposal(&context);
        assert_eq!(
            validate_proposal(
                &context,
                &serde_json::to_vec(&input).unwrap(),
                context.revision + 1
            ),
            Err(Rejection::Stale)
        );
        for key in ["requestId", "contextDigest"] {
            let mut changed = input.clone();
            changed[key] = if key == "contextDigest" {
                json!("f".repeat(64))
            } else {
                json!("another-request")
            };
            assert_eq!(
                validate_changed(&context, &changed),
                Err(Rejection::ContextBinding)
            );
        }
        let mut changed = value(&context).unwrap();
        changed["progression"]["explorerLevel"] = json!(9);
        let unbound: QuestGenerationContext = serde_json::from_value(changed.clone()).unwrap();
        assert_eq!(validate_context(&unbound), Err(Rejection::ContextBinding));
        let rebound = bind_context(changed).unwrap();
        assert_ne!(rebound.context_digest, context.context_digest);
        assert_eq!(
            validate_changed(&rebound, &input),
            Err(Rejection::ContextBinding)
        );
        let mut changed = input;
        changed["expectedRevision"] = json!(context.revision + 1);
        assert_eq!(
            validate_changed(&context, &changed),
            Err(Rejection::ContextBinding)
        );
    }

    #[test]
    fn malformed_oversized_and_authority_fields_reject_and_fallback() {
        let context = context();
        assert_eq!(
            validate_proposal(
                &context,
                &vec![b' '; MAX_PROPOSAL_BYTES + 1],
                context.revision
            ),
            Err(Rejection::Oversized)
        );
        assert_eq!(
            validate_proposal(&context, b"not-json", context.revision),
            Err(Rejection::Malformed)
        );
        let mut changed = proposal(&context);
        changed["document"]["rewards"] = json!({"xp":999});
        assert_eq!(validate_changed(&context, &changed), Err(Rejection::Schema));
        let selection = propose_with_fallback(
            &context,
            context.revision,
            Some(&serde_json::to_vec(&changed).unwrap()),
        )
        .unwrap();
        assert_eq!(selection.source, Source::AuthoredFallback);
        assert_eq!(selection.rejected_proposal, Some(Rejection::Schema));
    }

    #[test]
    fn actor_region_activity_learning_and_handoff_allowlists_are_enforced() {
        let context = context();
        for (field, replacement) in [("companionId", "tansy"), ("regionId", "cove")] {
            let mut changed = proposal(&context);
            changed["document"][field] = json!(replacement);
            assert_eq!(
                validate_changed(&context, &changed),
                Err(Rejection::NotAllowed)
            );
        }
        let mut changed = proposal(&context);
        changed["document"]["nodes"][1]["activitySlotId"] = json!("unknown-slot");
        assert_eq!(
            validate_changed(&context, &changed),
            Err(Rejection::InvalidGraph)
        );
        let mut changed = proposal(&context);
        let handoff = changed["document"]["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["type"] == "handoff")
            .unwrap();
        handoff["continuationTags"] = json!(["unapproved-thread"]);
        assert_eq!(
            validate_changed(&context, &changed),
            Err(Rejection::NotAllowed)
        );
        let mut input = value(&context).unwrap();
        input["allowed"]["activitySlots"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"nib-share","skillId":"M08","templateId":"equal_sharing"}));
        let restricted = bind_context(input).unwrap();
        let mut changed = proposal(&restricted);
        changed["document"]["nodes"][1]["activitySlotId"] = json!("nib-share");
        assert_eq!(
            validate_changed(&restricted, &changed),
            Err(Rejection::InvalidGraph)
        );
    }

    #[test]
    fn cycles_dead_ends_and_unreachable_nodes_cannot_enter_the_game() {
        let context = context();
        let mut changed = proposal(&context);
        changed["document"]["nodes"][1]["nextNodeId"] = json!("start");
        assert_eq!(
            validate_changed(&context, &changed),
            Err(Rejection::InvalidGraph)
        );
        changed["document"]["nodes"][1]["nextNodeId"] = json!("missing");
        assert_eq!(
            validate_changed(&context, &changed),
            Err(Rejection::InvalidGraph)
        );
        let mut changed = proposal(&context);
        let mut orphan = changed["document"]["nodes"][1].clone();
        orphan["id"] = json!("orphan");
        changed["document"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(orphan);
        assert_eq!(
            validate_changed(&context, &changed),
            Err(Rejection::InvalidGraph)
        );
    }

    #[test]
    fn conversation_memory_requires_permission_and_stays_separate_from_canonical_facts() {
        let context = context();
        let mut input = value(&context).unwrap();
        input["conversationMemory"]["summary"] = json!("Likes lantern stories.");
        assert_eq!(bind_context(input.clone()), Err(Rejection::NotAllowed));
        input["conversationMemory"]["summary"] = json!("");
        input["interests"][0]["source"] = json!("curated_conversation");
        assert_eq!(bind_context(input.clone()), Err(Rejection::NotAllowed));
        input["conversationMemory"] =
            json!({"permission":"shared_summary","summary":"Likes lantern stories."});
        let bound = bind_context(input).unwrap();
        assert_eq!(bound.world.canonical_facts, context.world.canonical_facts);
        assert!(authored_fallback(&bound, bound.revision).is_ok());
    }

    #[test]
    fn level_and_policy_binding_do_not_unlock_a_locked_personal_quest() {
        let context = context();
        let mut input = value(&context).unwrap();
        input["progression"]["companionLevel"] = json!(3);
        let locked = bind_context(input).unwrap();
        assert_eq!(
            authored_fallback(&locked, locked.revision),
            Err(Rejection::NoPlayableCandidate)
        );
        let mut input = value(&context).unwrap();
        input["rulesVersion"] = json!("future-unreviewed-rules");
        assert_eq!(bind_context(input), Err(Rejection::UnsupportedVersion));
    }
}
