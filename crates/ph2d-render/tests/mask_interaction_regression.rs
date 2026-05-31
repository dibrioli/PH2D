//! Mask2D / MaskInteraction stencil regression gate (spec §6.4/§6.6).
//!
//! A `Mask2D` SOURCE (opaque 2×2 square at the centre → silhouette pixels
//! [16,48]²) plus a larger responder (3×3 → [8,56]²) that pokes out past
//! the mask. Sampled headless at 3 pixels:
//!
//! ```text
//!    (32,32) inside-mask   — inside both mask + responder
//!    (12,32) outside-mask  — inside responder, OUTSIDE the mask silhouette
//!    ( 4, 4) background     — neither
//! ```
//!
//! | pixel        | VisibleInside | VisibleOutside | source-only (no responder) |
//! |--------------|---------------|----------------|----------------------------|
//! | inside-mask  | RED           | black          | black (source draws no color) |
//! | outside-mask | black         | RED            | black |
//! | background   | black         | black          | black |
//!
//! Proves: Inside responders appear ONLY where the mask is; Outside ONLY
//! where it isn't; and the Mask2D source itself never paints (it's a
//! mould). Skips gracefully on adapter-less CI.

use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, RenderInstance, SpriteRenderer, TextureAtlas};

const W: u32 = 64;
const H: u32 = 64;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn solid_rgba(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        out.extend_from_slice(&rgba);
    }
    out
}

fn make_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mask regression color target"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn readback(gpu: &GpuContext, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mask regression staging"),
        size: (padded * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    rx.recv().expect("map channel").expect("map ok");
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * H) as usize);
    for row in 0..H as usize {
        let start = row * padded as usize;
        out.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

fn assert_rgb(pixels: &[u8], x: u32, y: u32, expected: [u8; 3], what: &str) {
    let i = ((y * W + x) * 4) as usize;
    let got = [pixels[i], pixels[i + 1], pixels[i + 2]];
    for c in 0..3 {
        assert!(
            (got[c] as i32 - expected[c] as i32).abs() <= 18,
            "{what} @ ({x},{y}) ch {c}: got {got:?}, expected ~{expected:?}"
        );
    }
}

fn instance(texture_id: u32, size: [f32; 2], z: u32, clip_meta: u32) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id,
        z_order: z,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta,
    }
}

const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

fn render_pixels(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    target: &wgpu::Texture,
    view: &wgpu::TextureView,
    camera: &Camera2d,
    window: WindowSize,
    instances: &[RenderInstance],
) -> Vec<u8> {
    let mut present = PresentWorld::new();
    for inst in instances {
        present.world_mut().spawn(*inst);
    }
    renderer.render(view, &mut present, camera, window, BLACK);
    readback(gpu, target)
}

#[test]
fn mask_interaction_inside_outside_3px() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping mask_interaction_inside_outside_3px: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let mask_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 255, 255, 255]))
        .expect("mask tex");
    let resp_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 0, 0, 255]))
        .expect("responder tex");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    // Source: cutoff 0.5 in the shared bits + SOURCE role. z=0 (marks first).
    let source_meta = RenderInstance::with_mask_role(
        RenderInstance::pack_clip_meta(0, 0.5),
        RenderInstance::MASK_ROLE_SOURCE,
    );
    let inside_meta = RenderInstance::with_mask_role(0, RenderInstance::MASK_ROLE_INSIDE);
    let outside_meta = RenderInstance::with_mask_role(0, RenderInstance::MASK_ROLE_OUTSIDE);

    // VisibleInside: responder shows ONLY inside the mask silhouette.
    let inside = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[
            instance(mask_tex, [2.0, 2.0], 0, source_meta),
            instance(resp_tex, [3.0, 3.0], 1, inside_meta),
        ],
    );
    assert_rgb(&inside, 32, 32, [255, 0, 0], "Inside @ inside-mask");
    assert_rgb(
        &inside,
        12,
        32,
        [0, 0, 0],
        "Inside @ outside-mask (clipped)",
    );
    assert_rgb(&inside, 4, 4, [0, 0, 0], "Inside @ background");

    // VisibleOutside: responder shows ONLY outside the mask silhouette.
    let outside = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[
            instance(mask_tex, [2.0, 2.0], 0, source_meta),
            instance(resp_tex, [3.0, 3.0], 1, outside_meta),
        ],
    );
    assert_rgb(
        &outside,
        32,
        32,
        [0, 0, 0],
        "Outside @ inside-mask (clipped)",
    );
    assert_rgb(&outside, 12, 32, [255, 0, 0], "Outside @ outside-mask");
    assert_rgb(&outside, 4, 4, [0, 0, 0], "Outside @ background");

    // Source-only: the Mask2D paints NO color (it's a mould).
    let source_only = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[instance(mask_tex, [2.0, 2.0], 0, source_meta)],
    );
    assert_rgb(
        &source_only,
        32,
        32,
        [0, 0, 0],
        "source-only @ inside-mask (invisible)",
    );
}
