<script>
  let { imageFile, oncropconfirm = () => {}, oncropcancel = () => {} } = $props();

  const CANVAS_SIZE = 300;
  const RADIUS = 80;

  let canvas;
  let dragging = false;
  let circleX = $state(CANVAS_SIZE / 2);
  let circleY = $state(CANVAS_SIZE / 2);
  let img = null;

  // Lädt das Bild neu, wenn sich imageFile ändert (unabhängig vom Redraw-Effect unten,
  // sonst entsteht ein Update-Loop zwischen Bildladen und Zeichnen).
  $effect(() => {
    if (!imageFile) return;
    const url = URL.createObjectURL(imageFile);
    const loaded = new Image();
    loaded.onload = () => {
      img = loaded;
      draw();
    };
    loaded.src = url;
    return () => URL.revokeObjectURL(url);
  });

  // Neu zeichnen, wenn sich die Kreisposition ändert.
  $effect(() => {
    circleX;
    circleY;
    draw();
  });

  function draw() {
    if (!canvas || !img) return;
    const ctx = canvas.getContext("2d");
    ctx.clearRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);
    ctx.drawImage(img, 0, 0, CANVAS_SIZE, CANVAS_SIZE);

    ctx.fillStyle = "rgba(0,0,0,0.5)";
    ctx.beginPath();
    ctx.rect(0, 0, CANVAS_SIZE, CANVAS_SIZE);
    ctx.arc(circleX, circleY, RADIUS, 0, Math.PI * 2);
    ctx.fill("evenodd");

    ctx.strokeStyle = "rgba(255,255,255,0.9)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(circleX, circleY, RADIUS, 0, Math.PI * 2);
    ctx.stroke();
  }

  function clamp(v) {
    return Math.max(RADIUS, Math.min(v, CANVAS_SIZE - RADIUS));
  }

  function pointerPos(e) {
    const rect = canvas.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  function onPointerDown(e) {
    const { x, y } = pointerPos(e);
    if (Math.hypot(x - circleX, y - circleY) < RADIUS + 20) {
      dragging = true;
    }
  }

  function onPointerMove(e) {
    if (!dragging) return;
    const { x, y } = pointerPos(e);
    circleX = clamp(x);
    circleY = clamp(y);
  }

  function onPointerUp() {
    dragging = false;
  }

  function confirm() {
    oncropconfirm({ x: Math.round(circleX), y: Math.round(circleY), radius: RADIUS });
  }
</script>

<div class="crop-overlay">
  <div class="crop-panel">
    <canvas
      bind:this={canvas}
      width={CANVAS_SIZE}
      height={CANVAS_SIZE}
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      onpointerleave={onPointerUp}
    ></canvas>
    <div class="crop-actions">
      <button onclick={oncropcancel}>Abbrechen</button>
      <button class="primary" onclick={confirm}>Ausschnitt übernehmen</button>
    </div>
  </div>
</div>

<style>
  .crop-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .crop-panel {
    background: white;
    padding: 1rem;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  canvas {
    cursor: grab;
    border-radius: 6px;
    touch-action: none;
  }
  .crop-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
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
</style>
