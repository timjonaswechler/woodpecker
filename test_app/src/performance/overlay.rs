//! Tiny bitmap overlay drawn after the cameras and before screenshot copies.
//! No UI layout, assets, fonts, input processing or simulation schedules are run.
use bevy::{
    prelude::*,
    render::{
        RenderStartup,
        camera::ExtractedCamera,
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue},
        view::ViewTarget,
    },
};

const COLUMNS: usize = 44;
const ROWS: usize = 6;
const BUFFER_SIZE: u64 = (16 + COLUMNS * ROWS * 16) as u64;

#[derive(Resource, Default)]
pub(super) struct Text {
    pub target: Option<bevy::camera::NormalizedRenderTarget>,
    pub size: UVec2,
    pub enabled: bool,
    pub lines: String,
    pub revision: u64,
}

#[derive(Resource)]
struct Gpu {
    buffer: Buffer,
    layout: BindGroupLayout,
    bind_group: BindGroup,
    pipeline: Option<(TextureFormat, RenderPipeline)>,
    prepared: Option<(u64, [u32; 4])>,
}

pub(super) fn install(render: &mut SubApp) {
    render
        .init_resource::<Text>()
        .add_systems(RenderStartup, initialize)
        .add_systems(
            RenderGraph,
            draw.after(RenderGraphSystems::Render)
                .before(RenderGraphSystems::Submit),
        );
}

fn initialize(mut commands: Commands, device: Res<RenderDevice>) {
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("performance overlay"),
        size: BUFFER_SIZE,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(
        "performance overlay",
        &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    );
    let bind_group = device.create_bind_group(
        "performance overlay",
        &layout,
        &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    );
    commands.insert_resource(Gpu {
        buffer,
        layout,
        bind_group,
        pipeline: None,
        prepared: None,
    });
}

fn pipeline(
    device: &RenderDevice,
    layout: &BindGroupLayout,
    format: TextureFormat,
) -> RenderPipeline {
    let shader = device.create_and_validate_shader_module(ShaderModuleDescriptor {
        label: Some("performance overlay"),
        source: ShaderSource::Wgsl(include_str!("overlay.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("performance overlay"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&RawRenderPipelineDescriptor {
        label: Some("performance overlay"),
        layout: Some(&layout),
        vertex: RawVertexState {
            module: &shader,
            entry_point: Some("vertex"),
            compilation_options: default(),
            buffers: &[],
        },
        fragment: Some(RawFragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            compilation_options: default(),
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: default(),
        depth_stencil: None,
        multisample: default(),
        multiview_mask: None,
        cache: None,
    })
}

fn draw(
    text: Res<Text>,
    mut gpu: ResMut<Gpu>,
    views: Query<(&ViewTarget, &ExtractedCamera)>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut context: RenderContext,
) {
    if !text.enabled {
        return;
    }
    let Some((target, _)) = views.iter().find(|(target, camera)| {
        camera.target.as_ref() == text.target.as_ref() && target.out_texture().is_some()
    }) else {
        return;
    };
    let Some(view) = target.out_texture() else {
        return;
    };
    let Some(format) = target.out_texture_view_format() else {
        return;
    };
    let scale = if text.size.x >= 640 { 2 } else { 1 };
    let width = (COLUMNS as u32 * 6 + 16) * scale;
    let height = (ROWS as u32 * 9 + 12) * scale;
    if text.size.x < width || text.size.y < height {
        return;
    }
    let x = text.size.x.saturating_sub(width + 24);
    let y = text.size.y.saturating_sub(height + 110);
    let geometry = [x, y, scale, 0];
    if gpu.prepared != Some((text.revision, geometry)) {
        let mut bytes = Vec::with_capacity(BUFFER_SIZE as usize);
        for value in geometry {
            bytes.extend(value.to_le_bytes());
        }
        let lines: Vec<_> = text.lines.lines().collect();
        for row in 0..ROWS {
            let line = lines.get(row).copied().unwrap_or("").as_bytes();
            for column in 0..COLUMNS {
                let bits = glyph(line.get(column).copied().unwrap_or(b' '));
                for value in [bits as u32, (bits >> 32) as u32, 0, 0] {
                    bytes.extend(value.to_le_bytes());
                }
            }
        }
        queue.write_buffer(&gpu.buffer, 0, &bytes);
        gpu.prepared = Some((text.revision, geometry));
    }
    if gpu
        .pipeline
        .as_ref()
        .is_none_or(|(cached, _)| *cached != format)
    {
        gpu.pipeline = Some((format, pipeline(&device, &gpu.layout, format)));
    }
    let mut pass = context
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("performance overlay"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(&gpu.pipeline.as_ref().unwrap().1);
    pass.set_bind_group(0, &*gpu.bind_group, &[]);
    pass.set_scissor_rect(x, y, width, height);
    pass.draw(0..3, 0..1);
}

fn glyph(character: u8) -> u64 {
    // Five columns, seven bits per column, least significant bit at the top.
    let columns = match character {
        b'A' => [0x7e, 0x11, 0x11, 0x11, 0x7e],
        b'B' => [0x7f, 0x49, 0x49, 0x49, 0x36],
        b'C' => [0x3e, 0x41, 0x41, 0x41, 0x22],
        b'D' => [0x7f, 0x41, 0x41, 0x22, 0x1c],
        b'E' => [0x7f, 0x49, 0x49, 0x49, 0x41],
        b'F' => [0x7f, 0x09, 0x09, 0x09, 0x01],
        b'G' => [0x3e, 0x41, 0x49, 0x49, 0x7a],
        b'H' => [0x7f, 0x08, 0x08, 0x08, 0x7f],
        b'I' => [0x00, 0x41, 0x7f, 0x41, 0x00],
        b'J' => [0x20, 0x40, 0x41, 0x3f, 0x01],
        b'K' => [0x7f, 0x08, 0x14, 0x22, 0x41],
        b'L' => [0x7f, 0x40, 0x40, 0x40, 0x40],
        b'M' => [0x7f, 0x02, 0x0c, 0x02, 0x7f],
        b'N' => [0x7f, 0x04, 0x08, 0x10, 0x7f],
        b'O' => [0x3e, 0x41, 0x41, 0x41, 0x3e],
        b'P' => [0x7f, 0x09, 0x09, 0x09, 0x06],
        b'Q' => [0x3e, 0x41, 0x51, 0x21, 0x5e],
        b'R' => [0x7f, 0x09, 0x19, 0x29, 0x46],
        b'S' => [0x46, 0x49, 0x49, 0x49, 0x31],
        b'T' => [0x01, 0x01, 0x7f, 0x01, 0x01],
        b'U' => [0x3f, 0x40, 0x40, 0x40, 0x3f],
        b'V' => [0x1f, 0x20, 0x40, 0x20, 0x1f],
        b'W' => [0x3f, 0x40, 0x38, 0x40, 0x3f],
        b'X' => [0x63, 0x14, 0x08, 0x14, 0x63],
        b'Y' => [0x07, 0x08, 0x70, 0x08, 0x07],
        b'Z' => [0x61, 0x51, 0x49, 0x45, 0x43],
        b'0' => [0x3e, 0x51, 0x49, 0x45, 0x3e],
        b'1' => [0x00, 0x42, 0x7f, 0x40, 0x00],
        b'2' => [0x42, 0x61, 0x51, 0x49, 0x46],
        b'3' => [0x21, 0x41, 0x45, 0x4b, 0x31],
        b'4' => [0x18, 0x14, 0x12, 0x7f, 0x10],
        b'5' => [0x27, 0x45, 0x45, 0x45, 0x39],
        b'6' => [0x3c, 0x4a, 0x49, 0x49, 0x30],
        b'7' => [0x01, 0x71, 0x09, 0x05, 0x03],
        b'8' => [0x36, 0x49, 0x49, 0x49, 0x36],
        b'9' => [0x06, 0x49, 0x49, 0x29, 0x1e],
        b'.' => [0, 0x60, 0x60, 0, 0],
        b':' => [0, 0x36, 0x36, 0, 0],
        b'-' => [8, 8, 8, 8, 8],
        b'/' => [0x20, 0x10, 8, 4, 2],
        _ => [0; 5],
    };
    columns
        .into_iter()
        .enumerate()
        .fold(0, |bits, (column, value)| {
            bits | ((value as u64) << (column * 7))
        })
}
