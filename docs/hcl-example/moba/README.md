### MOBA Example and Reusable Module

This folder contains:
- `modules/moba_core.hcl`: a reusable module exporting `core::BaseGame`, `core::Camera3D`, `core::Text`, `core::Creep`, `core::Projectile`, `core::AoE`.
- `scene.hcl`: client scene wiring inputs and UI to the server.
- `server.hcl`: server‑authoritative logic for movement, abilities, creeps, towers, persistence, and MMO hooks.
- `abilities.hcl`: optional abilities/UI example (client‑side VFX + input emit).
- `ez.hcl`: kids‑friendly EZ Mode sample.

Using the module in your scene
```hcl
modules = [ { name = "you::moba_core", alias = "core" } ]
entity "root" { children = [
  { name = "Game", include = ["core::BaseGame"] },
  { name = "Camera", include = ["core::Camera3D"] },
  { name = "Hero",   include = ["core::Creep"], components = { Name = "Player" } }
] }
```

Publishing
- Run: `vysma module publish --owner you --name moba_core --version 0.1.0 --hcl docs/hcl-example/moba/modules/moba_core.hcl`
- Consumers can import via: `modules = [{ name = "you::moba_core", alias = "core", version = "0.1.0" }]` 