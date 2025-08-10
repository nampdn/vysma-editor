assets {
  mesh "cube"   { builtin = "cube" }
  mesh "plane"  { builtin = "plane" }

  material "gold" { pbr = { base_color = "#FFD700", metallic = 1.0, roughness = 0.2 } }
}

entity "root" {
  components = { Name = "Root", Transform = { t = [0,0,0] } }
  children = [
    { name = "Ground", components = { MeshRef = { mesh = "plane" }, StandardMaterialRef = { material = "gold" }, Transform = { s = [10,1,10] } } },
    { name = "Box",    components = { MeshRef = { mesh = "cube"  }, StandardMaterialRef = { material = "gold" }, Transform = { t = [0,0.5,0] } } },
    { name = "Sun",    components = { DirectionalLight = { illuminance = 50000.0, shadows = true }, Transform = { euler = { x = -45, y = 60, z = 0 } } } },
    { name = "Camera", components = { Camera3d = { hdr = true }, Transform = { t = [0,4,10], look_at = [0,0.5,0] } } }
  ]
}
