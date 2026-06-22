<script>
  import { onMount, onDestroy } from "svelte";
  import cytoscape from "cytoscape";
  import { displayName } from "./api.js";

  // Props (Svelte 5 runes)
  let { people = [], relationships = [], onNodeClick = () => {}, onEdgeClick = () => {}, onCanvasClick = () => {}, connectMode = false, onConnectTarget = () => {} } = $props();

  let container;
  let cy;
  let pendingSource = null; // erste Person beim Verbinden-Modus

  function buildElements() {
    const nodes = people.map((p) => ({
      data: {
        id: String(p.id),
        label: displayName(p),
        color: p.color,
        shape: p.gender === "m" ? "rectangle" : p.gender === "w" ? "ellipse" : "round-rectangle",
      },
    }));
    const edges = relationships.map((r) => ({
      data: {
        id: "e" + r.id,
        source: String(r.from_id),
        target: String(r.to_id),
        kind: r.kind,
        strength: r.strength,
        relId: r.id,
      },
    }));
    return [...nodes, ...edges];
  }

  function render() {
    if (!cy) return;
    cy.elements().remove();
    cy.add(buildElements());
    cy.layout({ name: "cose", animate: true, padding: 40 }).run();
  }

  onMount(() => {
    cy = cytoscape({
      container,
      elements: buildElements(),
      style: [
        {
          selector: "node",
          style: {
            shape: "data(shape)",
            "background-color": "data(color)",
            label: "data(label)",
            color: "#1f2937",
            "font-size": 13,
            "text-valign": "bottom",
            "text-margin-y": 6,
            width: 46,
            height: 46,
            "border-width": 2,
            "border-color": "#ffffff",
          },
        },
        {
          selector: "node:selected",
          style: {
            "border-width": 4,
            "border-color": "#facc15",
          },
        },
        {
          selector: "edge",
          style: {
            width: "data(strength)",
            "line-color": "#94a3b8",
            "curve-style": "bezier",
            label: "data(kind)",
            "font-size": 10,
            color: "#64748b",
            "text-background-color": "#ffffff",
            "text-background-opacity": 0.85,
            "text-background-padding": 2,
          },
        },
        {
          selector: "edge:selected",
          style: {
            "line-color": "#facc15",
            width: 5,
          },
        },
      ],
      layout: { name: "cose", animate: true, padding: 40 },
      minZoom: 0.3,
      maxZoom: 3,
    });

    cy.on("tap", "node", (evt) => {
      const id = Number(evt.target.id());
      if (connectMode) {
        onConnectTarget(id);
      } else {
        onNodeClick(id);
      }
    });

    cy.on("tap", "edge", (evt) => {
      onEdgeClick(Number(evt.target.data("relId")));
    });

    cy.on("tap", (evt) => {
      if (evt.target === cy) onCanvasClick();
    });
  });

  onDestroy(() => {
    cy?.destroy();
  });

  // Re-render, wenn sich die Daten von außen ändern
  $effect(() => {
    render();
  });
</script>

<div class="graph-wrap" bind:this={container}></div>

<style>
  .graph-wrap {
    width: 100%;
    height: 100%;
    background: #f8fafc;
    border-radius: 8px;
  }
</style>
