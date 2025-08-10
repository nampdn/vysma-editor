assets {
  mesh "cube"  { builtin = "cube" }
  mesh "plane" { builtin = "plane" }

  material "gold" { pbr = { base_color = "#FFD700", metallic = 1.0, roughness = 0.2 } }
  material "red"  { pbr = { base_color = "#ff4d4d", metallic = 0.0, roughness = 0.8, emissive = "#110000" } }
}

prefab "BoxPrefab" {
  components = {
    MeshRef = { mesh = "cube" },
    StandardMaterialRef = { material = "red" },
    Transform = { s = [0.5, 0.5, 0.5] }
  }
}

entity "root" {
  components = { Name = "Root" }
  children = [
    { name = "Ground", components = { MeshRef = { mesh = "plane" }, StandardMaterialRef = { material = "gold" }, Transform = { s = [12, 1, 12] } } },

    // a small grid of boxes using prefab include + local Transform override
    { name = "BoxA", include = ["BoxPrefab"], components = { Transform = { t = [-3, 0.5, -3] } } },
    { name = "BoxB", include = ["BoxPrefab"], components = { Transform = { t = [ 0, 0.5, -3] } } },
    { name = "BoxC", include = ["BoxPrefab"], components = { Transform = { t = [ 3, 0.5, -3] } } },
    { name = "BoxD", include = ["BoxPrefab"], components = { Transform = { t = [-3, 0.5,  0] } } },
    { name = "BoxE", include = ["BoxPrefab"], components = { Transform = { t = [ 0, 0.5,  0] } } },
    { name = "BoxF", include = ["BoxPrefab"], components = { Transform = { t = [ 3, 0.5,  0] } } },

    // Sun light
    { name = "Sun", components = { DirectionalLight = { illuminance = 60000.0, shadows = true }, Transform = { euler = { x = -60, y = 45, z = 0 } } } },

    // Camera
    { name = "Camera", components = { Camera3d = { hdr = true }, Transform = { t = [6, 5, 12], look_at = [0, 0.5, 0] } } }
  ]
}

