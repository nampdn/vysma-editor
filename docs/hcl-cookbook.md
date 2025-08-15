### Vysma HCL Cookbook

A self-contained set of examples showing how to build different game styles with just HCL. Copy, paste, tweak numbers, and compose modules. No Rust required.

---

## 1) Arcade: Endless Runner (2D flavor with 3D primitives)

Goals: a player moves forward; obstacles spawn; jump on Space; score increments.

```hcl
assets {
  mesh "cube" { builtin = "cube" }
  material "player_mat" { pbr = { base_color = "#3aa7ff" } }
  material "obstacle_mat" { pbr = { base_color = "#ff7a3a" } }
}

vars = { speed = 8.0, jump = 12.0, gravity = -25.0, score = 0.0, cooldown = 0.0 }

prefab "Player" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "player_mat" }, Transform = { s = [0.5,0.5,0.5], t = [0,1,0] } } }

entity "root" { children = [ { name = "Runner", include = ["Player"] } ] }

triggers {
  # Constant forward move
  trigger "run" { on = { tick = { every = 0.016 } } target = { name = "Runner" } actions = [ { translate = { by = [0,0,-0.16] } }, { add_var = { name = "score", by = 0.016 } } ] }
  # Jump on Space, simple vertical impulse
  trigger "jump" {
    on = { key_pressed = "Space" }
    when = [ { expr = "cooldown <= 0" } ]
    actions = [ { add_var = { name = "vy", by = 12.0 } }, { set_var = { name = "cooldown", value = 0.5 } } ]
  }
  trigger "gravity" { on = { tick = { every = 0.016 } } actions = [ { add_var = { name = "vy", by = -0.4 } } ] }
  trigger "apply_vy" { on = { tick = { every = 0.016 } } target = { name = "Runner" } actions = [ { apply = { path = "Transform.t", value = [0, "vy", 0] } } ] }
  trigger "cooldown_tick" { on = { tick = { every = 0.05 } } actions = [ { add_var = { name = "cooldown", by = -0.05 } } ] }
}
```

Tips:
- Replace primitives with module `core::RunnerPlayer` later.
- For collisions, add a physics module and listen to `event = "physics.hit"`.

---

## 2) Platformer (compose a core module)

Import a `platformer_core` module that provides Camera2D, Player, Ground prefabs and common triggers.

```hcl
modules = [ { name = "alice::platformer_core", alias = "core" } ]

entity "root" {
  children = [
    { name = "Camera", include = ["core::Camera2D"] },
    { name = "Player", include = ["core::Player"], components = { Transform = { t = [0,1,0] } } },
    { name = "Ground", include = ["core::Ground"], components = { Transform = { t = [0,0,0], s = [10,1,1] } } }
  ]
}

# Optional: tweak run speed and jump
vars = { run = 6.5, jump = 11.0 }

# Add local flavor without touching the module
triggers { trigger "blink_camera" { on = { tick = { every = 0.5 } } target = { name = "Camera" } actions = [ { toggle_visibility = {} } ] } }
```

---

## 3) Top‑Down Shooter (waves)

Spawn enemies every few seconds; move player with WASD; fire on Space; basic damage loop.

```hcl
assets { mesh "cube" { builtin = "cube" } material "p" { pbr = { base_color = "#3aa7ff" } } material "e" { pbr = { base_color = "#ff3a5e" } } }
vars = { speed = 8.0, hp = 100.0 }
prefab "Player" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "p" }, Transform = { s = [0.6,0.6,0.6] } } }
prefab "Enemy" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "e" }, Transform = { s = [0.5,0.5,0.5] } } }
entity "root" { children = [ { name = "Player", include = ["Player"], components = { Transform = { t = [0,1,0] } } } ] }
triggers {
  # Move with WASD
  trigger "move" { on = { tick = { every = 0.016 } } target = { name = "Player" } actions = [
    { translate_axis = { vec = [ "KeyD?1:-1?0", 0, 0 ], speed_var = "speed" } },
    { translate_axis = { vec = [ 0, 0, "KeyS?1:-1?0" ], speed_var = "speed" } }
  ] }
  # Wave spawns
  trigger "spawn_wave" { on = { tick = { every = 3.0 } } actions = [ { spawn = { prefab = "Enemy", components = { Transform = { t = [4,1,-6] } } } } ] }
  # Shoot and hit (conceptual; add physics module for collisions)
  trigger "shoot" { on = { key_pressed = "Space" } actions = [ { emit = { name = "bullet.spawn" } } ] }
  trigger "hit" { on = { event = "combat.hit" } actions = [ { add_var = { name = "hp", by = -10.0 } } ] }
}
```

Note: For `KeyX?` ternary-like shortcuts, prefer explicit triggers by key for readability in production.

---

## 4) MOBA‑like (reusing modules)

```hcl
modules = [ { name = "alice::moba_core", alias = "core" }, { name = "alice::axe", alias = "axe" } ]

entity "root" {
  children = [
    { name = "Game", include = ["core::BaseGame"], components = { Transform = { t = [0,0,0] } } },
    { name = "HeroA", include = ["axe::Axe"], components = { Transform = { t = [2,0,0] } } }
  ]
}

vars = { spawn_period = 15.0 }

triggers {
  trigger "creeps" { on = { tick = { every = 15.0 } } actions = [ { spawn = { prefab = "core::Creep", components = { Transform = { t = [0,0,-5] } } } } ] }
}
```

---

## 5) Puzzle (grid interactions)

Minimal logic by toggling visibility and tracking a few vars.

```hcl
assets { mesh "cube" { builtin = "cube" } }
vars = { a = 0, b = 0 }
entity "root" { children = [ { name = "A" }, { name = "B" } ] }
triggers {
  trigger "toggle_a" { on = { key_pressed = "KeyA" } target = { name = "A" } actions = [ { toggle_visibility = {} }, { add_var = { name = "a", by = 1.0 } } ] }
  trigger "toggle_b" { on = { key_pressed = "KeyB" } target = { name = "B" } actions = [ { toggle_visibility = {} }, { add_var = { name = "b", by = 1.0 } } ] }
  trigger "win" { on = { tick = { every = 0.1 } } when = [ { expr = "a >= 2 && b >= 2" } ] actions = [ { emit = { name = "puzzle.win" } } ] }
}
```

---

## Composition tips
- Keep modules small and focused (camera, character, enemy, UI HUD) so kids can mix them.
- Prefer prefabs + entity includes; avoid custom function layers.
- Override via components and add your own triggers; don’t edit imported modules.

## Notes
- For physics, pathfinding, or inventory, import modules that emit events like `physics.hit`, `nav.arrived`, `inventory.changed` and react with triggers.
- For remote assets, publish modules so their assets resolve via HTTP automatically. 

---

## 6) MMO flavor: chat + matchmaking (server authority)

```hcl
# Enable server-authoritative triggers for backend actions
trigger "queue_for_match" { authority = "server" on = { event = "ui.queue" } actions = [ { match_queue = { mode = "pvp", size = 5 } } ] }
trigger "announce_match" { authority = "server" on = { event = "party.match_found" } actions = [ { send_chat = { channel = "party", text = "Match found!" } } ] }

# Teleport a party into a fresh instance when ready
trigger "enter_instance" {
  authority = "server"
  on = { event = "party.match_found" }
  actions = [
    { instance_create = { template = "dungeon01", store_as = "inst" } },
    { instance_transfer = { targets = { tag = "party" }, instance_id = "${inst}", spawn = [0,1,0] } }
  ]
}

# Simple chat relay example (backend wires chat.message events)
trigger "party_chat" { authority = "server" on = { event = "chat.message" } when = [ { expr = "channel == 2" } ] actions = [ { log = { message = "party chat received" } } ] }
``` 