use bevy::{prelude::*, render::render_resource::Extent3d, transform::components::Transform};
use clap::{Args, Parser, Subcommand};
use studio::{
    headless::{start_headless, CameraMotion, HeadlessMode},
    scene::SceneState,
};

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
        camera_motion: Option<String>,
        #[arg(long)]
        camera_motion_speed: Option<f32>,
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
            camera_motion,
            camera_motion_speed,
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
            let skybox = skybox.unwrap_or("gum_trees_4k".to_string());
            let mut app = start_headless(
                HeadlessMode::Scene(scene_dir, skybox),
                fps,
                &output_dir,
                frames,
                output_res,
            );

            if let Some(camera_motion) = camera_motion {
                let camera_motion_speed = camera_motion_speed.unwrap_or(1.0);
                match camera_motion.as_str() {
                    "orbit" => {
                        app.insert_resource(CameraMotion::Orbit(camera_motion_speed));
                    }
                    "pan" => {
                        app.insert_resource(CameraMotion::Pan(camera_motion_speed));
                    }
                    "zoom" => {
                        app.insert_resource(CameraMotion::Zoom(camera_motion_speed));
                    }
                    "static" => {
                        app.insert_resource(CameraMotion::Static);
                    }
                    _ => panic!("Unsupported camera motion type."),
                }

                app.add_systems(
                    Update,
                    update_camera_motion.run_if(in_state(SceneState::Active)),
                );
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

pub(crate) fn update_camera_motion(
    mut camera: Query<&mut Transform, (With<Camera>, Without<Parent>)>,
    camera_motion: Option<Res<CameraMotion>>,
    time: Res<Time>,
) {
    let Some(camera_motion) = camera_motion else {
        return;
    };
    println!("{}", camera.iter().count());
    let Ok(mut main_cam_transform) = camera.get_single_mut() else {
        return;
    };

    println!("Updating camera motion");

    match *camera_motion {
        CameraMotion::Orbit(speed) => {
            orbit_camera(&mut main_cam_transform, &time, speed);
        }
        CameraMotion::Pan(speed) => {
            pan_camera(&mut main_cam_transform, &time, speed);
        }
        CameraMotion::Zoom(speed) => {
            zoom_camera(&mut main_cam_transform, &time, speed);
        }
        CameraMotion::Static => {
            main_cam_transform.translation = Vec3::new(0.340, 1.1, -0.5);
            main_cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
        }
    }
    // main_cam_transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn zoom_camera(main_cam_transform: &mut Transform, time: &Time<()>, speed: f32) {
    let scaled_time = time.elapsed().as_secs_f32() * speed;
    let normalized_cos = (-scaled_time.cos() + 1.0) / 2.0;
    let start_pos = Vec3::new(0.366, 1.1, 2.5);
    let end_pos = Vec3::new(0.366, 1.1, -0.5);

    main_cam_transform.translation = start_pos.lerp(end_pos, normalized_cos);
    main_cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
}

fn pan_camera(main_cam_transform: &mut Transform, time: &Time<()>, speed: f32) {
    let scaled_time = time.elapsed().as_secs_f32() * speed;
    let normalized_cos = (-scaled_time.cos() + 1.0) / 2.0;
    let start_pos = Vec3::new(0.676, 1.1, 0.119);
    let end_pos = Vec3::new(0.067, 1.1, 0.121);

    main_cam_transform.translation = start_pos.lerp(end_pos, normalized_cos);
    main_cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
}

fn orbit_camera(main_cam_transform: &mut Mut<'_, Transform>, time: &Res<'_, Time>, speed: f32) {
    let scaled_time = time.elapsed().as_secs_f32() * speed;
    let max_angle = std::f32::consts::PI / 2.0;
    let normalized_cos = (scaled_time.cos() + 1.0) / 2.0;
    let angle = max_angle * normalized_cos;

    // let t = (time.elapsed().as_secs_f32() / 5.0) % 1.0;

    main_cam_transform.translation = Vec3::new(-2.2 * angle.cos(), 1.0, 2.5 * angle.sin());
    main_cam_transform.look_at(Vec3::new(0.347, 0.556, -1.0), Vec3::Y);
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
