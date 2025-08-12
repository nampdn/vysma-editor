# Vysma HCL: Data‑Driven Game Authoring

This docs set explains how to build games using the in‑engine HCL (HashiCorp Configuration Language) workflow layered over Bevy ECS. HCL lets you define assets, prefabs, entities, triggers, and runtime logic without changing Rust code.

Start here:
- hcl-spec.md — Full language and engine integration spec
- See `assets/scenes/moba_game.hcl` for a compact end‑to‑end example (terrain, hero model, movement, combat, respawn)

## Quickstart

1) Enable the HCL plugin (already done in `src/main.rs`):
- `app.add_plugins(bevy_in_app::hcl::HclPlugin);`
- `app.add_systems(Startup, bevy_in_app::hcl::load_scene_at_startup("scenes/moba_game.hcl"));`

2) Edit your HCL scene(s) under `assets/scenes/`. The engine hot‑reloads edited files and respawns the scene.

3) Iterate: change assets/prefabs/entities/triggers. Use debug logs to see which triggers fire and which actions run.

4) For a live overlay, toggle periodic log output from HCL:
- Add at startup: `{ set_var = { name = "debug_overlay_log", value = 1.0 } }`
- The engine prints a compact line with key vars and recent events every second.
- Optional on‑screen overlay exists behind a feature flag; see hcl-spec.md.

## Philosophy

- Rust remains domain‑agnostic. HCL defines game content and logic.
- ECS is the substrate. HCL components map to Bevy components via appliers.
- Triggers/Events/Actions define behavior. Expressions provide flexible runtime math.
- Modularity by includes and modules (WIP) enables sharing and composition.

## Contents

- hcl-spec.md — schema reference and authoring patterns
- Future topics (planned):
  - Function registry and stdlib utilities
  - Broader Apply/Get coverage and component reflection
  - Event payload reading in expressions/conditions
  - Typed variable scopes (global/entity/tag) 