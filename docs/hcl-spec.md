# Vysma HCL Specification and Authoring Guide

This document is the developer‑facing reference for writing HCL scenes and modules that drive the Bevy‑based engine at runtime. It covers the schema, supported features, and practical authoring tips for rapid iteration.

HCL files live under `assets/` (e.g., `assets/scenes/moba_game.hcl`). The engine hot‑reloads edited files.

## Authoring Philosophy (DX/UX)

- Human‑readable first: You define assets by name and local relative paths. You never paste hashes or opaque URLs in HCL.
- Name references everywhere: Components refer to assets by their declared names (`image = "HeroIcon"`, `scene = "Axe"`).
- Relative paths resolve locally: `file = "textures/hero.png"` is local while developing.
- Remote resolution is automatic: When you publish a module, the CLI generates a manifest (path → sha/url). At runtime, the engine maps `file` to a remote URL using the manifest—no HCL changes needed.

## Intuitive ECAS Model (Event → Condition → Action → State)

This is the mental model and target authoring shape. Current runtime already supports the core; extended conditioning/state scopes are planned and will be introduced without breaking existing scenes.

- **Event (E)**: what happened (key, tick, startup, timer, custom event).
- **Condition (C)**: guards that must pass. Simple boolean expressions and selectors. Planned: `any/all/none` groups.
- **Action (A)**: what to do (move, spawn, emit, set var, apply component fields).
- **State (S)**: variables you read/write. Global exists today; planned: entity/tag‑scoped state.

Canonical trigger shape:
```hcl
trigger "name" {
  on = { event = "..." | key_held = "KeyW" | tick = { every = 0.1 } | timer = "name" | startup = true }
  # Optional guard(s)
  when = [ { expr = "cooldown <= 0" } ]
  # Planned sugar (grouped conditions):
  # when_any = [ { expr = "hp < 30" }, { expr = "god_mode == 1" } ]
  # when_all = [ { expr = "alive == 1" } ]
  # when_none = [ { expr = "stunned == 1" } ]
  # Optional local bindings (planned)
  # let = { speed_now = "speed * 1.5", dx = "cos(t) * 0.1" }
  target = Selector?  # default target for actions
  actions = [ ... ]
}
```

Why this shape?
- Read left‑to‑right: event → guards → actions → state updates.
- Predictable execution: actions run in order; state writes are visible to later actions in the same trigger.

---

## Top‑Level Structure

A scene file is a single document with optional attributes and blocks:

- assets { ... } — Asset declarations used by the scene
- prefab "Name" { ... } — Reusable component collections
- entity "Name" { ... } — Scene graph nodes (with children)
- triggers { trigger "Name" { ... } ... } — Event‑driven behavior
- vars = { key = number, ... } — Global numeric variables (f64)
- includes = [ "path/to/other.hcl", ... ] — Merge other scenes into this one
- modules / exports — Module system (namespacing and sharing; evolving)
- functions — Parsed placeholders for future function evaluation integration

Example:
```hcl
assets { mesh "cube" { builtin = "cube" } material "blue" { pbr = { base_color = "#3aa7ff" } } }
vars = { speed = 6.0 }
prefab "Player" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "blue" }, Transform = { s = [1,1,1] } } }
entity "root" { children = [ { name = "Hero", include = ["Player"], components = { Transform = { t = [0,1,0] } } } ] }
triggers { trigger "move_w" { on = { key_held = "KeyW" } target = { name = "Hero" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed" } } ] } }
```

## Assets

Define meshes, materials, images, and glTF scenes referenced by components:

- mesh "name" { builtin = "cube" | "plane" }
- material "name" { pbr = { base_color = "#rrggbb" | {r,g,b,a?}, metallic = f32?, roughness = f32?, emissive = color? } }
- image "name" { file = "textures/..." }
- gltf  "name" { file = "mesh/...glb", node = "NodeName"? }

Notes
- Local workflow: `file` is a relative path you keep in your repo.
- Remote workflow: When running a published module, the engine uses the module manifest to map `file` to a CDN URL (e.g., `owner/name/<sha256>.ext`). No HCL changes are required.
- You may also specify `url` instead of `file` for advanced cases; the loader will treat `url` as a remote path directly.

The loader builds a per‑scene `ApplyCtx` with handles to these assets. `SceneRef` uses `gltf` entries.

## Prefabs

Reusable component sets by name:

```hcl
prefab "Crate" {
  components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "wood" }, Transform = { s = [0.5,0.5,0.5] } }
  tags = ["loot"]
}
```

Use `include = ["Crate"]` in entities to merge prefab components, then override in the entity.

## Entities

Scene graph nodes with components and children:

- name = auto from block label; or specify in components via `Name = "..."`
- include = ["Prefab", ...] merges prefab components first
- components = { ... } keys must match registered component appliers (see Components)
- tags = ["tag"]: tags for selectors
- children = [ { name = "Child", components = {...} }, ... ]

The engine attaches `Transform`, `GlobalTransform`, `Visibility`, `InheritedVisibility` by default.

## Components (Standard Appliers)

These HCL keys map to Bevy components via built‑in appliers:

- Transform = { t = [x,y,z]?, s = [x,y,z]?, r = [x,y,z,w]?, euler = { x,y,z }?, look_at = [x,y,z]? }
- Name = "Text"
- Visibility = "Visible" | "Hidden" | "Inherited"
- StandardMaterialRef = { material = "name" } (applies before mesh)
- MeshRef = { mesh = "name" } => inserts Mesh3d + MeshMaterial3d
- SceneRef = { scene = "name" } => inserts SceneRoot for glTF
- Camera3d = { hdr = bool? }
- DirectionalLight = { illuminance = f64?, shadows = bool? }
- PointLight = { intensity = f64?, range = f64? }

A default `__default` material is ensured if none is specified.

## Selectors

Identify entities to target with actions/conditions:

- { name = "EntityName" }
- { tag = "tagname" }
- { all = true }

## State (Global now; Local planned)

- Global: `vars = { speed = 6.0, hp = 100.0 }` — available to expressions and actions.
- Planned scopes:
  - `state.global { key = value }`
  - `state.tag "Enemies" { hp = 50 }`
  - `state.entity "Player" { stamina = 100 }`
- Planned actions will target scopes via `scope = "global" | "tag" | "entity"` and `targets = Selector` for tag/entity writes.

## Events

- on = { key_pressed = "KeyW" }
- on = { key_held = "KeyW" }
- on = { tick = { every = seconds_f32 } }
- on = { startup = true }
- on = { event = "custom_event" }
- on = { timer = "name" } — Named timers (see SetTimer action)

### Module-defined (custom) events

Modules can establish conventions for event names and emit them from actions. Any scene can listen with `on = { event = "..." }`.

- Emitting:
```hcl
{ emit = { name = "combat.hit", payload = { damage = 12 } } }
```

- Listening:
```hcl
trigger "on_hit" {
  on = { event = "combat.hit" }
  actions = [
    { add_var = { name = "hp", by = -12.0 } },
    { when = { expr = "hp <= 0" } },
    { emit = { name = "combat.death" } }
  ]
}
```

Note: In the current engine, event payloads are numeric maps stored for the frame; direct reads inside expressions are planned. Use `{ eval = { ... } }` and var actions to compose behavior.

## Conditions (when)

Optional list; all must pass to execute actions:

- { any_visible = Selector }
- { all_visible = Selector }
- { not = Condition }
- { expr = "a > 0 && b <= 2" } — Boolean expression (via evalexpr)

Planned intuitive grouping (will compile down to the above):
```hcl
# Any of these is true
when_any = [ { expr = "hp < 30" }, { expr = "god_mode == 1" } ]
# All must be true
when_all = [ { expr = "alive == 1" } ]
# None must be true
when_none = [ { expr = "stunned == 1" } ]
```

## Actions

- { toggle_visibility = { targets = Selector? } }
- { set_visibility = { targets = Selector?, value = "Hidden"|... } }
- { translate = { targets = Selector?, by = [x,y,z] } }
- { translate_axis = { targets = Selector?, vec = [x,y,z], speed_var = "var", use_dt = true } }
- { rotate_euler = { targets = Selector?, by = { x,y,z } } }
- { set_material = { targets = Selector?, material = "name" } }
- { spawn = { prefab = "OptionalPrefab", components = { ... }, parent = Selector? } }
- { despawn = { targets = Selector? } }
- { set_var = { name = "var", value = f64 } }
- { add_var = { name = "var", by = f64 } }
- { mul_var = { name = "var", by = f64 } }
- { emit = { name = "event", payload = { k = f64, ... }? } }
- { eval = { expr = "a+b*2", store_as = "var" } } — Evaluates to f64
- { set_timer = { name = "timer_name", seconds = f32, repeating = bool? } } — Use with on={ timer = "name" }
- { apply = { targets = Selector?, path = "Transform.t"|"Transform.s"|"StandardMaterialRef.material", value = ... } } — Generic setter for common fields

Planned scope‑aware state actions (compile to global actions on current runtime):
```hcl
{ set_var = { name = "speed", value = 7.5, scope = "global" } }
{ add_var = { name = "hp", by = -10, scope = "entity" }, targets = { name = "Player" } }
{ mul_var = { name = "morale", by = 1.1, scope = "tag" }, targets = { tag = "Enemies" } }
```

Note: Actions run in the order they appear. `spawn` merges a prefab then overrides with inline `components` before appliers run.

### Multiple actions per trigger (reactive chains)

A trigger can contain multiple actions that execute sequentially, allowing reactive flows. Example: move, then toggle, then emit an event:
```hcl
trigger "step_and_blink" {
  on = { key_pressed = "KeyD" }
  target = { name = "Player" }
  actions = [
    { translate = { by = [1,0,0] } },
    { toggle_visibility = {} },
    { emit = { name = "player.moved" } }
  ]
}
```

## Expressions

Expressions are evaluated with `evalexpr` against the current variable map.

- Supported: numeric operators, comparisons, logical ops.
- Builtins: no custom functions registered yet (planned). You can compute then store via `{ eval = { expr, store_as } }`.

### Practical usage patterns

- Compute and store for later actions in the same frame:
```hcl
triggers {
  trigger "dash" {
    on = { key_pressed = "Space" }
    actions = [
      { eval = { expr = "speed * 3.0", store_as = "dash_speed" } },
      { translate_axis = { vec = [0,0,-1], speed_var = "dash_speed", use_dt = true } },
      { set_var = { name = "cooldown", value = 1.5 } }
    ]
  }
}
```

- Boolean gating via `when` with `expr`:
```hcl
trigger "move_w" {
  on = { key_held = "KeyW" }
  when = [ { expr = "cooldown <= 0" } ]
  actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed", use_dt = true } } ]
}
```

- Timers plus arithmetic in expressions:
```hcl
triggers {
  trigger "tick_cooldown" { on = { tick = { every = 0.1 } } actions = [ { add_var = { name = "cooldown", by = -0.1 } } ] }
  trigger "clamp_cooldown" { on = { tick = { every = 0.2 } } when = [ { expr = "cooldown < 0" } ] actions = [ { set_var = { name = "cooldown", value = 0.0 } } ] }
}
```

## Timers

- Create: `{ set_timer = { name = "respawn", seconds = 5.0, repeating = true } }`
- Trigger: `on = { timer = "respawn" }`
- Timers tick globally; `set_timer` overwrites by name.

## Event Payloads

`emit` can carry a numeric payload map; the runtime stores it on the event for the frame. Reading payloads directly in conditions/expressions is planned.

## Includes and Modules

- includes = ["scenes/common.hcl"] merges another document: assets, prefabs, entities, triggers, and vars.
- modules/exports: allows namespacing of prefabs/entities/triggers/vars and asset names. Module merging resolves `alias::Name` to prevent collisions.

## Debugging and Overlay

- The runtime logs when HCL compiles, when triggers fire, and each action.
- To enable periodic overlay log line: set `debug_overlay_log` var > 0. The engine logs `HCL: ...` each second with key vars and recent events.
- Optional on‑screen overlay plugin exists behind Cargo feature `hcl_overlay_ui` and renders text using a 2D camera. To enable:
  - In Cargo.toml, add feature `hcl_overlay_ui` to the `bevy-in-app` crate.
  - Build with `--features hcl_overlay_ui`.

## Roadmap (Planned)

- Function registry and stdlib (clamp, lerp, min, max, abs, noise, rng)
- Scoped variables (global/entity/tag) and targeting in expressions
- Generic Get action for reading component fields into vars
- Event payload access in conditions/expressions
- Let‑bindings inside triggers compiled to `eval` + temporary vars
- Grouped conditions (`when_any`, `when_all`, `when_none`) compiled to current condition list
- Selector extensions: nearest/within_radius/raycast
- Cooldown helper built on timers

## Reactive Patterns and Module Events

Think of HCL triggers as a lightweight reactive graph: inputs (keys, timers, custom events) cause actions which may update variables or emit more events.

### Example: Combat module exposing events

A module can define event names and prefabs that emit them. Consumers listen via `on = { event = "..." }`.

Module `combat::core` (conceptual):
```hcl
exports = [ { name = "combat::core", triggers = ["combat_attack"], public = true } ]

trigger "combat_attack" {
  on = { event = "combat.attack" }
  actions = [
    { emit = { name = "combat.hit", payload = { damage = 10 } } },
    { emit = { name = "combat.vfx", payload = { k = 1 } } }
  ]
}
```

Game scene listening:
```hcl
triggers {
  trigger "take_damage" { on = { event = "combat.hit" } actions = [ { add_var = { name = "hp", by = -10.0 } } ] }
  trigger "death" { on = { event = "combat.hit" } when = [ { expr = "hp <= 0" } ] actions = [ { emit = { name = "combat.death" } } ] }
}
```

### Example: Timers and variables for reactive loops
```hcl
vars = { blink = 0, speed = 6.0 }
triggers {
  trigger "heartbeat" { on = { tick = { every = 0.5 } } actions = [ { add_var = { name = "blink", by = 1.0 } }, { emit = { name = "ui.blink" } } ] }
  trigger "blink_ui" { on = { event = "ui.blink" } target = { tag = "blinkers" } actions = [ { toggle_visibility = {} } ] }
}
```

## Reference Example: Minimal Scene

```hcl
assets { mesh "cube" { builtin = "cube" } material "hero" { pbr = { base_color = "#3aa7ff" } } }
vars = { speed = 6.0 }
prefab "Hero" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "hero" }, Transform = { s = [1,1,1] } } }
entity "root" { children = [ { name = "Player", include = ["Hero"], components = { Transform = { t = [0,1,0] } } } ] }
triggers {
  trigger "move_w" { on = { key_held = "KeyW" } target = { name = "Player" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed" } } ] }
  trigger "spawn_enemy" { on = { tick = { every = 5.0 } } actions = [ { spawn = { components = { Name = "Enemy", MeshRef = { mesh = "cube" }, Transform = { t = [4,1,0] } } } } ] }
}
```

### Advanced Example: Reactive multi-action sequence
```hcl
assets {
  mesh "cube" { builtin = "cube" }
  material "hero" { pbr = { base_color = "#3aa7ff", emissive = { r = 0.1, g = 0.1, b = 0.2 } } }
}
vars = { speed = 6.0, cooldown = 0.0, hp = 100.0 }
prefab "Hero" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "hero" }, Transform = { s = [1,1,1] } } }
entity "root" { children = [ { name = "Player", include = ["Hero"], components = { Transform = { t = [0,1,0] } } } ] }
triggers {
  trigger "move" {
    on = { key_held = "KeyW" }
    target = { name = "Player" }
    when = [ { expr = "cooldown <= 0" } ]
    actions = [
      { translate_axis = { vec = [0,0,-1], speed_var = "speed", use_dt = true } },
      { add_var = { name = "cooldown", by = -0.016 } } # simulate decay when moving
    ]
  }
  trigger "dash" {
    on = { key_pressed = "Space" }
    target = { name = "Player" }
    actions = [
      { eval = { expr = "speed * 3.0", store_as = "dash_speed" } },
      { translate_axis = { vec = [0,0,-1], speed_var = "dash_speed", use_dt = true } },
      { set_var = { name = "cooldown", value = 1.0 } },
      { emit = { name = "player.dashed" } }
    ]
  }
  trigger "cooldown_tick" { on = { tick = { every = 0.1 } } actions = [ { add_var = { name = "cooldown", by = -0.1 } } ] }
  trigger "clamp_cd" { on = { tick = { every = 0.2 } } when = [ { expr = "cooldown < 0" } ] actions = [ { set_var = { name = "cooldown", value = 0.0 } } ] }
  trigger "take_damage" { on = { event = "combat.hit" } actions = [ { add_var = { name = "hp", by = -10.0 } } ] }
  trigger "die" { on = { event = "combat.hit" } when = [ { expr = "hp <= 0" } ] actions = [ { despawn = { targets = { name = "Player" } } } ] }
}
```

## Execution Notes (engine implementation)

Linking the spec to actionable engine tasks. These will be implemented in phases without breaking existing scenes.

- Expressions
  - Precompile expressions on load; evaluate against a typed context (vars, dt, payloads).
  - Add numeric builtins: `clamp, lerp, abs, min, max, deg2rad, rad2deg, pi, time, rng, rng_range, var(scope,name)`.
  - String interpolation `${...}` for UI/name fields (evaluate on apply, not every frame).

- Conditions (ECAS)
  - Keep current `when` forms; add sugar `when_any/when_all/when_none` compiled to current constructs.
  - Provide `cooldown` shorthand on triggers (compiles to SetTimer + guards) [optional].

- State
  - Implement `VarScopes` (global/tag/entity) with `HclVars` component for entity local state.
  - Extend `set_var/add_var/mul_var` with `scope` and `targets` for tag/entity.

- Events and networking
  - Trigger metadata: `authority` and `channel` annotations (docs first, then code).
  - Custom events remain numeric‑payload; add payload access in expressions later.

- Performance and ECS alignment
  - Selector indices for Name/Tag; avoid world scans.
  - Typed appliers to minimize JSON walking.
  - Deterministic RNG for reproducibility (seeded).

- Tooling
  - hclfmt + lint: unknown keys, unresolved includes/prefabs, cyclic includes.
  - Editor diagnostics: show parse/compile errors with line/column; trigger inspector panel.

- Tests
  - Golden HCL fixtures; parser/spawn/runtime tests (movement, timers, emit/vars).
  - Benchmarks for trigger dispatch and expression evaluation. 

## Dynamic Variables & Runtime Binding (Terraform-inspired)

We support simple, declarative variables with defaults and runtime overrides, plus computed values. Think: inputs (with defaults), locals (computed), and expressions — but all numeric and safe.

Concepts (current and planned notation, backwards compatible):
- Inputs (globals today):
```hcl
vars = { speed = 6.0, jump = 12.0, hp = 100.0 }
```
- Computed values (today via eval action; planned sugar as `locals` compiled to eval on startup):
```hcl
# Planned sugar (compiles to a Startup trigger with eval actions)
locals = { dash = "speed * 3.0", run = "speed * 1.5" }
```
- Interpolation (planned) for strings only: `"Hero ${hp}"` evaluated on apply.
- Precedence (planned): inline > locals > vars (defaults); modules may bring their own defaults, overridden by the scene’s `vars`.

Simple runtime binding pattern (works today):
```hcl
triggers {
  trigger "bind_locals" { on = { startup = true } actions = [
    { eval = { expr = "speed * 1.5", store_as = "run" } },
    { eval = { expr = "speed * 3.0", store_as = "dash" } }
  ] }
}
```

Profiles/overrides (planned DX):
- CLI can pass overrides at serve time, e.g. `vysma serve --set speed=8.0 --set hp=150`.
- Editor UI quick sliders map to global vars.

Guidelines:
- Keep vars numeric; use component fields for complex data.
- Compute once and reuse inside actions to avoid repeated math.

## Module Composition (Declarative)

Modules should compose by: import → alias → include/override — no custom functions layer.

- Import with alias:
```hcl
modules = [ { name = "alice::platformer_core", alias = "core" } ]
```
- Use exported prefabs/entities by alias:
```hcl
entity "root" {
  children = [
    { name = "Camera", include = ["core::Camera2D"] },
    { name = "Player", include = ["core::Player"], components = { Transform = { t = [0,1,0] } } }
  ]
}
```
- Override locally, never mutate the module: add `components` or extra triggers in your scene.
- Namespaces keep it kid-friendly: everything from a module is `alias::Thing`. 

## Persistent State & Backend Compute (Server authority)

Make some state durable in the backend while keeping HCL declarative. You describe models, bind them to entities or globals, and use simple actions to load/save/query. All persistence runs on the server with authority.

### Models (schema hints)

Declare models your game needs. The CLI can provision them in Appwrite (planned). Fields are hints; engine treats them as typed columns when possible and falls back to JSON.

```hcl
models {
  model "Player" { key = "id", fields = { hp = 100.0, xp = 0.0 } }
  model "Leaderboard" { key = "id", fields = { score = 0.0, name = 0.0 } } # name stored as string by backend; HCL only reads numeric fields into vars
}
```

Notes:
- `key`: primary key field name used for load/save.
- Numeric fields map to HCL vars. Strings are supported for UI bindings but not used in numeric expressions.

### Bindings (auto sync of numeric fields)

Bind a model to a scope and selector so you can load/save a record with a single action.

```hcl
persist_bind "PlayerState" {
  model = "Player"
  id    = "${player_id}"        # or a constant like "p1"
  scope = "entity"               # global|tag|entity
  targets = { name = "Player" }  # used for entity/tag scopes
  map = { hp = "hp", xp = "xp" } # model.field -> var name in scope
}
```

### Actions (server)

- `{ persist_load = { binding = "PlayerState" } }` — fetches doc by `id` and writes mapped numeric fields into vars in the chosen scope.
- `{ persist_save = { binding = "PlayerState" } }` — reads mapped vars and upserts them into the model by `id`.
- `{ persist_set = { model = "Player", id = "p1", data = { hp = 90.0, xp = 10.0 }, merge = true } }` — direct upsert without a binding.
- `{ persist_query = { model = "Leaderboard", where = [ { field = "mode", op = "==", value = 1.0 } ], order_by = [ { field = "score", dir = "desc" } ], limit = 10, store_as = "lb_" } }`
  - Stores numeric fields as namespaced global vars: `lb_0_score`, `lb_1_score`, ... (strings available to UI binding only).

All persist actions are executed on the server even if authored in a client scene. Use `authority = "server"` on triggers for clarity.

### Backend compute (jobs)

Server‑side recurring logic can be declared as triggers with `authority = "server"` and `on = { tick = { every = X } }` or custom events (e.g., `combat.death`). These can read/write persistent state.

```hcl
trigger "award_xp" {
  authority = "server"
  on = { event = "combat.death" }
  actions = [
    { add_var = { name = "xp", by = 10.0, scope = "entity" }, targets = { name = "Player" } },
    { persist_save = { binding = "PlayerState" } }
  ]
}
```

### Example: load on join, save on quit

```hcl
# At startup, load player vars from DB into the Player entity scope
trigger "load_player" { authority = "server" on = { startup = true } actions = [ { persist_load = { binding = "PlayerState" } } ] }
# On exit or checkpoint, save
trigger "save_player" { authority = "server" on = { event = "checkpoint" } actions = [ { persist_save = { binding = "PlayerState" } } ] }
```

Security:
- Server enforces auth; bindings respect project membership.
- Client can’t directly mutate DB; only via server-authoritative actions. 

## MMO Hooks (events/actions/conditions)

For large online worlds, these declarative hooks extend HCL (executed on server authority unless noted):

Events (listen with `on = { event = "..." }`)
- presence.join, presence.leave
- chat.message (payload: { channel, sender_id, text? })
- party.match_found, instance.created, instance.transferred
- guild.invite, guild.join, guild.leave

Actions
- send_chat { channel = "global|party|guild", text = "..." }
- match_queue { mode = "pvp|raid", size = 5 }
- instance_create { template = "dungeon01", store_as = "inst_id" }
- instance_transfer { targets = Selector, instance_id = "...", spawn = [x,y,z] }
- teleport { targets = Selector, zone = "east_1", pos = [x,y,z] }
- guild_invite { to_id = "user123" }, guild_accept { invite_id = "..." }

Conditions
- in_aoi { a = Selector, b = Selector, r = 50.0 }
- population_lt { zone = "east_1", max = 500 }
- has_permission { role = "mod|gm", action = "kick|ban|announce" }

Notes
- These map to backend services (presence/chat/matchmaking/instances) and are delivered as feature modules. HCL remains declarative; Rust implements the heavy lifting. 

## Scopes and Two‑way Bindings (local/entity/tag/global)

Make state easy to read/write across triggers without scripting. Bind component fields to vars with clear direction and minimal runtime cost.

### Scopes
- Global: `vars = { hp = 100.0, score = 0.0 }`
- Entity: per‑entity numeric vars held in a component (engine: `HclVars`)
- Tag: shared vars per tag (engine: resource keyed by tag)
- Local (transient): ephemeral vars attached to an entity with optional TTL; great for cooldowns, impulses

Declare/mark transient vars via actions (current engine) or planned sugar:
```hcl
# Transient var with TTL (planned sugar; compiles to set_var + expiry timer)
transient { entity "Player" { knockback = { value = 1.0, ttl = 0.5 } } }
```

### Bindings (change‑driven)

Two binding forms are supported; the engine executes them on change with throttling. All forms are numeric‑only in compute path; strings are for UI.

1) Expression binding (one‑way)
```hcl
bindings = [
  { targets = { name = "HpText" }, path = "Text.value", expr = "\"HP: \" + hp", on = "tick", throttle = 0.1 }
]
```

2) Var↔Field binding (two‑way)
```hcl
bindings = [
  { targets = { name = "Player" }, path = "Transform.t[1]", var = "jump_height", scope = "entity", dir = "inout", on = "change", epsilon = 0.001, throttle = 0.02, priority = 0 }
]
```
- `dir`: "in" (field→var), "out" (var→field), "inout" (both ways with conflict resolution)
- `on`: "change" (default; engine uses change detection), or "tick" (avoid unless necessary), or `event = "..."` (planned)
- `epsilon`: minimum delta to consider a change (floats)
- `throttle`: minimum seconds between updates for this binding
- `priority`: tie‑breaker when multiple bindings write the same field in a frame

Batch map binding (declare many at once):
```hcl
bindings = [
  { targets = { name = "Player" }, dir = "out", scope = "entity", map = [
      { path = "Transform.t[0]", var = "x" },
      { path = "Transform.t[1]", var = "y" },
      { path = "Transform.t[2]", var = "z" }
  ] }
]
```

### Conflict resolution and loops
- Single writer per frame: last‑writer‑wins by `priority` then declaration order.
- `inout` prevents loops via edge marking: a field update that originated from the paired var within the throttle window is ignored until new input arrives (engine keeps a short history of origins per binding).
- Authority applies: server wins over client on replicated fields/vars.

### Performance (engine semantics)
- Change‑driven: we subscribe to component `Changed<T>` and scoped var deltas; no world scans.
- Per‑binding state caches last value, last write time, and origin flag; epsilon avoids flicker.
- Batching: batch evaluate bindings per entity to minimize lookups; vec paths are pre‑indexed.

### Examples
- Local impulse into global var and field (two‑way):
```hcl
# Player presses Space: set a local impulse, the binding drives both entity var and Transform.y
trigger "impulse" { on = { key_pressed = "Space" } actions = [ { add_var = { name = "impulse", by = 1.0, scope = "entity" }, targets = { name = "Player" } } ] }
bindings = [ { targets = { name = "Player" }, path = "Transform.t[1]", var = "impulse", scope = "entity", dir = "inout", on = "change", epsilon = 0.001, throttle = 0.016 } ]
```

- Tag scoped buff applied to all enemies:
```hcl
trigger "rage" { on = { event = "combat.rage" } actions = [ { mul_var = { name = "speed", by = 1.2, scope = "tag" }, targets = { tag = "Enemies" } } ] }
bindings = [ { targets = { tag = "Enemies" }, path = "StandardMaterialRef.material", expr = "speed > 1.0 ? \"red\" : \"enemy\"", on = "tick", throttle = 0.2 } ]
```

- UI slider to entity var (client) then persisted (server):
```hcl
# Client local: slider writes entity var via binding (dir=out)
bindings = [ { targets = { name = "Slider" }, path = "UiSlider.value", var = "sfx", scope = "entity", dir = "out", on = "change", throttle = 0.05 } ]
# Server: save on checkpoint
trigger "save_sfx" { authority = "server" on = { event = "checkpoint" } actions = [ { persist_save = { binding = "PlayerState" } } ] }
``` 

## Beginner‑friendly Syntax (Sugar)

For a gentle learning curve, you can write short, readable sugar that compiles to the canonical HCL shown above. Start simple; add detail only when needed.

Examples (sugar → meaning):

- Move while holding W
```hcl
# Sugar
rule "move_w" when key "W" do move -z speed

# Canonical equivalent
trigger "move_w" { on = { key_held = "KeyW" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed", use_dt = true } } ] }
```

- Dash on Space with cooldown and local let
```hcl
# Sugar
rule "dash" when press "Space" do seq {
  let dash = speed * 3
  move -z dash
  set cooldown = 1s
}

# Canonical equivalent
trigger "dash" {
  on = { key_pressed = "Space" }
  actions = [
    { eval = { expr = "speed * 3", store_as = "dash" } },
    { translate_axis = { vec = [0,0,-1], speed_var = "dash", use_dt = true } },
    { set_var = { name = "cooldown", value = 1.0 } }
  ]
}
```

- Bind text and a value two‑ways
```hcl
# Sugar
bind HpText.text <- "HP: " + hp every 0.1s
bind Player.y <-> var:jump_height epsilon 0.001

# Canonical equivalent
bindings = [
  { targets = { name = "HpText" }, path = "Text.value", expr = "\"HP: \" + hp", on = "tick", throttle = 0.1 },
  { targets = { name = "Player" }, path = "Transform.t[1]", var = "jump_height", scope = "entity", dir = "inout", on = "change", epsilon = 0.001 }
]
```

- Entity‑local rules (sugar nests rules under the entity)
```hcl
entity "Player" {
  at [0,1,0]
  has "Hero"
  rules {
    when key "W" do move -z speed
    when press "Space" do seq { let dash = speed*3; move -z dash; set cooldown = 1s }
  }
}
# Compiles to entity + triggers with target={ name = "Player" }
```

Sugar keywords
- `rule NAME when key|press KEY do ...` → trigger with key_held/key_pressed
- `move ±x|±y|±z SPEED` → translate_axis with unit vector and speed_var
- `set NAME = VALUE` → set_var
- `add NAME BY VALUE` → add_var
- `seq { ... }` → sequence; `delay 0.2s`; `every 0.1s` → tick throttle for bindings
- `bind A.field <- EXPR` → one‑way expr binding; `bind A.field <-> var:NAME` → inout var/field
- `at [x,y,z]` → Transform.t; `has "Prefab"` → include

Notes
- Units: `1s` → 1.0 seconds (compiled to numbers); `10ms` → 0.01.
- Sugar is optional. You can freely mix sugar and canonical HCL in one file.
- The engine compiles sugar at load time; errors show canonical line/column references with sugar context when possible. 

## EZ Mode (Kids‑friendly syntax)

Write game logic like simple sentences. The engine compiles EZ Mode → sugar → canonical HCL. You can mix EZ with normal HCL in one file.

Rules
- Case‑insensitive keywords; minimal punctuation.
- `When ...:` starts a rule. Indent actions under it.
- Directions: forward/back/left/right/up/down map to unit vectors.
- Times: `1s`, `200ms`.
- Variables: simple `set name = number`.

Direction words
- forward = [0,0,-1]
- back = [0,0,1]
- left = [-1,0,0]
- right = [1,0,0]
- up = [0,1,0]
- down = [0,-1,0]

Mini examples (EZ → meaning)
```hcl
# Place and vars
Place Player at [0,1,0] has Hero
Set speed = 6

# Move while holding W
When key W:
  Move Player forward by speed

# Dash on Space with cooldown
When press Space:
  Let dash = speed*3
  Move Player forward by dash
  Set cooldown = 1s

# Show HP text every 0.1s
Every 0.1s:
  Show "HP: {hp}" on HpText

# Two‑way bind Player y with jump_height
Bind Player.y <-> jump_height epsilon 0.001
```

What it compiles to (conceptually)
- `Place` → `entity` with `Transform.t` and `include` prefab
- `When key W:` → `trigger on={key_held}` with `translate_axis`
- `Move NAME forward by X` → `translate_axis vec=[0,0,-1] speed_var=X`
- `Every 0.1s:` → `trigger on={tick}`
- `Show "text" on Name` → `binding expr → Text.value`
- `Bind A.y <-> var` → inout var/field binding with epsilon

10‑line game (EZ)
```hcl
Place Player at [0,1,0] has Hero
Place Camera at [0,3,6]
Set speed = 6
Set hp = 100

When key W:
  Move Player forward by speed

Every 0.1s:
  Show "HP: {hp}" on HpText
```

Tips
- Start with EZ sentences. As you need more power, switch to sugar or canonical blocks for that part.
- Editor can show the compiled form so you learn by seeing both. 

## Extending EZ Mode & Sugar via Modules

Modules can add their own friendly words and short forms so authors can write simple sentences that map to the module’s prefabs, events, and actions. The engine compiles module EZ/Sugar → canonical HCL at load time.

Design
- Pattern files: modules may ship a `syntax/ez.hcl` (or `syntax/sugar.hcl`) describing patterns and expansions.
- Namespacing: patterns are namespaced by module alias; unqualified patterns from multiple modules use priority and must not collide.
- Safety: patterns can only expand to canonical HCL; no code execution.
- Versioning: each pattern file has a `version = 1` to allow future evolution.

Pattern kinds
- Rule pattern: transforms an EZ/Sugar rule into a canonical trigger or actions
- Verb pattern: maps short imperative like `Jump Player` to one or more canonical actions
- Binding pattern: maps `Bind A.x <-> var` short form to binding descriptors

Example pattern file (`syntax/ez.hcl` inside module)
```hcl
version = 1
module = "platformer_core"

patterns = [
  # Verb: Jump NAME by VAR|NUMBER → translate + cooldown sugar
  { kind = "verb", name = "Jump", expands_to = [
      { translate_axis = { vec = [0,1,0], speed_var = "$by", use_dt = false }, targets = { name = "$name" } }
    ]
  },

  # Rule: When land NAME: … → listen to a module event
  { kind = "rule", match = "When land $name:", expands_to = { on = { event = "platformer.land" }, target = { name = "$name" } } },

  # Binding: Show "text {var}" on NAME
  { kind = "binding", match = "Show \"$text\" on $name", expands_to = { expr = "$text", path = "Text.value", targets = { name = "$name" }, on = "tick", throttle = 0.1 } }
]
```

Usage in a scene
```hcl
modules = [ { name = "alice::platformer_core", alias = "core" } ]

# EZ sentences using module verbs/rules
When land Player:
  Jump Player by 8
  Show "Jump!" on HudText
```

Precedence and conflicts
- Patterns are applied in this order: scene-local overrides → module alias-qualified (e.g., `core:Jump`) → unqualified module patterns by priority → built-in.
- Colliding unqualified patterns are rejected with a clear error listing the modules and how to disambiguate (`core:Jump`).

Performance
- Pattern expansion occurs at load time; resulting HCL is canonical and executed normally (no runtime penalty).

Authoring guidance
- Keep verbs short and intuitive. Prefer mapping verbs to one or a few canonical actions.
- Expose module events with natural names so rule patterns can use them (e.g., `When land ...`). 