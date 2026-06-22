# Soziale Beziehungen — AI Agent Guidelines

**Desktop app** (Tauri 2 + Svelte 5 + Rust + SQLite) for visualizing social relationships as an interactive graph using Cytoscape.js.

## Quick Start

**Prerequisites**: Rust (via `cargo`), Node.js/pnpm, Tauri system dependencies ([see v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/))

```bash
pnpm install
pnpm tauri dev      # Dev server on :1420 + Tauri window
pnpm tauri build    # Release binary
```

Database auto-creates at `%APPDATA%/social-graph/social-graph.db` on first run via `sqlx::migrate!()`.

See [README.md](README.md) for full setup and user guide.

---

## Architecture

### Frontend-Backend Communication (Tauri IPC)

- **Pattern**: Svelte 5 components call `invoke()` from [src/lib/api.js](src/lib/api.js) → Rust `#[tauri::command]` handlers in [src-tauri/src/commands.rs](src-tauri/src/commands.rs)
- **Naming convention**: Frontend uses `camelCase` (e.g., `addPerson`); API layer converts to `snake_case` for Rust
- **Payload format**: `invoke("add_person", { payload: { name, note, color } })`
- **Error handling**: Rust `Result<T, String>` → JS try-catch; errors are hardcoded in German
- **Pattern**: Full graph refreshed via `getGraph()` after each mutation (no incremental updates)

**Example**: Adding a person
```
src/App.svelte → api.addPerson({ name, note, color })
→ src/lib/api.js (camelCase → snake_case)
→ invoke("add_person", ...)
→ src-tauri/src/commands.rs (add_person handler)
→ src-tauri/src/db.rs (SQL insert)
→ Rust Result<Person> → JS response
→ Frontend calls getGraph() to refresh
```

### State Management (Svelte 5 Runes)

- **Location**: Centralized in [src/App.svelte](src/App.svelte) using `$state` and `$derived`
- **Key state**: `people`, `relationships`, `selectedPersonId`, `selectedEdgeId`, `connectMode`, `connectSource`
- **Reactivity**: Svelte 5 runes auto-trigger component updates (no external store subscriptions needed)

### Database

**Schema** ([src-tauri/migrations/0001_init.sql](src-tauri/migrations/0001_init.sql)):
```
people: id, name, note, color, created_at
relationships: id, from_id, to_id, kind, strength, note, created_at
  UNIQUE(from_id, to_id)
  CHECK(from_id < to_id)
  FOREIGN KEY constraints with CASCADE
```

**Critical design**: Relationships are **undirected** and **normalized**.
- When saving: `from_id < to_id` is enforced via `normalize_pair()` in [commands.rs](src-tauri/src/commands.rs)
- **Why**: Prevents duplicate edges A↔B and B↔A; ensures single row per relationship pair
- **Gotcha**: Always normalize before querying or inserting; frontend errors pass (B, A) when (A, B) exists

### Visualization (Cytoscape.js)

- **Component**: [src/lib/Graph.svelte](src/lib/Graph.svelte) renders nodes (colored circles) and edges (width = relationship strength)
- **Layout**: Force-directed ("cose") with animation on refresh
- **Events**: Node/edge/canvas taps dispatch callbacks to [src/App.svelte](src/App.svelte) for selection and connection flow
- **Gotcha**: Cytoscape layout re-runs on every `getGraph()` refresh; consider layout caching for >5k relationships

---

## Key Patterns & Conventions

| Aspect | Pattern | Note |
|--------|---------|------|
| **Naming** | Rust: snake_case; Svelte: camelCase | [src/lib/api.js](src/lib/api.js) bridges the gap |
| **Errors** | German error messages in Rust | Hardcoded; plan for i18n if needed |
| **Forms** | Save = commit immediately; no drafts | Closing form = lose unsaved edits (by design) |
| **Connection mode** | Two-step: select first node → select second node → add edge | See [src/App.svelte](src/App.svelte) `connectMode` logic |
| **Relationship normalization** | `from_id < to_id` always | [commands.rs](src-tauri/src/commands.rs) `normalize_pair()` enforces this |

---

## Common Gotchas

1. **sqlx::migrate! macro fails** → Ensure [src-tauri/migrations/](src-tauri/migrations/) exists with .sql files. May need `cargo sqlx database create`.
2. **Duplicate edges** → Frontend must normalize (A, B) to ensure from_id < to_id before calling backend; UNIQUE constraint will reject violators.
3. **Port :1420 conflict** → Check if another process uses :1420 or set `VITE_PORT` env var.
4. **Cytoscape layout stutter** → On >5k relationships, layout animation may lag. Consider `animate: false` or incremental updates.
5. **Tauri prerequisites missing** → Windows: Visual C++ build tools required. macOS/Linux: see [v2.tauri.app](https://v2.tauri.app/start/prerequisites/).

---

## Files & Responsibilities

| File | Purpose |
|------|---------|
| [src/App.svelte](src/App.svelte) | Root component, state management, event orchestration |
| [src/lib/api.js](src/lib/api.js) | JS-to-Rust IPC wrapper, camelCase ↔ snake_case conversion |
| [src/lib/Graph.svelte](src/lib/Graph.svelte) | Cytoscape visualization, event handling |
| [src/lib/PersonForm.svelte](src/lib/PersonForm.svelte) | Person CRUD UI panel |
| [src/lib/EdgeForm.svelte](src/lib/EdgeForm.svelte) | Relationship CRUD UI panel |
| [src-tauri/src/commands.rs](src-tauri/src/commands.rs) | Tauri RPC handlers (CRUD logic) |
| [src-tauri/src/db.rs](src-tauri/src/db.rs) | Database pool init + query helpers |
| [src-tauri/src/models.rs](src-tauri/src/models.rs) | `Person`, `Relationship`, `GraphData` struct defs |
| [src-tauri/migrations/](src-tauri/migrations/) | SQL schema migrations (sqlx-managed) |

---

## Development Tips for Agents

- **Safe to refactor**: Frontend state is centralized in [src/App.svelte](src/App.svelte) — can extract components or stores safely
- **API layer is flexible**: [src/lib/api.js](src/lib/api.js) can cache, validate, or queue calls without affecting Rust backend
- **Database constraints are reliable**: Schema enforces `UNIQUE`, `CHECK`, and `FOREIGN KEY` — trust the schema, don't duplicate validation in app logic
- **No tests yet**: Good opportunity to add Rust integration tests (for db.rs) and E2E tests (for Graph.svelte workflows)
- **Scaling consideration**: Full `getGraph()` after each mutation is N+1 safe but may slow with 10k+ relationships; consider cursor-based incremental queries

---

## Debugging Checklist

- ❌ Tauri window won't open? → Check Tauri prerequisites and system dependencies
- ❌ `sqlx::migrate!` macro error? → Verify [src-tauri/migrations/](src-tauri/migrations/) exists; try `cargo sqlx database create`
- ❌ Duplicate relationships appearing? → Check that frontend normalizes (from_id, to_id) before calling backend
- ❌ Graph layout jittery? → Consider disabling Cytoscape animation or implementing incremental layout
- ❌ Graph not updating after CRUD? → Ensure backend returns updated state; frontend calls `getGraph()` after mutations

---

## Next Steps / Feature Ideas

See [README.md](README.md) "Mögliche nächste Schritte" for roadmap (search, cluster detection, export, tags/groups).
