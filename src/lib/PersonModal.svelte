<script>
  import { displayName, placeholderFor } from "./api.js";
  import PersonForm from "./PersonForm.svelte";
  import FamilyPanel from "./FamilyPanel.svelte";
  import RelationsPanel from "./RelationsPanel.svelte";

  let {
    person,
    people = [],
    relationships = [],
    peopleById,
    onSave = () => {},
    onDelete = () => {},
    onClose = () => {},
    onChange = () => {},
    onSelectPerson = () => {},
    onSelectEdge = () => {},
  } = $props();

  let editing = $state(false);

  $effect(() => {
    person?.id;
    editing = false;
  });

  async function handleSave(payload) {
    await onSave(payload);
    editing = false;
  }

  function onKeydown(e) {
    if (e.key === "Escape") onClose();
  }

  function fmtDate(d) {
    if (!d) return null;
    return new Date(d).toLocaleDateString("de-DE");
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if person}
  <div class="overlay" onclick={onClose}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      {#if editing}
        <button class="close" onclick={onClose} aria-label="Schließen">×</button>
        <PersonForm {person} onSave={handleSave} {onDelete} onClose={() => (editing = false)} />
      {:else}
        <div class="header">
          <h2>{displayName(person)}</h2>
          <div class="header-actions">
            <button class="edit-btn" onclick={() => (editing = true)}>✎ Bearbeiten</button>
            <button class="close" onclick={onClose} aria-label="Schließen">×</button>
          </div>
        </div>

        <div class="view">
          <img
            src={person.image_data ?? placeholderFor(person.gender)}
            alt={displayName(person)}
            class="avatar"
            style="border-color:{person.color}"
          />

          <dl>
            <dt>Vorname</dt>
            <dd>{person.first_name ?? "—"}</dd>
            <dt>Nachname</dt>
            <dd>{person.last_name}</dd>
            {#if person.nickname}
              <dt>Spitzname</dt>
              <dd>{person.nickname}</dd>
            {/if}
            <dt>Geschlecht</dt>
            <dd>{person.gender === "m" ? "männlich" : person.gender === "w" ? "weiblich" : "—"}</dd>
            <dt>Geburtsdatum</dt>
            <dd>{fmtDate(person.birth_date) ?? "—"}</dd>
            <dt>Kennengelernt am</dt>
            <dd>{fmtDate(person.known_since) ?? "—"}</dd>
            {#if person.address}
              <dt>Adresse</dt>
              <dd>{person.address}</dd>
            {/if}
            {#if person.employer}
              <dt>Arbeitgeber</dt>
              <dd>{person.employer}</dd>
            {/if}
            {#if person.note}
              <dt>Notiz</dt>
              <dd class="note">{person.note}</dd>
            {/if}
          </dl>
        </div>

        <FamilyPanel {person} {people} onChange={onChange} />
        <RelationsPanel {person} {relationships} {peopleById} {onSelectPerson} {onSelectEdge} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: transparent;
    display: flex;
    align-items: stretch;
    justify-content: flex-end;
    z-index: 100;
  }
  .modal {
    position: relative;
    width: min(340px, 90vw);
    height: 100%;
    overflow-y: auto;
    background: #f8fafc;
    box-shadow: -4px 0 24px rgba(0, 0, 0, 0.15);
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .header h2 {
    margin: 0;
    font-size: 1.15rem;
    color: #1e293b;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .edit-btn {
    padding: 0.4rem 0.7rem;
    border-radius: 6px;
    border: 1px solid #3b82f6;
    background: #eff6ff;
    color: #1d4ed8;
    cursor: pointer;
    font-size: 0.8rem;
  }
  .close {
    border: none;
    background: transparent;
    font-size: 1.4rem;
    line-height: 1;
    color: #64748b;
    cursor: pointer;
    padding: 0.1rem 0.4rem;
    border-radius: 6px;
  }
  .close:hover {
    background: #e2e8f0;
  }
  .view {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.85rem;
    background: white;
    border-radius: 8px;
    padding: 1rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  .avatar {
    width: 88px;
    height: 88px;
    border-radius: 50%;
    object-fit: cover;
    border: 3px solid #cbd5e1;
  }
  dl {
    width: 100%;
    margin: 0;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 0.75rem;
  }
  dt {
    font-size: 0.78rem;
    color: #64748b;
    font-weight: 600;
  }
  dd {
    margin: 0;
    font-size: 0.88rem;
    color: #1e293b;
  }
  dd.note {
    white-space: pre-wrap;
  }
</style>
