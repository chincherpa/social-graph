<script>
  import { onMount, onDestroy } from "svelte";
  import L from "leaflet";
  import "leaflet/dist/leaflet.css";
  import * as api from "./api.js";
  import { displayName, placeholderFor } from "./api.js";

  let { people, onPersonClick, onGeocoded } = $props();

  let mapEl;
  let map;
  let markersLayer;
  let geocoding = $state(false);
  let geocodeStatus = $state("");

  const peopleWithCoords = $derived(people.filter((p) => p.lat != null && p.lon != null));
  const peopleMissingCoords = $derived(
    people.filter((p) => p.address && p.address.trim() && (p.lat == null || p.lon == null))
  );
  const peopleWithoutAddress = $derived(people.filter((p) => !p.address || !p.address.trim()));

  onMount(() => {
    map = L.map(mapEl).setView([20, 0], 2);
    L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution: "&copy; OpenStreetMap-Mitwirkende",
      maxZoom: 18,
    }).addTo(map);
    markersLayer = L.layerGroup().addTo(map);
  });

  onDestroy(() => {
    map?.remove();
  });

  function drawMarkers() {
    if (!markersLayer) return;
    markersLayer.clearLayers();
    for (const p of peopleWithCoords) {
      const src = p.image_data || placeholderFor(p.gender);
      const icon = L.divIcon({
        className: "avatar-marker-wrap",
        html: `<div class="avatar-marker" style="border-color:${p.color}"><img src="${src}" /></div>`,
        iconSize: [40, 40],
        iconAnchor: [20, 20],
      });
      const marker = L.marker([p.lat, p.lon], { icon });
      marker.bindTooltip(displayName(p));
      marker.on("click", () => onPersonClick?.(p.id));
      marker.addTo(markersLayer);
    }
  }

  $effect(() => {
    drawMarkers();
  });

  // Nominatim erlaubt max. 1 Request/Sekunde -> sequentiell mit Pause.
  async function geocodeMissing() {
    geocoding = true;
    const targets = [...peopleMissingCoords];
    for (let i = 0; i < targets.length; i++) {
      geocodeStatus = `Geocodiere ${i + 1}/${targets.length}: ${displayName(targets[i])}…`;
      try {
        await api.geocodePerson(targets[i].id);
      } catch (e) {
        console.error(`Geocoding fehlgeschlagen für ${displayName(targets[i])}:`, e);
      }
      if (i < targets.length - 1) {
        await new Promise((r) => setTimeout(r, 1100));
      }
    }
    geocoding = false;
    geocodeStatus = "";
    await onGeocoded?.();
  }
</script>

<div class="map-wrap">
  <div class="status-bar">
    <span>{peopleWithCoords.length} auf der Karte</span>
    {#if peopleMissingCoords.length > 0}
      <span class="pending">{peopleMissingCoords.length} ungeocodiert</span>
      <button onclick={geocodeMissing} disabled={geocoding}>
        {geocoding ? geocodeStatus : "Adressen geocodieren"}
      </button>
    {/if}
    {#if peopleWithoutAddress.length > 0}
      <span class="muted">{peopleWithoutAddress.length} ohne Adresse</span>
    {/if}
  </div>
  <div class="map" bind:this={mapEl}></div>
</div>

<style>
  .map-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }
  .status-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 0.75rem;
    background: white;
    border-bottom: 1px solid #e2e8f0;
    font-size: 0.8rem;
    color: #475569;
  }
  .status-bar button {
    padding: 0.3rem 0.7rem;
    border-radius: 6px;
    border: 1px solid #cbd5e1;
    background: #f8fafc;
    cursor: pointer;
    font-size: 0.8rem;
  }
  .status-bar button:disabled {
    cursor: default;
    opacity: 0.7;
  }
  .pending {
    color: #b45309;
  }
  .muted {
    color: #94a3b8;
  }
  .map {
    flex: 1;
    min-height: 0;
  }

  :global(.avatar-marker) {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 3px solid #3b82f6;
    background: white;
    overflow: hidden;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
  }
  :global(.avatar-marker img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
