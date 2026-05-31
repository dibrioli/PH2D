//! BlendMode compositing regression gate (Sprite Inspector v2 §10).
//!
//! Renders an opaque mid-gray BACKGROUND sprite (z=0, Mix) then an opaque
//! mid-gray FOREGROUND sprite (z=1) carrying a given [`BlendMode`] tag in
//! `flip_uv` bits 5-7, and samples the overlapped centre pixel (32,32).
//!
//! **Compositing is LINEAR-space** (textures sample sRGB→linear, the
//! `Rgba8Unorm` target stores linear), so the byte 128 sprite is linear
//! `0.216` (`srgb_to_linear(0.502)`), and the readback byte is
//! `round(linear · 255)`. With both colors linear 0.216 each mode lands
//! on a distinct value:
//!
//! | mode       | formula (linear, F=B=0.216) | byte |
//! |------------|-----------------------------|------|
//! | Mix        | F                           | 55   |
//! | Add        | min(B+F, 1) = 0.432         | 110  |
//! | Subtract   | max(B−F, 0) = 0             | 0    |
//! | Multiply   | F·B = 0.047                 | 12   |
//! | Screen     | 1−(1−B)(1−F) = 0.386        | 98   |
//! | PremultAlpha | F (premult over)          | 55   |
//!
//! Ordering is the real proof: Multiply < Mix < Screen < Add, and
//! Subtract bottoms out — i.e. each pipeline composites as advertised.
//! Skips gracefully on adapter-less CI.

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
        label: Some("blend regression color target"),
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
        label: Some("blend regression staging"),
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

fn channel(pixels: &[u8], x: u32, y: u32) -> u8 {
    let i = ((y * W + x) * 4) as usize;
    pixels[i] // gray, so R == G == B
}

const BLACK: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// Full-cover sprite at `z` with `blend_tag` packed into `flip_uv` 5-7.
fn instance(texture_id: u32, z: u32, blend_tag: u8) -> RenderInstance {
    RenderInstance {
        world_pos: [0.0, 0.0],
        size: [8.0, 8.0],
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        basis: RenderInstance::IDENTITY_BASIS,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: RenderInstance::pack_blend_bits(blend_tag),
        texture_id,
        z_order: z,
        sampling: RenderInstance::SAMPLING_DEFAULT,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
    }
}

#[test]
fn blend_modes_composite_as_advertised() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping blend_modes_composite_as_advertised: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    // Two opaque mid-gray (0.5) textures — same value so each mode lands
    // on a distinct composite (see module table).
    let gray = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("bg tex");
    let fg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [128, 128, 128, 255]))
        .expect("fg tex");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    let render_mode = |renderer: &mut SpriteRenderer, fg_blend: u8| -> u8 {
        let mut present = PresentWorld::new();
        present.world_mut().spawn(instance(gray, 0, 0)); // background, Mix
        present.world_mut().spawn(instance(fg, 1, fg_blend)); // foreground, mode
        renderer.render(&view, &mut present, &camera, window, BLACK);
        let px = readback(&gpu, &target);
        channel(&px, 32, 32)
    };

    // Linear-space expectations (see module table).
    let mix = render_mode(&mut renderer, 0);
    assert!((mix as i32 - 55).abs() <= 14, "Mix centre {mix}, expected ~55");
    let add = render_mode(&mut renderer, 1);
    assert!((add as i32 - 110).abs() <= 16, "Add centre {add}, expected ~110");
    assert!(add > mix, "Add must lighten vs Mix");
    let sub = render_mode(&mut renderer, 2);
    assert!(sub <= 14, "Subtract centre {sub}, expected ~0 (darkest)");
    let mul = render_mode(&mut renderer, 3);
    assert!((mul as i32 - 12).abs() <= 14, "Multiply centre {mul}, expected ~12");
    assert!(mul < mix, "Multiply must darken vs Mix");
    let screen = render_mode(&mut renderer, 4);
    assert!((screen as i32 - 98).abs() <= 16, "Screen centre {screen}, expected ~98");
    assert!(screen > mix && screen < add, "Screen lightens but softer than Add");
    let premult = render_mode(&mut renderer, 5);
    assert!((premult as i32 - 55).abs() <= 14, "PremultAlpha centre {premult}, expected ~55");
}

#[test]
fn absent_blend_is_zero_regression_over() {
    // tag 0 (Mix, the absent-component default) composites an opaque
    // foreground as a plain "over" — identical to the pre-§10 renderer.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping absent_blend_is_zero_regression_over: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let bg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [40, 40, 40, 255]))
        .expect("bg");
    let fg = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [200, 200, 200, 255]))
        .expect("fg");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);
    let mut present = PresentWorld::new();
    present.world_mut().spawn(instance(bg, 0, 0));
    present.world_mut().spawn(instance(fg, 1, 0));
    renderer.render(&view, &mut present, &camera, window, BLACK);
    let px = readback(&gpu, &target);
    let c = channel(&px, 32, 32);
    // fg byte 200 → linear 0.573 → readback byte ~146 (linear target).
    assert!((c as i32 - 146).abs() <= 16, "opaque fg over bg = fg (linear ~146), got {c}");
}
