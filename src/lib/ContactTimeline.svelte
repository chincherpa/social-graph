<script>
  import {
    contactTypes,
    contactTypeLabel,
    contactTypeIcon,
    listContactEvents,
    addContactEvent,
    deleteContactEvent,
  } from "./api.js";

  let { person, onChange = () => {} } = $props();

  let events = $state([]);
  let expanded = $state(false);
  let adding = $state(false);
  let pendingType = $state(null);
  let pendingDate = $state(today());
  let pendingNote = $state("");

  // Events neu laden, sobald sich die Person ändert.
  $effect(() => {
    person?.id;
    expanded = false;
    resetAdd();
    load();
  });

  function today() {
    return new Date().toISOString().slice(0, 10);
  }

  function fmtDate(d) {
    if (!d) return "—";
    return new Date(d).toLocaleDateString("de-DE");
  }

  async function load() {
    events = person?.id ? await listContactEvents(person.id) : [];
  }

  function resetAdd() {
    adding = false;
    pendingType = null;
    pendingDate = today();
    pendingNote = "";
  }

  function startAdd() {
    expanded = true;
    adding = true;
    pendingType = null;
    pendingDate = today();
    pendingNote = "";
  }

  async function saveEvent() {
    if (!pendingType || !pendingDate) return;
    await addContactEvent(person.id, pendingType, pendingDate, pendingNote.trim());
    resetAdd();
    await load();
    onChange();
  }

  async function removeEvent(id) {
    await deleteContactEvent(id);
    await load();
    onChange();
  }

  let latest = $derived(events[0] ?? null);
</script>

<div class="timeline">
  <div class="summary">
    <span class="summary-text">
      {#if latest}
        {contactTypeIcon(latest.contact_type)}
        {contactTypeLabel(latest.contact_type)} – {fmtDate(latest.contact_date)}
      {:else}
        —
      {/if}
    </span>
    <span class="summary-actions">
      <button class="mini-btn" onclick={startAdd} aria-label="Kontakt hinzufügen">＋</button>
      {#if events.length}
        <button
          class="mini-btn"
          onclick={() => (expanded = !expanded)}
          aria-label={expanded ? "Verlauf zuklappen" : "Verlauf aufklappen"}
        >
          {expanded ? "▴" : "▾"}
        </button>
      {/if}
    </span>
  </div>

  {#if adding}
    <div class="add-form">
      {#if !pendingType}
        <div class="pills">
          {#each contactTypes as ct}
            <button class="pill" onclick={() => (pendingType = ct.value)}>
              {contactTypeIcon(ct.value)} {ct.label}
            </button>
          {/each}
        </div>
      {:else}
        <div class="add-fields">
          <span class="add-type">{contactTypeIcon(pendingType)} {contactTypeLabel(pendingType)}</span>
          <input type="date" bind:value={pendingDate} />
          <input
            type="text"
            class="note-input"
            placeholder="Notiz (optional)"
            bind:value={pendingNote}
          />
          <div class="add-actions">
            <button class="pill primary" onclick={saveEvent}>Speichern</button>
            <button class="pill" onclick={resetAdd}>Abbrechen</button>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if expanded && events.length}
    <ul class="events">
      {#each events as ev (ev.id)}
        <li class="event">
          <span class="dot">{contactTypeIcon(ev.contact_type)}</span>
          <div class="event-body">
            <div class="event-head">
              <span class="event-label">{contactTypeLabel(ev.contact_type)}</span>
              <span class="event-date">{fmtDate(ev.contact_date)}</span>
              <button class="del-btn" onclick={() => removeEvent(ev.id)} aria-label="Löschen">×</button>
            </div>
            {#if ev.note}
              <div class="event-note">{ev.note}</div>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }
  .summary-text {
    font-size: 0.88rem;
    color: #1e293b;
  }
  .summary-actions {
    display: flex;
    gap: 0.1rem;
    flex-shrink: 0;
  }
  .mini-btn {
    border: none;
    background: transparent;
    color: #64748b;
    cursor: pointer;
    font-size: 0.95rem;
    line-height: 1;
    padding: 0.1rem 0.25rem;
    border-radius: 6px;
  }
  .mini-btn:hover {
    color: #1d4ed8;
    background: #eff6ff;
  }
  .add-form {
    background: #f1f5f9;
    border-radius: 8px;
    padding: 0.5rem;
  }
  .pills {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }
  .pill {
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    border: 1px solid #cbd5e1;
    background: white;
    color: #1e293b;
    font-size: 0.75rem;
    cursor: pointer;
  }
  .pill:hover {
    background: #e2e8f0;
  }
  .pill.primary {
    border-color: #3b82f6;
    background: #eff6ff;
    color: #1d4ed8;
  }
  .add-fields {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .add-type {
    font-size: 0.8rem;
    font-weight: 600;
    color: #334155;
  }
  .add-fields input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.3rem 0.4rem;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-size: 0.82rem;
  }
  .add-actions {
    display: flex;
    gap: 0.3rem;
  }
  .events {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0 0;
    display: flex;
    flex-direction: column;
  }
  .event {
    position: relative;
    display: flex;
    gap: 0.5rem;
    padding-bottom: 0.6rem;
  }
  /* Vertikale Linie zwischen den Dots */
  .event:not(:last-child)::before {
    content: "";
    position: absolute;
    left: 0.6rem;
    top: 1.25rem;
    bottom: 0;
    width: 1px;
    background: #cbd5e1;
  }
  .dot {
    flex-shrink: 0;
    width: 1.2rem;
    text-align: center;
    font-size: 0.85rem;
    z-index: 1;
  }
  .event-body {
    flex: 1;
    min-width: 0;
  }
  .event-head {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }
  .event-label {
    font-size: 0.82rem;
    font-weight: 600;
    color: #1e293b;
  }
  .event-date {
    font-size: 0.75rem;
    color: #64748b;
  }
  .del-btn {
    margin-left: auto;
    border: none;
    background: transparent;
    color: #94a3b8;
    cursor: pointer;
    font-size: 0.95rem;
    line-height: 1;
    padding: 0 0.2rem;
  }
  .del-btn:hover {
    color: #dc2626;
  }
  .event-note {
    font-size: 0.78rem;
    color: #475569;
    white-space: pre-wrap;
  }
</style>
