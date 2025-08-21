# Vysma Editor-as-Game Template

A powerful template that demonstrates real-time HCL editing within the game itself. The editor panel lets you modify the running scene without restarting.

## Project Structure

```
assets/
├── main.hcl          # Primary scene (auto-discovered)
├── scenes/           # Additional scene files
├── mesh/             # 3D models and meshes
├── textures/         # Image textures
└── fonts/            # Font files
```

## Quick Start

1. **Create project**: `vysma new --template editor_game myeditor`
2. **Run server**: `vysma serve` (auto-discovers HCL files)
3. **Run client**: `vysma client` (connects to local server with editor panel)
4. **Edit in real-time**: Use the editor panel to modify HCL and see instant changes

## Editor Panel Features

- **Real-time HCL editing**: Modify the scene while it's running
- **Apply button**: Send changes to the server and see them instantly
- **Edit/Preview toggle**: F5 to switch between editing and gameplay modes
- **Status display**: Shows last applied SHA and timestamp
- **Hot reload**: File changes automatically detected and applied

## Controls

- **WASD**: Move the blue cube player
- **Space**: Jump
- **F5**: Toggle Edit/Preview mode
- **F6**: Demo color change (see the trigger in main.hcl)

## Real-time Editing Examples

Try these edits in the editor panel:

1. **Change speed**: Modify the `speed` variable and see movement speed change instantly
2. **Add new triggers**: Create new key bindings or actions
3. **Modify entities**: Change positions, scales, or add new objects
4. **Create prefabs**: Define reusable components
5. **Adjust materials**: Change colors, textures, or properties

## Development Workflow

1. **Start the game**: Run server and client
2. **Play and observe**: Test the current gameplay
3. **Switch to Edit mode**: Press F5 to pause gameplay
4. **Edit HCL**: Use the editor panel to modify the scene
5. **Apply changes**: Click Apply to see changes instantly
6. **Test gameplay**: Switch back to Preview mode
7. **Iterate rapidly**: No restarts needed!

## Auto-Discovery

The CLI automatically finds and loads HCL files in this priority order:
1. `assets/main.hcl` (primary entry point)
2. `assets/scene.hcl` or `assets/game.hcl`
3. Any `.hcl` files in `assets/scenes/`
4. Any other `.hcl` files in `assets/`

## Hot Reload

- Edit any HCL file and save
- Server automatically detects changes
- All connected clients update in real-time
- Editor panel shows current content
- Apply button sends changes to server

## Next Steps

- Create multiple scene files for different levels
- Import community modules via `modules = [...]`
- Add more complex gameplay mechanics
- Publish your game as a module
- Share with others via the relay system

## Why Editor-as-Game?

This template demonstrates Vysma's core philosophy:
- **Live editing**: See changes immediately without restarting
- **Rapid iteration**: Test ideas quickly and refine gameplay
- **Visual feedback**: Understand how HCL affects the game world
- **Learning tool**: Experiment with different configurations
- **Production ready**: The same HCL can be used in shipped games

The editor panel isn't just for development - it's a powerful tool for understanding and modifying your game world in real-time!

