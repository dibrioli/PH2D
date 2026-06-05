//! GPU layer compositor — parity + perf gates (Painter W3, Block 2).
//!
//! These are `#[ignore]`d: they need a real device, so they run on a developer
//! machine / the GPU CI lane, not the headless CPU runners (which `None` out of
//! `try_headless_gpu`). They prove the WGSL compositor agrees with the blend
//! math source-of-truth `ph2d_painter_brush::apply_blend` (the same `apply`
//! the CPU `ph2d_tool_painter::compositor` uses), and that the perf / cap
//! budgets hold.
//!
//! Run all of them with:
//!   cargo test -p ph2d-render --test layer_compositor_gpu -- --ignored

use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use ph2d_gpu::GpuContext;
use ph2d_painter_brush::adjustments::{LevelsParams, levels_display_lut};
use ph2d_painter_brush::{BlendMode, MAX_BLEND_MODES, apply_blend};
use ph2d_render::{
    LayerCompositeError, LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
    SPATIAL_CHROMA, SPATIAL_GAUSSIAN, SPATIAL_MOTION, SPATIAL_SHARPEN, gaussian_weights,
    motion_weights,
};
use std::collections::BTreeMap;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// Test pixel provider: canvas-sized straight-sRGB8 buffers keyed by layer id.
#[derive(Default)]
struct MapProvider {
    layers: BTreeMap<u64, (u64, Vec<u8>)>,
}

impl MapProvider {
    fn insert(&mut self, key: u64, version: u64, bytes: Vec<u8>) {
        self.layers.insert(key, (version, bytes));
    }
    fn bytes(&self, key: u64) -> &[u8] {
        &self.layers.get(&key).expect("layer present").1
    }
}

impl LayerPixelProvider for MapProvider {
    fn layer_pixels(&self, key: u64) -> Option<LayerPixels<'_>> {
        self.layers.get(&key).map(|(v, b)| LayerPixels {
            version: *v,
            rgba8: b,
        })
    }
}

/// CPU reference for an `OP_ADJUSTMENT` op — mirrors the WGSL `apply_adjustment_op`
/// using the CANONICAL `ph2d_painter_brush::adjustments::apply_adjustment` (the
/// same fn the CPU compositor's Adjustment arm calls). The GPU↔CPU map of `kind`
/// code + `[f32;3]` params is the contract the painter tool's flatten emits.
fn cpu_adjust_op(code: u8, p: [f32; 3], blend: u8, opacity: f32, acc: [f32; 4]) -> [f32; 4] {
    use ph2d_painter_brush::adjustments::{
        AdjustmentParams, BrightnessContrastParams, ExposureParams, HsbParams, InvertParams,
        PosterizeParams, ThresholdParams, VibranceParams, apply_adjustment,
    };
    let params = match code {
        0 => AdjustmentParams::HueSaturationBrightness(HsbParams {
            h: p[0],
            s: p[1],
            b: p[2],
        }),
        1 => AdjustmentParams::BrightnessContrast(BrightnessContrastParams {
            brightness: p[0],
            contrast: p[1],
            legacy: false,
        }),
        2 => AdjustmentParams::Invert(InvertParams {}),
        3 => AdjustmentParams::Posterize(PosterizeParams { levels: p[0] as u8 }),
        4 => AdjustmentParams::Threshold(ThresholdParams {
            threshold: (p[0] * 255.0).round() as u8,
        }),
        5 => AdjustmentParams::Exposure(ExposureParams {
            exposure_ev: p[0],
            offset: p[1],
            gamma_correction: p[2],
        }),
        6 => AdjustmentParams::Vibrance(VibranceParams {
            vibrance: p[0],
            saturation: p[1],
        }),
        _ => return acc,
    };
    let kind = params.kind();
    let mut px = [acc];
    apply_adjustment(&kind, &params, &mut px);
    let src_px = [px[0][0], px[0][1], px[0][2], acc[3]];
    let blended = apply_blend(BlendMode::from_u8(blend), acc, src_px);
    let t = opacity.clamp(0.0, 1.0);
    [
        acc[0] + (blended[0] - acc[0]) * t,
        acc[1] + (blended[1] - acc[1]) * t,
        acc[2] + (blended[2] - acc[2]) * t,
        acc[3],
    ]
}

/// CPU reference — mirrors `layer_composite.wgsl`'s per-pixel stack machine
/// op-for-op, using the canonical `apply_blend` + sRGB transfer. This is the
/// math the GPU shader must reproduce (the tool's `composite` uses the same
/// `apply` over the same flattening).
fn cpu_composite(ops: &[LayerOp], prov: &MapProvider, w: u32, _h: u32, region: Region) -> Vec<u8> {
    let mut out = vec![0u8; (region.w as usize) * (region.h as usize) * 4];
    for ly in 0..region.h {
        for lx in 0..region.w {
            let gx = region.x + lx;
            let gy = region.y + ly;
            let mut stack = [[0.0f32; 4]; 9];
            let mut sp = 0usize;
            for op in ops {
                match op {
                    LayerOp::Layer {
                        key,
                        blend_mode,
                        opacity,
                    } => {
                        let b = prov.bytes(*key);
                        let i = ((gy * w + gx) * 4) as usize;
                        let mut s = [
                            srgb_to_linear_byte(b[i]),
                            srgb_to_linear_byte(b[i + 1]),
                            srgb_to_linear_byte(b[i + 2]),
                            b[i + 3] as f32 / 255.0,
                        ];
                        s[3] *= *opacity;
                        stack[sp] = apply_blend(BlendMode::from_u8(*blend_mode), stack[sp], s);
                    }
                    LayerOp::PushGroup => {
                        sp += 1;
                        stack[sp] = [0.0; 4];
                    }
                    LayerOp::PopGroup {
                        blend_mode,
                        opacity,
                    } => {
                        let mut sub = stack[sp];
                        sp -= 1;
                        sub[3] *= *opacity;
                        stack[sp] = apply_blend(BlendMode::from_u8(*blend_mode), stack[sp], sub);
                    }
                    LayerOp::Adjustment {
                        kind,
                        params,
                        blend_mode,
                        opacity,
                    } => {
                        stack[sp] = cpu_adjust_op(*kind, *params, *blend_mode, *opacity, stack[sp]);
                    }
                    // Spatial adjustments take the segmented pass-graph, not this
                    // single-pass reference — `gpu_gaussian_matches_cpu_reference`
                    // has its own materialise→blur→combine reference. Never present
                    // in op-lists handed to `cpu_composite`.
                    LayerOp::SpatialAdjustment { .. } => {}
                }
            }
            let acc = stack[0];
            let o = ((ly * region.w + lx) * 4) as usize;
            out[o] = linear_to_srgb_byte(acc[0]);
            out[o + 1] = linear_to_srgb_byte(acc[1]);
            out[o + 2] = linear_to_srgb_byte(acc[2]);
            out[o + 3] = (acc[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        }
    }
    out
}

/// A deterministic varied RGBA8 canvas (no two pixels alike → every blend
/// branch is exercised across the field).
fn varied_canvas(w: u32, h: u32, seed: u32) -> Vec<u8> {
    let mut v = vec![0u8; (w as usize) * (h as usize) * 4];
    for i in 0..(w * h) as usize {
        let p = i as u32;
        v[i * 4] = ((p.wrapping_mul(37).wrapping_add(seed.wrapping_mul(11))) % 256) as u8;
        v[i * 4 + 1] = ((p.wrapping_mul(53).wrapping_add(seed.wrapping_mul(29))) % 256) as u8;
        v[i * 4 + 2] = ((p.wrapping_mul(97).wrapping_add(seed.wrapping_mul(7))) % 256) as u8;
        // Keep alpha in [40, 255] so Behind/Clear see a non-trivial backdrop.
        v[i * 4 + 3] = (40 + (p.wrapping_mul(13).wrapping_add(seed) % 216)) as u8;
    }
    v
}

fn max_byte_diff(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.abs_diff(*y) as u32)
        .max()
        .unwrap_or(0)
}

/// Each of the 22 modes, isolated: backdrop + one layer with that mode, GPU vs
/// CPU reference. Catches a mis-ported formula or a wrong discriminant mapping
/// that a stacked test could mask. `shader_blend_modes_bit_identical_with_rust`
/// pins the literals; this proves the runtime output agrees within ±1 byte
/// (pow/sqrt are ULP-bounded across backends).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_matches_cpu_reference_each_mode() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (32u32, 32u32);
    let mut prov = MapProvider::default();
    prov.insert(0, 1, varied_canvas(w, h, 1)); // backdrop
    prov.insert(1, 1, varied_canvas(w, h, 2)); // top
    let mut comp = LayerCompositor::new(&gpu);
    let region = Region::full(w, h);

    for code in 0..MAX_BLEND_MODES {
        let ops = vec![
            LayerOp::Layer {
                key: 0,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::Layer {
                key: 1,
                blend_mode: code,
                opacity: 0.8,
            },
        ];
        comp.composite(&gpu, &ops, &prov, w, h, region)
            .expect("composite");
        let got = comp.read_output(&gpu).expect("readback");
        let want = cpu_composite(&ops, &prov, w, h, region);
        let d = max_byte_diff(&got, &want);
        assert!(
            d <= 1,
            "mode {} ({:?}) diverged from CPU reference by {d} bytes",
            code,
            BlendMode::from_u8(code),
        );
    }
}

/// A deep stack: many modes + a nested group + opacity, all at once. Proves the
/// stack machine (PushGroup / PopGroupBlend) composites groups identically to
/// the CPU recursion.
/// Each implemented adjustment kind, isolated: a varied backdrop + one
/// adjustment op (W4). Proves the WGSL adjustment kernels reproduce the
/// canonical `ph2d_painter_brush::adjustments::apply_adjustment` within
/// tolerance. OKLab uses `pow(x, 1/3)` on the GPU vs libm `cbrt` on the CPU,
/// and the display-space kinds use `pow` vs the CPU LUT — both ULP-bounded, so
/// the bound (±4 bytes) is wider than the ±1 of the pure-blend gate but still
/// comfortably sub-perceptual; it catches a mis-ported formula or wrong code.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_adjustment_matches_cpu_reference_each_kind() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let region = Region::full(w, h);
    // (kind code, params) for each implemented kind, deliberately non-neutral.
    let cases: &[(u8, [f32; 3])] = &[
        (0, [0.15, 0.4, 0.1]), // HSB
        (1, [0.2, 0.3, 0.0]),  // Brightness/Contrast
        (2, [0.0, 0.0, 0.0]),  // Invert
        (3, [6.0, 0.0, 0.0]),  // Posterize (6 levels)
        (4, [0.5, 0.0, 0.0]),  // Threshold (cut 0.5)
        (5, [1.0, 0.05, 0.2]), // Exposure (+1 EV, +offset, +gamma)
        (6, [0.6, 0.2, 0.0]),  // Vibrance
    ];
    let mut comp = LayerCompositor::new(&gpu);
    for &(code, params) in cases {
        let mut prov = MapProvider::default();
        prov.insert(1, 1, varied_canvas(w, h, 5));
        let ops = vec![
            LayerOp::Layer {
                key: 1,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::Adjustment {
                kind: code,
                params,
                blend_mode: 0,
                opacity: 1.0,
            },
        ];
        comp.composite(&gpu, &ops, &prov, w, h, region)
            .expect("composite");
        let got = comp.read_output(&gpu).expect("readback");
        let want = cpu_composite(&ops, &prov, w, h, region);
        let diff = max_byte_diff(&got, &want);
        assert!(
            diff <= 4,
            "adjustment kind {code}: GPU vs CPU max byte diff {diff}"
        );
    }
    // Partial opacity must lerp the effect toward the base (the arm's opacity).
    // Version 2 (not 1): `comp` is reused from the loop above, which cached key 1
    // at version 1 — a stale-version reuse would skip the re-upload (the cache is
    // correct; the test must bump the version when the pixels change).
    let mut prov = MapProvider::default();
    prov.insert(1, 2, varied_canvas(w, h, 9));
    let ops = vec![
        LayerOp::Layer {
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Adjustment {
            kind: 1,
            params: [0.6, 0.5, 0.0],
            blend_mode: 0,
            opacity: 0.5,
        },
    ];
    comp.composite(&gpu, &ops, &prov, w, h, region)
        .expect("composite");
    let got = comp.read_output(&gpu).expect("readback");
    let want = cpu_composite(&ops, &prov, w, h, region);
    let d = max_byte_diff(&got, &want);
    assert!(
        d <= 4,
        "partial-opacity adjustment parity: max byte diff {d}"
    );
}

/// W4 §2 — GPU display-space transfer LUTs (binding 6 `adj_luts`), indexed by
/// the op's `params[0]` base. Proves both the Curves per-channel indexing
/// (`base + c*256 + idx`, 3 distinct tables) and the Levels channel-uniform
/// indexing (`base + idx`, 1 table), the latter against the REAL
/// `levels_display_lut` exporter. For a single OPAQUE base layer (Normal,
/// opacity 1) + a Curves/Levels op (Normal, opacity 1), the GPU output byte for
/// channel `c` is `round(lut[c][base_byte] * 255)` — the decode→display
/// round-trip recovers the source byte to index the table.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_adjustment_luts_curves_levels_parity() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (32u32, 32u32);
    let region = Region::full(w, h);
    // Opaque varied base → alpha is trivial, the adjustment is pure RGB.
    let mut base = varied_canvas(w, h, 3);
    for i in 0..(w * h) as usize {
        base[i * 4 + 3] = 255;
    }
    let mut comp = LayerCompositor::new(&gpu);

    // ── Curves: 3 DISTINCT tables (identity R / invert G / half B) prove that
    //    `adj_luts[base + c*256 + idx]` selects the right per-channel table.
    let mut curves_lut = vec![0.0f32; 3 * 256];
    for i in 0..256 {
        let v = i as f32 / 255.0;
        curves_lut[i] = v; // R: identity
        curves_lut[256 + i] = 1.0 - v; // G: invert
        curves_lut[512 + i] = v * 0.5; // B: half
    }
    let ops = vec![
        LayerOp::Layer {
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Adjustment {
            kind: 7,
            params: [0.0, 0.0, 0.0],
            blend_mode: 0,
            opacity: 1.0,
        },
    ];
    let mut prov = MapProvider::default();
    prov.insert(1, 1, base.clone());
    comp.composite_with_luts(&gpu, &ops, &curves_lut, &prov, w, h, region)
        .expect("curves composite");
    let got = comp.read_output(&gpu).expect("readback");
    let mut want = vec![0u8; got.len()];
    for p in 0..(w * h) as usize {
        for c in 0..3 {
            let b = base[p * 4 + c] as usize;
            want[p * 4 + c] = (curves_lut[c * 256 + b] * 255.0 + 0.5) as u8;
        }
        want[p * 4 + 3] = 255;
    }
    let diff = max_byte_diff(&got, &want);
    assert!(
        diff <= 4,
        "Curves 3-table GPU LUT parity: max byte diff {diff}"
    );

    // ── Levels: 1 table (channel-uniform) from the REAL `levels_display_lut`.
    let lp = LevelsParams {
        black_point: 0.2,
        gamma: 1.5,
        white_point: 0.85,
        output_black: 0.05,
        output_white: 0.95,
    };
    let levels_lut = levels_display_lut(&lp).to_vec();
    let ops = vec![
        LayerOp::Layer {
            key: 1,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Adjustment {
            kind: 8,
            params: [0.0, 0.0, 0.0],
            blend_mode: 0,
            opacity: 1.0,
        },
    ];
    let mut prov = MapProvider::default();
    prov.insert(1, 2, base.clone());
    comp.composite_with_luts(&gpu, &ops, &levels_lut, &prov, w, h, region)
        .expect("levels composite");
    let got = comp.read_output(&gpu).expect("readback");
    let mut want = vec![0u8; got.len()];
    for p in 0..(w * h) as usize {
        for c in 0..3 {
            let b = base[p * 4 + c] as usize;
            want[p * 4 + c] = (levels_lut[b] * 255.0 + 0.5) as u8;
        }
        want[p * 4 + 3] = 255;
    }
    let diff = max_byte_diff(&got, &want);
    assert!(
        diff <= 4,
        "Levels 1-table GPU LUT parity: max byte diff {diff}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_matches_cpu_reference_grouped_stack() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (24u32, 24u32);
    let mut prov = MapProvider::default();
    for k in 0..6u64 {
        prov.insert(k, 1, varied_canvas(w, h, k as u32 + 1));
    }
    // bottom, [group: c2 multiply, c3 screen]@0.6 overlay, c4 soft-light, c5 hue
    let ops = vec![
        LayerOp::Layer {
            key: 0,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Layer {
            key: 1,
            blend_mode: 8,
            opacity: 0.5,
        }, // Add
        LayerOp::PushGroup,
        LayerOp::Layer {
            key: 2,
            blend_mode: 1,
            opacity: 1.0,
        }, // Multiply
        LayerOp::Layer {
            key: 3,
            blend_mode: 6,
            opacity: 0.7,
        }, // Screen
        LayerOp::PopGroup {
            blend_mode: 9,
            opacity: 0.6,
        }, // Overlay group
        LayerOp::Layer {
            key: 4,
            blend_mode: 10,
            opacity: 0.9,
        }, // SoftLight
        LayerOp::Layer {
            key: 5,
            blend_mode: 16,
            opacity: 0.8,
        }, // Hue
    ];
    let region = Region::full(w, h);
    let mut comp = LayerCompositor::new(&gpu);
    comp.composite(&gpu, &ops, &prov, w, h, region)
        .expect("composite");
    let got = comp.read_output(&gpu).expect("readback");
    let want = cpu_composite(&ops, &prov, w, h, region);
    let d = max_byte_diff(&got, &want);
    assert!(
        d <= 1,
        "grouped stack diverged from CPU reference by {d} bytes"
    );
}

/// MAX-depth nesting: a group chain 8 levels deep exercises the `cs_grouped`
/// per-pixel accumulator `stack[d]` for d=1..7 (the riskiest WGSL — a
/// hand-written, dynamically-indexed stack machine) against the CPU recursion.
/// The depth-1 grouped test above only reaches stack[0]; this reaches the cap.
/// (Audit 2026-06-01 coverage gap.)
#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_matches_cpu_reference_deep_nested_groups() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (24u32, 24u32);
    let depth = 8usize; // MAX_GROUP_STACK
    let mut prov = MapProvider::default();
    for k in 0..=depth as u64 {
        prov.insert(k, 1, varied_canvas(w, h, k as u32 + 1));
    }
    // bg, then `depth` nested [PushGroup, Layer(mode d)], closed by `depth`
    // PopGroups with varied blend modes — every stack level carries real blends.
    let mut ops = vec![LayerOp::Layer {
        key: 0,
        blend_mode: 0,
        opacity: 1.0,
    }];
    for d in 1..=depth {
        ops.push(LayerOp::PushGroup);
        ops.push(LayerOp::Layer {
            key: d as u64,
            blend_mode: (d % MAX_BLEND_MODES as usize) as u8,
            opacity: 0.8,
        });
    }
    for d in (1..=depth).rev() {
        ops.push(LayerOp::PopGroup {
            blend_mode: ((d * 3) % MAX_BLEND_MODES as usize) as u8,
            opacity: 0.7,
        });
    }
    let region = Region::full(w, h);
    let mut comp = LayerCompositor::new(&gpu);
    comp.composite(&gpu, &ops, &prov, w, h, region)
        .expect("composite");
    let got = comp.read_output(&gpu).expect("readback");
    let want = cpu_composite(&ops, &prov, w, h, region);
    let diff = max_byte_diff(&got, &want);
    assert!(
        diff <= 1,
        "depth-{depth} nested groups diverged from CPU reference by {diff} bytes"
    );
}

/// `layers_dirty_rect_correctness`: recompositing a sub-region is bit-identical
/// to the same rect cropped from a full composite (per-pixel independence).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_dirty_rect_matches_full() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (64u32, 48u32);
    let mut prov = MapProvider::default();
    prov.insert(0, 1, varied_canvas(w, h, 1));
    prov.insert(1, 1, varied_canvas(w, h, 2));
    let ops = vec![
        LayerOp::Layer {
            key: 0,
            blend_mode: 0,
            opacity: 1.0,
        },
        LayerOp::Layer {
            key: 1,
            blend_mode: 11,
            opacity: 0.65,
        }, // HardLight
    ];
    let mut comp = LayerCompositor::new(&gpu);

    comp.composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
        .expect("full");
    let full = comp.read_output(&gpu).expect("readback full");

    let region = Region {
        x: 13,
        y: 9,
        w: 20,
        h: 17,
    };
    comp.composite(&gpu, &ops, &prov, w, h, region)
        .expect("region");
    let part = comp.read_output(&gpu).expect("readback region");

    for ly in 0..region.h {
        for lx in 0..region.w {
            let gx = region.x + lx;
            let gy = region.y + ly;
            let fi = ((gy * w + gx) * 4) as usize;
            let pi = ((ly * region.w + lx) * 4) as usize;
            assert_eq!(
                &full[fi..fi + 4],
                &part[pi..pi + 4],
                "dirty-rect pixel ({gx},{gy}) != full recompose"
            );
        }
    }
}

/// Median wall-clock (ms) of `ops` composited over `region`, GPU idle between
/// runs. Warms once (not measured).
#[allow(clippy::too_many_arguments)]
fn measure_composite(
    gpu: &GpuContext,
    comp: &mut LayerCompositor,
    ops: &[LayerOp],
    prov: &MapProvider,
    w: u32,
    h: u32,
    region: Region,
    runs: u32,
) -> f64 {
    comp.composite(gpu, ops, prov, w, h, region).expect("warm");
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let mut times = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let t0 = std::time::Instant::now();
        comp.composite(gpu, ops, prov, w, h, region)
            .expect("composite");
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}

/// The PAYOFF (W4): an adjustment slider-drag recomposites the FULL canvas every
/// frame (the layers are cached — only the adjustment params buffer + the compute
/// dispatch re-run; zero CPU pixel work, zero re-upload). On the CPU reference
/// this was ~55 ms @1024² for HSB (OKLab cbrt-bound); on the GPU the same is
/// sub-millisecond — the whole point of routing the preview through here. Prints
/// the numbers; asserts a generous interactive bound (full-canvas adjustment
/// must beat one 60 Hz frame with headroom, hardware-independently).
#[test]
#[ignore = "needs a GPU device"]
fn gpu_adjustment_drag_full_canvas_perf() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut comp = LayerCompositor::new(&gpu);
    for &(w, h) in &[(1024u32, 1024u32), (2048u32, 2048u32)] {
        let mut prov = MapProvider::default();
        prov.insert(1, 1, varied_canvas(w, h, 3));
        let ops = vec![
            LayerOp::Layer {
                key: 1,
                blend_mode: 0,
                opacity: 1.0,
            },
            LayerOp::Adjustment {
                kind: 0, // HSB — the cbrt-heavy worst case
                params: [0.15, 0.4, 0.1],
                blend_mode: 0,
                opacity: 1.0,
            },
        ];
        let median = measure_composite(&gpu, &mut comp, &ops, &prov, w, h, Region::full(w, h), 32);
        eprintln!("[perf] base+HSB full {w}×{h} GPU composite: median {median:.3} ms");
        assert!(
            median < 8.0,
            "full-canvas adjustment {w}×{h} = {median:.2} ms exceeds the 8 ms interactive budget"
        );
    }
}

/// `layers_composite_50_4k_interactive_under_5ms` — the INTERACTIVE latency
/// gate. A stroke dirties a bounded region (a brush dab), so the real-time hot
/// path recomposites only that rect over the full stack, NOT the whole 4K
/// canvas. 50 layers over a 512×512 dirty region must land well under one
/// 60 Hz frame. (Full-canvas recompose — load/zoom/resize — is bandwidth-bound:
/// 50 × 33 MB = 1.66 GB of reads, ~5 ms only at ≥330 GB/s; see the scaling gate
/// below. Dirty-rect is what keeps editing responsive on every GPU.)
#[test]
#[ignore = "needs a GPU device + ~0.5 GB"]
fn gpu_composite_50_layers_dirty_rect_under_5ms() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (3840u32, 2160u32);
    let cap = LayerCompositor::new(&gpu).cache_cap(w, h).clamp(1, 16);
    let mut prov = MapProvider::default();
    for k in 0..cap as u64 {
        prov.insert(k, 1, varied_canvas(w, h, k as u32 + 1));
    }
    let ops: Vec<LayerOp> = (0..50)
        .map(|i| LayerOp::Layer {
            key: (i % cap) as u64,
            blend_mode: (i % MAX_BLEND_MODES as u32) as u8,
            opacity: 0.9,
        })
        .collect();
    let region = Region {
        x: 1000,
        y: 700,
        w: 512,
        h: 512,
    };
    let mut comp = LayerCompositor::new(&gpu);
    let median = measure_composite(&gpu, &mut comp, &ops, &prov, w, h, region, 32);
    eprintln!("[perf] 50-layer 512² dirty-rect composite: median {median:.2} ms");
    assert!(
        median < 5.0,
        "interactive dirty-rect composite {median:.2} ms exceeds 5 ms"
    );
}

/// `layers_composite_full_4k_scales_linearly` — the SHADER-EFFICIENCY gate for
/// the worst case (full 4K recomposite). Full-canvas time is bounded by memory
/// bandwidth (each layer = one 33 MB read), so it CANNOT beat the device's
/// bandwidth floor (~23 ms on this ~70 GB/s unified-memory laptop; ~4 ms on a
/// ≥330 GB/s discrete GPU). What we CAN gate, hardware-independently, is that
/// cost scales ~linearly with layer count — i.e. occupancy does not collapse
/// into a superlinear cliff. 50 layers must cost < 6× the 10-layer time
/// (linear ≈ 5×). Prints the absolute numbers for the record.
#[test]
#[ignore = "needs a GPU device + ~0.5 GB"]
fn gpu_composite_full_4k_scales_linearly() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (3840u32, 2160u32);
    let cap = LayerCompositor::new(&gpu).cache_cap(w, h).clamp(1, 16);
    let mut prov = MapProvider::default();
    for k in 0..cap as u64 {
        prov.insert(k, 1, varied_canvas(w, h, k as u32 + 1));
    }
    let make = |n: u32| -> Vec<LayerOp> {
        (0..n)
            .map(|i| LayerOp::Layer {
                key: (i % cap) as u64,
                blend_mode: (i % MAX_BLEND_MODES as u32) as u8,
                opacity: 0.9,
            })
            .collect()
    };
    let region = Region::full(w, h);
    let mut comp = LayerCompositor::new(&gpu);
    let t10 = measure_composite(&gpu, &mut comp, &make(10), &prov, w, h, region, 8);
    let t50 = measure_composite(&gpu, &mut comp, &make(50), &prov, w, h, region, 8);
    eprintln!(
        "[perf] full 4K composite: 10-layer {t10:.2} ms, 50-layer {t50:.2} ms (ratio {:.2}×)",
        t50 / t10
    );
    assert!(
        t50 < t10 * 6.0,
        "full 4K composite scales superlinearly: 50-layer {t50:.2} ms vs 10-layer {t10:.2} ms (>6×) — occupancy regression"
    );
}

/// `layers_max_count_per_budget`: referencing more distinct layers than the
/// per-budget cap is refused (no OOM). At 4096² a slice is 64 MB, so the
/// 512 MB budget caps at 8; a 9-distinct op-list errors before any upload.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_too_many_layers_errors_at_budget_cap() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let (w, h) = (4096u32, 4096u32);
    let mut comp = LayerCompositor::new(&gpu);
    let cap = comp.cache_cap(w, h);
    assert_eq!(
        cap, 8,
        "expected 512 MiB / 64 MiB = 8 at 4096^2 (device max permitting)"
    );

    // One shared buffer reused for every key — the error fires in ensure_array
    // (distinct > cap) before any slice is uploaded, so we only hold one.
    let mut prov = MapProvider::default();
    let shared = vec![128u8; (w as usize) * (h as usize) * 4];
    for k in 0..(cap as u64 + 1) {
        prov.insert(k, 1, shared.clone());
    }
    let ops: Vec<LayerOp> = (0..(cap as u64 + 1))
        .map(|k| LayerOp::Layer {
            key: k,
            blend_mode: 0,
            opacity: 1.0,
        })
        .collect();
    let err = comp
        .composite(&gpu, &ops, &prov, w, h, Region::full(w, h))
        .unwrap_err();
    assert_eq!(
        err,
        LayerCompositeError::TooManyLayers {
            requested: cap + 1,
            cap
        }
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// SPATIAL PASS-GRAPH (Painter W4) — Gaussian-blur parity
// ═══════════════════════════════════════════════════════════════════════════
//
// Proves the segmented infra (materialise-below → separable H/V ping-pong →
// combine → continue-above → encode) reproduces a full CPU recompose of the same
// op-list, AND that the dirty-rect ⊕ halo path equals the full recompose cropped.
// The Gaussian weights are the provisional `gaussian_weights` (the σ↔radius math
// is the painter impl's canonical `apply_gaussian` once reconciled) — both GPU
// and this reference read the SAME weights, so the test proves the *mechanism*.

/// Decode one straight-sRGB8 texel at byte offset `i` to straight linear RGBA.
fn cpu_decode_px(b: &[u8], i: usize) -> [f32; 4] {
    [
        srgb_to_linear_byte(b[i]),
        srgb_to_linear_byte(b[i + 1]),
        srgb_to_linear_byte(b[i + 2]),
        b[i + 3] as f32 / 255.0,
    ]
}

/// Composite a run of `Layer`/`Adjustment` ops over the full canvas into a linear
/// RGBA buffer, starting each pixel from `base` (or zero). Mirror of `cs_segment`
/// for the test's group-free op runs.
fn cpu_seg_linear(
    ops: &[LayerOp],
    prov: &MapProvider,
    w: u32,
    h: u32,
    base: Option<&[[f32; 4]]>,
) -> Vec<[f32; 4]> {
    let n = (w * h) as usize;
    let mut out = vec![[0.0f32; 4]; n];
    for (p, slot) in out.iter_mut().enumerate() {
        let mut acc = base.map_or([0.0f32; 4], |b| b[p]);
        let i = p * 4;
        for op in ops {
            match op {
                LayerOp::Layer {
                    key,
                    blend_mode,
                    opacity,
                } => {
                    let bb = prov.bytes(*key);
                    let mut s = cpu_decode_px(bb, i);
                    s[3] *= *opacity;
                    acc = apply_blend(BlendMode::from_u8(*blend_mode), acc, s);
                }
                LayerOp::Adjustment {
                    kind,
                    params,
                    blend_mode,
                    opacity,
                } => {
                    acc = cpu_adjust_op(*kind, *params, *blend_mode, *opacity, acc);
                }
                _ => {} // no groups / spatial ops inside a segment run (test invariant)
            }
        }
        *slot = acc;
    }
    out
}

/// Separable Gaussian over a full-canvas linear buffer (H then V, clamp-to-edge),
/// using the symmetric `weights[0..=half]`. Mirror of `cs_blur_h`/`cs_blur_v`.
fn cpu_blur_linear(src: &[[f32; 4]], w: u32, h: u32, weights: &[f32], half: u32) -> Vec<[f32; 4]> {
    let wi = w as i32;
    let hi = h as i32;
    let pass = |inp: &[[f32; 4]], dx: i32, dy: i32| -> Vec<[f32; 4]> {
        let mut out = vec![[0.0f32; 4]; inp.len()];
        for y in 0..hi {
            for x in 0..wi {
                let mut acc = [0.0f32; 4];
                let c = inp[(y * wi + x) as usize];
                for k in 0..4 {
                    acc[k] += c[k] * weights[0];
                }
                for i in 1..=half as i32 {
                    let xa = (x + dx * i).clamp(0, wi - 1);
                    let ya = (y + dy * i).clamp(0, hi - 1);
                    let xb = (x - dx * i).clamp(0, wi - 1);
                    let yb = (y - dy * i).clamp(0, hi - 1);
                    let a = inp[(ya * wi + xa) as usize];
                    let b = inp[(yb * wi + xb) as usize];
                    for k in 0..4 {
                        acc[k] += (a[k] + b[k]) * weights[i as usize];
                    }
                }
                out[(y * wi + x) as usize] = acc;
            }
        }
        out
    };
    let tmp = pass(src, 1, 0);
    pass(&tmp, 0, 1)
}

/// Derive the kernel result from base + blurred, blend it over the base per
/// `blend`/`opacity`, preserving coverage. Mirror of `cs_combine`. `sharpen`
/// = `None` → Gaussian (passthrough blurred); `Some(amount)` → unsharp mask
/// (`base + amount·(base − blurred)`, clamped).
fn cpu_combine_linear(
    base: &[[f32; 4]],
    blurred: &[[f32; 4]],
    blend: u8,
    opacity: f32,
    sharpen: Option<f32>,
) -> Vec<[f32; 4]> {
    base.iter()
        .zip(blurred)
        .map(|(acc, bl)| {
            let adj = match sharpen {
                None => [bl[0], bl[1], bl[2]],
                Some(a) => [
                    (acc[0] + a * (acc[0] - bl[0])).clamp(0.0, 1.0),
                    (acc[1] + a * (acc[1] - bl[1])).clamp(0.0, 1.0),
                    (acc[2] + a * (acc[2] - bl[2])).clamp(0.0, 1.0),
                ],
            };
            let src_px = [adj[0], adj[1], adj[2], acc[3]];
            let blended = apply_blend(BlendMode::from_u8(blend), *acc, src_px);
            let t = opacity.clamp(0.0, 1.0);
            [
                acc[0] + (blended[0] - acc[0]) * t,
                acc[1] + (blended[1] - acc[1]) * t,
                acc[2] + (blended[2] - acc[2]) * t,
                acc[3],
            ]
        })
        .collect()
}

/// Encode a full-canvas linear buffer → straight sRGB8. Mirror of `encode_final`.
fn cpu_encode_full(buf: &[[f32; 4]]) -> Vec<u8> {
    let mut out = vec![0u8; buf.len() * 4];
    for (p, px) in buf.iter().enumerate() {
        out[p * 4] = linear_to_srgb_byte(px[0]);
        out[p * 4 + 1] = linear_to_srgb_byte(px[1]);
        out[p * 4 + 2] = linear_to_srgb_byte(px[2]);
        out[p * 4 + 3] = (px[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    }
    out
}

/// Crop a full-canvas sRGB8 buffer to `region`.
fn cpu_crop(canvas: &[u8], w: u32, region: Region) -> Vec<u8> {
    let mut out = vec![0u8; (region.w * region.h * 4) as usize];
    for ly in 0..region.h {
        for lx in 0..region.w {
            let src = (((region.y + ly) * w + (region.x + lx)) * 4) as usize;
            let dst = ((ly * region.w + lx) * 4) as usize;
            out[dst..dst + 4].copy_from_slice(&canvas[src..src + 4]);
        }
    }
    out
}

/// GPU spatial pass-graph vs a full CPU recompose, for a Gaussian-blur
/// adjustment between a (below) opaque base and an (above) layer.
///
/// (1) Full-region: proves materialise → blur → combine → continue → encode.
/// (2) Sub-region: proves the dirty-rect ⊕ halo recompose equals the full
///     recompose cropped to the same rect (no seam at the work-region edge).
///
/// Opaque base ⇒ coverage is 1 everywhere, so the blur's alpha/premultiply
/// handling is unambiguous (that choice is a kernel-semantics refinement owned by
/// the impl's `apply_gaussian`). Tolerance ±4 bytes: `Rgba32Float` intermediates
/// match the CPU f32, so divergence is only `pow` (sRGB encode) + add-reassoc ULP.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_gaussian_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let mut prov = MapProvider::default();
    // Opaque base (force alpha 255) + a partially-transparent top layer.
    let mut base = varied_canvas(w, h, 3);
    for px in base.chunks_mut(4) {
        px[3] = 255;
    }
    prov.insert(0, 1, base);
    prov.insert(1, 1, varied_canvas(w, h, 7));

    let radius = 4.0f32;
    let (weights, half) = gaussian_weights(radius);
    let below = [LayerOp::Layer {
        key: 0,
        blend_mode: 0,
        opacity: 1.0,
    }];
    let above = [LayerOp::Layer {
        key: 1,
        blend_mode: 0,
        opacity: 0.6,
    }];
    let ops = vec![
        below[0],
        LayerOp::SpatialAdjustment {
            kernel: SPATIAL_GAUSSIAN,
            params: [radius, 0.0, 0.0, 0.0],
            blend_mode: 0,
            opacity: 1.0,
        },
        above[0],
    ];

    // CPU full-canvas reference.
    let mat = cpu_seg_linear(&below, &prov, w, h, None);
    let blurred = cpu_blur_linear(&mat, w, h, &weights, half);
    let combined = cpu_combine_linear(&mat, &blurred, 0, 1.0, None);
    let above_lin = cpu_seg_linear(&above, &prov, w, h, Some(&combined));
    let want_full = cpu_encode_full(&above_lin);

    let mut comp = LayerCompositor::new(&gpu);

    // (1) Full region.
    let full = Region::full(w, h);
    comp.composite(&gpu, &ops, &prov, w, h, full)
        .expect("composite full");
    let got_full = comp.read_output(&gpu).expect("readback full");
    let d_full = max_byte_diff(&got_full, &want_full);
    assert!(
        d_full <= 4,
        "gaussian full-region GPU vs CPU max byte diff {d_full}"
    );

    // (2) Sub-region — the work_region is dilated by the halo and cropped back.
    let sub = Region {
        x: 20,
        y: 18,
        w: 24,
        h: 22,
    };
    comp.composite(&gpu, &ops, &prov, w, h, sub)
        .expect("composite sub");
    let got_sub = comp.read_output(&gpu).expect("readback sub");
    let want_sub = cpu_crop(&want_full, w, sub);
    let d_sub = max_byte_diff(&got_sub, &want_sub);
    assert!(
        d_sub <= 4,
        "gaussian sub-region (dirty-rect ⊕ halo) GPU vs CPU max byte diff {d_sub}"
    );
}

/// GPU Sharpen (unsharp mask) vs a full CPU recompose — proves the pass-graph
/// generalises to a SECOND kernel on the SAME blur machinery (Sharpen = Gaussian
/// blur + a combine variant: `base + amount·(base − blur(base))`). Same opaque-
/// base / full + sub-region structure + ±4B tolerance as the Gaussian gate.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_sharpen_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let mut prov = MapProvider::default();
    let mut base = varied_canvas(w, h, 4);
    for px in base.chunks_mut(4) {
        px[3] = 255;
    }
    prov.insert(0, 1, base);
    prov.insert(1, 1, varied_canvas(w, h, 8));

    let amount = 0.8f32;
    let radius = 3.0f32;
    let (weights, half) = gaussian_weights(radius);
    let below = [LayerOp::Layer {
        key: 0,
        blend_mode: 0,
        opacity: 1.0,
    }];
    let above = [LayerOp::Layer {
        key: 1,
        blend_mode: 0,
        opacity: 0.5,
    }];
    let ops = vec![
        below[0],
        LayerOp::SpatialAdjustment {
            kernel: SPATIAL_SHARPEN,
            params: [amount, radius, 0.0, 0.0], // amount, blur radius
            blend_mode: 0,
            opacity: 1.0,
        },
        above[0],
    ];

    // CPU full-canvas reference: materialise → blur(base) → unsharp combine → above.
    let mat = cpu_seg_linear(&below, &prov, w, h, None);
    let blurred = cpu_blur_linear(&mat, w, h, &weights, half);
    let combined = cpu_combine_linear(&mat, &blurred, 0, 1.0, Some(amount));
    let above_lin = cpu_seg_linear(&above, &prov, w, h, Some(&combined));
    let want_full = cpu_encode_full(&above_lin);

    let mut comp = LayerCompositor::new(&gpu);

    let full = Region::full(w, h);
    comp.composite(&gpu, &ops, &prov, w, h, full)
        .expect("composite full");
    let got_full = comp.read_output(&gpu).expect("readback full");
    let d_full = max_byte_diff(&got_full, &want_full);
    assert!(
        d_full <= 4,
        "sharpen full-region GPU vs CPU max byte diff {d_full}"
    );

    let sub = Region {
        x: 22,
        y: 16,
        w: 20,
        h: 24,
    };
    comp.composite(&gpu, &ops, &prov, w, h, sub)
        .expect("composite sub");
    let got_sub = comp.read_output(&gpu).expect("readback sub");
    let want_sub = cpu_crop(&want_full, w, sub);
    let d_sub = max_byte_diff(&got_sub, &want_sub);
    assert!(
        d_sub <= 4,
        "sharpen sub-region (dirty-rect ⊕ halo) GPU vs CPU max byte diff {d_sub}"
    );
}

/// Directional (motion) blur over a full-canvas linear buffer — a single 1-D
/// pass averaging `2·half+1` taps along `dir`, nearest sampling at floor(x+0.5),
/// clamp-to-edge. Mirror of `cs_blur_dir`.
fn cpu_motion_blur_linear(
    src: &[[f32; 4]],
    w: u32,
    h: u32,
    weights: &[f32],
    half: u32,
    dir: [f32; 2],
) -> Vec<[f32; 4]> {
    let wi = w as i32;
    let hi = h as i32;
    let mut out = vec![[0.0f32; 4]; src.len()];
    for y in 0..hi {
        for x in 0..wi {
            let px = x as f32;
            let py = y as f32;
            let mut acc = [0.0f32; 4];
            let c = src[(y * wi + x) as usize];
            for k in 0..4 {
                acc[k] += c[k] * weights[0];
            }
            for i in 1..=half as i32 {
                let ox = dir[0] * i as f32;
                let oy = dir[1] * i as f32;
                let xa = ((px + ox + 0.5).floor() as i32).clamp(0, wi - 1);
                let ya = ((py + oy + 0.5).floor() as i32).clamp(0, hi - 1);
                let xb = ((px - ox + 0.5).floor() as i32).clamp(0, wi - 1);
                let yb = ((py - oy + 0.5).floor() as i32).clamp(0, hi - 1);
                let a = src[(ya * wi + xa) as usize];
                let b = src[(yb * wi + xb) as usize];
                for k in 0..4 {
                    acc[k] += (a[k] + b[k]) * weights[i as usize];
                }
            }
            out[(y * wi + x) as usize] = acc;
        }
    }
    out
}

/// GPU MotionBlur vs a full CPU recompose — proves the pass-graph swaps the BLUR
/// STAGE (directional 1-pass) while reusing materialise/combine/encode, the
/// complement to Sharpen (which swapped the combine). Direction is computed
/// CPU-side (cos/sin of the angle) for both GPU + reference, so there is no GPU
/// transcendental in the hot path. Same opaque-base / full + sub-region / ±4B.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_motion_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let mut prov = MapProvider::default();
    let mut base = varied_canvas(w, h, 6);
    for px in base.chunks_mut(4) {
        px[3] = 255;
    }
    prov.insert(0, 1, base);
    prov.insert(1, 1, varied_canvas(w, h, 2));

    let distance = 9.0f32;
    let angle = 0.6f32; // radians
    let (weights, half) = motion_weights(distance);
    let dir = [angle.cos(), angle.sin()];
    let below = [LayerOp::Layer {
        key: 0,
        blend_mode: 0,
        opacity: 1.0,
    }];
    let above = [LayerOp::Layer {
        key: 1,
        blend_mode: 0,
        opacity: 0.7,
    }];
    let ops = vec![
        below[0],
        LayerOp::SpatialAdjustment {
            kernel: SPATIAL_MOTION,
            params: [distance, angle, 0.0, 0.0],
            blend_mode: 0,
            opacity: 1.0,
        },
        above[0],
    ];

    // CPU full-canvas reference.
    let mat = cpu_seg_linear(&below, &prov, w, h, None);
    let blurred = cpu_motion_blur_linear(&mat, w, h, &weights, half, dir);
    let combined = cpu_combine_linear(&mat, &blurred, 0, 1.0, None);
    let above_lin = cpu_seg_linear(&above, &prov, w, h, Some(&combined));
    let want_full = cpu_encode_full(&above_lin);

    let mut comp = LayerCompositor::new(&gpu);

    let full = Region::full(w, h);
    comp.composite(&gpu, &ops, &prov, w, h, full)
        .expect("composite full");
    let got_full = comp.read_output(&gpu).expect("readback full");
    let d_full = max_byte_diff(&got_full, &want_full);
    assert!(
        d_full <= 4,
        "motion full-region GPU vs CPU max byte diff {d_full}"
    );

    let sub = Region {
        x: 18,
        y: 20,
        w: 26,
        h: 18,
    };
    comp.composite(&gpu, &ops, &prov, w, h, sub)
        .expect("composite sub");
    let got_sub = comp.read_output(&gpu).expect("readback sub");
    let want_sub = cpu_crop(&want_full, w, sub);
    let d_sub = max_byte_diff(&got_sub, &want_sub);
    assert!(
        d_sub <= 4,
        "motion sub-region (dirty-rect ⊕ halo) GPU vs CPU max byte diff {d_sub}"
    );
}

/// Chromatic-aberration gather over a full-canvas linear buffer — per-channel
/// radial resample, nearest at floor(x+0.5), clamp-to-edge. Mirror of `cs_chroma`
/// (full canvas ⇒ work origin = 0, centre = canvas centre). The per-channel
/// scale = shift / half_diag is precomputed exactly as the orchestrator does.
fn cpu_chroma_gather(src: &[[f32; 4]], w: u32, h: u32, shifts: [f32; 3]) -> Vec<[f32; 4]> {
    let cw = w as f32;
    let ch = h as f32;
    let half_diag = 0.5 * (cw * cw + ch * ch).sqrt();
    let inv = if half_diag > 0.0 {
        1.0 / half_diag
    } else {
        0.0
    };
    let cx = cw * 0.5;
    let cy = ch * 0.5;
    let scale = [shifts[0] * inv, shifts[1] * inv, shifts[2] * inv];
    let wi = w as i32;
    let hi = h as i32;
    let mut out = vec![[0.0f32; 4]; src.len()];
    for y in 0..hi {
        for x in 0..wi {
            let lfx = x as f32;
            let lfy = y as f32;
            let dirx = lfx - cx;
            let diry = lfy - cy;
            let sample = |sc: f32| -> usize {
                let sx = ((lfx - dirx * sc + 0.5).floor() as i32).clamp(0, wi - 1);
                let sy = ((lfy - diry * sc + 0.5).floor() as i32).clamp(0, hi - 1);
                (sy * wi + sx) as usize
            };
            let r = src[sample(scale[0])][0];
            let g = src[sample(scale[1])][1];
            let b = src[sample(scale[2])][2];
            let a = src[(y * wi + x) as usize][3];
            out[(y * wi + x) as usize] = [r, g, b, a];
        }
    }
    out
}

/// GPU ChromaticAberration vs a full CPU recompose — proves the pass-graph also
/// handles a GATHER stage (per-channel divergent radial resample), distinct from
/// the neighbourhood-average blurs. Centre + per-channel scales are precomputed
/// CPU-side so the gather does no per-pixel sqrt (parity-robust nearest sampling).
/// Same opaque-base / full + sub-region / ±4B structure as the other spatial gates.
#[test]
#[ignore = "needs a GPU device"]
fn gpu_chroma_matches_cpu_reference() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let mut prov = MapProvider::default();
    let mut base = varied_canvas(w, h, 5);
    for px in base.chunks_mut(4) {
        px[3] = 255;
    }
    prov.insert(0, 1, base);
    prov.insert(1, 1, varied_canvas(w, h, 3));

    // Classic CA: red fringes outward, blue inward, green fixed.
    let shifts = [3.0f32, 0.0, -3.0];
    let below = [LayerOp::Layer {
        key: 0,
        blend_mode: 0,
        opacity: 1.0,
    }];
    let above = [LayerOp::Layer {
        key: 1,
        blend_mode: 0,
        opacity: 0.5,
    }];
    let ops = vec![
        below[0],
        LayerOp::SpatialAdjustment {
            kernel: SPATIAL_CHROMA,
            params: [shifts[0], shifts[1], shifts[2], 0.0],
            blend_mode: 0,
            opacity: 1.0,
        },
        above[0],
    ];

    // CPU full-canvas reference: materialise → chroma gather → passthrough → above.
    let mat = cpu_seg_linear(&below, &prov, w, h, None);
    let gathered = cpu_chroma_gather(&mat, w, h, shifts);
    let combined = cpu_combine_linear(&mat, &gathered, 0, 1.0, None);
    let above_lin = cpu_seg_linear(&above, &prov, w, h, Some(&combined));
    let want_full = cpu_encode_full(&above_lin);

    let mut comp = LayerCompositor::new(&gpu);

    let full = Region::full(w, h);
    comp.composite(&gpu, &ops, &prov, w, h, full)
        .expect("composite full");
    let got_full = comp.read_output(&gpu).expect("readback full");
    let d_full = max_byte_diff(&got_full, &want_full);
    assert!(
        d_full <= 4,
        "chroma full-region GPU vs CPU max byte diff {d_full}"
    );

    let sub = Region {
        x: 20,
        y: 20,
        w: 24,
        h: 24,
    };
    comp.composite(&gpu, &ops, &prov, w, h, sub)
        .expect("composite sub");
    let got_sub = comp.read_output(&gpu).expect("readback sub");
    let want_sub = cpu_crop(&want_full, w, sub);
    let d_sub = max_byte_diff(&got_sub, &want_sub);
    assert!(
        d_sub <= 4,
        "chroma sub-region (dirty-rect ⊕ halo) GPU vs CPU max byte diff {d_sub}"
    );
}
