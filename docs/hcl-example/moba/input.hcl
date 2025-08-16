# Client input wiring (emits to server; predicted visuals optional)

vars = { speed = 6.0 }

triggers {
  # Movement
  trigger "input_w" { authority = "client" channel = "unreliable" on = { key_held = "KeyW" } actions = [ { emit = { name = "input.move_w" } }, { translate_axis = { vec = [0,0,-1], speed_var = "speed", use_dt = true } } ] }
  trigger "input_s" { authority = "client" channel = "unreliable" on = { key_held = "KeyS" } actions = [ { emit = { name = "input.move_s" } }, { translate_axis = { vec = [0,0, 1], speed_var = "speed", use_dt = true } } ] }
  trigger "input_a" { authority = "client" channel = "unreliable" on = { key_held = "KeyA" } actions = [ { emit = { name = "input.move_a" } }, { translate_axis = { vec = [-1,0,0], speed_var = "speed", use_dt = true } } ] }
  trigger "input_d" { authority = "client" channel = "unreliable" on = { key_held = "KeyD" } actions = [ { emit = { name = "input.move_d" } }, { translate_axis = { vec = [ 1,0,0], speed_var = "speed", use_dt = true } } ] }

  # Dash
  trigger "input_dash" { authority = "client" channel = "reliable" on = { key_pressed = "Space" } actions = [
    { emit = { name = "input.dash" } },
    { sequence = [
        { tween = { path = "Transform.t", to = [0,1.2,0], seconds = 0.08, easing = "quad_out" } },
        { tween = { path = "Transform.t", to = [0,1.0,0], seconds = 0.12, easing = "quad_in" } }
      ] }
  ] }

  # Abilities QWER
  trigger "input_q" { authority = "client" channel = "reliable" on = { key_pressed = "KeyQ" } actions = [ { emit = { name = "ability.q" } } ] }
  trigger "input_w_ability" { authority = "client" channel = "reliable" on = { key_pressed = "KeyW" } actions = [ { emit = { name = "ability.w" } } ] }
  trigger "input_e" { authority = "client" channel = "reliable" on = { key_pressed = "KeyE" } actions = [ { emit = { name = "ability.e" } } ] }
  trigger "input_r" { authority = "client" channel = "reliable" on = { key_pressed = "KeyR" } actions = [ { emit = { name = "ability.r" } } ] }

  # Map interactions: ping, place ward, shop
  trigger "input_ping" { authority = "client" channel = "reliable" on = { key_pressed = "KeyG" } actions = [ { emit = { name = "input.ping" } } ] }
  trigger "input_place_ward" { authority = "client" channel = "reliable" on = { key_pressed = "KeyV" } actions = [ { emit = { name = "input.place_ward" } } ] }
  trigger "input_shop_speed" { authority = "client" channel = "reliable" on = { key_pressed = "KeyB" } actions = [ { emit = { name = "input.shop_speed" } } ] }
} 