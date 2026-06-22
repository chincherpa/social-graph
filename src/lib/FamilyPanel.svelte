<script>
  import * as api from "./api.js";
  import { displayName } from "./api.js";

  let { person, people = [], onChange = () => {} } = $props();

  let family = $state([]);
  let relationType = $state("Ehepartner");
  let nameInput = $state("");
  let error = $state("");

  const relationTypes = ["Ehepartner", "Kind", "Mutter", "Vater", "Geschwister", "Sonstige"];

  async function loadFamily() {
    if (!person) {
      family = [];
      return;
    }
    try {
      family = await api.getFamily(person.id);
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    loadFamily();
  });

  function findExistingByName(name) {
    const lower = name.trim().toLowerCase();
    return people.find((p) => displayName(p).toLowerCase() === lower || p.last_name.toLowerCase() === lower);
  }

  async function addMember() {
    if (!nameInput.trim() || !person) return;
    const existing = findExistingByName(nameInput);
    try {
      await api.addFamilyMember({
        personId: person.id,
        familyId: existing?.id ?? null,
        newFamilyLastName: existing ? null : nameInput.trim(),
        relationType,
      });
      nameInput = "";
      await loadFamily();
      onChange();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeMember(familyId) {
    if (!person) return;
    try {
      await api.removeFamilyMember(person.id, familyId);
      await loadFamily();
      onChange();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="panel">
  <h3>Familie</h3>

  {#if error}
    <button type="button" class="error" onclick={() => (error = "")}>{error}</button>
  {/if}

  <ul class="family-list">
    {#each family as fm (fm.id)}
      <li>
        <span class="rel">{fm.relation_type}</span>
        <span class="who">{fm.family_nickname ?? [fm.family_first_name, fm.family_last_name].filter(Boolean).join(" ")}</span>
        <button class="remove" onclick={() => removeMember(fm.family_id)} aria-label="Entfernen">×</button>
      </li>
    {/each}
    {#if family.length === 0}
      <li class="empty">Keine Familienmitglieder</li>
    {/if}
  </ul>

  <div class="add-row">
    <select bind:value={relationType}>
      {#each relationTypes as r}
        <option value={r}>{r}</option>
      {/each}
    </select>
    <input
      type="text"
      bind:value={nameInput}
      placeholder="Name eingeben…"
      onkeydown={(e) => e.key === "Enter" && addMember()}
    />
    <button class="primary" onclick={addMember}>+</button>
  </div>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 1rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }
  h3 {
    margin: 0;
    font-size: 1rem;
  }
  .family-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .family-list li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    padding: 0.35rem 0.5rem;
    background: #f8fafc;
    border-radius: 6px;
  }
  .family-list li.empty {
    color: #94a3b8;
    font-style: italic;
  }
  .rel {
    font-weight: 600;
    color: #475569;
  }
  .who {
    flex: 1;
  }
  .remove {
    border: none;
    background: transparent;
    color: #b91c1c;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0 0.25rem;
  }
  .add-row {
    display: flex;
    gap: 0.4rem;
  }
  .add-row select {
    flex: 0 0 auto;
  }
  .add-row input {
    flex: 1;
    min-width: 0;
  }
  select,
  input {
    padding: 0.45rem;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-size: 0.85rem;
    font-family: inherit;
  }
  button {
    padding: 0.45rem 0.7rem;
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
  .error {
    background: #fee2e2;
    color: #b91c1c;
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    font-size: 0.8rem;
    cursor: pointer;
  }
</style>
