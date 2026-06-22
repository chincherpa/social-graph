<script>
  import { uploadPersonImage, deletePersonImage, placeholderFor } from "./api.js";
  import ImageCropTool from "./ImageCropTool.svelte";

  let { person = null, onSave = () => {}, onDelete = () => {}, onClose = () => {} } = $props();

  let firstName = $state(person?.first_name ?? "");
  let lastName = $state(person?.last_name ?? "");
  let nickname = $state(person?.nickname ?? "");
  let birthDate = $state(person?.birth_date ?? "");
  let knownSince = $state(person?.known_since ?? "");
  let address = $state(person?.address ?? "");
  let employer = $state(person?.employer ?? "");
  let note = $state(person?.note ?? "");
  let color = $state(person?.color ?? "#3b82f6");
  let gender = $state(person?.gender ?? null);

  let pendingFile = $state(null);
  let imageError = $state("");
  let fileInput;

  function imageSrc() {
    if (pendingFile) return null; // crop tool is showing instead
    if (person?.image_data) return person.image_data;
    return placeholderFor(person?.gender);
  }

  function onFileChosen(e) {
    const file = e.target.files?.[0];
    e.target.value = ""; // allow re-selecting the same file later
    if (!file) return;

    if (file.size > 5 * 1024 * 1024) {
      imageError = "Datei ist größer als 5 MB";
      return;
    }
    if (!["image/jpeg", "image/png"].includes(file.type)) {
      imageError = "Nur JPG und PNG werden unterstützt";
      return;
    }

    imageError = "";
    pendingFile = file;
  }

  async function onCropConfirm({ x, y, radius }) {
    const bytes = new Uint8Array(await pendingFile.arrayBuffer());
    try {
      const updated = await uploadPersonImage(person.id, bytes, x, y, radius);
      person = updated;
      onSave; // no-op reference kept; actual persistence already happened server-side
    } catch (err) {
      imageError = String(err);
    } finally {
      pendingFile = null;
    }
  }

  function onCropCancel() {
    pendingFile = null;
  }

  async function onDeleteImage() {
    const updated = await deletePersonImage(person.id);
    person = updated;
  }

  // Felder neu befüllen, wenn eine andere Person ausgewählt wird
  $effect(() => {
    firstName = person?.first_name ?? "";
    lastName = person?.last_name ?? "";
    nickname = person?.nickname ?? "";
    birthDate = person?.birth_date ?? "";
    knownSince = person?.known_since ?? "";
    address = person?.address ?? "";
    employer = person?.employer ?? "";
    note = person?.note ?? "";
    color = person?.color ?? "#3b82f6";
    gender = person?.gender ?? null;
  });

  const colors = ["#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#a855f7", "#ec4899", "#14b8a6"];

  let lastNameError = $state(false);

  function submit() {
    if (!lastName.trim()) {
      lastNameError = true;
      return;
    }
    lastNameError = false;
    onSave({
      id: person?.id,
      firstName: firstName.trim() || null,
      lastName: lastName.trim(),
      nickname: nickname.trim() || null,
      birthDate: birthDate || null,
      knownSince: knownSince || null,
      address: address.trim() || null,
      employer: employer.trim() || null,
      note: note.trim() || null,
      color,
      gender,
    });
  }
</script>

<div class="panel">
  <h3>{person ? "Person bearbeiten" : "Neue Person"}</h3>

  {#if person}
    <div class="image-section">
      <img src={imageSrc()} alt={lastName} class="avatar" />
      <div class="image-actions">
        <input
          type="file"
          accept="image/jpeg,image/png"
          bind:this={fileInput}
          onchange={onFileChosen}
          style="display:none"
        />
        <button type="button" onclick={() => fileInput.click()}>Foto hochladen</button>
        {#if person.image_data}
          <button type="button" class="danger" onclick={onDeleteImage}>Foto entfernen</button>
        {/if}
      </div>
      {#if imageError}
        <span class="error-text">{imageError}</span>
      {/if}
    </div>
    {#if pendingFile}
      <ImageCropTool imageFile={pendingFile} oncropconfirm={onCropConfirm} oncropcancel={onCropCancel} />
    {/if}
  {/if}

  <label>
    Vorname
    <input type="text" bind:value={firstName} placeholder="z.B. Hannah" />
  </label>

  <label>
    Nachname *
    <input
      type="text"
      bind:value={lastName}
      placeholder="z.B. Müller"
      class:error={lastNameError}
      oninput={() => (lastNameError = false)}
    />
    {#if lastNameError}
      <span class="error-text">Nachname ist erforderlich</span>
    {/if}
  </label>

  <label>
    Spitzname
    <input type="text" bind:value={nickname} placeholder="optional" />
  </label>

  <label>
    Geschlecht
    <div class="gender-toggle">
      <button
        type="button"
        class:active={gender === "m"}
        onclick={() => (gender = gender === "m" ? null : "m")}
      >
        ◻ männlich
      </button>
      <button
        type="button"
        class:active={gender === "w"}
        onclick={() => (gender = gender === "w" ? null : "w")}
      >
        ◯ weiblich
      </button>
    </div>
  </label>

  <label>
    Geburtsdatum
    <input type="date" bind:value={birthDate} />
  </label>

  <label>
    Kennengelernt am
    <input type="date" bind:value={knownSince} />
  </label>

  <label>
    Adresse
    <input type="text" bind:value={address} placeholder="optional" />
  </label>

  <label>
    Arbeitgeber
    <input type="text" bind:value={employer} placeholder="optional" />
  </label>

  <label>
    Notiz
    <textarea bind:value={note} placeholder="optional" rows="3"></textarea>
  </label>

  <div class="color-row">
    {#each colors as c}
      <button
        type="button"
        class="swatch"
        class:active={color === c}
        style="background:{c}"
        onclick={() => (color = c)}
        aria-label="Farbe wählen"
      ></button>
    {/each}
  </div>

  <div class="actions">
    <button class="primary" onclick={submit}>Speichern</button>
    {#if person}
      <button class="danger" onclick={() => onDelete(person.id)}>Löschen</button>
    {/if}
    <button onclick={onClose}>Abbrechen</button>
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
  input,
  textarea {
    padding: 0.5rem;
    border: 1px solid #cbd5e1;
    border-radius: 6px;
    font-size: 0.9rem;
    font-family: inherit;
  }
  input.error {
    border-color: #ef4444;
  }
  .error-text {
    color: #ef4444;
    font-size: 0.75rem;
  }
  .image-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }
  .avatar {
    width: 96px;
    height: 96px;
    border-radius: 50%;
    object-fit: cover;
    border: 2px solid #cbd5e1;
  }
  .image-actions {
    display: flex;
    gap: 0.5rem;
  }
  .gender-toggle {
    display: flex;
    gap: 0.4rem;
  }
  .gender-toggle button {
    flex: 1;
    padding: 0.4rem;
    border-radius: 6px;
    border: 1px solid #cbd5e1;
    background: #f8fafc;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .gender-toggle button.active {
    background: #3b82f6;
    color: white;
    border-color: #3b82f6;
  }
  .color-row {
    display: flex;
    gap: 0.4rem;
  }
  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
  }
  .swatch.active {
    border-color: #1f2937;
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
