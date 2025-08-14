### Developer Setup and Fast Iteration Guide

This repo includes a CLI (`vysma`) and Bevy app (`bevy-in-app`) for server/client.
Use these steps to get a fast loop for iterating on features and testing end-to-end.

#### Prereqs
- Rust toolchain (latest stable)
- just (optional but recommended): `brew install just` (macOS)
- Appwrite project (optional for publish): endpoint + API key + database/collections

#### Env
Copy `.env.example` to `.env` and edit values, or export env vars in your shell.

- Required to publish modules:
  - `APPWRITE_ENDPOINT`, `APPWRITE_PROJECT_ID`, `APPWRITE_API_KEY`
  - `APPWRITE_DATABASE_ID`, `APPWRITE_MODULES_COLLECTION_ID`, `APPWRITE_MODULEVERSIONS_COLLECTION_ID`
- Optional:
  - `APPWRITE_MODULE_ASSETS_BUCKET_ID` (defaults to `module-assets`)
  - `APPWRITE_MODULE_ASSETS_INDEX_COLLECTION_ID`
  - `VYSMA_ASSET_BASE_URL` (e.g., `https://storage.example.com`)

#### Common Commands
If you have `just`, run `just` to list commands. Otherwise, run the cargo equivalents.

- Build (default): `just build` (or `cargo build --workspace`)
- Build (all features): `just build-all`
- Run server: `just serve` (or `cargo run -- server`)
- Run client: `just client` (or `cargo run -- client -c 1`)
- Bootstrap new project: `just new mygame` (creates `sandbox/mygame` with assets)
- Ensure schema: `just ensure-schema`
- Verify env: `just verify`
- Publish module (example):
  - `just publish alice axe 0.1.0 assets/moba_hcl/heroes/axe.hcl assets/mesh/heroes`
- Dry-run publish (no network; compute manifest only):
  - `just publish-dry alice axe 0.1.0 assets/moba_hcl/heroes/axe.hcl assets/mesh/heroes`

#### Local Iteration Flow
1) Start server in one terminal: `just serve`
2) Start client in another: `just client`
3) Edit HCL files under `assets/`.
   - In Edit mode (F5), pressing F6 sends an example update, or use the editor flow to push `HclUpdateRequest`.
4) To test module imports from Appwrite: set env, ensure schema, publish a module, and include it via `modules = [...]`.
   - Set `VYSMA_ASSET_BASE_URL` so runtime resolves manifest `url_path` to a full URL.
   - Enable HTTP asset IO when needed with `--features http_assets`.

#### Tips
- Use `vysma module publish --dry-run` to quickly see the manifest and sha.
- When iterating on remote modules, restart server after publishing to pick up changes.
- Keep default build green; optional features like Steam/visualizer are disabled by default. 