use bevy::{
    app::AppExit,
    core_pipeline::{core_3d, prepass::ViewPrepassTextures},
    prelude::*,
    render::{
        render_graph::{RenderGraphApp, RenderLabel, ViewNode, ViewNodeRunner},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, ImageCopyBuffer, ImageDataLayout, MapMode,
        },
        renderer::RenderDevice,
        view::ViewTarget,
        Extract, Render, RenderSet,
    },
    tasks::Task,
};
use bytemuck::AnyBitPattern;
use image::{
    error::UnsupportedErrorKind, EncodableLayout, ImageBuffer, ImageError, Pixel,
    PixelWithColorType, Rgba,
};
use std::sync::Mutex;
use wgpu::{Extent3d, MaintainBase};

use crate::scene::SceneState;

//==============================================================================
//                        Screen Shot Plugin
//==============================================================================

pub struct ScreenShotPlugin;

impl Plugin for ScreenShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_screen_capture_on_extract_step).run_if(in_state(SceneState::Active)),
        );

        let Ok(render_world) = app.get_sub_app_mut(bevy::render::RenderApp) else {
            return;
        };

        render_world
            .add_systems(ExtractSchedule, extract_screen_capture_components)
            .add_systems(
                Render,
                (
                    prepare_screen_capture_buffers.in_set(RenderSet::PrepareResources),
                    save_buffer_to_file
                        .after(RenderSet::Render)
                        .before(RenderSet::Cleanup),
                ),
            )
            //Render Graph Additions
            .add_render_graph_node::<ViewNodeRunner<ScreenCaptureNode>>(
                core_3d::graph::Core3d,
                ScreenCaptureNode,
            )
            .add_render_graph_edge(
                core_3d::graph::Core3d,
                core_3d::graph::Node3d::Tonemapping,
                ScreenCaptureNode,
            )
            .add_render_graph_edge(
                core_3d::graph::Core3d,
                ScreenCaptureNode,
                core_3d::graph::Node3d::EndMainPassPostProcessing,
            );
    }
}

//==============================================================================
//                        Sceen Shot Component
//==============================================================================

#[derive(Component, Clone)]
pub struct ScreenCapture {
    frames: u32,
    max_frames: u32,
    to_directory: String,
    capture_depth: bool,
    currently_captureing: bool,
}

impl ScreenCapture {
    pub fn from_out(output_dir: &str) -> Self {
        ScreenCapture {
            frames: 0,
            to_directory: output_dir.to_string(),
            capture_depth: false,
            currently_captureing: false,
            max_frames: u32::MAX,
        }
    }

    pub fn with_max_frames(mut self, max_frames: u32) -> Self {
        self.max_frames = max_frames;
        self
    }

    pub fn capture_depth(mut self) -> Self {
        self.capture_depth = true;
        self
    }

    pub fn is_capturing(&self) -> bool {
        self.currently_captureing
    }

    pub fn stop_capturing(&mut self) {
        self.currently_captureing = false;
    }

    pub fn start_capturing(&mut self) {
        self.currently_captureing = true;
    }

    pub fn should_capture(&self) -> bool {
        self.frames <= self.max_frames && self.is_capturing()
    }

    pub fn frames_captured(&self) -> u32 {
        self.frames
    }

    pub fn max_frames(&self) -> u32 {
        self.max_frames
    }
}

fn update_screen_capture_on_extract_step(
    mut screen_captures: Query<&mut ScreenCapture>,
    mut exit_event: EventWriter<AppExit>,
) {
    let loading_count;
    {
        let mut queue = QUEUE_SAVE.lock().unwrap();
        queue.retain(|task| !task.is_finished());
        loading_count = queue.len();
    }

    for mut screen_capture in screen_captures.iter_mut() {
        screen_capture.frames += 1;

        if !screen_capture.should_capture() && loading_count == 0 {
            exit_event.send(AppExit);
        }
        // if screen_capture.number_of_frames == 0 {
        //     commands.entity(entity).remove::<ScreenCapture>();
        // }
    }
}

//This system extracts the ScreenCaptureComponent to the render world.
fn extract_screen_capture_components(
    mut commands: Commands,
    screen_capure_comps: Extract<Query<(Entity, &ScreenCapture), With<Camera>>>,
    scene_state: Extract<Res<State<SceneState>>>,
) {
    if *scene_state.get() != SceneState::Active {
        return;
    };
    for (entity, screen_capture) in screen_capure_comps.iter() {
        if screen_capture.should_capture() {
            info!(
                "Capturing Frame: {} / {}",
                screen_capture.frames, screen_capture.max_frames
            );
            commands.get_or_spawn(entity).insert(screen_capture.clone());
        }
    }
}

//==============================================================================
//                        ScreenCaptureBuffers
//==============================================================================

#[derive(Component)]
struct ScreenCaptureBuffers {
    main_buffer: Buffer,
    main_buffer_extent: Extent3d,
    main_bytes_per_row: u32,
    main_padded_bytes_per_row: u32,
}

//During this function, we are making the buffers for the screenshot to be saved in
//This is where we are instatiating the ScreenCaptureBuffers component.
fn prepare_screen_capture_buffers(
    mut commands: Commands,
    views: Query<(Entity, &ViewTarget, &ScreenCapture)>,
    device: Res<RenderDevice>,
) {
    for (entity, view_target, screen_capture) in views.iter() {
        if screen_capture.frames > 0 {
            let texture = view_target.main_texture();
            let size = texture.size();
            let format = texture.format();
            let bytes_per_row =
                (size.width / format.block_dimensions().0) * format.block_copy_size(None).unwrap();
            let padded_bytes_per_row =
                RenderDevice::align_copy_bytes_per_row(bytes_per_row as usize) as u32;

            let main_buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                size: (size.height * bytes_per_row) as u64,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            commands.entity(entity).insert(ScreenCaptureBuffers {
                main_buffer,
                main_buffer_extent: size,
                main_bytes_per_row: bytes_per_row,
                main_padded_bytes_per_row: padded_bytes_per_row,
            });
        }
    }
}

//==============================================================================
//                        Screen Shot View Node
//==============================================================================

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel, Default)]
struct ScreenCaptureNode;

impl ViewNode for ScreenCaptureNode {
    type ViewQuery = (
        Entity,
        &'static ViewTarget,
        &'static ScreenCapture,
        &'static ScreenCaptureBuffers,
        Option<&'static ViewPrepassTextures>,
    );

    fn run(
        &self,
        _: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        view_query: bevy::ecs::query::QueryItem<Self::ViewQuery>,
        _: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        render_context.command_encoder().copy_texture_to_buffer(
            view_query.1.main_texture().as_image_copy(),
            ImageCopyBuffer {
                buffer: &view_query.3.main_buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(view_query.3.main_padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            view_query.1.main_texture().size(),
        );
        Ok(())
    }
}

//==============================================================================
//                        Saving the buffer to file
//==============================================================================

static QUEUE_SAVE: Mutex<Vec<Task<()>>> = Mutex::new(Vec::new());

fn save_buffer_to_file(
    mut frames_to_skip: Local<Option<u32>>,
    buffers: Query<(&ScreenCapture, &ScreenCaptureBuffers)>,
    device: Res<RenderDevice>,
) {
    for (screen_capture, buffers) in buffers.iter() {
        // ALERT!! ALERT!! Filthy Hack in effect
        // Engine does not load scene in time for the first 3 frames
        // So we skip them. Oops :(
        // TODO: Fix the Race Condition and properly take screenshots.
        let frames_skipped = frames_to_skip.unwrap_or(3);

        if frames_skipped > 0 {
            println!("Skipping Frame");
            *frames_to_skip = Some(frames_skipped - 1);
            return;
        }
        // Filthy Hack Ends

        if buffers.main_bytes_per_row != buffers.main_padded_bytes_per_row {
            println!("These don't match!");
        }

        let main_image_values = {
            let main_buffer_slice = buffers.main_buffer.slice(..);

            let (tx, rx) = futures::channel::oneshot::channel();
            main_buffer_slice.map_async(MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            device.poll(MaintainBase::Wait);
            futures_lite::future::block_on(rx).unwrap().unwrap();

            main_buffer_slice.get_mapped_range().to_vec()
        };

        std::fs::create_dir_all(&screen_capture.to_directory)
            .expect("Output path could not be created");

        let path = format!(
            "{}img_{:0>4}.png",
            screen_capture.to_directory, screen_capture.frames
        );

        let pool = bevy::tasks::AsyncComputeTaskPool::get();
        let extent = buffers.main_buffer_extent;
        let task = pool.spawn(async move {
            save_buffer::<Rgba<u8>>(main_image_values, extent, path.clone());
        });
        QUEUE_SAVE.lock().unwrap().push(task);
    }
}

fn save_buffer<P: Pixel + PixelWithColorType>(
    image_bytes: Vec<P::Subpixel>,
    source_size: Extent3d,
    path: String,
) where
    P::Subpixel: AnyBitPattern,
    [P::Subpixel]: EncodableLayout,
{
    match ImageBuffer::<P, _>::from_raw(source_size.width, source_size.height, image_bytes) {
        Some(buffer) => {
            if let Err(ImageError::Unsupported(err)) = buffer.save(path) {
                if let UnsupportedErrorKind::Format(hint) = err.kind() {
                    println!("Image format {} is not supported", hint);
                }
            }
        }
        None => {
            println!("Failed creating image buffer for '{}'", path);
        }
    }
}
