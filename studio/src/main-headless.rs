use bevy::{prelude::*, render::render_resource::Extent3d, transform::components::Transform};
use clap::{Args, Parser, Subcommand};
use studio::headless::{start_headless, HeadlessMode};

#[derive(Parser)]
#[command(name = "StorytellerStudioHeadless")]
#[command(author = "The Storyteller Team")]
#[command(version = "1.0")]
#[command(
    about = "Executable for Storyteller Studio Headless. Allows you to render frames to images."
)]
#[command(propagate_version = true)]
struct StudioArgs {
    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Scene {
        scene_dir: String,
        #[arg(long)]
        orbit: bool,
        #[arg(long, value_parser = clap::value_parser!(f32))]
        orbit_radius: Option<f32>,
        #[arg(long, value_parser = clap::value_parser!(f32))]
        orbit_speed: Option<f32>,
        #[arg(long, short)]
        output_dir: Option<String>,
        #[arg(long)]
        skybox: Option<String>,
        #[arg(long = "res", value_parser = extent3d_parser)]
        output_resolution: Option<Extent3d>,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        fps: Option<u32>,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        frames: Option<u32>,
    },
    Bvh {
        bvh_dir: String,
        #[arg(long, short = 'o', short)]
        out: Option<String>,
        #[arg(long)]
        skybox: Option<String>,
        #[arg(long = "res", value_parser = extent3d_parser)]
        output_resolution: Option<Extent3d>,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        fps: Option<u32>,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        frames: Option<u32>,
    },
    Mixamo {
        mixamo_dir: String,
        #[arg(long, short = 'o', short)]
        out: Option<String>,
        #[arg(long)]
        skybox: Option<String>,
        #[arg(long = "res", value_parser = extent3d_parser)]
        output_resolution: Option<Extent3d>,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        fps: Option<u32>,
        #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
        frames: Option<u32>,
    },
}

#[derive(Args, Clone)]
struct CommonArgs {}

fn main() {
    let args = StudioArgs::parse();

    let mut app = match args.subcommand {
        Subcommands::Scene {
            scene_dir,
            orbit,
            orbit_speed,
            orbit_radius,
            skybox,
            output_dir,
            output_resolution,
            fps,
            frames,
        } => {
            let output_dir = output_dir.unwrap_or("./screen_shots/".to_string());
            let output_res = output_resolution.unwrap_or(Extent3d {
                width: 1280,
                height: 720,
                depth_or_array_layers: 1,
            });
            let fps = fps.unwrap_or(24);
            let frames = frames.unwrap_or(30);
            let skybox = skybox.unwrap_or("gum_trees_4k".to_string());
            let mut app = start_headless(
                HeadlessMode::Scene(scene_dir, skybox),
                fps,
                &output_dir,
                frames,
                output_res,
            );
            let orbit_speed = orbit_speed.unwrap_or(1.0);
            let orbit_radius = orbit_radius.unwrap_or(8.0);

            if orbit {
                app.insert_resource(OrbitSettings {
                    radius: orbit_radius,
                    speed: orbit_speed,
                })
                .add_systems(Update, orbit_scene);
            }

            app
        }
        Subcommands::Bvh {
            bvh_dir,
            skybox,
            out: output_dir,
            output_resolution,
            fps,
            frames,
        } => {
            let output_dir = output_dir.unwrap_or("./screen_shots/".to_string());
            let output_res = output_resolution.unwrap_or(Extent3d {
                width: 1280,
                height: 720,
                depth_or_array_layers: 1,
            });
            let fps = fps.unwrap_or(24);
            let frames = frames.unwrap_or(u32::MAX);
            let skybox = skybox.unwrap_or("gum_trees_4k".to_string());
            start_headless(
                HeadlessMode::Bvh(bvh_dir, skybox),
                fps,
                &output_dir,
                frames,
                output_res,
            )
        }
        Subcommands::Mixamo {
            mixamo_dir,
            out,
            skybox,
            output_resolution,
            fps,
            frames,
        } => {
            let output_dir = out.unwrap_or("./screen_shots/".to_string());
            let output_res = output_resolution.unwrap_or(Extent3d {
                width: 1280,
                height: 720,
                depth_or_array_layers: 1,
            });
            let fps = fps.unwrap_or(24);
            let frames = frames.unwrap_or(u32::MAX);
            let skybox = skybox.unwrap_or("gum_trees_4k".to_string());
            start_headless(
                HeadlessMode::Mixamo(mixamo_dir, skybox),
                fps,
                &output_dir,
                frames,
                output_res,
            )
        }
    };

    app.run();
}

#[derive(Resource)]
struct OrbitSettings {
    radius: f32,
    speed: f32,
}

fn orbit_scene(
    mut camera: Query<&mut Transform, With<Camera>>,
    orbit_settings: Res<OrbitSettings>,
    time: Res<Time>,
) {
    let Ok(mut main_cam_transform) = camera.get_single_mut() else {
        return;
    };

    main_cam_transform.translation = Vec3::new(
        orbit_settings.radius * (time.elapsed_seconds() * orbit_settings.speed).cos(),
        6.0,
        orbit_settings.radius * (time.elapsed_seconds() * orbit_settings.speed).sin(),
    );
    main_cam_transform.look_at(Vec3::ZERO, Vec3::Y);
}

//==============================================================================
//                        Parsers
//==============================================================================

fn extent3d_parser(input: &str) -> Result<Extent3d, String> {
    let split = input.split('x').collect::<Vec<_>>();

    if split.len() != 2 {
        return Err(
            "Invalid format for the Resolution argument, must follow the following pattern: <WIDTH>x<HEIGHT>"
                .to_string()
        );
    }

    let Ok(width) = split[0].parse::<u32>() else {
        return Err("Width must be a number when defining the output resolution.".to_string());
    };

    let Ok(height) = split[1].parse::<u32>() else {
        return Err("Width must be a number when defining the output resolution.".to_string());
    };

    Ok(Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    })
}
