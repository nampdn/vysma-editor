use bevy::prelude::*;
use crate::hcl::registry::{ComponentApplier, EntityScratch, ApplyCtx, Json, from_json};
use serde::Deserialize;

// MOBA Game Components

#[derive(Component, Debug, Clone)]
pub struct Hero {
    pub name: String,
    pub level: u32,
    pub experience: f32,
    pub primary_attribute: AttributeType,
    pub base_health: f32,
    pub base_mana: f32,
    pub health_regen: f32,
    pub mana_regen: f32,
    pub base_armor: f32,
    pub base_damage: f32,
    pub attack_speed: f32,
    pub attack_range: f32,
    pub movement_speed: f32,
    pub turn_rate: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Ability {
    pub name: String,
    pub cooldown: f32,
    pub mana_cost: f32,
    pub damage: f32,
    pub damage_type: DamageType,
    pub range: f32,
    pub cast_time: f32,
    pub duration: f32,
    pub effects: Vec<Effect>,
}

#[derive(Component, Debug, Clone)]
pub struct Combat {
    pub current_health: f32,
    pub max_health: f32,
    pub current_mana: f32,
    pub max_mana: f32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub damage_block: f32,
    pub status_resistance: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Movement {
    pub speed: f32,
    pub target_position: Option<Vec3>,
    pub path: Vec<Vec3>,
    pub is_moving: bool,
    pub can_move: bool,
}

#[derive(Component, Debug, Clone)]
pub struct Team {
    pub id: u8,
    pub name: String,
    pub color: Color,
    pub is_radiant: bool,
}

#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub max_slots: usize,
    pub gold: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub cost: u32,
    pub effects: Vec<ItemEffect>,
    pub stackable: bool,
    pub max_stack: u32,
    pub current_stack: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Effect {
    pub name: String,
    pub effect_type: EffectType,
    pub duration: f32,
    pub remaining_time: f32,
    pub magnitude: f32,
    pub is_positive: bool,
    pub can_stack: bool,
    pub stack_count: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub speed: f32,
    pub target: Option<Entity>,
    pub target_position: Option<Vec3>,
    pub damage: f32,
    pub damage_type: DamageType,
    pub lifetime: f32,
    pub remaining_time: f32,
    pub pierces: bool,
    pub pierce_count: u32,
    pub current_pierces: u32,
}

#[derive(Component, Debug, Clone)]
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub movement_modifier: f32,
    pub vision_modifier: f32,
    pub destructible: bool,
    pub health: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Vision {
    pub range: f32,
    pub is_visible: bool,
    pub team_visibility: Vec<u8>,
    pub fog_of_war: bool,
}

// Enums for component properties
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeType {
    Strength,
    Agility,
    Intelligence,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DamageType {
    Physical,
    Magical,
    Pure,
    Chaos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectType {
    Stun,
    Slow,
    Silence,
    Root,
    Disarm,
    Mute,
    Break,
    Hex,
    Taunt,
    Fear,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Consumable,
    Basic,
    Upgrade,
    Artifact,
    Secret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerrainType {
    Grass,
    Forest,
    Mountain,
    Water,
    Road,
    Building,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemEffect {
    DamageBonus(f32),
    ArmorBonus(f32),
    HealthBonus(f32),
    ManaBonus(f32),
    SpeedBonus(f32),
    AttackSpeedBonus(f32),
    AbilityEffect(String),
}

// Component Appliers

pub struct HeroApplier;
impl ComponentApplier for HeroApplier {
    fn key(&self) -> &'static str { "Hero" }
    fn priority(&self) -> u8 { 50 }
    
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct HeroDef {
            name: String,
            #[serde(default)]
            level: u32,
            #[serde(default)]
            experience: f32,
            #[serde(default)]
            primary_attribute: String,
            #[serde(default)]
            base_health: f32,
            #[serde(default)]
            base_mana: f32,
            #[serde(default)]
            health_regen: f32,
            #[serde(default)]
            mana_regen: f32,
            #[serde(default)]
            base_armor: f32,
            #[serde(default)]
            base_damage: f32,
            #[serde(default)]
            attack_speed: f32,
            #[serde(default)]
            attack_range: f32,
            #[serde(default)]
            movement_speed: f32,
            #[serde(default)]
            turn_rate: f32,
        }

        let def: HeroDef = from_json(payload)?;
        let primary_attr = match def.primary_attribute.as_str() {
            "strength" | "Strength" => AttributeType::Strength,
            "agility" | "Agility" => AttributeType::Agility,
            "intelligence" | "Intelligence" => AttributeType::Intelligence,
            _ => AttributeType::Strength,
        };

        let hero = Hero {
            name: def.name,
            level: def.level,
            experience: def.experience,
            primary_attribute: primary_attr,
            base_health: def.base_health,
            base_mana: def.base_mana,
            health_regen: def.health_regen,
            mana_regen: def.mana_regen,
            base_armor: def.base_armor,
            base_damage: def.base_damage,
            attack_speed: def.attack_speed,
            attack_range: def.attack_range,
            movement_speed: def.movement_speed,
            turn_rate: def.turn_rate,
        };

        entity.insert(hero);
        Ok(())
    }
}

pub struct AbilityApplier;
impl ComponentApplier for AbilityApplier {
    fn key(&self) -> &'static str { "Ability" }
    fn priority(&self) -> u8 { 60 }
    
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct AbilityDef {
            name: String,
            #[serde(default)]
            cooldown: f32,
            #[serde(default)]
            mana_cost: f32,
            #[serde(default)]
            damage: f32,
            #[serde(default)]
            damage_type: String,
            #[serde(default)]
            range: f32,
            #[serde(default)]
            cast_time: f32,
            #[serde(default)]
            duration: f32,
            #[serde(default)]
            effects: Vec<String>,
        }

        let def: AbilityDef = from_json(payload)?;
        let damage_type = match def.damage_type.as_str() {
            "physical" | "Physical" => DamageType::Physical,
            "magical" | "Magical" => DamageType::Magical,
            "pure" | "Pure" => DamageType::Pure,
            "chaos" | "Chaos" => DamageType::Chaos,
            _ => DamageType::Physical,
        };

        let effects = def.effects.into_iter()
            .map(|name| Effect {
                name,
                effect_type: EffectType::Stun, // Default, should be parsed from effect name
                duration: 0.0,
                remaining_time: 0.0,
                magnitude: 0.0,
                is_positive: false,
                can_stack: false,
                stack_count: 1,
            })
            .collect();

        let ability = Ability {
            name: def.name,
            cooldown: def.cooldown,
            mana_cost: def.mana_cost,
            damage: def.damage,
            damage_type,
            range: def.range,
            cast_time: def.cast_time,
            duration: def.duration,
            effects,
        };

        entity.insert(ability);
        Ok(())
    }
}

pub struct CombatApplier;
impl ComponentApplier for CombatApplier {
    fn key(&self) -> &'static str { "Combat" }
    fn priority(&self) -> u8 { 70 }
    
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct CombatDef {
            #[serde(default)]
            max_health: f32,
            #[serde(default)]
            max_mana: f32,
            #[serde(default)]
            armor: f32,
            #[serde(default)]
            magic_resistance: f32,
            #[serde(default)]
            damage_block: f32,
            #[serde(default)]
            status_resistance: f32,
        }

        let def: CombatDef = from_json(payload)?;
        let combat = Combat {
            current_health: def.max_health,
            max_health: def.max_health,
            current_mana: def.max_mana,
            max_mana: def.max_mana,
            armor: def.armor,
            magic_resistance: def.magic_resistance,
            damage_block: def.damage_block,
            status_resistance: def.status_resistance,
        };

        entity.insert(combat);
        Ok(())
    }
}

pub struct TeamApplier;
impl ComponentApplier for TeamApplier {
    fn key(&self) -> &'static str { "Team" }
    fn priority(&self) -> u8 { 80 }
    
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct TeamDef {
            id: u8,
            name: String,
            #[serde(default)]
            color: String,
            #[serde(default)]
            is_radiant: bool,
        }

        let def: TeamDef = from_json(payload)?;
        let color = parse_hex_color(&def.color).unwrap_or(Color::WHITE);
        
        let team = Team {
            id: def.id,
            name: def.name,
            color,
            is_radiant: def.is_radiant,
        };

        entity.insert(team);
        Ok(())
    }
}

pub struct MovementApplier;
impl ComponentApplier for MovementApplier {
    fn key(&self) -> &'static str { "Movement" }
    fn priority(&self) -> u8 { 90 }
    
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct MovementDef {
            #[serde(default)]
            speed: f32,
            #[serde(default)]
            can_move: bool,
        }

        let def: MovementDef = from_json(payload)?;
        let movement = Movement {
            speed: def.speed,
            target_position: None,
            path: Vec::new(),
            is_moving: false,
            can_move: def.can_move,
        };

        entity.insert(movement);
        Ok(())
    }
}

// Helper function for parsing hex colors
fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim();
    let hex = s.strip_prefix('#').unwrap_or(s);
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    let bytes = match hex.len() {
        6 => u32::from_str_radix(hex, 16).ok().map(|v| (v << 8) | 0xFF)?,
        8 => u32::from_str_radix(hex, 16).ok()?,
        _ => return None,
    };
    let r = ((bytes >> 24) & 0xFF) as u8;
    let g = ((bytes >> 16) & 0xFF) as u8;
    let b = ((bytes >> 8) & 0xFF) as u8;
    let a = (bytes & 0xFF) as u8;
    Some(Color::srgba_u8(r, g, b, a))
} 