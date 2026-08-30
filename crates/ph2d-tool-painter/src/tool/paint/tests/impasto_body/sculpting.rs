//! **Os verbos que MEXEM no relevo já depositado.** A faca (smear/plow) que arrasta o corpo através
//! da fronteira, o push que empurra a tinta para o lado conservando o volume, o assentamento que a
//! acomoda no pen-up, o relevo como composite por-camada, e o recorte do commit ao que o traço tocou.

use super::*;

#[test]
fn impasto_smoothing_settles_the_paint_it_just_laid() {
    // Smoothing is a knob I very nearly shipped DEAD — declared in the spec, threaded to the panel, and
    // read by nothing. (That is the exact species the 2026-07-12 sweep spent itself exterminating, so
    // shipping a fresh one would have been quite the joke.) It settles the deposit like a heavy medium
    // relaxing: the ridges soften. Measured as what it IS — the peak gradient of the relief falls.
    let ridge = |smoothing: f32| -> Vec<f32> {
        let size = 60u32;
        let mut t = impasto_canvas(size);
        let mut b = t.paint.brush;
        b.impasto_smoothing = smoothing;
        b.radius_px = 8.0; // a hard disk ⇒ a sharp-walled slab: maximum gradient to soften
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([30.0, 15.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([30.0, 45.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([30.0, 45.0], PointerPhase::Up));
        relief(&t)
    };
    let steepest = |h: &[f32]| {
        let size = 60usize;
        let mut m = 0.0f32;
        for y in 1..size - 1 {
            for x in 1..size - 1 {
                let g = (h[y * size + x + 1] - h[y * size + x - 1]).abs();
                m = m.max(g);
            }
        }
        m
    };
    let sharp = ridge(0.0);
    let settled = ridge(1.0);
    let (gs, gt) = (steepest(&sharp), steepest(&settled));
    assert!(gs > 0.0, "the sharp ridge has walls to soften");
    assert!(
        gt < gs * 0.7,
        "Smoothing settles the paint: the steepest wall falls from {gs} to {gt}"
    );
    // Volume is conserved — settling SPREADS the paint, it does not evaporate it. (A blur that leaked
    // volume would quietly flatten every stroke the artist smoothed.)
    let vol = |h: &[f32]| h.iter().sum::<f32>();
    let (vs, vt) = (vol(&sharp), vol(&settled));
    assert!(
        (vt - vs).abs() < vs * 0.05,
        "the paint spreads, it does not vanish ({vs} → {vt})"
    );
}

#[test]
fn impasto_plow_drags_the_relief_with_the_paint() {
    // **Plow** — the palette knife (plan §6, deferred since the first cut; Corel's `Plow`, ArtRage's
    // Flat blade). Until now the Smear dragged the COLOUR and left the body of the paint where it was:
    // thick paint was unworkable once it landed, and the light kept shading a ridge that the pigment had
    // already left. This gate says the two move as one thing.
    let size = 80u32;
    let smear_with_plow = |plow: f32| -> (Vec<f32>, Vec<u8>) {
        let mut t = impasto_canvas(size);
        // 1. Lay a ridge of thick paint on the LEFT half.
        let mut b = t.paint.brush;
        b.radius_px = 10.0;
        b.impasto_depth = 1.0;
        t.paint.brush = b;
        t.paint.brush_by_mode.fill(b);
        t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([20.0, 60.0], PointerPhase::Up));

        // 2. Then take the KNIFE across it, to the right.
        let mut k = t.paint.brush;
        k.impasto_plow = plow;
        k.strength = 1.0;
        k.hardness = 1.0; // a firm blade: the drag is the claim, not the falloff
        t.paint.brush = k;
        t.paint.brush_by_mode.fill(k);
        t.paint.paint_mode = PaintMode::Smear;
        t.on_canvas_pointer(cp([20.0, 40.0], PointerPhase::Down));
        for x in 1..=20 {
            t.on_canvas_pointer(cp([20.0 + f32::from(x as u8), 40.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));

        let active = t.layers.active().expect("a layer");
        (
            t.heights
                .get(&active)
                .map(|f| f.as_ref().clone())
                .unwrap_or_default(),
            t.covers
                .get(&active)
                .map(|c| c.as_ref().clone())
                .unwrap_or_default(),
        )
    };

    let at = |v: &[f32], x: u32, y: u32| v[(y * size + x) as usize];
    let (no_plow, _) = smear_with_plow(0.0);
    let (plowed, cov) = smear_with_plow(1.0);

    // Sanity: the ridge is there, on the left, in both.
    assert!(at(&no_plow, 20, 40) > 0.3, "the ridge was laid");

    // 1. WITHOUT Plow the knife leaves the body exactly where it was — the default is "smear the
    //    pigment, do not touch the paint's thickness" (and the artist gets the old behaviour untouched).
    assert!(
        at(&no_plow, 34, 40) < 0.02,
        "no Plow ⇒ the relief stays put ({})",
        at(&no_plow, 34, 40)
    );

    // 2. WITH Plow the relief is DRAGGED into the path of the knife — thick paint where there was none.
    assert!(
        at(&plowed, 34, 40) > 0.1,
        "Plow ⇒ the knife carries the body along ({} at x=34)",
        at(&plowed, 34, 40)
    );

    // 3. And it moves WITH the paint, not beside it: where the knife dragged relief, it dragged the
    //    coverage too. A relief that travelled without its coverage would be a ridge the light shades
    //    over paint that is no longer there — the ghost the Eraser gate already refuses, arriving by
    //    another door.
    let c = |x: u32, y: u32| f32::from(cov[(y * size + x) as usize]) / 255.0;
    assert!(
        c(34, 40) > 0.1,
        "the paint's presence travelled with its body ({} at x=34)",
        c(34, 40)
    );
}

/// **The knife carries the BODY as far as it carries the PIGMENT** (Enio, 2026-07-18:
/// *"operações como smear não conseguem levar o relevo para além das fronteiras do traço original"*).
///
/// The mechanism was never broken — it was switched OFF. `impasto_plow` defaulted to `0.0` on the
/// rationale that *"the Smear drags the COLOUR and leaves the body where it is"*, and the measured price
/// of that sentence is this: a knife dragged across a thick stroke pushed pigment to **x = 99** while the
/// relief stopped at **x = 41**, which is the exact edge of where the stroke had been painted. Paint with
/// no body, spread over the canvas, and nothing in the tool would tell you why.
///
/// Turning it on was **necessary and not sufficient**, and the first version of this gate could not tell
/// the difference — it compared REACH along the drag axis, and reach was the one thing that already
/// worked. Enio, shown the "fix": *"as fronteiras não são vencidas. o relevo não é levado além. **nada
/// resolvido**"*. The body crossed as a **one-texel filament**: mass nowhere, needle everywhere.
///
/// So the verdict is a **CROSS-SECTION**, cut across the trail past the frontier — reach stays only as a
/// precondition. The older sibling (`impasto_plow_drags_the_relief_with_the_paint`) samples one texel
/// just past the ridge, which is true at any Plow above zero; between them, two green gates sat beside a
/// red product. This asks the artist's real question: *is there THICKNESS in what I pushed, or just a
/// scratch through it?*
///
/// **Mutations that must bleed:** (a) put `impasto_plow` back to `0.0` in `spec_default` — the relief
/// stops dead at the original stroke's frontier (kills the reach precondition); (b) restore the
/// per-dab lift-and-blend transport in place of the accumulated field — reach survives, the
/// cross-section collapses to ~1 px (this is the mutation the old gate could not feel).
#[test]
fn the_knife_carries_the_body_across_the_frontier_as_mass_not_a_filament() {
    // Canvas and drag are sized on the PRODUCT's proportions (probe scene 13: the knife travels ~8
    // brush-radii out of the ridge). A short drag hides this bug — the trail is still full width where
    // the brush is parked, and the collapse only compounds with distance behind it.
    let size = 220u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 1.0;
    b.color = [0.8, 0.1, 0.1];
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    // A vertical ridge of thick paint at x = 40.
    t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 190.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 190.0], PointerPhase::Up));

    let active = t.layers.active().expect("a layer");
    let reach = |v: &[f32]| {
        (0..size)
            .filter(|x| (0..size).any(|y| v[(y * size + x) as usize] > 0.02))
            .max()
            .unwrap_or(0)
    };
    let ridge_edge = reach(&t.heights.get(&active).expect("relief").as_ref().clone());
    assert!(
        (45..70).contains(&ridge_edge),
        "precondition: the ridge ends where the brush ended ({ridge_edge}) — if it already spanned the \
         canvas there would be no frontier to cross"
    );

    // Now take the knife straight across it, far past the ridge — through the REAL mode door, and with
    // the brush the Smear tool brings with it.
    //
    // ⚠️ Both halves of that sentence are load-bearing, and the first version of this gate got both
    // wrong. Poking `paint.paint_mode` skips `switch_brush_slot`, which is what loads the Smear tool's
    // OWN slot (its spacing, its falloff) — the probe already learned this the expensive way. And on top
    // of the poke it forced `hardness = 1.0`: a hard disk has `w = 1` across the whole footprint, so the
    // per-dab product `wⁿ` never decays and the filament CANNOT form. The fixture excluded the exact
    // phenomenon it existed to catch, and reported 24 px of body under 24 px of pigment — a perfect
    // score, on a canvas where the bug was unreachable.
    t.set_paint_tool_mode("smear");
    t.set_brush_size_px(16.0);
    let knife_r = t.paint.brush.radius_px;
    let (knife_y, knife_end) = (110.0f32, 200.0f32);
    t.on_canvas_pointer(cp([40.0, knife_y], PointerPhase::Down));
    for x in 1..=160 {
        t.on_canvas_pointer(cp([40.0 + x as f32, knife_y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([knife_end, knife_y], PointerPhase::Up));

    // Question ONE — reach: how far along the knife's own line did each thing get? This is the half the
    // old gate asked, and it was already green. Kept as a precondition, never as the verdict.
    let y = knife_y as u32;
    let heights = t.heights.get(&active).expect("relief").as_ref().clone();
    let rgba = t.canvas_rgba.as_ref().clone();
    let relief_x = (0..size)
        .filter(|x| heights[(y * size + x) as usize] > 0.02)
        .max()
        .unwrap_or(0);
    // The pigment is red on white paper, so the GREEN channel falling is the ink arriving.
    let pigment_x = (0..size)
        .filter(|x| rgba[((y * size + x) * 4) as usize + 1] < 200)
        .max()
        .unwrap_or(0);

    assert!(
        pigment_x > ridge_edge + 20,
        "precondition: the knife must actually have pushed pigment past the ridge \
         (pigment {pigment_x}, ridge edge {ridge_edge})"
    );
    assert!(
        relief_x + 4 >= pigment_x,
        "precondition (reach): the knife pushed pigment to x={pigment_x} but its BODY only to \
         x={relief_x} (ridge edge was x={ridge_edge})"
    );

    // Question TWO — MASS, and this is the verdict.
    //
    // ⚠️ The oracle is the BRUSH, never the pigment. Measuring the body against the colour was the
    // second thing this gate got wrong: the colour Smear (`smear_dab`) has the IDENTICAL per-dab
    // structure, so it collapses by the identical law — measured here, `relief_w == pigment_w` to the
    // texel at every station along the trail. A ratio between two equally sick quantities is green by
    // construction. What the artist is owed is absolute: drag a knife this wide and the trail it leaves
    // is about that wide.
    //
    // Nor is one station enough. Where the brush is PARKED the trail is always full width (those dabs
    // have had no steps applied to them yet); the collapse compounds with distance BEHIND the tip. So
    // walk the clean trail — past the ridge frontier, before the parked footprint — and take its
    // NARROWEST point.
    let trail_from = ridge_edge + knife_r as u32;
    let trail_to = (knife_end - knife_r) as u32;
    assert!(
        trail_to > trail_from + 16,
        "precondition: the drag must leave a stretch of trail that is neither ridge nor parked brush \
         (x {trail_from}..{trail_to})"
    );
    let width_at =
        |x: u32, plane: &dyn Fn(u32, u32) -> bool| (0..size).filter(|&yy| plane(x, yy)).count();
    let has_relief = |x: u32, yy: u32| heights[(yy * size + x) as usize] > 0.02;
    let has_pigment = |x: u32, yy: u32| rgba[((yy * size + x) * 4) as usize + 1] < 200;
    let (thin_x, relief_w) = (trail_from..trail_to)
        .map(|x| (x, width_at(x, &has_relief)))
        .min_by_key(|&(_, w)| w)
        .expect("a trail to walk");
    let pigment_w = width_at(thin_x, &has_pigment);

    // The knife is `2·knife_r` across. Half of that is a generous floor — the falloff means the rim
    // contributes little, so a healthy trail sits well above it while a filament is nowhere near.
    let want = knife_r as usize;
    assert!(
        relief_w >= want,
        "at its thinnest the knife's trail carries {relief_w} px of BODY (x={thin_x}), but the knife is \
         {} px across — the relief crossed the frontier as a FILAMENT, not as mass. Transport that \
         re-samples the PREVIOUS STEP'S RESULT decays geometrically off the drag axis: dab spacing is \
         ~1 px, so the trail is a product h·wⁿ over the whole drag — on the axis t=0 ⇒ w=1 exactly and \
         the needle survives, 6 px off it 0.8¹⁵⁰ ≈ 0. Measured on the product (`push_look_probe` scene \
         13): across the trail at x=250, `y194 h0.00 · y200 h3.73 · y206 h0.00`. The displacement has \
         to ACCUMULATE and be applied ONCE to a frozen source — the law `warp/apply.rs` already uses",
        2.0 * knife_r
    );
    assert!(
        pigment_w >= want,
        "at its thinnest the knife's trail carries {pigment_w} px of PIGMENT (x={thin_x}) against a \
         knife {} px across. The colour Smear has the same per-dab structure as the body's, and Enio \
         chose to fix both from the SAME field — so the colour is held to the same law",
        2.0 * knife_r
    );
}

/// Paint a ridge and drag the knife across it, sampling the drag at `stride` px. Returns the trail's
/// width at its thinnest, and how far the pigment reached.
#[cfg(test)]
fn smear_trail_probe(size: u32, stride: f32) -> (usize, u32) {
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 1.0;
    b.color = [0.8, 0.1, 0.1];
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([40.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 190.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([40.0, 190.0], PointerPhase::Up));

    t.set_paint_tool_mode("smear");
    t.set_brush_size_px(16.0);
    let (y, end) = (110.0f32, 200.0f32);
    t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
    let mut x = 40.0 + stride;
    while x < end {
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        x += stride;
    }
    t.on_canvas_pointer(cp([end, y], PointerPhase::Up));

    let rgba = t.canvas_rgba.as_ref().clone();
    let wide = |x: u32| {
        (0..size)
            .filter(|&yy| rgba[((yy * size + x) * 4) as usize + 1] < 200)
            .count()
    };
    let thin = (70..180).map(wide).min().unwrap_or(0);
    let reach = (0..size)
        .filter(|&x| rgba[((y as u32 * size + x) * 4) as usize + 1] < 200)
        .max()
        .unwrap_or(0);
    (thin, reach)
}

/// **The knife's trail is a fact of the PATH, not of how finely the motion was sampled.**
///
/// The sibling of `the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing` (the bow wave's bite) and of
/// the relief capsule's law — the third time this line has met the same disease, and the reason the
/// Smear's transport had to stop being a per-dab lerp. A sequential accumulation that re-reads its own
/// output raises the dab count to a power: at ~1 px spacing a long drag is ~170 steps, `wⁿ`, and the
/// trail dies everywhere except exactly on the axis.
///
/// Sampling the SAME 160 px path coarsely must therefore give the same trail — a different mouse, or a
/// different polling rate, is not a different brush.
///
/// **Mutation that must bleed:** make the kernel's sink `disp[i] += step·add` (a sum instead of the map
/// composition) — the two samplings then disagree about how far the paint reached, because a summed
/// field's travel is bounded by the brush width rather than by the path.
#[test]
fn the_smear_trail_is_a_fact_of_the_path_not_the_dab_spacing() {
    let size = 220u32;
    let (fine_w, fine_reach) = smear_trail_probe(size, 1.0);
    let (coarse_w, coarse_reach) = smear_trail_probe(size, 4.0);
    assert!(
        fine_w >= 12,
        "precondition: the finely-sampled drag must leave a trail with mass ({fine_w} px)"
    );
    assert!(
        coarse_w * 2 >= fine_w && fine_w * 2 >= coarse_w,
        "the same 160 px path sampled at 1 px and at 4 px left trails {fine_w} px and {coarse_w} px \
         across. The knife is a property of the path, not of how often the OS delivered a pointer event"
    );
    let (lo, hi) = (fine_reach.min(coarse_reach), fine_reach.max(coarse_reach));
    assert!(
        hi - lo <= 12,
        "the same path reached x={fine_reach} sampled finely and x={coarse_reach} sampled coarsely — \
         transport whose distance depends on the sampling is the product law again"
    );
}

/// **The knife's warp session is per STROKE, and leaving the tool ends it.**
///
/// Deform's session spans strokes (Reconstruct needs the history); the Smear's must not, because it has
/// no Apply or Reset to close one and each stroke's result is the next stroke's baseline. A session that
/// outlived its stroke would hold a frozen `pre` describing a canvas the next stroke has already moved,
/// and the next stroke would re-warp from it — paint jumping back to where it used to be.
///
/// The second half is the invariant `warp::session`'s docs state: at most one tool may own a live
/// session, so leaving the mode ends it. Without it a `disp` accumulated by the knife would be read by
/// Reshape as its own.
///
/// **Mutations that must bleed:** drop the `end_smear_session()` call from `close_stroke`; drop the
/// `PaintMode::Smear` arm from `set_paint_tool_mode`'s teardown.
#[test]
fn the_knives_warp_session_does_not_outlive_its_stroke() {
    let size = 120u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 10.0;
    b.impasto_depth = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 100.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 100.0], PointerPhase::Up));

    t.set_paint_tool_mode("smear");
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    for x in 1..=20 {
        t.on_canvas_pointer(cp([30.0 + x as f32, 60.0], PointerPhase::Move));
    }
    assert!(
        t.paint.warp.active,
        "the knife opens a warp session while it is dragging"
    );
    t.on_canvas_pointer(cp([50.0, 60.0], PointerPhase::Up));
    assert!(
        !t.paint.warp.active,
        "…and the session dies with the stroke, so the next one starts from what this one left"
    );
    assert!(
        t.paint.warp.disp.is_empty(),
        "the displacement is freed with the session, not left to be re-applied"
    );

    // And a session that IS live when the artist changes tool must not survive the change.
    t.on_canvas_pointer(cp([30.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([40.0, 80.0], PointerPhase::Move));
    assert!(t.paint.warp.active, "fixture: a session is live mid-stroke");
    t.set_paint_tool_mode("brush");
    assert!(
        !t.paint.warp.active,
        "leaving the Smear ends its session — one live session, one owner"
    );
}

/// **A pilha do Composite Brush é o TERCEIRO membro da família do smear, e a guarda que fechava a
/// sessão era uma ENUMERAÇÃO dos dois primeiros.**
///
/// `end_smear_session` perguntava `paint_mode.smears()` — *Smear ou Knife*. A camada Smear da pilha roda
/// em `PaintMode::Paint`, então a resposta era NÃO e **a sessão nunca era encerrada**: a fonte congelada
/// continuava a do primeiro pen-down do documento, o campo de deslocamento somava para sempre, e cada
/// batch re-resolvia a região CUMULATIVA através dele. Medido antes do conserto, três traços em
/// composite: `disp` ≠ 0 em **9.904 → 19.808 → 29.712** texels e a região re-renderizada em **h 41 → 81
/// → 121** — *o desenho inteiro escorregando enquanto o artista pinta*, com o custo crescendo junto.
/// O doc de `PaintMode::smears` já avisava exatamente isto sobre enumerar os sítios.
///
/// As quatro metades importam e nenhuma implica as outras:
/// 1. **CONTROLE** — a sessão está VIVA no meio do traço. Sem ele o gate passa com o Smear morto.
/// 2. Ela morre no pen-up e o `disp` é liberado.
/// 3. Ela **não cresce entre traços** — é a metade que o número acima descreve.
/// 4. **O Deform continua isento** — a sessão dele atravessa traços de propósito (o Reconstruct precisa
///    da história, e Apply/Reset a encerram). É a metade que um predicado só-`warp.active` quebraria.
///
/// **Mutação que must bleed:** voltar a guarda para `self.paint.paint_mode.smears()`.
#[test]
fn the_composite_stack_closes_its_smear_session_at_pen_up() {
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.paint.composite_enabled = true;

    let mut widths = Vec::new();
    for k in 0..3 {
        let y = 40.0 + k as f32 * 30.0;
        t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down));
        for i in 1..=20 {
            t.on_canvas_pointer(cp([30.0 + i as f32 * 5.0, y], PointerPhase::Move));
        }
        // (1) o CONTROLE: a pilha de fato abriu uma sessão de smear.
        assert!(
            t.paint.warp.active,
            "fixture: a camada Smear da pilha abre uma sessao durante o traco"
        );
        t.on_canvas_pointer(cp([130.0, y], PointerPhase::Up));
        // (2) ela morre com o traço.
        assert!(
            !t.paint.warp.active,
            "a sessao da pilha morre no pen-up, como a do Smear"
        );
        assert!(
            t.paint.warp.disp.is_empty(),
            "o deslocamento e' liberado com a sessao, nao deixado para ser re-aplicado"
        );
        widths.push(t.paint.warp.touched_all.map_or(0, |r| r.h));
    }
    // (3) a região re-renderizada é POR TRAÇO: ela não pode crescer com a história.
    assert!(
        widths[2] <= widths[0] + 2,
        "a regiao re-renderizada cresce com os tracos ({widths:?}) — a sessao esta' sobrevivendo"
    );

    // (4) e o Deform, que POSSUI uma sessão cross-traço, segue isento.
    t.paint.composite_enabled = false;
    t.set_paint_tool_mode("liquify");
    t.on_canvas_pointer(cp([60.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 80.0], PointerPhase::Up));
    assert!(
        t.paint.warp.active,
        "a sessao do Deform atravessa tracos de proposito — Apply/Reset e' que a encerram"
    );
}

/// **The knife's kill criterion**, mirroring the sculpt's: canvas 2048² and 4096², brush radius 100, a
/// dragged stroke over a relief-bearing layer — so the body rides the same map and its three planes are
/// re-rendered too. Target **≤ 4 ms/move**, **kill at 8**.
///
/// The field transport is not obviously cheaper or dearer than the lift-and-blend it replaced, and
/// "obviously" is not a measurement. Per dab it walks the same footprint once, then snapshots the old map
/// over that footprint (the composition must not read a texel it has already written) — one extra pass
/// over a dab-sized window. Per BATCH it resamples the frozen source once over the union of what moved,
/// which is the work the blend used to do per dab.
///
/// Move 0 is excluded for the reason the sculpt's gate documents: the session opens there (canvas-sized
/// `disp` allocated and first-touched, four planes frozen), and charging a once-per-stroke cost to every
/// move reports a number no frame actually pays. It is asserted separately rather than discarded — a
/// hitch at the start of every stroke is something the artist feels.
#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn smear_perf_kill_criterion() {
    use std::time::Instant;
    const MOVES: u32 = 20;
    const KILL_MS: f64 = 8.0;
    const SETUP_KILL_MS: f64 = 40.0;
    const WARMUP_MOVES: usize = 1;

    let run = |size: u32| -> (f64, f64) {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let layer = t.layers.active().expect("a layer");
        let n = (size * size) as usize;
        let relief: Vec<f32> = (0..n)
            .map(|i| {
                let x = (i as u32 % size) as f32;
                let y = (i as u32 / size) as f32;
                0.4 * ((x * 0.031).fract() + (y * 0.017).fract())
            })
            .collect();
        t.heights.insert(layer, Arc::new(relief));
        t.covers.insert(layer, Arc::new(vec![255u8; n]));
        t.sync_relief_flags();
        t.set_paint_tool_mode("smear");
        let mut b = t.paint.brush;
        b.radius_px = 100.0;
        b.strength = 1.0;
        t.paint.brush = b;
        t.paint.brush_by_mode[PaintMode::Smear.slot()] = b;

        let mid = (size / 2) as f32;
        t.on_canvas_pointer(cp([200.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut per_move = Vec::with_capacity(MOVES as usize);
        for k in 1..=MOVES {
            let x = 200.0 + 12.0 * k as f32;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            let _ = t.take_preview_arc();
            per_move.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        t.on_canvas_pointer(cp([200.0 + 12.0 * MOVES as f32, mid], PointerPhase::Up));
        let setup = per_move[..WARMUP_MOVES].iter().cloned().fold(0.0, f64::max);
        let steady =
            per_move[WARMUP_MOVES..].iter().sum::<f64>() / (per_move.len() - WARMUP_MOVES) as f64;
        (setup, steady)
    };

    let (s2, m2) = run(2048);
    let (s4, m4) = run(4096);
    eprintln!(
        "[smear-perf] 2048²: setup {s2:.2} ms · steady {m2:.2} ms/move │ \
         4096²: setup {s4:.2} ms · steady {m4:.2} ms/move (kill {KILL_MS})"
    );
    assert!(
        m2 < KILL_MS && m4 < KILL_MS,
        "the knife costs {m2:.2} ms/move @2048² and {m4:.2} ms/move @4096², against a kill of {KILL_MS}"
    );
    assert!(
        s2 < SETUP_KILL_MS && s4 < SETUP_KILL_MS,
        "opening the knife's session costs {s2:.2} ms @2048² and {s4:.2} ms @4096² — a hitch at the \
         start of every stroke, against a bar of {SETUP_KILL_MS}"
    );
}

/// **The knife's transport relays ACROSS strokes**: a second drag carries the paint further than the
/// first left it, rather than starting over from the first stroke's frozen source.
///
/// ⚠️ Read the claim narrowly. This does NOT prove the session is per-stroke — a warp map composes, and
/// composition is associative, so a session wrongly spanning both strokes reconstructs very nearly the
/// same picture and this gate stays green. That was measured, not assumed: removing the pen-up teardown
/// leaves this test passing. The session's lifetime is guarded by
/// `the_knives_warp_session_does_not_outlive_its_stroke`, which asks about the session directly; the two
/// halves of the teardown (pen-up, and leaving the mode mid-stroke) are separate defences and it kills
/// the mutation of each. What this gate owns is only the artist-visible relay.
#[test]
fn a_second_smear_stroke_builds_on_the_first() {
    let size = 160u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 1.0;
    b.color = [0.8, 0.1, 0.1];
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 140.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 140.0], PointerPhase::Up));

    t.set_paint_tool_mode("smear");
    t.set_brush_size_px(14.0);
    let reach = |t: &PainterTool| {
        let rgba = t.canvas_rgba.as_ref().clone();
        (0..size)
            .filter(|&x| rgba[((80 * size + x) * 4) as usize + 1] < 200)
            .max()
            .unwrap_or(0)
    };
    let drag = |t: &mut PainterTool, x0: f32, x1: f32| {
        t.on_canvas_pointer(cp([x0, 80.0], PointerPhase::Down));
        let mut x = x0 + 1.0;
        while x < x1 {
            t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
            x += 1.0;
        }
        t.on_canvas_pointer(cp([x1, 80.0], PointerPhase::Up));
    };
    drag(&mut t, 30.0, 80.0);
    let after_first = reach(&t);
    drag(&mut t, 60.0, 120.0);
    let after_second = reach(&t);
    assert!(
        after_second > after_first,
        "the second stroke carried the paint FURTHER (first reached x={after_first}, second \
         x={after_second}) — if the second re-warped the first stroke's frozen source instead of its \
         result, the paint would have snapped back to where it started"
    );
}

// ── Impasto (#16) — the relief as a per-LAYER composite (plan §10.8) ───────────────────────────────

#[test]
fn impasto_layer_depth_reaches_strokes_the_brush_no_longer_can() {
    // THE claim of the whole feature. The brush's Depth is baked into each stroke as it lands; the live
    // re-derive (`refresh_live_relief`) reaches only the LAST one. So the moment you lay a second
    // stroke, the first one's thickness is frozen — and until now nothing in the product could ever
    // touch it again. The layer's Depth can: it is a COMPOSITE parameter, so it acts on everything ever
    // sculpted on the layer, forever, without re-sculpting a single texel.
    //
    // (This is opacity's exact bargain, one axis over: `0` mutes, it does not erase. Which is why the
    // gate ends by turning it back up and demanding the sculpture return *bit for bit*.)
    let mut t = impasto_canvas(60);
    t.on_canvas_pointer(cp([15.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([15.0, 30.0], PointerPhase::Up));
    t.on_canvas_pointer(cp([45.0, 30.0], PointerPhase::Down)); // the first stroke is now HISTORY
    t.on_canvas_pointer(cp([45.0, 30.0], PointerPhase::Up));
    let layer = t.layers.active().expect("a layer");

    let (old, new) = (t.composed_relief_at(15, 30), t.composed_relief_at(45, 30));
    assert!(old > 0.1 && new > 0.1, "both strokes stand ({old} / {new})");

    // Down to zero: BOTH vanish from the light — including the one the brush can no longer reach.
    t.set_layer_impasto_depth(layer, 0.0);
    assert_eq!(
        (t.composed_relief_at(15, 30), t.composed_relief_at(45, 30)),
        (0.0, 0.0),
        "Depth 0 mutes the layer's whole relief, the old stroke included"
    );

    // Negative INVERTS: the ridge becomes a groove. (A scale, not a switch — this is the reading that
    // makes the plan's `Subtract` mode redundant, and a redundant mode is a dead knob waiting.)
    t.set_layer_impasto_depth(layer, -1.0);
    assert!(
        t.composed_relief_at(15, 30) < -0.1,
        "negative Depth carves what it used to raise ({})",
        t.composed_relief_at(15, 30)
    );

    // And back up: the sculpture returns EXACTLY. Nothing was destroyed — that is the whole difference
    // between a composite parameter and an edit.
    t.set_layer_impasto_depth(layer, 1.0);
    assert!(
        (t.composed_relief_at(15, 30) - old).abs() < 1e-6
            && (t.composed_relief_at(45, 30) - new).abs() < 1e-6,
        "the relief comes back bit for bit ({} / {})",
        t.composed_relief_at(15, 30),
        t.composed_relief_at(45, 30)
    );
}

#[test]
fn impasto_level_buries_what_is_under_it_in_the_stacking_order() {
    // `Level` is the one thing the depth SCALE cannot say: thick opaque paint whose surface REPLACES
    // the texture under it instead of inheriting it (the research's "composite, don't add" —
    // `docs/Painter/17_impasto_deposito_pesquisa2.md`). Everything else the plan listed as a mode —
    // Add / Subtract / Ignore — is a reading of the signed slider, and shipping them as an enum would
    // have been the old "Amount" all over again.
    //
    // And `Level` is what makes the fold's ORDER load-bearing. Until it existed the composite was a
    // plain sum, and a sum commutes — so the first cut folded the height map in `BTreeMap` key order
    // and nobody could tell. This gate is built so that key order and Z-ORDER DISAGREE: two layers
    // created in one order, then stacked in the other. Fold by key and the answer is wrong.
    let size = 60u32;
    let mut t = impasto_canvas(size);
    // Ground floor: a thick ridge on the base layer.
    let mut b = t.paint.brush;
    b.impasto_depth = 0.8;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Up));
    let ground = t.composed_relief_at(30, 30);
    assert!(ground > 0.5, "the base carries a ridge ({ground})");

    // Two layers ABOVE it, created low-then-high (so their ids ascend)…
    let mid = t.add_raster_layer("mid").expect("a layer");
    let top = t.add_raster_layer("top").expect("a layer");
    // …then stacked the OTHER way round: `top` (the higher id) is pushed BELOW `mid`. Now the id order
    // (mid, top) and the z-order (top, mid) disagree, which is the whole point of the fixture.
    t.move_layer_down(top);
    let z = t.layers.z_order_bottom_up();
    let (zi_top, zi_mid) = (
        z.iter().position(|&i| i == top).expect("top in z"),
        z.iter().position(|&i| i == mid).expect("mid in z"),
    );
    assert!(zi_top < zi_mid, "the fixture stacks `top` UNDER `mid`");
    assert!(top.0 > mid.0, "…while its id sorts AFTER it");

    // Each of them lays its own thin skin of paint over the same spot.
    let mut thin = t.paint.brush;
    thin.impasto_depth = 0.25;
    t.paint.brush = thin;
    t.paint.brush_by_mode.fill(thin);
    for l in [top, mid] {
        t.select_layer(l);
        t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([30.0, 30.0], PointerPhase::Up));
    }
    let piled = t.composed_relief_at(30, 30);
    assert!(
        piled > ground,
        "with everything on Add, paint piles up ({piled} over {ground})"
    );

    // Now the TOPMOST layer (`mid`) levels: its own skin becomes the surface, and the mountain under it
    // — the base's ridge AND `top`'s skin — is buried.
    t.set_layer_impasto_composite(mid, crate::layers::ReliefComposite::Level);
    let levelled = t.composed_relief_at(30, 30);
    assert!(
        levelled < ground,
        "Level buries the pile beneath it ({levelled} vs a ground of {ground})"
    );
    // The mutation that this fixture exists to kill: folding in key order would put `top` LAST, so its
    // skin would be ADDED on top of the levelled surface. The answer would land near `levelled + top's
    // own skin` instead of `mid`'s skin alone.
    let mid_alone = t
        .layer_height_view(mid)
        .map(|f| f[(30 * size + 30) as usize]);
    assert!(
        (levelled - mid_alone.expect("mid has relief")).abs() < 1e-5,
        "…down to exactly the levelling layer's OWN body — a fold in id order would leave the layer \
         below it stacked back on top ({levelled} vs {mid_alone:?})"
    );

    // Base (the default) is untouched: Add still sums, or none of the above means anything.
    t.set_layer_impasto_composite(mid, crate::layers::ReliefComposite::Add);
    assert!(
        (t.composed_relief_at(30, 30) - piled).abs() < 1e-6,
        "back on Add, the pile returns exactly"
    );
}

#[test]
fn impasto_the_panel_learns_a_layer_has_relief_the_moment_it_does() {
    // The Depth row is painted on a row only `if layer.has_relief` — a flag the tool PROJECTS from its
    // height map, because the relief lives with the pixels and the panel only ever sees a clone of the
    // stack. Two ways that goes wrong, and both are this house's greatest hits:
    //
    //   1. The flag drifts from the map ⇒ a knob on a layer it cannot act on, or no knob on a layer it
    //      could ([[feedback_tool_unit_green_integration_dead]]). So: pin the INVARIANT, not a value.
    //   2. The flag is right and the panel never hears about it. The snapshot republishes on
    //      `layers_revision`, and a paint stroke is a PIXEL edit — it does not bump it. Sculpt the first
    //      ridge on a layer and the row would appear only after some unrelated layer edit happened to
    //      bump the revision by accident. That is a bug you cannot see in a unit test that only reads
    //      the flag, so this gate reads the revision too.
    let mut t = impasto_canvas(40);
    let layer = t.layers.active().expect("a layer");
    let other = t.add_raster_layer("untouched").expect("a layer");
    t.select_layer(layer);

    let invariant = |t: &PainterTool, where_: &str| {
        for id in t.layers.all_ids() {
            let flag = t.layers.get(id).expect("layer").has_relief;
            let truth = t.heights.contains_key(&id);
            assert_eq!(
                flag, truth,
                "{where_}: layer {id:?} flag {flag} vs map {truth}"
            );
        }
    };
    invariant(&t, "a fresh canvas");
    assert!(
        !t.layers.get(layer).expect("layer").has_relief,
        "nothing is sculpted yet"
    );

    let rev_before = t.layers_revision();
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));

    invariant(&t, "after the first stroke");
    assert!(
        t.layers.get(layer).expect("layer").has_relief,
        "the sculpted layer says so"
    );
    assert!(
        !t.layers.get(other).expect("layer").has_relief,
        "…and the untouched one does not"
    );
    assert_ne!(
        t.layers_revision(),
        rev_before,
        "the panel must be TOLD — an unchanged revision means it never republishes the stack, so the \
         Depth row never appears however right the flag is"
    );

    // Undo takes the relief back, and the flag goes with it: the snapshot carries the stack and the
    // height map together, so they cannot come back disagreeing.
    assert!(t.undo_last(), "the stroke is one undo step");
    invariant(&t, "after undo");
    assert!(
        !t.layers.get(layer).expect("layer").has_relief,
        "undo un-sculpts the layer, and the row goes with it"
    );
}

#[test]
fn impasto_smoothing_settles_every_stroke_the_moment_the_pen_leaves_the_canvas() {
    // Enio, 2026-07-12, the SECOND report and the sharper one: *"o primeiro traço aplica o smoothing
    // automaticamente. A partir do segundo não aplica até que mova o slider."*
    //
    // The settle was running — it always was. What was missing is that nobody asked for the pixels to be
    // LIT again. At pen-up the relief under the painting is swapped (the raw envelope the stroke was
    // drawn with becomes the settled field), and **no pixel changed**, so nothing on that path marked
    // the canvas dirty: the composite cache went on showing the lighting it drew during the stroke, from
    // the UNSETTLED relief. Move any Body knob and `refresh_live_relief` invalidates the composite — and
    // the smoothing appears, late. Exactly what he described.
    //
    // Why the FIRST stroke worked, and why that is the whole lesson: it flips the layer's `has_relief`
    // (§10.8), and that flag change invalidates the composite as a side effect. The first stroke was
    // being rescued **by accident**. A one-stroke fixture therefore CANNOT contain this bug — and the
    // one I wrote yesterday did not, and shipped green over a live defect. The phenomenon lives in the
    // second stroke, so the gate paints three.
    let size = 120u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 14.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.impasto_depth = 1.0;
    b.impasto_body = 1.0;
    b.impasto_smoothing = 1.0; // the knob under test, at its loudest
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);

    for (n, y) in [(1u32, 30.0f32), (2, 60.0), (3, 90.0)] {
        // The stroke, at the app's real cadence: a preview drain after every pointer event, as a frame
        // would. (A test that never drains starts `preview_dirty` and full-recomposes at the end — which
        // is a fixture that has quietly removed the bug it was written to catch.)
        t.on_canvas_pointer(cp([30.0, y], PointerPhase::Down));
        let _ = t.take_preview_arc();
        for i in 1..=6 {
            t.on_canvas_pointer(cp([30.0 + 10.0 * i as f32, y], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([90.0, y], PointerPhase::Up));

        // The frame the artist is looking at when they say it did not settle…
        let (seen, w, h) = t.take_preview_arc().expect("a preview after the stroke");
        // …against the truth: the same document, composited from scratch.
        t.composited = None;
        t.preview_dirty = true;
        let (truth, _, _) = t.take_preview_arc().expect("a full recompose");

        let (mut differing, mut worst) = (0usize, 0i32);
        for p in 0..(w * h) as usize {
            for c in 0..3 {
                let d = i32::from(seen[p * 4 + c]) - i32::from(truth[p * 4 + c]);
                if d != 0 {
                    differing += 1;
                }
                worst = worst.max(d.abs());
            }
        }
        assert_eq!(
            differing, 0,
            "stroke {n}: the frame after pen-up must already BE the settled painting — {differing} \
             channels differ from a full recompose, worst {worst} levels. The relief was swapped for \
             its settled self and nobody asked for the pixels to be lit again."
        );
    }
}

#[test]
fn a_gpu_lane_drain_leaves_no_partial_lane_behind() {
    // The GPU preview producer drains the dirty flag WITHOUT compositing (`take_preview_dirty`) —
    // so the change that raised the flag is never folded into `composited`, and the dirty-rect
    // describing it is consumed by a lane that doesn't read it. If both survive the drain, the
    // next `take_preview_arc` (a GPU→CPU producer handoff: the first impasto dab on a
    // GPU-composited stack) takes the PARTIAL lane over a cache from another era, and hands the
    // bridge a sub-rect bbox — which the bridge trusts to patch the slot ("the fast lane only
    // fires after a full upload synced the texture" is the invariant its B.1 comment states, and
    // this drain used to break it). Today every eligibility-flipping door happens to also
    // invalidate the composite, so the stale blit is latent — held shut by an ENUMERATION of
    // doors, which is exactly the condition that rots when door N+1 arrives. The contract, held
    // here: a `true` GPU drain ⇒ the next CPU drain is a FULL recompose (bbox `None`).
    //
    // **Mutation that must bleed:** in `take_preview_dirty`, keep returning the flag but stop
    // dropping `composited`/`dirty_rect` — the bbox below comes back `Some`.
    //
    // The stack must be NON-trivial: the trivial fast lane hands back the live `canvas_rgba`
    // Arc — a buffer that cannot go stale — and a `Some(bbox)` there is correct. The hazard
    // under gate lives in the COMPOSITE lane's cache. (This gate's first draft was trivial and
    // red in BOTH worlds — a gate that was never seen green.)
    let mut t = white_canvas(64, 6.0);
    let active = t.layers.active().expect("a layer");
    t.set_layer_opacity(active, 0.9);
    let _ = t.take_preview_arc(); // seed the composite cache (composited = Some)
    let _ = t.take_preview_upload_bbox();

    // A stroke marks a rect; the GPU lane drains the flag (a GPU-owned frame).
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 20.0], PointerPhase::Up));
    assert!(
        t.take_preview_dirty(),
        "fixture: the stroke marked the preview dirty"
    );

    // Handoff: the next CPU drain must not trust a cache the GPU frames left behind.
    t.on_canvas_pointer(cp([44.0, 44.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([44.0, 44.0], PointerPhase::Up));
    let drained = t.take_preview_arc();
    assert!(drained.is_some(), "fixture: the second stroke re-dirtied");
    assert_eq!(
        t.take_preview_upload_bbox(),
        None,
        "the first CPU drain after a GPU-lane drain must be a FULL recompose + full upload — a \
         Some(bbox) here is the partial lane running over a composite cache that missed every \
         GPU-owned frame"
    );
}

/// Drive one stroke of `method` and make it PERMANENT the way the artist does — pen-up for the freehand
/// methods, Apply for the five that leave an editable shape behind. Returns the layer's committed relief.
fn sculpt_and_apply(size: u32, method: StrokeMethod, smoothing: f32) -> Vec<f32> {
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 14.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.impasto_depth = 1.0;
    b.impasto_body = 1.0;
    b.impasto_smoothing = smoothing;
    b.stroke_method = method;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    if matches!(method, StrokeMethod::Line) {
        // The Line is a POLYLINE: it is authored click-by-click, not by dragging. Driving it with a
        // drag paints nothing at all — pigment included — which is a fixture that does not contain the
        // phenomenon, not a bug in the tool. (It cost this gate its first red.)
        t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Up));
        t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));
    } else {
        t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
        for i in 1..=6 {
            t.on_canvas_pointer(cp([30.0 + 10.0 * i as f32, 60.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));
    }
    t.commit_open_shape(); // no-op for the methods that already committed at pen-up
    let id = t.layers.active().expect("a layer");
    t.heights
        .get(&id)
        .map(|f| f.as_ref().clone())
        .unwrap_or_default()
}

#[test]
fn impasto_every_stroke_method_settles_its_relief_and_then_keeps_it() {
    // Enio's smoke, 2026-07-12: *"smoothing nem sempre se aplica no fim do traço"*. He is right, and the
    // word that mattered was **nem sempre**.
    //
    // Smoothing is applied in exactly ONE place — `rebuild_live_layer_relief`, reached from
    // `commit_stroke_height` — so the arithmetic was never the suspect. What varies is whether that
    // commit RUNS. The five SHAPE methods (Line / Arc / Ellipse / Polygon / Free Hand) deliberately keep
    // the stroke OPEN at pen-up (the shape stays editable until Apply), so `close_stroke` never fired for
    // them: the light went on reading the raw envelope, and Smoothing — and the whole live Body card —
    // silently did nothing. The freehand methods commit at pen-up and settled fine. Hence *sometimes*.
    //
    // And the relief was doing worse than not settling: with nothing owning it, the next `pen-down`
    // wiped it (`reset_stroke_height`). Apply a curve, start another stroke, and the thickness of the
    // first EVAPORATED — the pigment stayed, the body did not. Measured, and it is the third claim here.
    //
    // The gate is a TABLE over every method the brush has, because the bug was never in the code that
    // was written — it was in the paths nobody connected. A sixth shape added tomorrow without a commit
    // goes red here.
    let size = 120u32;
    for method in [
        StrokeMethod::Space,
        StrokeMethod::Dots,
        StrokeMethod::Airbrush,
        StrokeMethod::Anchored,
        StrokeMethod::DragDot,
        StrokeMethod::Line,
        StrokeMethod::Arc,
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
        StrokeMethod::FreeHand,
    ] {
        // 1. The relief is COMMITTED to the layer — it lives with the pixels, not in an envelope that
        //    the next pen-down throws away.
        let raw = sculpt_and_apply(size, method, 0.0);
        let settled = sculpt_and_apply(size, method, 1.0);
        assert!(
            raw.iter().any(|&h| h > 0.5),
            "{method:?}: the stroke left relief on the layer"
        );

        // 2. Smoothing ACTS on it. (Stated as "the knob changes the sculpture", not as a shape: what
        //    the settle does to a plateau is soften its walls, and pinning a number here would be
        //    pinning the blur's radius, not the artist's claim.)
        assert_eq!(raw.len(), settled.len(), "{method:?}: same canvas");
        let moved = raw
            .iter()
            .zip(settled.iter())
            .filter(|(a, b)| (*a - *b).abs() > 1e-3)
            .count();
        assert!(
            moved > 50,
            "{method:?}: Smoothing must SETTLE the deposit — only {moved} texels moved between \
             Smoothing 0 and 1, so the knob is dead on this method (it is applied at COMMIT, and the \
             shape methods never used to commit)"
        );
        // …and it settles by SOFTENING: the sculpture cannot come out taller for being smoothed.
        let peak = |f: &[f32]| f.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            peak(&settled) <= peak(&raw) + 1e-4,
            "{method:?}: settling is a blur, not a gain ({} vs {})",
            peak(&settled),
            peak(&raw)
        );
    }
}

#[test]
fn impasto_an_applied_shape_keeps_its_body_when_the_next_stroke_lands() {
    // The half of the bug the artist would have found AFTER the smoothing one, and cursed louder: a
    // shape's relief lived in `stroke_height` with no owner, so the next pen-down's `reset_stroke_height`
    // deleted it. The curve stayed painted and went FLAT.
    let size = 120u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 14.0;
    b.impasto_depth = 1.0;
    b.stroke_method = StrokeMethod::Arc; // an editable shape: the family that never committed
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    for i in 1..=6 {
        t.on_canvas_pointer(cp([30.0 + 10.0 * i as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));
    t.commit_open_shape();
    let on_the_shape = t.composed_relief_at(60, 46);
    assert!(
        on_the_shape > 0.3,
        "the applied shape has body ({on_the_shape})"
    );

    // Now paint somewhere else entirely.
    let mut b2 = t.paint.brush;
    b2.stroke_method = StrokeMethod::Space;
    t.paint.brush = b2;
    t.paint.brush_by_mode.fill(b2);
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Up));

    assert!(
        (t.composed_relief_at(60, 46) - on_the_shape).abs() < 1e-5,
        "…and the shape still has it after the next stroke lands ({} vs {on_the_shape}) — the pigment \
         used to stay while the body evaporated",
        t.composed_relief_at(60, 46)
    );
}

#[test]
fn impasto_a_cancelled_shape_leaves_no_ghost_ridge() {
    // The mirror image, and the reason the fix is TWO lines and not one: Esc peels the pixels back to
    // pristine, so the relief has to go with them. An envelope that survived the cancel would be a ridge
    // the light shades over paint that is no longer there — the same ghost the Eraser gate refuses,
    // arriving through the Esc key.
    let size = 120u32;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 14.0;
    b.impasto_depth = 1.0;
    b.stroke_method = StrokeMethod::Arc;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
    for i in 1..=6 {
        t.on_canvas_pointer(cp([30.0 + 10.0 * i as f32, 60.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([90.0, 60.0], PointerPhase::Up));
    assert!(
        t.composed_relief_at(60, 46) > 0.3,
        "the shape is standing there before the Esc"
    );

    assert!(t.cancel_open_shape(), "Esc drops the open shape");

    assert_eq!(
        t.composed_relief_at(60, 46),
        0.0,
        "…and takes its body with it: a cancelled shape may not leave a ridge behind"
    );
}

#[test]
fn impasto_the_stroke_commit_is_cropped_to_the_stroke_and_byte_identical() {
    // The pen-up used to re-derive, SETTLE, re-base and diff over the **whole canvas** for a stroke that
    // touched a corner of it. Measured 2026-07-12: **1010 ms at 4096²** — a full second of freeze at the
    // end of every stroke, and the kill-criterion never saw it because it only ever timed the `Move`.
    //
    // The commit now works inside a window: the stroke's dab footprint, grown by the settle's reach. That
    // is not an approximation and the gate says so in the only way that cannot be argued with — the
    // cropped commit must be **BIT-FOR-BIT** what the whole-canvas one produced. It can be, because the
    // relief outside the window is exactly zero, the blur of zeros is zeros, and a box blur that clamps at
    // a border of zeros reads the same zeros the whole-canvas pass read from beyond it.
    //
    // The reference is not a re-implementation (an oracle that re-derives the thing it tests agrees with
    // the bug): it is the SAME `derive_height` and the SAME `settle`, run over the full canvas — i.e.
    // literally the code path this replaced.
    //
    // The stroke deliberately runs OFF THE CANVAS EDGE, which is where a crop is most likely to differ:
    // there the whole-canvas settle clamps against the canvas border, and the window's border IS that
    // border.
    let size = 200u32;
    let n = (size * size) as usize;
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 10.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.impasto_depth = 0.9;
    b.impasto_body = 0.4;
    b.impasto_smoothing = 1.0; // the settle at full reach: the whole point of the window
    b.impasto_source = DepthSource::Grain; // per-texel grain: no smooth field to hide a seam in
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);

    // A first stroke, so the second one has a GROUND to be re-based onto (the patch of the layer's
    // pre-stroke relief — the other thing the crop replaced, and the 64 MB clone it removed).
    t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 40.0], PointerPhase::Up));
    let layer = t.layers.active().expect("a layer");
    let before: Vec<f32> = t.heights.get(&layer).expect("a ground").as_ref().clone();

    // The second stroke, running off the left edge.
    t.on_canvas_pointer(cp([30.0, 70.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([10.0, 70.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([-6.0, 70.0], PointerPhase::Up));

    // The window really is a window — this is the perf claim, made executable. The ground is kept as a
    // PATCH of it, so its size IS the crop.
    // The window is the stroke's own footprint plus a CONSTANT pad (the settle's reach and the
    // displacement's widest bank) — so it is a function of the STROKE, never of the canvas, which is what
    // keeps the commit `O(stroke)`. On this 200² fixture a short stroke plus that pad lands around a fifth
    // of the canvas; on a 4096² one it is the same window.
    let cells = t.paint.relief.live_relief_base.len();
    assert!(
        cells > 0 && cells * 3 < n,
        "the commit works in a window, not on the canvas: the ground patch is {cells} of {n} texels"
    );

    // …and the window is byte-exact. Re-run the OLD pipeline over the whole canvas and demand equality.
    let brush = t.paint.brush;
    let mut reference: Vec<f32> = t
        .paint
        .relief
        .live_paint
        .iter()
        .zip(t.paint.relief.live_grain.iter())
        .map(|(&m, &g)| ph2d_painter_brush::height::derive_height(&brush, m, f32::from(g) / 255.0))
        .collect();
    super::impasto_settle::settle(
        &mut reference,
        size,
        size,
        brush.effective_impasto_smoothing(),
    );
    // The STORED field's only bound is the sanity guard — the glass ceiling is a display transform now
    // (`impasto_ceiling::soft_ceiling`), applied at the light, so the buffer holds the true relief.
    let ceil = super::impasto_ceiling::H_MAX;
    for (r, base) in reference.iter_mut().zip(before.iter()) {
        *r = (*r + base).clamp(-ceil, ceil);
    }

    let got = t
        .heights
        .get(&layer)
        .expect("the stroke committed")
        .as_ref();
    let worst = got
        .iter()
        .zip(reference.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert_eq!(
        got, &reference,
        "the cropped commit must be BIT-FOR-BIT the whole-canvas one (worst texel off by {worst})"
    );
}

#[test]
fn an_undo_snapshot_never_copies_the_pixels_of_a_layer_the_stroke_did_not_touch() {
    // Found while closing the Impasto pen-up (2026-07-12) and NOT an impasto bug — it is the Painter's
    // own undo, and it has been there the whole time. A stroke touches exactly ONE layer (the active one,
    // whose pixels live in `canvas_rgba`); every other layer's pixels sit in `images` and do not move.
    // The snapshot deep-cloned them anyway — all of them, on every stroke.
    //
    // Measured at 4096², steady state:
    //
    //     1 layer  ->  pen-up  7.6 ms, 0 MB copied
    //     3 layers ->  pen-up 31.6 ms, 128 MB copied per snapshot
    //     5 layers ->  pen-up 56.1 ms, 256 MB copied per snapshot
    //
    // …and an undo ENTRY keeps two snapshots (before + after), with a cap of 300 entries. A 5-layer 4K
    // painting was spending half a gigabyte per brush stroke duplicating layers nobody had painted on.
    // It does not survive a real painting; it runs out of memory long before it runs out of undo.
    //
    // The fix is the `Arc`, and this is the claim that says so — in the only terms that cannot drift:
    // the snapshot's buffer must be the SAME ALLOCATION as the live one. Not "equal": the same. A gate
    // on timing would be machine-dependent and a gate on equality would pass a deep copy.
    let size = 64u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.add_raster_layer("B");
    t.add_raster_layer("C");
    let ids: Vec<_> = t.layers.all_ids().collect();

    // Paint something on each layer, so `images` really holds pixels for the inactive ones…
    t.set_brush_size_px(8.0);
    for &id in &ids {
        t.select_layer(id);
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    }
    assert!(
        !t.images.is_empty(),
        "the inactive layers carry pixels — otherwise this gate is about nothing"
    );

    // …then snapshot, exactly as a stroke's undo entry does.
    let snap = t.snapshot_model();

    for (id, live) in &t.images {
        let taken = snap.images.get(id).expect("the snapshot keeps every layer");
        assert!(
            std::sync::Arc::ptr_eq(live, taken),
            "layer {id:?}: the snapshot must SHARE the pixels of a layer the stroke never touched, not \
             copy them (this is 64 MB per layer per stroke at 4096, twice per undo entry, 300 entries deep)"
        );
    }
}

// ── Impasto (#16) — VOLUME CONSERVATION: the brush shoves paint aside (plan §13) ───────────────────

/// A canvas with a thick slab of paint already on it — the ground a second stroke has to plough through.
fn slab_canvas(size: u32) -> (PainterTool, RtLayerId) {
    let mut t = impasto_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 30.0;
    b.hardness = 1.0;
    b.falloff = Falloff::Constant; // a flat slab: a level ground makes the ridge unambiguous
    // Depth 0.3, not 1.0, and that is not timidity. A narrow rim receiving the bite of a wide brush can
    // pile past the GLASS CEILING (`H_CEIL` — Corel's "pressed against glass"), and paint clamped at the
    // ceiling is paint LOST. That ceiling is real and deliberate; conservation is a statement about the
    // displacement, and it can only be made where the ceiling is not in the way.
    b.impasto_depth = 0.3;
    b.impasto_body = 1.0;
    b.impasto_smoothing = 0.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    // A broad field of thick paint across the middle of the canvas.
    t.on_canvas_pointer(cp([20.0, 80.0], PointerPhase::Down));
    for x in 1..=8 {
        t.on_canvas_pointer(cp([20.0 + 15.0 * x as f32, 80.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([140.0, 80.0], PointerPhase::Up));
    let layer = t.layers.active().expect("a layer");
    (t, layer)
}

/// Total relief on the layer — the *volume* of paint. Conservation is a statement about this number.
fn volume(t: &PainterTool, layer: RtLayerId) -> f32 {
    t.heights.get(&layer).map_or(0.0, |f| f.iter().sum())
}

#[test]
fn impasto_push_conserves_the_paint_it_shoves() {
    // THE claim, and the one that makes this physics rather than an effect: **nothing is created and
    // nothing is destroyed — the paint only moves.** A stroke that ploughs through a slab must leave the
    // canvas holding exactly as much paint as it found.
    //
    // Everything else about Push is a matter of taste and can be argued about with the eyes. This cannot.
    // It is also the property the naive build silently breaks: a "displacement" implemented as a blend or
    // a smear AVERAGES, and averaging loses volume — drag long enough and the sculpture melts. (The Plow
    // knife does exactly that today, deliberately: it drags the body along with the pigment, which is a
    // different gesture. This is the conservative one.)
    let size = 200u32;
    let (mut t, layer) = slab_canvas(size);
    let before = volume(&t, layer);
    assert!(before > 100.0, "there is a real slab to plough ({before})");
    // Volume conservation is a statement about the STORED field, and the only thing that clamps it is the
    // sanity guard (the glass ceiling compresses the *appearance*, at the light, and takes no volume away).
    let ceiling = super::impasto_ceiling::H_MAX;
    assert!(
        t.heights
            .get(&layer)
            .expect("relief")
            .iter()
            .all(|h| h.abs() < ceiling * 0.9),
        "…and it is nowhere near the sanity bound, so nothing can be clamped away"
    );

    // A second stroke, crossing it, with the brush shoving everything aside.
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 0.0; // deposit NOTHING: isolate the displacement from the deposit
    b.impasto_push = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
    for y in 1..=6 {
        t.on_canvas_pointer(cp([80.0, 40.0 + 12.0 * y as f32], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));

    let after = volume(&t, layer);
    let drift = (after - before).abs() / before;
    assert!(
        drift < 1e-3,
        "the paint is conserved: {before} of it went in, {after} came out ({:.2}% drift). A \
         displacement that loses volume is a smear, not a plough",
        drift * 100.0
    );
}

#[test]
fn impasto_push_ploughs_a_channel_and_stands_the_paint_up_at_its_edges() {
    // The percept, stated as the artist sees it: drag a brush through thick paint and it leaves a CHANNEL
    // with RIDGES along its edges. Conservation alone does not give you that — a pass that took paint from
    // the middle and spread it uniformly over the whole canvas would conserve perfectly and look like
    // nothing at all. So: the paint under the stroke goes DOWN, and the paint just outside it goes UP.
    let size = 200u32;
    let (mut t, layer) = slab_canvas(size);
    let at = |t: &PainterTool, x: u32, y: u32| {
        t.heights
            .get(&layer)
            .map_or(0.0, |f| f[(y * size + x) as usize])
    };
    let (in_path, at_rim) = (at(&t, 80, 80), at(&t, 80 + 14, 80));
    assert!(
        in_path > 0.2 && at_rim > 0.2,
        "the slab is level to begin with ({in_path} in the path, {at_rim} beside it)"
    );

    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 0.0; // no deposit: what changes is the ground, and only the ground
    b.impasto_push = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
    for y in 1..=6 {
        t.on_canvas_pointer(cp([80.0, 40.0 + 12.0 * y as f32], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));

    let channel = at(&t, 80, 80);
    let ridge = at(&t, 80 + 14, 80);
    assert!(
        channel < in_path * 0.25,
        "the brush ploughs a CHANNEL where it passed ({in_path} → {channel})"
    );
    assert!(
        ridge > at_rim * 1.05,
        "…and the paint it displaced STANDS UP beside it ({at_rim} → {ridge}) — without this the \
         volume is conserved and the eye sees nothing"
    );
}

#[test]
fn impasto_push_is_live_on_a_stroke_that_was_laid_with_it_armed() {
    // Push is derived from the stroke's FOOTPRINT, not driven along its path — which is what lets it be a
    // pure function of `(ground, footprint)`. Three things follow, and all three are the point:
    //
    //  1. It is LIVE: dial it after the stroke and the ridge grows and shrinks under the artist's hand.
    //  2. It is IDEMPOTENT: re-deriving never eats the ground a second time. A destructive per-dab plough
    //     would, and the SHAPE editors — which re-stamp the whole shape on every pointer move — would
    //     have carved a canyon in a couple of seconds. That is the whole reason for the design.
    //  3. It is REVERSIBLE: back to 0 gives the ground back bit for bit.
    //
    // ⚠️ **The PRECONDITION changed on 2026-08-06 and this gate now states it:** the stroke has to have
    // been laid with the knob ARMED. Laying it at Push 0 records no ingredient, so there is nothing to
    // re-derive later — see the sibling `a_stroke_laid_with_push_off_has_no_ingredient_to_re_derive`,
    // which pins the other half deliberately. The capability did not go away; its precondition moved.
    let size = 200u32;
    let (mut t, layer) = slab_canvas(size);
    // Claim 1 is that Push is LIVE on the finished stroke, so the fixture asks for live editing instead
    // of inheriting it — the default is the artist's and went OFF on 2026-07-19.
    t.paint.impasto_live_edit = true;
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 0.5; // a loaded brush, as an artist would hold it
    b.impasto_push = 0.5; // …with the knob ARMED, which is what records the ingredient
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
    for y in 1..=6 {
        t.on_canvas_pointer(cp([80.0, 40.0 + 12.0 * y as f32], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));

    let rim = |f: &[f32]| f[(80 * size + 80 + 14) as usize];
    let vol = |f: &[f32]| f.iter().sum::<f32>();

    // The knob at rest, on a stroke that DID record its ingredient.
    t.set_brush_impasto_push(0.0);
    let zero: Vec<f32> = t.heights.get(&layer).expect("relief").as_ref().clone();

    // Reach for the knob — AFTER the stroke.
    t.set_brush_impasto_push(1.0);
    let pushed: Vec<f32> = t.heights.get(&layer).expect("relief").as_ref().clone();
    assert!(
        rim(&pushed) > rim(&zero) * 1.05,
        "the ridge rises under the artist's hand, on a stroke already laid ({} → {})",
        rim(&zero),
        rim(&pushed)
    );

    // Re-derive a dozen times. The ground is never eaten twice.
    for _ in 0..12 {
        t.set_brush_impasto_push(1.0);
    }
    let again: Vec<f32> = t.heights.get(&layer).expect("relief").as_ref().clone();
    assert_eq!(
        again, pushed,
        "idempotent: re-deriving does not plough the same ground again (a destructive per-dab plough \
         would have dug a canyon here, and the shape editors re-stamp every pointer move)"
    );

    // …and all the way back down: the field returns EXACTLY to what it was before the knob was raised.
    t.set_brush_impasto_push(0.0);
    let back: Vec<f32> = t.heights.get(&layer).expect("relief").as_ref().clone();
    assert_eq!(
        back,
        zero,
        "Push 0 gives the ground back bit for bit — it displaces, it does not destroy (volume \
         {} vs {})",
        vol(&back),
        vol(&zero)
    );
}

#[test]
fn a_stroke_laid_with_push_off_has_no_ingredient_to_re_derive() {
    // The other half of the trade Enio took on 2026-08-06, pinned so nobody restores the old gate by
    // accident and quietly puts ~30% back on every stroke.
    //
    // **The measurement that bought it:** the bite used to be armed whenever the LAYER had relief, and
    // `impasto_push` defaults to **0.0** — so every stroke of a normal brush, on any layer with relief
    // anywhere, paid the whole bow-wave walk for a feature that was off. Measured at the product's
    // radius (`measure_impasto_cost::what_the_height_walk_is_made_of`, 4096², r=185): the tail cost
    // **53,74 ms** crossing the BARE part of a dirty layer and **54,79** over its own paint — the same
    // — against **15,40** on a virgin layer. Gating on the knob takes the height walk from 136,64 to
    // **96,93 ms**.
    //
    // ⚠️ **The frame is byte-identical**, and that is why the trade is only about the FUTURE: the
    // re-derivation is `field[i] = deposit + push * push_plane[i]`, so at `push == 0` everything the
    // bite would have written is multiplied out anyway. What the artist loses is reaching for the knob
    // *afterwards* — Push became a before-the-stroke decision, and it is now the one knob in the Body
    // card that is not live. That exception is deliberate and has a number next to it.
    let size = 200u32;
    let (mut t, layer) = slab_canvas(size);
    t.paint.impasto_live_edit = true;
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 0.5;
    b.impasto_push = 0.0; // the knob OFF — the default a normal brush ships with
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
    for y in 1..=6 {
        t.on_canvas_pointer(cp([80.0, 40.0 + 12.0 * y as f32], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));

    let off: Vec<f32> = t.heights.get(&layer).expect("relief").as_ref().clone();
    t.set_brush_impasto_push(1.0);
    let after: Vec<f32> = t.heights.get(&layer).expect("relief").as_ref().clone();
    assert_eq!(
        after, off,
        "a stroke laid with Push OFF has no ingredient, so raising the knob afterwards re-derives \
         nothing — the ~30% it would have cost was not spent"
    );
}

#[test]
fn impasto_push_banks_the_ridge_while_the_brush_is_still_moving() {
    // Enio's smoke, 2026-07-12: *"funcionou mas não em tempo real. Apenas no mouse up. Precisa ser em
    // tempo real."*
    //
    // He is right, and it was a direct consequence of the design, not an oversight: the first build
    // derived the displacement from the stroke's whole FOOTPRINT at commit — pure, idempotent, live — and
    // a footprint only exists once the stroke is over. The artist was painting blind and finding out what
    // they had done at pen-up.
    //
    // The rebuild banks per DAB, into a plane that is the displacement at `Push = 1`. Because the whole
    // thing is LINEAR in Push, that plane is an ingredient like any other: the ridge appears under the
    // brush as it moves, AND the knob stays live afterwards. This gate is the first half — mid-drag, with
    // the pen still down, the ridge is already standing.
    let size = 200u32;
    let (mut t, _layer) = slab_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    b.impasto_depth = 0.0; // a dry brush: what changes is the GROUND, and only the ground
    b.impasto_push = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    let rim_before = t.composed_relief_at(80 + 14, 80);
    assert!(rim_before > 0.2, "the slab is there ({rim_before})");

    // Drag ACROSS the slab — and stop, with the pen still DOWN.
    t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
    for y in 1..=6 {
        t.on_canvas_pointer(cp([80.0, 40.0 + 12.0 * y as f32], PointerPhase::Move));
    }

    assert!(
        t.composed_relief_at(80, 80) < 0.25 * rim_before,
        "mid-drag, the channel is already cut ({})",
        t.composed_relief_at(80, 80)
    );
    assert!(
        t.composed_relief_at(80 + 14, 80) > rim_before * 1.05,
        "…and the ridge is already standing beside it ({} → {}). The pen has NOT been lifted.",
        rim_before,
        t.composed_relief_at(80 + 14, 80)
    );
}

#[test]
fn impasto_push_banks_a_smooth_ridge_with_no_crease_scored_along_it() {
    // Enio's smoke, same day: *"a tinta deslocada ficou com bordas duras."*
    //
    // The first build shaped the rim as `blur(footprint) − footprint`, with a BOX blur. A box blur of a
    // step is a *linear ramp*: continuous, but with a **discontinuous derivative**. The light reads the
    // derivative of the height — so the ridge came out with a hard crease scored along it, exactly at the
    // blur's radius. The rim is now an analytic `C¹` profile (`push_rim_weight`), which has no line to
    // draw and costs no blur at all.
    //
    // The gate measures what the LIGHT measures: the second difference of the height across the bank. A
    // crease is a spike in it, and no amount of squinting at the first difference would find one.
    let size = 200u32;
    let (mut t, layer) = slab_canvas(size);
    let mut b = t.paint.brush;
    b.radius_px = 12.0;
    // A SOFT brush — the one an artist actually holds, and the one Enio was holding. (The slab was laid
    // with a hard disk on purpose, to give a level ground; ploughing with a hard disk would cut a
    // hard-edged channel *by construction*, which is physically right and would say nothing about the
    // rim. The claim here is about the BANK's own profile.)
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth;
    b.impasto_depth = 0.0;
    b.impasto_smoothing = 0.0; // no settle to hide behind: the RIM's own shape is on trial
    b.impasto_push = 1.0;
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.on_canvas_pointer(cp([80.0, 40.0], PointerPhase::Down));
    for y in 1..=6 {
        t.on_canvas_pointer(cp([80.0, 40.0 + 12.0 * y as f32], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([80.0, 120.0], PointerPhase::Up));

    // A cut straight across the stroke, through the bank, at the stroke's mid-height.
    let f = t.heights.get(&layer).expect("relief");
    let row: Vec<f32> = (80..110).map(|x| f[(80 * size + x) as usize]).collect();
    let ridge = row.iter().cloned().fold(0.0f32, f32::max);
    assert!(ridge > 0.2, "there IS a bank to inspect ({ridge})");

    // The crease: the largest kink in the profile. A box-blurred rim scores one at its radius; a C¹
    // profile has none. Stated relative to the ridge, so it says nothing about how TALL the bank is.
    let mut worst_kink = 0.0f32;
    for w in row.windows(3) {
        worst_kink = worst_kink.max((w[0] - 2.0 * w[1] + w[2]).abs());
    }
    assert!(
        worst_kink < ridge * 0.10,
        "the bank must be SMOOTH — the worst kink across it is {worst_kink} against a ridge of {ridge} \
         ({:.0}% of it). The light reads the derivative of the height: a kink is a hard line drawn on \
         the paint.",
        worst_kink / ridge * 100.0
    );
}

/// **The deposit's bow wave, through the REAL stroke** — the tool-level half of the frontier law
/// (`ph2d-painter-brush`'s `the_ploughed_paint_waits_at_the_strokes_frontier` proves the kernels;
/// this proves the LOOP that drives them: un-paint the standing lobe BEFORE the dab deposits, one
/// wave per Symmetry copy, reset with the stroke). Plough a Push=1 stroke through committed thick
/// paint and read the displacement plane mid-stroke, the moment the artist is looking at it.
///
/// **Mutation that must bleed:** drop the un-paint block in `stamp_dabs_height` (the standing lobe
/// is laid and never taken back) — a fossil lobe trail down the whole channel; the swath bound
/// explodes and the frontier share collapses.
#[test]
fn the_deposits_wave_travels_through_the_real_stroke() {
    let size = 300u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(30.0);
    t.toggle_brush_impasto();
    // The slab: thick committed paint for the plough to shove.
    for y in [130.0f32, 150.0, 170.0] {
        t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp([40.0 + 28.0 * i as f32, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([264.0, y], PointerPhase::Up));
    }
    // The plough, mid-stroke (captured before the Up — real-time is the law here).
    t.set_brush_size_px(14.0);
    t.set_brush_impasto_push(1.0);
    let (x0, x1, y) = (70.0f32, 220.0f32, 150.0f32);
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    for i in 1..=10 {
        t.on_canvas_pointer(cp([x0 + 15.0 * i as f32, y], PointerPhase::Move));
    }
    let plane = t.paint.relief.stroke_push.clone();
    // THE TIP IS THE LAST DAB, not the pointer: the stabilizer holds the stroke ~a radius behind
    // the cursor, and this gate's first draft measured "ahead" from the pointer — red over a wave
    // that was standing exactly where it should. The wave state itself says where the tip is.
    let tip = t.paint.relief.stroke_wave.first().and_then(|(_, t)| *t);
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
    assert_eq!(
        plane.len(),
        (size * size) as usize,
        "fixture: the plough recorded R1"
    );
    let tip = tip.expect("fixture: the plough carried a wave (tip recorded)");
    let tip_x = tip.center[0];

    let radius = 14.0f32;
    // The rim now anchors at the paint's BODY edge (`t0·radius`), not the dab's geometric rim
    // (2026-07-15) — so "ahead of the stroke" is measured against the PAINT's frontier, exactly as the
    // kernel's zone gate does. Against `radius` the ridge butting the body's front would be miscounted
    // as sitting inside the channel. On the default Smooth brush `t0 ≈ 0.60`, so the edge is ~8.4 px.
    let edge = ph2d_painter_brush::height_push::rim_t0(&t.paint.brush, false) * radius;
    let (mut ahead, mut behind, mut lateral, mut total) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for py in 0..size as usize {
        for px in 0..size as usize {
            let v = plane[py * size as usize + px];
            if v <= 0.0 {
                continue;
            }
            total += f64::from(v);
            let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
            if fx > tip_x + edge {
                ahead += f64::from(v);
            } else if fx < x0 - edge {
                behind += f64::from(v);
            } else if (fy - y).abs() > edge {
                lateral += f64::from(v);
            }
        }
    }
    assert!(
        total > 10.0,
        "fixture: the plough banked almost nothing ({total:.1}) — no ground under it?"
    );
    let swath = total - ahead - behind - lateral;
    assert!(
        ahead / total >= 0.30,
        "only {:.1}% of the shoved paint stands ahead of the tip mid-stroke — the wave is not \
         travelling through the REAL stroke (kernel gate green means the loop is the suspect: \
         warm-up, per-copy slot, or the un-paint order)",
        100.0 * ahead / total
    );
    assert!(
        swath / total <= 0.20,
        "{:.1}% of the shoved paint sits inside the channel — a fossil lobe trail: the standing \
         lobe is being laid but never taken back up as the tip moves",
        100.0 * swath / total
    );
    let _ = behind;
}
