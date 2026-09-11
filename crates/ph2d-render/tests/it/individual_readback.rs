//! GPU round-trip: acquire individual texture → readback → bytes match.
//!
//! Exercises the Trim Transparency / Background Removal edit-path
//! contract — when a sprite is already on an individual texture, the
//! shell needs to recover its pixels to feed the next edit. Headless
//! GPU mirrors the convention in `atlas.rs::tests::try_headless_gpu`:
//! `None` from the adapter skips the test gracefully in CI runners
//! that don't expose a GPU.

use ph2d_gpu::GpuContext;
use ph2d_render::{IndividualTextureError, IndividualTextureStore};

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

/// Build the `material_bgl` the way `SpriteRenderer` does (texture +
/// sampler). Kept inline because re-using `SpritePipeline::new` here
/// would pull the full renderer dependency chain into a test that
/// only needs the store.
fn material_bgl(gpu: &GpuContext) -> wgpu::BindGroupLayout {
    gpu.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("test material bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
}

fn checkerboard_rgba(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let on = (x / 8 + y / 8) % 2 == 0;
            let v = if on { 220 } else { 30 };
            out[idx] = v;
            out[idx + 1] = (v as u32 / 2) as u8;
            out[idx + 2] = ((255 - v as u32) / 2) as u8;
            out[idx + 3] = 255;
        }
    }
    out
}

#[test]
fn acquire_and_readback_round_trips_pixels() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping: no headless GPU adapter");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);

    // Pick dimensions that exercise row padding: `64 * 4 = 256` bytes
    // per row IS the alignment, so no padding. We want a case WITH
    // padding too — use 7×5 (28 bytes per row, padded to 256).
    for (w, h) in [(64u32, 64u32), (7u32, 5u32), (300u32, 200u32)] {
        let input = checkerboard_rgba(w, h);
        let id = store.acquire(&gpu, &bgl, w, h, &input).expect("acquire");
        let (ow, oh, output) = store.readback(&gpu, id).expect("readback");
        assert_eq!((ow, oh), (w, h));
        assert_eq!(
            output.len(),
            (w * h * 4) as usize,
            "readback for {w}×{h} returned {} bytes (expected {})",
            output.len(),
            w * h * 4
        );
        assert_eq!(
            output, input,
            "readback bytes differ from acquire bytes for {w}×{h}"
        );
        let _ = store.release(id);
    }
}

#[test]
fn mip_chain_downsamples_in_linear_light() {
    // The 2026-06-17 trilinear-minification fix: acquiring a texture must fill its
    // whole mip chain by a 2× box downsample IN LINEAR LIGHT. A half-white /
    // half-black 8×8 image (power-of-2 split stays half/half at every level) must
    // bottom out at a 1×1 mip that is the LINEAR average (0.5 linear → sRGB ≈ 188),
    // NOT the naïve 8-bit sRGB midpoint (128). That gap is the whole point —
    // averaging sRGB bytes directly is the classic too-dark downsample bug.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping: no headless GPU adapter");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);

    let (w, h) = (8u32, 8u32);
    let mut input = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let v = if x < w / 2 { 255 } else { 0 }; // left white, right black
            input[idx] = v;
            input[idx + 1] = v;
            input[idx + 2] = v;
            input[idx + 3] = 255; // opaque (premul-irrelevant at full alpha)
        }
    }
    let id = store.acquire(&gpu, &bgl, w, h, &input).expect("acquire");

    // mip 3 of an 8×8 texture is 1×1 — the average of the whole image.
    let (mw, mh, bottom) = store.readback_mip(&gpu, id, 3).expect("readback mip 3");
    assert_eq!((mw, mh), (1, 1), "8×8 → mip 3 is 1×1");
    let r = bottom[0];
    assert!(
        (180..=196).contains(&r),
        "1×1 mip must be the LINEAR average of white+black ≈ 188, got {r} \
         (128 would mean a wrong sRGB-space average)"
    );
    assert_eq!(bottom[3], 255, "alpha average of opaque image stays opaque");
    let _ = store.release(id);
}

#[test]
fn readback_unknown_id_yields_not_found() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping: no headless GPU adapter");
        return;
    };
    let store = IndividualTextureStore::new(&gpu);
    match store.readback(&gpu, 999) {
        Err(IndividualTextureError::NotFound(999)) => {}
        other => panic!("expected NotFound(999), got {other:?}"),
    }
}

#[test]
fn replace_pixels_region_updates_only_the_subrect() {
    // Painter W3 dirty-rect (item 1a): a partial upload must change only
    // the bounding box a stroke touched, leaving every other pixel intact.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping: no headless GPU adapter");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);

    let (w, h) = (32u32, 24u32);
    let base = vec![10u8; (w * h * 4) as usize]; // uniform value 10 everywhere
    let id = store.acquire(&gpu, &bgl, w, h, &base).expect("acquire");

    // Overwrite an 8×8 sub-rect at (4,2) with 0xFF.
    let (rx, ry, rw, rh) = (4u32, 2u32, 8u32, 8u32);
    let patch = vec![0xFFu8; (rw * rh * 4) as usize];
    store
        .replace_pixels_region(&gpu, id, rx, ry, rw, rh, &patch)
        .expect("region replace");

    let (ow, oh, out) = store.readback(&gpu, id).expect("readback");
    assert_eq!(
        (ow, oh),
        (w, h),
        "dims must stay stable across a region write"
    );
    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * 4) as usize;
            let in_patch = x >= rx && x < rx + rw && y >= ry && y < ry + rh;
            let expected = if in_patch { 0xFF } else { 10 };
            assert_eq!(
                out[idx], expected,
                "pixel ({x},{y}) in_patch={in_patch}: got {} expected {expected}",
                out[idx]
            );
        }
    }
    let _ = store.release(id);
}

#[test]
fn replace_pixels_region_rejects_out_of_bounds_and_bad_length() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping: no headless GPU adapter");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);
    let (w, h) = (16u32, 16u32);
    let id = store
        .acquire(&gpu, &bgl, w, h, &vec![0u8; (w * h * 4) as usize])
        .expect("acquire");

    // Sub-rect past the right edge (10 + 8 > 16).
    let oob = store.replace_pixels_region(&gpu, id, 10, 0, 8, 4, &[0u8; 8 * 4 * 4]);
    assert!(
        matches!(oob, Err(IndividualTextureError::RegionOutOfBounds { .. })),
        "expected RegionOutOfBounds, got {oob:?}"
    );

    // In-bounds rect but the pixel slice is the wrong length.
    let bad_len = store.replace_pixels_region(&gpu, id, 0, 0, 4, 4, &[0u8; 7]);
    assert!(
        matches!(
            bad_len,
            Err(IndividualTextureError::PixelLengthMismatch { .. })
        ),
        "expected PixelLengthMismatch, got {bad_len:?}"
    );

    // A zero-area dirty-rect is a clean no-op.
    assert!(
        store
            .replace_pixels_region(&gpu, id, 0, 0, 0, 0, &[])
            .is_ok(),
        "zero-area region must be a no-op"
    );

    // An unknown id is a silent no-op (mirror of replace_pixels).
    assert!(
        store
            .replace_pixels_region(&gpu, 999, 0, 0, 4, 4, &[0u8; 64])
            .is_ok(),
        "unknown id must be a silent no-op"
    );

    let _ = store.release(id);
}

/// Clear-on-alloc guard: a slot acquired EMPTY (`acquire_empty`, the Painter GPU-preview path) must read
/// back fully TRANSPARENT, never undefined GPU memory. Without the `clear_all_mips_transparent` on
/// creation, a frame sampling the slot before its first fill shows garbage — the "thin sliver of the
/// shape appears on the first paint of a region" artifact (HANDOFF_per_layer_color_perf_artifacts §1.R).
#[test]
fn acquire_empty_slot_reads_back_transparent_not_garbage() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("skipping: no headless GPU adapter");
        return;
    };
    let bgl = material_bgl(&gpu);
    let mut store = IndividualTextureStore::new(&gpu);
    // Sizes that exercise row padding + a non-power-of-two (so the mip chain has odd levels).
    for (w, h) in [(64u32, 64u32), (300u32, 200u32), (7u32, 5u32)] {
        let id = store.acquire_empty(&gpu, &bgl, w, h);
        let (ow, oh, output) = store.readback(&gpu, id).expect("readback empty slot");
        assert_eq!((ow, oh), (w, h));
        assert!(
            output.iter().all(|&b| b == 0),
            "empty slot {w}×{h} must be cleared transparent on alloc, found non-zero bytes \
             (first at {:?}) — clear-on-alloc regressed",
            output.iter().position(|&b| b != 0)
        );
        let _ = store.release(id);
    }
}
