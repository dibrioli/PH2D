//! **A ÁGUA: o que ela levanta, carrega e espalha.** Wet/rewet, o lift que clareia, o bleed, o soak
//! que aprofunda enquanto a tinta descansa, o spread que esvazia o centro da poça, e o wet-mix — a
//! carga do pincel, o que ela deposita, como se esgota ao longo do traço e o que sangra na saída.

use super::*;

/// Watercolor **Wet** (wet-on-wet rewetting, per-pixel, no physics — Enio 2026-07-06): a wash crossing
/// a dry red band must (a) LIFT the paint under it (the band under the wash reads lighter — pigment
/// pulled off the paper), and (b) DISSOLVE its colour into the wet region (the wash a few px OUTSIDE
/// the band reads redder — the one-shot diffusion bleed). Smudge 0 isolates the rewet; Wet 0 is the
/// control (and stays byte-identical to the plain wash, which the 13 base watercolor tests pin).
#[test]
fn watercolor_wet_lifts_and_bleeds_the_painted_paint() {
    fn run(wet: f32) -> PainterTool {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (40..70).contains(&x) {
                    [217u8, 13, 13, 255] // dry red band mid-canvas
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62], // a light blue wash — lift/bleed must read through it
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the rewet from the edge pooling
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            wet_smudge: 0.0,
            wet_rewet: wet,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 100.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up)));
        t
    }
    let size = 128u32;
    let dry = run(0.0);
    let wet = run(1.0);
    // (a) LIFT: deep inside the band, under the wash, the band's pigment is pulled off the paper — the
    // channel it absorbed least (R, its own reflectance) brightens strongly toward the paper. (Overall
    // luminance is NOT the right meter: the dissolved red simultaneously tints the wash, darkening G/B —
    // the pigment redistributes rather than vanishing.)
    let (ix, iy) = (55u32, 64u32);
    let d = px(&dry, size, ix, iy);
    let w = px(&wet, size, ix, iy);
    assert!(
        w[0] > d[0] + 40,
        "Wet must lift the paint under the wash: wet {w:?} vs dry {d:?}"
    );
    // (b) BLEED: in the wash a few px OUTSIDE the band, the dissolved red tints the wet region.
    let (ox, oy) = (73u32, 64u32);
    let d = px(&dry, size, ox, oy);
    let w = px(&wet, size, ox, oy);
    let redness = |c: [u8; 4]| i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2;
    assert!(
        redness(w) > redness(d) + 15,
        "Wet must bleed the dissolved colour beyond the band: wet {w:?} vs dry {d:?}"
    );
}

/// Wet **redistributes the wash's own pigment** on ANY canvas (blank included): more water = the
/// interior thins (pigment migrates out) while the receding front pools harder — so the Spread ring
/// reads MORE intense under Wet, never drowned (the old uniform pool + the white-canvas presence bug
/// flattened it — Enio 2026-07-06). On blank canvas there is still no lift/dissolve (nothing darkens
/// the paper), only the redistribution.
#[test]
fn watercolor_wet_redistributes_the_wash_on_blank_canvas() {
    fn run(wet: f32) -> PainterTool {
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
            fill: 0.4,
            depth: 2.0,
            wet_rewet: wet,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([24.0, 48.0], PointerPhase::Down)));
        for x in [36.0, 48.0, 60.0, 72.0] {
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([72.0, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let dry = run(0.0);
    let wet = run(1.0);
    let lum = |c: [u8; 4]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
    // Deep interior of the band: the wet wash is LIGHTER (its pigment migrated to the front).
    let di = px(&dry, size, 48, 48);
    let wi = px(&wet, size, 48, 48);
    assert!(
        lum(wi) > lum(di),
        "Wet must thin the wash interior: wet {wi:?} vs dry {di:?}"
    );
    // The rim band (just inside the boundary): the wet wash pools HARDER (darker ring).
    let dr = px(&dry, size, 48, 41);
    let wr = px(&wet, size, 48, 41);
    assert!(
        lum(wr) < lum(dr),
        "Wet must intensify the receding-front pool: wet {wr:?} vs dry {dr:?}"
    );
}

/// Wet lift **stays in the paint's hue without Pigment** (Enio 2026-07-06, screenshot: sem Pigment a
/// tinta rewetted ficava "pálida e amarelada"): rewetting red paint with a red wash must read light
/// RED (pink) — the density-proportional log-space lift walks the colour down its own Beer–Lambert
/// curve — never the cream of the virtual paper (the old linear lerp desaturated straight to cream,
/// R−G collapsing). Pigment OFF is the whole point here.
#[test]
fn watercolor_wet_lift_stays_in_hue_without_pigment() {
    let size = 96u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for px4 in src.as_chunks_mut::<4>().0.iter_mut() {
        px4.copy_from_slice(&[217, 13, 13, 255]); // a dry red wash everywhere
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.05, 0.05], // red over red — hue must survive the lift
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 7.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.12,
        depth: 1.2,
        pigment: false, // the un-checked path under test
        wet_smudge: 0.0,
        wet_rewet: 1.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([30.0, 48.0], PointerPhase::Down)));
    for x in [42.0, 54.0, 66.0] {
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([66.0, 48.0], PointerPhase::Up)));
    // Deep interior of the wash: lifted (lighter than the dry red) but still RED-dominant, not cream.
    let c = px(&t, size, 48, 48);
    assert!(c[0] > 220, "the lift lightened the red: {c:?}");
    assert!(
        i32::from(c[0]) - i32::from(c[1]) > 60,
        "the lifted paint must stay in hue (pink, not cream): {c:?}"
    );
    assert!(
        (i32::from(c[1]) - i32::from(c[2])).abs() < 20,
        "no yellow cast (G≈B for a lifted red): {c:?}"
    );
}

/// At full Wet the subtractive paint-mix runs at full strength with or WITHOUT the Pigment checkbox —
/// `mix = max(Pigment's Mix, wet)` — so the two paths converge byte-identical (the RYB blend is "o
/// segredo" of the good wet-on-wet, Enio 2026-07-06; it must not be locked behind the checkbox). At
/// `wet = 0` only the checkbox drives it (the byte-identical default, pinned by the base suite).
#[test]
fn watercolor_wet_drives_the_paint_mix_without_pigment() {
    fn run(pigment: bool) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px4 in src.as_chunks_mut::<4>().0.iter_mut() {
            px4.copy_from_slice(&[217, 13, 13, 255]); // dry red everywhere
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 12.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.85], // blue wash over red — the mix is unmistakable
            space_attenuation: false,
            watercolor: true,
            edge_gain: 2.0,
            edge_spread: 7.0,
            granulation: 0.3,
            warp: 3.0,
            fill: 0.12,
            depth: 1.2,
            pigment,
            wet_smudge: 0.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([30.0, 48.0], PointerPhase::Down)));
        for x in [42.0, 54.0, 66.0] {
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([66.0, 48.0], PointerPhase::Up)));
        t
    }
    let off = run(false);
    let on = run(true);
    assert_eq!(
        off.canvas_rgba.as_slice(),
        on.canvas_rgba.as_slice(),
        "at wet = 1 the paint-mix must run fully with Pigment unchecked (max(Mix, wet) = 1 both ways)"
    );
}

/// **T1 (doc 11 §5 F1) — the beige is dead:** a watercolor stroke on a TRANSPARENT layer over a
/// white layer below must flatten to the SAME appearance as the identical stroke painted directly
/// on an opaque white base. The old virtual-cream ground baked `T·PAPER·film_a` of beige into the
/// pixels — over a white backdrop the wash carried a permanent warm cast ("puxa para o bege").
#[test]
fn watercolor_ground_is_the_real_backdrop_not_a_virtual_cream() {
    let size = 96u32;
    fn wet_brush() -> BrushSpec {
        BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.15, 0.15],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 1.5,
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.2,
            depth: 1.2,
            wet_smudge: 0.0,
            wet_rewet: 0.0,
            ..Default::default()
        }
    }
    fn stroke(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([20.0, 48.0], PointerPhase::Down)));
        let mut x = 20.0f32;
        while x < 76.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
    }
    // (a) Reference: the stroke painted directly on an opaque white base.
    let mut direct = PainterTool::default();
    direct.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    direct.paint.brush = wet_brush();
    direct.paint.brush_by_mode.fill(direct.paint.brush);
    stroke(&mut direct);
    // (b) The stroke on a TRANSPARENT layer added above the same white base.
    let mut layered = PainterTool::default();
    layered.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    layered.add_raster_layer("wash").expect("add layer");
    layered.paint.brush = wet_brush();
    layered.paint.brush_by_mode.fill(layered.paint.brush);
    stroke(&mut layered);
    // Flatten (b) through the real compositor and compare inside the wash.
    let active = layered.layers.active().expect("active");
    let src = crate::tool::ToolPixelSource {
        active_id: active,
        active_rgba: &layered.canvas_rgba,
        images: &layered.images,
    };
    let flat = crate::compositor::composite(&layered.layers, &src, size, size);
    let mut worst = 0i32;
    for y in 40..57u32 {
        for x in 24..72u32 {
            let i = ((y * size + x) * 4) as usize;
            let d = px(&direct, size, x, y);
            for c in 0..3 {
                worst = worst.max((i32::from(flat[i + c]) - i32::from(d[c])).abs());
            }
        }
    }
    assert!(
        worst <= 2,
        "flatten(transparent layer over white) must equal painting on white directly \
         (un-premultiply bake, no ground baked in); worst channel delta {worst}"
    );
}

/// **T3-cinza (doc 11 §5 F1) — the rewet presence is ground-relative:** with the document PAPER
/// COLOUR set to the same mid-gray as the canvas, a plain gray canvas IS the paper — nothing to
/// lift, so Wet must not brighten the wash's interior. (A gray canvas under the default WHITE
/// paper is legitimately liftable paint — Rebelle rewets a gray fill the same way; the paper
/// colour field is exactly how the artist declares "this gray is my paper".) The old global-cream
/// reference had no such control and read ANY non-cream ground as paint.
#[test]
fn watercolor_wet_reads_no_paint_on_a_paper_colored_ground() {
    fn run(wet: f32) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px4 in src.as_chunks_mut::<4>().0.iter_mut() {
            px4.copy_from_slice(&[100, 100, 100, 255]); // uniform mid-gray, no paint anywhere
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.set_paper_color_rgb8(100, 100, 100); // declare the gray as the document paper
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the rewet from the edge pooling
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            wet_smudge: 0.0,
            wet_rewet: wet,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 80.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let dry = run(0.0);
    let wet = run(1.0);
    // Interior of the wash: with no paint below, wet vs dry may differ only by the wash's own
    // redistribution (interior thinning) — never by a lift toward a foreign paper colour.
    for x in [30u32, 48, 66] {
        let d = px(&dry, size, x, 48);
        let w = px(&wet, size, x, 48);
        for c in 0..3 {
            let delta = (i32::from(w[c]) - i32::from(d[c])).abs();
            assert!(
                delta <= 12,
                "plain gray must not be lifted (presence 0): x={x} c={c} wet {w:?} vs dry {d:?}"
            );
        }
    }
}

/// **T2 (doc 11 §5 F1) — paint LIGHTER than the old cream is liftable now:** a pale near-white
/// pink band (250,225,225) on white reads presence 0 against the old cream reference (no channel
/// darker than the paper ⇒ invisible to the rewet); against the real white ground its |Δ| = 30 on
/// G/B registers, so Wet lifts it toward white.
#[test]
fn watercolor_wet_lifts_paint_lighter_than_the_old_cream() {
    fn run(wet: f32) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (36..60).contains(&x) {
                    [250u8, 225, 225, 255] // pale pink band — LIGHTER than the old cream paper
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.98, 0.98, 0.98], // near-clear water: the lift dominates, not the wash's film
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            opacity: 0.0, // near-clear water = no body film; isolate the wet LIFT (its own test)
            wet_smudge: 0.0,
            wet_rewet: wet,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 80.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let dry = run(0.0);
    let wet = run(1.0);
    let d = px(&dry, size, 48, 48);
    let w = px(&wet, size, 48, 48);
    assert!(
        i32::from(w[1]) >= i32::from(d[1]) + 4 && i32::from(w[2]) >= i32::from(d[2]) + 4,
        "Wet must lift a pale band toward the white ground (old cream reference read it as \
         presence 0): wet {w:?} vs dry {d:?}"
    );
}

/// **F3 (doc 11 §5) — soak: the longer the water sits, the farther/deeper it dissolves.** Holding
/// the wet brush parked over a dry band (the tick heartbeat pours dwell) must (a) deepen the lift
/// under the nib and (b) push the dissolved tint FARTHER outside the band than a pass-through
/// stroke — the dissolve's blur lerps toward a 2× radius where the soak accumulated.
#[test]
fn watercolor_soak_deepens_and_widens_the_dissolve_while_parked() {
    fn run(hold_s: f32) -> PainterTool {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (52..76).contains(&x) {
                    [217u8, 13, 13, 255] // dry red band mid-canvas
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.25, 0.40, 0.62],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            edge_spread: 6.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.25,
            depth: 1.0,
            wet_smudge: 0.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down)));
        let mut x = 40.0f32;
        while x < 64.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        // Park over the band: each tick pours soak under the nib (0 ticks = pass-through control).
        let mut held = 0.0f32;
        while held < hold_s {
            t.paint_tick(0.1);
            held += 0.1;
        }
        assert!(t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up)));
        t
    }
    let size = 128u32;
    let quick = run(0.0);
    let held = run(2.0);
    // (a) Deeper lift under the parked nib (soak boosts the lift fraction).
    let q = px(&quick, size, 62, 64);
    let h = px(&held, size, 62, 64);
    // R saturates near the ground already (the band's own reflectance) — the deepened lift +
    // boosted dissolve read on the absorbed channels (G/B rise as more red mass is pulled out).
    assert!(
        i32::from(h[1]) > i32::from(q[1]) + 6,
        "2 s of dwell must deepen the lift under the nib: held {h:?} vs quick {q:?}"
    );
    // (b) Wider bleed: the dissolved red reaches farther LEFT of the band (into the wash) after
    // the hold — measure the farthest x (walking away from the band edge at 52) still tinted.
    let redness = |t: &PainterTool, x: u32| {
        let c = px(t, size, x, 64);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    let extent = |t: &PainterTool| {
        // Baseline: wash 18 px from the band — beyond even the 2× (soaked) blur radius.
        let base = redness(t, 34);
        // Walk AWAY-from-band → band edge; the FIRST tinted x is the farthest reach of the bleed.
        let mut e = 0u32;
        for x in 35..52u32 {
            if redness(t, x) > base + 10 {
                e = 52 - x;
                break;
            }
        }
        e
    };
    let eq = extent(&quick);
    let eh = extent(&held);
    // Also compare the total tint MASS beyond the band edge — the first-crossing extent is
    // threshold-granular, the mass meter sees the whole widened profile.
    let mass = |t: &PainterTool| {
        let base = redness(t, 34);
        (40..52u32)
            .map(|x| (redness(t, x) - base).max(0))
            .sum::<i32>()
    };
    let (mq, mh) = (mass(&quick), mass(&held));
    // Margin 1.15: measured +21% at the default knobs (SOAK_DISSOLVE doubles the tint under full
    // soak; the deepened lift is asserted above) — deterministic engine, so no flake headroom
    // needed. The perceptual tuning surface is the named SOAK_* consts (doc 11 §5 F3).
    assert!(
        eh >= eq && mh as f32 >= mq as f32 * 1.15,
        "2 s of dwell must push the dissolved tint farther/heavier into the wash:          held {eh}px/mass {mh} vs quick {eq}px/mass {mq}"
    );
}

/// **Spread clears the centre of the pool** (Enio 2026-07-07): a wet pool's interior LIGHTENS as the
/// pigment migrates to the receding front, and — the recovered dynamic — the clearing gets STRONGER
/// with Spread (a wider wet front empties the centre more). Before the fix, raising the cap to 48 let
/// Spread exceed the pool radius, `inner = blur(cov)` never saturated, and the edge term FLOODED the
/// centre (flat dark blob). The `core_r` cap + Spread-scaled thinning restore + strengthen it.
#[test]
fn watercolor_spread_clears_the_pool_centre() {
    fn centre_vs_rim(spread: f32) -> (i32, i32) {
        let size = 200u32;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: 34.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.20, 0.35, 0.75],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 2.0,
            edge_spread: spread,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.5,
            depth: 2.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Down)));
        assert!(t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Up)));
        let lum = |c: [u8; 4]| {
            (0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32) as i32
        };
        // Centre (x=100) vs a rim sample well inside the pool's dark ring (x=118, ~18px out).
        (lum(px(&t, size, 100, 100)), lum(px(&t, size, 118, 100)))
    }
    let (c48, r48) = centre_vs_rim(48.0);
    let (c16, _) = centre_vs_rim(16.0);
    // (a) At high Spread the centre is LIGHTER than the rim (the pool clears, not floods).
    assert!(
        c48 > r48 + 40,
        "high Spread must clear the pool centre: centre {c48} vs rim {r48}"
    );
    // (b) The clearing SCALES with Spread — Spread 48 clears the centre more than Spread 16.
    assert!(
        c48 > c16 + 20,
        "the clearing must strengthen with Spread: centre@48 {c48} vs centre@16 {c16}"
    );
}

/// **High-Spread live cost stays bounded** (Enio 2026-07-07 FPS fix): the rewet blur fields
/// downsample at wide Spread (`RewetFields`, `ds > 1`) + the no-Wet window uses the capped feather
/// reach, so a Spread-48 stroke's per-frame recomposite is a small multiple of the Spread-8 cost,
/// NOT the ~9× the full-res spread²-window path cost (measured 10.3 → 3.0 ms @2048²). Asserts the
/// SHAPE of the scaling (ratio), not an absolute ms — deterministic, machine-independent.
#[test]
#[ignore] // release-only timing; run with `--release -- --ignored`
fn watercolor_high_spread_frame_cost_bounded() {
    fn live_ms(spread: f32, dwell: bool) -> f64 {
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
            edge_spread: spread,
            granulation: 0.4,
            warp: 3.0,
            fill: 0.5,
            depth: 2.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        let y = 1024.0f32;
        assert!(t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down)));
        let n = 80usize;
        let mut ms = Vec::with_capacity(n);
        for i in 0..n {
            let x = 100.0 + (i as f32 + 1.0) * 4.0;
            if dwell {
                for _ in 0..3 {
                    t.paint_tick(0.033);
                }
            }
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        ms.iter().sum::<f64>() / ms.len() as f64
    }
    let lo = live_ms(8.0, false);
    let hi = live_ms(48.0, true);
    eprintln!(
        "live spread=8 {lo:.3} ms · spread=48+dwell {hi:.3} ms · ratio {:.1}",
        hi / lo
    );
    // The old spread²-window path made this ratio ~12×; the downsample + capped reach keep it low.
    assert!(
        hi < lo * 8.0,
        "high-Spread dwell frame cost must stay a small multiple of the baseline: \
         {hi:.3} ms vs {lo:.3} ms (ratio {:.1})",
        hi / lo
    );
}

/// **T5 (doc 11 §5 F2) — the Wet Mix carries picked-up colour downstream.** A wet mixer brush
/// (Charge < 1, some Pull) crossing a dry RED band on white picks the red up and drags it along the
/// gesture: downstream of the band the deposited stroke is redder than the same stroke with the mixer
/// OFF (Charge 1), and the carried red DECAYS with distance as the brush resamples the white beyond.
#[test]
fn watercolor_wet_mix_carries_colour_downstream() {
    fn run(charge: f32) -> PainterTool {
        let size = 160u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (44..56).contains(&x) {
                    [210u8, 30, 30, 255] // dry red band
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 7.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.20, 0.35, 0.75], // blue brush — carried red reads as a purple shift
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0, // isolate the mixer from edge pooling
            edge_spread: 4.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.6,
            depth: 1.5,
            wet_rewet: 0.0, // isolate the mixer from the per-pixel rewet
            wet_charge: charge,
            wet_pull: 0.6,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([16.0, 80.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 130.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Up)));
        t
    }
    let size = 160u32;
    let mixed = run(0.2); // pickup 0.8
    let plain = run(1.0); // mixer off
    let redness = |t: &PainterTool, x: u32| {
        let c = px(t, size, x, 80);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    // (a) Downstream (x=70, just past the band) the mixer stroke carries red the plain one lacks.
    assert!(
        redness(&mixed, 70) > redness(&plain, 70) + 15,
        "the mixer must carry the band's red downstream: mixed {} vs plain {}",
        redness(&mixed, 70),
        redness(&plain, 70)
    );
    // (b) The carried red DECAYS with distance (near the band > far from it).
    assert!(
        redness(&mixed, 70) > redness(&mixed, 110) + 8,
        "the carried colour must decay downstream: near {} vs far {}",
        redness(&mixed, 70),
        redness(&mixed, 110)
    );
}

/// **T6 (doc 11 §5 F2) — Charge controls the pickup amount.** A lower Charge (more depleted reserve)
/// blends MORE of the picked-up surface into the deposit: painting a blue mixer stroke straight over
/// a red field, a low Charge deposits a redder (more mixed) result than a high Charge.
#[test]
fn watercolor_wet_mix_charge_controls_pickup() {
    fn run(charge: f32) -> PainterTool {
        let size = 96u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for px4 in src.as_chunks_mut::<4>().0.iter_mut() {
            px4.copy_from_slice(&[210, 30, 30, 255]); // dry red everywhere
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.30, 0.80],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            edge_spread: 4.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.6,
            depth: 1.5,
            wet_rewet: 0.0,
            wet_charge: charge,
            wet_pull: 0.3,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
        let mut x = 16.0f32;
        while x < 80.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
        }
        assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
        t
    }
    let size = 96u32;
    let redness = |t: &PainterTool| {
        let c = px(t, size, 60, 48);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    let low = run(0.1); // heavy pickup
    let high = run(0.9); // light pickup
    assert!(
        redness(&low) > redness(&high) + 15,
        "lower Charge must pick up more of the red surface: low {} vs high {}",
        redness(&low),
        redness(&high)
    );
}

/// **T7 (doc 11 §5 F2) — the mixer is inert at the default Charge = 1.** With a full fresh reserve
/// the brush deposits pure fresh colour: a blue stroke straight over a red field stays BLUE (no red
/// picked up), regardless of Pull — the byte-identical-default guarantee (the mixer path is skipped).
#[test]
fn watercolor_wet_mix_default_charge_deposits_pure_colour() {
    let size = 96u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for px4 in src.as_chunks_mut::<4>().0.iter_mut() {
        px4.copy_from_slice(&[210, 30, 30, 255]);
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.10, 0.25, 0.85],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.7,
        depth: 2.0,
        wet_rewet: 0.0,
        wet_charge: 1.0, // default → mixer OFF
        wet_pull: 0.8,   // even with Pull set, no pickup at full charge
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([16.0, 48.0], PointerPhase::Down)));
    let mut x = 16.0f32;
    while x < 80.0 {
        x += 3.0;
        t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([x, 48.0], PointerPhase::Up)));
    let c = px(&t, size, 60, 48);
    // Deposit is blue: B channel dominates, no red carried up from the field.
    assert!(
        c[2] > c[0] + 40,
        "Charge 1 must deposit pure fresh blue (mixer off), got {c:?}"
    );
}

/// **Wet Mix exit bleed mirrors the entry** (Enio 2026-07-07, photo). A wet mixer brush (Charge < 1,
/// Pull 0) drawn ACROSS a painted pool picks its colour up; the ENTRY into the pool always bled the
/// picked-up colour into the incoming stroke, but the EXIT was a HARD CUT — the reservoir reset to the
/// bare surface the instant the centre left, and the following dabs overwrote the carried colour
/// (source-over recency). The asymmetric load/unload reservoir (fast load, slow unload) makes the
/// picked-up colour LINGER past the pool, so the exit bleeds too. Asserts the exit is no longer a hard
/// cut (bleeds red near the pool, fading with distance) and that its red EXTENT is comparable to the
/// entry's — not a perfect mirror (the entry deposits at full pickup, the exit is a fading carry), but
/// a real symmetric-looking bleed on both sides.
#[test]
fn watercolor_wet_mix_exit_bleed_mirrors_entry() {
    let size = 160u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    let (band0, band1) = (55u32, 105u32); // wide red pool
    for y in band0..band1 {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[210, 30, 30, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 11.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.30, 0.80],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.15,
        wet_pull: 0.0, // the reported Charge-only case
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 140.0 {
        y += 3.0;
        t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([80.0, y], PointerPhase::Up)));
    let redness = |yy: u32| {
        let c = px(&t, size, 80, yy);
        i32::from(c[0]) - (i32::from(c[1]) + i32::from(c[2])) / 2
    };
    // (a) The EXIT (below the pool) bleeds red near the edge — NOT the old flat blue (~ −113).
    let exit_near = redness(band1 + 1);
    assert!(
        exit_near > 15,
        "the exit must bleed the picked-up red past the pool (was a hard cut): {exit_near}"
    );
    // (b) The exit bleed FADES with distance (a gradient, not a slab).
    assert!(
        redness(band1 + 1) > redness(band1 + 7) + 15,
        "the exit bleed must fade with distance: near {} vs far {}",
        redness(band1 + 1),
        redness(band1 + 7)
    );
    // (c) Both sides bleed red over a comparable EXTENT (the reach mirrors, even if the entry — at
    //     full pickup — peaks higher than the fading exit carry). Count rows still red (> 8) each way.
    let entry_reach = (1..15).filter(|&d| redness(band0 - d) > 8).count();
    let exit_reach = (1..15).filter(|&d| redness(band1 + d) > 8).count();
    assert!(
        exit_reach >= 3 && exit_reach + 3 >= entry_reach,
        "the exit red reach must be comparable to the entry (mirror), entry {entry_reach} exit {exit_reach}"
    );
}

/// **Wet Mix carried colour is saturated, not watery** (Enio 2026-07-07). The mixer's disc pickup
/// averaged the RAW surface colour, so a brush half over a red pool picked up a pink AVERAGE of red +
/// white — the carried mix read bleached toward white instead of a rich blue+red purple. Presence-
/// weighting the sample (bare ground contributes to the weight, not the hue) picks up SATURATED red,
/// so the carried mix is a real purple. Asserts the carried region downstream of a red pool is
/// purple (R and B both well above G), not a pale near-grey.
#[test]
fn watercolor_wet_mix_carried_colour_is_saturated_not_watery() {
    let size = 160u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    let (band0, band1) = (55u32, 95u32);
    for y in band0..band1 {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[210, 30, 30, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.30, 0.80],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.2,
        wet_pull: 0.6,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([80.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 145.0 {
        y += 3.0;
        t.on_canvas_pointer(cp([80.0, y], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([80.0, y], PointerPhase::Up)));
    // Just past the pool: a rich purple (R and B each clearly above G), not a pale near-grey wash.
    // Margins re-pinned WITH the W-A decision (doc 12 OPT-1, 2026-07-07): the subtractive
    // (absorbance-space) mix legitimately weights the heavily-carried RED pigment more than the old
    // sRGB lerp did — deep maroon-purple (measured (166,67,92): R−G=99, B−G=25), exactly how real
    // red+blue pigment mixes. The test's INTENT is unchanged: G suppressed on both sides + strongly
    // chromatic (the original bug was a pale wash bleached toward white).
    let c = px(&t, size, 80, band1 + 3);
    let (r, g, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
    assert!(
        r > g + 60 && b > g + 15,
        "the carried mix must be a saturated purple (R,B > G), not watery: {c:?}"
    );
}

/// **Wet Mix deposit priority: a low-pickup dab can't wash out a high-pickup one** (Enio 2026-07-07).
/// The mixer scales each dab's colour-deposit alpha by its pickup strength, so a bare-ground dab
/// (leaving a pool) barely writes and cannot overwrite the picked-up colour laid by the in-pool dabs.
/// Reproduces the reported crossing: a blue mixer stroke drawn through a red pool — the pool's EXIT
/// edge must stay coloured (the picked-up red survives the exiting dabs), not wash back to plain blue.
/// (Some entry>exit difference is inherent to a DIRECTIONAL smudge — entering a pool ≠ leaving it —
/// but the exit must retain a clear share of the pickup, not be a hard cut.)
#[test]
fn watercolor_wet_mix_exit_edge_keeps_pickup() {
    let size = 200u32;
    let mut src = vec![255u8; (size * size * 4) as usize];
    let (b0, b1) = (78u32, 122u32); // red band rows
    for y in b0..b1 {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[210, 30, 30, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 20.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.15, 0.30, 0.80],
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.2,
        wet_pull: 0.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([100.0, 30.0], PointerPhase::Down)));
    let mut y = 30.0f32;
    while y < 175.0 {
        y += 3.0;
        t.on_canvas_pointer(cp([100.0, y], PointerPhase::Move));
    }
    assert!(t.on_canvas_pointer(cp([100.0, y], PointerPhase::Up)));
    let purple = |yy: u32| {
        let c = px(&t, size, 100, yy);
        i32::from(c[0]) + i32::from(c[2]) - 2 * i32::from(c[1])
    };
    let entry = purple(b0 + 4); // entry edge (into the pool from the top)
    let exit = purple(b1 - 4); // exit edge (leaving the pool at the bottom)
    // The exit edge keeps a clear majority of the entry's pickup (was a near-hard-cut before the
    // priority deposit + asymmetric reservoir).
    assert!(
        exit > 60 && exit * 3 >= entry * 2,
        "the exit edge must retain the picked-up colour (not wash to blue): entry {entry} exit {exit}"
    );
}

/// **W-A (doc 12 OPT-1) — the subtractive-mixing discriminant: blue over yellow makes GREEN, not
/// grey.** A blue mixer brush (Charge 0.3) crossing a dry YELLOW pool must deposit a GREEN-dominant
/// mix at the pool's exit (absorbance-space lerp = pigment mixing). The sRGB lerp this replaced
/// deposited khaki/grey — R≈G, measured (128,128,115) pre-fix — the exact "blue and yellow make
/// gray" defect the Mixbox paper names as the flagship failure of RGB-mixing paint software.
#[test]
fn watercolor_wet_mix_blue_over_yellow_deposits_green() {
    let size = 160u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            let p = if (44..70).contains(&x) {
                [250u8, 220, 40, 255] // dry yellow pool
            } else {
                [255u8, 255, 255, 255]
            };
            src[i..i + 4].copy_from_slice(&p);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 7.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.10, 0.20, 0.70], // blue
        space_attenuation: false,
        watercolor: true,
        edge_gain: 0.0,
        edge_spread: 4.0,
        granulation: 0.0,
        warp: 0.0,
        fill: 0.6,
        depth: 1.5,
        wet_rewet: 0.0,
        wet_charge: 0.3,
        wet_pull: 0.6,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([16.0, 80.0], PointerPhase::Down)));
    let mut x = 16.0f32;
    while x < 130.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([130.0, 80.0], PointerPhase::Up));
    // At the pool's exit edge the deposited colour must be GREEN-dominant (pigment mix of blue +
    // yellow), with a real margin — the grey failure mode had R ≈ G.
    // (x = 66/74 sit at the pool's trailing edge, where the carried mix peaks; farther out the
    // carry is already decaying toward the brush's blue and the G margin thins by design.)
    for probe_x in [66u32, 74] {
        let i = ((80 * size + probe_x) * 4) as usize;
        let c = &t.paint.stroke_color[i..i + 4];
        let (r, g, b) = (i32::from(c[0]), i32::from(c[1]), i32::from(c[2]));
        assert!(
            g > r + 10 && g > b + 10,
            "blue over yellow must deposit GREEN at the pool exit (x={probe_x}): rgba=({r},{g},{b})"
        );
    }
    // Far downstream the carry decays and the deposit returns toward the BRUSH's blue.
    let i = ((80 * size + 100) * 4) as usize;
    let c = &t.paint.stroke_color[i..i + 4];
    assert!(
        c[2] > c[0],
        "downstream the carry decays back toward the blue brush: rgba={c:?}"
    );
}

/// **MIX-1 (doc 12, W-C): Charge DEPLETA com a distância do traço** — a assinatura nº 1 do
/// Procreate (Handbook: "the longer you drag your stroke out... the trail of color it leaves will
/// become fainter"). Um traço LONGO em canvas branco com Charge baixo (reserva curta, nada a
/// captar no branco) precisa desbotar: a cauda deposita menos que a cabeça. Charge = 1 (default)
/// pula o mixer inteiro — byte-idêntico, coberto pela suíte.
#[test]
fn watercolor_wet_mix_charge_depletes_along_the_stroke() {
    let size = 256u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
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
        granulation: 0.0,
        wet_charge: 0.25, // short reserve; white canvas ⇒ nothing to pick up ⇒ pure depletion
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([12.0, 128.0], PointerPhase::Down)));
    let mut x = 12.0f32;
    while x < 240.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 128.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([240.0, 128.0], PointerPhase::Up));
    let mean_g = |x0: u32, x1: u32| -> f32 {
        let mut acc = 0.0f32;
        for x in x0..x1 {
            acc += f32::from(px(&t, size, x, 128)[1]);
        }
        acc / (x1 - x0) as f32
    };
    let head = mean_g(20, 60); // fresh reserve
    let tail = mean_g(190, 230); // depleted
    assert!(
        tail > head + 15.0,
        "the trail must fade as the Charge depletes (head G {head:.1} vs tail G {tail:.1} — lighter = fainter)"
    );
}

/// **MIX-1 regressão (Enio smoke 2026-07-08):** um pincel ESGOTADO cruzando uma poça deposita
/// proporcional às DUAS intensidades — (a) poça PÁLIDA ⇒ quase nada ("explode em muito pigmento"
/// era o `depl = max(fresh, t)` com `t` = peso de mistura, que salta pra ~1 em qualquer poça);
/// (b) poça RICA ⇒ o smudge continua vivo (o fix não pode matar o carry). E (c) a CABEÇA de um
/// traço com Charge baixo mantém a anatomia completa (reserva começa em 1.0 — Charge controla a
/// duração, nunca a intensidade inicial: escalar a cobertura inundava o interior com edge residual).
#[test]
fn watercolor_wet_mix_depleted_brush_respects_pool_intensity() {
    let size = 256u32;
    let spec = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 0.0,
        edge_spread: 4.0,
        warp: 0.0,
        granulation: 0.0,
        ..Default::default()
    };
    let stroke_v = |t: &mut PainterTool, x: f32| {
        assert!(t.on_canvas_pointer(cp([x, 12.0], PointerPhase::Down)));
        let mut y = 12.0f32;
        while y < 240.0 {
            y += 2.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 240.0], PointerPhase::Up));
    };
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = spec;
    // Pale pool at x∈[40..110], rich pool at x∈[150..220], both horizontal at y = 200 — far enough
    // down that a vertical Charge-0.1 stroke (span ≈ 107 px) arrives fully depleted (travel ≈ 188).
    t.paint.brush.color = [1.0, 0.78, 0.78]; // pale pink: little pigment to pick up
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([40.0, 200.0], PointerPhase::Down)));
    let mut x = 40.0f32;
    while x < 110.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 200.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([110.0, 200.0], PointerPhase::Up));
    t.paint.brush.color = [0.75, 0.05, 0.05]; // rich red: a real reservoir to smudge from
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([150.0, 200.0], PointerPhase::Down)));
    let mut x = 150.0f32;
    while x < 220.0 {
        x += 2.0;
        t.on_canvas_pointer(cp([x, 200.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([220.0, 200.0], PointerPhase::Up));
    // Two depleted blue crossings (Charge 0.1), one through each pool.
    t.paint.brush.color = [0.15, 0.25, 0.7];
    t.paint.brush.wet_charge = 0.1;
    t.paint.brush_by_mode.fill(t.paint.brush);
    stroke_v(&mut t, 75.0); // through the PALE pool
    stroke_v(&mut t, 185.0); // through the RICH pool
    let g = |x: u32, y: u32| f32::from(px(&t, size, x, y)[1]);
    let head = g(75, 16); // (c) stroke head: full fresh reserve — a strong mark
    assert!(
        head < 140.0,
        "the head of a low-Charge stroke must open at FULL reserve (G {head:.1} — dark = strong)"
    );
    let bare_trail = g(75, 160); // depleted, outside any pool: ~plain water
    let pale_cross = g(75, 200); // (a) depleted × pale pool
    let pale_pool = g(55, 200); // the pale pool away from the crossing
    assert!(
        pale_cross > pale_pool - 45.0,
        "a depleted brush over a PALE pool must not explode with pigment (pool G {pale_pool:.1} → crossing G {pale_cross:.1})"
    );
    assert!(
        pale_cross > head + 20.0,
        "…and deposits far less than the brush's own fresh head (head G {head:.1}, crossing G {pale_cross:.1})"
    );
    let rich_cross = g(185, 200); // (b) depleted × rich pool
    let below_rich = g(185, 226); // just past the rich pool: the carried smudge trails out
    assert!(
        below_rich < bare_trail - 8.0,
        "crossing a RICH pool must re-ink the depleted brush (trail after pool G {below_rich:.1} vs bare trail G {bare_trail:.1})"
    );
    assert!(
        rich_cross < pale_cross,
        "the smudge tracks the pool's intensity (rich crossing G {rich_cross:.1} < pale crossing G {pale_cross:.1})"
    );
}
