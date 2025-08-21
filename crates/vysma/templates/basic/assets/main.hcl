# Vysma Basic Template - Main Scene
# This is the primary entry point for your game

assets {
  mesh "cube" { builtin = "cube" }
  mesh "plane" { builtin = "plane" }
  material "hero" { pbr = { base_color = "#3aa7ff" } }
  material "ground" { pbr = { base_color = "#8B4513" } }
}

# Global variables
vars = { 
  speed = 6.0,
  jump_height = 2.0
}

# Reusable prefabs
prefab "Hero" { 
  components = { 
    MeshRef = { mesh = "cube" }, 
    StandardMaterialRef = { material = "hero" }, 
    Transform = { s = [1,1,1] } 
  } 
}

prefab "Ground" {
  components = {
    MeshRef = { mesh = "plane" },
    StandardMaterialRef = { material = "ground" },
    Transform = { s = [20,1,20] }
  }
}

# Main scene entities
entity "root" { 
  children = [ 
    { name = "Ground", include = ["Ground"], components = { Transform = { t = [0,0,0] } } },
    { name = "Player", include = ["Hero"], components = { Transform = { t = [0,1,0] } } },
    { name = "Camera", components = { Camera3d = { hdr = true }, Transform = { t = [0,5,10], look_at = [0,1,0] } } },
    { name = "Sun", components = { DirectionalLight = { illuminance = 50000.0, shadows = true }, Transform = { euler = { x = -45, y = 45, z = 0 } } } }
  ] 
}

# Gameplay triggers
triggers { 
  trigger "move_w" { 
    on = { key_held = "KeyW" } 
    target = { name = "Player" } 
    actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed" } } ] 
  }
  
  trigger "move_s" { 
    on = { key_held = "KeyS" } 
    target = { name = "Player" } 
    actions = [ { translate_axis = { vec = [0,0,1], speed_var = "speed" } } ] 
  }
  
  trigger "move_a" { 
    on = { key_held = "KeyA" } 
    target = { name = "Player" } 
    actions = [ { translate_axis = { vec = [-1,0,0], speed_var = "speed" } } ] 
  }
  
  trigger "move_d" { 
    on = { key_held = "KeyD" } 
    target = { name = "Player" } 
    actions = [ { translate_axis = { vec = [1,0,0], speed_var = "speed" } } ] 
  }
  
  trigger "jump" { 
    on = { key_pressed = "Space" } 
    target = { name = "Player" } 
    actions = [ { translate_axis = { vec = [0,1,0], speed_var = "jump_height" } } ] 
  }
}
