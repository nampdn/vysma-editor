### Repo Refactor Plan (root/src → crates)

Goal: make the codebase modular, maintainable, and fast to iterate by moving remaining `root/src` into focused crates, aligning with docs and keeping every step build‑green.

#### Current state (summary)
- Workspace crates: `vysma-hcl` (HCL runtime), `vysma-cloud` (Appwrite), `vysma` (CLI)
- Root crate `bevy-in-app` under `src/` hosts cross‑cutting app code: `common/`, `client/`, `server/`, `renderer/`, `protocol.rs`, `input_binding.rs`, platform bits under `app_view/`, `ffi/`, `android_asset_io.rs`.
- Legacy duplicate `src/hcl/` exists but we re‑export `vysma-hcl`.

#### Target crate layout
- `crates/vysma-app` (lib): Bevy app orchestration (formerly `bevy-in-app`)
  - Modules: `common` (window/log/dx), `renderer`, `app_modes` (Edit/Preview), `input_binding`, `protocol`
  - Features: `client`, `server`, `gui`, `http_assets` (proxy to `vysma-hcl/http_assets`)
- `crates/vysma-net` (lib): Lightyear wrappers & networking glue
  - `client_network`, `server_network`, `shared_consts`
- `crates/vysma-platform` (lib): Platform integration (Android/iOS)
  - `android_asset_io`, `ffi` (android/ios), `app_view` (mobile view layer)
  - Features: `android`, `ios`
- Remove root `src/hcl/` (unify on `vysma-hcl`)
- Keep `vysma` (CLI), `vysma-hcl`, `vysma-cloud` as is

Optional (post‑migration):
- `crates/vysma-editor-ui` (future desktop overlay/editor GUI)

#### Migration steps (each step must build green)
1) Prep: freeze optional features
- Disable or feature‑gate unresolved extras in root (steam, visualizer, metrics dashboard) so default build is green.
- Verify: `cargo build --workspace`

2) Extract networking to `crates/vysma-net`
- Create crate; move `src/common/client_network.rs`, `src/common/server_network.rs`, `src/common/shared.rs` to it.
- Expose minimal API: `ClientNetwork`, `ServerNetwork`, `SharedSettings`, helpers.
- Update imports in app to use `vysma-net`.
- Verify: `cargo build --workspace` and run `just serve`/`just client`.

3) Extract platform code to `crates/vysma-platform`
- Move `src/android_asset_io.rs`, `src/ffi/*`, `src/app_view/*`.
- Feature‑gate per platform; ensure no desktop build regressions.
- Update `vysma-app` to depend on `vysma-platform` via features.
- Verify desktop build; smoke mobile builds unchanged.

4) Create `crates/vysma-app` and move app modules
- Create crate; move from root `src/`:
  - `common/` (except networks already moved), `renderer/`, `protocol.rs`, `input_binding.rs`, `server/`, `client/`, `shared.rs` (if not network constants)
- Rename root crate from `bevy-in-app` → `vysma-app` (package name + path) or make root just a thin bin that depends on `vysma-app`.
- Update `src/main.rs` (if any bin remains) to call into `vysma-app`.
- Verify: `cargo build --workspace`.

5) Remove legacy `root/src/hcl`
- Delete `src/hcl/`; ensure all references use `vysma-hcl`.
- Verify hot‑reload and module imports still work.

6) Thin root package
- Root `Cargo.toml` becomes a pure workspace manifest.
- Any root bins (scripts) remain under `scripts/` as [[bin]] pointing into simple drivers.
- Verify: `cargo build --workspace`.

7) File size/complexity cleanup (within crates)
- Split large modules:
  - In `vysma-hcl`: consider splitting `runtime.rs` into `runtime/{events.rs,actions.rs,conditions.rs,exec.rs}` (no behavior change).
  - In `vysma-app`: split `common/cli.rs` and renderer utilities into smaller files (`window.rs`, `log.rs`).
- Verify: builds + quick `just serve/client`.

8) Feature alignment & documentation
- Ensure feature flags:
  - `http_assets` flows from `vysma-app` → `vysma-hcl`.
  - Platform features `android/ios` isolated in `vysma-platform`.
- Update docs to reflect crate boundaries, features, and dev flows.

9) CI updates
- Build matrix for: default desktop, `--features http_assets`.
- Add a job to run `vysma module publish --dry-run` and `just build`.

10) Cleanup checklist
- Remove stale imports and dead code (cargo fix/clippy where safe).
- Delete unused assets and duplicate fonts/shaders if moved to shared asset locations.

#### Paths mapping (reference)
- `src/common/client_network.rs` → `crates/vysma-net/src/client.rs`
- `src/common/server_network.rs` → `crates/vysma-net/src/server.rs`
- `src/common/shared.rs` → `crates/vysma-net/src/shared.rs`
- `src/android_asset_io.rs` → `crates/vysma-platform/src/android_asset_io.rs`
- `src/ffi/{android,ios}.rs` → `crates/vysma-platform/src/ffi/{android,ios}.rs`
- `src/app_view/*` → `crates/vysma-platform/src/app_view/*`
- `src/renderer.rs` → `crates/vysma-app/src/renderer.rs`
- `src/common/*` (non-net) → `crates/vysma-app/src/common/*`
- `src/protocol.rs` → `crates/vysma-app/src/protocol.rs`
- `src/input_binding.rs` → `crates/vysma-app/src/input_binding.rs`
- `src/{client,server}/` → `crates/vysma-app/src/{client,server}/`
- Remove `src/hcl/*` (use `vysma-hcl`)

#### Acceptance per step
- Build green: `cargo build --workspace`
- Runtime: `just serve` + `just client` connects; HCL loads; F6 demo works in Edit mode
- CLI: `vysma module publish --dry-run` prints sha + manifest

#### Guidelines (consistency)
- Public APIs typed and documented; long function files split by domain
- Features clearly propagated; optional deps behind features
- Keep default build lean; heavy integrations opt‑in 