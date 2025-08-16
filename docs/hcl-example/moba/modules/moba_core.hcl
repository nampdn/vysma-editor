# Module: moba_core (reusable)

assets {
  material "creep_mat" { pbr = { base_color = "#7bdc5b" } }
  material "projectile_mat" { pbr = { base_color = "#ffd54a" } }
  material "aoe_mat" { pbr = { base_color = "#ff7a7a" } }
  material "tower_mat" { pbr = { base_color = "#8ab4f8" } }
  material "nexus_mat" { pbr = { base_color = "#f28b82" } }
  material "ward_mat" { pbr = { base_color = "#a3ffcc" } }
  material "shop_mat" { pbr = { base_color = "#c9bfff" } }
  material "jungle_mat" { pbr = { base_color = "#9ccc65" } }
  material "brush_mat" { pbr = { base_color = "#2e7d32" } }
}

# Base lighting and camera rigs
prefab "Camera3D" { components = { Camera3d = { hdr = true }, Transform = { t = [0,6,12] } } }
prefab "SunLight" { components = { DirectionalLight = { illuminance = 50000.0, shadows = true }, Transform = { euler = { x = 45, y = -30, z = 0 } } } }

# HUD helpers (placeholder Text prefab)
prefab "Text" { components = { Name = "Text", Transform = { t = [0,0,0] } } }

# Base game root (light, map placeholder)
prefab "BaseGame" { components = { }, tags = ["Base"] }

# Gameplay prefabs
prefab "Creep" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "creep_mat" }, Transform = { s = [0.6,0.6,0.6] } }, tags = ["Creep"] }
prefab "Projectile" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "projectile_mat" }, Transform = { s = [0.2,0.2,0.6] } } }
prefab "AoE" { components = { MeshRef = { mesh = "plane" }, StandardMaterialRef = { material = "aoe_mat" }, Transform = { s = [1.0,0.05,1.0] } } }

# Core MOBA map primitives
prefab "SpawnPoint" { components = { Name = "SpawnPoint", Transform = { t = [0,0,0] } } }
prefab "Tower" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "tower_mat" }, Transform = { s = [0.8,2.0,0.8] } }, tags = ["Tower"] }
prefab "Nexus" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "nexus_mat" }, Transform = { s = [1.5,1.5,1.5] } }, tags = ["Nexus"] }
prefab "Ward" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "ward_mat" }, Transform = { s = [0.2,0.6,0.2] } }, tags = ["Ward"] }
prefab "Shop" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "shop_mat" }, Transform = { s = [1.0,1.0,1.0] } }, tags = ["Shop"] }
prefab "JungleCamp" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "jungle_mat" }, Transform = { s = [1.0,1.0,1.0] } }, tags = ["Jungle"] }
prefab "Brush" { components = { MeshRef = { mesh = "plane" }, StandardMaterialRef = { material = "brush_mat" }, Transform = { s = [3.0,0.05,3.0] } }, tags = ["Brush"] }

# Exports so consumers import via alias::Name
exports = [ { name = "core", prefabs = [
  "BaseGame","Camera3D","Text",
  "Creep","Projectile","AoE",
  "SpawnPoint","Tower","Nexus",
  "Ward","Shop","JungleCamp","Brush"
] } ] 