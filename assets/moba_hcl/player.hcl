// Player prefab and scene demonstrating variables, hot reload, and movement

assets {
  mesh "cube"  { builtin = "cube" }
  material "blue" { pbr = { base_color = "#3aa7ff", metallic = 0.0, roughness = 0.7 } }
}

// default variables
vars = { speed = 3.0 }

prefab "PlayerPrefab" {
  components = {
    MeshRef = { mesh = "cube" },
    StandardMaterialRef = { material = "blue" },
    Transform = { s = [0.8, 0.8, 0.8] }
  }
}

entity "root" {
  components = { Name = "Root" }
  children = [
    { name = "Ground", components = { MeshRef = { mesh = "cube" }, Transform = { s = [20, 0.1, 20] } } },
    { name = "Player", include = ["PlayerPrefab"], persist_key = "player", components = { Transform = { t = [0, 0.6, 0] } } },
    { name = "Sun", components = { DirectionalLight = { illuminance = 60000.0, shadows = true }, Transform = { euler = { x = -60, y = 45, z = 0 } } } },
    { name = "Camera", components = { Camera3d = { hdr = true }, Transform = { t = [6, 5, 12], look_at = [0, 0.5, 0] } } }
  ]
}

// Controls: WASD movement using speed * dt along axes
triggers {
  trigger "move_forward" {
    on = { key_held = "KeyW" }
    target = { name = "Player" }
    actions = [
      { translate_axis = { vec = [0, 0, -1], speed_var = "speed" } }
    ]
  }
  trigger "move_back" {
    on = { key_held = "KeyS" }
    target = { name = "Player" }
    actions = [
      { translate_axis = { vec = [0, 0, 1], speed_var = "speed" } }
    ]
  }
  trigger "move_left" {
    on = { key_held = "KeyA" }
    target = { name = "Player" }
    actions = [
      { translate_axis = { vec = [-1, 0, 0], speed_var = "speed" } }
    ]
  }
  trigger "move_right" {
    on = { key_held = "KeyD" }
    target = { name = "Player" }
    actions = [
      { translate_axis = { vec = [1, 0, 0], speed_var = "speed" } }
    ]
  }
}
