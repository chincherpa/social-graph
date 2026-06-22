<script>
  import { displayName } from "./api.js";
  let { relationship, peopleById, onSave = () => {}, onDelete = () => {}, onClose = () => {} } = $props();

  let kind = $state(relationship?.kind ?? "kennt");
  let strength = $state(relationship?.strength ?? 3);
  let note = $state(relationship?.note ?? "");

  $effect(() => {
    kind = relationship?.kind ?? "kennt";
    strength = relationship?.strength ?? 3;
    note = relationship?.note ?? "";
  });

  const kinds = [
    "Ehefrau",
    "Ehemann",
    "Enkel",
    "Enkelin",
    "Ex-Ehefrau",
    "Ex-Ehemann",
    "Ex-Partner",
    "Familie",
    "Freund",
    "kennt",
    "Kollege",
    "Oma",
    "Onkel",
    "Opa",
    "Partner",
    "Sohn",
    "Tante",
    "Tochter"
    ];

  function submit() {
    onSave({ id: relationship.id, kind, strength, note: note.trim() || null });
  }

  const fromName = () => displayName(peopleById.get(relationship?.from_id));
  const toName = () => displayName(peopleById.get(relationship?.to_id));
</script>

<div class="panel">
  <h3>{fromName()} ↔ {toName()}</h3>

  <label>
    Art der Beziehung
    <select bind:value={kind}>
      {#each kinds as k}
        <option value={k}>{k}</option>
      {/each}
    </select>
  </label>

  <label>
    Stärke ({strength})
    <input type="range" min="1" max="5" bind:value={strength} />
  </label>

  <label>
    Notiz
    <textarea bind:value={note} rows="2" placeholder="optional"></textarea>
  </label>

  <div class="actions">
    <button class="primary" onclick={submit}>Speichern</button>
    <button class="danger" onclick={() => onDelete(relationship.id)}>Löschen</button>
    <button onclick={onClose}>Schließen</button>
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  h3 {
    margin: 0;
    font-size: 1rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #475569;
  }
  select,
  textarea,
  input[type="range"] {
    padding: 0.4rem;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-family: inherit;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  button {
    padding: 0.5rem 0.9rem;
    border-radius: 6px;
    border: 1px solid #cbd5e1;
    background: #f1f5f9;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .primary {
    background: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }
  .danger {
    background: #fee2e2;
    color: #b91c1c;
    border-color: #fca5a5;
  }
</style>
