// Complete MOBA-like Demo Scene (concise, fully HCL-driven logic)

// Game configuration
vars = {
  speed = 6.0
  game_time = 0.0
  enemy_hp = 100.0
  damage = 25.0
}

// Assets
assets {
  mesh "cube" { builtin = "cube" }
  mesh "plane" { builtin = "plane" }

  material "grass" { pbr = { base_color = "#228B22", metallic = 0.0, roughness = 0.9 } }
  material "rock" { pbr = { base_color = "#6e6e6e", metallic = 0.0, roughness = 0.95 } }
  material "dirt" { pbr = { base_color = "#6b4f2f", metallic = 0.0, roughness = 0.9 } }
  material "hero" { pbr = { base_color = "#3aa7ff", metallic = 0.0, roughness = 0.8 } }
  material "enemy" { pbr = { base_color = "#dc143c", metallic = 0.0, roughness = 0.8 } }

  // Load GLTF hero model; default to first scene if node not specified
  gltf "axe" { file = "mesh/heroes/axe.glb" }
}

entity "root" {
  components = { Name = "DemoRoot" }
  children = [
    // Terrain: wide ground + simple stepped cliffs
    {
      name = "Terrain"
      components = { MeshRef = { mesh = "plane" }, StandardMaterialRef = { material = "grass" }, Transform = { s = [200, 1, 200] } }
      children = [
        // Dirt path strip
        { name = "Path", components = { MeshRef = { mesh = "plane" }, StandardMaterialRef = { material = "dirt" }, Transform = { t = [0, 0.01, 0], s = [40, 1, 6] } } },
        // Cliff: three steps
        // { name = "Cliff1", components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "rock" }, Transform = { t = [-10, 1, -5], s = [20, 2, 4] } } },
        // { name = "Cliff2", components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "dirt" }, Transform = { t = [-10, 2.5, -1], s = [20, 5, 2] } } },
        // { name = "Cliff3", components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "grass" }, Transform = { t = [-10, 5.5, 1], s = [20, 11, 2] } } }
      ]
    },

    // Player hero: render axe GLTF
    {
      name = "Player"
      components = {
        SceneRef = { scene = "axe" }
        Transform = { t = [0, 0, 0], s = [10.0, 10.0, 10.0] }
      }
    },

    // Enemy dummy
    {
      name = "Enemy"
      components = {
        MeshRef = { mesh = "cube" }
        StandardMaterialRef = { material = "enemy" }
        Transform = { t = [7, 1, 0], s = [1.5, 1.5, 1.5] }
      }
      tags = ["enemy"]
    },

    // Camera and light
    { name = "Camera", components = { Camera3d = { hdr = true }, Transform = { t = [0, 20, 30], look_at = [0, 1, 0] } } },
    { name = "Sun", components = { DirectionalLight = { illuminance = 60000.0, shadows = true }, Transform = { euler = { x = -60, y = 45, z = 0 } } } }
  ]
}

// Triggers: movement, combat loop, spawning
triggers {
  // Startup init
  trigger "init" {
    on = { startup = true }
    actions = [
      { set_var = { name = "game_time", value = 0.0 } }
    ]
  }

  // Movement WASD, scaled by dt and speed
  trigger "move_w" {
    on = { key_held = "KeyW" }
    target = { name = "Player" }
    actions = [ { translate_axis = { vec = [0, 0, -1], speed_var = "speed" } } ]
  }
  trigger "move_s" {
    on = { key_held = "KeyS" }
    target = { name = "Player" }
    actions = [ { translate_axis = { vec = [0, 0, 1], speed_var = "speed" } } ]
  }
  trigger "move_a" {
    on = { key_held = "KeyA" }
    target = { name = "Player" }
    actions = [ { translate_axis = { vec = [-1, 0, 0], speed_var = "speed" } } ]
  }
  trigger "move_d" {
    on = { key_held = "KeyD" }
    target = { name = "Player" }
    actions = [ { translate_axis = { vec = [1, 0, 0], speed_var = "speed" } } ]
  }

  // Attack on Space: subtract HP and check for death
  trigger "attack" {
    on = { key_pressed = "Space" }
    actions = [
      { add_var = { name = "enemy_hp", by = -25.0 } },
      { emit = { name = "check_enemy" } }
    ]
  }
  trigger "enemy_die_check" {
    on = { event = "check_enemy" }
    when = [ { expr = "enemy_hp <= 0" } ]
    target = { name = "Enemy" }
    actions = [ { despawn = { } } ]
  }

  // Respawn enemy every 5s if none visible
  trigger "respawn_enemy" {
    on = { tick = { every = 5.0 } }
    when = [ { not = { all_visible = { name = "Enemy" } } } ]
    actions = [
      { set_var = { name = "enemy_hp", value = 100.0 } },
      { spawn = { components = { Name = "Enemy", MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "enemy" }, Transform = { t = [6, 1, 0], s = [1.5, 1.5, 1.5] } } } }
    ]
  }
} 