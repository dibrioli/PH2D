//! W15.3 — GPU composite structural gates (ADR-0085: GPU is the single source of truth).
//!
//! The CPU↔GPU composite-parity tests are gone (ADR-0085 §2.2 retired the CPU twin). What
//! remains here are GPU-only structural gates: the per-frame banded/pipelined/fast-path
//! composites must agree with the one-shot full-canvas composite (internal consistency),
//! the K–M glaze must read green over a blue backdrop with no black fringe, lift must reveal
//! paper not transparency, and the wet-sheen-off path must be byte-identical. Each asserts a
//! property of the GPU pipeline against itself — no CPU reference.
//!
//! Not bit-equality across backends: GPU lowers `exp`/`pow`/`sqrt` differently, so where a
//! tolerance is used it is a tight mean/worst |Δ| over the RGBA8 output (a correct shader
//! agrees to a fraction of an LSB; a wrong one diverges by many LSB everywhere).
//!
//! `#[ignore]` — needs a real device (like the solver gate):
//!   cargo test -p ph2d-painter-fluid --features fluid --test composite_parity -- --ignored --nocapture
#![cfg(feature = "fluid")]

use ph2d_gpu::GpuContext;
use ph2d_painter_brush::diffusion::DiffusionGrid;
use ph2d_painter_fluid::{FluidCompositor, FluidSolver};

const SCALE: u32 = 2;
const COVERAGE_K: f32 = 1.06;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// A bloomed yellow wash on a low-res field — the overlapping splats give a smooth radial
/// gradient that exercises the bicubic upsample + both glaze paths (catches sampling bugs).
/// ADR-0085: no CPU `step` — the splatted field is already smooth and non-uniform, and these
/// gates only need a non-trivial field to composite (GPU↔GPU consistency, not CPU parity).
fn seeded_field(gw: u32, gh: u32) -> DiffusionGrid {
    let mut g = DiffusionGrid::new(gw, gh, SCALE as f32);
    // Wet pool + a yellow dab straddling the canvas mid-line (where the backdrop
    // flips opaque→transparent), so one composite exercises BOTH glaze paths.
    g.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        gw as f32 * 0.4,
        0.7,
        [0.0, 0.0, 0.0],
        0.0 + 0.0 + 0.0,
        0.0,
    );
    g.splat(
        gw as f32 * 0.5,
        gh as f32 * 0.5,
        7.0,
        0.8,
        [0.55, 0.42, 0.02],
        0.55 + 0.42 + 0.02,
        0.0,
    );
    g
}

/// Backdrop: left half opaque saturated blue (K–M glaze → green), right half fully
/// transparent (straight-alpha "over" → pigment colour, no black fringe).
fn split_backdrop(cw: u32, ch: u32) -> Vec<u8> {
    let mut b = vec![0u8; (cw * ch * 4) as usize];
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y * cw + x) * 4) as usize;
            if x < cw / 2 {
                b[i..i + 4].copy_from_slice(&[20, 40, 200, 255]);
            } // else stays [0,0,0,0]
        }
    }
    b
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_rows_matches_full_band() {
    // The shell per-frame path reads back only the wet row band; it must equal the
    // corresponding rows of the full-canvas composite (guards the offset/slicing).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    // `composite_to_rgba` uploads `pig` itself; for the rows path stash the same
    // pigment in a solver buffer (no step) and bind it — both composite the SAME field.
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);
    let compositor = FluidCompositor::new(&gpu.device);
    let full = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );
    let (band, (px_lo, py_lo, px_hi, py_hi)) = compositor.composite_buffer_rows(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        solver.pigment_buffer(),
        &backdrop,
        region,
    );
    // The band is full-width, so it equals the full composite's row band; the rect's
    // columns are what the shell actually blits (the sub-rect that avoids erasure).
    let lo = (py_lo * cw * 4) as usize;
    let hi = (py_hi * cw * 4) as usize;
    assert_eq!(band.len(), hi - lo, "row band length");
    assert_eq!(
        band,
        full[lo..hi],
        "row band must equal the full composite's band"
    );
    assert!(
        px_hi > px_lo && px_hi <= cw,
        "rect cols in range: {px_lo}..{px_hi}"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_frame_fast_path_matches_one_shot() {
    // The persistent-buffer hot path (begin_stroke + composite_frame) must produce
    // the SAME band + rect as the per-call one-shot (composite_buffer_rows) — proves
    // the perf rewrite didn't change pixels.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);
    let mut compositor = FluidCompositor::new(&gpu.device);

    // Fast path (ss=2 to match the one-shot's WET_COMPOSITE_SS).
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        2,
        solver.pigment_buffer(),
        &backdrop,
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
    );
    let (band_fast, rect_fast) = compositor.composite_frame(&gpu.device, &gpu.queue, region);

    // One-shot (the tested path).
    let (band_one, rect_one) = compositor.composite_buffer_rows(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        solver.pigment_buffer(),
        &backdrop,
        region,
    );
    assert_eq!(rect_fast, rect_one, "fast-path rect must match one-shot");
    assert_eq!(
        band_fast, band_one,
        "fast-path band must match one-shot (byte-exact)"
    );

    // ss=1 (the full-res hot path) must also composite correctly: a wet opaque-blue
    // pixel still goes K–M green-dominant (single-sample, no supersampling).
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        1,
        solver.pigment_buffer(),
        &backdrop,
        &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
        None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
        None,      // wet-sheen water: dormant (no live water buffer)
    );
    let (band_ss1, (px_lo, py_lo, px_hi, _)) =
        compositor.composite_frame(&gpu.device, &gpu.queue, region);
    assert!(!band_ss1.is_empty(), "ss=1 composite must produce output");
    // Probe a wet pixel in the opaque-blue (left) half, inside the band.
    let cyr = ch / 2;
    let cxr = (cw / 2).saturating_sub(3).max(px_lo + 1).min(px_hi - 1);
    let i = ((cyr - py_lo) * cw + cxr) as usize * 4;
    let (r, g, b) = (
        band_ss1[i] as i32,
        band_ss1[i + 1] as i32,
        band_ss1[i + 2] as i32,
    );
    assert!(
        g >= r && g >= b,
        "ss=1 K–M still green-dominant over blue: [{r},{g},{b}]"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_frame_pipelined_matches_sync() {
    // ADR-0078 S2: the pipelined (async, 1-frame-late) composite must produce the SAME
    // pixels as the synchronous composite_frame — only the read timing differs (no
    // per-frame device.poll(wait) stall). Frame 1 returns empty (its band maps async);
    // frame 2 returns frame 1's band, which must equal the sync band for the same field.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);

    let begin = |c: &mut FluidCompositor| {
        c.begin_stroke(
            &gpu.device,
            &gpu.queue,
            gw,
            gh,
            cw,
            ch,
            SCALE,
            COVERAGE_K,
            1,
            solver.pigment_buffer(),
            &backdrop,
            &backdrop, // ADR-0084 paper-reveal: paper == backdrop ⇒ exact no-op
            None,      // ADR-0084: dormant backdrop-lift (no lift buffer)
            None,      // wet-sheen water: dormant (no live water buffer)
        );
    };

    // Sync reference.
    let mut sync = FluidCompositor::new(&gpu.device);
    begin(&mut sync);
    let (band_sync, rect_sync) = sync.composite_frame(&gpu.device, &gpu.queue, region);

    // Pipelined: same field, two frames; the 2nd call returns the 1st's band.
    let mut pipe = FluidCompositor::new(&gpu.device);
    begin(&mut pipe);
    let (band0, _) = pipe.composite_frame_pipelined(&gpu.device, &gpu.queue, region);
    assert!(
        band0.is_empty(),
        "first pipelined frame returns no band yet"
    );
    // Simulate the inter-frame gap: live, the GPU finishes the tiny copy within the
    // ~4 ms frame + the next frame's non-blocking poll collects it. Back-to-back in a
    // test there's no gap, so force completion here.
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let (band1, rect1) = pipe.composite_frame_pipelined(&gpu.device, &gpu.queue, region);

    assert_eq!(rect1, rect_sync, "pipelined rect (1-late) must match sync");
    assert_eq!(
        band1, band_sync,
        "pipelined band (1-late) must be byte-identical to the sync composite"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_km_signature_and_no_fringe() {
    // The two discriminant cases run ON THE GPU (proving the K–M + straight-alpha
    // paths are alive, not just a backdrop copy): yellow over opaque blue → green;
    // partial coverage over a transparent backdrop → warm pigment, no black fringe.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let compositor = FluidCompositor::new(&gpu.device);
    let out = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );

    // K–M signature: the wettest opaque-blue pixel (canvas centre, left of mid) is
    // green-dominant — the yellow-over-blue glaze (a linear "over" never is).
    let cyx = (ch / 2 * cw + (cw / 2 - 3)) * 4;
    let i = cyx as usize;
    let (r, g, b) = (out[i] as i32, out[i + 1] as i32, out[i + 2] as i32);
    eprintln!("GPU yellow-over-blue = [{r},{g},{b}]");
    assert!(
        g > r && g > b,
        "GPU K–M glaze must be green-dominant over blue: [{r},{g},{b}]"
    );

    // No black fringe: every painted pixel in the transparent (right) half keeps a
    // warm hue (red ≥ blue) and is not a near-black partial-coverage ring.
    for y in 0..ch {
        for x in cw / 2..cw {
            let p = ((y * cw + x) * 4) as usize;
            if out[p + 3] > 8 {
                assert!(
                    out[p] >= out[p + 2],
                    "GPU coral keeps red≥blue (no fringe) @({x},{y}): {:?}",
                    &out[p..p + 4]
                );
                assert!(
                    out[p] as u32 + out[p + 1] as u32 + out[p + 2] as u32 > 24,
                    "GPU partial-coverage edge is not a black fringe @({x},{y}): {:?}",
                    &out[p..p + 4]
                );
            }
        }
    }
}

#[test]
#[ignore = "needs a GPU device"]
fn gpu_composite_multi_pigment_subtractive_mix_is_green() {
    // ADR-0080: with the 28-channel wet field, overlapping pigments mix SUBTRACTIVELY in
    // the composite — a blue dab + an overlapping yellow dab composite to GREEN at the
    // overlap (an additive RGB average would be grey). On a transparent backdrop (straight-
    // alpha "over", so the canvas colour IS the pigment colour — no backdrop tint to confound
    // the check), the GPU composite reading the multi-channel field must read green-dominant
    // at the overlap. This is the composite-level partner of INV-4 (which checks the field).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let region = (0u32, 0u32, gw - 1, gh - 1);
    // Fully transparent backdrop → the composited colour is the pigment colour itself.
    let backdrop = vec![0u8; (cw * ch * 4) as usize];

    // Overlapping blue + yellow dabs at the field centre — the overlap cells carry both
    // pigments, which reduce subtractively when the K–M field is composited.
    let mut grid = DiffusionGrid::new(gw, gh, SCALE as f32);
    let blue = [0.05f32, 0.10, 0.85];
    let yellow = [0.85f32, 0.80, 0.05];
    let (ovx, ovy) = (gw as f32 * 0.5, gh as f32 * 0.5);
    grid.splat(
        ovx - 2.0,
        ovy,
        7.0,
        0.8,
        blue,
        blue[0] + blue[1] + blue[2],
        0.0,
    );
    grid.splat(
        ovx + 2.0,
        ovy,
        7.0,
        0.8,
        yellow,
        yellow[0] + yellow[1] + yellow[2],
        0.0,
    );
    let pig = grid.pigment();

    // GPU composite.
    let compositor = FluidCompositor::new(&gpu.device);
    let gpu_canvas = compositor.composite_to_rgba(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        pig,
        &backdrop,
        region,
    );

    // The overlap canvas pixel must read green-dominant on the GPU output (subtractive mix).
    let oi = (((ovy * SCALE as f32) as u32) * cw + ((ovx * SCALE as f32) as u32)) as usize * 4;
    let (r, g, b) = (
        gpu_canvas[oi] as i32,
        gpu_canvas[oi + 1] as i32,
        gpu_canvas[oi + 2] as i32,
    );
    eprintln!("GPU multi-pigment composite overlap = [{r},{g},{b}] (green-dominant?)");
    assert!(
        g > r && g > b,
        "GPU blue⊗yellow composite overlap must be green-dominant: [{r},{g},{b}]"
    );
}

#[test]
#[ignore = "needs a GPU device"]
fn composite_lift_reveals_paper_not_transparency() {
    // ADR-0084 paper-reveal: lifting returns the pixel toward the session's ORIGINAL paper
    // content (Curtis desorption / Rebelle) — NEVER toward transparency. The old model dropped
    // the backdrop ALPHA (`eff_back_a = back_a·(1−lf)`), which over an OPAQUE canvas punched
    // holes revealing the dark editor background behind the sprite (the dark-blur smoke).
    // Setup: opaque beige backdrop = paper everywhere, a red square PAINTED into the backdrop
    // (backdrop ≠ paper only there), `lifted_frac = 1` over the square, NO wet pigment.
    // The square must come back ~beige with alpha STAYING 255; outside the square the output
    // must be byte-identical to the backdrop (`lf = 0` ⇒ the byte-exact bare-paper copy).
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (32u32, 32u32);
    let scale = 1u32; // grid cell == canvas pixel, so the lf mask maps 1:1
    let (cw, ch) = (gw, gh);
    let beige = [228u8, 214, 184, 255];
    let red = [200u8, 30, 30, 255];
    let in_square = |x: u32, y: u32| (8..24).contains(&x) && (8..24).contains(&y);

    // Paper = solid opaque beige; backdrop = paper + the painted red square.
    let mut paper = vec![0u8; (cw * ch * 4) as usize];
    for px in paper.chunks_exact_mut(4) {
        px.copy_from_slice(&beige);
    }
    let mut backdrop = paper.clone();
    for y in 0..ch {
        for x in 0..cw {
            if in_square(x, y) {
                let i = ((y * cw + x) * 4) as usize;
                backdrop[i..i + 4].copy_from_slice(&red);
            }
        }
    }

    // lifted_frac: 1.0 over the square cells, 0 elsewhere.
    let mut lf = vec![0.0f32; (gw * gh) as usize];
    for gy in 0..gh {
        for gx in 0..gw {
            if in_square(gx, gy) {
                lf[(gy * gw + gx) as usize] = 1.0;
            }
        }
    }
    let lf_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test lifted_frac (lf=1 over square)"),
        size: (lf.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    gpu.queue
        .write_buffer(&lf_buf, 0, bytemuck::cast_slice(&lf));

    // NO wet pigment: a fresh (zero) field buffer — every glaze sample is dry.
    use ph2d_painter_brush::diffusion::PIG_CH;
    let pig_buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test zero pigment field"),
        size: ((gw * gh) as usize * PIG_CH * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let mut compositor = FluidCompositor::new(&gpu.device);
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        scale,
        COVERAGE_K,
        1,
        &pig_buf,
        &backdrop,
        &paper,
        Some(&lf_buf),
        None, // wet-sheen water: dormant
    );
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let (band, (_, py_lo, _, py_hi)) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    assert_eq!(py_lo, 0, "full-canvas region starts at row 0");
    assert_eq!(py_hi, ch, "full-canvas region covers every row");
    assert_eq!(band.len(), (cw * ch * 4) as usize, "full-canvas band");

    let mut worst_rgb = 0u8; // worst lifted-pixel |Δ| vs the paper, RGB only
    for y in 0..ch {
        for x in 0..cw {
            let i = ((y * cw + x) * 4) as usize;
            let px = &band[i..i + 4];
            if in_square(x, y) {
                // (a) Lifted: the red paint came back ~beige (sRGB→linear→sRGB roundtrip ⇒ a
                // few-LSB tolerance) and the ALPHA STAYED 255 — the key regression: lifting an
                // opaque canvas must NOT punch an alpha hole.
                for c in 0..3 {
                    worst_rgb = worst_rgb.max(px[c].abs_diff(beige[c]));
                    assert!(
                        px[c].abs_diff(beige[c]) <= 2,
                        "lifted pixel must return to the paper @({x},{y}) ch{c}: {px:?} vs {beige:?}"
                    );
                }
                assert_eq!(
                    px[3], 255,
                    "lifted pixel alpha must STAY opaque @({x},{y}): {px:?}"
                );
            } else {
                // (b) lf = 0 ⇒ the byte-exact backdrop copy (non-destructive invariant).
                assert_eq!(
                    px,
                    &backdrop[i..i + 4],
                    "untouched pixel must be byte-identical to the backdrop @({x},{y})"
                );
            }
        }
    }
    eprintln!(
        "paper-reveal: lifted square back to paper, worst RGB |Δ| = {worst_rgb} LSB, alpha = 255 \
         everywhere; outside byte-identical"
    );
}

// ─── E4: premultiplied preview-texture output ────────────────────────────────

/// LOCAL mirror of the CPU premultiply the shell applies before uploading the live
/// preview (`shells/desktop/src/render_loop/painter_bridge.rs` calls
/// `ph2d_render::premultiply_rgba8`): `rgb' = (rgb·a + 127) / 255` — integer
/// round-to-nearest on the sRGB-ENCODED bytes (no linearisation), alpha unchanged.
/// MUST stay byte-identical to that fn; the gates below prove the GPU
/// `cs_premul_tex`/`cs_premul_init` match it byte-for-byte, which is what makes the
/// E4 texture path a drop-in replacement for readback + CPU premultiply + re-upload.
fn premultiply_rgba8_local(rgba: &mut [u8]) {
    for px in rgba.chunks_exact_mut(4) {
        let a = px[3] as u32;
        for c in &mut px[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
}

/// Read the canvas-res rgba8 preview texture back to tightly-packed bytes.
/// `copy_texture_to_buffer` requires `bytes_per_row` aligned to 256 — copy padded,
/// then strip the padding per row.
fn read_texture_rgba8(gpu: &GpuContext, tex: &wgpu::Texture, cw: u32, ch: u32) -> Vec<u8> {
    let padded_bpr =
        (cw * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test preview-tex readback"),
        size: u64::from(padded_bpr) * u64::from(ch),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("test preview-tex copy"),
        });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(ch),
            },
        },
        wgpu::Extent3d {
            width: cw,
            height: ch,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    rx.recv().expect("map channel").expect("mapped");
    let mapped = staging.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((cw * ch * 4) as usize);
    for y in 0..ch {
        let off = (y * padded_bpr) as usize;
        out.extend_from_slice(&mapped[off..off + (cw * 4) as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

// ─── Wet-paper sheen (view-only preview-texture effect) ──────────────────────

/// f32 twins of the WGSL sRGB transfer fns (`srgb_to_linear`/`linear_to_srgb`).
#[test]
#[ignore = "needs a GPU device"]
fn wet_sheen_off_is_byte_identical() {
    // With the wet-sheen flag OFF (the default) the preview texture must be
    // byte-identical to the CPU-premultiply reference EVEN WITH a live (non-zero)
    // water buffer bound — the flag, not the binding, gates the effect. This pins
    // the non-destructive default: enabling the plumbing changes zero pixels.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (gw, gh) = (40u32, 32u32);
    let (cw, ch) = (gw * SCALE, gh * SCALE);
    let grid = seeded_field(gw, gh);
    let pig = grid.pigment();
    let region = (0u32, 0u32, gw - 1, gh - 1);
    let backdrop = split_backdrop(cw, ch);
    let solver = FluidSolver::new(&gpu.device, gw, gh);
    solver.upload(&gpu.queue, grid.water(), grid.paper(), pig);
    let mut compositor = FluidCompositor::new(&gpu.device);
    compositor.set_wet_sheen(false); // explicit (also the default)
    compositor.begin_stroke(
        &gpu.device,
        &gpu.queue,
        gw,
        gh,
        cw,
        ch,
        SCALE,
        COVERAGE_K,
        1,
        solver.pigment_buffer(),
        &backdrop,
        &backdrop,
        None,
        Some(solver.water_buffer()), // LIVE water bound — flag off must still be a no-op
    );
    let (band, rect) = compositor.composite_frame(&gpu.device, &gpu.queue, region);
    let (px_lo, py_lo, px_hi, py_hi) = rect;
    assert!(!band.is_empty(), "sync composite produced a band");
    let mut expected = backdrop.clone();
    premultiply_rgba8_local(&mut expected);
    for y in py_lo..py_hi {
        for x in px_lo..px_hi {
            let bi = (((y - py_lo) * cw + x) * 4) as usize;
            let mut px = [band[bi], band[bi + 1], band[bi + 2], band[bi + 3]];
            premultiply_rgba8_local(&mut px);
            expected[((y * cw + x) * 4) as usize..][..4].copy_from_slice(&px);
        }
    }
    let rect_tex = compositor
        .composite_frame_to_texture(&gpu.device, &gpu.queue, region)
        .expect("non-empty region composites");
    assert_eq!(rect_tex, rect);
    let tex = compositor.preview_texture().expect("stroke state live");
    let got = read_texture_rgba8(&gpu, tex, cw, ch);
    assert_eq!(
        expected, got,
        "wet_sheen OFF must be byte-identical to the CPU premultiply reference"
    );
    eprintln!("wet_sheen off: byte-identical over {cw}×{ch} (live water bound)");
}
