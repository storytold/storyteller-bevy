use bevy::{
    core_pipeline::{core_3d, prepass::ViewPrepassTextures},
    prelude::*,
    render::{
        render_graph::{RenderGraphApp, ViewNode, ViewNodeRunner},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, ImageCopyBuffer, ImageDataLayout, MapMode,
        },
        renderer::RenderDevice,
        view::ViewTarget,
        Extract, Render, RenderSet,
    },
};
use bytemuck::AnyBitPattern;
use image::{
    error::UnsupportedErrorKind, EncodableLayout, ImageBuffer, ImageError, Pixel,
    PixelWithColorType, Rgba,
};
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
            (test_system, update_screen_capture_on_extract_step)
                .run_if(in_state(SceneState::Active)),
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
                core_3d::graph::NAME,
                ScreenCaptureNode::NAME,
            )
            .add_render_graph_edges(
                core_3d::graph::NAME,
                &[
                    core_3d::graph::node::TONEMAPPING,
                    ScreenCaptureNode::NAME,
                    core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                ],
            );
    }
}

fn test_system(
    mut commands: Commands,
    camera: Query<Entity, With<Camera>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F10) {
        for entity in camera.iter() {
            commands
                .entity(entity)
                .insert(ScreenCapture::number_of_frames(16, "./scrnshot/"));
        }
    }
}

//==============================================================================
//                        Sceen Shot Component
//==============================================================================

#[derive(Component, Clone)]
pub struct ScreenCapture {
    number_of_frames: u32,
    to_directory: String,
    capture_depth: bool,
}

impl ScreenCapture {
    pub fn number_of_frames(frames: u32, output_dir: &str) -> Self {
        ScreenCapture {
            number_of_frames: frames,
            to_directory: output_dir.to_string(),
            capture_depth: false,
        }
    }

    pub fn capture_depth(mut self) -> Self {
        self.capture_depth = true;
        self
    }

    pub fn is_complete(&self) -> bool {
        println!("Number of Frames Left: {}", self.number_of_frames);
        self.number_of_frames == 1
    }
}

fn update_screen_capture_on_extract_step(
    mut commands: Commands,
    mut screen_captures: Query<(Entity, &mut ScreenCapture)>,
) {
    for (entity, mut screen_capture) in screen_captures.iter_mut() {
        screen_capture.number_of_frames -= 1;
        if screen_capture.number_of_frames == 0 {
            commands.entity(entity).remove::<ScreenCapture>();
        }
    }
}

//This system extracts the ScreenCaptureComponent to the render world.
fn extract_screen_capture_components(
    mut commands: Commands,
    screen_capure_comps: Extract<Query<(Entity, &ScreenCapture), With<Camera>>>,
) {
    for (entity, screen_capture) in screen_capure_comps.iter() {
        commands.get_or_spawn(entity).insert(screen_capture.clone());
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
        if screen_capture.number_of_frames > 0 {
            let texture = view_target.main_texture();
            let size = texture.size();
            let format = texture.format();
            let bytes_per_row =
                (size.width / format.block_dimensions().0) * format.block_size(None).unwrap();
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

            println!("Buffers Created!");
        }
    }
}

//==============================================================================
//                        Screen Shot View Node
//==============================================================================

#[derive(Default)]
struct ScreenCaptureNode;

impl ScreenCaptureNode {
    pub const NAME: &'static str = "ScreenCaptureNode";
}

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
        println!("Saved to buffer!");
        Ok(())
    }
}

//==============================================================================
//                        Saving the buffer to file
//==============================================================================

fn save_buffer_to_file(
    buffers: Query<(&ScreenCapture, &ScreenCaptureBuffers)>,
    device: Res<RenderDevice>,
) {
    for (screen_capture, buffers) in buffers.iter() {
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
            "{}img_{:0.5}.png",
            screen_capture.to_directory, screen_capture.number_of_frames
        );

        save_buffer::<Rgba<u8>>(&main_image_values, &buffers.main_buffer_extent, &path);
    }
}

fn save_buffer<P: Pixel + PixelWithColorType>(
    image_bytes: &[P::Subpixel],
    source_size: &Extent3d,
    path: &str,
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
