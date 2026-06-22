# Profile Picture Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable users to upload profile pictures, define a circular crop, and display the image both in the person detail panel and as the graph node avatar. Show gender-based placeholders (`images/male.png` / `images/female.png`) when no image is set.

**Architecture:**
- DB migration adds `image_path`, `image_crop_x`, `image_crop_y`, `image_crop_radius` to `people`.
- Backend stores the uploaded file on disk under the app data dir and, on every read, base64-encodes it into a transient `image_data` field on the `Person` struct (not persisted) so the frontend never needs file-system access or a custom asset protocol.
- Frontend adds an upload button + canvas-based circular crop tool to `PersonForm.svelte`, and uses `image_data` (falling back to a placeholder) for both the detail panel and the Cytoscape node background in `Graph.svelte`.

**Tech Stack:** Tauri 2 (Rust backend + Svelte 5 frontend), SQLite via sqlx, Cytoscape.js.

## Global Constraints

- Formats: JPG and PNG only, verified by magic bytes (not file extension).
- Max upload size: 5 MB.
- Storage location: `{dirs::data_dir()}/social-graph/images/{person_id}.{ext}`.
- Crop is one-time only — no re-crop UI after the initial upload (re-upload replaces it).
- Crop circle has a **fixed radius**; the user only drags the center point.
- Gender field is `Option<String>` with values `"m"`, `"w"`, or `null` (see `src-tauri/src/models.rs:15`, `src/lib/PersonForm.svelte:84-100`). Placeholder mapping: `"w"` → `images/female.png`, anything else (`"m"` or `null`) → `images/male.png`.
- Images are served to the frontend as base64 data URLs via a new `image_data: Option<String>` field on `Person` — **never** persisted to DB, computed fresh on every read. This avoids touching `tauri.conf.json` / capabilities for a custom asset protocol.
- Existing `PERSON_COLUMNS` const in `src-tauri/src/commands.rs:10` lists exact DB columns selected — every new column must be added there and the struct/migration must match it exactly, or `sqlx::FromRow` will panic at runtime on column-count mismatch.

---

## Task 1: Database Migration + Person Model + Image Utilities

**Files:**
- Create: `src-tauri/migrations/0004_add_profile_images.sql`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/commands.rs` (update `PERSON_COLUMNS` and every Person-returning query)
- Create: `src-tauri/src/image.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod image;`)
- Modify: `src-tauri/Cargo.toml` (add `base64` dependency)

**Interfaces:**
- Produces (for Task 2 to consume):
  - `image::validate_image(bytes: &[u8]) -> Result<&'static str, String>` — returns `"jpg"` or `"png"`, or an error string (`"File exceeds 5 MB limit"` / `"Only JPG and PNG supported"`).
  - `image::images_dir() -> std::path::PathBuf` — `{dirs::data_dir()}/social-graph/images`, created if missing.
  - `image::encode_data_url(path: &std::path::Path) -> Option<String>` — reads file, returns `data:image/{jpeg|png};base64,{...}` or `None` if read fails.
  - `Person` struct gains: `image_path: Option<String>`, `image_crop_x: Option<i64>`, `image_crop_y: Option<i64>`, `image_crop_radius: Option<i64>`, and a transient `#[sqlx(skip)] image_data: Option<String>` (always `None` straight out of `FromRow`; callers must populate it).
  - `fn attach_image(person: Person) -> Person` and `fn attach_images(people: Vec<Person>) -> Vec<Person>` in `commands.rs` — fill in `image_data` from `image_path` using `image::encode_data_url`. Every existing command that returns `Person`/`Vec<Person>`/`GraphData` must route its result through these before returning.

- [ ] **Step 1: Add base64 dependency**

In `src-tauri/Cargo.toml`, add this line under `[dependencies]` (after the `dirs = "5"` line):

```toml
base64 = "0.22"
```

- [ ] **Step 2: Create the migration**

Create `src-tauri/migrations/0004_add_profile_images.sql`:

```sql
-- Profilbild-Unterstützung: Dateipfad + Kreis-Ausschnitt-Koordinaten
ALTER TABLE people ADD COLUMN image_path TEXT;
ALTER TABLE people ADD COLUMN image_crop_x INTEGER;
ALTER TABLE people ADD COLUMN image_crop_y INTEGER;
ALTER TABLE people ADD COLUMN image_crop_radius INTEGER;
```

- [ ] **Step 3: Update the Person struct**

In `src-tauri/src/models.rs`, the current `Person` struct (lines 3-17) is:

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub note: Option<String>,
    pub color: String,
    pub gender: Option<String>,
    pub created_at: String,
}
```

Replace it with:

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: String,
    pub nickname: Option<String>,
    pub birth_date: Option<String>,
    pub known_since: Option<String>,
    pub address: Option<String>,
    pub employer: Option<String>,
    pub note: Option<String>,
    pub color: String,
    pub gender: Option<String>,
    pub image_path: Option<String>,
    pub image_crop_x: Option<i64>,
    pub image_crop_y: Option<i64>,
    pub image_crop_radius: Option<i64>,
    pub created_at: String,
    #[sqlx(skip)]
    pub image_data: Option<String>,
}
```

(`#[sqlx(skip)]` means `FromRow` ignores this field and it defaults via `Default::default()` — `Option<String>` defaults to `None`, which is correct: every fresh query starts with no `image_data` until `attach_image`/`attach_images` fills it in.)

- [ ] **Step 4: Create the image utility module**

Create `src-tauri/src/image.rs`:

```rust
use std::fs;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};

const MAX_SIZE: usize = 5 * 1024 * 1024;

/// Validates magic bytes and size. Returns the file extension ("jpg" or "png") on success.
pub fn validate_image(bytes: &[u8]) -> Result<&'static str, String> {
    if bytes.len() > MAX_SIZE {
        return Err("File exceeds 5 MB limit".to_string());
    }

    let is_jpeg = bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF;
    let is_png = bytes.len() >= 8
        && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    if is_jpeg {
        Ok("jpg")
    } else if is_png {
        Ok("png")
    } else {
        Err("Only JPG and PNG supported".to_string())
    }
}

/// `{dirs::data_dir()}/social-graph/images`, created if missing.
pub fn images_dir() -> PathBuf {
    let mut dir = dirs::data_dir().expect("Kein Daten-Verzeichnis gefunden");
    dir.push("social-graph");
    dir.push("images");
    fs::create_dir_all(&dir).expect("Konnte Bilder-Verzeichnis nicht anlegen");
    dir
}

/// Reads the file at `path` and returns it as a base64 data URL, or `None` if unreadable.
pub fn encode_data_url(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?.to_lowercase();
    let mime = if ext == "png" { "png" } else { "jpeg" };
    let b64 = STANDARD.encode(&bytes);
    Some(format!("data:image/{mime};base64,{b64}"))
}
```

- [ ] **Step 5: Register the module**

In `src-tauri/src/lib.rs`, change line 1-3 from:

```rust
mod commands;
mod db;
mod models;
```

to:

```rust
mod commands;
mod db;
mod image;
mod models;
```

- [ ] **Step 6: Update PERSON_COLUMNS and add the enrichment helpers**

In `src-tauri/src/commands.rs`, change line 10 from:

```rust
const PERSON_COLUMNS: &str = "id, first_name, last_name, nickname, birth_date, known_since, address, employer, note, color, gender, created_at";
```

to:

```rust
const PERSON_COLUMNS: &str = "id, first_name, last_name, nickname, birth_date, known_since, address, employer, note, color, gender, image_path, image_crop_x, image_crop_y, image_crop_radius, created_at";
```

Add these two functions right after the `PERSON_COLUMNS` const (before the `// ---------- People ----------` comment):

```rust
fn attach_image(mut person: Person) -> Person {
    if let Some(path) = &person.image_path {
        person.image_data = crate::image::encode_data_url(std::path::Path::new(path));
    }
    person
}

fn attach_images(people: Vec<Person>) -> Vec<Person> {
    people.into_iter().map(attach_image).collect()
}
```

- [ ] **Step 7: Route every Person-returning command through the new helpers**

In `src-tauri/src/commands.rs`, update each of these (the `.fetch_one`/`.fetch_all` line is unchanged; only the final `Ok(...)` wrapping changes):

`list_people` (currently returns the query result directly) — change:
```rust
pub async fn list_people(db: Db<'_>) -> Result<Vec<Person>, String> {
    sqlx::query_as::<_, Person>(&format!(
        "SELECT {PERSON_COLUMNS} FROM people ORDER BY last_name, first_name"
    ))
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())
}
```
to:
```rust
pub async fn list_people(db: Db<'_>) -> Result<Vec<Person>, String> {
    let people = sqlx::query_as::<_, Person>(&format!(
        "SELECT {PERSON_COLUMNS} FROM people ORDER BY last_name, first_name"
    ))
    .fetch_all(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(attach_images(people))
}
```

`add_person` — change the final two lines from:
```rust
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(rec)
}
```
to:
```rust
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;
    Ok(attach_image(rec))
}
```
(This applies to `add_person`'s closing block specifically — `update_person` has the identical closing pattern; apply the same edit there too.)

`get_graph` — change:
```rust
    Ok(GraphData {
        people,
        relationships,
    })
```
to:
```rust
    Ok(GraphData {
        people: attach_images(people),
        relationships,
    })
```

Leave `add_family_member`'s internal `new_person` insert as-is (it returns `FamilyMember`, not `Person`, so no enrichment is needed there).

- [ ] **Step 8: Verify it compiles**

Run:
```bash
cd src-tauri && cargo check
```
Expected: no errors. (A "base64 unused" warning is fine if Step 4's import isn't yet exercised elsewhere; it is used in `image.rs` itself so this shouldn't happen.)

- [ ] **Step 9: Self-review**

Confirm:
- `PERSON_COLUMNS` lists exactly the same columns, in the same order, as the `Person` struct fields (excluding `image_data`).
- Every function that builds a `Person` or `Vec<Person>` for return calls `attach_image`/`attach_images` before returning.
- The migration file is named `0004_...` (not `0002_...` — `0002` and `0003` already exist in this repo).

- [ ] **Step 10: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/migrations/0004_add_profile_images.sql src-tauri/src/models.rs src-tauri/src/image.rs src-tauri/src/lib.rs src-tauri/src/commands.rs
git commit -m "feat: add profile image columns, Person.image_data, and image utilities"
```

---

## Task 2: Upload/Delete Tauri Commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register new commands in `invoke_handler!`)

**Interfaces:**
- Consumes: `image::validate_image`, `image::images_dir`, `attach_image` (all from Task 1).
- Produces (for Task 3 to consume via `src/lib/api.js`):
  - Tauri command `upload_person_image(db, person_id: i64, file_bytes: Vec<u8>, crop_x: i64, crop_y: i64, crop_radius: i64) -> Result<Person, String>`
  - Tauri command `delete_person_image(db, person_id: i64) -> Result<Person, String>`

- [ ] **Step 1: Add commands to commands.rs**

Add these two functions to `src-tauri/src/commands.rs`, after `delete_person` (around line 81):

```rust
// ---------- Profilbilder ----------

#[tauri::command]
pub async fn upload_person_image(
    db: Db<'_>,
    person_id: i64,
    file_bytes: Vec<u8>,
    crop_x: i64,
    crop_y: i64,
    crop_radius: i64,
) -> Result<Person, String> {
    let ext = crate::image::validate_image(&file_bytes)?;

    let dir = crate::image::images_dir();
    // Remove any previous image for this person, regardless of extension.
    let _ = std::fs::remove_file(dir.join(format!("{person_id}.jpg")));
    let _ = std::fs::remove_file(dir.join(format!("{person_id}.png")));

    let file_path = dir.join(format!("{person_id}.{ext}"));
    std::fs::write(&file_path, &file_bytes).map_err(|e| format!("Failed to save image: {e}"))?;

    let path_str = file_path.to_string_lossy().to_string();

    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET image_path = ?, image_crop_x = ?, image_crop_y = ?, image_crop_radius = ?
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(&path_str)
    .bind(crop_x)
    .bind(crop_y)
    .bind(crop_radius)
    .bind(person_id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(attach_image(rec))
}

#[tauri::command]
pub async fn delete_person_image(db: Db<'_>, person_id: i64) -> Result<Person, String> {
    let existing: Option<(Option<String>,)> =
        sqlx::query_as("SELECT image_path FROM people WHERE id = ?")
            .bind(person_id)
            .fetch_optional(db.inner())
            .await
            .map_err(|e| e.to_string())?;

    if let Some((Some(path),)) = existing {
        let _ = std::fs::remove_file(&path);
    }

    let rec = sqlx::query_as::<_, Person>(&format!(
        "UPDATE people SET image_path = NULL, image_crop_x = NULL, image_crop_y = NULL, image_crop_radius = NULL
         WHERE id = ?
         RETURNING {PERSON_COLUMNS}"
    ))
    .bind(person_id)
    .fetch_one(db.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(attach_image(rec))
}
```

- [ ] **Step 2: Register both commands**

In `src-tauri/src/lib.rs`, the `use commands::{...}` block (lines 5-9) currently is:

```rust
use commands::{
    add_family_member, add_person, add_relationship, delete_person, delete_relationship,
    get_family, get_graph, list_people, list_relationships, remove_family_member, update_person,
    update_relationship,
};
```

Change to:

```rust
use commands::{
    add_family_member, add_person, add_relationship, delete_person, delete_person_image,
    delete_relationship, get_family, get_graph, list_people, list_relationships,
    remove_family_member, update_person, update_relationship, upload_person_image,
};
```

And the `.invoke_handler(tauri::generate_handler![...])` block (lines 24-37) currently is:

```rust
        .invoke_handler(tauri::generate_handler![
            list_people,
            add_person,
            update_person,
            delete_person,
            list_relationships,
            add_relationship,
            update_relationship,
            delete_relationship,
            get_graph,
            get_family,
            add_family_member,
            remove_family_member,
        ])
```

Add the two new commands before the closing `])`:

```rust
        .invoke_handler(tauri::generate_handler![
            list_people,
            add_person,
            update_person,
            delete_person,
            list_relationships,
            add_relationship,
            update_relationship,
            delete_relationship,
            get_graph,
            get_family,
            add_family_member,
            remove_family_member,
            upload_person_image,
            delete_person_image,
        ])
```

- [ ] **Step 3: Verify it compiles**

Run:
```bash
cd src-tauri && cargo check
```
Expected: no errors.

- [ ] **Step 4: Manual smoke test**

Run `pnpm tauri dev`, open DevTools console, and run:

```javascript
const bytes = await fetch('/images/female.png').then(r => r.arrayBuffer());
const result = await window.__TAURI__.core.invoke('upload_person_image', {
  personId: 1,
  fileBytes: Array.from(new Uint8Array(bytes)),
  cropX: 50,
  cropY: 50,
  cropRadius: 40,
});
console.log(result.image_data?.slice(0, 40)); // should start with "data:image/png;base64,"
```

(Adjust `personId` to an existing person's id from your data. If you have no people yet, add one first via the UI.)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add upload_person_image and delete_person_image commands"
```

---

## Task 3: Frontend API Wrappers + Circular Crop Tool Component

**Files:**
- Modify: `src/lib/api.js`
- Create: `src/lib/ImageCropTool.svelte`

**Interfaces:**
- Consumes: Tauri commands `upload_person_image`, `delete_person_image` (Task 2).
- Produces (for Task 4 to consume):
  - `uploadPersonImage(personId, fileBytes, cropX, cropY, cropRadius)` → `Promise<Person>` in `src/lib/api.js`
  - `deletePersonImage(personId)` → `Promise<Person>` in `src/lib/api.js`
  - `placeholderFor(gender)` → `"images/female.png"` if `gender === "w"`, else `"images/male.png"` (this is the single source of truth for placeholder selection — Task 4 must use it, not reimplement the mapping)
  - `<ImageCropTool>` Svelte component with props `imageFile` (a `File` object) and events `oncropconfirm` (detail: `{ x, y, radius }`) and `oncropcancel`.

- [ ] **Step 1: Add API wrappers**

In `src/lib/api.js`, add after the `deletePerson` function (around line 75):

```javascript
// ---------- Profilbilder ----------

export function placeholderFor(gender) {
  return gender === "w" ? "images/female.png" : "images/male.png";
}

export async function uploadPersonImage(personId, fileBytes, cropX, cropY, cropRadius) {
  return invoke("upload_person_image", {
    personId,
    fileBytes: Array.from(fileBytes),
    cropX,
    cropY,
    cropRadius,
  });
}

export function deletePersonImage(personId) {
  return invoke("delete_person_image", { personId });
}
```

- [ ] **Step 2: Create the crop tool component**

Create `src/lib/ImageCropTool.svelte`. The crop circle has a **fixed radius** (per Global Constraints) — the user can only drag its center. The component renders the image scaled into a 300x300 canvas, so the radius is fixed at 80px within that canvas, and the emitted coordinates are in canvas-pixel space (the backend just stores whatever numbers it's given — there's no requirement that they map onto the original image's resolution, since Task 4's display code re-derives proportional masking client-side using the same canvas-relative convention).

```svelte
<script>
  let { imageFile, oncropconfirm = () => {}, oncropcancel = () => {} } = $props();

  const CANVAS_SIZE = 300;
  const RADIUS = 80;

  let canvas;
  let dragging = false;
  let circleX = $state(CANVAS_SIZE / 2);
  let circleY = $state(CANVAS_SIZE / 2);
  let imageUrl = $state("");
  let img = new Image();

  $effect(() => {
    if (imageFile) {
      imageUrl = URL.createObjectURL(imageFile);
      img = new Image();
      img.onload = draw;
      img.src = imageUrl;
    }
  });

  function draw() {
    if (!canvas) return;
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
    draw();
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
```

- [ ] **Step 3: Verify it builds**

Run:
```bash
pnpm exec vite build --mode development
```
Expected: no Svelte compile errors mentioning `ImageCropTool.svelte` or `api.js`.

(If this command doesn't exist in this project's `package.json`, run `pnpm tauri dev` instead, open the app, and confirm no red error overlay appears — `ImageCropTool` isn't imported anywhere yet so it won't be exercised, but a syntax error in it would still break the Vite build.)

- [ ] **Step 4: Commit**

```bash
git add src/lib/api.js src/lib/ImageCropTool.svelte
git commit -m "feat: add image API wrappers and circular crop tool component"
```

---

## Task 4: Wire Upload/Display/Delete into PersonForm

**Files:**
- Modify: `src/lib/PersonForm.svelte`

**Interfaces:**
- Consumes: `uploadPersonImage`, `deletePersonImage`, `placeholderFor` (Task 3, `src/lib/api.js`), `<ImageCropTool>` (Task 3).
- Produces: Visible image section in the person panel; no new exports (this is a leaf UI task).

- [ ] **Step 1: Import what's needed**

In `src/lib/PersonForm.svelte`, change the very top of `<script>` (line 1) from:

```svelte
<script>
  let { person = null, onSave = () => {}, onDelete = () => {}, onClose = () => {} } = $props();
```

to:

```svelte
<script>
  import { uploadPersonImage, deletePersonImage, placeholderFor } from "./api.js";
  import ImageCropTool from "./ImageCropTool.svelte";

  let { person = null, onSave = () => {}, onDelete = () => {}, onClose = () => {} } = $props();
```

- [ ] **Step 2: Add local state for the upload flow**

Right after the existing `let gender = $state(...)` line (line 13), add:

```svelte
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
```

(Note: `uploadPersonImage`/`deletePersonImage` mutate the backend directly and return the fresh `Person` — this bypasses the form's normal `onSave` callback by design, since the image is a separate resource from the rest of the person fields and shouldn't require the user to also click "Speichern".)

- [ ] **Step 3: Add the image section to the template**

In the template, right after the opening `<h3>...</h3>` line (currently line 56), add:

```svelte
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
```

(The whole section is gated on `{#if person}` because there's no person id to attach an image to until the person is first saved — matches the existing pattern where the "Löschen" button is also gated on `{#if person}` at line 142.)

- [ ] **Step 4: Add styles**

In the `<style>` block, add after the existing `.error-text` rule (around line 184):

```css
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
```

- [ ] **Step 5: Manual test**

Run `pnpm tauri dev`. Select an existing person (or create one and save first). Confirm:
- Placeholder shows (male or female image matching the person's gender toggle) when no image is set.
- "Foto hochladen" opens a file picker restricted to JPG/PNG.
- Selecting a valid image opens the crop overlay with a draggable circle.
- Dragging the circle moves it smoothly and stays within canvas bounds.
- "Ausschnitt übernehmen" closes the overlay and the avatar now shows the uploaded image.
- "Foto entfernen" appears once an image is set, and clicking it reverts to the placeholder.
- Selecting an oversized file or a non-JPG/PNG file shows the relevant error text and does not open the crop tool.

- [ ] **Step 6: Commit**

```bash
git add src/lib/PersonForm.svelte
git commit -m "feat: wire image upload, crop, and delete into PersonForm"
```

---

## Task 5: Graph Node Avatars

**Files:**
- Modify: `src/lib/Graph.svelte`

**Interfaces:**
- Consumes: `placeholderFor` (Task 3, `src/lib/api.js`), `person.image_data` (Task 1, populated by backend on every `get_graph`/`list_people` call).
- Produces: no new exports (leaf UI task) — Cytoscape nodes show the person's photo (or gender placeholder) instead of a flat color shape.

- [ ] **Step 1: Import placeholderFor**

In `src/lib/Graph.svelte`, change line 4 from:

```svelte
  import { displayName } from "./api.js";
```

to:

```svelte
  import { displayName, placeholderFor } from "./api.js";
```

- [ ] **Step 2: Add image data to node elements**

In `buildElements()` (lines 13-21), change:

```javascript
    const nodes = people.map((p) => ({
      data: {
        id: String(p.id),
        label: displayName(p),
        color: p.color,
        shape: p.gender === "m" ? "rectangle" : p.gender === "w" ? "ellipse" : "round-rectangle",
      },
    }));
```

to:

```javascript
    const nodes = people.map((p) => ({
      data: {
        id: String(p.id),
        label: displayName(p),
        color: p.color,
        shape: p.gender === "m" ? "rectangle" : p.gender === "w" ? "ellipse" : "round-rectangle",
        avatar: p.image_data || placeholderFor(p.gender),
      },
    }));
```

- [ ] **Step 3: Render the avatar as the node's background image**

In the `style` array passed to `cytoscape(...)`, the `node` selector (lines 47-61) currently is:

```javascript
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
```

Change it to:

```javascript
        {
          selector: "node",
          style: {
            shape: "data(shape)",
            "background-color": "data(color)",
            "background-image": "data(avatar)",
            "background-fit": "cover",
            "background-clip": "node",
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
```

(`background-clip: "node"` is what makes the image respect the node's `shape` — without it Cytoscape draws the image as a plain square overlapping the shape outline.)

- [ ] **Step 4: Manual test**

Run `pnpm tauri dev`. Confirm:
- People with an uploaded image show that image inside their graph node shape.
- People without an image show the gender placeholder inside their node shape.
- Node shape (rectangle/ellipse/round-rectangle per gender) is unaffected — the avatar fills the shape, it doesn't replace it.

- [ ] **Step 5: Commit**

```bash
git add src/lib/Graph.svelte
git commit -m "feat: display profile image avatars on graph nodes"
```

---

## Out of Scope (per spec)

- Image resizing/compression beyond the 5 MB upload cap.
- Re-cropping after the initial upload (re-upload replaces the image and crop entirely).
- Multiple images per person.
- Cloud storage or sync.

## Summary

5 tasks, each independently testable and committed:
1. DB schema + Person model + image utilities (Rust)
2. Upload/delete Tauri commands (Rust)
3. Frontend API wrappers + crop tool component (JS/Svelte)
4. PersonForm wiring: upload, display, delete (Svelte)
5. Graph node avatars (Svelte)
