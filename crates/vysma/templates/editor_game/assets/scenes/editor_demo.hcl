assets { mesh "cube" { builtin = "cube" } material "hero" { pbr = { base_color = "#3aa7ff" } } }
vars = { speed = 6.0 }
prefab "Hero" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "hero" }, Transform = { s = [1,1,1] } } }
entity "root" { children = [ { name = "Player", include = ["Hero"], components = { Transform = { t = [0,1,0] } } } ] }
triggers { trigger "move_w" { on = { key_held = "KeyW" } target = { name = "Player" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed" } } ] } }


