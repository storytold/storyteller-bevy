use bevy::{app::Update, window::close_on_esc};
use studio::{StudioMode, ViewportSize};

fn main() {
    // Edit the arguments here to change the default startup scene or skybox
    let mut app = studio::start_internal(
        "sample-room",
        "gum_trees_4k",
        "", // ignored for non-WASM
        "Storyteller Studio",
        ViewportSize::default(),
        StudioMode::Editor,
    );

    // Add any additional systems, resources, etc. for local debugging
    app.add_systems(Update, close_on_esc);

    app.run();
}
