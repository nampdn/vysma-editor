### HCL EZ Mode (Kids‑friendly) — Guide

Goal: let beginners write game logic as simple sentences. The engine compiles EZ → Sugar → Canonical HCL. You can mix EZ with normal HCL.

Core ideas
- Sentences start with verbs: Place, Set, When, Every, Show, Move, Let, Bind.
- Minimal punctuation; case‑insensitive keywords.
- Directions are words (forward/back/left/right/up/down).
- Times: `1s`, `200ms`.

Mini grammar (simplified)
- Place NAME at [x,y,z] (has PREFAB)?
- Set VAR = NUMBER
- When key KEY:
  - INDENT actions...
- When press KEY:
  - INDENT actions...
- Every TIME:
  - INDENT actions...
- Move NAME (forward|back|left|right|up|down) by VAR|NUMBER
- Let NAME = EXPR
- Show "TEXT {var}" on NAME
- Bind NAME.(x|y|z|text) <-> VAR (epsilon NUMBER)?

Direction map
- forward=[0,0,-1], back=[0,0,1], left=[-1,0,0], right=[1,0,0], up=[0,1,0], down=[0,-1,0]

Examples
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

Two‑way binding example
```hcl
Bind Player.y <-> jump_height epsilon 0.001
```

What it compiles to (concept)
- Place → entity with Transform.t and include prefab
- When key → trigger on key_held
- Move → translate_axis with unit vector and speed_var
- Every → trigger on tick
- Show → bindings expr → Text.value
- Bind a.y <-> var → inout binding with epsilon

Tips
- Start in EZ Mode, peek at the compiled Sugar/Canonical view to learn.
- Keep names simple and unique. Use `has "Prefab"` to reuse prefabs.
- You can use `Let` for quick math: `Let dash = speed*3` then use `dash`. 