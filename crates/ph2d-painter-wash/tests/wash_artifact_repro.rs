//! ADR-0089 fidelity guards motivated by the "rectangular artifacts after undo" report (2026-06-13).
//! These pin that the FIELD layer is artifact-free, so any visible seam is a shell/render-integration
//! issue, not the wash core: (1) a snapshot→restore round-trip is byte-identical, and (2) region-
//! scoped stepping (the bridge's painting path) equals full-grid stepping in the wet area (the region
//! boundary sits in dry, gated-off cells ⇒ no seam).
//!   cargo test -p ph2d-painter-wash --features gpu --test wash_artifact_repro -- --ignored --nocapture
#![cfg(feature = "gpu")]

use ph2d_gpu::GpuContext;
use ph2d_painter_wash::km::KmModel;
use ph2d_painter_wash::{Dab, WashCompositor, WashParams, WashSolver};

fn try_gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// Max abs per-channel diff between two RGBA8 buffers (0 = identical).
fn max_diff(a: &[u32], b: &[u32]) -> i32 {
    a.iter()
        .zip(b)
        .flat_map(|(&x, &y)| {
            (0..4).map(move |s| ((x >> (8 * s)) & 0xff) as i32 - ((y >> (8 * s)) & 0xff) as i32)
        })
        .map(i32::abs)
        .max()
        .unwrap_or(0)
}

fn composite_full(gpu: &GpuContext, s: &WashSolver, w: u32, h: u32, model: u32) -> Vec<u32> {
    let n = (w * h) as usize;
    let mut comp = WashCompositor::new(&gpu.device);
    let backdrop = vec![0xffff_ffffu32; n]; // white
    comp.begin_stroke(
        &gpu.device,
        &gpu.queue,
        w,
        h,
        w,
        h,
        &backdrop,
        1.0,
        model,
        s.pig_buffer(),
        s.dye_buffer(),
        s.res_buffer(),
    );
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    comp.encode_composite(&gpu.queue, &mut enc, (0, 0, w, h));
    gpu.queue.submit([enc.finish()]);
    comp.read_preview(&gpu.device, &gpu.queue).unwrap()
}

/// A vertical red stroke of overlapping dabs (water bump + concentrations + dye), like the bridge.
fn red_stroke_dabs(km: &KmModel) -> Vec<Dab> {
    let color = [0.85f32, 0.15, 0.15];
    (0..9)
        .map(|i| {
            let cy = 22.0 + i as f32 * 6.0;
            let m = 0.6;
            let mut conc = km.rgb_to_concentrations(color);
            for c in &mut conc {
                *c *= m;
            }
            let dye = [color[0] * m, color[1] * m, color[2] * m, m];
            Dab::from_concentrations(34.0, cy, 11.0, 0.5, conc).with_dye(dye)
        })
        .collect()
}

#[test]
#[ignore = "needs a GPU device"]
fn snapshot_restore_is_byte_identical_and_region_step_has_no_seam() {
    let Some(gpu) = try_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (96u32, 96u32);
    let km = KmModel::new();
    let dabs = red_stroke_dabs(&km);
    let wp = WashParams {
        diffusivity: 0.14,
        flow_outward: 0.0,
        evaporation: 0.0,
        ..Default::default()
    };

    // DIRECT: splat + full-grid settle. The reference look.
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, wp);
    s.splat(&gpu.device, &gpu.queue, &dabs);
    s.step(&gpu.device, &gpu.queue, 30 * 4); // full-grid settle
    let direct_km = composite_full(&gpu, &s, w, h, 1);
    let direct_lin = composite_full(&gpu, &s, w, h, 0);

    // (1) Snapshot BOTH channels, restore into a FRESH solver, composite → must match DIRECT exactly
    // (the undo restore path — any non-zero diff means the snapshot/restore loses or warps the field).
    let snap_pig = s.read_pigment(&gpu.device, &gpu.queue);
    let snap_dye = s.read_dye(&gpu.device, &gpu.queue);
    let s2 = WashSolver::new(&gpu.device, w, h);
    s2.upload_pigment(&gpu.queue, &snap_pig);
    s2.upload_dye(&gpu.queue, &snap_dye);
    let d_km = max_diff(&direct_km, &composite_full(&gpu, &s2, w, h, 1));
    let d_lin = max_diff(&direct_lin, &composite_full(&gpu, &s2, w, h, 0));
    eprintln!("snapshot→restore max diff: K–M={d_km}  Linear={d_lin}  (want 0)");
    assert!(
        d_km == 0 && d_lin == 0,
        "snapshot→restore must be byte-identical (K–M={d_km} Lin={d_lin})"
    );

    // (2) Region-scoped step (the bridge's painting path) vs full-grid: identical in the wet area.
    let region = (14u32, 6u32, 40u32, 84u32); // envelope-ish bbox around the stroke
    let sr = WashSolver::new(&gpu.device, w, h);
    sr.set_params(&gpu.queue, wp);
    let enc = gpu.device.create_command_encoder(&Default::default());
    let enc = sr.encode_step(&gpu.queue, enc, &dabs, 30 * 4, region); // splat + region-step
    gpu.queue.submit([enc.finish()]);
    let d_region = max_diff(&direct_km, &composite_full(&gpu, &sr, w, h, 1));
    eprintln!(
        "region-step vs full-step max diff: K–M={d_region}  (want ~0; a localized jump = a seam)"
    );
    assert!(
        d_region < 2,
        "region-scoped stepping must not seam vs full-grid (diff={d_region})"
    );
}

/// ADR-0090 regression (Enio 2026-06-14: "a mancha que apaguei com o undo volta"): restoring a
/// snapshot then PAINTING a new stroke must NOT resurrect the undone pigment. The undo-restore writes
/// the field; the next region-scoped step copies the FULL twin buffer back over the canonical one, so
/// if the restore left the twin (`pig_b`/`dye_b`) stale, the paint resurrects the undone stroke
/// OUTSIDE the painted region. The control logic was always correct — this is the solver twin bug.
///   cargo test -p ph2d-painter-wash --features gpu --test wash_artifact_repro -- --ignored --nocapture
#[test]
#[ignore = "needs a GPU device"]
fn restore_then_paint_does_not_resurrect_undone_pigment() {
    let Some(gpu) = try_gpu() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let (w, h) = (96u32, 96u32);
    let km = KmModel::new();
    let wp = WashParams {
        diffusivity: 0.14,
        flow_outward: 0.0,
        evaporation: 0.0,
        ..Default::default()
    };
    let s = WashSolver::new(&gpu.device, w, h);
    s.set_params(&gpu.queue, wp);

    let blob = |cx: f32, cy: f32| -> Vec<Dab> {
        let color = [0.85f32, 0.15, 0.15];
        let m = 0.6;
        let mut conc = km.rgb_to_concentrations(color);
        for c in &mut conc {
            *c *= m;
        }
        let dye = [color[0] * m, color[1] * m, color[2] * m, m];
        vec![Dab::from_concentrations(cx, cy, 8.0, 0.5, conc).with_dye(dye)]
    };
    let region_mass = |field: &[[f32; 4]], x0: u32, y0: u32, x1: u32, y1: u32| -> f32 {
        let mut sum = 0.0;
        for y in y0..y1 {
            for x in x0..x1 {
                let p = field[(y * w + x) as usize];
                sum += p[0] + p[1] + p[2] + p[3];
            }
        }
        sum
    };

    // Stroke X (top-left) → snapshot S (the state an undo will return to).
    s.splat(&gpu.device, &gpu.queue, &blob(24.0, 24.0));
    s.step(&gpu.device, &gpu.queue, 12);
    let snap_pig = s.read_pigment(&gpu.device, &gpu.queue);
    let snap_dye = s.read_dye(&gpu.device, &gpu.queue);
    let snap_water = s.read_water(&gpu.device, &gpu.queue);

    // Stroke Y (bottom-right) → field = X + Y.
    s.splat(&gpu.device, &gpu.queue, &blob(72.0, 72.0));
    s.step(&gpu.device, &gpu.queue, 12);
    let y_before = region_mass(&s.read_pigment(&gpu.device, &gpu.queue), 60, 60, 84, 84);
    assert!(
        y_before > 0.01,
        "stroke Y must have laid pigment (got {y_before})"
    );

    // Undo Y: restore snapshot S (= X only) — colour AND water, writes both ping-pong twins.
    s.upload_pigment(&gpu.queue, &snap_pig);
    s.upload_dye(&gpu.queue, &snap_dye);
    s.upload_water(&gpu.queue, &snap_water);

    // Paint stroke Z (bottom-left) with a REGION-scoped step (the bridge's painting path), region
    // disjoint from Y. The full copy-back must NOT bring Y back.
    let z_region = (8u32, 56u32, 32u32, 32u32); // around (24, 72)
    let enc = gpu.device.create_command_encoder(&Default::default());
    let enc = s.encode_step(&gpu.queue, enc, &blob(24.0, 72.0), 3, z_region);
    gpu.queue.submit([enc.finish()]);

    let field = s.read_pigment(&gpu.device, &gpu.queue);
    let y_after = region_mass(&field, 60, 60, 84, 84);
    let x_mass = region_mass(&field, 12, 12, 36, 36);
    let z_mass = region_mass(&field, 12, 60, 36, 84);
    eprintln!(
        "after restore+paint: X={x_mass:.3} Y={y_after:.3} (was {y_before:.3}) Z={z_mass:.3} — Y must be ~0"
    );
    assert!(
        y_after < 0.001,
        "UNDONE stroke Y resurrected after painting (Y mass {y_after}, was {y_before}) — stale twin buffer"
    );
    assert!(
        x_mass > 0.01,
        "restored stroke X must remain (got {x_mass})"
    );
    assert!(z_mass > 0.01, "new stroke Z must be present (got {z_mass})");

    // ...and the undo must be COMPLETE: Y's WATER is gone too (else its area stays wet forever at
    // Evaporation 0 and bleeds into later strokes — the incomplete-undo bug, Enio 2026-06-14).
    let water = s.read_water(&gpu.device, &gpu.queue);
    let mut y_water = 0.0f32;
    for y in 60..84u32 {
        for x in 60..84u32 {
            y_water += water[(y * w + x) as usize];
        }
    }
    assert!(
        y_water < 0.001,
        "UNDONE stroke Y's WATER persisted after restore+paint ({y_water}) — incomplete undo"
    );
}
