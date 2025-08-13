### Vysma Cloud Architecture Plan (MVP → V1)

This document is the canonical plan for the server‑authoritative, cloud‑hosted HCL editor and live preview platform. It tracks current status, defines scope, and links to detailed specs.

Links:
- See `docs/module-registry.md` for community module registry (username::module) resolution and publishing.
- See `docs/appwrite-integration.md` for persistence, identity, and API flows.
- See `docs/http-asset-io.md` for remote asset loading via HTTP.
- See `docs/editor-ui.md` for desktop editor UI/UX.

---

### Goals
- **Single source of truth**: Server holds authoritative HCL; publishes content to clients in realtime.
- **Live preview** across iOS/Android/Web/Desktop using Lightyear replication.
- **Edit/Preview** modes: Pause gameplay in Edit; run in Preview.
- **Community modules**: Import by `username::module_name[@version]` with namespacing.
- **DX/UX**: Clean, readable code; fast build‑verify cadence; minimal abstraction; high performance.

### Out of scope (until V1)
- Multi‑tenant billing, complex ACL, and full SaaS ops (kept simple for MVP).
- Heavy in‑editor gizmos; start with a text editor + Apply.

---

### Feature Matrix

- **Workspace split; crates for HCL and Cloud** — [x] Done (crates: `vysma-hcl`, `vysma-cloud`)
- **HCL core (loader/runtime/spawn/hot‑reload/timers/actions/includes)** — [x] Done
- **Editor/Preview mode resource + toggle (F5 + GUI button)** — [x] Done
- **Trigger gating in Edit mode** — [x] Done
- **Network skeleton (HclSceneBlob replicate; client apply)** — [x] Done
- **Client file watch + local publish (Edit mode) → server** — [x] Done
- **Server apply in‑memory and republish** — [x] Done
- **Server publish prefers content (content‑first)** — [x] Done (fallback to path if unreadable)
- **Module loader + registry types** — [x] Done (not wired to spawn)
- **Module import by `username::module` via Appwrite** — [x] Done (runtime reads via `unofficial_appwrite`)
- **Module publishing CLI to Appwrite** — [x] Done (`vysma` crate: module publish + assets upload)
- **Scene persistence to Appwrite on update** — [ ] Planned (MVP+)
- **HTTP Asset IO (load glTF/images via URLs)** — [ ] Planned (MVP+)
- **Desktop editor text UI (Apply + status)** — [ ] Planned (MVP)
- **Editor auth (JWT) to gate updates** — [ ] Planned (MVP)
- **Rollback to previous version** — [ ] Planned (Post‑MVP)
- **Flattened single‑doc publish (includes/modules)** — [ ] Planned (Post‑MVP)

Legend: [x] implemented in code; [ ] not yet.

---

### High‑Level Architecture
- **Authoritative Server**: Bevy app with `HclPlugin` and Lightyear server plugins.
  - Receives editor updates, parses to `HclSceneAsset`, sets `HclEntry`, republishes `HclSceneBlob { content, sha }`.
  - (MVP+) Persists latest HCL to Appwrite (SceneVersions) and reloads on start.
  - Resolves module imports via Appwrite registry when `modules = [...]` present (MVP).
- **Editor Client (Desktop)**: Bevy client with GUI.
  - Edit mode pauses triggers; shows HCL text panel and Apply button.
  - Sends `HclUpdateRequest` (with JWT) to server; live preview on all clients.
- **Viewer Clients (Mobile/Desktop/Web)**: Bevy clients; always Preview; no editing.
- **Appwrite**: Identity + DB + Storage for projects/scenes/modules/assets.

---

### Phased Plan (each step should build green)

Current status: Phase 1 completed (remote module resolve wired). Phase 2: CLI implemented with assets upload. Next: cross‑machine import test; then Phase 3.

Phase 2.1: CLI DX foundation (MVP)
- Add `vysma new <name>`: scaffold a game repo with example HCL and `assets/` layout.
- Add `vysma serve`: run the server app with hot‑reload and log overlay; watch `assets/`.
- Add `vysma client`: run a local client connected to `serve`.
- Acceptance:
  - Single command creates and runs a project locally; editing HCL hot‑reloads.

Phase 2.2: Content‑addressed assets + manifest (MVP)
- Hashing: sha256 per file; IDs are sha prefix (≤32 chars); URL path `owner/name/<sha>.ext`.
- Upload: idempotent (409 skip); parallel uploads with retries.
- Manifest: embed manifest array into ModuleVersion; optional `ModuleAssetsIndex` rows.
- Resolver: runtime maps `file` fields to URL paths via manifest; local paths remain for dev.
- Acceptance: remote client loads module assets via CDN URLs with HTTP asset IO enabled.

Phase 3: Editor UI + Auth (MVP)
- Desktop GUI: multiline text area, Apply, mode toggle, status (sha/error).
- Add JWT on `HclUpdateRequest`; server verifies JWT with Appwrite and project membership.
- Acceptance: Unauthorized updates ignored; authorized updates apply and propagate.

Phase 4: Scene Persistence (MVP+)
- On accepted update, save `SceneVersion` (HCL, sha, author) in Appwrite; update `Scenes.publishedVersionId`.
- Server startup loads latest published scene into `HclSource` and republishes.
- Acceptance: Restart preserves the current scene.

Phase 5: HTTP Asset IO (MVP+)
- Implement `HttpAssetIo` for http(s) URLs; integrate via feature flag `http_assets`.
- Add in‑memory cache; later, add disk cache under `~/.vysma/cache/`.
- Acceptance: Assets load over HTTP across native/wasm.

---

### Detailed implementation tasks (DX‑centric)

CLI (`vysma`)
- new:
  - Create directories: `assets/scenes`, `assets/mesh`, `assets/textures`.
  - Add example `scenes/moba_game.hcl` and README.
  - Optionally init git and write `.gitignore` (target/, .DS_Store).
- serve:
  - Build and run server binary with watch (`cargo watch` optional) and hot‑reload enabled.
  - Print URLs and keybindings; surface parse errors live.
- client:
  - Run client with connection to local server; pass flags for renderer/GUI.
- publish:
  - Validate env, read HCL, compute sha.
  - Upload deduped assets (parallel), build manifest, create module/version.
  - Print import snippet and resolved assets table.

Runtime
- Keep manifest alongside imported module; inject into `ApplyCtx` for asset resolution.
- Fallback: if manifest missing, use `file` directly.

Docs
- Update HCL spec to emphasize name‑based references and relative paths.
- Add quickstart using `vysma new` + `vysma serve` + editing HCL.
- Document `http_assets` feature flag usage.

---

### Acceptance Criteria by Feature
- Remote module resolve: imports with `alias` namespace all prefabs/entities/triggers/assets; conflicts avoided via prefixing.
- Publishing CLI: Publishing a module version updates latest pointer when specified; assets accessible via URLs.
- Editor UI: In Edit mode, triggers paused; Apply causes live respawn on server and all clients.
- Auth: Invalid JWT or unauthorized user → server logs and ignores; no state changes.
- Persistence: Latest HCL survives server restart.
- HTTP Asset IO: `asset_server.load("https://.../file.glb#Scene0")` works; images load into materials.

---

### Configuration (env/CLI)
Server:
- `APPWRITE_ENDPOINT`, `APPWRITE_PROJECT_ID`, `APPWRITE_API_KEY`
- `VYSMA_REGISTRY_ENABLED=true` (toggle remote module resolve)
- `VYSMA_PROJECT_ID`, `VYSMA_SCENE_ID` (for persistence)

Client (Editor):
- `APPWRITE_PUBLIC_ENDPOINT` (for login flows if used)

---

### Risks and Mitigations
- Remote asset latency → HTTP Asset IO caching/CDN.
- Auth complexity → start with server‑side API key and JWT verification; expand later.
- Module conflicts → require aliasing and enforce namespace prefix.

---

### Testing Strategy
- Unit: parse HCL strings with `parse_hcl_to_asset`; invalid HCL returns error.
- Integration: import module over Appwrite on a clean client; assert prefabs/entities present.
- E2E: editor → server update applies and clients respawn. 