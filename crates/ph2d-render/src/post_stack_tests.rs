//! Tests for [`super`] (the app HDR post-stack, ADR-0145). CPU maths gates run
//! everywhere; the device gates no-op on an adapter-less runner (the `#[ignore]` GPU
//! parity gate runs on the RTX with `-- --ignored`).

use super::*;

// ------------------------------------------------------------------ neutrality

#[test]
fn the_default_grade_is_neutral() {
    assert!(GradeParams::default().is_neutral());
    assert!(GradeParams::NEUTRAL.is_neutral());
}

#[test]
fn each_perturbed_knob_breaks_neutrality() {
    // The shell SKIPS the pass at neutral (byte-identity). If `is_neutral` wrongly
    // returned true for any of these, that knob would silently do nothing.
    for p in [
        GradeParams {
            exposure: 0.1,
            ..GradeParams::NEUTRAL
        },
        GradeParams {
            contrast: 1.1,
            ..GradeParams::NEUTRAL
        },
        GradeParams {
            saturation: 0.9,
            ..GradeParams::NEUTRAL
        },
        GradeParams {
            tint: [1.0, 0.9, 1.0],
            ..GradeParams::NEUTRAL
        },
        GradeParams {
            vignette: 0.01,
            ..GradeParams::NEUTRAL
        },
    ] {
        assert!(!p.is_neutral(), "should NOT be neutral: {p:?}");
    }
}

#[test]
fn vignette_shape_alone_is_still_neutral() {
    // With amount 0 the vignette factor is exactly 1 whatever radius/softness are, so a
    // grade differing only in those is neutral (the pass is skipped, correctly).
    let p = GradeParams {
        vignette_radius: 0.1,
        vignette_softness: 0.9,
        ..GradeParams::NEUTRAL
    };
    assert!(p.is_neutral());
}

// ------------------------------------------------------------------ grade maths

const CENTER: [f32; 2] = [0.5, 0.5];

#[test]
fn the_neutral_grade_is_the_identity() {
    // Not bit-exact (the shell skips it in production), but the maths must land back on
    // the input to a hair — a stray transform would show as a tint on a "neutral" frame.
    let c = grade_pixel(&GradeParams::NEUTRAL, [0.2, 0.5, 0.9], CENTER, 16.0 / 9.0);
    for (got, want) in c.iter().zip([0.2, 0.5, 0.9]) {
        assert!((got - want).abs() < 1e-6, "neutral moved {got} from {want}");
    }
}

#[test]
fn exposure_is_a_power_of_two_multiply() {
    // +1 stop doubles, -1 halves. Applied before everything, at the centre (no vignette).
    let up = grade_pixel(
        &GradeParams {
            exposure: 1.0,
            vignette: 0.0,
            ..GradeParams::NEUTRAL
        },
        [0.25, 0.25, 0.25],
        CENTER,
        1.0,
    );
    for x in up {
        assert!(
            (x - 0.5).abs() < 1e-5,
            "+1 stop should double 0.25 -> 0.5, got {x}"
        );
    }
    let down = grade_pixel(
        &GradeParams {
            exposure: -1.0,
            ..GradeParams::NEUTRAL
        },
        [0.4, 0.4, 0.4],
        CENTER,
        1.0,
    );
    for x in down {
        assert!(
            (x - 0.2).abs() < 1e-5,
            "-1 stop should halve 0.4 -> 0.2, got {x}"
        );
    }
}

#[test]
fn tint_multiplies_each_channel() {
    let c = grade_pixel(
        &GradeParams {
            tint: [1.0, 0.5, 0.25],
            ..GradeParams::NEUTRAL
        },
        [0.6, 0.6, 0.6],
        CENTER,
        1.0,
    );
    assert!((c[0] - 0.6).abs() < 1e-5);
    assert!((c[1] - 0.3).abs() < 1e-5);
    assert!((c[2] - 0.15).abs() < 1e-5);
}

#[test]
fn saturation_zero_greys_to_luma() {
    // sat 0 collapses every channel to the Rec.709 luma → a neutral grey.
    let rgb = [0.8, 0.2, 0.1];
    let c = grade_pixel(
        &GradeParams {
            saturation: 0.0,
            ..GradeParams::NEUTRAL
        },
        rgb,
        CENTER,
        1.0,
    );
    let luma = LUMA[0] * rgb[0] + LUMA[1] * rgb[1] + LUMA[2] * rgb[2];
    for x in c {
        assert!(
            (x - luma).abs() < 1e-5,
            "sat 0 should give luma {luma}, got {x}"
        );
    }
}

#[test]
fn contrast_pushes_around_the_pivot() {
    // A value ABOVE the pivot rises with contrast > 1; a value BELOW falls.
    let hi = grade_pixel(
        &GradeParams {
            contrast: 2.0,
            ..GradeParams::NEUTRAL
        },
        [0.5, 0.5, 0.5],
        CENTER,
        1.0,
    );
    assert!(hi[0] > 0.5, "above-pivot value should rise, got {}", hi[0]);
    let lo = grade_pixel(
        &GradeParams {
            contrast: 2.0,
            ..GradeParams::NEUTRAL
        },
        [0.05, 0.05, 0.05],
        CENTER,
        1.0,
    );
    assert!(lo[0] < 0.05, "below-pivot value should fall, got {}", lo[0]);
    // Contrast can push below 0 — the clamp keeps it non-negative.
    let neg = grade_pixel(
        &GradeParams {
            contrast: 20.0,
            ..GradeParams::NEUTRAL
        },
        [0.0, 0.0, 0.0],
        CENTER,
        1.0,
    );
    assert!(
        neg.iter().all(|&x| x >= 0.0),
        "clamp must keep non-negative, got {neg:?}"
    );
}

#[test]
fn the_vignette_darkens_the_corners_not_the_centre() {
    // The whole point of the feature: centre bright, corners dimmed. Full amount.
    let p = GradeParams {
        vignette: 1.0,
        vignette_radius: 0.2,
        vignette_softness: 0.6,
        ..GradeParams::NEUTRAL
    };
    let white = [1.0, 1.0, 1.0];
    let center = grade_pixel(&p, white, [0.5, 0.5], 1.0)[0];
    let corner = grade_pixel(&p, white, [0.0, 0.0], 1.0)[0];
    let corner2 = grade_pixel(&p, white, [1.0, 1.0], 1.0)[0];
    assert!(center > 0.99, "centre must stay bright, got {center}");
    assert!(corner < 0.5, "corner must be darkened, got {corner}");
    assert!(
        (corner - corner2).abs() < 1e-5,
        "opposite corners must dim equally: {corner} vs {corner2}"
    );
    // A wide frame stays round in PIXELS: at aspect 16:9 the horizontal mid-edge is
    // FARTHER (in the aspect-corrected metric) than the vertical mid-edge, so it dims
    // more — the aspect correction is doing its job, not stretching the vignette.
    let mid_h = grade_pixel(&p, white, [0.0, 0.5], 16.0 / 9.0)[0];
    let mid_v = grade_pixel(&p, white, [0.5, 0.0], 16.0 / 9.0)[0];
    assert!(
        mid_h < mid_v,
        "wide frame: side edge should dim more than top, {mid_h} vs {mid_v}"
    );
}

// ------------------------------------------------------------------ device gates

/// Headless GpuContext, cached per test binary (see the `game_rt`/`motion_fx` tests).
/// `None` on an adapter-less runner → device gates no-op there.
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

/// **The blank-screen guard.** Constructing `PostStack` compiles `post_stack.wgsl` and
/// builds the pipeline + bind group against a real device — a shader error, a layout
/// mismatch or a wrong texture format dies HERE, not as a broken grade at runtime.
/// `ensure_size` exercises the resize rebuild, and `grade` encodes the copy + fullscreen
/// pass; `poll(Wait)` drains it so any deferred validation surfaces before the test ends.
#[test]
fn the_grade_is_a_valid_pipeline_on_a_real_device() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut fx = PostStack::new(&gpu, (256, 256));
    fx.ensure_size(&gpu, (320, 200));
    let game = crate::GameRt::new(&gpu, (320, 200));
    let p = GradeParams {
        exposure: 0.5,
        vignette: 0.8,
        ..GradeParams::NEUTRAL
    };
    fx.grade(&gpu, game.texture(), game.view(), &p);
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
}

/// **GPU parity: the shader IS `grade_pixel`.** Clear a `game_rt` to a known linear
/// colour, grade it on the device, read it back, and compare a set of sample pixels to
/// the CPU reference. Drift between `post_stack.wgsl` and `grade_pixel` fails here.
/// `#[ignore]` → runs on the RTX with `-- --ignored` (needs a real adapter).
#[test]
#[ignore = "needs a real GPU adapter; run with -- --ignored"]
fn the_shader_matches_the_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no adapter — parity skipped");
        return;
    };
    const W: u32 = 64;
    const H: u32 = 64;
    let aspect = W as f32 / H as f32;
    // A non-trivial, non-neutral grade so every op moves the pixel.
    let p = GradeParams {
        exposure: 0.7,
        contrast: 1.3,
        saturation: 0.6,
        tint: [1.05, 0.98, 0.90],
        vignette: 0.9,
        vignette_radius: 0.25,
        vignette_softness: 0.55,
    };
    let input = [0.35_f32, 0.55, 0.75]; // linear scene colour to clear game_rt to

    let game = crate::GameRt::new(&gpu, (W, H));
    // Clear game_rt to `input` (a render pass with no draw writes the clear colour).
    {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear"),
            });
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear game_rt"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: game.view(),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: input[0] as f64,
                        g: input[1] as f64,
                        b: input[2] as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        gpu.queue.submit(Some(enc.finish()));
    }

    let fx = PostStack::new(&gpu, (W, H));
    fx.grade(&gpu, game.texture(), game.view(), &p);

    // Readback game_rt (Rgba16Float) into a buffer. 64px * 8 B/px = 512 B/row (already a
    // multiple of the 256 B row alignment), so no padding.
    let bytes_per_row = W * 8;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("post-stack readback"),
        size: (bytes_per_row * H) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("copy"),
            });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: game.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(H),
                },
            },
            wgpu::Extent3d {
                width: W,
                height: H,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit(Some(enc.finish()));
    }
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let data = slice.get_mapped_range();

    let px = |col: u32, row: u32| -> [f32; 3] {
        let base = (row * bytes_per_row + col * 8) as usize;
        let ch = |k: usize| {
            let bits = u16::from_le_bytes([data[base + k * 2], data[base + k * 2 + 1]]);
            half::f16::from_bits(bits).to_f32()
        };
        [ch(0), ch(1), ch(2)]
    };

    // Sample centre, four corners, mid-edges — where the vignette varies most.
    let samples = [
        (32u32, 32u32),
        (0, 0),
        (63, 0),
        (0, 63),
        (63, 63),
        (0, 32),
        (32, 0),
    ];
    let mut worst = 0.0_f32;
    for (col, row) in samples {
        let uv = [(col as f32 + 0.5) / W as f32, (row as f32 + 0.5) / H as f32];
        let want = grade_pixel(&p, input, uv, aspect);
        let got = px(col, row);
        for k in 0..3 {
            worst = worst.max((got[k] - want[k]).abs());
        }
        for k in 0..3 {
            assert!(
                (got[k] - want[k]).abs() < 5e-3,
                "px ({col},{row}) ch {k}: gpu {} vs cpu {} (worst so far {worst})",
                got[k],
                want[k]
            );
        }
    }
    eprintln!(
        "post-stack parity: worst delta {worst:.6} over {} samples",
        samples.len()
    );
    drop(data);
    buf.unmap();
}
