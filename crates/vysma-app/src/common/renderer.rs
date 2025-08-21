use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
#[cfg(feature = "bevygap_client")]
use bevygap_client_plugin::prelude::*;
use lightyear::connection::client::ClientState;
use lightyear::prelude::*;

use vysma_hcl::hcl::{EditorMode, EditorState};

pub struct ClientRendererPlugin {
    /// The name of the example, which must also match the edgegap application name.
    pub name: String,
}

impl ClientRendererPlugin {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Resource)]
struct GameName(String);

#[derive(Component)]
struct StatusMessageMarker;

#[derive(Component)]
pub(crate) struct ClientButton;

#[derive(Component)]
struct ModeText;

#[derive(Component)]
struct ToggleModeButton;

impl Plugin for ClientRendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameName(self.name.clone()));
        app.insert_resource(ClearColor::default());
        app.add_systems(Startup, set_window_title);
        spawn_connect_button(app);
        spawn_mode_controls(app);
        app.add_systems(Update, (update_button_text, update_mode_text));
        app.add_observer(on_update_status_message);
        app.add_observer(handle_connection);
        app.add_observer(handle_disconnection);
    }
}

fn set_window_title(mut window: Query<&mut Window>, game_name: Res<GameName>) {
    let mut window = window.single_mut().unwrap();
    window.title = format!("Vysma Editor: {}", game_name.0);
}

#[derive(Event, Debug)]
pub struct UpdateStatusMessage(pub String);

fn on_update_status_message(
    trigger: Trigger<UpdateStatusMessage>,
    mut q: Query<&mut Text, With<StatusMessageMarker>>,
) {
    for mut text in &mut q {
        text.0 = trigger.event().0.clone();
    }
}

/// Create a button that allow you to connect/disconnect to a server
pub(crate) fn spawn_connect_button(app: &mut App) {
    app.world_mut()
        .spawn(Node {
            width: Val::Percent(30.0),
            height: Val::Percent(30.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            justify_self: JustifySelf::End,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text("[Client]".to_string()),
                TextColor(Color::srgb(0.9, 0.9, 0.9).with_alpha(0.4)),
                TextFont::from_font_size(18.0),
                StatusMessageMarker,
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));
            parent
                .spawn((
                    Text("Connect".to_string()),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextFont::from_font_size(20.0),
                    ClientButton,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                ))
                .observe(
                    |_: Trigger<Pointer<Click>>,
                     mut commands: Commands,
                     query: Query<(Entity, &Client)>| {
                        let Ok((entity, client)) = query.single() else { return; };
                        match client.state {
                            ClientState::Disconnected => {
                                commands.trigger_targets(Connect, entity);
                            }
                            _ => {
                                commands.trigger_targets(Disconnect, entity);
                            }
                        };
                    },
                );
        });
}

fn spawn_mode_controls(app: &mut App) {
    app.world_mut()
        .spawn(Node {
            width: Val::Percent(50.0),
            height: Val::Px(60.0),
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text("Mode: Preview".to_string()),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextFont::from_font_size(18.0),
                ModeText,
            ));
            parent
                .spawn((
                    Text("Toggle Edit (F5)".to_string()),
                    TextColor(Color::srgb(0.1, 0.9, 0.9)),
                    TextFont::from_font_size(18.0),
                    ToggleModeButton,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                ))
                .observe(|_: Trigger<Pointer<Click>>, mut mode: ResMut<EditorState>| {
                    mode.0 = match mode.0 { EditorMode::Edit => EditorMode::Preview, EditorMode::Preview => EditorMode::Edit };
                    info!("HCL EditorMode -> {:?}", mode.0);
                });
            parent.spawn((
                Text("In Edit: F6 to publish demo, 1/2 to swap color".to_string()),
                TextColor(Color::srgb(0.8, 0.8, 0.8).with_alpha(0.7)),
                TextFont::from_font_size(14.0),
            ));
        });
}

pub(crate) fn update_button_text(
    client: Single<&Client>,
    mut text_query: Query<&mut Text, (With<Button>, With<ClientButton>)>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        match client.state {
            ClientState::Disconnecting => {
                text.0 = "Disconnecting".to_string();
            }
            ClientState::Disconnected => {
                text.0 = "Connect".to_string();
            }
            ClientState::Connecting => {
                text.0 = "Connecting".to_string();
            }
            ClientState::Connected => {
                text.0 = "Disconnect".to_string();
            }
        }
    }
}

fn update_mode_text(mode: Option<Res<EditorState>>, mut q: Query<&mut Text, With<ModeText>>) {
    if let Ok(mut text) = q.get_single_mut() {
        let m = mode.as_ref().map(|m| m.0).unwrap_or(EditorMode::Preview);
        text.0 = match m {
            EditorMode::Edit => "Mode: Edit".to_string(),
            EditorMode::Preview => "Mode: Preview".to_string(),
        };
    }
}

/// Component to identify the text displaying the client id
#[derive(Component)]
pub struct ClientIdText;

/// Listen for events to know when the client is connected, and spawn a text entity
/// to display the client id
pub(crate) fn handle_connection(
    trigger: Trigger<OnAdd, Connected>,
    query: Query<&LocalId, Or<((With<LinkOf>, With<Client>), Without<LinkOf>)>>,
    mut commands: Commands,
) {
    if let Ok(client_id) = query.get(trigger.target()) {
        commands.spawn((
            Text(format!("Client {}", client_id.0)),
            TextFont::from_font_size(30.0),
            ClientIdText,
        ));
    }
}

/// Listen for events to know when the client is disconnected, and print out the reason
/// of the disconnection
pub(crate) fn handle_disconnection(
    _trigger: Trigger<OnAdd, Disconnected>,
    mut commands: Commands,
    debug_text: Query<Entity, With<ClientIdText>>,
) {
    // TODO: add reason
    commands.trigger(UpdateStatusMessage(String::from("Disconnected")));
    for entity in debug_text.iter() {
        commands.entity(entity).despawn();
    }
}
