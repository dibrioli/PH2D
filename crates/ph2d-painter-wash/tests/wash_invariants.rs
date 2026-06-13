//! Physical-invariant gates for the minimal watercolor core (ADR-0086 §9). The shader is
//! the single source of truth; we assert the PHYSICS holds with loose tolerances (a correct
//! kernel passes easily, a structural bug breaks an invariant by a wide margin).
//!
//! `#[ignore]` — needs a real device:
//!   cargo test -p ph2d-painter-wash --features gpu --test wash_invariants -- --ignored --nocapture
#![cfg(feature = "gpu")]

use ph2d_gpu::GpuContext;
use ph2d_painter_wash::{Dab, WashCompositor, WashParams, WashSolver};

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn idx(x: u32, y: u32, w: u32) -> usize {
    (y * w + x) as usize
}

/// Σ pigment mass (channel `.w`) — the conserved quantity under pure diffusion.
fn total_mass(pig: &[[f32; 4]]) -> f64 {
    pig.iter().map(|p| f64::from(p[3])).sum()
}

/// A uniformly-wet field with a centred square pigment blob (mass=1, a red-ish absorbance).
fn seed_blob(w: u32, h: u32, water: f32, blob: u32) -> (Vec<f32>, Vec<f32>, Vec<[f32; 4]>) {
    let n = (w * h) as usize;
    let water_f = vec![water; n];
    let paper = vec![0.5f32; n];
    let mut pig = vec![[0.0f32; 4]; n];
    let (cx, cy) = (w / 2, h / 2);
    for y in 0..h {
        for x in 0..w {
            if x.abs_diff(cx) <= blob && y.abs_diff(cy) <= blob {
                pig[idx(x, y, w)] = [0.20, 0.10, 0.05, 1.0];
            }
        }
    }
    (water_f, paper, pig)
}

// ── INV-1 — pigment mass is conserved (pure diffusion: λ=0, evap=0) ──────────────────
#[test]
#[ignore = "needs a GPU device"]
fn inv_mass_conserved_under_diffusion() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (32u32, 32u32);
    let (water, paper, pig) = seed_blob(w, h, 0.9, 4);
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, WashParams { diffusivity: 0.22, flow_outward: 0.0, evaporation: 0.0, ..Default::default() });
    s.upload(&gpu.queue, &water, &paper, &pig);
    let before = total_mass(&s.read_pigment(&gpu.device, &gpu.queue));
    s.step(&gpu.device, &gpu.queue, 128);
    let after = total_mass(&s.read_pigment(&gpu.device, &gpu.queue));
    eprintln!("mass: {before:.5} → {after:.5}");
    assert!((after - before).abs() < before * 0.01, "diffusion must conserve pigment mass: {before} → {after}");
}

// ── INV-2 — the bloom: diffusion spreads pigment (centre drops, ring rises); D has effect ─
#[test]
#[ignore = "needs a GPU device"]
fn inv_diffusion_blooms_and_has_slider_effect() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 48u32);
    let (cx, cy) = (w / 2, h / 2);
    let run = |d: f32| -> (f64, f64) {
        let (water, paper, pig) = seed_blob(w, h, 0.9, 3);
        let s = WashSolver::new(&gpu.device, w, h);
        s.set_params(&gpu.queue, WashParams { diffusivity: d, flow_outward: 0.0, evaporation: 0.0, ..Default::default() });
        s.upload(&gpu.queue, &water, &paper, &pig);
        let c0 = f64::from(s.read_pigment(&gpu.device, &gpu.queue)[idx(cx, cy, w)][3]);
        s.step(&gpu.device, &gpu.queue, 64);
        let pig = s.read_pigment(&gpu.device, &gpu.queue);
        let centre = f64::from(pig[idx(cx, cy, w)][3]);
        let ring = f64::from(pig[idx(cx + 8, cy, w)][3]); // outside the initial blob
        (centre / c0, ring) // centre as fraction of its start, ring absolute
    };
    let (centre_lo, ring_lo) = run(0.05);
    let (centre_hi, ring_hi) = run(0.24);
    eprintln!("D=0.05 centre%={centre_lo:.3} ring={ring_lo:.5} | D=0.24 centre%={centre_hi:.3} ring={ring_hi:.5}");
    assert!(centre_hi < 0.9, "diffusion must lower the centre peak");
    assert!(ring_hi > 1e-5, "diffusion must carry pigment outward (bloom)");
    assert!(centre_hi < centre_lo, "higher D must spread MORE (lower centre)");
    assert!(ring_hi > ring_lo, "higher D must reach the ring MORE");
}

// ── INV-3 — edge-darkening: a realistic dab (water bump, high centre → dry rim) drives
//    FlowOutward, migrating pigment OUTWARD; the perimeter ends darker than the centre. ──
#[test]
#[ignore = "needs a GPU device"]
fn inv_flow_outward_darkens_the_rim() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 48u32);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let radius = 14.0f32;
    let n = (w * h) as usize;
    // A realistic dab: a RADIAL WATER BUMP (wet centre → dry rim, so ∇w ≠ 0) with UNIFORM
    // pigment across the wet region. This is the water profile a falloff brush actually lays.
    let mut water = vec![0.0f32; n];
    let paper = vec![0.5f32; n];
    let mut pig = vec![[0.0f32; 4]; n];
    for y in 0..h {
        for x in 0..w {
            let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            if r < radius {
                let t = 1.0 - r / radius; // 1 centre → 0 rim
                water[idx(x, y, w)] = 0.15 + 0.75 * t; // wet everywhere, peaked at the centre
                pig[idx(x, y, w)] = [0.20, 0.10, 0.05, 0.5]; // uniform pigment
            }
        }
    }
    let s = WashSolver::new(&gpu.device, w, h);
    // Strong outward drift, low diffusion (don't just symmetrise it back), gentle drying.
    s.set_params(&gpu.queue, WashParams { diffusivity: 0.03, flow_outward: 2.0, evaporation: 0.008, ..Default::default() });
    s.upload(&gpu.queue, &water, &paper, &pig);
    s.step(&gpu.device, &gpu.queue, 80);
    let pig = s.read_pigment(&gpu.device, &gpu.queue);

    // Means measured strictly INSIDE the wet region (avoid the dry exterior).
    let ring_mean = |r0: f32, r1: f32| -> f64 {
        let mut sum = 0.0;
        let mut cnt = 0.0;
        for y in 0..h {
            for x in 0..w {
                let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                if r >= r0 && r < r1 {
                    sum += f64::from(pig[idx(x, y, w)][3]);
                    cnt += 1.0;
                }
            }
        }
        if cnt > 0.0 { sum / cnt } else { 0.0 }
    };
    let centre = ring_mean(0.0, 3.0);
    let outer = ring_mean(9.0, 12.0); // interior ring, safely inside the wet disk
    eprintln!("edge-darkening: centre mean={centre:.5}  outer-ring mean={outer:.5}  (outer/centre={:.2})", outer / centre.max(1e-9));
    assert!(outer > centre * 1.15, "FlowOutward must migrate pigment outward (outer ring {outer} > centre {centre})");
}

// ── INV-4 — stability: extreme params, 1000 substeps, no NaN / no runaway ───────────
#[test]
#[ignore = "needs a GPU device"]
fn inv_stable_under_extreme_params() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let (water, paper, pig) = seed_blob(w, h, 0.95, 6);
    let before = total_mass(&pig);
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, WashParams { diffusivity: 0.25, flow_outward: 2.0, evaporation: 0.0005, ..Default::default() });
    s.upload(&gpu.queue, &water, &paper, &pig);
    s.step(&gpu.device, &gpu.queue, 1000);
    let pig = s.read_pigment(&gpu.device, &gpu.queue);
    let after = total_mass(&pig);
    let finite = pig.iter().all(|p| p.iter().all(|v| v.is_finite()));
    let max_mass = pig.iter().map(|p| p[3]).fold(0.0f32, f32::max);
    eprintln!("stability: finite={finite} max_mass={max_mass:.4} mass {before:.3}→{after:.3}");
    assert!(finite, "field must stay finite (no NaN/Inf)");
    assert!(max_mass < 10.0, "no runaway concentration (max mass {max_mass})");
    assert!(after <= before * 1.001, "mass must not grow (no source term)");
}

// ── INV-5 — subtractive compositing (WashCompositor → preview texture): stacked absorbance
//    darkens multiplicatively over a white backdrop. ─────────────────────────────────
#[test]
#[ignore = "needs a GPU device"]
fn inv_subtractive_compositing_darkens() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32); // 64 → bytes-per-row already 256-aligned
    let n = (w * h) as usize;
    let water = vec![0.0f32; n];
    let paper = vec![0.5f32; n];
    // cell 0: one glaze (absorb green+blue); cell 1: two glazes stacked (double absorbance).
    let mut pig = vec![[0.0f32; 4]; n];
    pig[0] = [0.0, 0.7, 0.7, 1.0];
    pig[1] = [0.0, 1.4, 1.4, 1.0];
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, WashParams::default());
    s.upload(&gpu.queue, &water, &paper, &pig);

    let mut comp = WashCompositor::new(&gpu.device);
    let backdrop = vec![0xffff_ffffu32; n]; // white, opaque
    comp.begin_stroke(&gpu.device, &gpu.queue, w, h, w, h, &backdrop, 1.0, s.pig_buffer());
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    comp.encode_composite(&gpu.queue, &mut enc, (0, 0, w, h));
    gpu.queue.submit([enc.finish()]);
    let out = comp.read_preview(&gpu.device, &gpu.queue).unwrap();
    let g = |px: usize| ((out[px] >> 8) & 0xff) as i32;
    let r = |px: usize| (out[px] & 0xff) as i32;
    eprintln!("composite: single G={} double G={} (R stays {})", g(0), g(1), r(0));
    assert_eq!(r(0), 255, "no red absorbance ⇒ red channel stays white");
    assert!((120..=235).contains(&g(0)), "single green glaze: exp(-0.7) in linear → sRGB ≈ 188");
    assert!(g(1) < g(0) - 20, "stacking glazes must darken green further (subtractive)");
}

// ── INV-6 — preview texture: a coloured DAB glazes the real backdrop (Beer–Lambert), tints
//    the painted area toward the pigment + darkens it; bare pixels stay the backdrop. ───
#[test]
#[ignore = "needs a GPU device"]
fn wash_preview_texture_premul() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let n = (w * h) as usize;
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, WashParams::default());
    s.upload(&gpu.queue, &vec![0.0f32; n], &vec![0.5f32; n], &vec![[0.0f32; 4]; n]);
    // A blue-ish pigment dab in the centre.
    let dab = Dab::from_color_mass(32.0, 32.0, 10.0, 0.9, [0.2, 0.4, 0.9], 1.0);
    s.splat(&gpu.device, &gpu.queue, &[dab]);

    let mut comp = WashCompositor::new(&gpu.device);
    let gray = 153u32; // sRGB ~0.6
    let bd = gray | (gray << 8) | (gray << 16) | (0xff << 24);
    let backdrop = vec![bd; n];
    comp.begin_stroke(&gpu.device, &gpu.queue, w, h, w, h, &backdrop, 1.0, s.pig_buffer());
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    comp.encode_composite(&gpu.queue, &mut enc, (0, 0, w, h));
    gpu.queue.submit([enc.finish()]);
    let out = comp.read_preview(&gpu.device, &gpu.queue).unwrap();

    let px = |x: u32, y: u32| out[(y * w + x) as usize];
    let chan = |word: u32, i: u32| ((word >> (8 * i)) & 0xff) as i32;
    let centre = px(32, 32);
    let corner = px(0, 0);
    eprintln!(
        "preview centre=({},{},{}) corner=({},{},{})",
        chan(centre, 0), chan(centre, 1), chan(centre, 2),
        chan(corner, 0), chan(corner, 1), chan(corner, 2),
    );
    // Bare corner == backdrop, untouched.
    assert_eq!((chan(corner, 0), chan(corner, 1), chan(corner, 2)), (153, 153, 153), "bare pixel = backdrop");
    // Painted centre: blue-tinted (B > R, B > G) and darkened (R below the backdrop's 153).
    assert!(chan(centre, 2) > chan(centre, 0), "blue pigment ⇒ B > R");
    assert!(chan(centre, 2) > chan(centre, 1), "blue pigment ⇒ B > G");
    assert!(chan(centre, 0) < 153, "pigment must darken the backdrop");
    assert_eq!(chan(centre, 3), 255, "preview is opaque");
}

// ── INV-7 — paper saturation (B1 regression): overlapping the SAME spot far past the paper's
//    capacity must converge to the pigment's masstone, NEVER to black. ─────────────────────
#[test]
#[ignore = "needs a GPU device"]
fn inv_overlap_saturates_to_pigment_not_black() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (64u32, 64u32);
    let n = (w * h) as usize;
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, WashParams::default());
    s.upload(&gpu.queue, &vec![0.0f32; n], &vec![0.5f32; n], &vec![[0.0f32; 4]; n]);
    // Hammer the centre with the SAME red dab 50× — mass piles to ~50, far over MASS_MAX=1.
    // Pre-cap this drove absorb→∞ ⇒ exp(−absorb)→0 = pure black (the reported B1).
    let red: [f32; 3] = [0.7, 0.1, 0.1];
    let dab = Dab::from_color_mass(32.0, 32.0, 8.0, 0.0, red, 1.0);
    for _ in 0..50 {
        s.splat(&gpu.device, &gpu.queue, &[dab]);
    }
    let accumulated_mass = s.read_pigment(&gpu.device, &gpu.queue)[idx(32, 32, w)][3];
    assert!(accumulated_mass > 10.0, "test must actually pile mass past the cap (got {accumulated_mass})");

    let mut comp = WashCompositor::new(&gpu.device);
    let backdrop = vec![0xffff_ffffu32; n]; // white
    comp.begin_stroke(&gpu.device, &gpu.queue, w, h, w, h, &backdrop, 1.0, s.pig_buffer());
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    comp.encode_composite(&gpu.queue, &mut enc, (0, 0, w, h));
    gpu.queue.submit([enc.finish()]);
    let out = comp.read_preview(&gpu.device, &gpu.queue).unwrap();
    let chan = |word: u32, i: u32| ((word >> (8 * i)) & 0xff) as i32;
    let centre = out[(32 * w + 32) as usize];
    let (cr, cg, cb) = (chan(centre, 0), chan(centre, 1), chan(centre, 2));
    eprintln!("saturation: mass={accumulated_mass:.1} centre=({cr},{cg},{cb}) (cap → pigment masstone, c≈0.7,0.1,0.1)");
    // The killer assertion: not black. The dominant (red) channel must stay bright.
    assert!(cr > 150, "saturated overlap must stay the pigment colour, not go black (R={cr})");
    // Still recognisably red (the masstone), not a grey mud.
    assert!(cr > cg + 40 && cr > cb + 40, "masstone must keep its hue (R={cr} G={cg} B={cb})");
    assert_eq!(chan(centre, 3), 255, "preview is opaque");
}

// ── INV-8 — positivity / NO checkerboard (keep-wet/evap-0 regression): a heavy dab at a water
//    peak with extreme diffusion+advection must NOT decouple into the odd-even dither. The old
//    kernel clamped D and v independently (combined outflux > 1) ⇒ cells went negative ⇒ the
//    max(·,0) clamp snapped them to white holes between red cells. The shared CFL budget forbids it.
#[test]
#[ignore = "needs a GPU device"]
fn inv_no_checkerboard_under_extreme_flow() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (48u32, 48u32);
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let radius = 16.0f32;
    let n = (w * h) as usize;
    // A realistic dab: radial water bump (drives FlowOutward forever, the keep-wet case) with a
    // uniform pigment disk. EXTREME params: max diffusion AND max advection at the same cells —
    // the exact worst case for combined-CFL overshoot.
    let mut water = vec![0.0f32; n];
    let paper = vec![0.5f32; n];
    let mut pig = vec![[0.0f32; 4]; n];
    for y in 0..h {
        for x in 0..w {
            let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            if r < radius {
                water[idx(x, y, w)] = 0.2 + 0.8 * (1.0 - r / radius);
                pig[idx(x, y, w)] = [0.20, 0.10, 0.05, 1.0];
            }
        }
    }
    let s = WashSolver::new(&gpu.device, w, h);
    // Pathological: diffusivity ABOVE the cap + huge flow_outward, no evaporation (keep-wet).
    s.set_params(&gpu.queue, WashParams { diffusivity: 0.25, flow_outward: 5.0, evaporation: 0.0, ..Default::default() });
    s.upload(&gpu.queue, &water, &paper, &pig);
    s.step(&gpu.device, &gpu.queue, 400);
    let out = s.read_pigment(&gpu.device, &gpu.queue);

    let m = |x: u32, y: u32| f64::from(out[idx(x, y, w)][3]);
    // Checkerboard signature: an interior wet cell that collapsed to ~0 while its 4 neighbours
    // hold pigment (a white hole between red cells). A positive scheme produces NONE.
    let mut holes = 0;
    let mut finite = true;
    for y in 2..h - 2 {
        for x in 2..w - 2 {
            let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            if r >= radius - 2.0 {
                continue; // interior only
            }
            for v in &out[idx(x, y, w)] {
                finite &= v.is_finite();
            }
            let nb = (m(x - 1, y) + m(x + 1, y) + m(x, y - 1) + m(x, y + 1)) / 4.0;
            if nb > 0.02 && m(x, y) < 0.15 * nb {
                holes += 1;
            }
        }
    }
    eprintln!("checkerboard: {holes} interior holes (want 0), finite={finite}");
    assert!(finite, "field must stay finite");
    assert_eq!(holes, 0, "positive scheme must not punch white holes (checkerboard) — found {holes}");
}
