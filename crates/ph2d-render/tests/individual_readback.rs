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
    let instance = GpuContext::default_instance();
    GpuContext::new(instance, None).ok()
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
