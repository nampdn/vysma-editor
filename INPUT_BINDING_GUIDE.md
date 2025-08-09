# Comprehensive Input Binding System for iOS Touch Devices

This guide explains how to use the comprehensive input binding system that maps all desktop input events to their iOS touch equivalents, ensuring Bevy can receive all input events correctly on touch devices.

## Overview

The input binding system provides a complete solution for converting touch input on iOS devices to standard Bevy input events that your game logic expects. This allows you to write input handling code once and have it work seamlessly across desktop and mobile platforms.

## Features

### 1. **Touch-to-Mouse Binding**
- Converts single touch to mouse clicks
- Maps touch movement to mouse motion
- Supports multi-touch gestures for mouse wheel simulation

### 2. **Touch-to-Keyboard Binding**
- Maps touch regions to keyboard keys
- Supports gesture-based keyboard input (swipes, long press)
- Configurable touch zones for different actions

### 3. **Touch-to-Gamepad Binding**
- Virtual joystick support
- Touch button mapping to gamepad buttons
- Configurable dead zones and sensitivity

### 4. **Multi-Touch Gestures**
- Tap, double-tap, and long press detection
- Swipe gestures in all directions
- Pinch-to-zoom support (framework ready)

### 5. **Touch UI Interaction**
- Touch UI element mapping
- Custom callback support
- Bounds-based touch detection

## Quick Start

### 1. Add the Plugin

```rust
use bevy_in_app::{InputBindingPlugin, InputBindingExt};

fn main() {
    App::new()
        .add_plugins(InputBindingPlugin)
        .setup_default_touch_bindings() // For iOS/Android
        .run();
}
```

### 2. Default Bindings

The system comes with sensible defaults:

- **Touch Regions to Keys:**
  - Top-left: `W` (move up)
  - Top-right: `E` (interact)
  - Bottom-left: `S` (move down)
  - Bottom-right: `D` (move right)
  - Center: `Space` (jump/action)

- **Touch to Mouse:**
  - Center region: Left mouse button
  - Touch movement: Mouse motion
  - Two-finger vertical movement: Mouse wheel

- **Virtual Joystick:**
  - Position: (100, 100)
  - Radius: 50 pixels
  - Dead zone: 10 pixels
  - Maps to: Left stick X/Y axes

## Advanced Configuration

### Custom Touch-to-Keyboard Mapping

```rust
use bevy::input::KeyCode;
use bevy_in_app::{InputBindings, InputBindingExt};

fn setup_custom_bindings(mut app: &mut App) {
    let mut bindings = InputBindings::default();
    
    // Map specific regions to keys
    bindings.touch_to_keyboard.insert("top_left".to_string(), KeyCode::KeyW);
    bindings.touch_to_keyboard.insert("top_right".to_string(), KeyCode::KeyE);
    bindings.touch_to_keyboard.insert("bottom_left".to_string(), KeyCode::KeyS);
    bindings.touch_to_keyboard.insert("bottom_right".to_string(), KeyCode::KeyD);
    bindings.touch_to_keyboard.insert("center".to_string(), KeyCode::Space);
    
    app.insert_resource(bindings);
}
```

### Custom Touch-to-Mouse Mapping

```rust
use bevy::input::MouseButton;
use bevy_in_app::{InputBindings, InputBindingExt};

fn setup_mouse_bindings(mut app: &mut App) {
    let mut bindings = InputBindings::default();
    
    // Map regions to mouse buttons
    bindings.touch_to_mouse.insert("center".to_string(), MouseButton::Left);
    bindings.touch_to_mouse.insert("right_side".to_string(), MouseButton::Right);
    
    app.insert_resource(bindings);
}
```

### Virtual Joystick Configuration

```rust
use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy_in_app::{VirtualJoystick, InputBindingExt};

fn setup_virtual_joysticks(mut app: &mut App) {
    // Movement joystick
    let movement_joystick = VirtualJoystick {
        id: "movement".to_string(),
        position: Vec2::new(100.0, 100.0),
        radius: 50.0,
        dead_zone: 10.0,
        axis_mapping: (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
        button_mapping: Some(GamepadButton::South),
    };
    
    app.add_virtual_joystick(movement_joystick);
}
```

### Touch UI Elements

```rust
use bevy::input::{KeyCode, MouseButton};
use bevy_in_app::{TouchUIElement, InputBindingExt};

fn setup_touch_ui(mut app: &mut App) {
    // Jump button
    let jump_button = TouchUIElement {
        id: "jump_button".to_string(),
        bounds: Rect::new(300.0, 100.0, 400.0, 200.0), // x, y, width, height
        key_mapping: Some(KeyCode::Space),
        mouse_mapping: Some(MouseButton::Left),
        gamepad_mapping: Some(GamepadButton::South),
        callback: Some("jump_action".to_string()),
    };
    
    app.add_touch_ui_element(jump_button);
}
```

### Gesture Settings

```rust
use bevy_in_app::{GestureSettings, InputBindingExt};

fn setup_gesture_settings(mut app: &mut App) {
    let mut bindings = InputBindings::default();
    
    bindings.gesture_settings = GestureSettings {
        tap_threshold: 0.1,        // 100ms
        double_tap_threshold: 0.3, // 300ms
        long_press_threshold: 0.5, // 500ms
        swipe_threshold: 50.0,     // 50 pixels
        pinch_threshold: 0.1,      // 10% scale change
    };
    
    app.insert_resource(bindings);
}
```

## Input Event Mapping

### Desktop Input → iOS Touch Equivalent

| Desktop Input | iOS Touch Equivalent | Description |
|---------------|---------------------|-------------|
| Mouse Click | Single Touch | Tap anywhere on screen |
| Mouse Drag | Touch + Move | Touch and drag |
| Mouse Wheel | Two-finger Vertical Swipe | Scroll up/down |
| Keyboard WASD | Touch Regions | Tap corners for movement |
| Space Bar | Center Touch | Tap center for jump/action |
| Arrow Keys | Swipe Gestures | Swipe in direction |
| Long Press | Touch + Hold | Hold touch for 500ms |

### Gesture Recognition

The system automatically recognizes these gestures:

1. **Tap**: Quick touch and release
2. **Double Tap**: Two quick taps within 300ms
3. **Long Press**: Touch and hold for 500ms
4. **Swipe**: Touch, move, and release (50px minimum)
5. **Pinch**: Two-finger pinch gesture (framework ready)

## Usage in Game Logic

### Standard Input Handling

Your game logic remains unchanged! The input binding system ensures that all touch input is converted to standard Bevy input events:

```rust
// This works the same on desktop and iOS
fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut transform: Query<&mut Transform, With<Player>>,
) {
    for mut transform in &mut transform {
        if keyboard_input.pressed(KeyCode::KeyW) {
            transform.translation.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            transform.translation.y -= 1.0;
        }
        // ... etc
    }
}
```

### Mouse Input Handling

```rust
// Mouse input works the same on desktop and iOS
fn handle_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // Spawn something at cursor position
        commands.spawn(SpriteBundle::default());
    }
}
```

### Gamepad Input Handling

```rust
// Gamepad input works with virtual joysticks
fn handle_gamepad_input(
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
) {
    if gamepad_buttons.just_pressed(GamepadButton::South) {
        // Jump action
    }
    
    let left_stick_x = gamepad_axes.get(GamepadAxis::LeftStickX);
    let left_stick_y = gamepad_axes.get(GamepadAxis::LeftStickY);
    // Handle movement
}
```

## Platform-Specific Setup

### iOS Setup

The input binding system is automatically configured for iOS when you use:

```rust
#[cfg(target_os = "ios")]
app.setup_default_touch_bindings();
```

### Android Setup

For Android, the same system works:

```rust
#[cfg(target_os = "android")]
app.setup_default_touch_bindings();
```

### Desktop Setup

On desktop, the input binding system is disabled by default, so standard keyboard/mouse input works normally.

## Customization Examples

### RPG Game Controls

```rust
fn setup_rpg_controls(mut app: &mut App) {
    let mut bindings = InputBindings::default();
    
    // Movement joystick
    bindings.virtual_joysticks.push(VirtualJoystick {
        id: "movement".to_string(),
        position: Vec2::new(80.0, 80.0),
        radius: 60.0,
        dead_zone: 15.0,
        axis_mapping: (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
        button_mapping: None,
    });
    
    // Action buttons
    bindings.touch_ui_elements.push(TouchUIElement {
        id: "attack".to_string(),
        bounds: Rect::new(600.0, 100.0, 700.0, 200.0),
        key_mapping: Some(KeyCode::KeyA),
        mouse_mapping: Some(MouseButton::Left),
        gamepad_mapping: Some(GamepadButton::South),
        callback: None,
    });
    
    bindings.touch_ui_elements.push(TouchUIElement {
        id: "defend".to_string(),
        bounds: Rect::new(600.0, 250.0, 700.0, 350.0),
        key_mapping: Some(KeyCode::KeyD),
        mouse_mapping: Some(MouseButton::Right),
        gamepad_mapping: Some(GamepadButton::West),
        callback: None,
    });
    
    app.insert_resource(bindings);
}
```

### Platformer Game Controls

```rust
fn setup_platformer_controls(mut app: &mut App) {
    let mut bindings = InputBindings::default();
    
    // Simple touch regions for platformer
    bindings.touch_to_keyboard.insert("left_half".to_string(), KeyCode::ArrowLeft);
    bindings.touch_to_keyboard.insert("right_half".to_string(), KeyCode::ArrowRight);
    bindings.touch_to_keyboard.insert("top_half".to_string(), KeyCode::Space);
    
    // Gesture-based controls
    bindings.gesture_settings.swipe_threshold = 30.0; // Lower threshold for quick response
    
    app.insert_resource(bindings);
}
```

## Best Practices

1. **Test on Real Devices**: Always test touch controls on actual iOS devices
2. **Provide Visual Feedback**: Show touch zones and joystick areas
3. **Consider Screen Sizes**: Make touch targets large enough (44pt minimum)
4. **Support Multiple Input Methods**: Allow both touch and keyboard input
5. **Use Gestures Intuitively**: Swipe up for jump, swipe down for crouch, etc.
6. **Provide Options**: Let users customize touch sensitivity and button placement

## Troubleshooting

### Touch Not Responding
- Check that `InputBindingPlugin` is added to your app
- Verify touch regions are correctly configured
- Ensure touch targets are large enough

### Gestures Not Working
- Adjust gesture thresholds in `GestureSettings`
- Check that multi-touch is enabled
- Verify gesture recognition logic

### Performance Issues
- Limit the number of active touch events
- Use efficient touch region checking
- Consider using spatial partitioning for many UI elements

## Conclusion

The comprehensive input binding system provides a complete solution for making your Bevy games work seamlessly on iOS touch devices. By converting touch input to standard Bevy input events, you can write your game logic once and have it work across all platforms.

The system is highly configurable and can be adapted to any type of game, from simple puzzle games to complex RPGs. With proper setup and testing, your iOS users will have a native touch experience that feels natural and responsive. 