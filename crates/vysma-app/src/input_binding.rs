use bevy::prelude::*;
use bevy::input::{
    ButtonInput,
    keyboard::KeyCode,
    mouse::{MouseButton, MouseMotion, MouseWheel},
    touch::{TouchInput, TouchPhase},
    gamepad::{GamepadAxis, GamepadButton},
};
use std::collections::HashMap;

/// Comprehensive input binding system for mapping desktop inputs to touch device equivalents
pub struct InputBindingPlugin;

impl Plugin for InputBindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputBindings>()
            .add_systems(Update, (
                touch_to_mouse_binding,
                touch_to_keyboard_binding,
                touch_to_gamepad_binding,
                multi_touch_gestures,
                touch_ui_interaction,
            ));
    }
}

/// Resource that stores input bindings and mappings
#[derive(Resource, Default)]
pub struct InputBindings {
    /// Maps touch regions to keyboard keys
    pub touch_to_keyboard: HashMap<String, KeyCode>,
    /// Maps touch gestures to mouse buttons
    pub touch_to_mouse: HashMap<String, MouseButton>,
    /// Maps touch gestures to gamepad inputs
    pub touch_to_gamepad: HashMap<String, GamepadButton>,
    /// Virtual joystick configuration
    pub virtual_joysticks: Vec<VirtualJoystick>,
    /// Touch UI elements
    pub touch_ui_elements: Vec<TouchUIElement>,
    /// Gesture recognition settings
    pub gesture_settings: GestureSettings,
}

/// Virtual joystick for touch input
#[derive(Clone)]
pub struct VirtualJoystick {
    pub id: String,
    pub position: Vec2,
    pub radius: f32,
    pub dead_zone: f32,
    pub axis_mapping: (GamepadAxis, GamepadAxis), // (x_axis, y_axis)
    pub button_mapping: Option<GamepadButton>,
}

/// Touch UI element
#[derive(Clone)]
pub struct TouchUIElement {
    pub id: String,
    pub bounds: Rect,
    pub key_mapping: Option<KeyCode>,
    pub mouse_mapping: Option<MouseButton>,
    pub gamepad_mapping: Option<GamepadButton>,
    pub callback: Option<String>, // For custom callbacks
}

/// Gesture recognition settings
#[derive(Clone)]
pub struct GestureSettings {
    pub tap_threshold: f32,
    pub double_tap_threshold: f32,
    pub long_press_threshold: f32,
    pub swipe_threshold: f32,
    pub pinch_threshold: f32,
}

impl Default for GestureSettings {
    fn default() -> Self {
        Self {
            tap_threshold: 0.1,
            double_tap_threshold: 0.3,
            long_press_threshold: 0.5,
            swipe_threshold: 50.0,
            pinch_threshold: 0.1,
        }
    }
}

/// System that converts touch events to mouse input
fn touch_to_mouse_binding(
    mut touch_events: EventReader<TouchInput>,
    mut mouse_button: ResMut<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventWriter<MouseMotion>,
    _mouse_wheel_events: EventWriter<MouseWheel>,
    bindings: Res<InputBindings>,
    _window_query: Query<&Window>,
) {
    let mut touch_positions: HashMap<u64, Vec2> = HashMap::new();
    let mut touch_start_positions: HashMap<u64, Vec2> = HashMap::new();
    
    for event in touch_events.read() {
        match event.phase {
            TouchPhase::Started => {
                touch_start_positions.insert(event.id, event.position);
                touch_positions.insert(event.id, event.position);
                
                // Map to mouse button based on position or gesture
                if let Some(button) = get_mouse_button_for_touch(&bindings, event.position) {
                    mouse_button.press(button);
                } else {
                    // Default to left mouse button
                    mouse_button.press(MouseButton::Left);
                }
            }
            TouchPhase::Moved => {
                let previous_position = touch_positions.get(&event.id).copied().unwrap_or(event.position);
                touch_positions.insert(event.id, event.position);
                
                // Send mouse motion event
                let delta = event.position - previous_position;
                mouse_motion_events.write(MouseMotion { delta });
                
                // Check for scroll gestures (two finger vertical movement)
                if touch_positions.len() == 2 {
                    let positions: Vec<Vec2> = touch_positions.values().copied().collect();
                    let avg_y = positions.iter().map(|p| p.y).sum::<f32>() / positions.len() as f32;
                    let _scroll_delta = avg_y * 0.01; // Scale factor
                    // Note: MouseWheel requires window entity, simplified for now
                    // mouse_wheel_events.send(MouseWheel { unit: MouseWheelUnit::Line, x: 0.0, y: scroll_delta });
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                touch_positions.remove(&event.id);
                touch_start_positions.remove(&event.id);
                
                // Release mouse button
                if let Some(button) = get_mouse_button_for_touch(&bindings, event.position) {
                    mouse_button.release(button);
                } else {
                    mouse_button.release(MouseButton::Left);
                }
            }
        }
    }
}

/// System that converts touch events to keyboard input
fn touch_to_keyboard_binding(
    mut touch_events: EventReader<TouchInput>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
) {
    for event in touch_events.read() {
        if let Some(key_code) = get_keyboard_key_for_touch(&bindings, event.position) {
            match event.phase {
                TouchPhase::Started => {
                    keyboard_input.press(key_code);
                }
                TouchPhase::Ended | TouchPhase::Canceled => {
                    keyboard_input.release(key_code);
                }
                _ => {}
            }
        }
    }
}

/// System that converts touch events to gamepad input
fn touch_to_gamepad_binding(
    mut touch_events: EventReader<TouchInput>,
    mut gamepad_button: ResMut<ButtonInput<GamepadButton>>,
    mut _gamepad_axis: ResMut<Axis<GamepadAxis>>,
    bindings: Res<InputBindings>,
) {
    for joystick in &bindings.virtual_joysticks {
        // Handle virtual joystick input
        for event in touch_events.read() {
            if is_touch_in_joystick_area(event.position, joystick) {
                let _joystick_value = calculate_joystick_value(event.position, joystick);
                
                // Update gamepad axis - simplified for now
                // gamepad_axis.set(GamepadAxis::new(Gamepad::new(0), joystick.axis_mapping.0), joystick_value.x);
                // gamepad_axis.set(GamepadAxis::new(Gamepad::new(0), joystick.axis_mapping.1), joystick_value.y);
                
                // Handle joystick button
                if let Some(button) = joystick.button_mapping {
                    match event.phase {
                        TouchPhase::Started => {
                            gamepad_button.press(button);
                        }
                        TouchPhase::Ended | TouchPhase::Canceled => {
                            gamepad_button.release(button);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// System that handles multi-touch gestures
fn multi_touch_gestures(
    mut touch_events: EventReader<TouchInput>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut mouse_button: ResMut<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
) {
    let mut active_touches: HashMap<u64, TouchInfo> = HashMap::new();
    
    for event in touch_events.read() {
        match event.phase {
            TouchPhase::Started => {
                active_touches.insert(event.id, TouchInfo {
                    position: event.position,
                    start_time: std::time::Instant::now(),
                    start_position: event.position,
                });
            }
            TouchPhase::Moved => {
                if let Some(touch_info) = active_touches.get_mut(&event.id) {
                    touch_info.position = event.position;
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if let Some(touch_info) = active_touches.remove(&event.id) {
                    // Handle gestures
                    handle_gesture(&touch_info, &mut keyboard_input, &mut mouse_button, &bindings);
                }
            }
        }
    }
}

/// System that handles touch UI interactions
fn touch_ui_interaction(
    mut touch_events: EventReader<TouchInput>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut mouse_button: ResMut<ButtonInput<MouseButton>>,
    bindings: Res<InputBindings>,
) {
    for event in touch_events.read() {
        for ui_element in &bindings.touch_ui_elements {
            if ui_element.bounds.contains(event.position) {
                match event.phase {
                    TouchPhase::Started => {
                        if let Some(key) = ui_element.key_mapping {
                            keyboard_input.press(key);
                        }
                        if let Some(mouse_btn) = ui_element.mouse_mapping {
                            mouse_button.press(mouse_btn);
                        }
                    }
                    TouchPhase::Ended | TouchPhase::Canceled => {
                        if let Some(key) = ui_element.key_mapping {
                            keyboard_input.release(key);
                        }
                        if let Some(mouse_btn) = ui_element.mouse_mapping {
                            mouse_button.release(mouse_btn);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// Helper functions

fn get_mouse_button_for_touch(bindings: &InputBindings, position: Vec2) -> Option<MouseButton> {
    // Check if position matches any touch-to-mouse binding
    for (region, button) in &bindings.touch_to_mouse {
        if is_position_in_region(position, region) {
            return Some(*button);
        }
    }
    None
}

fn get_keyboard_key_for_touch(bindings: &InputBindings, position: Vec2) -> Option<KeyCode> {
    // Check if position matches any touch-to-keyboard binding
    for (region, key) in &bindings.touch_to_keyboard {
        if is_position_in_region(position, region) {
            return Some(*key);
        }
    }
    None
}

fn is_position_in_region(position: Vec2, region: &str) -> bool {
    // Parse region string (e.g., "top_left", "bottom_right", "center")
    match region {
        "top_left" => position.x < 0.5 && position.y > 0.5,
        "top_right" => position.x > 0.5 && position.y > 0.5,
        "bottom_left" => position.x < 0.5 && position.y < 0.5,
        "bottom_right" => position.x > 0.5 && position.y < 0.5,
        "center" => position.x > 0.3 && position.x < 0.7 && position.y > 0.3 && position.y < 0.7,
        _ => false,
    }
}

fn is_touch_in_joystick_area(position: Vec2, joystick: &VirtualJoystick) -> bool {
    let distance = (position - joystick.position).length();
    distance <= joystick.radius
}

fn calculate_joystick_value(position: Vec2, joystick: &VirtualJoystick) -> Vec2 {
    let delta = position - joystick.position;
    let distance = delta.length();
    
    if distance <= joystick.dead_zone {
        Vec2::ZERO
    } else {
        let normalized = delta / distance;
        let clamped_distance = (distance - joystick.dead_zone).min(joystick.radius - joystick.dead_zone);
        normalized * (clamped_distance / (joystick.radius - joystick.dead_zone))
    }
}

#[derive(Clone)]
struct TouchInfo {
    position: Vec2,
    start_time: std::time::Instant,
    start_position: Vec2,
}

fn handle_gesture(
    touch_info: &TouchInfo,
    keyboard_input: &mut ResMut<ButtonInput<KeyCode>>,
    mouse_button: &mut ResMut<ButtonInput<MouseButton>>,
    bindings: &InputBindings,
) {
    let duration = touch_info.start_time.elapsed().as_secs_f32();
    let distance = (touch_info.position - touch_info.start_position).length();
    
    // Long press gesture
    if duration > bindings.gesture_settings.long_press_threshold {
        keyboard_input.press(KeyCode::Space); // Example: long press = space
    }
    
    // Swipe gestures
    if distance > bindings.gesture_settings.swipe_threshold {
        let direction = (touch_info.position - touch_info.start_position).normalize();
        
        if direction.y.abs() > direction.x.abs() {
            // Vertical swipe
            if direction.y > 0.0 {
                keyboard_input.press(KeyCode::ArrowUp);
            } else {
                keyboard_input.press(KeyCode::ArrowDown);
            }
        } else {
            // Horizontal swipe
            if direction.x > 0.0 {
                keyboard_input.press(KeyCode::ArrowRight);
            } else {
                keyboard_input.press(KeyCode::ArrowLeft);
            }
        }
    }
}

/// Extension trait for easy input binding setup
pub trait InputBindingExt {
    fn setup_default_touch_bindings(&mut self);
    fn add_virtual_joystick(&mut self, joystick: VirtualJoystick);
    fn add_touch_ui_element(&mut self, element: TouchUIElement);
}

impl InputBindingExt for App {
    fn setup_default_touch_bindings(&mut self) {
        let mut bindings = InputBindings::default();
        
        // Default touch-to-keyboard mappings
        bindings.touch_to_keyboard.insert("top_left".to_string(), KeyCode::KeyW);
        bindings.touch_to_keyboard.insert("top_right".to_string(), KeyCode::KeyE);
        bindings.touch_to_keyboard.insert("bottom_left".to_string(), KeyCode::KeyS);
        bindings.touch_to_keyboard.insert("bottom_right".to_string(), KeyCode::KeyD);
        bindings.touch_to_keyboard.insert("center".to_string(), KeyCode::Space);
        
        // Default touch-to-mouse mappings
        bindings.touch_to_mouse.insert("center".to_string(), MouseButton::Left);
        
        // Default virtual joystick
        bindings.virtual_joysticks.push(VirtualJoystick {
            id: "movement".to_string(),
            position: Vec2::new(100.0, 100.0),
            radius: 50.0,
            dead_zone: 10.0,
            axis_mapping: (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
            button_mapping: Some(GamepadButton::South),
        });
        
        self.insert_resource(bindings);
    }
    
    fn add_virtual_joystick(&mut self, _joystick: VirtualJoystick) {
        // Note: This would need to be implemented differently in a real app
        // For now, we'll just ignore this call
    }
    
    fn add_touch_ui_element(&mut self, _element: TouchUIElement) {
        // Note: This would need to be implemented differently in a real app
        // For now, we'll just ignore this call
    }
} 