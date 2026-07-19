//! Gates for the impasto **displacement** — the Push / bow wave / rim anchor family
//! ([`crate::height_push`]), split from [`super::tests`] for the workspace file-LOC cap.
//!
//! They live together because they all drive the SAME fixture: a plough shoved through a uniform ground,
//! read as a displacement plane. The sibling file gates what the brush DEPOSITS; this one gates what it
//! SHOVES OUT OF THE WAY.

/// Drive a plough (uniform 1-load ground, a straight run of overlapping dabs with `impasto_push`)
/// through the REAL kernel sequence the deposit runs — un-paint the standing wave lobe, bite, bank
/// with `share`, re-lay the lobe at the new tip — and return the displacement plane.
fn ploughed_plane(share: f32, count: u32) -> (Vec<f32>, f32, f32, f32, f32) {
    use crate::height::{HeightDab, HeightFields, accumulate_dab_height};
    use crate::height_push::{PushBite, bank_dab_push, rim_t0, wave_lobe};
    const W: u32 = 200;
    let n = (W * W) as usize;
    let radius = 12.0f32;
    let ground = vec![1.0f32; n];
    let mut plane = vec![0.0f32; n];
    let mut height = vec![0.0f32; n];
    let mut paint = vec![0.0f32; n];
    let mut grain = vec![0u8; n];
    let mut film = vec![0u8; n];
    let mut radius_pl = vec![0.0f32; n];
    let mut scratch = Vec::new();
    let spec = crate::BrushSpec {
        radius_px: radius,
        impasto: true,
        impasto_depth: 0.5,
        impasto_push: 1.0,
        space_attenuation: false,
        ..Default::default()
    };
    // The default falloff is a soft Smooth, so the rim anchors INSIDE the geometric rim (`t0 ≈ 0.6`)
    // — the whole point of the 2026-07-15 fix. Stroke-constant, resolved once, as the tool does.
    let t0 = rim_t0(&spec, false);
    let (y, x0, step) = (100.0f32, 60.0f32, 6.0f32);
    let mut wave = 0.0f32;
    let mut last_tip: Option<HeightDab> = None;
    for k in 0..count {
        let cx = x0 + step * k as f32;
        let dab = HeightDab {
            center: [cx, y],
            radius,
            coverage: 1.0,
            footprint: spec.footprint_deform(),
            prev_center: (k > 0).then_some([cx - step, y]),
            shape: None,
            grain: None,
            grain_image: None,
        };
        // The tool's order: un-paint the standing lobe FIRST (paint unchanged since it was laid,
        // so the (1 − paint) weights recompute to the exact numbers that laid it).
        if let (Some(tip), true) = (last_tip.take(), wave > 0.0) {
            let _ = wave_lobe(&mut plane, &paint, &mut scratch, W, W, &tip, t0, wave, -1.0);
        }
        let mut fields = HeightFields {
            height: &mut height,
            paint: &mut paint,
            grain: &mut grain,
            film: &mut film,
            radius: &mut radius_pl,

            accum: None,
        };
        let mut bite = PushBite {
            ground: &ground,
            plane: &mut plane,
            displaced: 0.0,
        };
        let _ = accumulate_dab_height(&mut fields, W, W, &spec, &dab, Some(&mut bite));
        let displaced = bite.displaced;
        let (_, carried) = bank_dab_push(
            &mut plane,
            &paint,
            &mut scratch,
            W,
            W,
            &dab,
            t0,
            displaced,
            share,
        );
        wave += carried;
        if wave > 0.0
            && wave_lobe(&mut plane, &paint, &mut scratch, W, W, &dab, t0, wave, 1.0).is_some()
        {
            last_tip = Some(HeightDab { ..dab });
        }
        // The ledger closes at EVERY step, not just at the end — a lobe that lays more than it
        // un-lays would still sum to zero eventually if a later error cancelled it.
        let ledger: f64 = plane.iter().map(|&v| f64::from(v)).sum();
        assert!(
            ledger.abs() < 0.5,
            "dab {k}: the plane's ledger drifted to {ledger:+.3} loads*px^2 — displacement must \
             move paint, never create or destroy it"
        );
    }
    (plane, x0, step * (count - 1) as f32 + x0, radius, t0)
}

/// Zone split of a plough's banked (positive) plane: (ahead of the end, behind the start,
/// lateral, inside the swath), as fractions of the banked total.
///
/// `edge` is the stroke's BODY half-width (`t0 · radius`) — the paint's own frontier, which is where
/// the rim now anchors (2026-07-15). "Ahead of the stroke" means ahead of the PAINT, not ahead of the
/// dab's geometric circle; measuring against `radius` would count the ridge butting the body's front
/// edge as if it sat inside the channel. With a hard falloff (`t0 = 1`) `edge = radius` and the split
/// is exactly the pre-fix one.
fn plough_zones(plane: &[f32], first_x: f32, last_x: f32, edge: f32) -> (f64, f64, f64, f64) {
    const W: usize = 200;
    let y = 100.0f32;
    let (mut ahead, mut behind, mut lateral, mut total) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for py in 0..W {
        for px in 0..W {
            let v = plane[py * W + px];
            if v <= 0.0 {
                continue;
            }
            total += f64::from(v);
            let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
            if fx > last_x + edge {
                ahead += f64::from(v);
            } else if fx < first_x - edge {
                behind += f64::from(v);
            } else if (fy - y).abs() > edge {
                lateral += f64::from(v);
            }
        }
    }
    let swath = total - ahead - behind - lateral;
    (
        ahead / total,
        behind / total,
        lateral / total,
        swath / total,
    )
}

/// **The ploughed paint waits at the stroke's FRONTIER — the bow wave** (Enio's fila-última item:
/// *"o desenho da tinta deslocada não convence"*; diagnosed by rendering: the displaced paint
/// ringed the whole footprint like a stamped cookie-cutter — as tall behind the START as anywhere
/// — because every dab banked around itself; a blade pushes a WAVE ahead of its tip, IMPaSTo NPAR
/// 2004, `v_p = −c∇p`).
///
/// Per-dab forward *banking* cannot make that wave: the next dab paints over it and the envelope
/// bite (a one-shot `ground × Δm`) cannot re-transport it — measured: 0.4% ever landed ahead and
/// 16% fossilised inside the swath, which is MUT S quantified. So the wave is a session SCALAR
/// ([`crate::height_push::DEPOSIT_FORWARD_SHARE`] of each bite joins it, the rest sheds into the
/// lateral wake) painted as an exactly-removable lobe at the CURRENT tip ([`wave_lobe`]) — it
/// travels with the brush and rests at the frontier when the stroke ends.
///
/// Measured on this fixture, against the PAINT's frontier (the body edge `t0 ≈ 0.60`, where the rim
/// now anchors): share 0.6 ⇒ ahead 52.2% · lateral 41.8% · behind 5.7% · swath 0.3%; share 0 (the old
/// purely-lateral drawing) ⇒ ahead 1.2% · lateral 92.4%. The bars are an order of magnitude on both
/// sides. (Before the 2026-07-15 anchor fix, measured against the geometric rim: ahead 42.8%.)
///
/// **Mutations that must bleed:** (a) `DEPOSIT_FORWARD_SHARE → 0.0` — no wave, `ahead` collapses;
/// (b) drop the un-paint of the standing lobe — a fossil lobe trail down the whole swath, the
/// swath bound explodes; (c) un-negate `sweep_axis` in [`wave_lobe`] — the "wave" paints backward
/// into the swath.
#[test]
fn the_ploughed_paint_waits_at_the_strokes_frontier() {
    use crate::height_push::DEPOSIT_FORWARD_SHARE;
    let (plane, first_x, last_x, radius, t0) = ploughed_plane(DEPOSIT_FORWARD_SHARE, 12);
    let (ahead, behind, lateral, swath) = plough_zones(&plane, first_x, last_x, t0 * radius);
    assert!(
        ahead >= 0.30,
        "only {:.1}% of the banked paint waits ahead of the stroke's end — the wave never made it \
         to the frontier (measured healthy: 52.2%)",
        ahead * 100.0
    );
    assert!(
        ahead >= 5.0 * behind,
        "the frontier ({:.1}%) must dwarf the start ({:.1}%) — a blade leaves nothing standing \
         where it set off; symmetry here is the stamped-cookie-cutter drawing this gate exists to \
         refuse",
        ahead * 100.0,
        behind * 100.0
    );
    assert!(
        lateral >= 0.25,
        "the lateral wake vanished ({:.1}%) — the wave should SHED as it rolls, leaving ridges \
         along the channel's sides",
        lateral * 100.0
    );
    assert!(
        swath <= 0.15,
        "{:.1}% of the displaced paint sits INSIDE the channel — either the standing lobe is not \
         being un-painted before the tip moves (a fossil trail), or the lobe points backward",
        swath * 100.0
    );

    // The presence sibling: with the share at 0 the old, purely lateral drawing must come back —
    // this pins that the wave is OPT-IN plumbing.
    let (plane0, f0, l0, r0, t0_0) = ploughed_plane(0.0, 12);
    let (ahead0, _, lateral0, _) = plough_zones(&plane0, f0, l0, t0_0 * r0);
    assert!(
        ahead0 < 0.05 && lateral0 > 0.75,
        "share 0 must reproduce the purely lateral bank (ahead {:.1}%, lateral {:.1}%)",
        ahead0 * 100.0,
        lateral0 * 100.0
    );
}

/// **The wave TRAVELS — it stands at the current tip, not where it used to be.** Real-time was
/// the first complaint the Push ever drew (*"não em tempo real"*, 2026-07-12), and a wave that
/// only appears at pen-up would be the same failure in new clothes. Mid-plough, the standing
/// positive plane ahead of the CURRENT tip must dwarf whatever remains around the tip of several
/// dabs ago (bit-for-bit zero, in fact — the un-paint is an exact negation; the tolerance here is
/// only the lateral wake the old tip legitimately keeps beside itself).
#[test]
fn the_wave_travels_with_the_tip() {
    use crate::height_push::DEPOSIT_FORWARD_SHARE;
    const W: usize = 200;
    let (plane, _, last_x, radius, _t0) = ploughed_plane(DEPOSIT_FORWARD_SHARE, 12);
    let y = 100.0f32;
    // Positive plane in the forward half-disc of a tip position.
    let probe = |cx: f32| -> f64 {
        let mut sum = 0.0f64;
        for py in 0..W {
            for px in 0..W {
                let v = plane[py * W + px];
                if v <= 0.0 {
                    continue;
                }
                let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - y);
                if dx > 0.0 && (dx * dx + dy * dy) <= (radius * 2.2) * (radius * 2.2) {
                    sum += f64::from(v);
                }
            }
        }
        sum
    };
    let at_tip = probe(last_x);
    let mid_x = last_x - 6.0 * 6.0; // the tip six dabs ago
    let at_old_tip = probe(mid_x) - probe(last_x).min(0.0);
    assert!(
        at_tip > 0.0,
        "no wave stands ahead of the final tip — the lobe never landed"
    );
    assert!(
        at_tip >= 2.0 * at_old_tip,
        "the wave left as much standing at an OLD tip ({at_old_tip:.1}) as at the current one \
         ({at_tip:.1}) — it is being laid but not taken back up: a trail, not a travelling wave"
    );
}

const IB_W: u32 = 256;
const IB_CENTER: f32 = 128.0;

/// A single directed dab's lateral bank, in ISOLATION — a fresh plane, no bite, no neighbours, and no
/// paint anywhere (so no `(1 − paint)` suppression). The plane is PURELY the rim, so where its first
/// positive texel sits on the perpendicular is exactly where the anchor `t0` puts it — undiluted by
/// the bite's tail or a neighbour's diagonal bank (which is why the integrated plough's inner edge is
/// noisy). The dab carries a synthetic predecessor one radius back, so its heading is `+x` and the
/// whole annulus banks LATERALLY (±y): the axis the inner edge is read on. share 0.0 ⇒ all lateral.
fn isolated_bank_plane(spec: &crate::BrushSpec, t0: f32) -> Vec<f32> {
    use crate::height::HeightDab;
    use crate::height_push::bank_dab_push;
    let radius = spec.radius_px;
    let center = [IB_CENTER, IB_CENTER];
    let dab = HeightDab {
        center,
        radius,
        coverage: 1.0,
        footprint: spec.footprint_deform(),
        prev_center: Some([center[0] - radius, center[1]]),
        shape: None,
        grain: None,
        grain_image: None,
    };
    let mut plane = vec![0.0f32; (IB_W * IB_W) as usize];
    let paint = vec![0.0f32; (IB_W * IB_W) as usize];
    let mut scratch = Vec::new();
    let _ = bank_dab_push(
        &mut plane,
        &paint,
        &mut scratch,
        IB_W,
        IB_W,
        &dab,
        t0,
        1000.0,
        0.0,
    );
    plane
}

/// The perpendicular inner edge of an isolated bank plane: first positive texel scanning +y from the
/// dab centre, in pixels. `u8::MAX` if no ridge is found — a loud failure, not a silent zero.
fn isolated_inner_edge(plane: &[f32], radius: f32) -> f32 {
    let (cx, cy) = (IB_CENTER as usize, IB_CENTER as usize);
    for dy in 0..(radius as usize + 24) {
        if plane[(cy + dy) * IB_W as usize + cx] > 1e-4 {
            return dy as f32;
        }
    }
    f32::from(u8::MAX)
}

fn impasto_spec(falloff: crate::Falloff, radius: f32) -> crate::BrushSpec {
    crate::BrushSpec {
        radius_px: radius,
        impasto: true,
        impasto_depth: 0.5,
        impasto_push: 1.0,
        falloff,
        space_attenuation: false,
        ..Default::default()
    }
}

/// **The banked ridge rises from the paint's BODY edge — not the dab's geometric circle.**
///
/// Enio's smoke, 2026-07-15: *"é usada a circunferência do gizmo do brush para empurrar a massa e não
/// o alpha do falloff"* — on a soft falloff the displaced paint stood a whole silhouette's-width of
/// bare canvas out from the paint, a hard, perfectly circular collar clamped to the geometric rim
/// (`t = 1`). The physics of the wave was right; its ANCHOR was wrong. The rim now begins at
/// [`crate::height_push::rim_t0`] — the body edge `t0`, where the silhouette crosses [`W_TAIL`], the
/// SAME threshold the film uses to decide a texel carries paint.
///
/// The zone gates ([`the_ploughed_paint_waits_at_the_strokes_frontier`]) never pinned this — they
/// measure where the volume GOES (ahead / beside / behind), not where the ridge's inner edge is BORN.
/// This is the missing pin, on the handoff's own example (a Smooth brush of radius 40): the ridge's
/// foot sits at the body edge (`t0 ≈ 0.60`, so ~24 px), well inside the geometric rim (40 px) where
/// the old anchor stood it — the 16 px of bare canvas the smoke saw.
///
/// **Mutation that must bleed:** anchor the rim at the rim again — `rim_lift`'s guard `t <= t0` back
/// to `t <= 1.0`, or the caller's `rim_t0` → `1.0`. The inner edge jumps out to ~`radius` and the
/// `radius`-referenced bound (not the `t0`-referenced one, which the mutation moves with it) goes red.
/// The companion [`a_hard_falloff_anchors_the_rim_at_the_geometric_rim`] pins that a Constant brush is
/// byte-identical, so the fix cannot be "just always recede".
#[test]
fn the_rim_rises_from_the_body_edge_not_the_geometric_rim() {
    use crate::height_push::rim_t0;
    let radius = 40.0f32;
    let spec = impasto_spec(crate::Falloff::Smooth, radius);
    let t0 = rim_t0(&spec, false);
    let plane = isolated_bank_plane(&spec, t0);
    let inner = isolated_inner_edge(&plane, radius);
    let body_edge = t0 * radius;
    // The ridge's foot is AT the paint's body edge (±3 px), not floating somewhere else.
    assert!(
        (inner - body_edge).abs() <= 3.0,
        "the rim's inner edge is {inner:.1} px, but the paint's body ends at {body_edge:.1} px \
         (t0={t0:.2}·r) — the ridge should butt against the paint"
    );
    // The crisp, NON-self-referential discriminator: the inner edge is clearly INSIDE the geometric
    // rim. Referenced against `radius` (a fixed fact), not `t0` (which the anchor mutation moves), so
    // reverting the anchor to the rim pushes `inner` out to ~40 px, past this bound, and it goes red.
    assert!(
        inner < radius * 0.75,
        "the inner edge {inner:.1} px is not clearly inside the geometric rim {radius:.1} px — the \
         anchor never receded to the body edge (a value that also passes for t0 = 1 is not measuring \
         the fix)"
    );
}

/// **A hard falloff keeps the rim at the geometric rim — byte-identical to the pre-fix drawing.**
///
/// The fix is not "always recede the rim": a Constant brush (and any `hardness ≥ 1` disk) carries its
/// body right out to `t = 1`, so [`crate::height_push::rim_t0`] returns **exactly** `1.0` and the whole
/// bank is byte-for-byte the pre-2026-07-15 drawing. The [`crate::height_film::body_edge_t`] fast-path
/// guarantees that exact `1.0` — a bisection would land a hair below and shift the ridge sub-pixel.
///
/// **Mutations that must bleed:** (a) make `rim_t0` recede unconditionally (drop the hard-edge guard)
/// — `t0 ≠ 1.0` and the `assert_eq` fails; (b) let `body_edge_t` skip its `RIM_PROBE` fast-path and
/// bisect Constant to `0.99999…` — the fingerprint (bank at the resolved anchor vs at a hard-coded
/// `1.0`) diverges. Either way the "byte-identical" claim is refuted.
#[test]
fn a_hard_falloff_anchors_the_rim_at_the_geometric_rim() {
    use crate::height_push::rim_t0;
    let radius = 40.0f32;
    let spec = impasto_spec(crate::Falloff::Constant, radius);
    let t0 = rim_t0(&spec, false);
    assert_eq!(
        t0, 1.0,
        "a Constant falloff's body reaches the geometric rim → t0 must be exactly 1.0"
    );
    // Fingerprint: the bank at the resolved anchor is byte-for-byte the bank at a hard-coded t0 = 1.
    let resolved = isolated_bank_plane(&spec, t0);
    let forced = isolated_bank_plane(&spec, 1.0);
    assert!(
        resolved == forced,
        "Constant's bank drifted off the geometric-rim anchor — not byte-identical to the pre-fix rim"
    );
    // …and that anchor really is the geometric rim (not some other constant): inner edge ≈ radius.
    let inner = isolated_inner_edge(&resolved, radius);
    assert!(
        (inner - radius).abs() <= 2.0,
        "a Constant falloff's rim inner edge is {inner:.1} px, not at the geometric rim {radius:.1} px"
    );
}

/// Plough a straight run of `count` dabs spaced `step` px through a uniform 1-load ground with the
/// given falloff, and return the displacement plane. Same PATH (same start, same end) at any `step` —
/// only the SAMPLING of it changes, which is exactly what the trench must not depend on.
fn ploughed_at_spacing(falloff: crate::Falloff, step: f32, count: u32) -> (Vec<f32>, f32) {
    use crate::height::{HeightDab, HeightFields, accumulate_dab_height};
    use crate::height_push::{DEPOSIT_FORWARD_SHARE, PushBite, bank_dab_push, rim_t0, wave_lobe};
    const WD: u32 = 200;
    let n = (WD * WD) as usize;
    let radius = 12.0f32;
    let spec = crate::BrushSpec {
        radius_px: radius,
        impasto: true,
        impasto_depth: 0.5,
        impasto_push: 1.0,
        falloff,
        space_attenuation: false,
        ..Default::default()
    };
    let t0 = rim_t0(&spec, false);
    let ground = vec![1.0f32; n];
    let (mut plane, mut height, mut paint, mut grain, mut film, mut radp) = (
        vec![0.0f32; n],
        vec![0.0f32; n],
        vec![0.0f32; n],
        vec![0u8; n],
        vec![0u8; n],
        vec![0.0f32; n],
    );
    let mut scratch = Vec::new();
    let (y, x0) = (100.0f32, 60.0f32);
    let mut wave = 0.0f32;
    let mut last_tip: Option<HeightDab> = None;
    for k in 0..count {
        let cx = x0 + step * k as f32;
        let dab = HeightDab {
            center: [cx, y],
            radius,
            coverage: 1.0,
            footprint: spec.footprint_deform(),
            prev_center: (k > 0).then_some([cx - step, y]),
            shape: None,
            grain: None,
            grain_image: None,
        };
        if let (Some(tip), true) = (last_tip.take(), wave > 0.0) {
            let _ = wave_lobe(
                &mut plane,
                &paint,
                &mut scratch,
                WD,
                WD,
                &tip,
                t0,
                wave,
                -1.0,
            );
        }
        let mut fields = HeightFields {
            height: &mut height,
            paint: &mut paint,
            grain: &mut grain,
            film: &mut film,
            radius: &mut radp,

            accum: None,
        };
        let mut bite = PushBite {
            ground: &ground,
            plane: &mut plane,
            displaced: 0.0,
        };
        let _ = accumulate_dab_height(&mut fields, WD, WD, &spec, &dab, Some(&mut bite));
        let d = bite.displaced;
        let (_, carried) = bank_dab_push(
            &mut plane,
            &paint,
            &mut scratch,
            WD,
            WD,
            &dab,
            t0,
            d,
            DEPOSIT_FORWARD_SHARE,
        );
        wave += carried;
        if wave > 0.0
            && wave_lobe(
                &mut plane,
                &paint,
                &mut scratch,
                WD,
                WD,
                &dab,
                t0,
                wave,
                1.0,
            )
            .is_some()
        {
            last_tip = Some(HeightDab { ..dab });
        }
    }
    (plane, x0 + step * (count - 1) as f32)
}

/// The peak-to-peak ripple of the trench floor ALONG the stroke, sampled every texel over a window
/// well inside the run (away from the ends), at lateral offset `lat`.
fn trench_ripple(plane: &[f32], lat: usize, x_lo: usize, x_hi: usize) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for x in x_lo..x_hi {
        let v = plane[(100 + lat) * 200 + x];
        lo = lo.min(v);
        hi = hi.max(v);
    }
    hi - lo
}

/// **The trench is a fact of the PATH, not of the dab spacing** — and that is what killed the coil.
///
/// Enio's smoke of the anchor fix (2026-07-15): Smooth came out right, but `Sphere` grew a ribbed,
/// coiled artefact where the plough bites thick paint. The anchor was innocent (the same coil renders
/// with the old `t = 1` rim). The culprit was the BITE: `take = (g + p)·Δm` makes `q = g + p` evolve as
/// `q ← q·(1 − Δm)`, so the total is `g·(1 − Π(1 − Δm_k))` — a PRODUCT over the increments. A product
/// depends on how many steps the envelope was reached in and on each texel's PHASE against the dab
/// grid, so the trench floor ripples at exactly the dab period. `Smooth` hides it (small, even `Δm`);
/// `Sphere`'s silhouette has a VERTICAL tangent at the rim, so its `Δm` jumps and the phase term is
/// loud enough to read as a coil.
///
/// This is the same disease the capsule sweep cured for the DEPOSIT (*"the relief must be a property of
/// the brush and the PATH — never of how finely the engine happened to sample that path"*,
/// [`accumulate_dab_height`]) — the deposit is immune because it takes an ENVELOPE (a `max`, a pure
/// function of distance to the path); the bite was a sequential accumulation, so it was not.
///
/// Normalising the increment by the remaining headroom telescopes the product exactly
/// (`Π (1 − m_k)/(1 − m_{k−1}) = 1 − m_final`), so the bite lands on `g·m_final` at ANY spacing.
///
/// **Mutation that must bleed:** drop the `/ head` normalisation in [`accumulate_dab_height`]'s bite
/// (back to the raw `(m − paint[i])`). The Sphere ripple returns (measured ~10× this bound) and the two
/// spacings stop agreeing.
#[test]
fn the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing() {
    for falloff in [crate::Falloff::Smooth, crate::Falloff::Sphere] {
        // The SAME path, sampled two ways: 60 dabs of 1 px and 30 of 2 px both run 60 px.
        let (fine, end_a) = ploughed_at_spacing(falloff, 1.0, 61);
        let (coarse, end_b) = ploughed_at_spacing(falloff, 2.0, 31);
        assert_eq!(
            end_a, end_b,
            "fixture: the two samplings walk the same path"
        );
        // Deep inside the run, away from both ends, at the trench floor and on its flank.
        for lat in [0usize, 3] {
            let ripple = trench_ripple(&coarse, lat, 80, 110);
            assert!(
                ripple < 0.010,
                "{falloff:?} lat+{lat}: the trench floor ripples {ripple:.3} loads along the stroke \
                 — a per-dab corrugation the light reads as a coil (Enio, 2026-07-15). The bite must \
                 be a function of the envelope, not of the increment sequence"
            );
            for x in 80..110 {
                let (a, b) = (fine[(100 + lat) * 200 + x], coarse[(100 + lat) * 200 + x]);
                assert!(
                    (a - b).abs() < 0.02,
                    "{falloff:?} lat+{lat} x={x}: the trench is {a:.3} at 1 px spacing and {b:.3} at \
                     2 px — the artist's relief must not depend on how finely the engine sampled \
                     their stroke"
                );
            }
        }
    }
}

/// **The RIM is a fact of the path too — and the "residual" that said otherwise was a ramp.**
///
/// `CLAUDE.md` carried an open item for this: *"dar ao BANCO a cura que a mordida ganhou — cada dab ainda
/// normaliza o próprio aro = um produto sobre a lista de dabs; residual 0,0286"*. Measured, the item does
/// not exist, and the number came from where it was taken rather than from the bank:
///
/// ```text
///   Sphere lat+12   window 80..110  0,0894   |  MID 82..98  0,0180  |  END 100..118  0,1585
///   Smooth lat+9    window 80..110  0,0170   |  MID 82..98  0,0017  |  END 100..118  0,1084
/// ```
///
/// The old window straddles the END of the stroke, where the ridge ramps from full height down to
/// nothing — and `trench_ripple` is peak-to-peak, so across a ramp it reports the ramp. Mid-stroke the rim
/// is essentially uniform. And the giveaway that it was never a sampling artefact: halving the spacing
/// does not move it (**ratio 0,91–1,21×**, where a per-dab corrugation would roughly halve).
///
/// So the bank already has the property the "cure" was meant to buy, and this gate pins it where it is
/// honestly true — on the RIM, mid-stroke — rather than leaving a false open item for the next reader to
/// re-derive. (The sibling above owns the trench; this one owns the pile outside it.)
///
/// **Mutation that must bleed:** make the bank composite instead of accumulate in `bank_dab_push`
/// (`plane[i] = plane[i]*(1.0 - k) + k*scale` in place of `plane[i] += k*scale`). That IS the disease the
/// open item described — each dab re-weighting what the last one banked — and it makes the ridge a
/// function of how finely the path was sampled.
#[test]
fn the_rim_is_a_fact_of_the_path_not_of_the_dab_spacing() {
    /// Mid-stroke: far enough from the start for the pile to have built, far enough from the end that
    /// the ridge's own ramp-down is not in the window. The window IS the measurement here.
    const X_LO: usize = 82;
    const X_HI: usize = 98;
    for falloff in [crate::Falloff::Smooth, crate::Falloff::Sphere] {
        let (fine, _) = ploughed_at_spacing(falloff, 1.0, 61);
        let (coarse, _) = ploughed_at_spacing(falloff, 2.0, 31);
        // Outside the brush (radius 12 ⇒ the pile lives past it), where the banked ridge stands.
        for lat in [9usize, 12] {
            let mut h = 0.0f32;
            for x in X_LO..X_HI {
                h += coarse[(100 + lat) * 200 + x] / ((X_HI - X_LO) as f32);
            }
            assert!(
                h.abs() > 0.05,
                "fixture: lat+{lat} holds {h:+.4} loads — there is no pile here to measure, so a \
                 uniform one is uniformly nothing"
            );
            let ripple = trench_ripple(&coarse, lat, X_LO, X_HI);
            assert!(
                ripple < 0.030,
                "{falloff:?} lat+{lat}: the banked ridge ripples {ripple:.4} loads along a STRAIGHT \
                 stroke over uniform paint (measured 0,0180 worst). The light reads the height's \
                 gradient, so a corrugation at the dab period is a coil the artist sees."
            );
            for x in X_LO..X_HI {
                let (a, b) = (fine[(100 + lat) * 200 + x], coarse[(100 + lat) * 200 + x]);
                assert!(
                    (a - b).abs() < 0.030,
                    "{falloff:?} lat+{lat} x={x}: the ridge is {a:.4} at 1 px spacing and {b:.4} at \
                     2 px. The pile the artist gets must not depend on how finely the engine sampled \
                     their stroke — the same law the trench obeys one gate up."
                );
            }
        }
    }
}
