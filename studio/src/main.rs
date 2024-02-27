use bevy::{
    app::Update,
    window::{close_on_esc, Window},
};
use clap::Parser;
use studio::{screenshot::ScreenShotPlugin, InitAsset, InitParams, StudioMode};

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

    let object_id = args.model.unwrap_or("Mannequin.gltf".to_string());
    let skybox_id = args.skybox.unwrap_or("gum_trees_4k".to_string());

    // Edit the arguments here to change the default startup scene or skybox
    let mut app = studio::start(
        InitParams::FromAsset {
            asset: InitAsset::LibraryObject(object_id),
            skybox_id,
        },
        StudioMode::Editor,
        Some(Window {
            title: "Storyteller Studio".into(),
            ..Default::default()
        }),
    );

    // Add any additional systems, resources, etc. for local debugging
    app.add_systems(Update, close_on_esc);
    app.add_plugins(ScreenShotPlugin);

    app.run();
}
