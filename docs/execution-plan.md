### Execution Plan (AI-Friendly) — From MVP to Shareable Editor-as-Game

This plan turns the architecture into small, verifiable steps an AI or developer can execute safely. Each step lists the goal, concrete tasks, acceptance, commands, and files/crates to touch. Every step must build green and keep code clean and self-describing.

Links: see `docs/architecture-plan.md`, `docs/appwrite-integration.md`, `docs/http-asset-io.md`, `docs/editor-ui.md`, `docs/module-registry.md`, `docs/relay-discovery.md`.

---

## Ground rules
- Ship in small steps; each step ends with: `cargo build --workspace` and a runnable demo.
- Prefer progressive enhancement: blocks → EZ → canonical HCL share the same runtime.
- Defaults safe: localhost only; `--lan/--public` is opt-in with token + clear banner.
- Keep acceptance visible: print success lines and endpoints in CLI.

---

## 0) Prep and sanity
- Goal: Ensure workspace builds cleanly and key just/CLI commands exist as stubs.
- Tasks
  - Add `just build`, `just serve`, `just client`, `just preview` recipes if missing.
  - Ensure `vysma` crate compiles and has stub subcommands printing help.
- Acceptance
  - `cargo build --workspace` succeeds.
  - `vysma -h` prints global help; `vysma new -h`, `vysma serve -h`, `vysma client -h`, `vysma preview -h` print usage.
- Files/Crates
  - `crates/vysma/` — CLI; add subcommand stubs.
  - `Justfile` — ensure basic recipes exist.

---

## 1) CLI DX foundation (Phase 2.1)
- Goal: One command creates and runs a project locally; editing HCL hot‑reloads.
- Tasks
  - `vysma new <name>`: scaffold folders `assets/{scenes,mesh,textures,fonts}` and an example scene.
  - Write `.gitignore` (`target/`, `.DS_Store`). Optional `README.md` in project.
  - `vysma serve`: run server bin with hot‑reload; print endpoints, keybindings, and mDNS/QR placeholders.
  - `vysma client --connect lan|ws(s)://...`: run a viewer client; default connects to local server.
- Acceptance
  - `vysma new demo && cd demo && vysma serve` logs "Watching assets" and HCL parse success.
  - `vysma client --connect lan` renders the sample scene; Apply from editor (when implemented) propagates.
- Commands
  - `vysma new demo`
  - `vysma serve`
  - `vysma client --connect lan`
- Files/Crates
  - `crates/vysma/` — implement subcommands.
  - Add `templates/basic/` and `templates/editor_game/` under CLI crate.
  - `crates/vysma-app/` — ensure server/client bins callable from CLI.

---

-## 2) Content‑addressed assets + manifest + bundle index (Phase 2.2)
- Goal: Publish deduped assets, embed manifest, and upload a bundle `index.toml`; runtime maps `file`→Appwrite Storage URLs.
- Tasks
  - Hash assets (sha256), parallel uploads with retries; 409 → skip.
  - Manifest rows store `url_path = fileId` (flat storage id). Persist manifest as string JSON in ModuleVersion.
  - Generate and upload `index.toml` with module metadata, deps, and resources; include it in the manifest.
  - Optional `ModuleAssetsIndex` rows for search/debug.
- Acceptance
  - `vysma module publish --owner you --name demo --version 0.0.1 --hcl assets/scenes/demo.hcl --assets assets/` prints manifest table; re-run skips unchanged.
- Files/Crates
  - `crates/vysma/` — publish command (+ bundle index generation).
  - `crates/vysma-cloud/` — Appwrite client helpers.

---

## 3) Editor UI (desktop) — minimal (Phase 3 part A)
- Goal: In-app panel with Edit/Preview toggle, multiline editor, Apply, status.
- Tasks
  - Feature gate: `gui && client`.
  - Resources: `EditorBuffer`, `LastApplied`.
  - Systems: buffer sync from `HclSceneBlob` (first time), Apply → send `HclUpdateRequest`.
  - Disable Apply in Preview; show last sha/time.
- Acceptance
  - F5 toggles modes; Apply in Edit updates server; clients respawn.
- Files/Crates
  - `crates/vysma-app/` — client GUI systems under `features = ["gui", "client"]`.
  - `docs/editor-ui.md` — check off MVP items.

---

## 4) Editor-as-Game template (requested)
- Goal: The Editor itself is a starter game you can scaffold and extend; it ships as a template runnable via CLI.
- Tasks
  - Add `templates/editor_game/` project including:
    - A Bevy app that enables `gui` editor panel and loads an example HCL scene.
    - HCL that demonstrates editing itself (e.g., HUD text showing a var; simple movement rule).
  - `vysma new --template editor_game <name>` to scaffold this project.
  - Document how to remove/toggle editor panel for shipping builds.
- Acceptance
  - `vysma new --template editor_game my_editor && cd my_editor && cargo run` opens a window with the editor panel and example scene; editing HCL and Apply changes the running scene.
- Files/Crates
  - `crates/vysma/` — template selection support.
  - `templates/editor_game/**` — new template content.

---

## 5) Editor auth (JWT) — minimal enforcement (Phase 3 part B)
- Goal: Gate Apply with a JWT verified against Appwrite JWKS.
- Tasks
  - CLI `vysma login` stores profile in `~/.vysma/config.toml`.
  - Editor includes `Authorization: Bearer <jwt>` on update; server verifies via JWKS and project membership.
- Acceptance
  - Without JWT (when required), server logs and ignores updates; with JWT, Apply works.
- Files/Crates
  - `crates/vysma/` — login flow, profiles.
  - `crates/vysma-cloud/` — JWKS verify helper.
  - `crates/vysma-app/` — attach JWT in client; verify in server.

---

## 6) Local‑first asset server + mDNS + QR (Priority from Local‑first section)
- Goal: Serve `assets/` over HTTP for same‑device and LAN preview; easy phone testing.
- Tasks
  - Implement CLI subcommand `vysma assets serve [--lan|--public] [--port Port]`:
    - Headers: `Access-Control-Allow-Origin: *`, `Cross-Origin-Resource-Policy: cross-origin`, `ETag`, range requests.
    - Default bind `127.0.0.1`; `--lan|--public` binds `0.0.0.0` and issues a random token (header or `?t=`) with short TTL.
  - mDNS broadcast `{ host, port }` under `_vysma._udp.local`.
  - CLI prints WS URL + asset URL and a QR code for mobile.
- Acceptance
  - Second device on LAN loads assets and mirrors updates within <1s on Apply.
- Files/Crates
  - `crates/vysma/` — HTTP static server, mDNS, QR output.
  - `docs/architecture-plan.md` — mark acceptance for Local‑first.

---

## 7) WASM browser preview (Phase 5 slice for MVP)
- Goal: `vysma preview --open` builds wasm target and opens a page connecting via WebSocket to local server.
- Tasks
  - Target: `wasm32-unknown-unknown` with `http_assets` feature enabled.
  - Serve a static HTML/JS shell; ensure COOP/COEP on the page; asset server provides CORP.
- Acceptance
  - Browser renders the example scene; live updates apply; console shows connect/log lines.
- Files/Crates
  - `crates/vysma/` — preview command.
  - `docs/http-asset-io.md` — CORS/CORP checklist update.

---

## 8) Diagnostics with fix‑its
- Goal: Error messages include actionable fix‑its; editor UI can “Jump to line”.
- Tasks
  - Parser surfaces line/column and error code; map common errors to suggestions (unknown component key, missing asset, bad include).
  - Editor shows the error and offers a one‑click jump; CLI prints caret and suggestion.
- Acceptance
  - Intentionally breaking HCL shows a clear fix‑it both in CLI and editor UI.
- Files/Crates
  - `crates/vysma-hcl/` — error codes/messages.
  - `crates/vysma-app/` — editor UI integration.

---

## 9) Module gallery (curated install)
- Goal: Browse and add modules visually; inserts import block and assets manifest resolves transparently.
- Tasks
  - CLI: `vysma modules search <term>` and `vysma modules add <owner::name>@<ver?>`.
  - In-editor gallery panel (optional for MVP): fetch curated list; click to add import stub.
- Acceptance
  - Adding a module updates the scene and it loads on next Apply.
- Files/Crates
  - `crates/vysma/` — search/add.
  - `crates/vysma-cloud/` — list APIs.

---

## 10) Share link via Relay (Phase 6)
- Goal: One click prints a short project URL and QR; remote device connects via relay.
- Tasks
  - Server registers with relay; prints URL.
  - CLI `vysma share` toggles relay on and prints QR; tokens are short‑lived and single‑use preferred.
- Acceptance
  - Remote client connects via short URL and receives updates; unauthorized Apply still blocked without JWT.
- Files/Crates
  - `crates/vysma-net/` + relay glue
  - `crates/vysma/` — share command
  - `docs/relay-discovery.md` — confirm URL/token flows

---

## 11) Scene persistence (Phase 4)
- Goal: On accepted update, server saves a SceneVersion; reloads latest on restart.
- Tasks
  - Server writes `SceneVersions` with { hcl, sha, author } and updates `Scenes.publishedVersionId`.
  - On startup, server loads published scene if present.
- Acceptance
  - Restart preserves the current scene.
- Files/Crates
  - `crates/vysma-cloud/` — scene APIs
  - `crates/vysma-app/` — server hooks

---

## Editor-as-Game specifics (design contract)
- Templates
  - `templates/editor_game/` includes:
    - `src/main.rs` that enables `gui` and `client` features of `vysma-app`.
    - `assets/scenes/editor_demo.hcl` with:
      - Vars and bindings that update HUD text every 0.1s.
      - A Player prefab/entity with `move_w` rule.
    - `README.md` explaining the editor panel and how to toggle it.
- CLI
  - `vysma new --template editor_game <name>` copies the template and runs `cargo run`.
- Success demo
  - Edit the HUD text or speed var in the panel, press Apply, and watch the running game change.

---

## Milestone acceptance snapshots
- M1 (Steps 1–3):
  - Create → Serve → Client → Apply works on desktop; publish prints manifest.
- M2 (Steps 4–6):
  - Editor-as-Game template runs; LAN preview via asset server + QR; JWT gate enforced when enabled.
- M3 (Steps 7–10):
  - Browser preview works; share link via relay connects from remote; diagnostics have fix‑its.
- M4 (Step 11):
  - Scene persistence survives restart.

---

## Command reference (expected UX)
- `vysma new <name>` — scaffold basic project
- `vysma new --template editor_game <name>` — scaffold Editor-as-Game
- `vysma serve [--lan]` — run server with hot-reload; print ws url and discovery status
- `vysma client --connect <lan|ws(s)://...>` — run viewer client
- `vysma assets serve [--lan|--public]` — serve static assets with safe headers
- `vysma preview --open` — build wasm and open browser page
- `vysma module publish --owner <u> --name <n> --version <v> --hcl <file> --assets <dir>` — publish
- `vysma login --endpoint <url> --project <id> [--profile dev|prod]` — auth for JWT/editor
- `vysma share` — register with relay and print short URL + QR

---

## Notes for implementers
- Keep features aligned across crates: `client`, `server`, `gui`, `http_assets`.
- Print friendly, structured logs and `--json` for machine parsing.
- Avoid deep abstractions; small, named modules and explicit types improve DX.


