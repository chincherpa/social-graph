<script>
  import { onMount } from "svelte";
  import Graph from "./lib/Graph.svelte";
  import MapView from "./lib/MapView.svelte";
  import PersonForm from "./lib/PersonForm.svelte";
  import PersonModal from "./lib/PersonModal.svelte";
  import EdgeForm from "./lib/EdgeForm.svelte";
  import * as api from "./lib/api.js";

  let people = $state([]);
  let relationships = $state([]);

  let selectedPersonId = $state(null); // für Edit-Panel
  let selectedEdgeId = $state(null);
  let showNewPersonForm = $state(false);

  let view = $state("graph"); // "graph" | "map"

  let connectMode = $state(false);
  let connectSource = $state(null); // id der ersten gewählten Person beim Verbinden

  let error = $state("");

  const peopleById = $derived(new Map(people.map((p) => [p.id, p])));
  const selectedPerson = $derived(people.find((p) => p.id === selectedPersonId) ?? null);
  const selectedEdge = $derived(relationships.find((r) => r.id === selectedEdgeId) ?? null);

  async function refresh() {
    try {
      const graph = await api.getGraph();
      people = graph.people;
      relationships = graph.relationships;
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);

  // ---------- Personen ----------

  function handleNodeClick(id) {
    selectedEdgeId = null;
    showNewPersonForm = false;
    selectedPersonId = id;
  }

  function openNewPerson() {
    selectedPersonId = null;
    selectedEdgeId = null;
    showNewPersonForm = true;
  }

  async function savePerson(payload) {
    try {
      if (payload.id) {
        await api.updatePerson(payload);
      } else {
        await api.addPerson(payload);
        selectedPersonId = null;
      }
      showNewPersonForm = false;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function deletePerson(id) {
    try {
      await api.deletePerson(id);
      selectedPersonId = null;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  // ---------- Beziehungen ----------

  function handleEdgeClick(relId) {
    selectedPersonId = null;
    showNewPersonForm = false;
    selectedEdgeId = relId;
  }

  function toggleConnectMode() {
    connectMode = !connectMode;
    connectSource = null;
    selectedPersonId = null;
    selectedEdgeId = null;
    showNewPersonForm = false;
  }

  async function handleConnectTarget(id) {
    if (connectSource === null) {
      connectSource = id;
      return;
    }
    if (connectSource === id) {
      connectSource = null;
      return;
    }
    try {
      const rel = await api.addRelationship({
        personA: connectSource,
        personB: id,
        kind: "kennt",
        strength: 3,
        note: null,
      });
      connectMode = false;
      connectSource = null;
      await refresh();
      selectedEdgeId = rel.id;
    } catch (e) {
      error = String(e);
      connectSource = null;
    }
  }

  async function saveRelationship(payload) {
    try {
      await api.updateRelationship(payload);
      selectedEdgeId = null;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function swapRelationship(id) {
    try {
      await api.swapRelationshipDirection(id);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function deleteRelationship(id) {
    try {
      await api.deleteRelationship(id);
      selectedEdgeId = null;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  function closeAllPanels() {
    selectedPersonId = null;
    selectedEdgeId = null;
    showNewPersonForm = false;
  }
</script>

<main>
  <header>
    <h1>Soziale Beziehungen</h1>
    <div class="toolbar">
      <button class:active={view === "graph"} onclick={() => (view = "graph")}>Graph</button>
      <button class:active={view === "map"} onclick={() => (view = "map")}>Karte</button>
      <button onclick={openNewPerson}>+ Person</button>
      <button class:active={connectMode} onclick={toggleConnectMode}>
        {connectMode ? (connectSource !== null ? "Zweite Person wählen…" : "Erste Person wählen…") : "Verbinden"}
      </button>
    </div>
  </header>

  {#if error}
    <div class="error" onclick={() => (error = "")}>{error} (klicken zum Schließen)</div>
  {/if}

  <div class="layout">
    <div class="canvas">
      {#if view === "graph"}
        <Graph
          {people}
          {relationships}
          {selectedPersonId}
          {connectMode}
          onNodeClick={handleNodeClick}
          onEdgeClick={handleEdgeClick}
          onCanvasClick={closeAllPanels}
          onConnectTarget={handleConnectTarget}
        />
      {:else}
        <MapView {people} onPersonClick={handleNodeClick} onGeocoded={refresh} />
      {/if}
    </div>

    {#if showNewPersonForm || selectedEdge}
      <div class="sidebar">
        {#if showNewPersonForm}
          <PersonForm onSave={savePerson} onClose={() => (showNewPersonForm = false)} />
        {:else if selectedEdge}
          <EdgeForm
            relationship={selectedEdge}
            {peopleById}
            onSave={saveRelationship}
            onDelete={deleteRelationship}
            onSwap={swapRelationship}
            onClose={() => (selectedEdgeId = null)}
          />
        {/if}
      </div>
    {/if}
  </div>

  <PersonModal
    person={selectedPerson}
    {people}
    {relationships}
    {peopleById}
    onSave={savePerson}
    onDelete={deletePerson}
    onClose={() => (selectedPersonId = null)}
    onChange={refresh}
    onSelectPerson={handleNodeClick}
    onSelectEdge={handleEdgeClick}
  />
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, "Segoe UI", Roboto, sans-serif;
    background: #f1f5f9;
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.25rem;
    background: white;
    border-bottom: 1px solid #e2e8f0;
  }
  h1 {
    font-size: 1.1rem;
    margin: 0;
    color: #1e293b;
  }
  .toolbar {
    display: flex;
    gap: 0.5rem;
  }
  .toolbar button {
    padding: 0.5rem 0.9rem;
    border-radius: 6px;
    border: 1px solid #cbd5e1;
    background: #f8fafc;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .toolbar button.active {
    background: #facc15;
    border-color: #eab308;
  }
  .layout {
    flex: 1;
    display: flex;
    gap: 1rem;
    padding: 1rem;
    overflow: hidden;
  }
  .canvas {
    flex: 1;
    min-width: 0;
  }
  .sidebar {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow-y: auto;
  }
  .error {
    background: #fee2e2;
    color: #b91c1c;
    padding: 0.5rem 1.25rem;
    font-size: 0.85rem;
    cursor: pointer;
  }
</style>
