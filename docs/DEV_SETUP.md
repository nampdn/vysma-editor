### Developer Setup

- Install Rust stable and just (optional):
  - rustup default stable
  - brew install just

- Common commands:
  - `just build` — build current crate
  - `just build-all` — build entire workspace
  - `just serve` — run desktop server
  - `just client` — run desktop client
  - `just host` — run host-client
  - `just build-http` — build workspace with HTTP assets
  - `just publish-dry module name version root` — dry-run manifest for a module

- Cargo aliases (via `.cargo/config.toml`):
  - `cargo cl` — run client (GUI)
  - `cargo sv` — run server
  - `cargo hc` — run host-client
  - `cargo cl-http` / `cargo sv-http` — with HTTP assets feature

- CLI (Dev/SaaS)
  - `vysma login --endpoint <APPWRITE_ENDPOINT> --project <APPWRITE_PROJECT_ID> [--profile dev|prod]` — authenticate and store tokens under `~/.vysma/config.toml`
  - `vysma serve` — start local authoritative server with hot‑reload; prints LAN address and optional relay URL
  - `vysma client --connect <ws(s)://... or short url>` — connect a viewer client (desktop) to the server or relay
  - `vysma module publish --owner <u> --name <n> --version <v> --hcl <file> --assets <dir>` — publish a module to Appwrite
  - `vysma preview --open` — build/run WASM browser preview connecting to the local server/relay

- Typical flow:
  1) `cargo run -p vysma -- new demo`
  2) `vysma login --endpoint https://appwrite.dev/v1 --project <id> --profile dev`
  3) `vysma serve` (server) and `vysma client --connect lan` (client)
  4) Edit HCL and assets; server hot‑reloads; connected clients update live
  5) Optionally, `vysma preview --open` for browser
  6) `just build-all`
  7) `vysma module publish --owner <you> --name demo --version 0.0.1 --hcl assets/moba_hcl/moba_game.hcl --assets assets/` 