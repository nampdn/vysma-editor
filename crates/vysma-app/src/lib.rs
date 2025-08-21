pub mod prelude {
    pub use bevy::prelude::*;
    pub use vysma_net as net;
    pub use vysma_hcl as hcl;
}

pub mod protocol;
pub mod input_binding;
pub use input_binding::InputBindingExt;
pub mod client;
pub mod server;
pub mod common;
pub mod renderer;
pub mod shared; 

#[cfg(all(feature = "gui", feature = "client"))]
pub use client::editor_ui::EditorUiPlugin;