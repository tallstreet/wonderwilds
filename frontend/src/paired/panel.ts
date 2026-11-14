import type {
  HouseholdProfile,
  PairedCommand,
  PairedSnapshot,
} from '../../../contracts/generated/contracts.ts';
import { PairedClient, PairedSession } from './client.ts';
const escape = (value: string) =>
  String(value).replace(
    /[&<>"']/g,
    (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[char]!,
  );
const title = (value: string) =>
  value.replaceAll('_', ' ').replace(/\b\w/g, (char) => char.toUpperCase());
export function profileCode(id: string) {
  return id.replaceAll('-', '').slice(-8).toUpperCase();
}
export function profileLabel(profiles: HouseholdProfile[], id: string) {
  const index = profiles.findIndex((profile) => profile.id === id);
  return `${index < 0 ? 'Player not listed' : `Player ${index + 1}`} · ${profileCode(id)}`;
}
export function selectedProfileUrl(current: string, profileId: string) {
  const url = new URL(current);
  url.searchParams.set('player', profileId);
  return url.href;
}
export function allowedProfile(
  requested: string | null,
  fallback: string,
  profiles: HouseholdProfile[],
) {
  if (!requested) return fallback;
  if (!profiles.some((profile) => profile.id === requested))
    throw new Error('Choose a player from this household.');
  return requested;
}
type Payload = PairedCommand['payload'];
export class PairedPanel {
  private readonly commands: PairedSession;
  private loading = false;
  private loaded = false;
  private status = '';
  constructor(
    private client: PairedClient,
    private actor: string,
    private profiles: HouseholdProfile[],
    private changed: () => void,
    private canAct: () => boolean,
    private art: (id: string) => string,
  ) {
    this.commands = new PairedSession(client, actor, localStorage);
    window.addEventListener('pagehide', () => client.destroy(), { once: true });
  }
  get unresolved() {
    return this.commands.busy || this.commands.hasPending || this.commands.needsRefresh;
  }
  async load() {
    if (this.loading) return;
    this.loading = true;
    this.changed();
    try {
      if (this.commands.hasPending) await this.commands.retry();
      await this.commands.refresh();
      this.loaded = true;
      this.status = '';
    } catch {
      this.status = this.commands.hasPending
        ? 'Your saved shared-device move needs a retry.'
        : 'Shared-device play could not connect. Your solo adventure stays saved.';
    } finally {
      this.loading = false;
      this.changed();
    }
  }
  handle(button: HTMLButtonElement) {
    if (button.id === 'paired-refresh') {
      void this.load();
      return true;
    }
    if (button.id === 'paired-retry') {
      void this.retry();
      return true;
    }
    if (button.dataset.pairedPayload) {
      void this.run(JSON.parse(button.dataset.pairedPayload) as Payload);
      return true;
    }
    if (button.dataset.selectProfile) {
      if (!this.canAct() || this.unresolved) {
        this.status = 'Finish or retry the current move before changing players.';
        this.changed();
        return true;
      }
      if (!this.profiles.some((profile) => profile.id === button.dataset.selectProfile))
        return true;
      location.assign(selectedProfileUrl(location.href, button.dataset.selectProfile));
      return true;
    }
    return false;
  }
  private button(label: string, payload: Payload, disabled = false) {
    return `<button ${disabled ? 'disabled' : ''} data-paired-payload="${escape(JSON.stringify(payload))}">${escape(label)}</button>`;
  }
  render() {
    const actorLabel = profileLabel(this.profiles, this.actor);
    const switches = `${!this.canAct() ? '<p class="campaign-hazard">Finish the current solo activity or confirm its saved move before changing players or starting shared play.</p>' : ''}<h3>Who is holding the device?</h3><p>Active player: <strong>${escape(actorLabel)}</strong>. Any learning answers belong only to this selected player. Navigator and keeper are teamwork roles; swapping roles does not switch the learner.</p><div class="campaign-actions">${this.profiles.map((profile) => `<button data-select-profile="${escape(profile.id)}" ${profile.id === this.actor || this.unresolved || !this.canAct() ? 'disabled' : ''}>${profile.id === this.actor ? 'Playing as' : 'Choose'} ${escape(profileLabel(this.profiles, profile.id))}</button>`).join('')}</div>`;
    const status = `<p class="campaign-status" role="status">${escape(this.status)}</p>`;
    if (!this.profiles.length)
      return '<p class="campaign-empty">The saved household player list is unavailable. Continue your solo adventure and reconnect to choose players.</p>';
    if (this.commands.hasPending)
      return `${switches}<div class="campaign-hazard"><strong>Confirm your saved shared-device move</strong><div class="campaign-actions"><button id="paired-retry">Retry saved move</button></div></div>${status}`;
    if (this.commands.needsRefresh)
      return `${switches}<p>Your move is saved. Refresh the shared expedition before choosing again.</p><div class="campaign-actions"><button id="paired-refresh">Refresh shared expedition</button></div>${status}`;
    if (!this.loaded)
      return `${switches}<p>${this.loading ? 'Loading shared-device play…' : 'Play together on this device. Each person keeps their own adventure and friends.'}</p><div class="campaign-actions"><button id="paired-refresh" ${this.loading ? 'disabled' : ''}>Open shared-device play</button></div>${status}`;
    const s = this.commands.snapshot!;
    const keepsakes = s.keepsakes.length
      ? `<h3>Your shared keepsakes</h3>${(s.keepsakeCount ?? s.keepsakes.length) > s.keepsakes.length ? `<p>Showing the most recent ${s.keepsakes.length} of ${s.keepsakeCount} keepsakes.</p>` : ''}<ul>${s.keepsakes.map((item) => `<li>${escape(title(item.keepsakeId))}${item.sourceProfileId ? ` from ${escape(profileLabel(this.profiles, item.sourceProfileId))}` : ''}</li>`).join('')}</ul>`
      : '';
    if (s.mode === 'solo')
      return `${switches}<h3>A shared trail</h3><p>One navigator scouts the path. One keeper protects the team. Choose two existing players; each person brings a friend they actually own.</p>${
        this.profiles.length < 2
          ? '<p class="campaign-empty">Shared expeditions need two saved players in this household. Your solo adventure and every reward remain available.</p>'
          : `<div class="campaign-actions">${this.profiles
              .filter((profile) => profile.id !== this.actor)
              .map((profile) =>
                this.button(
                  `Pair with ${profileLabel(this.profiles, profile.id)}`,
                  { type: 'start', partnerProfileId: profile.id },
                  !this.canAct(),
                ),
              )
              .join('')}</div>`
      }${keepsakes}${status}`;
    const role =
      s.navigatorProfileId === this.actor
        ? 'navigator'
        : s.keeperProfileId === this.actor
          ? 'keeper'
          : undefined;
    const partner = s.navigatorProfileId === this.actor ? s.keeperProfileId : s.navigatorProfileId;
    const active = s.practice?.outcome === 'active';
    const ownCompanion = s.companions.find((entry) => entry.profileId === this.actor);
    const ownOffer = s.offers.find((offer) => offer.profileId === this.actor);
    const confirmed = s.confirmedProfileIds.includes(this.actor);
    const exchangeTerms = `<div class="campaign-grid" aria-label="Both keepsake offers">${[
      s.navigatorProfileId!,
      s.keeperProfileId!,
    ]
      .map((owner) => {
        const offer = s.offers.find((item) => item.profileId === owner);
        return `<article class="campaign-card"><strong>${escape(profileLabel(this.profiles, owner))}</strong><p>${offer ? escape(title(offer.keepsakeId)) : 'No keepsake chosen yet'}</p><p>${s.confirmedProfileIds.includes(owner) ? 'Confirmed these offers' : 'Waiting for confirmation'}</p></article>`;
      })
      .join('')}</div>`;

    return `${switches}<div class="campaign-dialogue"><strong>Navigator: ${escape(profileLabel(this.profiles, s.navigatorProfileId!))}</strong><p>Keeper: ${escape(profileLabel(this.profiles, s.keeperProfileId!))}</p><p>You are the ${role ?? 'observer'}. Your friends and solo progress stay with their owners.</p></div><h3>Bring your own friend</h3><div class="campaign-actions">${s.availableCompanions.map((friend) => this.button(`${ownCompanion?.companionId === friend.id ? 'Selected: ' : ''}${title(friend.id)} · Level ${friend.level}`, { type: 'select_companion', companionId: friend.id }, active || !role || !this.canAct())).join('') || '<p>Choose a starter in your own solo adventure first, then come back.</p>'}</div><div class="campaign-grid">${s.companions.map((entry) => `<article class="campaign-card">${this.art(entry.companionId)}<strong>${escape(title(entry.companionId))}</strong><p>Belongs to ${escape(profileLabel(this.profiles, entry.profileId))}</p></article>`).join('')}</div>${s.practice ? this.practice(s) : '<p>Take turns scouting and protecting the team in a shared practice. It does not record academic answers or award learning XP.</p>'}<div class="campaign-actions">${!active && role === 'navigator' ? this.button(s.practice ? 'Practise again together' : 'Start shared practice', { type: 'start_practice' }, s.companions.length < 2 || !this.canAct()) : ''}${!active ? this.button('Swap navigator and keeper', { type: 'swap_roles' }, !role || !this.canAct()) : ''}${this.button('Continue solo', { type: 'continue_solo' }, !role || !this.canAct())}</div><h3>Exchange a keepsake</h3>${exchangeTerms}<p>Offer a keepsake from your own saved bag. Both players choose and confirm separately on their own profile. You keep the original; a copy becomes a memory for your partner.</p>${s.exchangeComplete ? '<p class="campaign-dialogue">Both players confirmed. Your shared keepsakes are saved independently.</p>' : `<p>Your offer: ${ownOffer ? escape(title(ownOffer.keepsakeId)) : 'not chosen yet'}${confirmed ? ' · confirmed' : ''}</p><div class="campaign-actions">${s.availableKeepsakeIds.map((id) => this.button(`Offer ${title(id)}`, { type: 'offer_keepsake', keepsakeId: id }, !role || confirmed || !this.canAct())).join('') || '<p>Find an eligible keepsake in your own adventure before offering one.</p>'}${ownOffer && !confirmed ? this.button('Confirm these two copied keepsakes', { type: 'confirm_exchange' }, !role || s.offers.length < 2 || !this.canAct()) : ''}</div><p>${partner ? escape(profileLabel(this.profiles, partner)) : 'Your partner'} must choose their own profile to offer and confirm their keepsake.</p>`}${keepsakes}${status}`;
  }
  private practice(s: PairedSnapshot) {
    const p = s.practice!;
    return `<div class="campaign-hazard"><strong>${p.outcome === 'active' ? `${title(p.nextRole)}’s turn` : p.outcome === 'won' ? 'Shared trail restored!' : 'Take a breath and try again together.'}</strong><progress class="campaign-meter" aria-label="Shared trail light" value="${p.progress}" max="${s.practiceTarget}">${p.progress} / ${s.practiceTarget}</progress><p>Light ${p.progress} / ${s.practiceTarget} · Pressure ${p.pressure} / ${s.practicePressureLimit} · Turns ${p.turns} / ${s.practiceTurnLimit}</p><p>${p.outcome === 'active' && !s.legalMovements.length ? 'Choose the player holding the next role to take their turn.' : 'Navigator scouts or rests; keeper braces or rallies. Every turn stays credited to the selected player and their own friend.'}</p><div class="campaign-actions">${s.legalMovements.map((move) => this.button(title(move), { type: 'practice_turn', movement: move as 'scout' | 'rest' | 'brace' | 'rally' }, !this.canAct())).join('')}</div><ol>${p.history
      .slice(-4)
      .map(
        (turn) =>
          `<li>${escape(profileLabel(this.profiles, turn.profileId))} · ${escape(title(turn.companionId))}: ${escape(title(turn.movement))} (${escape(turn.role)})</li>`,
      )
      .join('')}</ol></div>`;
  }
  private async run(payload: Payload) {
    if (!this.canAct() || this.unresolved) return;
    this.loading = true;
    this.changed();
    try {
      await this.commands.choose(payload);
      await this.commands.refresh();
      this.loaded = true;
      this.status = 'Shared expedition saved.';
    } catch {
      this.status = this.commands.hasPending
        ? 'That move needs confirmation. Retry the exact saved move.'
        : 'That move is not available. Refresh the shared expedition to continue.';
    } finally {
      this.loading = false;
      this.changed();
    }
  }
  private async retry() {
    if (this.commands.busy) return;
    try {
      await this.commands.retry();
      await this.commands.refresh();
      this.loaded = true;
      this.status = 'Your shared move is safely saved.';
    } catch {
      this.status = 'Your move could not be confirmed. Retry or refresh when connected.';
    }
    this.changed();
  }
}
