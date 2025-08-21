# Vysma Basic Template

A clean, minimal Vysma project template with automatic HCL scene discovery.

## Project Structure

```
assets/
├── main.hcl          # Primary scene (auto-discovered)
├── scenes/           # Additional scene files
│   └── example.hcl   # Alternative scene
├── mesh/             # 3D models and meshes
├── textures/         # Image textures
└── fonts/            # Font files
```

## Quick Start

1. **Create project**: `vysma new mygame`
2. **Run server**: `vysma serve` (auto-discovers HCL files)
3. **Run client**: `vysma client` (connects to local server)
4. **Edit HCL**: Modify `assets/main.hcl` and see live updates

## HCL Scene Discovery

The CLI automatically discovers and loads HCL files in this priority order:
1. `assets/main.hcl` (primary entry point)
2. `assets/scene.hcl` or `assets/game.hcl`
3. Any `.hcl` files in `assets/scenes/`
4. Any other `.hcl` files in `assets/`

## Controls

- **WASD**: Move the blue cube player
- **Space**: Jump
- **F5**: Toggle Edit/Preview mode
- **Edit mode**: Pause gameplay, edit HCL in editor panel
- **Preview mode**: Run gameplay, editor panel shows status

## Hot Reload

- Edit any HCL file and save
- Server automatically detects changes
- All connected clients update in real-time
- No need to restart or reconnect

## Development Workflow

1. Edit HCL files in your preferred editor
2. Save changes (auto-reload)
3. Test gameplay in Preview mode
4. Use Edit mode for fine-tuning
5. Iterate rapidly with instant feedback

## Next Steps

- Add more entities and prefabs
- Create additional scene files
- Import modules from the community registry
- Publish your game as a module

