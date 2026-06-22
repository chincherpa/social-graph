# Profile Picture Feature Design

**Date:** 2026-06-22  
**Feature:** Upload, crop, and display user profile pictures in person detail page and graph nodes

---

## Overview

Add profile picture support to the social graph app. Users can upload JPG/PNG photos to person records, define a circular crop region, and see the images displayed both in the detail panel and as small avatars in the graph visualization. Fallback to gender-specific placeholders (female.png/male.png) when no image exists.

---

## Data Model

### Database Schema Changes

Add three columns to the `people` table:

```sql
ALTER TABLE people ADD COLUMN image_path TEXT;
ALTER TABLE people ADD COLUMN image_crop_x INTEGER;
ALTER TABLE people ADD COLUMN image_crop_y INTEGER;
ALTER TABLE people ADD COLUMN image_crop_radius INTEGER;
```

- `image_path`: Relative or absolute path to stored image file (e.g., `images/person_123.jpg`). NULL if no image.
- `image_crop_x`, `image_crop_y`: Pixel coordinates of crop circle center within original image
- `image_crop_radius`: Radius of crop circle in pixels

All crop fields are NULL if no image uploaded.

### Backend Struct

Update `Person` struct in `models.rs`:

```rust
pub struct Person {
    pub id: i64,
    pub name: String,
    pub note: Option<String>,
    pub color: String,
    pub gender: String,  // Already exists
    pub image_path: Option<String>,
    pub image_crop_x: Option<i32>,
    pub image_crop_y: Option<i32>,
    pub image_crop_radius: Option<i32>,
    pub created_at: String,
}
```

---

## File Storage

### Location

Store images in the app's local data directory:  
`{app_data_dir}/social-graph/images/`

Example paths:
- Linux/macOS: `~/.config/social-graph/images/`
- Windows: `%APPDATA%\social-graph\images\`

### Filename Convention

`{person_id}.{ext}` where `ext` is `jpg` or `png`

Example: `person_42.jpg`

### Cleanup

When a person is deleted (CASCADE from DB), delete the corresponding image file from disk.

---

## Upload Workflow

### User Interaction (Frontend)

1. User opens person detail page
2. Clicks "Upload Photo" button (or similar)
3. File picker opens, filtered to `.jpg`, `.png` only
4. User selects file

### Validation (Frontend)

- File size must not exceed 5 MB
- File must be JPG or PNG
- Show error toast if validation fails

### Preview & Crop (Frontend)

1. Image preview displays in a modal/panel
2. Fixed-size circular overlay appears on top of image
3. User can drag the circle to reposition it (center point moveable)
4. Circle radius is fixed (not resizable by user)
5. On confirm, capture crop center (x, y) and radius in pixels

### Upload & Save (Backend)

Backend command: `upload_person_image`

**Input:**
- `person_id: i64`
- `file_bytes: Vec<u8>`
- `filename: String` (e.g., "image.jpg")
- `crop_x: i32`, `crop_y: i32`, `crop_radius: i32`

**Process:**
1. Validate file size (< 5 MB)
2. Validate MIME type (JPG or PNG only)
3. Save bytes to `{app_data_dir}/social-graph/images/{person_id}.{ext}`
4. If old image exists for this person, delete it
5. Update `people` table with image metadata
6. Return success or error

**Error Cases:**
- File too large → "File exceeds 5 MB limit"
- Invalid format → "Only JPG and PNG supported"
- Disk write failure → "Failed to save image"

---

## Display

### Detail Page

In `PersonForm.svelte`:
- Show profile image area (square or circular container)
- If `image_path` exists: load and display with circular CSS mask using crop coords
- Otherwise: show placeholder (`images/female.png` or `images/male.png` based on gender field)
- Include "Delete Photo" button to remove image (revert to placeholder)

### Graph Node Avatar

In `Graph.svelte` (Cytoscape):
- Person nodes currently show as circles with color
- Add small avatar image (if exists) as node background image, sized to fit node
- Apply same circular mask using crop coords
- Placeholder if no image
- Avatar size: scales with node size in graph (responsive)

### Crop Application

Both frontend and backend should apply crop the same way:
- Center circle at `(crop_x, crop_y)`
- Use `crop_radius` to define visible region
- CSS: `border-radius: 50%` on a masked container
- Display size may differ (large in detail panel, small in graph), but crop region stays proportional

---

## Image Deletion

User can remove a person's image:
- Button in person detail panel: "Delete Photo"
- Backend command: `delete_person_image(person_id)`
- Deletes file from disk
- Sets `image_path`, `crop_*` to NULL in DB
- Person reverts to placeholder on next display

Automatic deletion:
- When person is deleted, cascade delete image file

---

## Backend Implementation

### New Commands

#### `upload_person_image`
```rust
#[tauri::command]
async fn upload_person_image(
    person_id: i64,
    file_bytes: Vec<u8>,
    filename: String,
    crop_x: i32,
    crop_y: i32,
    crop_radius: i32,
    db: State<'_, DbState>,
) -> Result<(), String>
```

#### `delete_person_image`
```rust
#[tauri::command]
async fn delete_person_image(
    person_id: i64,
    db: State<'_, DbState>,
) -> Result<(), String>
```

### Validation & File Handling

- Validate MIME type (magic bytes for JPG/PNG, not just extension)
- Check file size before saving
- Use `std::fs` to save/delete, handle errors gracefully
- Path: derive from `tauri::api::path::app_data_dir()` and person ID

---

## Frontend Implementation

### Components

**Image Upload Input** (in `PersonForm.svelte`):
- Upload button triggering file picker
- On file select:
  1. Read file as data URL
  2. Display preview + crop tool

**Crop Tool** (new component or inline):
- Display image in preview
- Draw draggable circle overlay
- On drag, update circle center coords
- On confirm, extract crop params and call backend

**Image Display**:
- Use existing person detail layout
- Add image area (if exists) with circular CSS mask
- Fallback to placeholder

---

## Testing Checklist

### Backend
- Upload valid JPG → saved correctly, DB updated
- Upload valid PNG → saved correctly, DB updated
- Upload oversized file (> 5 MB) → rejected with error
- Upload invalid format (BMP, GIF, etc.) → rejected with error
- Crop params stored correctly in DB
- Delete image → file removed, DB cleared
- Delete person → cascade deletes image file
- Get person → returns image metadata

### Frontend
- File picker filters to JPG/PNG only
- Preview displays uploaded image correctly
- Crop circle drags smoothly
- On upload, circle appears in detail page with correct positioning
- Crop applied correctly (circular region visible)
- Graph node shows small avatar with crop
- Switch to person without image → placeholder shows
- Delete photo → reverts to placeholder

---

## Out of Scope

- Image resizing (assume user uploads reasonable dimensions)
- Animated crop tool (simple drag only)
- Multiple images per person
- Crop adjustment after upload (one-time only)
- Cloud storage or sync

---

## Database Migration

Create `src-tauri/migrations/0002_add_profile_images.sql`:

```sql
-- Add profile image support to people table
ALTER TABLE people ADD COLUMN image_path TEXT;
ALTER TABLE people ADD COLUMN image_crop_x INTEGER;
ALTER TABLE people ADD COLUMN image_crop_y INTEGER;
ALTER TABLE people ADD COLUMN image_crop_radius INTEGER;
```

Migration runs automatically via `sqlx::migrate!` on app startup.
