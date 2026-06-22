<script>
  import { displayName, directionalKinds, reciprocalKind } from "./api.js";
  let {
    relationship,
    peopleById,
    onSave = () => {},
    onDelete = () => {},
    onSwap = () => {},
    onClose = () => {},
  } = $props();

  let kind = $state(relationship?.kind ?? "kennt");
  let strength = $state(relationship?.strength ?? 3);
  let note = $state(relationship?.note ?? "");

  $effect(() => {
    kind = relationship?.kind ?? "kennt";
    strength = relationship?.strength ?? 3;
    note = relationship?.note ?? "";
  });

  const kinds = [
    "Bruder",
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
    "Mutter",
    "Oma",
    "Onkel",
    "Opa",
    "Partner",
    "Schwester",
    "Sohn",
    "Tante",
    "Tochter",
    "Vater"
    ];

  const isDirectional = () => directionalKinds.has(kind);

  function submit() {
    onSave({ id: relationship.id, kind, strength, note: note.trim() || null });
  }

  const fromName = () => displayName(peopleById.get(relationship?.from_id));
  const toName = () => displayName(peopleById.get(relationship?.to_id));
  const reciprocalLabel = () => reciprocalKind(kind, peopleById.get(relationship?.to_id)?.gender);
</script>

<div class="panel">
  {#if isDirectional()}
    <h3>
      {fromName()} <span class="arrow">→ {kind} von →</span> {toName()}
    </h3>
    {#if reciprocalLabel() !== kind}
      <span class="reciprocal">{toName()} ist also {reciprocalLabel()} von {fromName()}</span>
    {/if}
    <button class="swap" onclick={() => onSwap(relationship.id)}>⇄ Richtung tauschen</button>
  {:else}
    <h3>{fromName()} ↔ {toName()}</h3>
  {/if}

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
  .arrow {
    color: #94a3b8;
    font-weight: 400;
    font-size: 0.85rem;
  }
  .reciprocal {
    color: #64748b;
    font-size: 0.78rem;
  }
  .swap {
    align-self: flex-start;
    padding: 0.3rem 0.6rem;
    font-size: 0.78rem;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    background: #f1f5f9;
    cursor: pointer;
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
