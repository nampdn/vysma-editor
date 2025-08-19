### Vysma Cloud Architecture Plan (MVP → V1)

This document is the canonical plan for the server‑authoritative, cloud‑hosted HCL editor and live preview platform. It tracks current status, defines scope, and links to detailed specs.

Links:
- See `docs/module-registry.md` for community module registry (username::module) resolution and publishing.
- See `docs/appwrite-integration.md` for persistence, identity, and API flows.
- See `docs/http-asset-io.md` for remote asset loading via HTTP.
- See `docs/editor-ui.md` for desktop editor UI/UX.
- See `docs/relay-discovery.md` for LAN discovery and Internet relay (Expo‑style URLs).

---

### Goals
- **Single source of truth**: Server holds authoritative HCL; publishes content to clients in realtime.
- **Live preview** across iOS/Android/Web/Desktop using Lightyear replication.
- **Edit/Preview** modes: Pause gameplay in Edit; run in Preview.
- **Community modules**: Import by `username::module_name[@version]` with namespacing.
- **DX/UX**: Clean, readable code; fast build‑verify cadence; minimal abstraction; high performance.
- **Expo‑style workflow**: A multi‑platform "Vysma Client" app connects to a project URL (LAN or Internet) and mirrors the latest server HCL instantly.
- **WASM browser preview**: One‑click open in browser; same HCL via WebSocket transport; HTTP Asset IO enabled with CORS.
- **Dev ↔ SaaS alignment**: CLI can auth with Appwrite Dev account, operate locally, and also target the SaaS server seamlessly.

---

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
- **WASM browser preview (WebSocket transport + CORS)** — [ ] Planned (MVP)
- **Relay/discovery for LAN + Internet (rendezvous + NAT traversal/tunnel)** — [ ] Planned (MVP)
- **Expo‑style multi‑platform client UX** — [ ] Planned (MVP)
- **Remix & plugin metadata (license, deps, semver)** — [ ] Planned (MVP+)

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
- **Relay/Discovery**: Rendezvous service for project URLs, optional NAT traversal or HTTPS/WebSocket tunnel for remote access.

---

### Networking and Discovery (LAN + Internet)
- **Discovery**
  - LAN: mDNS broadcast `vysma.local` with project ID and port.
  - Internet: short project URL `vysma.dev/<project-code>` resolving to relay.
- **Transport**
  - Native: QUIC for LAN; WebSocket fallback for firewall traversal.
  - WASM: WebSocket only.
- **Relay**
  - Minimal WebSocket relay service that forwards Lightyear frames between server and clients.
  - Token‑gated: server registers `project_code`, clients connect with a short‑lived token from CLI or server.
- **Security**
  - Editor updates require JWT verified by server against Appwrite JWKS.
  - Viewer clients read‑only; receive `HclSceneBlob` updates.

---

### WASM Browser Preview
- Build target: `wasm32-unknown-unknown` (Bevy).
- Network: WebSocket transport to local server or relay.
- Assets: `http_assets` feature enabled; CORS allowed from `localhost` and SaaS origins; see `docs/http-asset-io.md`.
- Hosting: simple static page served by CLI (`vysma preview --open`).

---

### DX User Journeys (CLI‑first)
- **Login**
  - `vysma login --endpoint <url> --project <id>` → device/session token stored under `~/.vysma/config.toml`.
- **Develop locally, share on LAN/Internet**
  - `vysma serve` starts local server with hot‑reload; announces via mDNS and optionally registers with relay to get a URL.
  - `vysma client --connect <url|lan>` launches a viewer client (desktop/mobile/wasm) subscribing to updates.
- **Publish module**
  - `vysma module publish --owner <u> --name <n> --version <v> --hcl <file> --assets <dir> [--set-latest]` uploads HCL + assets manifest to Appwrite.
- **Persist scene (MVP+)**
  - On Apply, server persists SceneVersion and marks as published; on restart, it auto‑loads the latest.
- **Provision DB (Planned)**
  - `vysma db provision` reads HCL `models { ... }` and creates/validates Appwrite collections and indexes.

### Local‑first development mode (Priority)
- Rationale: minimize server storage/egress during development; keep ultra‑low latency by serving HCL and assets from the developer’s machine. “Publish” uploads only when ready.
- Source of truth
  - Local files for HCL and assets (project on disk). Editor syncs UI ↔ HCL via an in‑memory doc graph and writes back to local files.
  - Browser: File System Access API (Chromium) for folder access; fallback desktop wrapper (Tauri) for full portability.
- Asset I/O for preview
  - Option A (same‑device): BrowserAssetIo reads bytes from File System Access/OPFS for the embedded preview.
  - Option B (LAN, recommended): a local HTTP asset server (CLI) serves `assets/` at `http://<dev-host>:<port>` with:
    - CORS: `Access-Control-Allow-Origin: *`
    - CORP: `Cross-Origin-Resource-Policy: cross-origin` (required for COEP on the editor origin)
    - ETag/immutable caching; range requests; optional directory index.
  - mDNS advertises `{ host, port }` for LAN clients.
- Cross‑origin isolation and headers
  - Editor origin sets `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` to enable WASM threads/OffscreenCanvas.
  - Remote assets must be CORP‑compatible; local asset server adds CORP header above.
- HCL apply and sync
  - Editor computes sha and sends `HclUpdateRequest { content, sha, jwt? }` to the local server.
  - Server parses/merges (no asset uploads), republishes `HclSceneBlob` to all connected clients.
  - Preview (same tab or other devices on LAN) fetches assets from the local asset server.
- Multi‑device preview on LAN
  - Clients connect to the local server for HCL and to the dev’s asset server for resources; discovery via mDNS.
  - Relay not required in local mode; can be enabled later for Internet testing.
- Publish workflow (when sharing beyond LAN)
  - One‑click “Publish”: hash and upload deduped assets to Appwrite; build manifest; create/advance ModuleVersion.
  - Runtime maps HCL `file` paths to CDN URLs via manifest; HCL content remains unchanged.
- Security & ergonomics
  - Asset server binds to localhost by default; `--public`/`--lan` binds `0.0.0.0` with random token (query/header) and rate limits.
  - Editor warns when COOP/COEP/CORP conditions aren’t met.
- Acceptance criteria
  - Edit → Apply → Play on same device without uploads.
  - Second device on the same LAN previews with low latency (assets streamed from dev host).
  - Publish swaps to CDN URLs via manifest; remote clients work without the dev asset server.

---

### Phased Plan (each step should build green)

Current status: Phase 1 completed (remote module resolve wired). Phase 2: CLI implemented with assets upload. Next: cross‑machine import test; then Phase 3.

Phase 2.1: CLI DX foundation (MVP)
- Add `vysma new <name>`: scaffold a game repo with example HCL and `assets/` layout.
- Add `vysma serve`: run the server app with hot‑reload and log overlay; watch `assets/`.
- Add `vysma client`: run a local client connected to the dev server.
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

Phase 6: Relay/Discovery + Expo‑style Client (MVP)
- Add mDNS discovery; implement minimal WebSocket relay; define project URL codes.
- Package a "Vysma Client" app for mobile/desktop/web that connects via URL and hot‑reloads.
- Acceptance: Multiple devices connect over LAN/Internet and see changes in realtime.

---

### Detailed implementation tasks (DX‑centric)

CLI (`vysma`)
- new:
  - Create directories: `assets/scenes`, `assets/mesh`, `assets/textures`.
  - Add example `scenes/moba_game.hcl` and README.
  - Optionally init git and write `.gitignore` (target/, .DS_Store).
- login:
  - Device/session auth flow with Appwrite; persist token locally; support `--endpoint` and `--project-id`.
- serve:
  - Build and run server binary with watch; optionally register with relay and print project URL.
  - Print URLs and keybindings; surface parse errors live.
- client:
  - Run client with connection to local server or relay URL; pass flags for renderer/GUI.
- publish:
  - Validate env, read HCL, compute sha.
  - Upload deduped assets (parallel), build manifest, create module/version.
  - Print import snippet and resolved assets table.

Runtime
- Keep manifest alongside imported module; inject into `ApplyCtx` for asset resolution.
- Fallback: if manifest missing, use `file` directly.
- WebSocket transport support for wasm; QUIC/WS for native.

Docs
- Update HCL spec to emphasize name‑based references and relative paths.
- Add quickstart using `vysma new` + `vysma serve` + editing HCL.
- Document `http_assets` and `wasm` feature usage, relay URL, and CORS.

---

### Acceptance Criteria by Feature
- Remote module resolve: imports with `alias` namespace all prefabs/entities/triggers/assets; conflicts avoided via prefixing.
- Publishing CLI: Publishing a module version updates latest pointer when specified; assets accessible via URLs.
- Editor UI: In Edit mode, triggers paused; Apply causes live respawn on server and all clients.
- Auth: Invalid JWT or unauthorized user → server logs and ignores; no state changes.
- Persistence: Latest HCL survives server restart.
- HTTP Asset IO: `asset_server.load("https://.../file.glb#Scene0")` works; images load into materials.
- WASM Preview: `vysma preview --open` loads the scene via WebSocket and renders in browser.
- Relay/Discovery: Multiple devices connect via LAN URL or short project URL and receive updates.

---

### Configuration (env/CLI)
Server:
- `APPWRITE_ENDPOINT`, `APPWRITE_PROJECT_ID`, `APPWRITE_API_KEY`
- `VYSMA_REGISTRY_ENABLED=true` (toggle remote module resolve)
- `VYSMA_PROJECT_ID`, `VYSMA_SCENE_ID` (for persistence)
- `VYSMA_RELAY_URL` (optional; enable remote access)

Client (Editor/Viewer):
- `APPWRITE_PUBLIC_ENDPOINT` (for login flows if used)
- `VYSMA_CONNECT` (ws(s):// host or relay short URL)

---

### Risks and Mitigations
- Remote asset latency → HTTP Asset IO caching/CDN.
- Auth complexity → start with server‑side API key and JWT verification; expand later.
- Module conflicts → require aliasing and enforce namespace prefix.
- NAT traversal → relay fallback over HTTPS/WebSocket; QUIC when available.

---

### Testing Strategy
- Unit: parse HCL strings with `parse_hcl_to_asset`; invalid HCL returns error.
- Integration: import module over Appwrite on a clean client; assert prefabs/entities present.
- E2E: editor → server update applies and clients respawn.
- WASM: headless CI runs wasm bindgen build, starts a test WebSocket server, and verifies first frame renders.

---

### Engine Execution Plan (ECS/DX/Perf)

Phased tasks to align the engine with ECS best practices, high DX, and performance. Each phase must build green and include minimal tests.

P1: Compile-time and runtime performance
- Precompile expressions: parse at HCL load into ASTs; evaluate only on event fire.
- Selector indices: maintain `Name→Entity` and `Tag→Vec<Entity>` resources; avoid world scans.
- Deterministic RNG resource with seeding; add builtins (`rng()`, `rng_range(a,b)`).
- Typed component appliers: avoid walking `serde_json::Value` in hot loops; decode to typed payloads once.

P2: State scopes (global/tag/entity)
- Implement `VarScopes` resource and optional `HclVars` component for entity-local state.
- Extend var actions with `scope` and selector‑based targeting; back‑compat for global.
- Document `var(scope,name)` for expressions and add examples.
- Implement change‑driven binding engine: in/out/inout bindings, on=change, epsilon, throttle, priority, conflict resolution history.

P3: Networking annotations and authority
- Trigger metadata: `authority="server|client"`, `channel="reliable|unreliable"` (docs first, then code).
- Server‑authoritative world changes; client‑only UI triggers; echo suppression where needed.
- Snapshot + diff model for `HclSceneBlob` updates; sha‑versioned.

P4: Trigger systemization and diagnostics
- Register static triggers (startup, tick, timers) as Bevy systems with `run_if` guards.
- Event dispatcher for key/custom events; minimal allocations.
- Editor panels: trigger list with last‑fired timestamps and guard status; line/column diagnostics for parse/compile errors.

P5: Assets, WASM, and caching
- Asset preload hints (`preload`, `priority`) and LOD guidance.
- WASM Service Worker cache for manifest assets; CORS guidance baked into docs.
- HTTP Asset IO disk cache (optional, behind feature).

P6: Persistence & Backend Compute
- HCL `models {}` and `persist_bind` parsing; expose as resources.
- Server systems to implement `persist_load/save/set/query` actions via Appwrite.
- CLI `vysma db provision`; add readme guidance in projects.
- Auth: enforce server-only execution for persist actions.
- Examples: load on startup, save on checkpoint, leaderboard query → UI vars.

Acceptance per phase
- Bench: reduced per‑frame allocations and world scans; event‑only evaluation of expressions.
- Correctness: same authored HCL produces identical results pre/post changes.
- UX: clearer errors, visible trigger status, smoother multi‑device preview.
- Persistence: numeric fields round-trip; queries populate vars correctly on server authority. 

### MMO Support Addendum

Targeted engine/platform capabilities for large online worlds. These complement existing plans and are feature-gated; each item can be delivered incrementally.

World topology
- Shards: horizontal copies of a world for population control.
- Zones: spatial partitions (grid/quadtree) with handoffs; entity migration across zones.
- Instances: ephemeral copies for dungeons/raids; lifecycle managed by matchmaking.

Interest management (AOI)
- Spatial index per zone (grid/quadtree); entity subscribes to AOI radius.
- Relevancy tiers: critical (reliable, high rate), nearby (delta, medium), distant (LOD/heartbeat), out-of-range (pause updates).
- Priority budget: per-tick bandwidth quotas by priority; drop/merge updates when over budget.

Networking and tick
- Server tick broadcast and client time sync; configurable tick rate per zone.
- Snapshot + delta compression; per-entity baselines; bit-packing for common components.
- Channels/QoS: reliable for state, unreliable for frequent positions; backpressure metrics.

Persistence and data integrity
- Event-sourced ledger for economy/inventory (append-only), with periodic snapshots.
- Idempotency keys on write actions; server-side validation/sanity checks.
- Transactions for multi-entity writes (atomic when supported) with fallback compensation.

Social systems
- Accounts/sessions, presence, friends, parties, guilds; text chat channels and whispers.
- Matchmaking/lobbies → instances assignment; party stickiness across migrations.
- Moderation: mute/kick/ban hooks and audit logs.

Security and anti-cheat
- Server authority on physics/combat; deterministic validation of client-reported inputs.
- Rate limits per identity/IP; cooldowns on costly actions; anomaly detection hooks.
- Signed action tokens with short TTL; region/gateway verification.

Ops and observability
- Metrics (tick time, send/recv bytes, queue depths), tracing, structured logs.
- Live ops: feature flags, announcements, safe maintenance modes, targeted drain/migrate.
- Replay logs for root-cause and arbitration; privacy redaction pipeline.

Scalability
- Stateless gateways (protocol termination) + zone servers behind a coordinator.
- Redis/pubsub (or NATS/Kafka) for cross-zone events (chat, matchmaking, presence).
- Container orchestration (Kubernetes) for auto-scaling instances/zones.

HCL hooks (see hcl-spec)
- New events: presence.join/leave, chat.message, party.match_found, instance.created, guild.invite.
- New actions: send_chat, match_queue, instance_create/transfer, teleport, guild_invite/accept.
- Conditions: in_aoi, population_lt, has_permission.

Acceptance
- Zone handoff tests; AOI correctness under load; stable tick; bounded bandwidth; persistence round-trips; basic social flows. 

### Authoritative Flow & Input Pipeline

End‑to‑end loop for fair, responsive play.

1) Capture (client)
- Sample inputs at frame time; pack as `{ seq, t_client, axes/buttons }`
- Optionally set local predicted vars for visuals (e.g., dash blend) without committing gameplay state

2) Send (client → server)
- Unreliable channel for high‑rate inputs; include periodic reliable heartbeats
- Resend window for packet loss; small client queue for late arrival reconciliation

3) Process (server)
- Stamp `t_server`; map inputs to HCL events; execute triggers with `authority="server"`
- Apply actions: vars/components, FSM, sequences, persistence
- Produce world delta: component changes and `HclSceneBlob` updates

4) Replicate (server → clients)
- Reliable channel for authoritative state; unreliable for frequent transforms where appropriate
- Include snapshot baselines as needed; compress/pack common components

5) Render & correct (client)
- Interpolate/extrapolate transforms; apply authoritative corrections when deltas arrive
- Reconcile predicted vars to server values; smooth via easing/signals to avoid pops

Authoring rules
- Mark gameplay triggers `authority="server"`; restrict client triggers to UI/VFX/SFX
- Use `channel` to hint transport; engine defaults pick safe QoS when omitted
- Prefer event‑driven HCL; avoid per‑frame polling on client 