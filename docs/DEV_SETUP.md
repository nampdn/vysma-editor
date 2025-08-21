### Developer Setup

- Install Rust stable and just (optional):
  - rustup default stable
  - brew install just

- Common commands:
  - `just build` — build current crate
  - `just build-all` — build entire workspace
  - `just serve` — run desktop server with auto-discovery
  - `just serve-quick 5` — run server for 5s and auto-exit (useful in CI)
  - `just client` — run desktop client with auto-discovery
  - `just client-quick 5` — run client for 5s and auto-exit (useful in CI)
  - `just host` — run host-client
  - `just host-quick 8` — run server+client for 8s and auto-exit
  - `just build-http` — build workspace with HTTP assets
  - `just module-publish-dry owner name version hcl assets` — dry-run publish; pass '-' for no assets

- Cargo aliases (via `.cargo/config.toml`):
  - `cargo cl` — run client (GUI)
  - `cargo sv` — run server
  - `cargo hc` — run host-client
  - `cargo cl-http` / `cargo sv-http` — with HTTP assets feature

- CLI (Dev/SaaS)
  - `vysma login --endpoint <APPWRITE_ENDPOINT> --project <APPWRITE_PROJECT_ID> [--profile dev|prod]` — authenticate and store tokens under `~/.vysma/config.toml`
  - `vysma serve` — start local authoritative server with hot‑reload and auto-discovery; prints LAN address and optional relay URL
  - `vysma client --connect <ws(s)://... or short url>` — connect a viewer client (desktop) to the server or relay
  - `vysma module publish --owner <u> --name <n> --version <v> --hcl <file> --assets <dir>` — publish a module to Appwrite
  - `vysma preview --open` — build/run WASM browser preview connecting to the local server/relay

- **COMPLETED**: Auto-discovery and Smart CLI ✅
  - `vysma new <name>` — scaffold basic project with `assets/main.hcl`
  - `vysma new --template editor_game <name>` — scaffold editor-as-game project
  - CLI automatically discovers HCL files in priority order: `main.hcl` > `scene.hcl` > `scenes/*.hcl` > any `.hcl`
  - No more hardcoded paths - works from any project directory
  - **TESTED**: Successfully creates projects, auto-discovers scenes, and runs without path errors

- **IN PROGRESS**: Editor Authentication (JWT) 🔄
  - `vysma auth login --endpoint <url> --project <id> --key <api_key> --profile <name>` — store Appwrite credentials
  - `vysma auth logout --profile <name>` — remove stored profile
  - `vysma auth list` — show configured profiles
  - **STATUS**: CLI auth working, JWT infrastructure ready, need editor integration

- Typical flow:
  1) `vysma new demo` or `vysma new --template editor_game demo`
  2) `vysma login --endpoint https://appwrite.dev/v1 --project <id> --profile dev`
  3) `vysma serve` (server with auto-discovery) and `vysma client --connect lan` (client)
  4) Edit HCL and assets; server hot‑reloads; connected clients update live
  5) Optionally, `vysma preview --open` for browser
  6) `just build-all`
  7) `vysma module publish --owner <you> --name demo --version 0.0.1 --hcl assets/main.hcl --assets assets/`

## Auto-Discovery System

The CLI now automatically discovers and loads HCL files from the current working directory:

### Priority Order
1. `assets/main.hcl` (primary entry point)
2. `assets/scene.hcl` or `assets/game.hcl`
3. Any `.hcl` files in `assets/scenes/`
4. Any other `.hcl` files in `assets/`

### Benefits
- **No hardcoded paths**: Works from any project directory
- **Flexible structure**: Supports different project organizations
- **Automatic detection**: New HCL files are automatically found
- **Smart loading**: Chooses the most appropriate scene file
- **Hot reload**: File changes trigger automatic scene switching

### Example Project Structure
```
mygame/
├── assets/
│   ├── main.hcl          # Auto-discovered primary scene
│   ├── scenes/
│   │   ├── level1.hcl    # Additional scenes
│   │   └── level2.hcl
│   ├── mesh/
│   └── textures/
└── README.md
```

The CLI will automatically load `assets/main.hcl` and watch for changes in all HCL files. 