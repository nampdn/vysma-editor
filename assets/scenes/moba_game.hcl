// Complete MOBA Game Scene
// Demonstrates cross-file bundling and modular architecture

modules = [
  {
    name = "moba_core"
    path = "scenes/moba_core.hcl"
    alias = "core"
  },
  {
    name = "axe_hero"
    path = "scenes/heroes/axe.hcl"
    alias = "axe"
  }
]

// Game configuration
vars = {
  map_size = 12000.0
  base_health = 5000.0
  tower_health = 2000.0
  barracks_health = 1500.0
  creep_spawn_interval = 30.0
  gold_per_second = 1.0
}

// Game assets
assets {
  mesh "tower" { builtin = "cube" }
  mesh "barracks" { builtin = "cube" }
  mesh "base" { builtin = "cube" }
  mesh "creep" { builtin = "cube" }
  mesh "terrain" { builtin = "plane" }
  
  material "radiant_gold" { pbr = { base_color = "#FFD700", metallic = 0.8, roughness = 0.2 } }
  material "dire_red" { pbr = { base_color = "#DC143C", metallic = 0.8, roughness = 0.2 } }
  material "neutral_gray" { pbr = { base_color = "#808080", metallic = 0.5, roughness = 0.5 } }
  material "terrain_grass" { pbr = { base_color = "#228B22", metallic = 0.0, roughness = 0.8 } }
  material "tower_stone" { pbr = { base_color = "#696969", metallic = 0.0, roughness = 0.9 } }
}

// Game entities
entity "root" {
  components = { Name = "MOBAGame" }
  children = [
    // Terrain (enlarged so it fills the view)
    {
      name = "Terrain"
      components = {
        MeshRef = { mesh = "terrain" }
        StandardMaterialRef = { material = "terrain_grass" }
        // Plane primitive is 1x1; scale it large to avoid only flat green with no references
        Transform = { s = [2000, 1, 2000] }
      }
    },

    // Debug cube near origin so something obvious is visible
    {
      name = "DebugBox"
      components = {
        MeshRef = { mesh = "base" }
        StandardMaterialRef = { material = "tower_stone" }
        Transform = { t = [0, 1, 0], s = [2, 2, 2] }
      }
    },
    
    // Radiant Base
    {
      name = "RadiantBase"
      components = {
        MeshRef = { mesh = "base" }
        StandardMaterialRef = { material = "radiant_gold" }
        Transform = { t = [-5000, 2, -5000], s = [8, 4, 8] }
        include = ["core::BaseCombat"]
        Combat = { max_health = 5000.0, max_mana = 0.0, armor = 10.0, magic_resistance = 25.0 }
        Team = { id = 1, name = "Radiant", color = "#FFD700", is_radiant = true }
      }
      children = [
        {
          name = "RadiantAncient"
          components = {
            MeshRef = { mesh = "base" }
            StandardMaterialRef = { material = "radiant_gold" }
            Transform = { t = [0, 0, 0], s = [4, 6, 4] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 5000.0, max_mana = 0.0, armor = 15.0, magic_resistance = 25.0 }
          }
        }
      ]
    },
    
    // Dire Base
    {
      name = "DireBase"
      components = {
        MeshRef = { mesh = "base" }
        StandardMaterialRef = { material = "dire_red" }
        Transform = { t = [5000, 2, 5000], s = [8, 4, 8] }
        include = ["core::BaseCombat"]
        Combat = { max_health = 5000.0, max_mana = 0.0, armor = 10.0, magic_resistance = 25.0 }
        Team = { id = 2, name = "Dire", color = "#DC143C", is_radiant = false }
      }
      children = [
        {
          name = "DireAncient"
          components = {
            MeshRef = { mesh = "base" }
            StandardMaterialRef = { material = "dire_red" }
            Transform = { t = [0, 0, 0], s = [4, 6, 4] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 5000.0, max_mana = 0.0, armor = 15.0, magic_resistance = 25.0 }
          }
        }
      ]
    },
    
    // Radiant Towers
    {
      name = "RadiantTowers"
      components = { Name = "RadiantTowers" }
      children = [
        {
          name = "RadiantT1Top"
          components = {
            MeshRef = { mesh = "tower" }
            StandardMaterialRef = { material = "tower_stone" }
            Transform = { t = [-3000, 3, -3000], s = [2, 6, 2] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 2000.0, max_mana = 0.0, armor = 5.0, magic_resistance = 25.0 }
            Team = { id = 1, name = "Radiant", color = "#FFD700", is_radiant = true }
          }
        },
        {
          name = "RadiantT1Mid"
          components = {
            MeshRef = { mesh = "tower" }
            StandardMaterialRef = { material = "tower_stone" }
            Transform = { t = [-2000, 3, -2000], s = [2, 6, 2] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 2000.0, max_mana = 0.0, armor = 5.0, magic_resistance = 25.0 }
            Team = { id = 1, name = "Radiant", color = "#FFD700", is_radiant = true }
          }
        },
        {
          name = "RadiantT1Bot"
          components = {
            MeshRef = { mesh = "tower" }
            StandardMaterialRef = { material = "tower_stone" }
            Transform = { t = [-3000, 3, -3000], s = [2, 6, 2] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 2000.0, max_mana = 0.0, armor = 5.0, magic_resistance = 25.0 }
            Team = { id = 1, name = "Radiant", color = "#FFD700", is_radiant = true }
          }
        }
      ]
    },
    
    // Dire Towers
    {
      name = "DireTowers"
      components = { Name = "DireTowers" }
      children = [
        {
          name = "DireT1Top"
          components = {
            MeshRef = { mesh = "tower" }
            StandardMaterialRef = { material = "tower_stone" }
            Transform = { t = [3000, 3, 3000], s = [2, 6, 2] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 2000.0, max_mana = 0.0, armor = 5.0, magic_resistance = 25.0 }
            Team = { id = 2, name = "Dire", color = "#DC143C", is_radiant = false }
          }
        },
        {
          name = "DireT1Mid"
          components = {
            MeshRef = { mesh = "tower" }
            StandardMaterialRef = { material = "tower_stone" }
            Transform = { t = [2000, 3, 2000], s = [2, 6, 2] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 2000.0, max_mana = 0.0, armor = 5.0, magic_resistance = 25.0 }
            Team = { id = 2, name = "Dire", color = "#DC143C", is_radiant = false }
          }
        },
        {
          name = "DireT1Bot"
          components = {
            MeshRef = { mesh = "tower" }
            StandardMaterialRef = { material = "tower_stone" }
            Transform = { t = [3000, 3, 3000], s = [2, 6, 2] }
            include = ["core::BaseCombat"]
            Combat = { max_health = 2000.0, max_mana = 0.0, armor = 5.0, magic_resistance = 25.0 }
            Team = { id = 2, name = "Dire", color = "#DC143C", is_radiant = false }
          }
        }
      ]
    },
    
    // Hero Spawners
    {
      name = "RadiantHeroSpawner"
      components = {
        Name = "RadiantHeroSpawner"
        Transform = { t = [-4000, 0, -4000] }
        Team = { id = 1, name = "Radiant", color = "#FFD700", is_radiant = true }
      }
      children = [
        {
          name = "Axe"
          include = ["axe::Axe"]
          components = {
            Transform = { t = [0, 0.6, 0] }
            Team = { id = 1, name = "Radiant", color = "#FFD700", is_radiant = true }
          }
        }
      ]
    },
    
    {
      name = "DireHeroSpawner"
      components = {
        Name = "DireHeroSpawner"
        Transform = { t = [4000, 0, 4000] }
        Team = { id = 2, name = "Dire", color = "#DC143C", is_radiant = false }
      }
      children = [
        {
          name = "EnemyAxe"
          include = ["axe::Axe"]
          components = {
            Transform = { t = [0, 0.6, 0] }
            Team = { id = 2, name = "Dire", color = "#DC143C", is_radiant = false }
          }
        }
      ]
    },
    
    // Camera and Lighting
    {
      name = "MainCamera"
      components = {
        Camera3d = { hdr = true }
        Transform = { t = [0, 20, 30], look_at = [0, 0, 0] }
      }
    },
    
    {
      name = "Sun"
      components = {
        DirectionalLight = { illuminance = 60000.0, shadows = true }
        Transform = { euler = { x = -60, y = 45, z = 0 } }
      }
    }
  ]
}

// Game triggers
triggers {
  trigger "game_start" {
    name = "GameStart"
    on = { startup = true }
    category = "game"
    description = "Initialize game state and spawn initial entities"
    actions = [
      { set_var = { name = "game_time", value = 0.0 } },
      { emit = { name = "game_started" } }
    ]
  }
  
  trigger "creep_spawn" {
    name = "CreepSpawn"
    on = { tick = { every = 30.0 } }
    category = "spawning"
    description = "Spawn creeps for both teams"
    actions = [
      { spawn = { prefab = "core::BaseCombat", components = { Team = { id = 1 } } } },
      { spawn = { prefab = "core::BaseCombat", components = { Team = { id = 2 } } } }
    ]
  }
  
  trigger "tower_destroyed" {
    name = "TowerDestroyed"
    on = { event = "tower_destroyed" }
    category = "combat"
    description = "Handle tower destruction and award gold"
    actions = [
      { add_var = { name = "gold", by = 200.0 } },
      { emit = { name = "tower_fallen" } }
    ]
  }
  
  trigger "hero_killed" {
    name = "HeroKilled"
    on = { event = "hero_killed" }
    category = "combat"
    description = "Handle hero death and respawn"
    actions = [
      { add_var = { name = "experience", by = 100.0 } },
      { emit = { name = "hero_fallen" } }
    ]
  }
  
  trigger "victory_condition" {
    name = "VictoryCondition"
    on = { event = "ancient_destroyed" }
    category = "game"
    description = "Check victory conditions and end game"
    actions = [
      { emit = { name = "game_ended" } }
    ]
  }
} 