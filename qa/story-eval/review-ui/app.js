"use strict";

const $ = (id) => document.getElementById(id);
const list = (value) => (Array.isArray(value) ? value : []);
const name = (value) =>
  String(value || "")
    .replaceAll("_", " ")
    .replace(/\b\w/g, (s) => s.toUpperCase());
const key = (item) => `${item.scenarioId}/${item.candidateId}`;
let data,
  selected = null,
  busy = false,
  view = 0,
  originalDraft = "",
  gradeLoading = false;
let lastQueue = "train",
  lastFilter = "all";

function element(tag, text, className) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  if (className) node.className = className;
  return node;
}

function headingList(parent, title, values, ordered = false) {
  if (!values.length) return;
  parent.append(element("h3", title));
  const items = element(ordered ? "ol" : "ul");
  for (const value of values) items.append(element("li", value));
  parent.append(items);
}

function draft() {
  return JSON.stringify({
    reviewer: $("reviewer").value,
    notes: $("notes").value,
    suggestedText: $("suggested").value,
  });
}

function dirty() {
  return selected && draft() !== originalDraft;
}
function allowNavigation() {
  return !busy && (!dirty() || window.confirm("Discard these unsaved review changes?"));
}

function queueItems() {
  if (!data) return [];
  return data.items.filter((item) => {
    if ($("queue").value === "all") return true;
    return item.shortlisted && item.split === $("queue").value && !item.excludedForEvalOverlap;
  });
}

function visibleItems() {
  return queueItems().filter(
    (item) =>
      $("review-filter").value === "all" ||
      ($("review-filter").value === "pending" ? !item.review : Boolean(item.review)),
  );
}

function setBusy(value) {
  busy = value;
  for (const id of ["reviewer", "queue", "review-filter", "notes", "suggested"])
    $(id).disabled = value;
  updateNavigation();
  updateDecisionButtons();
}

function updateDecisionButtons() {
  $("approve").disabled =
    busy || !selected || !selected.hardPassed || Boolean($("suggested").value.trim());
  $("revise").disabled = busy || !selected;
  $("reject").disabled = busy || !selected;
  $("revision-help").textContent = $("suggested").value.trim()
    ? "Replacement text is a revision proposal. Clear it before approving the unchanged original. A revised scene needs its own validation and review."
    : "Suggested text is a revision proposal. It must become a new candidate and pass validation and review before training.";
}

function updateNavigation() {
  const queue = queueItems(),
    visible = visibleItems(),
    index = selected ? visible.findIndex((item) => key(item) === key(selected)) : -1;
  const reviewed = queue.filter((item) => item.review).length;
  const totalReviewed = data ? data.items.filter((item) => item.review).length : 0;
  $("progress").textContent = `${reviewed} of ${queue.length} decided in this queue`;
  $("progress-bar").max = Math.max(queue.length, 1);
  $("progress-bar").value = reviewed;
  $("dataset").textContent = data
    ? `${totalReviewed} / ${data.items.length} total · ${data.datasetId}`
    : "";
  $("position").textContent = selected
    ? index < 0
      ? "Saved scene · outside the current filter"
      : `Scene ${index + 1} of ${visible.length} in this view`
    : `${visible.length} scenes in this view`;
  $("previous").disabled = busy || !visible.length || index === 0;
  $("next").disabled = busy || !visible.length || index === visible.length - 1;
  $("next-pending").disabled =
    busy || !queue.some((item) => !item.review && (!selected || key(item) !== key(selected)));
}

function renderState(item) {
  const input = item.input,
    context = input.context || {},
    here = input.here || {},
    mission = input.mission;
  const summary = $("state-summary");
  summary.replaceChildren();
  summary.append(element("p", name(context.locationId)), element("p", here.description || ""));
  headingList(
    summary,
    "Here with you",
    list(here.characters).map((c) => `${name(c.id)} — ${c.description}`),
  );
  if (!list(here.characters).length) summary.append(element("p", "No characters are present."));
  headingList(
    summary,
    "Carrying",
    list(context.inventory).map((i) => `${i.quantity} ${String(i.itemId).replaceAll("_", " ")}`),
  );
  if (!list(context.inventory).length) summary.append(element("p", "Your bag is empty."));
  summary.append(element("h3", "Current mission"));
  if (!mission) summary.append(element("p", "No mission has been accepted yet."));
  else {
    summary.append(
      element("p", mission.goal),
      element("p", `Status: ${mission.status === "abandoned" ? "Paused" : name(mission.status)}`),
    );
    if (mission.current) summary.append(element("p", `Current milestone: ${mission.current}`));
    headingList(summary, "Already completed", list(mission.completed));
    headingList(summary, "Still to do", list(mission.remaining), true);
    if (mission.narrative?.payoff)
      summary.append(element("p", `Outcome: ${mission.narrative.payoff}`));
  }
  headingList(
    summary,
    "Actions available now",
    list(input.availableActions).map((a) => a.description),
  );
  const history = $("history");
  history.replaceChildren();
  headingList(history, "World facts", list(context.memory?.facts));
  headingList(history, "Recent events", list(context.memory?.recentEvents), true);
  headingList(history, "Previous missions", list(context.memory?.recentMissions));
  headingList(
    history,
    "Nearby objects",
    list(here.objects).map((o) => `${name(o.id)}: ${o.state}`),
  );
  $("raw-input").textContent = JSON.stringify(input, null, 2);
}

function render(item, focus = false) {
  selected = item || null;
  view++;
  $("empty").hidden = Boolean(selected);
  $("candidate-view").hidden = !selected;
  $("save-error").hidden = true;
  $("feedback-error").hidden = true;
  $("retry-feedback").hidden = true;
  $("model-feedback").replaceChildren();
  $("model-feedback").hidden = true;
  $("feedback-locked").hidden = false;
  $("feedback-locked").textContent =
    "Hidden until you save your own decision. There is no model score to guide your first impression.";
  gradeLoading = false;
  if (selected) {
    renderState(selected);
    $("candidate-id").textContent = `${selected.scenarioId} · ${selected.candidateId}`;
    $("split-notice").textContent = selected.excludedForEvalOverlap
      ? "Excluded from training: this input overlaps the held-out evaluation set."
      : selected.split === "eval"
        ? "Held-out evaluation example. Your review will not put it into training."
        : "Training candidate. Approval applies only to the unchanged scene below.";
    const entries = $("scene-entries");
    entries.replaceChildren();
    for (const entry of list(selected.output.presentation)) {
      const block = element("div", undefined, "scene-entry");
      block.append(
        element("div", entry.type === "dialogue" ? name(entry.speakerId) : "Narration", "speaker"),
      );
      block.append(element("p", entry.text));
      entries.append(block);
    }
    const choices = $("scene-choices");
    choices.replaceChildren();
    for (const choice of list(selected.output.choices)) choices.append(element("li", choice.label));
    $("hard-warning").hidden = selected.hardPassed;
    $("notes").value = selected.review?.notes || "";
    $("suggested").value = selected.review?.suggestedText || "";
    $("decision-status").textContent = selected.review
      ? savedStatus(selected.review)
      : "Not decided yet. Your decision will be saved on this computer.";
    try {
      localStorage.setItem(
        `wonderwilds-review:${data.datasetId}:${data.datasetSha256}`,
        JSON.stringify({
          key: key(selected),
          queue: $("queue").value,
          filter: $("review-filter").value,
        }),
      );
    } catch {
      /* Decisions live on the server. */
    }
    if (selected.review) loadFeedback(selected);
    if (focus) $("scene-heading").focus();
  }
  originalDraft = draft();
  updateNavigation();
  updateDecisionButtons();
}

function savedStatus(review) {
  const decision =
    { approve: "Approved unchanged", revise: "Needs revision", reject: "Rejected" }[
      review.decision
    ] || review.decision;
  return `Saved · ${decision} · ${review.reviewer}${review.updatedAt ? ` · ${new Date(review.updatedAt).toLocaleString()}` : ""}`;
}

async function api(path, options = {}) {
  const response = await fetch(path, {
    ...options,
    cache: "no-store",
    headers: {
      ...(options.body ? { "Content-Type": "application/json" } : {}),
      ...(data?.token ? { "X-Review-Token": data.token } : {}),
      ...options.headers,
    },
  });
  const result = await response.json();
  if (!response.ok) throw new Error(result.error || `Request failed (${response.status}).`);
  return result;
}

async function loadFeedback(item) {
  if (!item.review || gradeLoading) return;
  const revision = view;
  gradeLoading = true;
  $("feedback-locked").textContent = "Your decision is saved. Loading model feedback…";
  $("feedback-error").hidden = true;
  $("retry-feedback").hidden = true;
  try {
    const params = new URLSearchParams({
      scenarioId: item.scenarioId,
      candidateId: item.candidateId,
    });
    const result = await api(`/api/grade?${params}`);
    if (view !== revision || selected !== item || !item.review) return;
    const target = $("model-feedback");
    target.replaceChildren();
    if (result.recommendation) {
      target.append(
        element("h3", `Model recommendation: ${name(result.recommendation.recommendation)}`),
      );
      target.append(element("p", result.recommendation.reason));
    }
    const grid = element("div", undefined, "grade-list");
    for (const [criterion, finding] of Object.entries(result.judgment?.criteria || {})) {
      const block = element("div", undefined, "grade");
      const title = element("div", undefined, "grade-title");
      title.append(
        element("span", name(criterion)),
        element(
          "span",
          finding.verdict,
          `grade-verdict ${["pass", "fail", "uncertain"].includes(finding.verdict) ? finding.verdict : ""}`,
        ),
      );
      block.append(title, element("p", finding.evidence));
      grid.append(block);
    }
    target.append(
      grid,
      element(
        "p",
        "The model can be wrong. Keep your own judgment; if you change a saved decision, record why in your notes.",
        "small",
      ),
    );
    $("feedback-locked").hidden = true;
    target.hidden = false;
  } catch (error) {
    if (view !== revision) return;
    $("feedback-locked").textContent =
      "Your decision is saved. Model feedback is temporarily unavailable.";
    $("feedback-error").textContent = error.message;
    $("feedback-error").hidden = false;
    $("retry-feedback").hidden = false;
  } finally {
    if (view === revision) gradeLoading = false;
  }
}

async function decide(decision) {
  if (busy || !selected) return;
  const reviewer = $("reviewer").value.trim(),
    notes = $("notes").value.trim(),
    suggestedText = $("suggested").value.trim();
  $("save-error").hidden = true;
  if (!reviewer) {
    $("reviewer").focus();
    return showSaveError("Enter your reviewer name before saving a decision.");
  }
  if (decision !== "approve" && !notes) {
    $("notes").focus();
    return showSaveError("Add a reason for revision or rejection.");
  }
  if (decision === "approve" && suggestedText)
    return showSaveError(
      "Clear the suggested replacement before approving the unchanged original.",
    );
  if (decision !== "revise" && suggestedText)
    return showSaveError(
      "Suggested replacement text belongs to a revision. Clear it or choose Needs revision.",
    );
  const item = selected;
  setBusy(true);
  $("decision-status").textContent = "Saving your decision…";
  try {
    const result = await api("/api/decision", {
      method: "POST",
      body: JSON.stringify({
        scenarioId: item.scenarioId,
        candidateId: item.candidateId,
        candidateSha256: item.candidateSha256,
        decision,
        reviewer,
        notes,
        suggestedText,
      }),
    });
    item.review = result.review;
    data.reviewer = reviewer;
    $("notes").value = result.review.notes || "";
    $("suggested").value = result.review.suggestedText || "";
    originalDraft = draft();
    $("decision-status").textContent = savedStatus(result.review);
    loadFeedback(item);
  } catch (error) {
    $("decision-status").textContent =
      "Save not confirmed. Your changes are still here; retry the same decision.";
    showSaveError(error.message);
  } finally {
    setBusy(false);
  }
}

function showSaveError(message) {
  $("save-error").textContent = message;
  $("save-error").hidden = false;
}

function move(offset) {
  if (!allowNavigation()) return;
  const items = visibleItems(),
    index = items.findIndex((item) => selected && key(item) === key(selected));
  render(items[index < 0 ? (offset > 0 ? 0 : items.length - 1) : index + offset], true);
}

function filterChanged() {
  if (!allowNavigation()) {
    $("queue").value = lastQueue;
    $("review-filter").value = lastFilter;
    return;
  }
  lastQueue = $("queue").value;
  lastFilter = $("review-filter").value;
  const items = visibleItems();
  render(items.find((item) => !item.review) || items[0], true);
}

async function start() {
  $("loading").hidden = false;
  $("connection-error").hidden = true;
  $("reconnect").hidden = true;
  try {
    data = await api("/api/review");
    if (!Array.isArray(data.items))
      throw new Error("The review server returned an invalid workspace.");
    $("reviewer").value = data.reviewer || "";
    $("brief").textContent =
      `Editorial brief: ${data.editorialBrief || "Review against the intended audience and story style."}`;
    $("workspace").hidden = false;
    let remembered;
    try {
      const saved = JSON.parse(
        localStorage.getItem(`wonderwilds-review:${data.datasetId}:${data.datasetSha256}`) ||
          "null",
      );
      if (
        saved &&
        ["train", "eval", "all"].includes(saved.queue) &&
        ["all", "pending", "decided"].includes(saved.filter)
      ) {
        $("queue").value = saved.queue;
        $("review-filter").value = saved.filter;
        lastQueue = saved.queue;
        lastFilter = saved.filter;
        remembered = saved.key;
      }
    } catch {
      /* Optional navigation convenience. */
    }
    const items = visibleItems();
    render(
      items.find((item) => key(item) === remembered) ||
        items.find((item) => !item.review) ||
        items[0],
    );
  } catch (error) {
    $("connection-error").textContent = `Could not load the review workspace: ${error.message}`;
    $("connection-error").hidden = false;
    $("reconnect").hidden = false;
  } finally {
    $("loading").hidden = true;
  }
}

$("previous").addEventListener("click", () => move(-1));
$("next").addEventListener("click", () => move(1));
$("next-pending").addEventListener("click", () => {
  if (!allowNavigation()) return;
  const items = queueItems(),
    index = items.findIndex((item) => selected && key(item) === key(selected));
  const ordered = [...items.slice(index + 1), ...items.slice(0, index + 1)];
  render(
    ordered.find((item) => !item.review),
    true,
  );
});
for (const id of ["queue", "review-filter"]) $(id).addEventListener("change", filterChanged);
for (const id of ["reviewer", "notes", "suggested"])
  $(id).addEventListener("input", () => {
    updateDecisionButtons();
    if (selected)
      $("decision-status").textContent = dirty()
        ? "Unsaved changes."
        : selected.review
          ? savedStatus(selected.review)
          : "Not decided yet.";
  });
$("approve").addEventListener("click", () => decide("approve"));
$("revise").addEventListener("click", () => decide("revise"));
$("reject").addEventListener("click", () => decide("reject"));
$("retry-feedback").addEventListener("click", () => selected && loadFeedback(selected));
$("reconnect").addEventListener("click", start);
window.addEventListener("beforeunload", (event) => {
  if (dirty() || busy) {
    event.preventDefault();
    event.returnValue = "";
  }
});
start();
