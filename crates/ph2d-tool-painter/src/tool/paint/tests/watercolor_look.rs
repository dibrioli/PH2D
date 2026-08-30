//! **O LOOK da aquarela: o que o wash PARECE.** O rim que escurece a borda que recua, a granulação
//! que texturiza, o pigmento que mistura molhado-sobre-molhado, a opacidade que dá corpo, o wash vivo
//! antes do pen-up, o smudge, e o carimbo do editor de forma.

use super::*;

/// Watercolor render-path #1 — the **edge** term pools pigment at the receding boundary: a rim band
/// (just inside the wash) reads DARKER than the deep interior when Edge is on, and NOT darker when Edge
/// is off. Granulation + Warp are zeroed to isolate the edge term (the paper-noise + boundary-warp
/// fields would otherwise perturb the sampled pixels). Drives the real optical composite end-to-end
/// through `paint_end` (the "efeito perceptual" DIRETIVA §4 asserts). See `super::watercolor_render`.
#[test]
fn watercolor_edge_darkens_the_rim_not_the_interior() {
    fn wet_brush(radius: f32, edge_gain: f32) -> BrushSpec {
        BrushSpec {
            radius_px: radius,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.24, 0.39, 0.63], // mid blue → darkening is measurable in every channel
            space_attenuation: false,
            watercolor: true,
            edge_gain,
            edge_spread: 7.0,
            granulation: 0.0, // isolate the edge term from the paper granulation
            warp: 0.0,        // and from the organic-boundary displacement
            ..Default::default()
        }
    }
    fn paint_dab(brush: BrushSpec, size: u32, center: [f32; 2]) -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = brush;
        t.paint.brush_by_mode.fill(brush);
        assert!(t.on_canvas_pointer(cp(center, PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp(center, PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let center = [48.0f32, 48.0];
    let lum = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    // The rim band sits a few px inside the 20 px disc boundary (where cover is high but blur(cover) has
    // fallen — the edge term peaks); the interior is the disc centre (cover ≈ 1, blur ≈ 1 → edge ≈ 0).
    let rim_y = 32; // 16 px above centre

    // Edge ON: the rim band is darker than the interior (pigment pooled at the receding front).
    let t = paint_dab(wet_brush(20.0, 3.0), size, center);
    let interior = px(&t, size, 48, 48);
    let rim = px(&t, size, 48, rim_y);
    assert!(
        lum(rim) < lum(interior),
        "edge darkening must pool pigment at the rim: rim {rim:?} not darker than interior {interior:?}"
    );

    // Edge OFF (gain 0): no rim pooling — the boundary only has LESS coverage, so it is never darker than
    // the interior (density there is `cover·fill`, and `cover ≤ 1`).
    let t0 = paint_dab(wet_brush(20.0, 0.0), size, center);
    assert!(
        lum(px(&t0, size, 48, rim_y)) >= lum(px(&t0, size, 48, 48)),
        "with Edge off there is no rim pooling (the boundary is lighter, never darker, than the interior)"
    );
}

/// Watercolor render-path #2 — **granulation** textures the wash: the paper-tooth field modulates the
/// optical density (`gran = 1 + (paperHeight − 0.5)·2·granAmt`), so turning Granulation up raises the
/// spatial VARIANCE of the interior (mottled tooth) versus a flat wash at Granulation 0. Symmetric
/// around the mean (wet_edges), so it redistributes pigment, not a net wipe. Real optical composite,
/// Edge off. DIRETIVA §4.
#[test]
fn watercolor_granulation_textures_the_wash() {
    fn interior_variance(granulation: f32) -> f64 {
        let size = 64u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 26.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.0, 0.0, 0.0], // black on white → deposit reads as darkening
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate granulation from the edge term
            warp: 0.0,      // sample the true tooth, un-displaced
            fill: 0.6,      // a solid wash so the tooth variation is well above quantisation
            granulation,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up)));
        // Variance of the R channel over a deep-interior window (well inside the 26 px disc → cover ≈ 1,
        // so only the tooth varies the value).
        let vals: Vec<f64> = (24..40)
            .flat_map(|y| (24..40).map(move |x| (x, y)))
            .map(|(x, y)| f64::from(t.canvas_rgba[((y * size + x) * 4) as usize]))
            .collect();
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64
    }
    let flat = interior_variance(0.0);
    let granulated = interior_variance(0.8);
    assert!(
        flat < 1.0,
        "Granulation 0 is a flat wash (near-zero interior variance): got {flat}"
    );
    assert!(
        granulated > flat + 4.0,
        "Granulation must texture the wash (raise interior variance): granulated {granulated} vs flat {flat}"
    );
}

/// Watercolor render-path #3 — **Pigment** mixes wet-on-wet subtractively: painting yellow over an
/// opaque blue base with Pigment on lifts GREEN in the overlap (RYB: blue + yellow → green), where the
/// plain optical composite (Pigment off) stays a muddy Beer–Lambert blend. A dense wash (high Fill/Depth)
/// so the pigment film is opaque enough to mix. Real composite. DIRETIVA §4.
#[test]
fn watercolor_pigment_mixes_wet_on_wet_toward_green() {
    fn center_pixel(pigment_mix: f32) -> [u8; 4] {
        let size = 48u32;
        // A solid opaque blue base already on the canvas (the "previous wash" to mix into).
        let mut src = vec![0u8; (size * size * 4) as usize];
        for p in src.as_chunks_mut::<4>().0.iter_mut() {
            p.copy_from_slice(&[30, 55, 195, 255]);
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 14.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.90, 0.80, 0.10], // yellow
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate pigment from the edge term
            granulation: 0.0,
            warp: 0.0,
            fill: 0.85, // dense wash → an opaque pigment film that mixes strongly
            depth: 2.0,
            pigment: pigment_mix > 0.0,
            pigment_mix,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up)));
        px(&t, size, 24, 24)
    }
    let off = center_pixel(0.0); // Pigment off → plain Beer–Lambert blend (a muddy YELLOW-green: high red)
    let on = center_pixel(1.0); // Pigment on → subtractive RYB → a true GREEN (green pulls ahead of red)
    // "Toward green" = green dominates red more strongly with Pigment on. (The green *channel* alone is
    // higher in the yellow-green off-state — yellow carries green too — so the signature is green−red.)
    let green_lead = |p: [u8; 4]| i32::from(p[1]) - i32::from(p[0]);
    assert!(
        green_lead(on) > green_lead(off),
        "wet-on-wet pigment must swing toward green (green leading red) vs the plain blend: on {on:?} vs off {off:?}"
    );
}

/// Watercolor render-path #4 — **Opacity (pigment body)** lets a LIGHT-valued pigment deposit at its hue
/// (doc 13 #17: "azul e amarelo quase não aparecem"). Pure Beer–Lambert (`opacity = 0`) can only subtract
/// light, so a light yellow over white paper leaves its bright channels at `Tᵢ ≈ 1` and barely darkens —
/// the reported bug. Turning Opacity up lays the pigment's own colour (scattering / hiding power), so the
/// SAME wash darkens substantially MORE and keeps its yellow character (blue absorbed hardest). The
/// `opacity = 0` render is byte-identical to the old path by construction: `body_cov = 0` ⇒ the fold term
/// `(s2l[pig] − optical)·0.0` is exactly `0.0` and `max(1−t_min, 0)` is unchanged. Real composite,
/// Edge/Warp/Granulation off to isolate the body term. DIRETIVA §4 (verified RED by neutering the fold).
#[test]
fn watercolor_opacity_gives_light_pigments_body() {
    fn light_yellow_center(opacity: f32) -> [u8; 4] {
        let size = 64u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 20.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.95, 0.85, 0.20], // light yellow → bright R,G (near-transparent under Beer–Lambert)
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the body term from the rim
            granulation: 0.0,
            warp: 0.0,
            fill: 0.15, // a thin default-ish wash — where the light-pigment invisibility bites hardest
            depth: 1.2,
            opacity,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up)));
        px(&t, size, 32, 32)
    }
    // Total darkening away from white paper — the "how much did the wash actually deposit" meter.
    let deposit =
        |p: [u8; 4]| (255 - u32::from(p[0])) + (255 - u32::from(p[1])) + (255 - u32::from(p[2]));
    let transparent = light_yellow_center(0.0); // pure Beer–Lambert → the faint, near-invisible wash
    let bodied = light_yellow_center(0.8); //       body on → the pigment shows at its hue
    assert!(
        deposit(bodied) > deposit(transparent) + 30,
        "Opacity must give a light pigment body (deposit far more): bodied {bodied:?} (Δ{}) vs transparent {transparent:?} (Δ{})",
        deposit(bodied),
        deposit(transparent),
    );
    // The character stays YELLOW: blue is the hardest-absorbed channel in both (body lays the pigment's
    // OWN colour, it does not gray the wash toward paper).
    assert!(
        bodied[2] < bodied[0] && bodied[2] < bodied[1],
        "body must preserve the pigment hue (blue absorbed hardest): {bodied:?}"
    );
}

/// Watercolor render-path is **LIVE** — the wash appears *during* the stroke (each frame recomposited
/// from the growing coverage over the frozen base), not as a jump on release. Paint a horizontal band
/// and, WITHOUT releasing, assert (a) the interior already differs from the white base and (b) the rim
/// is already darker than the centreline. (Fix for the "não pinta em tempo real / escurece no final"
/// feedback; the pen-up bake is covered by `watercolor_edge_darkens_the_rim_not_the_interior`.)
#[test]
fn watercolor_wash_is_live_before_pen_up() {
    let size = 96u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.25, 0.40, 0.62],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 2.0,
        edge_spread: 5.0,
        granulation: 0.0,
        warp: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let lum = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    // Paint a horizontal band and STOP without releasing (no Up event).
    assert!(t.on_canvas_pointer(cp([24.0, 48.0], PointerPhase::Down)));
    for x in [32.0, 40.0, 48.0, 56.0, 64.0] {
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    frame(&mut t); // the frame those Moves belong to — the reconstruction is owed to it
    // Pointer still down: the wash is ALREADY on the canvas (interior differs from the white base) and
    // its rim is ALREADY darker than the centreline.
    let interior = px(&t, size, 44, 48);
    assert!(
        lum(interior) < 3 * 255,
        "the wash is live mid-stroke (interior no longer white)"
    );
    let rim = px(&t, size, 44, 40); // 8 px above centre → top rim of the radius-10 band
    assert!(
        lum(rim) < lum(interior),
        "edge must be LIVE mid-stroke: rim {rim:?} not darker than interior {interior:?}"
    );
}

/// Watercolor is **inert when off**: a `watercolor = false` stroke is byte-identical to a plain brush
/// (the render-path skips deposit AND composite — the skip-deposit gate must not leak into a normal
/// stroke). Paints the same dab with the flag off and confirms real pigment landed on the canvas.
#[test]
fn watercolor_off_is_a_plain_deposit() {
    let size = 48u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.10, 0.20, 0.80],
        space_attenuation: false,
        watercolor: false, // OFF → the plain deposit path, no optical composite
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Down)));
    assert!(t.on_canvas_pointer(cp([24.0, 24.0], PointerPhase::Up)));
    // The dab deposited the (opaque) brush colour straight — a plain Mix over white.
    let c = px(&t, size, 24, 24);
    assert_eq!(c[3], 255, "opaque deposit");
    assert!(
        c[2] > c[0] && c[2] > c[1],
        "the plain blue brush colour landed (blue dominant): {c:?}"
    );
}

/// Watercolor **TRUE SMEAR** — Smudge > 0 physically DRAGS the already-painted paint along the stroke
/// ("Smearing", not a colour tint): crossing a red band with Pickup 0 (pure brush-colour wash, so the
/// smear is isolated) must (a) drag red PAST the band's far edge, and (b) drag white INTO the band's
/// entry edge — displacement of the base, which the reservoir-only model never did ("levanta mas não
/// borra a já pintada", Enio 2026-07-06).
#[test]
fn watercolor_smudge_true_smears_the_painted_paint() {
    fn run(smudge: f32) -> PainterTool {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (40..70).contains(&x) {
                    [217u8, 13, 13, 255] // red band mid-canvas
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        // NB: the engine's default (soft) falloff — a Constant falloff at full strength degenerates the
        // smear into a rigid translation (the disc's initial content overwrites everything it crosses).
        t.paint.brush = BrushSpec {
            radius_px: 6.0,
            color: [0.1, 0.2, 0.85],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.3, // a light wash so the (smeared) base reads through
            depth: 1.0,
            wet_smudge: smudge,
            wet_rewet: 0.0, // isolate the physical smear from the wet-on-wet rewet
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        // Left-to-right stroke crossing the band and exiting into white.
        assert!(t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 96.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up)));
        t
    }
    let size = 128u32;
    let plain = run(0.0);
    let smeared = run(0.9);
    // (a) Past the band's far edge: the smear dragged red out of the band → markedly redder (lower G/B
    // vs R) than the plain wash over white.
    let (ex, ey) = (74u32, 64u32);
    let p = px(&plain, size, ex, ey);
    let s = px(&smeared, size, ex, ey);
    let redness = |c: [u8; 4]| i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2;
    assert!(
        redness(s) > redness(p) + 40,
        "smear must drag red past the band edge: smeared {s:?} vs plain {p:?}"
    );
    // (b) At the band's entry edge: the smear dragged white INTO the band → lighter than the plain
    // wash over pristine red.
    let (bx, by) = (43u32, 64u32);
    let p = px(&plain, size, bx, by);
    let s = px(&smeared, size, bx, by);
    let lum = |c: [u8; 4]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
    assert!(
        lum(s) > lum(p) + 60,
        "smear must drag white into the band's entry edge: smeared {s:?} vs plain {p:?}"
    );
}

/// **Watercolor Smudge wraps across the Tiling seam (doc 13 #2, follow-up a — Enio 2026-07-11).** With
/// Tiling on, the coverage/color wash already wraps (`tiled_dabs`); the TRUE SMEAR must wrap too, or the
/// far edge's wash composites over an UN-smeared base — a visible smudge seam. A rightward smear crossing
/// the RIGHT edge lifts the right-edge paint toroidally and stamps it onto the wrapped LEFT edge, so a red
/// right-edge band gets dragged onto the left edge (unreachable without the wrap). RED before the fix:
/// under Tiling the left edge is identical at smudge 0 vs 0.9 (the smear never touched the far edge).
#[test]
fn watercolor_smudge_wraps_across_the_tiling_seam() {
    let size = 64u32;
    fn run(smudge: f32, tiling: bool) -> PainterTool {
        let size = 64u32;
        // White canvas with a RED right THIRD (x∈[42,63]) — plenty of paint for the wrapped smear to drag.
        let mut src = vec![255u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 42..size {
                let i = ((y * size + x) * 4) as usize;
                src[i..i + 4].copy_from_slice(&[230u8, 15, 15, 255]);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 6.0,
            color: [0.1, 0.2, 0.85], // blue wash, so any RED on the far edge can only be dragged base
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.12, // very light wash so the (smeared) base reads through clearly
            depth: 1.0,
            wet_smudge: smudge,
            wet_rewet: 0.0, // isolate the physical smear from the wet-on-wet rewet
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.paint.tiling = [tiling, false];
        // Rightward stroke near the right edge, crossing the seam (dabs at x≈52..72, radius 6 ⇒ the copies
        // wrap onto the left edge); a dense step (2 px) so the drag accumulates, y=32.
        assert!(t.on_canvas_pointer(cp([50.0, 32.0], PointerPhase::Down)));
        let mut x = 50.0f32;
        while x < 74.0 {
            x += 2.0;
            t.on_canvas_pointer(cp([x, 32.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 32.0], PointerPhase::Up)));
        t
    }
    let redness = |c: [u8; 4]| i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2;
    let (lx, ly) = (2u32, 32u32); // a wrapped left-edge pixel
    let base = run(0.0, true); // Tiling on, no smudge: wash-only far edge (wash wraps in both runs)
    let smeared = run(0.9, true); // Tiling on + smudge: the wrapped smear drags red onto the far edge
    let off = run(0.9, false); // Tiling OFF: the smear can't reach the far edge (proves it's the wrap)
    let b = px(&base, size, lx, ly);
    let s = px(&smeared, size, lx, ly);
    let o = px(&off, size, lx, ly);
    assert!(
        redness(s) > redness(b) + 20,
        "the wrapped smear dragged red onto the far edge: smeared {s:?} vs wash-only {b:?}"
    );
    assert!(
        redness(s) > redness(o) + 20,
        "the far-edge red is the Tiling WRAP, not a non-tiled path: tiled {s:?} vs off {o:?}"
    );
}

/// Watercolor **dirty-rect** — the live recomposite is LOCAL to the frame's new dabs (wet_edges
/// `renderFrame`), so the per-frame cost tracks the brush, not the grown stroke (the old cumulative-bbox
/// recomposite was ~quadratic along a stroke — the "Performance muito aquém do MVP" symptom). Proof by
/// sentinel: a pixel poked into the ALREADY-painted area, far behind the stroke frontier, must survive
/// the live passes untouched (a full-bbox recomposite would overwrite it) — and then be recomposited by
/// the pen-up bake (wet_edges `endStroke`), which makes ONE cumulative pass from the incremental bbox.
#[test]
fn watercolor_live_recomposite_is_local_to_the_frame() {
    let size = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 6.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.70],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 2.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.5,
        depth: 2.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    // Paint the left third of a horizontal band, live.
    assert!(t.on_canvas_pointer(cp([30.0, 128.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([50.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([70.0, 128.0], PointerPhase::Move));
    frame(&mut t);
    let washed = px(&t, size, 30, 128);
    assert_ne!(
        washed,
        [255, 255, 255, 255],
        "the stroke start is washed live"
    );
    // Poke a sentinel into the already-painted area. Every later dab lands ≥ 40 px away — far beyond
    // the influence radius (radius 6 + spread 4 + pads) — so a frame-local recomposite must not touch it.
    const SENTINEL: [u8; 4] = [7, 250, 11, 255];
    {
        let buf = Arc::make_mut(&mut t.canvas_rgba);
        let i = ((128 * size + 30) * 4) as usize;
        buf[i..i + 4].copy_from_slice(&SENTINEL);
    }
    // Extend the stroke far to the right: the live passes recomposite only around the new dabs.
    // Both Moves ride the SAME frame, so the window this tick composites is their union — still local
    // to the frontier, which is the property under test (a full-bbox pass would eat the sentinel).
    t.on_canvas_pointer(cp([120.0, 128.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([170.0, 128.0], PointerPhase::Move));
    frame(&mut t);
    assert_eq!(
        px(&t, size, 30, 128),
        SENTINEL,
        "a live pass recomposited far behind the frontier — the dirty rect is not frame-local"
    );
    // The frame dirty rect is consumed by each composite; the cumulative one spans the whole band.
    assert!(t.paint.wet_frame_dirty.is_none(), "frame rect consumed");
    let cum = t.paint.wet_cum_dirty.expect("cumulative rect tracked");
    // (The stroke smoother lags the dabs behind the pointer, so the right edge trails the cursor.)
    assert!(
        cum.x <= 25 && cum.x + cum.w >= 120,
        "cumulative rect spans the stroke: {cum:?}"
    );
    // Pen-up: the bake recomposites the WHOLE stroke from the tracked bbox — the sentinel is repainted.
    assert!(t.on_canvas_pointer(cp([220.0, 128.0], PointerPhase::Up)));
    let baked = px(&t, size, 30, 128);
    assert_ne!(
        baked, SENTINEL,
        "the pen-up bake recomposites the full stroke"
    );
    assert_ne!(
        baked,
        [255, 255, 255, 255],
        "…back to the wash, not the base"
    );
}

/// Watercolor dirty-rect × moving preview (Drag Dot/Anchored/Line): those methods CLEAR the coverage and
/// re-stamp the whole shape each frame, so the frame dirty rect must be the UNION of the old + new shape
/// (`clear_wet_coverage` folds the cumulative rect in) — a rect of only the new dabs would leave the old
/// position composited as a stale trail. A Drag Dot moved across the canvas must restore the base at its
/// old position, live.
#[test]
fn watercolor_moving_preview_restores_the_old_position() {
    let size = 96u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 5.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.70],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 3.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 2.0,
        stroke_method: StrokeMethod::DragDot,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([70.0, 32.0], PointerPhase::Down)));
    assert_ne!(
        px(&t, size, 70, 32),
        [255, 255, 255, 255],
        "the dot preview is washed at the press point"
    );
    // Drag the dot far away: the old position is no longer covered → the live pass restores the base.
    // (Drag Dot is a re-stamp method, so the shell coalesces its Moves to ONE delivery per frame
    // anyway — the tick is where its composite lands either way.)
    t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Move));
    frame(&mut t);
    assert_eq!(
        px(&t, size, 70, 32),
        [255, 255, 255, 255],
        "the moved preview left a stale trail at the old position"
    );
    assert_ne!(
        px(&t, size, 24, 32),
        [255, 255, 255, 255],
        "the dot is washed at the new position"
    );
    assert!(t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Up)));
    assert_ne!(
        px(&t, size, 24, 32),
        [255, 255, 255, 255],
        "the release point keeps the committed dot"
    );
}

/// Watercolor render-path is gated on an OPEN stroke (the frozen base exists): the shape editors
/// (Line/Arc/Ellipse/Polygon/Free Hand, `stroke_multi`) stamp via the drag-preview WITHOUT the stroke
/// lifecycle — routed into the watercolor accumulation they painted NOTHING (no composite ever ran)
/// and leaked never-cleared coverage. Outside a stroke a dab must fall through to the plain deposit.
#[test]
fn watercolor_editor_stamp_deposits_without_an_open_stroke() {
    let size = 48u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.85],
        watercolor: true,
        ..Default::default()
    };
    // No pointer Down: this is how the shape editors stamp (stamp_drag_preview → stamp_dabs).
    let dab = Dab {
        center: [24.0, 24.0],
        radius_px: 8.0,
        coverage: 1.0,
        color: t.paint.brush.color,
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: 8.0,
    };
    t.stamp_dabs(&[dab]);
    assert_ne!(
        px(&t, size, 24, 24),
        [255, 255, 255, 255],
        "an editor dab with Watercolor on must paint (plain deposit), not a dead brush"
    );
    assert!(
        t.paint.stroke_coverage.iter().all(|&c| c == 0),
        "no watercolor coverage may leak outside an open stroke"
    );
}

/// Manual perf probe (not a gate): per-frame watercolor cost along a LONG stroke on a big canvas —
/// the dirty-rect must keep it ~constant (the old cumulative recomposite grew it ~quadratically).
/// Run: `cargo test -p ph2d-tool-painter --release -- --ignored watercolor_perf`
#[test]
#[ignore = "manual perf probe — run in --release and read the printed ms"]
fn watercolor_perf_frame_cost_probe() {
    probe(0.0);
    probe(1.0);
}

fn probe(wet: f32) {
    let size = 2048u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 16.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.70],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 2.0,
        edge_spread: 8.0,
        granulation: 0.4,
        warp: 3.0,
        fill: 0.5,
        depth: 2.0,
        wet_rewet: wet,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let y = 1024.0f32;
    assert!(t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down)));
    let n = 440usize; // ~1760 px of stroke, 4 px per Move
    let mut ms = Vec::with_capacity(n);
    for i in 0..n {
        let x = 100.0 + (i as f32 + 1.0) * 4.0;
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let t0 = std::time::Instant::now();
    assert!(t.on_canvas_pointer(cp([100.0 + n as f32 * 4.0, y], PointerPhase::Up)));
    let commit_ms = t0.elapsed().as_secs_f64() * 1e3;
    let avg = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    let max = ms.iter().cloned().fold(0.0f64, f64::max);
    eprintln!(
        "watercolor per-frame (wet {wet}): first-40 {:.3} ms · last-40 {:.3} ms · max {max:.3} ms · commit {commit_ms:.3} ms ({n} moves, {size}² canvas)",
        avg(&ms[..40]),
        avg(&ms[n - 40..]),
    );
    eprintln!("total live {:.1} ms", ms.iter().sum::<f64>());
}

/// Granulation **Amount is inert without a settling substrate** (Enio 2026-07-06): with NO Grain image
/// and "Same as Paper" OFF there is nothing to settle into, so cranking Amount must not texture the
/// wash (it granulated out of thin air via the built-in-noise fallback). With Same-as-Paper ON (the
/// default) the paper tooth — built-in noise before a Paper is wired — granulates as before
/// (`watercolor_granulation_textures_the_wash` pins that side).
#[test]
fn watercolor_granulation_amount_is_inert_without_a_source() {
    let size = 64u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.paint.brush = BrushSpec {
        radius_px: 26.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        warp: 0.0,
        fill: 0.6,
        granulation: 1.0,             // full Amount…
        granulation_use_paper: false, // …but no source: Same-as-Paper off…
        ..Default::default()          // …and no Grain image set
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up)));
    // Deep-interior window (cover ≈ 1): with no substrate the wash must be FLAT (zero variance).
    let mut vals = Vec::new();
    for y in 24..40 {
        for x in 24..40 {
            vals.push(f64::from(px(&t, size, x, y)[0]));
        }
    }
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let var = vals.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / vals.len() as f64;
    assert!(
        var < 0.5,
        "Amount with no Grain image and Same-as-Paper off must not granulate (variance {var:.2})"
    );
}

/// **GRAN-1 (doc 12, Curtis §4.5): granulação é deposição nos VALES — vales escuros, picos claros.**
/// O sinal antigo era INVERTIDO (picos h altos escureciam) e o clamp furava o wash com speckle
/// branco em amount alto. Grain map metade preta (h=0, vales) / metade branca (h=1, picos),
/// granulation 1.0: o wash sobre os VALES deposita mais (mais escuro) que sobre os picos — e
/// nenhum texel do wash fica branco puro (sem speckle: o gate é limitado por γ < 1).
#[test]
fn watercolor_granulation_deposits_into_valleys_not_peaks() {
    let size = 64u32;
    let mut t = white_canvas(size, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0, // isolate the fill term (no rim)
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 1.0,
        granulation_use_paper: false, // the Grain slot IS the granulation map
        ..Default::default()
    };
    // Canvas-anchored granulation map: left half BLACK (valleys), right half WHITE (peaks).
    let mut lum = vec![255u8; 16 * 16];
    for y in 0..16 {
        for x in 0..8 {
            lum[y * 16 + x] = 0;
        }
    }
    t.set_brush_texture_image(lum, 16, 16);
    t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
    // Image sampling uses the dab-space convention (`u·0.5 + 0.5`): one tile unit = HALF the
    // image, so Size 8 makes the full 16-px image span the 64-px canvas — WHITE (peaks) lands on
    // the left half, BLACK (valleys) on the right (the u-wrap crosses the halves at x = 32).
    t.paint.brush.texture.size = [8.0, 8.0];
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down)));
    for i in 1..=20 {
        t.on_canvas_pointer(cp([12.0 + i as f32 * 2.0, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    // Mean green channel of the wash core on each half (row 32, away from the rim).
    let mean_g = |x0: u32, x1: u32| -> f32 {
        let mut acc = 0.0f32;
        for x in x0..x1 {
            acc += f32::from(px(&t, size, x, 32)[1]);
        }
        acc / (x1 - x0) as f32
    };
    let peaks = mean_g(16, 28); // white-map half (h = 1) — left under the dab-space wrap
    let valleys = mean_g(36, 48); // black-map half (h = 0) — right
    assert!(
        valleys + 8.0 < peaks,
        "valleys must deposit MORE pigment (darker) than peaks: valleys {valleys:.1} vs peaks {peaks:.1}"
    );
    // No white speckle: every core texel is painted (the old symmetric clamp punched h-low texels to 0
    // density... and at the new sign, γ < 1 bounds the peak side — nothing in the core stays white).
    for x in 16..48u32 {
        assert_ne!(
            px(&t, size, x, 32),
            [255, 255, 255, 255],
            "no white speckle inside the wash (x={x})"
        );
    }
}

/// **Settle take 3 está LIGADO (Enio 2026-07-08: "nem sei se está funcionando")**: o preview vivo
/// roda a ~80% do settle e o bake aplica 100% — então soltar a caneta CLAREIA os PICOS do tooth
/// (o pigmento termina de ceder pros vales) enquanto os VALES ficam praticamente iguais. Se live
/// e bake fossem idênticos (WYSIWYG) o delta seria 0; se o preview estivesse longe (take 1) o
/// delta seria um pop. Este teste pina o meio-termo: delta presente, pequeno e direcional.
#[test]
fn watercolor_granulation_bake_settles_beyond_the_live_preview() {
    let size = 64u32;
    let mut t = white_canvas(size, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.25, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 1.0,
        granulation_use_paper: false,
        wet_rewet: 0.0, // no water: live settle = GRAN_SETTLE_BASE exactly
        ..Default::default()
    };
    let mut lum = vec![255u8; 16 * 16];
    for y in 0..16 {
        for x in 0..8 {
            lum[y * 16 + x] = 0;
        }
    }
    t.set_brush_texture_image(lum, 16, 16);
    t.paint.brush.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
    t.paint.brush.texture.size = [8.0, 8.0];
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([12.0, 32.0], PointerPhase::Down)));
    for i in 1..=20 {
        t.on_canvas_pointer(cp([12.0 + i as f32 * 2.0, 32.0], PointerPhase::Move));
    }
    frame(&mut t); // the frame that pays those Moves' reconstruction — this IS "the last composite"
    // LIVE snapshot (last composite before release; the Up lands at the same position, so the
    // coverage is already saturated — the only delta left is the settle).
    let live: Vec<u8> = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
    let mean_g = |buf: &[u8], x0: u32, x1: u32| -> f32 {
        let mut acc = 0.0f32;
        for x in x0..x1 {
            acc += f32::from(buf[((32 * size + x) * 4 + 1) as usize]);
        }
        acc / (x1 - x0) as f32
    };
    // PEAKS (white-map half, left under the dab-space wrap): the bake sheds MORE pigment → lighter.
    let (peaks_live, peaks_baked) = (mean_g(&live, 16, 28), mean_g(&t.canvas_rgba, 16, 28));
    assert!(
        peaks_baked > peaks_live + 2.0,
        "the bake must settle beyond the live preview on the PEAKS (live {peaks_live:.1} → baked {peaks_baked:.1})"
    );
    // Upper bound = the physics ceiling at FULL amount (gate 0.28 → 0.10 ⇒ ~50 bytes here);
    // at the default Granulation 0.3 the felt delta is ~⅓ of this (Enio's smoke: "preview
    // próximo do bake"). The bound guards against a runaway (e.g. live base accidentally 0).
    assert!(
        peaks_baked - peaks_live < 80.0,
        "…bounded set, not a runaway pop (live {peaks_live:.1} → baked {peaks_baked:.1})"
    );
    // VALLEYS (black-map half): full deposit in both → essentially unchanged by the release.
    let (val_live, val_baked) = (mean_g(&live, 36, 48), mean_g(&t.canvas_rgba, 36, 48));
    assert!(
        (val_baked - val_live).abs() < 3.0,
        "valleys keep their deposit across the release (live {val_live:.1} → baked {val_baked:.1})"
    );
}
