# Vysma HCL Specification and Authoring Guide

This document is the developer‑facing reference for writing HCL scenes and modules that drive the Bevy‑based engine at runtime. It covers the schema, supported features, and practical authoring tips for rapid iteration.

HCL files live under `assets/` (e.g., `assets/scenes/moba_game.hcl`). The engine hot‑reloads edited files.

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
- image "name" { file = "textures/.." }
- gltf "name" { file = "mesh/...glb", node = "NodeName"? }

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

## Variables

`vars = { key = number }` declares global f64 variables. The engine also injects `dt` each frame (seconds as f64). Variables are accessible to `eval` expressions and actions.

## Events

- on = { key_pressed = "KeyW" }
- on = { key_held = "KeyW" }
- on = { tick = { every = seconds_f32 } }
- on = { startup = true }
- on = { event = "custom_event" }
- on = { timer = "name" } — Named timers (see SetTimer action)

## Conditions (when)

Optional list; all must pass to execute actions:

- { any_visible = Selector }
- { all_visible = Selector }
- { not = Condition }
- { expr = "a > 0 && b <= 2" } — Boolean expression (via evalexpr)

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

Note: Actions run in the order they appear. `spawn` merges a prefab then overrides with inline `components` before appliers run.

## Expressions

Expressions are evaluated with `evalexpr` against the current variable map.

- Supported: numeric operators, comparisons, logical ops.
- Builtins: no custom functions registered yet (planned). You can compute then store via `{ eval = { expr, store_as } }`.

## Timers

- Create: `{ set_timer = { name = "respawn", seconds = 5.0, repeating = true } }`
- Trigger: `on = { timer = "respawn" }`
- Timers tick globally; `set_timer` overwrites by name.

## Event Payloads

`emit` can carry a numeric payload map; the runtime stores it on the event for the frame. Reading payloads directly in conditions/expressions is planned.

## Includes and Modules

- includes = ["scenes/common.hcl"] merges another document: assets, prefabs, entities, triggers, and vars.
- modules/exports (early): allows namespacing of prefabs/entities/triggers/vars and asset names. Module merging resolves `alias::Name` to prevent collisions.

## Debugging and Overlay

- The runtime logs when HCL compiles, when triggers fire, and each action.
- To enable periodic overlay log line: set `debug_overlay_log` var > 0. The engine logs `HCL: ...` each second with key vars and recent events.
- Optional on‑screen overlay plugin exists behind Cargo feature `hcl_overlay_ui` and renders text using a 2D camera. To enable:
  - In Cargo.toml, add feature `hcl_overlay_ui` to the `bevy-in-app` crate.
  - Build with `--features hcl_overlay_ui`.

## Authoring Patterns

- Movement: `translate_axis` with `speed_var` and `use_dt = true`.
- Combat gating: modify vars (e.g., `enemy_hp`), then `when = [{ expr = "enemy_hp <= 0" }]` to drive despawn.
- Respawn/Waves: periodic `tick` + conditional spawn when no entities visible (`when = [{ not = { all_visible = { name = "Enemy" } } }]`).
- Chaining logic: `emit` an event, then a separate trigger with `on = { event = "..." }` applies further actions.
- Apply on demand: use `apply` to write back into Transform or material without creating new appliers.

## Roadmap (Planned)

- Function registry and stdlib (clamp, lerp, min, max, abs, noise, rng)
- Scoped variables (global/entity/tag) and targeting in expressions
- Generic Get action for reading component fields into vars
- Event payload access in conditions/expressions
- Selector extensions: nearest/within_radius/raycast
- Cooldown helper built on timers

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