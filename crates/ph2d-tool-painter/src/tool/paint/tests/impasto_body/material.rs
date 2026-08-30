//! **O MATERIAL da tinta.** Rugosidade, metálico e cera: o que cada um faz ao brilho e à sombra, o
//! filtro de cera que aquece o interior da tinta pálida, o ajuste do último traço (e o portão que
//! decide se os sliders chegam à tela), e o que o undo repõe por baixo.

use super::*;

// ── The paint's MATERIAL (Enio, 2026-07-13) ────────────────────────────────────────────────────────
//
// Four knobs, and the section's own history says what to fear: a knob that is wired, painted, routed
// and does NOTHING (§17 — Shine died that way once already, and the Amount knob before it). So there
// is one gate per knob asserting it MOVES the picture, and one asserting the contract survives all of
// them: flat paint is byte-identical at EVERY material, or the whole pass is a lie.

/// The same harness with WHITE paint — for the gates about what the FILTER does, where a coloured
/// pigment would lend the scattered light a colour of its own and the filter's contribution could not
/// be told apart from the paint's. White pigment eats nothing, so whatever colour comes back is the
/// filter's alone.
#[cfg(test)]
fn impasto_material_render_white(size: u32, edit: &dyn Fn(&mut PainterTool)) -> LitCanvas {
    impasto_material_render_with(size, [1.0, 1.0, 1.0], edit)
}

/// A red stroke with relief, lit, under a material the caller dials. The harness the four gates below
/// share, so a knob cannot pass by being measured differently from its neighbours.
/// `(pixels lit, the relief, the coverage)` — what the material gates read.
#[cfg(test)]
type LitCanvas = (Vec<u8>, Vec<f32>, Vec<u8>);

/// A named material to sweep: how to dial it, through the REAL setters.
#[cfg(test)]
type MaterialCase<'a> = (&'a str, &'a dyn Fn(&mut PainterTool));

#[cfg(test)]
fn impasto_material_render(size: u32, edit: &dyn Fn(&mut PainterTool)) -> LitCanvas {
    // Red paint: a metal's glint has somewhere to go, and the scattered light has a colour to wear.
    impasto_material_render_with(size, [0.9, 0.1, 0.1], edit)
}

#[cfg(test)]
fn impasto_material_render_with(
    size: u32,
    color: [f32; 3],
    edit: &dyn Fn(&mut PainterTool),
) -> LitCanvas {
    let mut t = impasto_canvas(size);
    // The caller dials the material through the REAL setters, AFTER the stroke — which only reaches the
    // paint on the canvas while "Adjust Last Stroke" is on. These gates are about the material, so the
    // harness states that premise rather than inheriting a default that is the artist's to move (it went
    // OFF on 2026-07-19).
    t.paint.impasto_live_edit = true;
    let mut b = t.paint.brush;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.radius_px = 40.0;
    b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
    b.color = color;
    b.impasto_source = DepthSource::Grain;
    b.impasto_smoothing = 0.15;
    b.texture.kind = ph2d_painter_brush::TextureKind::Noise;
    b.texture.mapping = ph2d_painter_brush::TextureMapping::Tiled;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.paint.impasto_rig.lights[0].angle_deg = 90;
    t.paint.impasto_rig.lights[0].elev_deg = 45;
    t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
    for i in 1..=8 {
        t.on_canvas_pointer(cp(
            [40.0 + 10.0 * f32::from(i as u8), 80.0],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
    // Dialled AFTER the stroke, through the REAL setters — which is the gesture that matters (the
    // artist lays a stroke and then dials it in while looking at it), and it is the one that exercises
    // the live material re-bake. Setting the fields before the stroke would test the deposit and quietly
    // leave the re-bake — the harder half — ungated.
    edit(&mut t);
    let img = lit(&mut t);
    let h = relief(&t);
    let active = t.layers.active().expect("a layer");
    let cov = t
        .covers
        .get(&active)
        .map(|c| c.as_ref().clone())
        .unwrap_or_default();
    (img, h, cov)
}

/// **The contract survives every material.** Flat paint — the plateau, where the gradient is zero —
/// comes out BYTE-IDENTICAL whatever the paint is made of.
///
/// **What it pins, exactly** — and it is worth being precise, because the first version of this comment
/// was wrong and the mutation run said so. A pixel with a zero gradient takes `shade`'s EARLY-OUT, so
/// what this gate defends is that no material knob can make the pass touch flat ground *at all*: not by
/// creating a slope where there is none (a material plane that bled into bare paper would), not by
/// adding an ambient term outside the relative path, not by letting the glint escape the `flat_spec`
/// subtraction. It is a real guard, and it is a guard for the FUTURE shape of this code.
///
/// **What it canNOT catch, so do not trust it to:** a broken flat DIVISOR. Drop the `wrapped_ndl` from
/// `MatShade::resolve`'s flat response and this gate stays GREEN — the pixels that would expose it are
/// sloped, and sloped pixels never reach here. That bug belongs to
/// [`wax_lights_the_far_flank_without_lifting_flat_paint`], which measures the COMPRESSION the wrap
/// must produce, and which does bleed on it. (Two gates, two claims. A gate whose comment promises a
/// mutation it does not catch is worse than no gate, because it retires the question.)
#[test]
fn flat_paint_is_byte_identical_at_every_material() {
    let size = 160u32;
    // The reference: the pass with no light at all. Every material must leave the flat pixels there.
    let (unlit, _, _) = impasto_material_render(size, &|t| {
        t.paint.impasto_show = false;
        t.invalidate_composite();
    });
    let cases: [MaterialCase; 5] = [
        ("glossy", &|t: &mut PainterTool| {
            t.set_impasto_roughness(0.0)
        }),
        ("matte", &|t: &mut PainterTool| t.set_impasto_roughness(1.0)),
        ("metal", &|t: &mut PainterTool| t.set_impasto_metallic(1.0)),
        ("waxy", &|t: &mut PainterTool| t.set_impasto_wax(1.0)),
        ("everything", &|t: &mut PainterTool| {
            t.set_impasto_shine(1.0);
            t.set_impasto_roughness(0.85);
            t.set_impasto_metallic(1.0);
            t.set_impasto_wax(1.0);
        }),
    ];
    for (name, edit) in cases {
        let (lit_img, h, _) = impasto_material_render(size, edit);
        let w = size as usize;
        let mut checked = 0u32;
        for y in 1..(size as usize - 1) {
            for x in 1..(size as usize - 1) {
                // FLAT = the central difference is exactly zero in both axes, which is what the pass
                // itself early-outs on. Read the relief the same way the pass does, or the oracle is
                // testing a different pixel than the code is.
                let dhx = h[y * w + x + 1] - h[y * w + x - 1];
                let dhy = h[(y + 1) * w + x] - h[(y - 1) * w + x];
                if dhx != 0.0 || dhy != 0.0 {
                    continue;
                }
                checked += 1;
                for c in 0..3 {
                    assert_eq!(
                        lit_img[(y * w + x) * 4 + c],
                        unlit[(y * w + x) * 4 + c],
                        "material '{name}' moved a FLAT pixel at ({x}, {y}) channel {c} — the pass is \
                         RELATIVE to a flat surface OF THE SAME MATERIAL, so flat paint must come out \
                         byte-identical at any material. It is the contract the whole section stands on."
                    );
                }
            }
        }
        assert!(
            checked > 1000,
            "material '{name}': only {checked} flat pixels examined — the gate is vacuous"
        );
    }
}

/// **Roughness is not decorative: it changes the WIDTH of the glint.**
///
/// The knob that did not exist — the exponent was `SHININESS = 24`, welded shut. A glossy paint puts
/// its light into FEW, BRIGHT pixels (a tight lobe on the crests); a matte one spreads the same light
/// over MANY, dimmer ones. So the count of brightly-glinting pixels must GROW with roughness while the
/// brightest pixel gets no brighter — which is the difference between a broader lobe and simply more
/// light, and it is the thing a fake "roughness" (a second gain on Shine) could not fake.
#[test]
fn roughness_broadens_the_glint_instead_of_brightening_it() {
    let size = 160u32;
    let glint_count = |roughness: f32| -> (u32, i32) {
        let (matte_ref, _, _) = impasto_material_render(size, &move |t| {
            t.set_impasto_shine(0.0);
            t.set_impasto_roughness(roughness);
        });
        let (glossy, _, _) = impasto_material_render(size, &move |t| {
            t.set_impasto_shine(1.0);
            t.set_impasto_roughness(roughness);
        });
        // How many pixels the glint LIFTED, and by how much at its peak. Measured against the SAME
        // canvas with Shine 0, so the diffuse (which roughness does not touch) cancels exactly and what
        // is left is the highlight alone.
        let (mut n, mut peak) = (0u32, 0i32);
        for p in 0..(size * size) as usize {
            let mut best = 0i32;
            for c in 0..3 {
                best = best.max(i32::from(glossy[p * 4 + c]) - i32::from(matte_ref[p * 4 + c]));
            }
            if best >= 8 {
                n += 1;
            }
            peak = peak.max(best);
        }
        (n, peak)
    };
    let (tight_n, tight_peak) = glint_count(0.0);
    let (broad_n, broad_peak) = glint_count(1.0);
    assert!(
        tight_peak > 0 && broad_peak > 0,
        "sanity: the glint must exist at BOTH roughnesses (tight {tight_peak}, broad {broad_peak}) — \
         a gate on a highlight that is not there proves nothing"
    );
    assert!(
        broad_n > tight_n,
        "a rougher paint must spread its highlight over MORE pixels: matte lit {broad_n}, glossy lit \
         {tight_n}. Roughness is the WIDTH of the lobe; if this does not move, the exponent is still \
         welded shut and the knob is decoration."
    );
}

/// **Metallic makes the glint take the PAINT's colour, not the lamp's.**
///
/// The one line that separates a conductor from a dielectric in every PBR model, and the reason gold
/// leaf glints gold under a white light while white paint glints white. Under a WHITE lamp on RED paint:
/// a dielectric highlight pushes every channel up together (it drives the pixel toward white, which is
/// what desaturates it); a metal's pushes mostly RED, so the pigment SURVIVES the highlight.
#[test]
fn metallic_gives_the_glint_the_paints_own_colour() {
    let size = 160u32;
    // Same rig, same stroke, same Shine — only the conductor/dielectric line moves.
    let (dielectric, _, _) = impasto_material_render(size, &|t| {
        t.set_impasto_shine(1.0);
        t.set_impasto_metallic(0.0);
    });
    let (metal, _, _) = impasto_material_render(size, &|t| {
        t.set_impasto_shine(1.0);
        t.set_impasto_metallic(1.0);
    });
    // Chroma = how much RED the pixel still has over its BLUE. A white highlight destroys it (both
    // channels climb toward the ceiling together); a red one preserves it.
    let chroma = |img: &[u8], p: usize| i32::from(img[p * 4]) - i32::from(img[p * 4 + 2]);
    let (mut moved, mut metal_keeps_more) = (0u32, 0u32);
    for p in 0..(size * size) as usize {
        let (cd, cm) = (chroma(&dielectric, p), chroma(&metal, p));
        if cd == cm {
            continue;
        }
        moved += 1;
        if cm > cd {
            metal_keeps_more += 1;
        }
    }
    assert!(
        moved > 200,
        "Metallic moved only {moved} pixels — the knob is inert, which is exactly the species this \
         section keeps shipping (§17)"
    );
    // Not "every pixel", because in shadowed pixels the glint is zero and the two are equal there; the
    // claim is about the DIRECTION wherever it does move.
    assert!(
        metal_keeps_more * 10 >= moved * 9,
        "where Metallic changes a pixel, it must PRESERVE the paint's chroma (its highlight is red, not \
         white): only {metal_keeps_more} of {moved} changed pixels kept more red. A metal that bleaches \
         its own colour is a dielectric with extra steps."
    );
}

/// **Wax lights the shadowed flank — and brightens nothing that is flat.**
///
/// The honest half of "SSS": light wrapping around the terminator, which is what makes wax and thick
/// oil read as soft. The trap it must not fall into is the one the whole pass exists to avoid — an
/// ABSOLUTE brightening. So the claim is precisely two-sided: the flank that FACES AWAY from the lamp
/// must come up, and (this is `flat_paint_is_byte_identical_at_every_material`'s job, asserted again
/// here in situ) the flat plateau must not move by a single level.
#[test]
fn wax_lights_the_far_flank_without_lifting_flat_paint() {
    let size = 160u32;
    let (dry, h, _) = impasto_material_render(size, &|t| {
        t.set_impasto_shine(0.0); // isolate the DIFFUSE: Wax is a diffuse term
        t.set_impasto_wax(0.0);
    });
    let (waxy, _, _) = impasto_material_render(size, &|t| {
        t.set_impasto_shine(0.0);
        t.set_impasto_wax(1.0);
    });
    let w = size as usize;
    let (mut shadowed_lifted, mut shadowed_seen, mut flat_moved) = (0u32, 0u32, 0u32);
    for y in 1..(size as usize - 1) {
        for x in 1..(size as usize - 1) {
            let p = y * w + x;
            let dhx = h[p + 1] - h[p - 1];
            let dhy = h[(y + 1) * w + x] - h[(y - 1) * w + x];
            let delta = i32::from(waxy[p * 4]) - i32::from(dry[p * 4]);
            if dhx == 0.0 && dhy == 0.0 {
                if delta != 0 {
                    flat_moved += 1;
                }
                continue;
            }
            // The lamp comes from azimuth 90°, so a flank whose normal tilts AWAY from it is in shadow.
            // That is where a wrapped diffuse has something to say, and where an unwrapped one has the
            // pixel bottomed out on the ambient floor.
            if dhy > 0.0 {
                shadowed_seen += 1;
                if delta > 0 {
                    shadowed_lifted += 1;
                }
            }
        }
    }
    assert_eq!(
        flat_moved, 0,
        "Wax lifted {flat_moved} FLAT pixels — the wrap must be applied to the flat DIVISOR too, or \
         every flat canvas brightens the moment the knob leaves zero"
    );
    assert!(
        shadowed_seen > 500,
        "only {shadowed_seen} shadowed pixels examined — vacuous"
    );
    assert!(
        shadowed_lifted * 2 > shadowed_seen,
        "Wax must LIGHT the flank turned away from the lamp (that IS the soft terminator): only \
         {shadowed_lifted} of {shadowed_seen} shadowed pixels came up. If this does not move, the knob \
         is decoration."
    );

    // ── And the OTHER half, which is the one with teeth: a wrap COMPRESSES. ──────────────────────────
    //
    // Wrapping raises the shadow AND lowers the highlight — it narrows the range, it does not shift it.
    // So the flank FACING the lamp must come out DARKER under Wax, never brighter.
    //
    // This is the assertion that pins the flat DIVISOR, and it is the reason it exists: if the divisor
    // is left unwrapped while the pixel is wrapped, the ratio is inflated EVERYWHERE and the whole
    // relief brightens — the absolute-shading bug the entire pass is built to avoid, arriving through a
    // new door. `flat_paint_is_byte_identical_at_every_material` cannot see it (flat pixels early-out
    // before the divisor is ever read — the mutation run proved that, which is why this is here).
    //
    // MUT proved RED: `let fd = l.dir[2].max(0.0)` in `MatShade::resolve` (the divisor without the
    // wrap) ⇒ the lit flank BRIGHTENS and this fires.
    let (mut lit_seen, mut lit_darkened) = (0u32, 0u32);
    for y in 1..(size as usize - 1) {
        for x in 1..(size as usize - 1) {
            let p = y * w + x;
            let dhy = h[(y + 1) * w + x] - h[(y - 1) * w + x];
            if dhy >= 0.0 {
                continue; // the flank TOWARD the lamp is the one whose highlight must compress
            }
            lit_seen += 1;
            if i32::from(waxy[p * 4]) <= i32::from(dry[p * 4]) {
                lit_darkened += 1;
            }
        }
    }
    assert!(
        lit_seen > 500,
        "only {lit_seen} lit pixels examined — vacuous"
    );
    assert!(
        lit_darkened * 20 >= lit_seen * 19,
        "Wax BRIGHTENED the flank facing the lamp ({} of {lit_seen} lit pixels went up) — a wrap \
         compresses the range, it does not lift it. The flat divisor is not being wrapped along with \
         the pixel, so the ratio is inflated everywhere and the relief is glowing.",
        lit_seen - lit_darkened
    );
}

/// **The light that scatters through the paint comes back wearing the paint's COLOUR.**
///
/// The signature of subsurface scattering, and the thing a monochrome wrap cannot fake: an ear against
/// the sun is RED, marble in shadow is WARM, wax is GOLDEN. The medium doing the absorbing IS the
/// pigment, so what comes back out is what the pigment did not eat — which means the tint is not a knob
/// anyone gets to pick. It is already in the pixel. (Enio, 2026-07-13: *"seria possível ter cor para
/// wax?"* — it is not merely possible: it is the half that makes it read as translucency rather than as
/// blurry shading.)
///
/// On RED paint under a WHITE lamp: where Wax lifts the shadowed flank, it must lift the RED channel
/// far more than the BLUE one. An untinted wrap lifts all three together — it makes the shadow PALER,
/// not WARMER, and the paint reads as plastic.
///
/// MUT proved RED: drop the `* albedo[c]` from the scattered term in `Rig::shade` ⇒ the channels lift
/// together and this fires.
#[test]
fn wax_bleeds_the_paints_own_colour_into_the_shadow() {
    let size = 160u32;
    // Shine 0 throughout: the glint is a separate term with its own gates, and a white highlight would
    // drown the very chroma this gate is measuring.
    let (dry, h, cov) = impasto_material_render(size, &|t| {
        t.set_impasto_shine(0.0);
        t.set_impasto_wax(0.0);
    });
    let (waxy, _, _) = impasto_material_render(size, &|t| {
        t.set_impasto_shine(0.0);
        t.set_impasto_wax(1.0);
    });
    let w = size as usize;
    let (mut seen, mut red_gained, mut blue_gained, mut worst_cold) = (0u32, 0i64, 0i64, 0i32);
    for y in 1..(size as usize - 1) {
        for x in 1..(size as usize - 1) {
            let p = y * w + x;
            // SOLID paint only: the film's screen-space AA (BUGS #16) leaves the rim texels pale
            // pink (fractional paint over white paper), whose near-neutral albedo gains red≈blue by
            // construction — a population that dilutes the aggregate the claim is measured on
            // (2.56:1 → 1.56:1) without touching the tint mechanism. The claim is about light
            // scattered through PAINT, so it is asserted where the paint is whole.
            if cov.get(p).copied().unwrap_or(0) != 255 {
                continue;
            }
            let dhy = h[(y + 1) * w + x] - h[(y - 1) * w + x];
            if dhy <= 0.0 {
                continue; // the flank turned AWAY from the lamp: where the scattered light shows
            }
            let d_red = i32::from(waxy[p * 4]) - i32::from(dry[p * 4]);
            let d_blue = i32::from(waxy[p * 4 + 2]) - i32::from(dry[p * 4 + 2]);
            if d_red == 0 && d_blue == 0 {
                continue; // Wax changed nothing here — it has no claim to make
            }
            seen += 1;
            red_gained += i64::from(d_red);
            blue_gained += i64::from(d_blue);
            worst_cold = worst_cold.max(d_blue - d_red);
        }
    }
    assert!(seen > 500, "only {seen} shadowed pixels moved — vacuous");
    // The claim is about the LIGHT, so it is asserted on the light: how much red the shadow gained
    // against how much blue. A per-pixel MAJORITY was the first form of this, and it was the wrong
    // instrument — it read 88% and failed a 95% bar on pixels that had gone the "wrong" way by ONE
    // level, which is `u8` rounding, not physics. The aggregate cannot be fooled by a rounding step.
    //
    // The bar is 2×, and it is MEASURED, not chosen: the tinted pass gives **2.56:1** and the untinted
    // mutation gives **1.20:1**. Note what that second number means — an untinted wrap ALSO ends up
    // with more red levels than blue, because the red pixel is brighter and the same multiplicative
    // lift buys more levels there. So "red gained more than blue" is NOT a discriminating claim, and a
    // gate that asserted only that would have passed on the bug. The threshold has to sit between the
    // two measurements, and it can only do that if both are measured.
    assert!(
        red_gained > blue_gained * 2,
        "the light Wax bleeds into the shadow of RED paint must be RED: the shadow gained \
         {red_gained} levels of red against {blue_gained} of blue. An untinted wrap lifts all three \
         channels together — it makes the shadow PALER, not WARMER: soft plastic, not wax."
    );
    // …and no pixel may go the other way by more than a rounding step. A tint that warmed the picture
    // ON AVERAGE while cooling parts of it would be a bug the aggregate alone would hide.
    assert!(
        worst_cold <= 1,
        "a shadowed pixel gained {worst_cold} more levels of BLUE than of RED — the scattered light is \
         not wearing the paint's colour there"
    );
}

/// **"Adjust Last Stroke" governs whether the sliders reach the paint already on the canvas.**
///
/// Enio, 2026-07-13: *"Slider para que o último traço pintado possa ser ajustado pelos sliders (como
/// está agora). Se desmarcado, os ajustes dos sliders só afetam os traços que ainda serão pintados."*
///
/// Both halves are asserted, because each one alone is a plausible bug: unticked and still editing (the
/// checkbox is decoration — this section's signature failure), or ticked and NOT editing (the live
/// re-derive that the whole Body card exists for, silently dead). And the third: the stroke's
/// ingredients must SURVIVE being unticked, or the toggle is destructive and ticking it back on can
/// never reach the stroke again.
#[test]
fn adjust_last_stroke_gates_whether_the_sliders_reach_the_canvas() {
    let size = 160u32;
    // Paint one stroke, then move a slider — with the box ticked, and with it unticked.
    let after_edit = |live_edit: bool| -> Vec<u8> {
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
        b.color = [0.9, 0.1, 0.1];
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
        // Set the flag EXPLICITLY rather than toggling off a default: the default is the artist's, and it
        // has already moved once (ON until 2026-07-19, OFF since). A fixture that reaches its state by
        // toggling silently inverts the day someone flips the default, and both halves of this gate would
        // still pass — testing the opposite claim.
        t.paint.impasto_live_edit = live_edit;
        // The gesture the artist makes: a stroke is down, now move the sliders. One from the Body card
        // (Depth re-derives the relief) and one from the Material card (Shine re-bakes the material) —
        // the two independent choke points, so a fix to one that forgets the other goes red.
        t.set_brush_impasto_depth(-0.9);
        t.set_impasto_shine(1.0);
        lit(&mut t)
    };
    let baseline = {
        // The canvas as the stroke LEFT it — no slider moved at all.
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
        b.color = [0.9, 0.1, 0.1];
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
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

    let ticked = after_edit(true);
    let unticked = after_edit(false);

    let moved = |a: &[u8], b: &[u8]| -> u32 {
        (0..(size * size) as usize)
            .filter(|p| (0..3).any(|c| a[p * 4 + c] != b[p * 4 + c]))
            .count() as u32
    };

    // TICKED — the historical behaviour: the sliders reach back and re-derive the stroke on the canvas.
    let changed = moved(&ticked, &baseline);
    assert!(
        changed > 500,
        "with 'Adjust Last Stroke' TICKED, moving Depth and Shine changed only {changed} pixels of the \
         stroke already painted — the live re-derive the whole Body card exists for is dead"
    );

    // UNTICKED — the paint on the canvas is FINISHED. Not one byte moves.
    let untouched = moved(&unticked, &baseline);
    assert_eq!(
        untouched, 0,
        "with 'Adjust Last Stroke' UNTICKED, moving the sliders still changed {untouched} pixels of the \
         stroke already painted. The checkbox is decoration — which is exactly the species this section \
         keeps shipping (§17)."
    );
}

/// **Out of the box, finished paint is finished** — "Adjust Last Stroke" is OFF by default (Enio,
/// 2026-07-19). The historical default was ON, which made dialling the brush in for the NEXT stroke
/// silently rewrite the one already on the canvas.
///
/// This asserts the BEHAVIOUR a fresh tool has, not the flag: nobody touches `impasto_live_edit` here, so
/// what it pins is what the artist meets. A gate on the boolean would go green again the day the field is
/// read by one more site that forgets to honour it. It is the twin of
/// `adjust_last_stroke_gates_whether_the_sliders_reach_the_canvas` — that one proves the switch WORKS in
/// both positions, this one proves which position it starts in.
#[test]
fn a_fresh_brush_does_not_adjust_the_last_stroke() {
    let size = 160u32;
    let paint = |t: &mut PainterTool| {
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.175;
        b.color = [0.9, 0.1, 0.1];
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
    };

    // The canvas as the stroke left it.
    let baseline = {
        let mut t = impasto_canvas(size);
        paint(&mut t);
        lit(&mut t)
    };
    // The same stroke, then the artist reaches for the sliders — with NOTHING configured.
    let after = {
        let mut t = impasto_canvas(size);
        paint(&mut t);
        t.set_brush_impasto_depth(-0.9);
        t.set_impasto_shine(1.0);
        lit(&mut t)
    };

    assert_eq!(
        after, baseline,
        "straight out of the box, moving Depth and Shine rewrote the stroke already on the canvas. \
         'Adjust Last Stroke' is meant to start UNTICKED (Enio 2026-07-19): dialling the brush in for \
         the next stroke must not reach back and edit the last one."
    );
}

/// …and the toggle is **NOT destructive**: unticking it, moving sliders, then ticking it back on must
/// leave the stroke exactly as reachable as it was. If the ingredients were dropped when the box was
/// cleared, the artist would have silently lost the ability to edit their stroke by clicking a checkbox
/// twice — a checkbox that quietly discards work is not a checkbox.
///
/// The claim carries no threshold, and it must not: the off→on cycle has to leave the canvas
/// **byte-identical** to the run that never toggled at all. (My first two attempts at this file's gates
/// both picked a pixel-count bar out of the air and both were wrong — 88% against a 95% bar, then 454
/// pixels against a 500 bar. A number you had to guess is a number the gate did not need.)
#[test]
fn adjust_last_stroke_does_not_destroy_the_strokes_ingredients() {
    let size = 160u32;
    let stroke = |t: &mut PainterTool| {
        let mut b = t.paint.brush;
        b.hardness = 0.0;
        b.falloff = Falloff::Smooth;
        b.radius_px = 40.0;
        b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
        b.color = [0.9, 0.1, 0.1];
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), 80.0],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, 80.0], PointerPhase::Up));
    };

    // The control: never toggled, one Shine edit.
    // Both runs START ticked, and say so explicitly — the claim is about the off→on CYCLE, so the state
    // it cycles from has to be pinned by the fixture and not inherited from a default that has moved once
    // already (ON until 2026-07-19, OFF since).
    let control = {
        let mut t = impasto_canvas(size);
        stroke(&mut t);
        t.paint.impasto_live_edit = true;
        t.set_impasto_shine(1.0);
        lit(&mut t)
    };
    // The gesture: untick, dial the brush in for the NEXT stroke, tick back on, then the same Shine edit.
    let cycled = {
        let mut t = impasto_canvas(size);
        stroke(&mut t);
        t.paint.impasto_live_edit = true;
        t.toggle_impasto_live_edit(); // OFF — the stroke on the canvas is finished
        t.set_brush_impasto_depth(-0.9); // …and this must not touch it
        t.toggle_impasto_live_edit(); // back ON
        t.set_impasto_shine(1.0); // …and now the sliders reach it again, in full
        lit(&mut t)
    };

    assert_eq!(
        cycled, control,
        "after unticking and re-ticking 'Adjust Last Stroke', the same Shine edit produced a DIFFERENT \
         canvas than the run that never toggled. Either the stroke's ingredients were thrown away when \
         the box was cleared (so there is no way back), or the Depth edit made while it was unticked \
         leaked into the paint anyway (so unticking does not mean what it says)."
    );
}

/// **The Wax filter reaches alabaster — a PALE surface with a WARM interior.**
///
/// The case the derived tint alone can never say, and the reason the swatch exists (Enio, 2026-07-13).
/// White paint scatters white: the pigment is the medium, and white pigment eats nothing. So with no
/// filter, a white stone can only ever be *soft* — never *warm inside*. Which is wrong about jade,
/// marble, alabaster and skin, i.e. about every material anyone reaches for Wax to paint.
///
/// WHITE paint, WHITE lamp, a WARM filter: the shadowed flank must come back WARM (red over blue). The
/// control is the same canvas with the filter open (white), where the shadow must stay NEUTRAL — that
/// second half is what makes this a gate on the FILTER rather than on the lamp or the pigment.
#[test]
fn the_wax_filter_gives_pale_paint_a_warm_interior() {
    let size = 160u32;
    let warmth = |img: &[u8], p: usize| i32::from(img[p * 4]) - i32::from(img[p * 4 + 2]);
    // White paint — the pigment has no colour of its own to lend the scattered light.
    let render = |filter: [f32; 3]| -> (Vec<u8>, Vec<f32>) {
        let (img, h, _) = impasto_material_render_white(size, &move |t| {
            t.set_impasto_shine(0.0); // the glint is a separate term with its own gates
            t.set_impasto_wax(1.0);
            t.set_impasto_wax_color(filter);
        });
        (img, h)
    };
    let (open, h) = render([1.0, 1.0, 1.0]); // the filter open: the physics, untouched
    let (warm, _) = render([1.0, 0.45, 0.2]); // a warm filter: the light picks up amber on the way

    let w = size as usize;
    let (mut seen, mut warmed, mut open_neutral) = (0u32, 0i64, 0i64);
    for y in 1..(size as usize - 1) {
        for x in 1..(size as usize - 1) {
            let p = y * w + x;
            let dhy = h[(y + 1) * w + x] - h[(y - 1) * w + x];
            if dhy <= 0.0 {
                continue; // the flank turned AWAY from the lamp: where the scattered light shows
            }
            seen += 1;
            warmed += i64::from(warmth(&warm, p));
            open_neutral += i64::from(warmth(&open, p).abs());
        }
    }
    // The LIT flank, as the control: the filter colours the light that goes THROUGH the paint, and on
    // the side facing the lamp that light is a small correction on top of a large direct term. So the
    // lit flank must warm far LESS than the shadowed one — and if it does not, the divisor is not
    // wearing the filter, the ratio is wrong in the filtered channels EVERYWHERE, and the whole relief
    // is tinted rather than just its shadow.
    let (mut lit_seen, mut lit_warmed) = (0u32, 0i64);
    for y in 1..(size as usize - 1) {
        for x in 1..(size as usize - 1) {
            let p = y * w + x;
            let dhy = h[(y + 1) * w + x] - h[(y - 1) * w + x];
            if dhy >= 0.0 {
                continue;
            }
            lit_seen += 1;
            lit_warmed += i64::from(warmth(&warm, p) - warmth(&open, p));
        }
    }
    assert!(seen > 500, "only {seen} shadowed pixels examined — vacuous");
    assert!(
        lit_seen > 500,
        "only {lit_seen} lit pixels examined — vacuous"
    );
    // The bar is MEASURED, not chosen: correct gives **0.25 levels/px** of warmth on the lit flank, and
    // the mutation that leaves the filter out of the DIVISOR gives **6.1**. (On the shadowed flank they
    // are 12.4 and 27 — both warm, which is why the shadow alone cannot tell them apart, and why this
    // half of the gate has to exist.) One level per pixel sits between them with room on both sides.
    //
    // MUT proved RED: `flat = mat.flat_base[c] + albedo[c] * mat.flat_wax[c]` (the divisor without the
    // filter) ⇒ the filtered channels get the wrong divisor EVERYWHERE, the whole relief tints instead
    // of just its shadow, and this fires. It is the same absolute-shading bug the pass is built to
    // avoid, arriving through the newest door — and it slipped past every gate that used a WHITE filter,
    // because `albedo × white = albedo` makes the two lines identical. A gate can only see a term it
    // actually turns on.
    assert!(
        lit_warmed <= i64::from(lit_seen),
        "a warm Wax filter warmed the flank FACING the lamp by {lit_warmed} levels over {lit_seen}          pixels. The filter colours the light that goes THROUGH the paint, which is a small correction          there — unless the DIVISOR is missing the filter, in which case the ratio is wrong in the          filtered channels everywhere and the whole relief is tinted."
    );
    assert!(
        warmed > i64::from(seen),
        "a WARM filter over WHITE paint left the shadow neutral ({warmed} levels of warmth over {seen} \
         pixels) — the filter is not reaching the scattered light, so alabaster is unpaintable and the \
         swatch is decoration"
    );
    // …and with the filter OPEN, white paint scatters WHITE. If this drifts, the "white = the physics"
    // default is a lie and every existing canvas just changed colour.
    assert_eq!(
        open_neutral, 0,
        "with the Wax filter OPEN (white), white paint tinted its own shadow by {open_neutral} levels \
         — the neutral filter is not neutral"
    );
}

/// **The impasto planes cost what the docs say they cost.**
///
/// HR-13 as amended by ADR-0117: *whoever declares a budget owns a gate that MEASURES*. The material
/// plane went from 4 bytes per pixel to 7 to buy the Wax filter, and "it is only three bytes" is exactly
/// the sentence that precedes a memory regression — so the three bytes are counted here, on the real
/// planes, rather than reasoned about in a comment.
///
/// Per sculpted layer: `heights` (f32) 4 B/px + `covers` 1 B/px + `mats` 7 B/px = **12 B/px**. A layer
/// nobody sculpted carries NONE of them (the maps are lazy), which is the property that keeps the cost
/// where the feature is.
#[test]
fn the_impasto_planes_cost_twelve_bytes_per_pixel() {
    let size = 128u32;
    let n = (size * size) as usize;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.impasto_depth = 0.7;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    // A layer nobody has sculpted pays nothing at all — the maps are lazy, and that is load-bearing.
    assert!(
        t.heights.is_empty() && t.covers.is_empty() && t.mats.is_empty(),
        "an untouched document is already paying for relief it does not have"
    );

    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));

    let active = t.layers.active().expect("a layer");
    let heights = t
        .heights
        .get(&active)
        .expect("the stroke laid relief")
        .len();
    let covers = t.covers.get(&active).expect("…and coverage").len();
    let mats = t.mats.get(&active).expect("…and a material").len();
    assert_eq!(
        (heights, covers, mats),
        (n, n, n),
        "one entry per canvas pixel"
    );

    let bytes = heights * std::mem::size_of::<f32>()
        + covers
        + mats * std::mem::size_of::<ph2d_painter_brush::material::MaterialBytes>();
    let per_px = bytes / n;
    assert_eq!(
        per_px, 12,
        "a sculpted layer costs {per_px} B/px, not the 12 the docs claim (heights 4 + covers 1 + \
         mats 7). If the material grew, say so where the budget is written — HR-13 is a promise, and \
         ADR-0117 is what happens when nobody measures it."
    );
}

/// **Undoing a stroke gives back the material of the paint UNDERNEATH it.**
///
/// The relief, the coverage and the material are three facts about one stroke, and they must roll back
/// together. `mats` was left out of `ModelSnapshot` when the material landed (2026-07-13) — and the hole
/// hid, because on BARE canvas an undone stroke's coverage goes to zero, the light weights its stale
/// material by zero, and nothing shows. The bug only speaks where there is paint to speak for: lay a
/// MATTE stroke, lay a GLOSSY one across it, undo the glossy one — and the matte paint comes back
/// wearing gloss it was never painted with.
///
/// So the gate is paint-over-paint, which is the only place the defect is observable. A gate on an empty
/// canvas would have been green with the bug in it, which is the whole lesson: *test where the fact can
/// be contradicted, not where it is convenient.*
#[test]
fn undoing_a_stroke_restores_the_material_underneath_it() {
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.radius_px = 40.0;
    b.impasto_depth = 0.175; // radius 40 now scales the deposit ×4 (Enio's size-scaling); this restores the calibrated 0.7-load relief so the gate keeps testing the profile, not the scale
    b.color = [0.9, 0.1, 0.1];
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    let stroke = |t: &mut PainterTool, y: f32| {
        t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp(
                [40.0 + 10.0 * f32::from(i as u8), y],
                PointerPhase::Move,
            ));
        }
        t.on_canvas_pointer(cp([120.0, y], PointerPhase::Up));
    };

    // 1. A MATTE stroke.
    t.set_impasto_shine(0.0);
    t.set_impasto_roughness(1.0);
    stroke(&mut t, 80.0);
    let after_matte = lit(&mut t);
    let active = t.layers.active().expect("a layer");
    let matte_mats = t
        .mats
        .get(&active)
        .expect("the stroke baked a material")
        .as_ref()
        .clone();

    // 2. A GLOSSY stroke straight across it — same place, so it overwrites the material there.
    //
    // "Adjust Last Stroke" comes OFF first, and that is not incidental: with it ON the two setters below
    // would re-bake the MATTE stroke to glossy before the second stroke even starts — which is precisely
    // what that toggle is for, and it is what made the first version of this fixture lie. The paint
    // underneath has to be finished paint, or there is nothing for the undo to give back.
    //
    // Written as an assignment, not a toggle: a toggle means "OFF" only while the default is ON, and the
    // default is the artist's to move (it went OFF on 2026-07-19). The fixture states what it needs.
    t.paint.impasto_live_edit = false;
    t.set_impasto_shine(1.0);
    t.set_impasto_roughness(0.0);
    stroke(&mut t, 80.0);
    let glossy_mats = t
        .mats
        .get(&active)
        .expect("…and re-baked it")
        .as_ref()
        .clone();
    assert_ne!(
        matte_mats, glossy_mats,
        "fixture: the glossy stroke must actually have changed the material, or the undo below proves nothing"
    );

    // 3. Undo it. The matte paint must come back MATTE — pixels and material both.
    assert!(t.undo_last(), "the glossy stroke must be undoable");
    let restored_mats = t
        .mats
        .get(&active)
        .expect("the material plane survives the undo")
        .as_ref()
        .clone();
    assert_eq!(
        restored_mats, matte_mats,
        "undoing the glossy stroke left its MATERIAL on the canvas — the matte paint underneath came \
         back wearing gloss it was never painted with. `mats` is missing from `ModelSnapshot`: the \
         relief, the coverage and the material are one fact and they roll back together or not at all."
    );
    let restored = lit(&mut t);
    assert_eq!(
        restored, after_matte,
        "…and the lit canvas must be the matte one again, to the byte"
    );
}
