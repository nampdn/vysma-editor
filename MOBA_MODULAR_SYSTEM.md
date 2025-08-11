# MOBA Modular Game System

A comprehensive, modular architecture for building complete MOBA games like Dota 2 using HCL (Hierarchical Configuration Language) and Bevy ECS.

## 🎯 Overview

This system provides a complete framework for creating MOBA games with:
- **Cross-file bundling** - Import/export modules between HCL files
- **Reusable game modules** - Heroes, abilities, items, terrain, etc.
- **ECS-based prefab system** - Modular component composition
- **Publishing ecosystem** - Share and reuse modules

## 🏗️ Architecture

### Core Components

1. **Module Registry** (`module_registry.rs`)
   - Manages cross-file modules and their exports
   - Handles dependency resolution and cycle detection
   - Provides module metadata for publishing

2. **Module Loader** (`module_loader.rs`)
   - Handles cross-file imports and dependency resolution
   - Merges modules with namespace support
   - Caches loaded modules for performance

3. **MOBA Components** (`moba_components.rs`)
   - Hero, Ability, Combat, Movement, Team components
   - Component appliers for HCL integration
   - Complete game mechanics implementation

4. **Module Marketplace** (`module_marketplace.rs`)
   - Discover, install, and manage published modules
   - Search and filtering capabilities
   - Compatibility checking and dependency management

## 📁 File Structure

```
assets/scenes/
├── moba_core.hcl          # Core MOBA mechanics
├── moba_game.hcl          # Complete game scene
└── heroes/
    └── axe.hcl            # Axe hero module

src/hcl/
├── mod.rs                 # Main HCL module
├── schema.rs              # Extended HCL schema
├── module_registry.rs     # Module management
├── module_loader.rs       # Cross-file loading
├── moba_components.rs     # Game components
└── module_marketplace.rs  # Publishing ecosystem
```

## 🚀 Getting Started

### 1. Basic Usage

```rust
use bevy::prelude::*;
use bevy_hcl_plugin::{HclPlugin, load_scene_at_startup};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HclPlugin)
        .add_systems(Startup, load_scene_at_startup("scenes/moba_game.hcl"))
        .run();
}
```

### 2. Loading Modules

```rust
use bevy_hcl_plugin::{HclPlugin, load_module_at_startup};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HclPlugin)
        .add_systems(Startup, load_module_at_startup(
            "axe_hero".to_string(),
            "scenes/heroes/axe.hcl".to_string()
        ))
        .run();
}
```

## 📝 HCL Module System

### Module Definition

```hcl
// Define module exports
exports = [
  {
    name = "my_module"
    prefabs = ["MyHero", "MyAbility"]
    entities = ["MySpawner"]
    triggers = ["my_trigger"]
    vars = ["my_var"]
    public = true
    category = "hero"
    description = "My custom hero module"
    version = "1.0.0"
  }
]

// Import other modules
modules = [
  {
    name = "moba_core"
    path = "scenes/moba_core.hcl"
    alias = "core"
  }
]
```

### Using Imported Modules

```hcl
// Use imported prefabs with namespace
prefab "MyHero" {
  components = {
    include = ["core::BaseHero"]  // Include from moba_core
    
    // Override with custom stats
    Hero = {
      name = "MyHero"
      base_health = 800.0
      movement_speed = 320.0
    }
  }
}

// Use imported entities
entity "MySpawner" {
  children = [
    {
      name = "MyHero"
      include = ["MyHero"]
      components = { Transform = { t = [0, 0.6, 0] } }
    }
  ]
}
```

## 🎮 MOBA Game Components

### Hero Component

```hcl
prefab "MyHero" {
  components = {
    Hero = {
      name = "MyHero"
      level = 1
      primary_attribute = "strength"
      base_health = 600.0
      base_mana = 300.0
      movement_speed = 300.0
      attack_range = 150.0
    }
    Combat = {
      max_health = 600.0
      max_mana = 300.0
      armor = 5.0
      magic_resistance = 25.0
    }
    Movement = {
      speed = 300.0
      can_move = true
    }
    Team = {
      id = 1
      name = "Radiant"
      color = "#FFD700"
      is_radiant = true
    }
  }
}
```

### Ability Component

```hcl
prefab "MyAbility" {
  components = {
    Ability = {
      name = "MyAbility"
      cooldown = 10.0
      mana_cost = 100.0
      damage = 150.0
      damage_type = "magical"
      range = 600.0
      cast_time = 0.5
      duration = 3.0
      effects = ["stun", "slow"]
    }
  }
}
```

## 🔧 Advanced Features

### Custom Component Appliers

```rust
use bevy::prelude::*;
use crate::hcl::registry::{ComponentApplier, EntityScratch, ApplyCtx, Json, from_json};

pub struct CustomComponentApplier;

impl ComponentApplier for CustomComponentApplier {
    fn key(&self) -> &'static str { "CustomComponent" }
    fn priority(&self) -> u8 { 100 }
    
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        #[derive(Deserialize)]
        struct CustomDef {
            value: f32,
            name: String,
        }

        let def: CustomDef = from_json(payload)?;
        
        // Apply custom component logic
        entity.insert(CustomComponent {
            value: def.value,
            name: def.name,
        });
        
        Ok(())
    }
}

// Register in startup system
fn register_custom_components(mut registry: ResMut<ComponentRegistry>) {
    registry.register(CustomComponentApplier);
}
```

### Module Publishing

```rust
use crate::hcl::module_marketplace::ModuleMarketplace;

// Publish a module
async fn publish_module(mut marketplace: ResMut<ModuleMarketplace>) {
    let module = PublishedModule {
        id: "my_hero_v1".to_string(),
        name: "My Hero".to_string(),
        author: "MyName".to_string(),
        description: "A custom hero module".to_string(),
        category: "hero".to_string(),
        tags: vec!["custom", "hero", "unique"].into_iter().map(|s| s.to_string()).collect(),
        version: "1.0.0".to_string(),
        downloads: 0,
        rating: 0.0,
        price: None,
        download_url: "https://my-modules.dev/my_hero_v1.hcl".to_string(),
        thumbnail_url: None,
        dependencies: vec!["moba_core".to_string()],
        compatibility: CompatibilityInfo {
            min_version: "1.0.0".to_string(),
            max_version: None,
            required_modules: vec!["moba_core".to_string()],
            conflicts: vec![],
        },
        changelog: vec![],
        reviews: vec![],
    };

    // Add to marketplace
    marketplace.available_modules.insert(module.id.clone(), module);
}
```

## 🌐 Publishing Ecosystem

### Module Categories

- **Core**: Basic game mechanics and systems
- **Hero**: Individual hero implementations
- **Ability**: Special abilities and skills
- **Item**: Equipment and consumables
- **Terrain**: Map and environment elements
- **UI**: User interface components
- **Audio**: Sound effects and music
- **Visual**: Particle effects and animations

### Module Metadata

Each module includes:
- Name, description, and version
- Author and licensing information
- Dependencies and compatibility
- Tags and categories
- Download statistics and ratings
- Changelog and user reviews

### Search and Discovery

```rust
// Search for modules
let results = marketplace.search_modules(
    "hero",           // Query
    Some("hero"),     // Category filter
    Some(&["strength".to_string()])  // Tag filter
);

// Browse by category
let hero_modules = marketplace.get_modules_by_category("hero");

// Check for updates
let updates = marketplace.check_for_updates();
```

## 🔄 Hot Reloading

The system supports hot reloading of HCL files:

```hcl
// Changes to HCL files are automatically detected
// and applied without restarting the game
prefab "MyHero" {
  components = {
    Hero = {
      base_health = 700.0  // Change this value and save
    }
  }
}
```

## 📊 Performance Considerations

- **Module Caching**: Loaded modules are cached to avoid repeated file I/O
- **Lazy Loading**: Modules are only loaded when needed
- **Dependency Resolution**: Efficient dependency graph traversal
- **Component Pooling**: Reusable component instances for frequently spawned entities

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loading() {
        let mut loader = ModuleLoader::new();
        loader.register_module_path("test".to_string(), "test.hcl".to_string());
        
        assert!(loader.is_module_loaded("test"));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_complete_game_scene() {
    let mut app = App::new();
    app.add_plugins(HclPlugin);
    
    // Load and test complete game scene
    app.add_systems(Startup, load_scene_at_startup("scenes/moba_game.hcl"));
    
    // Verify entities are spawned correctly
    // Test game mechanics and triggers
}
```

## 🚧 Roadmap

- [ ] **Network Multiplayer**: Real-time multiplayer support
- [ ] **AI Systems**: Computer-controlled heroes and creeps
- [ ] **Matchmaking**: Player pairing and game creation
- [ ] **Replay System**: Game recording and playback
- [ ] **Modding Tools**: Visual editor for HCL files
- [ ] **Performance Profiling**: Built-in performance monitoring
- [ ] **Cloud Sync**: Save games and settings across devices

## 🤝 Contributing

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Commit your changes**: `git commit -m 'Add amazing feature'`
4. **Push to the branch**: `git push origin feature/amazing-feature`
5. **Open a Pull Request**

### Module Development Guidelines

- Follow the established naming conventions
- Include comprehensive documentation
- Test with multiple module combinations
- Ensure backward compatibility
- Provide clear examples and usage

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Bevy Engine**: For the excellent ECS foundation
- **HCL Community**: For the configuration language inspiration
- **MOBA Game Developers**: For the game design patterns and mechanics

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/your-repo/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-repo/discussions)
- **Documentation**: [Wiki](https://github.com/your-repo/wiki)

---

**Build the next great MOBA game with modular, reusable components! 🎮** 