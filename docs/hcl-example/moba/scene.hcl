# MOBA Scene (Client‑side gameplay and UI)

modules = [
  { name = "alice::moba_core", alias = "core" },
  { name = "alice::axe", alias = "axe" }
]

# Include client input and abilities wiring
includes = [ "abilities.hcl", "input.hcl" ]

# Demo of HTTP asset IO (planned feature flag): you can mix local file paths and remote urls
assets {
  image "hud_font" { file = "fonts/FiraSans-Bold.ttf" }
  gltf  "map" { file = "mesh/heroes/axe.glb", node = "Scene0" }
}

# Global vars (read-only in client; server owns authoritative values)
vars = { speed = 6.0 }

# Prefabs come from modules (`core::BaseGame`, `axe::Axe`)
prefab "HudText" { components = { Name = "HpText" } }

entity "root" {
  children = [
    { name = "Game", include = ["core::BaseGame"], components = { Transform = { t = [0,0,0] } } },
    { name = "Camera", include = ["core::Camera3D"], components = { Transform = { t = [0,6,12] } } },
    { name = "Player", include = ["axe::Axe"], components = { Transform = { t = [0,1,0] }, Name = "Player" } },
    { name = "HUD", include = ["core::HudCanvas"], components = { Name = "HUD" } },
    { name = "HpText", include = ["core::Text"], components = { Name = "HpText" } }
  ]
}

# UI bindings: text updates from authoritative hp (replicated from server)
bindings = [
  { targets = { name = "HpText" }, path = "Text.value", expr = "\"HP: \" + hp", on = "tick", throttle = 0.1 }
]

# Note: All gameplay mutations are processed on server. This client only emits inputs and renders. 