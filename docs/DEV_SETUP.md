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

- Typical flow:
  1) `cargo run -p vysma -- new demo`
  2) `just serve` (server) and `just client` (client)
  3) Make changes; `just build-all`
  4) `just publish-dry module demo 0.0.1 assets/moba_hcl` 