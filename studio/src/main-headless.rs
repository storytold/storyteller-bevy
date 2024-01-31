use std::time::Duration;

use bevy::{
    app::{ScheduleRunnerPlugin, Update},
    ecs::system::CommandQueue,
    prelude::Commands,
    prelude::*,
    render::render_resource::Extent3d,
    window::close_on_esc,
};
use clap::{Args, Parser, Subcommand};
use studio::{screen_shot::ScreenShotPlugin, CommonPlugins, StudioMode, ViewportSize};

#[derive(Parser)]
#[command(name = "StorytellerStudioHeadless")]
#[command(author = "The Storyteller Team")]
#[command(version = "1.0")]
#[command(
    about = "Executable for Storyteller Studio Headless. Allows you to render frames to images."
)]
#[command(propagate_version = true)]
struct StudioArgs {
    output_dir: Option<String>,
    #[arg(value_parser = extent3d_parser)]
    output_resolution: Option<Extent3d>,
    #[arg(value_parser = clap::value_parser!(u16).range(1..))]
    fps: Option<u32>,
}

#[derive(Subcommand)]
enum StudioSubcommands {
    Scene {},
    Media {},
}

#[derive(Subcommand)]
enum StudioSubCommand {
    Render {
        output_dir: Option<String>,
        #[arg(value_parser = extent3d_parser)]
        output_resolution: Option<Extent3d>,
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        fps: Option<u32>,
    },
    Start {
        model: Option<String>,
        skybox: Option<String>,
    },
}

fn main() {
    let args = StudioArgs::parse();

    let output_dir = args.output_dir.unwrap_or("./".to_string());
    let output_res = args.output_resolution.unwrap_or(Extent3d {
        width: 1280,
        height: 720,
        depth_or_array_layers: 1,
    });
    let fps = args.fps.unwrap_or(24);

    studio::start_headless("Mannequin.gltf", "gum_trees_4k", fps, 300, output_res).run();
}

//==============================================================================
//                        Parsers
//==============================================================================

fn extent3d_parser(input: &str) -> Result<Extent3d, String> {
    let split = input.split('x').collect::<Vec<_>>();

    if split.len() != 2 {
        return Err(format!("Invalid format for the Resolution argument, must follow the following pattern: <WIDTH>x<HEIGHT>"));
    }

    let Ok(width) = split[0].parse::<u32>() else {
        return Err(format!(
            "Width must be a number when defining the output resolution."
        ));
    };

    let Ok(height) = split[0].parse::<u32>() else {
        return Err(format!(
            "Width must be a number when defining the output resolution."
        ));
    };

    Ok(Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    })
}
