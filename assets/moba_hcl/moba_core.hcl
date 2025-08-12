// MOBA Core Module - Basic game mechanics and systems
// This module can be imported by other MOBA game modules

exports = [
  {
    name = "moba_core"
    prefabs = ["BaseHero", "BaseAbility", "BaseCombat", "BaseTeam"]
    entities = ["GameManager", "TeamManager", "CombatSystem"]
    triggers = ["damage_calculation", "experience_gain", "level_up"]
    vars = ["game_time", "gold_rate", "experience_rate"]
    public = true
    category = "core"
    description = "Core MOBA game mechanics and systems"
    version = "1.0.0"
  }
]

// Core game variables
vars = {
  game_time = 0.0
  gold_rate = 1.0
  experience_rate = 1.0
  base_hero_speed = 300.0
  base_hero_health = 600.0
  base_hero_mana = 300.0
  base_hero_armor = 5.0
  base_hero_damage = 50.0
}

// Core prefabs
prefab "BaseHero" {
  components = {
    Hero = {
      name = "BaseHero"
      level = 1
      experience = 0.0
      primary_attribute = "strength"
      base_health = 600.0
      base_mana = 300.0
      health_regen = 1.0
      mana_regen = 0.5
      base_armor = 5.0
      base_damage = 50.0
      attack_speed = 1.0
      attack_range = 150.0
      movement_speed = 300.0
      turn_rate = 0.7
    }
    Combat = {
      max_health = 600.0
      max_mana = 300.0
      armor = 5.0
      magic_resistance = 25.0
      damage_block = 0.0
      status_resistance = 0.0
    }
    Movement = {
      speed = 300.0
      can_move = true
    }
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
  tags = ["hero", "controllable"]
  category = "hero"
  description = "Base hero template with standard MOBA stats"
  version = "1.0.0"
}

prefab "BaseAbility" {
  components = {
    Ability = {
      name = "BaseAbility"
      cooldown = 0.0
      mana_cost = 0.0
      damage = 0.0
      damage_type = "physical"
      range = 0.0
      cast_time = 0.0
      duration = 0.0
      effects = []
    }
  }
  tags = ["ability", "skill"]
  category = "ability"
  description = "Base ability template"
  version = "1.0.0"
}

prefab "BaseCombat" {
  components = {
    Combat = {
      max_health = 100.0
      max_mana = 0.0
      armor = 0.0
      magic_resistance = 25.0
      damage_block = 0.0
      status_resistance = 0.0
    }
  }
  tags = ["combat", "damageable"]
  category = "combat"
  description = "Base combat component for damageable entities"
  version = "1.0.0"
}

prefab "BaseTeam" {
  components = {
    Team = {
      id = 0
      name = "Neutral"
      color = "#808080"
      is_radiant = false
    }
  }
  tags = ["team"]
  category = "team"
  description = "Base team component"
  version = "1.0.0"
}

// Core game entities
entity "GameManager" {
  components = { Name = "GameManager" }
  tags = ["game_manager", "singleton"]
  category = "system"
  description = "Manages core game state and systems"
}

entity "TeamManager" {
  components = { Name = "TeamManager" }
  tags = ["team_manager", "singleton"]
  category = "system"
  description = "Manages team assignments and team-based logic"
}

entity "CombatSystem" {
  components = { Name = "CombatSystem" }
  tags = ["combat_system", "singleton"]
  category = "system"
  description = "Handles combat calculations and damage application"
}

// Core game triggers
triggers {
  trigger "damage_calculation" {
    name = "DamageCalculation"
    on = { event = "damage_dealt" }
    category = "combat"
    description = "Calculates final damage after armor and resistance"
    actions = [
      { deal_damage = { targets = { name = "target" }, amount = 100.0, damage_type = "physical" } }
    ]
  }
  
  trigger "experience_gain" {
    name = "ExperienceGain"
    on = { event = "hero_killed" }
    category = "progression"
    description = "Awards experience to nearby heroes when a hero is killed"
    actions = [
      { set_var = { name = "experience_gained", value = 100.0 } }
    ]
  }
  
  trigger "level_up" {
    name = "LevelUp"
    on = { event = "experience_threshold_reached" }
    category = "progression"
    description = "Handles hero level up mechanics"
    actions = [
      { add_var = { name = "level", by = 1.0 } }
    ]
  }
} 