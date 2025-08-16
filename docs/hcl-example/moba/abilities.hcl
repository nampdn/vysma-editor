# MOBA Abilities (Client‑side input + visuals; server validates effects)

vars = { q_cd = 0.0, w_cd = 0.0, e_cd = 0.0, r_cd = 0.0 }

# UI: bind cooldown text (demo expr binding)
bindings = [
  { targets = { name = "QText" }, path = "Text.value", expr = "q_cd>0 ? \"Q: \"+q_cd : \"Q\"", on = "tick", throttle = 0.1 },
  { targets = { name = "WText" }, path = "Text.value", expr = "w_cd>0 ? \"W: \"+w_cd : \"W\"", on = "tick", throttle = 0.1 },
  { targets = { name = "EText" }, path = "Text.value", expr = "e_cd>0 ? \"E: \"+e_cd : \"E\"", on = "tick", throttle = 0.1 },
  { targets = { name = "RText" }, path = "Text.value", expr = "r_cd>0 ? \"R: \"+r_cd : \"R\"", on = "tick", throttle = 0.1 }
]

triggers {
  # Q: Projectile skillshot forward
  trigger "cast_q" {
    on = { key_pressed = "KeyQ" }
    when = [ { expr = "q_cd <= 0" } ]
    actions = [
      { set_var = { name = "q_cd", value = 6.0 } },
      { spawn = { prefab = "core::Projectile", components = { Name = "QProj" } } },
      { sequence = [
          { tween = { path = "Transform.t", to = [0,1, -12], seconds = 0.4, easing = "linear" } },
          { despawn = { targets = { name = "QProj" } } }
        ]
      },
      { emit = { name = "ability.q", payload = { power = 40 } } }
    ]
  }

  # W: Self buff (tag scope)
  trigger "cast_w" {
    on = { key_pressed = "KeyW" }
    when = [ { expr = "w_cd <= 0" } ]
    actions = [
      { set_var = { name = "w_cd", value = 10.0 } },
      { mul_var = { name = "speed", by = 1.4 } },
      { set_timer = { name = "w_buff", seconds = 4.0, repeating = false } }
    ]
  }
  trigger "w_end" { on = { timer = "w_buff" } actions = [ { mul_var = { name = "speed", by = 1.0/1.4 } } ] }

  # E: Ground AOE at cursor (visual only here; server applies damage)
  trigger "cast_e" {
    on = { key_pressed = "KeyE" }
    when = [ { expr = "e_cd <= 0" } ]
    actions = [
      { set_var = { name = "e_cd", value = 8.0 } },
      { spawn = { prefab = "core::AoE", components = { Name = "EIndicator" } } },
      { tween = { path = "Transform.s", to = [3,0.1,3], seconds = 0.3, easing = "quad_out" } },
      { despawn = { targets = { name = "EIndicator" } } },
      { emit = { name = "ability.e", payload = { radius = 3, power = 30 } } }
    ]
  }

  # R: Ultimate (long cooldown)
  trigger "cast_r" {
    on = { key_pressed = "KeyR" }
    when = [ { expr = "r_cd <= 0" } ]
    actions = [
      { set_var = { name = "r_cd", value = 60.0 } },
      { emit = { name = "ability.r", payload = { power = 200 } } }
    ]
  }

  # Cooldown ticks (shared cadence)
  trigger "cd_tick" { on = { tick = { every = 0.2 } } actions = [
    { add_var = { name = "q_cd", by = -0.2 } },
    { add_var = { name = "w_cd", by = -0.2 } },
    { add_var = { name = "e_cd", by = -0.2 } },
    { add_var = { name = "r_cd", by = -0.2 } }
  ] }
  trigger "cd_clamp" { on = { tick = { every = 0.5 } } actions = [
    { set_var = { name = "q_cd", value = 0.0 } },
    { set_var = { name = "w_cd", value = 0.0 } },
    { set_var = { name = "e_cd", value = 0.0 } },
    { set_var = { name = "r_cd", value = 0.0 } }
  ] when = [ { expr = "q_cd<0 || w_cd<0 || e_cd<0 || r_cd<0" } ] }
} 