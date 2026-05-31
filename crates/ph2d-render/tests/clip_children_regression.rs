//! ClipChildren stencil-clip regression gate (Sprite_projeto §6.3 / W3
//! §8; ADR-0070-amendment-7 / ADR-0074-amendment-1).
//!
//! Godot shipped FIVE successive clip-regression bugs (#79885, #102190,
//! #102224, #91068, #90793), so the spec MANDATES a pixel-exact gate for
//! PH2D's silhouette clip. This renders a tiny headless scene — a 2×2
//! parent (the mask) plus a child sprite that pokes OUT of the parent
//! silhouette — once per `ClipMode`, then samples 4 canonical pixels:
//!
//! ```text
//!    world:        screen (64×64, 16 px/unit, center (0,0)):
//!    parent  [-1,1]²            → pixels [16,48]²   (the silhouette)
//!    child   x∈[-0.5,1.5] y∈[-1,1] → x∈[24,56] y∈[16,48]
//!
//!    (20,32) parent-only   — parent yes, child no
//!    (40,32) member-inside — parent + child overlap (inside silhouette)
//!    (52,32) member-outside— child yes, parent no (OUTSIDE silhouette)
//!    ( 4, 4) background     — neither
//! ```
//!
//! | pixel          | Disabled | ClipOnly | ClipAndDraw |
//! |----------------|----------|----------|-------------|
//! | parent-only    | WHITE    | black    | WHITE       |
//! | member-inside  | RED      | RED      | RED         |
//! | member-outside | RED      | black    | black       |
//! | background     | black    | black    | black       |
//!
//! `member-outside` proves the clip happens at all (RED → black); the
//! difference at `parent-only` (WHITE vs black) distinguishes ClipOnly
//! (invisible mould) from ClipAndDraw (parent draws too). A separate test
//! shows `alpha_cutoff` thresholds the silhouette.
//!
//! Skips gracefully (passes) on adapter-less CI; runs on dev Macs + Mac
//! CI, where the Enio smoke also runs.

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
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

/// A `w×h` texture of one solid RGBA value (uniform → interior sampling is
/// blend-stable regardless of filter mode).
fn solid_rgba(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        out.extend_from_slice(&rgba);
    }
    out
}

fn make_target(gpu: &GpuContext) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("clip regression color target"),
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

/// Copy the rendered color target back to a tightly-packed RGBA8 `Vec`
/// (strips wgpu's 256-byte row padding). Mirrors `IndividualTextureStore`
/// readback.
fn readback(gpu: &GpuContext, texture: &wgpu::Texture) -> Vec<u8> {
    let unpadded = W * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("clip regression staging"),
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

fn px(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
}

/// Assert the RGB of pixel `(x, y)` is within ±18 of `expected` (the test
/// colors are pure 0/255 channels; the tolerance only absorbs
/// rounding/filter ULPs). Alpha is ignored — premultiplied output keeps it
/// at ~255 everywhere.
fn assert_rgb(pixels: &[u8], x: u32, y: u32, expected: [u8; 3], what: &str) {
    let got = px(pixels, x, y);
    for c in 0..3 {
        let (a, b) = (got[c] as i32, expected[c] as i32);
        assert!(
            (a - b).abs() <= 18,
            "{what} @ ({x},{y}) channel {c}: got {got:?}, expected ~{expected:?}"
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn instance(
    texture_id: u32,
    world_pos: [f32; 2],
    size: [f32; 2],
    z: u32,
    clip_group: u32,
    clip_meta: u32,
) -> RenderInstance {
    RenderInstance {
        world_pos,
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
        clip_group,
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
fn clip_children_three_modes_4px() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping clip_children_three_modes_4px: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let parent_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 255, 255, 255]))
        .expect("parent tex");
    let child_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 0, 0, 255]))
        .expect("child tex");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    // Members carry no cutoff (only the mask source's matters).
    let member_meta = 0u32;

    // Modes: (clip_group, parent_role). Disabled = no clip (group 0).
    let disabled = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[
            instance(parent_tex, [0.0, 0.0], [2.0, 2.0], 0, 0, 0),
            instance(child_tex, [0.5, 0.0], [2.0, 2.0], 1, 0, 0),
        ],
    );
    assert_rgb(&disabled, 20, 32, [255, 255, 255], "Disabled parent-only");
    assert_rgb(&disabled, 40, 32, [255, 0, 0], "Disabled member-inside");
    assert_rgb(&disabled, 52, 32, [255, 0, 0], "Disabled member-outside");
    assert_rgb(&disabled, 4, 4, [0, 0, 0], "Disabled background");

    let clip_only_mask =
        RenderInstance::pack_clip_meta(RenderInstance::CLIP_ROLE_MASK_CLIP_ONLY, 0.5);
    let clip_only = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[
            instance(parent_tex, [0.0, 0.0], [2.0, 2.0], 0, 1, clip_only_mask),
            instance(child_tex, [0.5, 0.0], [2.0, 2.0], 1, 1, member_meta),
        ],
    );
    assert_rgb(
        &clip_only,
        20,
        32,
        [0, 0, 0],
        "ClipOnly parent-only (mould invisible)",
    );
    assert_rgb(&clip_only, 40, 32, [255, 0, 0], "ClipOnly member-inside");
    assert_rgb(
        &clip_only,
        52,
        32,
        [0, 0, 0],
        "ClipOnly member-outside (clipped)",
    );
    assert_rgb(&clip_only, 4, 4, [0, 0, 0], "ClipOnly background");

    let clip_draw_mask =
        RenderInstance::pack_clip_meta(RenderInstance::CLIP_ROLE_MASK_CLIP_AND_DRAW, 0.5);
    let clip_and_draw = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[
            instance(parent_tex, [0.0, 0.0], [2.0, 2.0], 0, 1, clip_draw_mask),
            instance(child_tex, [0.5, 0.0], [2.0, 2.0], 1, 1, member_meta),
        ],
    );
    assert_rgb(
        &clip_and_draw,
        20,
        32,
        [255, 255, 255],
        "ClipAndDraw parent-only",
    );
    assert_rgb(
        &clip_and_draw,
        40,
        32,
        [255, 0, 0],
        "ClipAndDraw member-inside",
    );
    assert_rgb(
        &clip_and_draw,
        52,
        32,
        [0, 0, 0],
        "ClipAndDraw member-outside (clipped)",
    );
    assert_rgb(&clip_and_draw, 4, 4, [0, 0, 0], "ClipAndDraw background");
}

#[test]
fn clip_member_survives_interloping_z_order() {
    // W3 §8 audit (HIGH): a clip group's instances must stay CONTIGUOUS in
    // the sorted buffer or the clip pass's consecutive-run span scan splits
    // the group and the members render against an unmarked stencil → VANISH.
    // Here a foreign non-clip sprite X (clip_group 0) gets a z_order rank
    // BETWEEN the clip-parent (z 0) and the member (z 2). The renderer's
    // clip-anchor sort must still keep [parent, member] adjacent so the
    // member clips correctly. (Pre-fix this asserted RED→black: the member
    // disappeared.)
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping clip_member_survives_interloping_z_order: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let parent_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 255, 255, 255]))
        .expect("parent");
    let child_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 0, 0, 255]))
        .expect("child");
    let interloper_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [0, 0, 255, 255]))
        .expect("interloper");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    let mask = RenderInstance::pack_clip_meta(RenderInstance::CLIP_ROLE_MASK_CLIP_ONLY, 0.5);
    let pixels = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &[
            // clip-parent (z 0) + member (z 2) in clip_group 1 ...
            instance(parent_tex, [0.0, 0.0], [2.0, 2.0], 0, 1, mask),
            instance(child_tex, [0.5, 0.0], [2.0, 2.0], 2, 1, 0),
            // ... with a foreign non-clip sprite whose rank (z 1) interleaves
            // between them. Placed at a top corner, off the sampled rows.
            instance(interloper_tex, [1.3, 1.3], [0.3, 0.3], 1, 0, 0),
        ],
    );
    assert_rgb(
        &pixels,
        40,
        32,
        [255, 0, 0],
        "interloped member-inside still clips RED",
    );
    assert_rgb(
        &pixels,
        52,
        32,
        [0, 0, 0],
        "interloped member-outside still clipped",
    );
    assert_rgb(
        &pixels,
        20,
        32,
        [0, 0, 0],
        "interloped ClipOnly mould still invisible",
    );
}

#[test]
fn clip_cutoff_thresholds_silhouette() {
    // A HALF-alpha (α≈0.5) parent in ClipOnly: a low cutoff (0.2) keeps the
    // silhouette (member draws inside), a high cutoff (0.8) discards it
    // entirely (member clipped to nothing). Proves `alpha_cutoff` carves
    // the mask (spec §6.4 / §6.8).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping clip_cutoff_thresholds_silhouette: no headless GPU");
        return;
    };
    let atlas = TextureAtlas::new(&gpu, 256);
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let parent_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 255, 255, 128]))
        .expect("half-alpha parent");
    let child_tex = renderer
        .acquire_individual(8, 8, &solid_rgba(8, 8, [255, 0, 0, 255]))
        .expect("child tex");
    let target = make_target(&gpu);
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(W, H);

    let scene = |cut: f32| {
        let mask = RenderInstance::pack_clip_meta(RenderInstance::CLIP_ROLE_MASK_CLIP_ONLY, cut);
        [
            instance(parent_tex, [0.0, 0.0], [2.0, 2.0], 0, 1, mask),
            instance(child_tex, [0.5, 0.0], [2.0, 2.0], 1, 1, 0),
        ]
    };

    let low = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &scene(0.2),
    );
    assert_rgb(
        &low,
        40,
        32,
        [255, 0, 0],
        "cutoff 0.2 keeps silhouette → member shows",
    );

    let high = render_pixels(
        &gpu,
        &mut renderer,
        &target,
        &view,
        &camera,
        window,
        &scene(0.8),
    );
    assert_rgb(
        &high,
        40,
        32,
        [0, 0, 0],
        "cutoff 0.8 discards silhouette → member clipped",
    );
}
