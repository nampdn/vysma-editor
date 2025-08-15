### Desktop Editor UI Spec (MVP)

Goal: Minimal, usable editor panel for authoring HCL and applying updates live.

---

### Features
- Mode display and toggle (Edit/Preview)
- Multiline text editor bound to current HCL
- Buttons:
  - Apply: compute sha, send `HclUpdateRequest` (+ JWT when enabled)
  - Revert buffer to last applied
- Status line: last sha, parse result on server (via log), timestamp

### Flows
- Startup
  - Load from latest `HclSceneBlob.content` into editor buffer; if absent, try current `HclEntry` path.
- Edit mode
  - Triggers paused; Apply enabled.
- Preview mode
  - Triggers active; Apply disabled (or warn).

### Implementation
- Feature gated: `gui && client`
- Components/resources
  - `EditorBuffer(pub String)`
  - `LastApplied(pub String)`
- Systems
  - Sync buffer from incoming `HclSceneBlob` (first time only unless user edited).
  - UI layout: simple column with mode text, editor box, buttons, status.
  - Apply handler: spawn `HclUpdateRequest` to server.

### Checklist
- [ ] Buffer sync from live content
- [ ] Apply button sends update
- [ ] Mode toggle integrated with global `EditorState`
- [ ] Status line shows sha/time
- [ ] Disable Apply in Preview mode (optional)

---

### Near‑term roadmap (Unity‑like UX)
- Project view:
  - Asset browser (images, models), drag‑drop into scene placeholders
  - Import pipeline: copy to `assets/`, compute sha, update HCL assets block
- Hierarchy/Inspector:
  - Entity tree view, select entity to edit components via form widgets
  - Prefab create/edit flow; apply overrides
- Scene tools (minimal):
  - Transform gizmo opt‑in behind feature flag (translate/rotate/scale)
  - Snap/grid options (later)
- HCL sync:
  - Bi‑directional: GUI edits write to HCL; HCL text edits reflect in GUI (source of truth remains HCL)
- DX details:
  - Undo/redo buffer; dirty state indicator; autosave toggle
  - Errors surfaced inline with jump‑to location

### Security and roles
- Apply requires editor JWT when server enforces auth.
- Viewers (no JWT) can connect in Preview mode but cannot Apply. 