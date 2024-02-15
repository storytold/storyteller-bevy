use bevy::{app::Update, window::close_on_esc};
use clap::Parser;
use studio::{screenshot::ScreenShotPlugin, StudioMode, ViewportSize};

#[derive(Parser)]
#[command(name = "StorytellerStudio")]
#[command(author = "The Storyteller Team")]
#[command(version = "1.0")]
#[command(about = "Executable for Storyteller Studio.")]
#[command(propagate_version = true)]
struct StudioArgs {
    model: Option<String>,
    skybox: Option<String>,
}

fn main() {
    let args = StudioArgs::parse();

    let model_path = args.model.unwrap_or("Mannequin.gltf".to_string());
    let skybox_path = args.skybox.unwrap_or("gum_trees_4k".to_string());

    // Edit the arguments here to change the default startup scene or skybox
    let mut app = studio::start_internal(
        model_path.as_str(),
        skybox_path.as_str(),
        "", // ignored for non-WASM
        "Storyteller Studio",
        ViewportSize::default(),
        StudioMode::Editor,
    );

    // Add any additional systems, resources, etc. for local debugging
    app.add_systems(Update, close_on_esc);
    app.add_plugins(ScreenShotPlugin);

    app.run();
}
