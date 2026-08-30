//! **O PASSE DE LUZ.** O relevo lido como levantado e não gravado, a sombra que escurece sem chegar ao
//! preto, o filme que a tinta forma na borda, o rig de lâmpadas (cada uma viva, a chave que não se
//! apaga, a que muda brilho sem mudar matiz) e o brilho que só soma luz.

use super::*;

#[test]
fn impasto_light_leaves_flat_paint_byte_identical() {
    // THE contract of the whole pass (T2.3, and stronger than the plan asked). The shading is RELATIVE:
    // a pixel's response is divided by a flat surface's response. So where there is no relief the pass
    // multiplies by exactly 1 and adds exactly 0.
    //
    // The naive `rgb × (N·L)` would fail this: a flat surface lit from 45° returns 0.707, so switching
    // the light on would darken the ENTIRE painting by 30%. That bug is in half the emboss filters ever
    // shipped, and this is the assertion that refuses it.
    let mut t = impasto_canvas(40);
    let mut b = t.paint.brush;
    b.impasto = false; // paint normally: pigment, no body
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));
    let unlit = lit(&mut t);

    // Now switch the light on, with the relief still empty. Not one byte may move.
    t.paint.impasto_show = true;
    t.invalidate_composite();
    let with_light = lit(&mut t);
    assert_eq!(
        unlit, with_light,
        "no relief ⇒ the light pass is a no-op, to the byte"
    );
}

#[test]
fn impasto_light_reads_as_raised_not_engraved() {
    // The APPEARANCE oracle ([[feedback_oracle_must_model_appearance_not_implementation]]): an oracle
    // derived from the shader would go green with the relief inverted on screen. So assert the thing a
    // human sees instead — a RIDGE lit from the left is BRIGHT on its left flank and DARK on its right.
    // Get the sign wrong (the classic emboss bug) and the paint reads as a groove carved INTO the
    // canvas; the arithmetic is just as self-consistent, and every shader-shaped oracle passes.
    let size = 60u32;
    let mut t = impasto_canvas(size);
    // A SOFT brush, deliberately: `impasto_canvas` paints with a hard disk, whose relief is a plateau
    // with vertical walls — h is identical at the centre and at both "flanks", so there is no gradient
    // to light and the test would have been asserting about nothing. (The sanity check below caught
    // exactly that on the first run.) A smooth falloff gives a real ridge with real flanks.
    let mut soft = t.paint.brush;
    soft.hardness = 0.0;
    soft.falloff = Falloff::Smooth;
    soft.radius_px = 10.0;
    t.paint.brush = soft;
    t.paint.brush_by_mode.fill(soft);
    t.paint.impasto_rig.lights[0].angle_deg = 180; // from the LEFT (-x)
    t.paint.impasto_rig.lights[0].elev_deg = 30;
    t.set_impasto_shine(0.0); // isolate the diffuse term — the highlight is a separate question
    // A vertical ridge of paint down the middle.
    t.on_canvas_pointer(cp([30.0, 10.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 50.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 50.0], PointerPhase::Up));

    let h = relief(&t);
    let img = lit(&mut t);
    let lum = |x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    // The flanks are FOUND from the relief itself — its steepest wall on each side of the crest. With
    // the body curve the interior is a PLATEAU, so a fixed offset (the old `25`/`35`) lands on flat
    // paint and asserts about nothing.
    let hx = |x: u32| h[(30 * size + x) as usize];
    assert!(hx(30) > 0.0, "the ridge is there");
    let steepest = |xs: std::ops::Range<u32>| {
        xs.max_by(|&a, &b| {
            let ga = (hx(a + 1) - hx(a - 1)).abs();
            let gb = (hx(b + 1) - hx(b - 1)).abs();
            ga.partial_cmp(&gb).unwrap()
        })
        .expect("a non-empty search band")
    };
    let (left_flank, right_flank) = (steepest(2..30), steepest(31..size - 2));
    assert!(
        hx(left_flank) < hx(30) && hx(right_flank) < hx(30),
        "the fixture really is a ridge (it falls away on both sides)"
    );
    let l = lum(left_flank, 30);
    let r = lum(right_flank, 30);

    // The reference is THE SAME PAINT WITH THE LIGHT OFF — not some other pixel. (My first attempt
    // used the canvas at x=2 as "flat": that is bare white paper, not flat paint, so the comparison was
    // meaningless and the assert failed for a reason that had nothing to do with the shading.)
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let base_img = lit(&mut t);
    let base = |x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(base_img[i]) + u32::from(base_img[i + 1]) + u32::from(base_img[i + 2])) as f32
            / 3.0
    };
    let (bl, br) = (base(left_flank, 30), base(right_flank, 30));
    t.paint.impasto_show = true;
    t.invalidate_composite();

    // THE appearance claim, stated the way an artist would: the flank turned TOWARD the light gets
    // brighter than the paint really is, and the flank turned AWAY gets darker. That is what "raised"
    // looks like. An implementation that merely darkened every edge would fail the first half; one with
    // the normal's sign flipped would fail both, and would look like a groove carved into the canvas.
    assert!(
        l > bl,
        "the flank facing the light is BRIGHTER than the paint under it ({l} vs {bl})"
    );
    assert!(
        r < br,
        "the flank turned away is DARKER than the paint under it ({r} vs {br})"
    );
    assert!(
        l > r,
        "so, lit from the left, the left flank beats the right ({l} vs {r})"
    );

    // Rotate the light 180° and the bright flank must SWAP. (A pass that merely darkened edges — any
    // edge, regardless of the light — would sail through the assertion above and die here.)
    t.paint.impasto_rig.lights[0].angle_deg = 0; // from the RIGHT (+x)
    t.invalidate_composite();
    let img = lit(&mut t);
    let lum2 = |x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    let (l2, r2) = (lum2(left_flank, 30), lum2(right_flank, 30));
    assert!(
        r2 > l2,
        "move the light to the RIGHT and the bright flank follows it ({l2} vs {r2})"
    );
}

#[test]
fn impasto_light_off_is_byte_identical_and_a_hidden_layer_casts_none() {
    // T2.3: `Show Impasto` off ⇒ the pass does not run ⇒ the composite is what it always was.
    // And the relief of a HIDDEN layer must go dark with it — otherwise the light keeps reporting a
    // ridge over paint that is no longer on screen.
    let mut t = impasto_canvas(40);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));

    t.paint.impasto_show = true;
    t.invalidate_composite();
    let shaded = lit(&mut t);

    t.paint.impasto_show = false;
    t.invalidate_composite();
    let plain = lit(&mut t);
    assert_ne!(shaded, plain, "the light pass is actually doing something");

    // Hide the layer that carries the relief: with the light back ON, the composite must match the
    // unlit one (no relief is visible, so none is lit).
    t.paint.impasto_show = true;
    let id = t.layers.active().expect("active layer");
    t.set_layer_visible(id, false);
    t.invalidate_composite();
    let hidden_lit = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let hidden_plain = lit(&mut t);
    assert_eq!(
        hidden_lit, hidden_plain,
        "a hidden layer's relief catches no light"
    );
}

#[test]
fn impasto_light_does_not_shade_paint_that_is_not_there() {
    // Enio's smoke (2026-07-12) showed a pale echo hugging each stroke where the eye saw no paint.
    // The light pass is a MULTIPLY, so it cannot tint bare white — but it CAN darken the brush's
    // near-invisible falloff tail: (255,248,248) × 0.75 = (191,186,186), a pink-grey halo over paint
    // nobody could see before. And the normal comes from the SLOPE, not the height: a film of paint one
    // thousandth deep, carrying a grain that swings per texel, has micro-slopes as steep as a real
    // ridge's — so it was shaded just as hard. Relief where there is no paint.
    //
    // The fix is the physical one (`BODY_MIN`): below a real body of paint, the pass fades to a no-op.
    // Measured: 53 offending pixels before, 0 after.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 200u32;
    // Isolate the light by toggling THE LIGHT (`impasto_show`), never the brush. Toggling `impasto`
    // was the original isolation and it stopped isolating anything the day the brush began to cut its
    // own pigment to the film it lays (`film_coverage`, Enio 2026-07-12): the two runs then differ by
    // the PIGMENT as well, and this gate was reading the paint the film removed as "the light shading
    // bare paper". Same brush, same pigment, light on vs off — that is the only clean seam.
    let paint = |show: bool| -> (Vec<u8>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 20.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain, // the per-texel grain is what makes the slopes steep
            impasto_smoothing: 0.15,
            jitter_spacing: 0.6, // the sweep grows with jitter — it must not overshoot onto bare canvas
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::ViewPlane;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        let path = [[100.0, 40.0], [110.0, 90.0], [100.0, 140.0], [110.0, 170.0]];
        t.on_canvas_pointer(cp(path[0], PointerPhase::Down));
        for p in &path[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        }
        t.on_canvas_pointer(cp(path[3], PointerPhase::Up));
        t.paint.impasto_show = show;
        t.invalidate_composite();
        let canvas = (*t.canvas_rgba).clone();
        let (comp, _, _) = t.take_preview_arc().expect("preview");
        (canvas, (*comp).clone())
    };
    let (canvas, unlit) = paint(false);
    let (_, litc) = paint(true);

    // BARE CANVAS — the paper the brush never touched at all. The bar was "≥ 96% white", and that is a
    // proxy that the Grain breaks: a deep grain VALLEY has its pigment scrubbed down to a few levels while
    // the body under it stands full (`DepthSource` decides whether the grain carves the relief; it always
    // textures the pigment). Such a pixel is thin PAINT, not paper, and the light is right to model it —
    // but the proxy filed it under "carries no paint" and then complained that the light had shaded it.
    // Invisible until `film_coverage` (Enio 2026-07-12) thinned those valleys by the few levels it took to
    // cross the bar; the shading itself is byte-identical to what it always was.
    //
    // The teeth this gate exists for are UNCHANGED and sharper for it: the height kernel sweeps a CAPSULE
    // back to the previous dab's centre, and if that sweep ever overshoots the paint it lays relief — and
    // shadow — on canvas the stroke never touched (26 px of it, the first time). That is bare paper: ink
    // exactly zero. `jitter_spacing` is here to stretch the sweep and hunt for it.
    // The film's screen-space AA (BUGS #16) adds one more case the green-channel proxy misfiles: a
    // RIM texel fractionally covered (~3%) whose pigment lands in a grain VALLEY quantizes to
    // nothing while the film (which excludes the grain by design) keeps a few levels — thin paint
    // again, adjacent to the stroke. True sweep overshoot lays relief on canvas the brush never went
    // NEAR (a 26 px swath, the first time), so the bare test also requires every 8-neighbour bare.
    let g_at = |i: usize| canvas[i + 1];
    let stride4 = 200usize * 4;
    let (mut faint, mut drifted, mut worst) = (0u32, 0u32, 0i32);
    for i in (0..canvas.len()).step_by(4) {
        if canvas[i + 1] != 255 {
            continue; // the brush left pigment here — however little. The light SHOULD model it.
        }
        let (x, y) = ((i / 4) % 200, (i / 4) / 200);
        if x == 0 || y == 0 || x == 199 || y == 199 {
            continue;
        }
        let near_paint = [
            i - 4,
            i + 4,
            i - stride4,
            i + stride4,
            i - stride4 - 4,
            i - stride4 + 4,
            i + stride4 - 4,
            i + stride4 + 4,
        ]
        .iter()
        .any(|&j| g_at(j) != 255);
        if near_paint {
            continue; // the stroke's own AA rim — thin paint, not paper
        }
        faint += 1;
        let d = (i32::from(litc[i + 1]) - i32::from(unlit[i + 1])).abs();
        if d > 8 {
            drifted += 1;
        }
        worst = worst.max(d);
    }
    assert!(
        faint > 10_000,
        "sanity: the fixture has a large unpainted field"
    );
    assert_eq!(
        drifted, 0,
        "the light pass shaded {drifted} pixels of BARE CANVAS (worst drift {worst} levels) — the \
         height sweep overshot the paint and laid relief where the brush never went"
    );
}

#[test]
fn impasto_shadowed_paint_is_dark_but_never_black() {
    // The black smudges on the stroke ENDS of Enio's smoke: a cap is where the height drops from full
    // to nothing across a pixel, so it is the steepest slope on the canvas — the first place a diffuse
    // term with a floor of ZERO bites. It multiplied the pixel straight to black. Paint in shadow is
    // dark; it is not a hole. `AMBIENT` is the floor, folded so a FLAT surface still returns exactly 1
    // (the byte-identity contract is untouched — `impasto_light_leaves_flat_paint_byte_identical`).
    let size = 60u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.impasto_depth = 1.0; // maximum relief ⇒ the steepest walls this brush can make
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Up));

    let h = relief(&t);
    let lum = |img: &[u8], x: u32, y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    t.paint.impasto_show = true;
    t.invalidate_composite();
    let shaded = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let plain = lit(&mut t);

    // The darkest LIT pixel that actually carries paint — measured against that same pixel unlit.
    let mut worst_ratio = f32::MAX;
    for y in 0..size {
        for x in 0..size {
            if h[(y * size + x) as usize].abs() < 0.05 {
                continue; // no body here — not what this gate is about
            }
            let base = lum(&plain, x, y).max(1.0);
            worst_ratio = worst_ratio.min(lum(&shaded, x, y) / base);
        }
    }
    assert!(
        worst_ratio < 1.0,
        "sanity: something on this stroke IS in shadow (else the gate proves nothing)"
    );
    assert!(
        worst_ratio > 0.25,
        "the deepest shadow on the paint crushed it to {:.0}% of its colour — paint in shadow is \
         dark, not a black hole",
        worst_ratio * 100.0
    );
}

#[test]
#[ignore = "diagnostic — run with --ignored --nocapture"]
fn halo_probe_translucent_edge() {
    // Enio 2026-07-12, white canvas vs black: a whitish halo rims the LIT flank on white and vanishes on
    // black. The light is a MULTIPLY on the composite — and at the stroke's translucent edge the
    // composite is mostly PAPER. So the pass is brightening the paper showing THROUGH the paint, and on
    // white paper `×1.65` bleaches a pale pink straight to white. Bucket the pixels by how much paint
    // they carry and see where the shift lands.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 200u32;
    let run = |paper: u8| -> (Vec<u8>, Vec<u8>, Vec<f32>) {
        let mut t = PainterTool::default();
        t.set_source(vec![paper; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_depth: 0.7,
            impasto_source: DepthSource::Grain,
            impasto_smoothing: 0.15,
            ..Default::default()
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([70.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([110.0, 100.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([80.0, 160.0], PointerPhase::Up));
        let canvas = (*t.canvas_rgba).clone();
        let h = relief(&t);
        t.paint.impasto_show = true;
        t.invalidate_composite();
        let lit_px = (*t.take_preview_arc().unwrap().0).clone();
        t.paint.impasto_show = false;
        t.invalidate_composite();
        let plain = (*t.take_preview_arc().unwrap().0).clone();
        let _ = canvas;
        (lit_px, plain, h)
    };
    for (name, paper) in [("WHITE paper", 255u8), ("BLACK paper", 0u8)] {
        let (litp, plain, h) = run(paper);
        // "Ink" = how far the pixel is from bare paper: 0 = untouched, 1 = fully covered.
        let core = plain
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| (i32::from(p[0]) - i32::from(p[1])).abs())
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let mut buckets = [(0u32, 0i32); 5]; // 0-20 / 20-40 / 40-60 / 60-80 / 80-100 % ink
        for i in (0..plain.len()).step_by(4) {
            let ink = (i32::from(plain[i]) - i32::from(plain[i + 1])).abs() as f32 / core;
            if ink <= 0.02 {
                continue;
            }
            let b = ((ink * 5.0) as usize).min(4);
            let shift = (i32::from(litp[i + 1]) - i32::from(plain[i + 1])).abs();
            buckets[b].0 += 1;
            buckets[b].1 = buckets[b].1.max(shift);
        }
        let hmax = h.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        println!("{name} (relief peak {hmax:.2}):");
        for (i, (n, worst)) in buckets.iter().enumerate() {
            println!(
                "   ink {:>3}-{:>3}%: {n:>6} px, light shifts up to {worst:>3} levels",
                i * 20,
                (i + 1) * 20
            );
        }
    }
}

#[test]
fn impasto_light_shades_the_paint_not_the_paper_showing_through_it() {
    // Enio, 2026-07-12, two photographs: the same strokes on a WHITE canvas and on a BLACK one. On
    // white, a bleached halo rimmed every stroke. On black it simply was not there — the tell. The pass
    // MULTIPLIES the composited pixel, and at a stroke's translucent edge that pixel is mostly PAPER
    // seen through the paint; shading it in full shades the paper, and on white that bleaches.
    //
    // The gate is stated as the property that is INDEFENSIBLE, and no more: **paint with no body gets
    // no light — not one byte — however the light is dialled.** Everything else the artist can judge
    // with their eyes; this they cannot, because a halo hides exactly where the paint is faintest.
    //
    // What this deliberately does NOT assert any more (it did, and it was wrong): that a lit edge keeps
    // its saturation. Under the artist's defaults (Depth 1, Body 0 — the relief follows the falloff all
    // the way out) a translucent edge DOES have relief, so the light legitimately brightens it; and
    // brightening a pixel whose pigment channel is already at the ceiling costs saturation, in paint as
    // in physics. Measured, it lands at 21% of the ink — and it is not a defect, it is the light. Pin
    // the paper instead: that line is absolute.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let size = 200u32;
    let render = |shine: f32, show: bool| -> (Vec<u8>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size); // WHITE paper: the hard case
        let mut b = BrushSpec {
            // radius 40 for the wide footprint this paper/paint metric was calibrated on; Depth 0.25 so
            // the size-scaling (×4 at this radius) lands back on the calibrated 1-load relief. Pinning the
            // radius to 10 instead shrank the footprint and concentrated the Grain slopes, which made the
            // ÷-coverage share metric unstable at the thinnest edge — the footprint has to stay big.
            radius_px: 40.0,
            impasto_depth: 0.25,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            impasto: true,
            impasto_source: DepthSource::Grain, // per-texel slopes: the harshest case for the weight
            ..Default::default()                // …and otherwise the ARTIST's defaults, on purpose
        };
        b.texture.kind = TextureKind::Noise;
        b.texture.mapping = TextureMapping::Tiled;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.set_impasto_shine(shine);
        t.on_canvas_pointer(cp([70.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([110.0, 100.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([80.0, 160.0], PointerPhase::Up));
        t.paint.impasto_show = show;
        t.invalidate_composite();
        let img = lit(&mut t);
        let active = t.layers.active().expect("a layer");
        let cov = t
            .covers
            .get(&active)
            .map(|c| c.as_ref().clone())
            .unwrap_or_default();
        (img, cov)
    };
    // The light at its LOUDEST — full Shine, the artist's Depth/Body/Angle/Elevation.
    let (loud, cov) = render(1.0, true);
    let (unlit, _) = render(1.0, false);

    // The statement, restated on the quantity the layers now hold (§15). `cov` used to be the RAW paint
    // (silhouette × dynamics) and the weight a curve over it, so "no body" could be spelled `cov < W_TAIL`
    // and the no-op below it was true BY CONSTRUCTION of the weight function — a tautology wearing a
    // gate's clothes. `cov` is now the SOLID PAINT itself (`solid_paint`: the body curve on the
    // silhouette, the dynamics multiplying afterwards), which is the light's weight, so the property has
    // to be said in two halves that are each independently falsifiable:
    //
    //   1. BARE PAPER — the pixel carries no pigment at all — is byte-identical, however the light is
    //      dialled. That line is absolute and it is the halo's actual door.
    //   2. TRANSLUCENT PAINT is touched only IN PROPORTION to the paint that is there. Measured: the
    //      worst the pass ever moves such a pixel is **half** the ink in it. That is what stops the pass
    //      from bleaching the paper *through* the paint, and — unlike the old threshold — it is a real
    //      measurement, not a restatement of the weight function.
    //
    // (MUT `paint_body(c) = 1.0`: bare paper lights up ⇒ half 1 RED. MUT `paint_body(c) = c.sqrt()`: the
    // thinnest paint takes 11× its share ⇒ half 2 RED. The old form caught neither on its own terms.)
    const MAX_SHARE: f32 = 0.75; // measured worst: 0.50 of the pixel's own ink
    let w_tail = ph2d_painter_brush::height::W_TAIL;
    let (mut bare, mut bare_drift, mut worst_share, mut worst_c) = (0u32, 0i32, 0.0f32, 0.0f32);
    for p in 0..(size * size) as usize {
        let c = f32::from(cov[p]) / 255.0;
        let d = (0..3)
            .map(|k| (i32::from(loud[p * 4 + k]) - i32::from(unlit[p * 4 + k])).abs())
            .max()
            .unwrap_or(0);
        if c == 0.0 {
            bare += 1;
            bare_drift = bare_drift.max(d);
        } else if c < w_tail {
            let share = d as f32 / (255.0 * c);
            if share > worst_share {
                worst_share = share;
                worst_c = c;
            }
        }
    }
    assert!(
        bare > 30_000,
        "sanity: most of this canvas is bare paper ({bare} px)"
    );
    assert_eq!(
        bare_drift, 0,
        "the light moved BARE PAPER by {bare_drift} levels — the pass must be a strict no-op where there \
         is no pigment at all. That is the white halo: it vanished on Enio's black canvas because there \
         was nothing white to bleach, which is how we know it is the paper and not the pigment."
    );
    assert!(
        worst_share <= MAX_SHARE,
        "the light moved a translucent pixel by {:.0}% of its own ink (at coverage {worst_c:.3}) — it may \
         only touch the paint that is actually there, never the paper showing through it",
        worst_share * 100.0
    );
}

#[test]
fn impasto_soft_stroke_reads_as_a_body_with_an_edge() {
    // THE appearance gate of the Fase 4 redesign (plan §10, T4.5) — derived from the DEFINITION of
    // thick paint, not from the shader ([[feedback_oracle_must_model_appearance_not_implementation]]):
    // a body of paint has a level top, a wall at its edge, and a stain that carries no relief. The
    // DEFAULT brush (hardness 0, Smooth — the one Enio actually smokes with) must read that way.
    //
    // Every threshold here was RED under the dome kernel, by the opening measurements (plan §10):
    // the dome curved everywhere (no plateau, tail relief 0.07, shading smeared over 62% of the
    // stroke's width with its weak peak — 7.3 levels — at 31%, nothing at the edge).
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.radius_px = 40.0;
    b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
    b.impasto_body = 1.0; // this gate IS the body curve (the artist's default is the round profile)
    b.impasto_source = DepthSource::Uniform; // isolate the body curve — grain is another gate
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.paint.impasto_rig.lights[0].angle_deg = 90; // straight across a horizontal stroke
    t.paint.impasto_rig.lights[0].elev_deg = 45;
    t.set_impasto_shine(0.0); // the diffuse modelling is the claim; the glint has its own gates
    t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
    for i in 1..=8 {
        t.on_canvas_pointer(cp(
            [40.0 + 10.0 * f32::from(i as u8), 80.0],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));

    let h = relief(&t);
    let img = lit(&mut t);
    t.paint.impasto_show = false;
    t.invalidate_composite();
    let base = lit(&mut t);
    let active = t.layers.active().unwrap();
    let cov = t
        .covers
        .get(&active)
        .map(|c| c.as_ref().clone())
        .unwrap_or_default();

    // Cross-section at mid-stroke, from the spine outward.
    let x = 80u32;
    let lum = |img: &[u8], y: u32| {
        let i = ((y * size + x) * 4) as usize;
        (u32::from(img[i]) + u32::from(img[i + 1]) + u32::from(img[i + 2])) as f32 / 3.0
    };
    let rows: Vec<(u32, u8, f32, f32)> = (80..160u32)
        .map(|y| {
            let i = (y * size + x) as usize;
            (y - 80, cov[i], h[i], lum(&img, y) - lum(&base, y))
        })
        .collect();
    let painted: Vec<&(u32, u8, f32, f32)> = rows.iter().filter(|r| r.1 > 4).collect();
    let half_width = painted.last().expect("a painted cross-section").0;
    let spine_h = rows[0].2;
    assert!(spine_h > 0.6, "sanity: the stroke laid its full depth");

    // 1. The top is a PLATEAU: at 25% of the half-width the paint is as thick as at the spine.
    //    (Dome: ~0.86 of the spine there — curved from the very centre.)
    let at = |frac: f32| {
        let d = (frac * half_width as f32) as u32;
        rows.iter().find(|r| r.0 == d).expect("inside the canvas")
    };
    assert!(
        at(0.25).2 >= 0.98 * spine_h,
        "the interior is a level film, not a dome (h {} at 25% vs spine {spine_h})",
        at(0.25).2
    );

    // 2. The relief ENDS WITH THE PAINT: past the film's own edge there is no body left standing.
    //
    //    It used to say "past 85% of the painted half-width the relief is zero", and that measured the
    //    stain — a band of pigment WITHOUT body, which is exactly what the brush no longer lays (§14:
    //    `film_coverage`; and `cov` is now the solid paint itself, so the painted half-width IS the
    //    body's). The claim survives, sharper, from the other side: step OUT past the paint and there
    //    must be nothing — no body, no pigment. (Dome kernel: 0.065 of relief out there, standing over
    //    near-invisible paint — the halo's raw material. Still RED.)
    let out = |frac: f32| {
        let d = (frac * half_width as f32).round() as u32;
        rows.iter().find(|r| r.0 == d).expect("inside the canvas")
    };
    for frac in [1.10f32, 1.25, 1.40] {
        let r = out(frac);
        assert!(
            r.2 == 0.0 && r.1 == 0,
            "past the paint's edge the canvas is BARE: at {frac}x the half-width the relief is {} and \
             the coverage {}",
            r.2,
            r.1
        );
    }

    // 3. The light lives on the WALL: the response is concentrated, and its peak is a real edge, not a
    //    haze (≥ 8 levels; the dome managed 7.3 with everything on).
    //
    //    The bar moved 40% → 55%, and the DENOMINATOR is why: `painted` used to run out to the stain's
    //    last visible pixel, and the stain is exactly what the brush stopped laying (§14). Measured over
    //    the film, the wall's share is its own geometry — for this brush it spans `t ∈ [0.35, 0.61]` of a
    //    film that ends at `t = 0.61`, i.e. **43%**, and 43% is what it measures. The dome, over the same
    //    denominator, smears **84%** (MUT `body_profile(w) = w`, with assertions 1–2 silenced so this one
    //    could speak). The bar sits between them, nearer the geometry than the slack.
    let visible = painted.iter().filter(|r| r.3.abs() >= 3.0).count();
    let concentration = visible as f32 / painted.len().max(1) as f32;
    assert!(
        concentration <= 0.55,
        "the shading is concentrated at the edge, not smeared over the stroke ({:.0}% of the width)",
        concentration * 100.0
    );
    let peak = rows
        .iter()
        .max_by(|a, b| a.3.abs().partial_cmp(&b.3.abs()).unwrap())
        .unwrap();
    assert!(
        peak.3.abs() >= 8.0,
        "the wall actually catches the light ({:.1} levels)",
        peak.3.abs()
    );
    // …and the peak sits ON the wall (where the height is falling), not on the plateau.
    let peak_h = rows.iter().find(|r| r.0 == peak.0).unwrap().2;
    assert!(
        peak_h < 0.95 * spine_h && peak_h > 0.0,
        "the brightest response is on the wall itself (h {peak_h} at the peak, spine {spine_h})"
    );
}

#[test]
fn impasto_shine_glints_on_the_wall_without_bleaching_the_rim() {
    // Enio, 2026-07-12: "shine não funciona." He was right, and the cause was geometric: the relief's
    // slope exists ONLY over the coverage band `W_TAIL..W_SOLID` (that IS the wall), while the glint
    // had been gated ABOVE `W_SOLID` — i.e. allowed only on the plateau, which is flat, where the pass
    // early-outs. Measured: 94% of the sloped pixels sat below the gate, and Shine 0 → 1 moved the
    // brightest pixel by ONE level. A knob that does nothing.
    //
    // This gate pins BOTH halves, because fixing either one alone is how it broke: the glint must be
    // VISIBLE (it was not) and it must not BLEACH the translucent rim (the white halo of the first
    // photograph — which came back the moment the glint was let onto the wall as a flat `+ add`, since
    // on a rim pixel the red channel is already at the ceiling and only the other channels move).
    let size = 160u32;
    let paint_with = |shine: f32| -> (Vec<u8>, Vec<f32>, Vec<u8>) {
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
        // RED paint on white paper: the rim's "ink" is measured as `R − G`, so the canvas fixture's own
        // dark blue would read as zero ink and the bleach half of this gate would be vacuous. (It said
        // so out loud on the first run — which is the anti-vacuity clause earning its keep.)
        b.color = [0.9, 0.1, 0.1];
        // And the SMOKE's own arming — a grain-sourced brush over noise. Not decoration: with a plain
        // Uniform brush the highlight never reaches the translucent rim at all, so the bleach half of
        // this gate passed even with a flat additive highlight (proved by mutation). The grain carves
        // crests everywhere, including out on the thin paint, which is precisely the condition that
        // photographed as a halo.
        b.impasto_source = DepthSource::Grain;
        b.impasto_smoothing = 0.15;
        b.texture.kind = ph2d_painter_brush::TextureKind::Noise;
        b.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.paint.impasto_rig.lights[0].angle_deg = 90;
        t.paint.impasto_rig.lights[0].elev_deg = 45;
        t.set_impasto_shine(shine);
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
        let img = lit(&mut t);
        let h = relief(&t);
        let active = t.layers.active().expect("a layer");
        let cov = t
            .covers
            .get(&active)
            .map(|c| c.as_ref().clone())
            .unwrap_or_default();
        (img, h, cov)
    };
    let (matte, h, cov) = paint_with(0.0);
    let (glossy, _, _) = paint_with(1.0);

    // 1. The glint is VISIBLE — on SOLID paint (the film AA's fractional rim texels scale the light
    // by their coverage, so a rim glint is legitimately dimmer; the claim is about the wall's body).
    let (mut best, mut best_i) = (0i32, 0usize);
    for i in (0..matte.len()).step_by(4) {
        if cov.get(i / 4).copied().unwrap_or(0) != 255 {
            continue;
        }
        let gain = i32::from(glossy[i + 1]) - i32::from(matte[i + 1]); // green: the pigment is red
        if gain > best {
            best = gain;
            best_i = i / 4;
        }
    }
    assert!(
        best >= 40,
        "Shine must actually light the paint (brightest gain {best} levels)"
    );

    // 2. It lands on the WALL — sloped paint with a real body — not on the flat plateau or the stain.
    let px = |i: usize| (i % size as usize, i / size as usize);
    let (bx, by) = px(best_i);
    let gx = (h[best_i + 1] - h[best_i - 1]).abs();
    let gy = (h[best_i + size as usize] - h[best_i - size as usize]).abs();
    assert!(
        gx.max(gy) > 0.005,
        "the brightest glint sits on SLOPED paint (grad {gx:.4}/{gy:.4} at {bx},{by})"
    );
    assert!(
        f32::from(cov[best_i]) / 255.0 > 0.4,
        "…and on paint with a body, not on the translucent stain (coverage {})",
        f32::from(cov[best_i]) / 255.0
    );

    // 3. And at FULL Shine the pass is still a STRICT NO-OP on the translucent stain — the paint too
    //    thin to have a body (`cover < W_TAIL`). That is the halo's actual door, and it is now nailed
    //    shut by construction: no body ⇒ no relief AND no lighting weight, so those pixels come out
    //    byte-identical no matter how the light is dialled.
    //
    //    What this deliberately does NOT claim: that a highlight never washes a *lit wall* toward
    //    white. It does — that is what a highlight is (the worst "bleached" pixel under an earlier,
    //    stricter version of this assertion turned out to be paint at 70% coverage whose red channel
    //    the DIFFUSE had already driven to 255; the chroma metric could not tell an honest glint from
    //    the halo). The default look is guarded instead by
    //    `impasto_light_shades_the_paint_not_the_paper_showing_through_it`, which is the gate that
    //    catches a flat additive highlight (proved: it goes red at 19% survival).
    let w_tail = ph2d_painter_brush::height::W_TAIL;
    let unlit = {
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
        b.color = [0.9, 0.1, 0.1];
        b.impasto_source = DepthSource::Grain;
        b.impasto_smoothing = 0.15;
        b.texture.kind = ph2d_painter_brush::TextureKind::Noise;
        b.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.paint.impasto_show = false; // the light pass does not run at all
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
        lit(&mut t)
    };
    let _ = &h;
    // Restated on the quantity the layers now hold (§15): `cov` is the SOLID PAINT — the light's weight
    // itself — so "the pass is a no-op on the stain" is no longer a threshold to check but a
    // PROPORTION: the highlight may only touch the paint that is actually there. Bare paper takes
    // nothing at all; a translucent pixel takes at most its own share. (The stain CAN carry a little
    // height — `Smoothing` settles the paint and the blur spreads it past the body's edge, which is what
    // settling paint does. What must not happen is the light bleaching the paper *through* it.)
    const MAX_SHARE: f32 = 0.75; // measured worst: 0.50 of the pixel's own ink
    let (mut stain_px, mut bare_drift, mut worst_share) = (0u32, 0i32, 0.0f32);
    for p in 0..(size * size) as usize {
        let c = f32::from(cov[p]) / 255.0;
        let d = (0..3)
            .map(|ch| (i32::from(glossy[p * 4 + ch]) - i32::from(unlit[p * 4 + ch])).abs())
            .max()
            .unwrap_or(0);
        if c == 0.0 {
            bare_drift = bare_drift.max(d);
        } else if c < w_tail {
            stain_px += 1;
            worst_share = worst_share.max(d as f32 / (255.0 * c));
        }
    }
    assert!(
        stain_px > 300,
        "sanity: the fixture HAS a translucent stain ({stain_px} px) — else this claim is vacuous"
    );
    assert_eq!(
        bare_drift, 0,
        "at full Shine the highlight moved BARE PAPER by {bare_drift} levels — the halo's door"
    );
    assert!(
        worst_share <= MAX_SHARE,
        "at full Shine the highlight moved a translucent pixel by {:.0}% of its own ink — the light must \
         not touch paint too thin to have a body",
        worst_share * 100.0
    );
}

/// **The film**: a brush that lays BODY lays no pigment where the light lays no shading.
///
/// Enio, 2026-07-12, two screenshots of the same crossing strokes: *"o efeito leva em consideração os
/// limites do pincel e não o peso do relevo. Este falloff (smooth) pinta tinta fora do relevo. Usando o
/// falloff Sphere fica mais preciso e a tinta corresponde ao relevo."*
///
/// Two things already agreed on where paint stops carrying a body — the relief (`body_profile`, zero
/// below `W_TAIL`) and the light (it weighs its shading by the same curve, so it will not bleach the
/// paper showing through a translucent rim). The PIGMENT knew nothing about it and ran all the way out
/// to the dab's geometric rim, so every impasto stroke wore a skirt of paint the light was RIGHT to
/// refuse to model. `ph2d_painter_brush::height::film_coverage` cuts it.
///
/// Stated as the property, not as a number: **every pixel the brush pigments, the light models.** Run on
/// BOTH falloffs — Smooth (where the skirt was 39% of the radius) and Sphere (6%, which is the whole
/// reason Sphere "looked more precise") — because the rule must not depend on the falloff. That is what
/// makes it a rule and not a preset.
#[test]
fn impasto_lays_no_pigment_where_the_light_lays_no_shading() {
    let size = 200u32;
    // Paint one stroke and return (canvas, cover) — the pigment that landed and the paint the light sees.
    let stroke = |falloff: Falloff, strength: f32| -> (Vec<u8>, Vec<u8>) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size); // WHITE paper
        let b = BrushSpec {
            radius_px: 40.0,
            color: [0.9, 0.1, 0.1],
            falloff,
            strength, // < 1 ⇒ Accumulate OFF: a DIFFERENT alpha funnel (build-toward-a-cap), same rule
            space_attenuation: false,
            impasto: true,
            ..Default::default() // …and otherwise the ARTIST's defaults (Depth 1, Body 0), on purpose
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([70.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([110.0, 100.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([80.0, 160.0], PointerPhase::Up));
        let active = t.layers.active().expect("a layer");
        let cover = t
            .covers
            .get(&active)
            .map(|c| c.as_ref().clone())
            .unwrap_or_default();
        (t.canvas_rgba.to_vec(), cover)
    };

    // At every FALLOFF and every STRENGTH. The Strength axis is the whole point of the second pass
    // (§15): the light used to weigh by `body_profile(cover)` over a `cover` that held the RAW paint —
    // dynamics INSIDE the body curve, where they starve it. Below Flow × Strength × pressure ≈ `W_TAIL`
    // the argument fell under the tail for every texel and the light modelled NOTHING anywhere on the
    // stroke, while the pigment was still right there: Enio's haze, hiding behind the mouse (which always
    // presses at 1.0). The layers now store the SOLID PAINT itself, so the threshold sits on the
    // silhouette and the dynamics multiply afterwards — and the rule holds at any pressure.
    for (falloff, strength) in [
        (Falloff::Smooth, 1.0f32), // the default brush — where Enio saw the haze
        (Falloff::Sphere, 1.0),    // his workaround: the rule must not depend on the falloff
        (Falloff::Smooth, 0.5),    // …nor on how hard you press. RED before §15.
        (Falloff::Smooth, 0.3), // Accumulate-OFF territory, and under the old W_TAIL on the dynamics
    ] {
        let (canvas, cover) = stroke(falloff, strength);
        let falloff = format!("{falloff:?} @ strength {strength}");
        assert_eq!(
            cover.len(),
            (size * size) as usize,
            "{falloff}: a cover map"
        );
        let (mut pigmented, mut orphan, mut worst) = (0u32, 0u32, 0.0f32);
        for p in 0..(size * size) as usize {
            // Did the brush leave pigment here? (White paper: any channel below 255 is paint.)
            let ink = (0..3).map(|c| 255 - canvas[p * 4 + c]).max().unwrap_or(0);
            if ink == 0 {
                continue;
            }
            pigmented += 1;
            // Then the light MUST model it. `cover` IS the light's weight now — the SOLID PAINT the
            // brush laid (`solid_paint`) — and a zero there is the light declaring this pixel bodyless:
            // paper showing through a stain.
            let weight = f32::from(cover[p]) / 255.0;
            if weight <= 0.0 {
                orphan += 1;
                worst = worst.max(f32::from(ink) / 255.0);
            }
        }
        assert!(
            pigmented > 500,
            "{falloff}: the stroke must actually paint ({pigmented} px)"
        );
        assert_eq!(
            orphan,
            0,
            "{falloff}: {orphan} px of pigment outside the relief the light will model \
             (worst {:.0}% ink) — the haze Enio photographed",
            worst * 100.0
        );
    }
}

/// …and the film binds ONLY a brush that lays body: an ordinary brush is byte-identical.
///
/// The other half of the rule, and the one that would rot silently. `film_coverage` sits in the alpha
/// funnel of EVERY deposit path (cached blit, per-pixel, canvas-cached, ramped, per-layer colour), so a
/// leak there would re-cut every brush in the app — hardening the edge of a soft airbrush that never
/// asked for a body. The gate: the same stroke with Impasto off, and with Impasto on but `DrawTo::Color`
/// (pigment, no thickness — a glaze over relief), must both come out exactly as they did before the film
/// existed. Compared against a THIRD run whose brush is plainly non-impasto: byte for byte.
#[test]
fn the_film_binds_only_a_brush_that_lays_body() {
    let size = 120u32;
    let stroke = |mutate: &dyn Fn(&mut BrushSpec)| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let mut b = BrushSpec {
            radius_px: 30.0,
            color: [0.9, 0.1, 0.1],
            space_attenuation: false,
            ..Default::default()
        };
        mutate(&mut b);
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([80.0, 90.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 90.0], PointerPhase::Up));
        t.canvas_rgba.to_vec()
    };
    let plain = stroke(&|_b| {});
    assert_eq!(
        stroke(&|b| b.impasto = false),
        plain,
        "Impasto OFF: the film must not touch an ordinary brush"
    );
    assert_eq!(
        stroke(&|b| {
            b.impasto = true;
            b.impasto_draw_to = DrawTo::Color; // pigment, no thickness — a glaze
        }),
        plain,
        "DrawTo::Color: a brush that lays no body lays its pigment untouched"
    );
    assert_eq!(
        stroke(&|b| {
            b.impasto = true;
            b.impasto_depth = 0.0; // no depth ⇒ no body ⇒ no film
        }),
        plain,
        "Depth 0: no body, no cut"
    );
    // …and the film DOES bite when the brush lays body — else the three assertions above are vacuous.
    assert_ne!(
        stroke(&|b| b.impasto = true),
        plain,
        "a body-laying brush must lay a DIFFERENT (cut) pigment — otherwise this gate proves nothing"
    );
}

/// The film may never STARVE the brush: an impasto stroke at low Strength still paints.
///
/// The first cut of `film_coverage` ran `body_profile` on the dab's FULL coverage — silhouette × grain ×
/// (pressure × Flow × Strength). At Strength 0.5 the dab's peak coverage is 0.25, which is under `W_TAIL`,
/// so the curve returned zero for every texel and **the stroke deposited nothing at all**. A brush that
/// paints nothing is not a fix; it is a worse bug than the one being fixed, and it would have shipped.
///
/// The rule it teaches: **a film's edge is a property of the TIP, not of how hard you press.** A light
/// touch of a loaded brush lays a thinner film, not a film with a different outline — so the curve runs on
/// the tip and the dynamics scale the result. MUT (revert to `body_profile(tip * dynamics)`): RED here.
#[test]
fn the_film_never_starves_the_brush_at_low_strength() {
    let size = 120u32;
    let ink = |strength: f32| -> u32 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 30.0,
            color: [0.9, 0.1, 0.1],
            strength,
            space_attenuation: false,
            impasto: true,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([80.0, 90.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 90.0], PointerPhase::Up));
        (0..(size * size) as usize)
            .filter(|p| (0..3).any(|c| t.canvas_rgba[p * 4 + c] != 255))
            .count() as u32
    };
    let full = ink(1.0);
    assert!(
        full > 500,
        "sanity: the full-strength stroke paints ({full} px)"
    );
    for strength in [0.5f32, 0.3, 0.15] {
        let faint = ink(strength);
        assert!(
            faint > full / 2,
            "Strength {strength}: an impasto brush laid {faint} px against {full} at full — the film cut \
             the DYNAMICS instead of the tip, and starved the brush"
        );
    }
}

/// The light models a **faint** stroke: a light touch lays a thinner film, not one the light refuses to see.
///
/// The hole §14.6 named and did not close. The pass weighed its shading by `body_profile(cover)` over a
/// `cover` that held the RAW paint — `silhouette × dynamics` — so the dynamics sat INSIDE the body curve,
/// where they could starve it: at Flow × Strength × pressure below `W_TAIL` the argument falls under the
/// tail for **every texel** and the light models nothing anywhere on the stroke. The pigment, cut on the
/// silhouette (`film_coverage`), is still perfectly there. That is Enio's haze — pigment with no visible
/// body — surviving at partial pressure, and it hid behind the mouse, which always presses at 1.0.
///
/// The fix is the film's own theorem applied to the other side: **the threshold belongs to the silhouette;
/// the dynamics multiply afterwards** (`solid_paint`). At full dynamics the two spellings are the same
/// number, so nothing a mouse ever drew moved.
///
/// MUT (`solid_paint(sil, dyn) = body_profile(sil * dyn)` — the dynamics back inside the curve): RED,
/// the faint stroke goes completely unlit.
#[test]
fn the_light_models_a_faint_stroke() {
    let size = 160u32;
    // The shading a stroke at `strength` produces: the biggest level the light moves any pixel.
    let shading = |strength: f32| -> i32 {
        let render = |show: bool| -> Vec<u8> {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
            let b = BrushSpec {
                radius_px: 30.0,
                color: [0.9, 0.1, 0.1],
                strength,
                space_attenuation: false,
                impasto: true,
                ..Default::default()
            };
            t.paint.brush = b;
            t.paint.brush_by_mode.fill(b);
            t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
            t.on_canvas_pointer(cp([120.0, 90.0], PointerPhase::Move));
            t.on_canvas_pointer(cp([40.0, 130.0], PointerPhase::Up));
            t.paint.impasto_show = show;
            t.invalidate_composite();
            lit(&mut t)
        };
        let (lit_img, flat) = (render(true), render(false));
        let mut worst = 0i32;
        for p in 0..(size * size) as usize {
            for c in 0..3 {
                let d = (i32::from(lit_img[p * 4 + c]) - i32::from(flat[p * 4 + c])).abs();
                worst = worst.max(d);
            }
        }
        worst
    };
    let full = shading(1.0);
    assert!(
        full > 20,
        "sanity: a full-strength stroke is modelled ({full} levels)"
    );
    // A light touch is THINNER paint, and Strength scales the THICKNESS as well as the opacity — so a
    // 30% stroke is a film 30% as tall AND 30% as opaque, and the two multiply. It is modelled an order
    // of magnitude more softly, and that is the physics, not the bug: thin paint catches little light.
    //
    // The property is that the light does not go DARK, and the bar is stated there and nowhere else —
    // inflating it to "must move ≥ N levels" would be inventing a look the paint does not have. Measured:
    // **149 / 30 / 2** levels at Strength 1.0 / 0.5 / 0.3. Under the MUT (`solid_paint` spelled
    // `body_profile(sil * dynamics)` — the dynamics back inside the curve) Strength 0.5 goes to exactly
    // **0**: the pass turns black, which is what Enio would have hit the first time he picked up the pen.
    //
    // Below ~0.25 the response falls under a LEVEL and rounds to zero, and that is arithmetic, not a
    // cliff: Strength scales the film's thickness AND its opacity, so a 20% stroke is 20% as tall and 20%
    // as opaque, and 4% of a shading is less than 1/255. Thin paint has no visible relief. Asserting
    // otherwise would be asserting a look the paint does not have.
    let mut prev = full;
    for strength in [0.5f32, 0.3] {
        let faint = shading(strength);
        assert!(
            faint > 0,
            "Strength {strength}: the light moved {faint} levels — a faint stroke is thinner paint, not \
             paint the light refuses to SEE (full strength moves {full})"
        );
        assert!(
            faint <= prev,
            "…and it tracks the paint: Strength {strength} may not be modelled harder ({faint}) than the \
             heavier stroke above it ({prev})"
        );
        prev = faint;
    }
}

/// The paint has an **EDGE**, not a fringe: a stroke of thick paint is opaque right up to where its body
/// ends, and then it stops.
///
/// Enio, 2026-07-12, third smoke: *"regrediu ao deixar a tinta extravasar o relevo e não resolveu a
/// distância da tinta levantada."* The film (§14) cut the pigment at the body's edge and the supports
/// matched exactly — `impasto_lays_no_pigment_where_the_light_lays_no_shading` said so, and it was right,
/// and it was **too weak to see what he saw**: the alpha *ramped* with the body, so the stroke ended in a
/// soft gradient of pale red carrying no 3D form. A soft gradient with no form IS a haze. The support was
/// identical and the picture was still wrong — a set-equality gate cannot tell a wall from a fog bank.
///
/// The physics that was wrong: **opacity is not thickness.** A film's opacity saturates long before its
/// thickness does (Beer–Lambert) — oil paint at a tenth of full thickness is already all but opaque,
/// which is why a palette knife leaves an edge and not a gradient. Modelling alpha as proportional to
/// body was modelling paint as glass (`film_opacity`).
///
/// So the property is stated where the eye reads it — as **area**, not as a threshold. Of all the paint a
/// stroke lays, how much is neither solid nor absent? Measured on the smoke's own brush:
///
/// | | opaque | translucent | haze |
/// |---|---|---|---|
/// | no film at all (the original bug) | 6122 | 6620 | **52%** |
/// | film ∝ thickness (the first cut) | 5108 | 2036 | **28.5%** |
/// | film with opacity (today) | 6396 | 1000 | **13.5%** |
///
/// …and the OPAQUE area grows as the haze falls: the paint did not shrink, it went solid.
#[test]
fn impasto_paint_has_an_edge_not_a_fringe() {
    const MAX_HAZE: f32 = 0.18; // measured: 0.135. Proportional-alpha: 0.285. No film: 0.52.
    let size = 240u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    // The SMOKE's arming, verbatim: the defaults + a size + Impasto on. A bad default has nowhere to hide.
    let b = BrushSpec {
        radius_px: 40.0,
        color: [0.9, 0.1, 0.1],
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([60.0, 120.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([180.0, 120.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([180.0, 120.0], PointerPhase::Up));

    let (mut opaque, mut translucent) = (0u32, 0u32);
    for p in 0..(size * size) as usize {
        let ink = 255i32 - i32::from(t.canvas_rgba[p * 4 + 1]); // green channel: red paint on white
        if ink >= 200 {
            opaque += 1;
        } else if ink >= 10 {
            translucent += 1; // paint the eye can see, that is not paint the eye can trust
        }
    }
    assert!(
        opaque > 3_000,
        "sanity: the stroke laid solid paint ({opaque} px)"
    );
    let haze = translucent as f32 / (opaque + translucent) as f32;
    assert!(
        haze <= MAX_HAZE,
        "{:.0}% of this stroke's paint is neither solid nor absent ({translucent} px against {opaque} \
         opaque) — a soft gradient carrying no form is the haze Enio photographed, whatever the supports \
         agree about",
        haze * 100.0
    );
}

/// The rig's contract: **flat paint stays byte-identical — even under coloured light.**
///
/// The shading is RELATIVE (a pixel's response divided by a FLAT surface's), and the rig keeps that per
/// CHANNEL: on flat paint `N·Lᵢ = Lᵢ.z` for every lamp, so `diffuse[c]/flat[c] = 1` in every channel
/// whatever the colours and intensities are. So a warm key + a cool fill tint the paint exactly where it
/// TILTS, and **a flat painting under a red lamp does not turn red**.
///
/// That is not a nicety. An absolute model would let a coloured lamp wash a colour over the whole canvas
/// — the light would stop being a property of the RELIEF and become a filter over the picture, and every
/// pixel of flat paint the artist mixed by eye would shift under it.
///
/// Run on a canvas with NO relief at all, and again with relief but Show off. RED under an absolute
/// model (`ratio = diffuse` with no divisor) and under a per-rig (rather than per-channel) divisor.
#[test]
fn a_coloured_light_rig_leaves_flat_paint_byte_identical() {
    let size = 120u32;
    let render = |arm: &dyn Fn(&mut PainterTool)| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        // FLAT paint: impasto OFF, so the stroke lays pigment and no body at all.
        let b = BrushSpec {
            radius_px: 25.0,
            color: [0.3, 0.6, 0.45],
            impasto: false,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([30.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([90.0, 80.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([90.0, 80.0], PointerPhase::Up));
        arm(&mut t);
        t.invalidate_composite();
        lit(&mut t)
    };
    let one_white_lamp = render(&|_t| {});
    // A four-lamp rig of saturated, wildly unbalanced colours — the worst case an artist could build.
    let loud_rig = render(&|t| {
        let r = &mut t.paint.impasto_rig;
        r.lights[0] = ImpastoLight {
            on: true,
            angle_deg: 20,
            elev_deg: 70,
            intensity: 1.8,
            color: [1.0, 0.1, 0.1],
        };
        r.lights[1] = ImpastoLight {
            on: true,
            angle_deg: 140,
            elev_deg: 15,
            intensity: 0.9,
            color: [0.1, 1.0, 0.2],
        };
        r.lights[2] = ImpastoLight {
            on: true,
            angle_deg: 260,
            elev_deg: 45,
            intensity: 1.3,
            color: [0.15, 0.2, 1.0],
        };
        r.lights[3] = ImpastoLight {
            on: true,
            angle_deg: 350,
            elev_deg: 88,
            intensity: 0.2,
            color: [1.0, 1.0, 0.0],
        };
    });
    assert!(
        one_white_lamp.iter().any(|&b| b != 255),
        "sanity: the fixture painted"
    );
    assert_eq!(
        one_white_lamp, loud_rig,
        "four saturated lamps moved FLAT paint — the light must be a property of the RELIEF, not a \
         filter over the picture. An absolute model washes the colour over every pixel the artist mixed."
    );
}

/// …and the rig is not vacuous: on paint that HAS relief, every lamp does something, and each knob counts.
///
/// The other half. Without it, the byte-identity gate above passes on a rig quietly wired to nothing at
/// all — the exact species of dead knob this module has spent its whole history exterminating. Four
/// claims, each independently falsifiable:
///
/// 1. Switching a second lamp ON changes the picture.
/// 2. Its ANGLE is live (the same lamp from the other side is a different picture).
/// 3. Its INTENSITY scales it — and at **zero** it is exactly the same as off (a lamp with no power is
///    not a lamp; `Rig::new` drops it, and the flat divisor must drop it too, or a dark lamp would
///    silently darken the whole canvas by inflating the denominator).
/// 4. Its COLOUR is live.
#[test]
fn every_lamp_in_the_rig_is_live() {
    let size = 140u32;
    let render = |arm: &dyn Fn(&mut PainterTool)| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 30.0,
            color: [0.85, 0.15, 0.15],
            impasto: true, // relief, so there is something for a lamp to model
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([35.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([105.0, 95.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([105.0, 95.0], PointerPhase::Up));
        arm(&mut t);
        t.invalidate_composite();
        lit(&mut t)
    };
    let key_only = render(&|_t| {});
    assert!(
        key_only.iter().any(|&b| b != 255),
        "sanity: the fixture painted lit relief"
    );

    // 1. A second lamp, ON.
    let fill = |t: &mut PainterTool| {
        t.paint.impasto_rig.lights[1] = ImpastoLight {
            on: true,
            angle_deg: 50,
            elev_deg: 25,
            intensity: 0.8,
            color: [1.0, 1.0, 1.0],
        };
    };
    let two = render(&fill);
    assert_ne!(two, key_only, "switching lamp 2 ON must change the picture");

    // 2. …from the OTHER side.
    let two_flipped = render(&|t| {
        fill(t);
        t.paint.impasto_rig.lights[1].angle_deg = 230; // now behind the key
    });
    assert_ne!(two_flipped, two, "the lamp's ANGLE is live");

    // 3. Intensity — and zero is exactly off. (RED if `Rig::new` counted a zero-power lamp into the flat
    //    divisor: the denominator would grow, every ratio would shrink, and the canvas would DARKEN.)
    let two_dim = render(&|t| {
        fill(t);
        t.paint.impasto_rig.lights[1].intensity = 0.2;
    });
    assert_ne!(two_dim, two, "the lamp's INTENSITY is live");
    let two_zero = render(&|t| {
        fill(t);
        t.paint.impasto_rig.lights[1].intensity = 0.0;
    });
    assert_eq!(
        two_zero, key_only,
        "a lamp at zero power is exactly a lamp that is OFF — it may not darken the canvas by inflating \
         the flat divisor"
    );

    // 4. Colour.
    let two_warm = render(&|t| {
        fill(t);
        t.paint.impasto_rig.lights[1].color = [1.0, 0.6, 0.2];
    });
    assert_ne!(two_warm, two, "the lamp's COLOUR is live");
}

/// **The key cannot be switched off** — and switching every OTHER lamp off returns the one-lamp canvas.
///
/// "Show Impasto" already is the master switch. A second one, hidden inside the rig, that can leave the
/// pass running with nothing to run it with, is how a divide-by-zero ships. `toggle_impasto_light_on`
/// refuses on lamp 0 and `Rig::new` returns `None` if it ever finds an empty rig anyway (belt and braces
/// — the guard that is not reachable today is the one that matters when someone adds a preset loader).
#[test]
fn the_key_light_cannot_be_switched_off() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.select_impasto_light(0);
    t.toggle_impasto_light_on();
    assert!(
        t.paint.impasto_rig.lights[0].on,
        "the key light must stay lit — Show Impasto is the master switch, and a lit canvas with no light \
         at all is a lie the pass would have to divide by"
    );
    // …but every other lamp toggles freely.
    for i in 1..MAX_IMPASTO_LIGHTS as u8 {
        t.select_impasto_light(i);
        let before = t.paint.impasto_rig.lights[i as usize].on;
        t.toggle_impasto_light_on();
        assert_ne!(
            t.paint.impasto_rig.lights[i as usize].on,
            before,
            "lamp {} must toggle",
            i + 1
        );
    }
}

/// **One lamp of any colour changes the paint's BRIGHTNESS, never its hue.**
///
/// The per-channel divisor, stated where it has content. With a single lamp the colour must CANCEL:
/// `diffuse[c] = tint[c]·(N·L)` and `flat[c] = tint[c]·L.z`, so `ratio[c] = (N·L)/L.z` — the same in
/// every channel, whatever the lamp's colour. There is only one light in the room, so it cannot cast a
/// hue *relative to itself*; it can only make the tilted paint brighter or darker.
///
/// (Where the colour DOES speak is a rig of several lamps at different angles: then each channel gets a
/// different mix of them, and the ratios genuinely differ — which is what `every_lamp_in_the_rig_is_live`
/// pins. The two gates are the two halves of one law.)
///
/// MUT (divide by the rig's AVERAGE flat response instead of the channel's): RED — a red lamp then tints
/// every lit slope red, and the relief starts painting hues the artist never mixed.
#[test]
fn a_single_lamp_shifts_brightness_never_hue() {
    let size = 140u32;
    // GREY paint: any hue in the output is the LIGHT's, not the pigment's — the cleanest instrument.
    let render = |color: [f32; 3]| -> Vec<u8> {
        let mut t = PainterTool::default();
        // The lamp colour and Shine are dialled AFTER the stroke, so live editing has to be on — stated,
        // not inherited (the default went OFF on 2026-07-19).
        t.paint.impasto_live_edit = true;
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 30.0,
            color: [0.5, 0.5, 0.5],
            impasto: true,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([35.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([105.0, 95.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([105.0, 95.0], PointerPhase::Up));
        t.paint.impasto_rig.lights[0].color = color;
        t.set_impasto_shine(0.0); // the DIFFUSE is what the divisor governs; the glint is gate #3's
        t.invalidate_composite();
        lit(&mut t)
    };
    let white = render([1.0, 1.0, 1.0]);
    for lamp in [[1.0f32, 0.2, 0.2], [0.2, 0.2, 1.0], [1.0, 0.85, 0.4]] {
        let tinted = render(lamp);
        let mut worst = 0i32;
        for p in 0..(size * size) as usize {
            // The pigment is grey, so under a single lamp every channel must move by the SAME amount.
            let d: Vec<i32> = (0..3)
                .map(|c| i32::from(tinted[p * 4 + c]) - i32::from(white[p * 4 + c]))
                .collect();
            worst = worst.max((d[0] - d[1]).abs().max((d[1] - d[2]).abs()));
        }
        assert!(
            worst <= 1, // 1 level of rounding across the 8-bit round-trip
            "a single {lamp:?} lamp shifted the channels of GREY paint apart by {worst} levels — one \
             light cannot cast a hue relative to itself; it can only brighten or darken"
        );
    }
}

/// **Turning every lamp down to zero is an UNLIT canvas — not a canvas darkened to the ambient floor.**
///
/// The empty rig. With every lamp at zero power the diffuse sums to zero; if the pass still ran, the
/// zero-divisor floor would turn the ratio into 0 and drive every lit pixel to `AMBIENT` — dialling the
/// lights DOWN would darken the painting to 35%. `Rig::new` drops powerless lamps and bails on an empty
/// rig, so it comes back exactly as it would with Show Impasto off.
///
/// MUT (`filter(|l| l.on)` — keep zero-power lamps): RED.
#[test]
fn the_lights_turned_all_the_way_down_is_an_unlit_canvas() {
    let size = 120u32;
    let render = |arm: &dyn Fn(&mut PainterTool)| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 28.0,
            color: [0.85, 0.15, 0.15],
            impasto: true,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([30.0, 35.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([90.0, 85.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([90.0, 85.0], PointerPhase::Up));
        arm(&mut t);
        t.invalidate_composite();
        lit(&mut t)
    };
    let unlit = render(&|t| t.paint.impasto_show = false);
    let lights_out = render(&|t| {
        for l in &mut t.paint.impasto_rig.lights {
            l.intensity = 0.0;
        }
    });
    let lit_normally = render(&|_t| {});
    assert_ne!(
        lit_normally, unlit,
        "sanity: the fixture IS lit when the lights are on"
    );
    assert_eq!(
        lights_out, unlit,
        "with every lamp at zero power the canvas must come back UNLIT — not darkened to the ambient \
         floor. Dialling the lights down may not dim the painting."
    );
}

/// **Shine only ever ADDS light** — a highlight that darkens is not a highlight.
///
/// Each lamp's specular is taken relative to ITS OWN flat response and clamped at zero there. Sum the raw
/// speculars and subtract the flat total instead, and a lamp facing AWAY from a slope contributes a
/// NEGATIVE term — it borrows headroom from a lamp facing it, and the "highlight" darkens the paint.
/// With one lamp the flat early-out hides it; with a rig it is visible.
///
/// MUT (drop the per-lamp `.max(0.0)`): RED.
#[test]
fn the_glint_only_ever_adds_light() {
    let size = 140u32;
    let render = |shine: f32| -> Vec<u8> {
        let mut t = PainterTool::default();
        // Shine is dialled AFTER the stroke, so this gate needs live editing on — stated, not inherited
        // (the default went OFF on 2026-07-19).
        t.paint.impasto_live_edit = true;
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: 30.0,
            color: [0.5, 0.5, 0.5],
            impasto: true,
            ..Default::default()
        };
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([35.0, 40.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([105.0, 95.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([105.0, 95.0], PointerPhase::Up));
        // A RIG — the lamps disagree about where the light is, which is the whole point.
        let r = &mut t.paint.impasto_rig;
        r.lights[1] = ImpastoLight {
            on: true,
            angle_deg: 40,
            elev_deg: 20,
            intensity: 0.9,
            color: [1.0, 1.0, 1.0],
        };
        r.lights[2] = ImpastoLight {
            on: true,
            angle_deg: 130,
            elev_deg: 60,
            intensity: 0.7,
            color: [1.0, 1.0, 1.0],
        };
        t.set_impasto_shine(shine);
        t.invalidate_composite();
        lit(&mut t)
    };
    let matte = render(0.0);
    let glossy = render(1.0);
    let (mut darkened, mut worst) = (0u32, 0i32);
    for p in 0..(size * size) as usize {
        for c in 0..3 {
            let d = i32::from(glossy[p * 4 + c]) - i32::from(matte[p * 4 + c]);
            if d < -1 {
                // 1 level of 8-bit rounding is not a darkening
                darkened += 1;
                worst = worst.min(d);
            }
        }
    }
    assert_eq!(
        darkened, 0,
        "Shine DARKENED {darkened} channels (worst {worst} levels) — under a rig, a lamp facing away \
         from a slope must contribute nothing to the glint, not a negative. A highlight that takes light \
         away is not a highlight."
    );
    // …and it is not vacuous: the glint must actually be visible somewhere.
    let brightest = (0..(size * size) as usize)
        .map(|p| i32::from(glossy[p * 4 + 1]) - i32::from(matte[p * 4 + 1]))
        .max()
        .unwrap_or(0);
    assert!(
        brightest > 5,
        "sanity: Shine must LIGHT the paint ({brightest} levels)"
    );
}
