// Axe Hero Module - Strength hero with battle hunger and berserker's call
// Imports from the core MOBA module

modules = [
  {
    name = "moba_core"
    path = "scenes/moba_core.hcl"
    alias = "core"
  }
]

exports = [
  {
    name = "axe_hero"
    prefabs = ["Axe", "BattleHunger", "BerserkersCall", "CounterHelix", "CullingBlade"]
    entities = ["AxeSpawner"]
    triggers = ["battle_hunger_damage", "berserkers_call_taunt", "counter_helix_proc", "culling_blade_execute"]
    vars = ["axe_armor_bonus", "battle_hunger_duration", "counter_helix_chance"]
    public = true
    category = "hero"
    description = "Axe the Axe - Strength hero with tanking and initiation abilities"
    version = "1.0.0"
  }
]

// Hero-specific variables
vars = {
  axe_armor_bonus = 8.0
  battle_hunger_duration = 8.0
  counter_helix_chance = 0.2
  culling_blade_threshold = 0.3
}

// Axe Hero Prefab
prefab "Axe" {
  components = {
    // Include base hero components
    include = ["core::BaseHero"]
    
    // Override with Axe-specific stats
    Hero = {
      name = "Axe"
      level = 1
      experience = 0.0
      primary_attribute = "strength"
      base_health = 700.0
      base_mana = 270.0
      health_regen = 2.5
      mana_regen = 0.5
      base_armor = 3.0
      base_damage = 55.0
      attack_speed = 1.0
      attack_range = 150.0
      movement_speed = 290.0
      turn_rate = 0.6
    }
    
    Combat = {
      max_health = 700.0
      max_mana = 270.0
      armor = 3.0
      magic_resistance = 25.0
      damage_block = 0.0
      status_resistance = 0.0
    }
    
    Movement = {
      speed = 290.0
      can_move = true
    }
    
    // Axe-specific components
    Inventory = {
      max_slots = 6
      gold = 625
    }
    
    Vision = {
      range = 1800.0
      is_visible = true
      fog_of_war = true
    }
  }
  tags = ["hero", "strength", "tank", "initiator", "axe"]
  category = "hero"
  description = "Axe - Strength hero specializing in tanking and team fights"
  version = "1.0.0"
}

// Battle Hunger Ability
prefab "BattleHunger" {
  components = {
    include = ["core::BaseAbility"]
    
    Ability = {
      name = "BattleHunger"
      cooldown = 8.0
      mana_cost = 75.0
      damage = 25.0
      damage_type = "magical"
      range = 650.0
      cast_time = 0.0
      duration = 8.0
      effects = ["slow", "damage_over_time"]
    }
  }
  tags = ["ability", "debuff", "slow", "damage_over_time"]
  category = "ability"
  description = "Slows and damages an enemy unit over time"
  version = "1.0.0"
}

// Berserker's Call Ability
prefab "BerserkersCall" {
  components = {
    include = ["core::BaseAbility"]
    
    Ability = {
      name = "BerserkersCall"
      cooldown = 16.0
      mana_cost = 80.0
      damage = 0.0
      damage_type = "physical"
      range = 300.0
      cast_time = 0.0
      duration = 2.0
      effects = ["taunt", "armor_bonus"]
    }
  }
  tags = ["ability", "taunt", "armor", "initiation"]
  category = "ability"
  description = "Forces nearby enemies to attack Axe and grants armor bonus"
  version = "1.0.0"
}

// Counter Helix Ability
prefab "CounterHelix" {
  components = {
    include = ["core::BaseAbility"]
    
    Ability = {
      name = "CounterHelix"
      cooldown = 0.0
      mana_cost = 0.0
      damage = 90.0
      damage_type = "physical"
      range = 275.0
      cast_time = 0.0
      duration = 0.0
      effects = ["counter_attack"]
    }
  }
  tags = ["ability", "passive", "counter_attack", "damage"]
  category = "ability"
  description = "Passive ability that deals damage to nearby enemies when attacked"
  version = "1.0.0"
}

// Culling Blade Ability
prefab "CullingBlade" {
  components = {
    include = ["core::BaseAbility"]
    
    Ability = {
      name = "CullingBlade"
      cooldown = 75.0
      mana_cost = 150.0
      damage = 150.0
      damage_type = "pure"
      range = 150.0
      cast_time = 0.0
      duration = 0.0
      effects = ["execute", "speed_bonus"]
    }
  }
  tags = ["ability", "ultimate", "execute", "pure_damage"]
  category = "ability"
  description = "Ultimate ability that executes low health enemies and grants speed bonus"
  version = "1.0.0"
}

// Axe Spawner Entity
entity "AxeSpawner" {
  components = { 
    Name = "AxeSpawner"
    Transform = { t = [0, 0, 0] }
  }
  tags = ["spawner", "hero_spawner"]
  category = "spawner"
  description = "Spawns Axe hero in the game"
  children = [
    {
      name = "Axe"
      include = ["Axe"]
      components = { Transform = { t = [0, 0.6, 0] } }
    }
  ]
}

// Hero-specific triggers
triggers {
  trigger "battle_hunger_damage" {
    name = "BattleHungerDamage"
    on = { event = "battle_hunger_cast" }
    category = "ability"
    description = "Applies Battle Hunger debuff to target enemy"
    target = { name = "target" }
    actions = [
      { apply_effect = { targets = { name = "target" }, effect = "battle_hunger", duration = 8.0 } },
      { deal_damage = { targets = { name = "target" }, amount = 25.0, damage_type = "magical" } }
    ]
  }
  
  trigger "berserkers_call_taunt" {
    name = "BerserkersCallTaunt"
    on = { event = "berserkers_call_cast" }
    category = "ability"
    description = "Taunts nearby enemies and grants armor bonus"
    target = { name = "Axe" }
    actions = [
      { apply_effect = { targets = { name = "Axe" }, effect = "armor_bonus", duration = 2.0 } },
      { emit = { name = "enemies_taunted" } }
    ]
  }
  
  trigger "counter_helix_proc" {
    name = "CounterHelixProc"
    on = { event = "axe_attacked" }
    category = "ability"
    description = "Chance to trigger Counter Helix when Axe is attacked"
    target = { name = "Axe" }
    actions = [
      { emit = { name = "counter_helix_triggered" } }
    ]
  }
  
  trigger "culling_blade_execute" {
    name = "CullingBladeExecute"
    on = { event = "culling_blade_cast" }
    category = "ability"
    description = "Executes low health enemies with Culling Blade"
    target = { name = "target" }
    actions = [
      { deal_damage = { targets = { name = "target" }, amount = 150.0, damage_type = "pure" } },
      { apply_effect = { targets = { name = "Axe" }, effect = "speed_bonus", duration = 6.0 } }
    ]
  }
} 