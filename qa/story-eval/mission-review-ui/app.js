const $ = (id) => document.getElementById(id);
const labels = {story: 'Story coherence and payoff', clarity: 'Clear next steps', variety: 'Variety / originality', learning_fit: 'Learning invitation fits the story', age_fit: 'Age suitability (6–8)'};
let workspace, selected, dirty = false, saving = false;
for (const [key, label] of Object.entries(labels)) {
  const wrapper = document.createElement('label'); wrapper.textContent = label;
  const select = document.createElement('select'); select.id = key; select.required = true; select.setAttribute('aria-label', label);
  for (const [value, text] of [['', 'Choose score…'], ...[1,2,3,4,5].map(n => [String(n), String(n)])]) {
    const option = document.createElement('option'); option.value = value; option.textContent = text; select.append(option);
  }
  wrapper.append(select); $('scores').append(wrapper);
}
function myReview(item) { return workspace.reviews.find(r => r.id === item.capture.id && r.fingerprint === item.fingerprint && r.reviewer === $('reviewer').value.trim()); }
function title(item) { return item.capture.output?.title || item.capture.output?.goal || `${item.capture.kind} · no valid title`; }
function mayLeave() { return !saving && (!dirty || confirm('Discard unsaved feedback?')); }
function queue() {
  $('queue').replaceChildren();
  const reviewed = workspace.items.filter(myReview).length;
  $('counts').textContent = `${workspace.items.length} captures · ${reviewed} graded by you`;
  for (const item of workspace.items.toReversed()) {
    if ($('filter').value === 'pending' && myReview(item)) continue;
    if ($('filter').value === 'failed' && item.capture.providerValidation !== 'failed') continue;
    const button = document.createElement('button'); button.textContent = `${myReview(item) ? '✓ ' : ''}${title(item)}`;
    button.setAttribute('aria-current', String(item.capture.id === selected?.capture.id));
    button.onclick = () => { if (mayLeave()) show(item); }; $('queue').append(button);
  }
}
function show(item) {
  selected = item; dirty = false; $('detail').hidden = false; $('empty').hidden = true;
  $('title').textContent = title(item);
  const record = item.capture;
  $('metadata').textContent = `${record.kind} · Provider validation: ${record.providerValidation} · ${record.elapsedMs} ms · ${record.id}. This describes the provider response, not a committed mission; further runtime validation may use a fallback.`;
  $('mission').textContent = record.output === null ? 'No final output (busy, transport or malformed envelope failure).' : JSON.stringify(record.output, null, 2);
  $('request').textContent = JSON.stringify(record.request, null, 2);
  const review = myReview(item);
  for (const key of Object.keys(labels)) $(key).value = review?.scores[key] ?? '';
  $('decision').value = review?.decision ?? ''; $('notes').value = review?.notes ?? '';
  $('saved').textContent = review ? `Saved ${review.reviewedAt}` : 'Not yet graded by you'; queue();
}
async function refresh() {
  if (!mayLeave()) return;
  try {
    const response = await fetch('/api/review'); const next = await response.json();
    if (!response.ok) throw new Error(next.error);
    workspace = next;
    $('status').textContent = workspace.partial ? 'Capture append in progress; refresh to see the last record.' : 'Captures loaded. Refresh to include newly generated missions.';
    const item = workspace.items.find(i => i.capture.id === selected?.capture.id) ?? workspace.items[0];
    if (item) show(item); else { queue(); $('detail').hidden = true; $('empty').hidden = false; }
  } catch (error) { $('status').textContent = error.message; }
}
$('review').addEventListener('input', () => { dirty = true; });
$('reviewer').addEventListener('change', () => {
  try { localStorage.setItem('mission-reviewer', $('reviewer').value); } catch {}
  if (selected && !dirty) show(selected); else queue();
});
$('review').onsubmit = async (event) => {
  event.preventDefault();
  const item = selected;
  const body = {id: item.capture.id, fingerprint: item.fingerprint, reviewer: $('reviewer').value,
    decision: $('decision').value, notes: $('notes').value, scores: Object.fromEntries(Object.keys(labels).map(key => [key, Number($(key).value)]))};
  saving = true;
  const controls = [$('reviewer'), ...$('review').querySelectorAll('input, select, textarea, button')];
  controls.forEach(control => { control.disabled = true; });
  try {
    const response = await fetch('/api/review', {method: 'POST', headers: {'Content-Type':'application/json', 'X-Review-Token':workspace.token}, body: JSON.stringify(body)});
    const review = await response.json(); if (!response.ok) throw new Error(review.error);
    workspace.reviews = workspace.reviews.filter(r => !(r.id === review.id && r.reviewer === review.reviewer)); workspace.reviews.push(review);
    if (selected === item) { dirty = false; $('saved').textContent = `Saved ${review.reviewedAt}`; }
    queue();
  } catch(error) { $('saved').textContent = `Feedback not saved: ${error.message}`; }
  finally { saving = false; controls.forEach(control => { control.disabled = false; }); }
};
$('refresh').onclick = refresh; $('filter').onchange = queue;
window.addEventListener('beforeunload', event => { if (dirty) { event.preventDefault(); event.returnValue = ''; } });
try { $('reviewer').value = localStorage.getItem('mission-reviewer') ?? ''; } catch {}
refresh();
