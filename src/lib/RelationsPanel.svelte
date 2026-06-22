<script>
  import { displayName, placeholderFor, directionalKinds, reciprocalKind } from "./api.js";

  let { person, relationships = [], peopleById, onSelectPerson = () => {}, onSelectEdge = () => {} } = $props();

  const rows = $derived.by(() => {
    if (!person) return [];
    return relationships
      .filter((r) => r.from_id === person.id || r.to_id === person.id)
      .map((r) => {
        const isFrom = r.from_id === person.id;
        const otherId = isFrom ? r.to_id : r.from_id;
        const other = peopleById.get(otherId);
        // gerichtete Kinds lesen sich "from ist [kind] von to" -> wenn person selbst
        // das to_id ist, gilt für sie die reziproke Bezeichnung (z.B. Tochter -> Mutter/Vater).
        const label = directionalKinds.has(r.kind) && !isFrom ? reciprocalKind(r.kind, person.gender) : r.kind;
        return { edge: r, other, label };
      })
      .filter((row) => row.other)
      .sort((a, b) => b.edge.strength - a.edge.strength || displayName(a.other).localeCompare(displayName(b.other)));
  });

  function avatarSrc(p) {
    return p.image_data ?? placeholderFor(p.gender);
  }
</script>

<div class="panel">
  <h3>Verbindungen <span class="count">({rows.length})</span></h3>

  <ul class="relations-list">
    {#each rows as row (row.edge.id)}
      <li>
        <button class="who" onclick={() => onSelectPerson(row.other.id)}>
          <img src={avatarSrc(row.other)} alt="" class="avatar" style="border-color:{row.other.color}" />
          <span class="info">
            <span class="name">{displayName(row.other)}</span>
            <span class="kind">{row.label}</span>
          </span>
        </button>
        <span class="strength" title="Stärke {row.edge.strength}/5">
          {#each Array(5) as _, i}
            <span class="dot" class:filled={i < row.edge.strength}></span>
          {/each}
        </span>
        <button class="edit" onclick={() => onSelectEdge(row.edge.id)} aria-label="Beziehung bearbeiten">✎</button>
      </li>
    {/each}
    {#if rows.length === 0}
      <li class="empty">Keine Verbindungen</li>
    {/if}
  </ul>
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
  .count {
    color: #94a3b8;
    font-weight: 400;
    font-size: 0.85rem;
  }
  .relations-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .relations-list li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.35rem;
    background: #f8fafc;
    border-radius: 6px;
  }
  .relations-list li.empty {
    color: #94a3b8;
    font-style: italic;
    padding: 0.35rem 0.5rem;
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    text-align: left;
    border-radius: 6px;
  }
  .who:hover {
    background: #e2e8f0;
  }
  .who:hover .name {
    text-decoration: underline;
    color: #1d4ed8;
  }
  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    object-fit: cover;
    border: 2px solid #cbd5e1;
    flex-shrink: 0;
  }
  .info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .name {
    font-size: 0.85rem;
    font-weight: 600;
    color: #1e293b;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .kind {
    font-size: 0.72rem;
    color: #64748b;
  }
  .strength {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #e2e8f0;
  }
  .dot.filled {
    background: #3b82f6;
  }
  .edit {
    border: none;
    background: transparent;
    color: #64748b;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0.2rem 0.3rem;
    flex-shrink: 0;
    border-radius: 4px;
  }
  .edit:hover {
    background: #e2e8f0;
    color: #1e293b;
  }
</style>
