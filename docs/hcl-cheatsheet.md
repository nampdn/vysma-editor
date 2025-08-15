### Vysma HCL Cheatsheet (1 page)

Basics
- assets: declare meshes/materials/images/gltf
- prefab: reusable components by name
- entity: place things in the world; `has` a prefab; `at [x,y,z]`
- vars: numbers you can read/write
- triggers (rules): event → actions

Sugar (left) → Canonical (right)

- Key hold move
```hcl
rule "move_w" when key "W" do move -z speed
# → trigger { on={ key_held="KeyW" } actions=[ { translate_axis={ vec=[0,0,-1], speed_var="speed", use_dt=true } } ] }
```

- Press dash
```hcl
rule "dash" when press "Space" do seq { let dash=speed*3; move -z dash; set cooldown=1s }
# → trigger { on={ key_pressed="Space" } actions=[ eval{speed*3->dash}, translate_axis(vec=[0,0,-1],speed_var="dash"), set_var(cooldown=1) ] }
```

- Bind text and y position
```hcl
bind HpText.text <- "HP: " + hp every 0.1s
bind Player.y <-> var:jump_height epsilon 0.001
# → bindings=[ {targets={name="HpText"}, path="Text.value", expr="\"HP: \" + hp", on="tick", throttle=0.1}, {targets={name="Player"}, path="Transform.t[1]", var="jump_height", scope="entity", dir="inout", on="change", epsilon=0.001} ]
```

Common events
- key "W" (hold), press "Space", startup, every 0.1s, timer "name", event "custom"

Common actions
- move ±x|±y|±z SPEED
- set NAME=VALUE, add NAME BY VALUE
- spawn { prefab: "Name", at:[x,y,z] }
- play_audio "Clip", tween PATH to VALUE in 0.2s ease "quad_out"

Entities
```hcl
entity "Player" {
  at [0,1,0]
  has "Hero"
  rules { when key "W" do move -z speed }
}
```

Patterns
- Cooldown: `set cooldown=1s` then action with `when cooldown<=0`
- UI text: `bind HpText.text <- "HP: "+hp every 0.1s`
- Sequence: `seq { tween Transform.t to [0,1.2,0] in 0.1s; delay 0.1s; tween Transform.t to [0,1,0] in 0.15s }`

Notes
- Units: `1s`=1.0, `10ms`=0.01
- Sugar is optional; you can mix sugar and canonical
- Keep names unique; use modules via `alias::Thing` 