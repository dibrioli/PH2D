//! Gates do **GATE DE PROTEÇÃO** — a tinta atravessando uma máscara, não a máscara em si
//! (`docs/Painter/25_avaliacao_gpu.md` §13.12 + **§13.13**). Irmão de `mask_tests`, que fica com a
//! cobertura da própria máscara; separado dele quando as duas waves juntas passaram do teto de 700 linhas.
//!
//! ## As duas leis que estes gates pinam, e por quê
//!
//! 1. **A proteção vale UMA vez por texel** (§13.12). Ela era aplicada uma vez por BATCH, então a força
//!    dela seguia a taxa de polling do mouse — 0,886 de tinta passava a 4 eventos por traço e 0,992 a 60,
//!    com o contorno andando 4 px por nada.
//! 2. **A proteção nunca ERODE** (§13.13, ordem do Enio). Cada traço era escalado separadamente, então N
//!    traços deixavam passar `1 − (1−keep)^N`: num texel meio-protegido, 4 passadas deixavam 0,949 e
//!    **8 deixavam 1,000**. É a semântica de **layer mask / alpha lock**, não a do sculpt mask do Blender.

use super::mask_probe::{coverage, cp, vstroke};
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_editor_core::tool::{
    CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool as _,
};

/// Canvas for the paint-through-protection fixtures. Smaller than `S` on purpose: these run a control
/// tool beside every measured one, so the fixture is paid for twice.
const G: u32 = 192;

/// Paint BLACK horizontal strokes across a protection whose feather runs down the middle, and report
/// `(paint, keep)` — both as fractions, both read the way the artist reads them (`paint` = how far the
/// texel travelled from the white canvas toward the ink; `keep` = what the mask overlay says it lets
/// through).
///
/// `mask_y0` places the protection: **30** puts its feather right across the paint band (the measured
/// case), **170** puts the identical gesture BELOW it (the control — same stroke count, same rng
/// history, same everything, but `keep == 1` where we measure). ⚠️ The mask brush's radius here is 40 px,
/// not 20: `set_brush_size_px` is the RADIUS, and the first control at y0 = 140 still feathered into the
/// band (keep 0.9961 at y = 101), which the gate's own fixture assertion caught.
///
/// `events` is the number of pointer Moves the stroke is delivered in — the mouse's report rate, which
/// nothing the artist did should be able to feel.
fn paint_through_protection(events: u32, strokes: u32, mask_y0: f32) -> (Vec<f32>, Vec<f32>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    // The protection: one soft vertical mask stroke (the feather is the whole point — a hard edge has
    // no partial-keep texels, and partial keep is where the reported crackle lives).
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(40.0);
    vstroke(&mut t, 96.0, mask_y0, 162.0, 24);
    // `keep = 1 − coverage`: the overlay's own definition. It agrees with the compositor's Rec.601
    // `mask_value` to the bit here because a mask stroke is desaturated by construction (R == G == B),
    // so this is the artist's number and not a second copy of the product's arithmetic.
    let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(16.0);
    for k in 0..strokes {
        let y = 96.0 + k as f32 * 11.0;
        t.on_canvas_pointer(cp([20.0, y], PointerPhase::Down));
        for i in 1..=events {
            let x = 20.0 + 152.0 * (i as f32) / (events as f32);
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([172.0, y], PointerPhase::Up));
        let _ = t.take_preview_arc();
    }
    let n = (G as usize) * (G as usize);
    let paint = (0..n)
        .map(|i| 1.0 - f32::from(t.canvas_rgba[i * 4]) / 255.0)
        .collect();
    (paint, keep)
}

/// **THE LAW.** A protection that says it lets `keep` through lets exactly `keep` of the paint through —
/// of the paint that WOULD have landed, measured by a control run of the identical gesture with the
/// identical rng history and the feather moved out of the way.
///
/// This is the layer-mask promise, and it is what a per-batch pull-back cannot keep: that door lerped the
/// stamped canvas against a snapshot taken per pointer event, so the paint surviving `N` events was
/// `1 − (1 − keep)^N` — a fact about the mouse, not about the mask (doc 25 §13.11/§13.12).
///
/// **Mutation that must bleed** (measured, reinstalling the old door): worst error **0.446** at 4
/// events/stroke and **0.500** at 60 — the bar is 0.02, and the second number is the whole feather
/// saturating to solid ink.
#[test]
fn the_gate_lets_through_exactly_the_keep_it_declares() {
    for events in [4u32, 60] {
        let (prot, keep) = paint_through_protection(events, 1, 30.0);
        let (free, ctrl_keep) = paint_through_protection(events, 1, 170.0);
        let mut worst = 0.0_f32;
        let mut at = (0usize, 0usize);
        let mut n = 0usize;
        for y in 88..106usize {
            for x in 20..172usize {
                let i = y * G as usize + x;
                // The control has to be UNGATED where we measure, or "what would have landed" is
                // itself gated and the oracle compares two doses of the same medicine.
                assert!(
                    ctrl_keep[i] > 0.999,
                    "fixture: the control's protection reaches the paint band at ({x},{y}) \
                     (keep {:.4}) — move it further away",
                    ctrl_keep[i]
                );
                if free[i] < 0.02 || keep[i] > 0.98 {
                    continue; // no paint to gate here, or nothing gated about this texel
                }
                let expect = free[i] * keep[i];
                let err = (prot[i] - expect).abs();
                if err > worst {
                    worst = err;
                    at = (x, y);
                }
                n += 1;
            }
        }
        assert!(
            n > 300,
            "fixture: the paint has to CROSS the feather — only {n} partially-protected texels \
             carried paint at {events} events/stroke"
        );
        assert!(
            worst <= 0.02,
            "at {events} events/stroke the gate let through {worst:.3} more than the keep it \
             declares (worst at {:?}, over {n} texels): the protection is compounding per batch",
            at
        );
    }
}

/// **THE REPORTED SYMPTOM** (Enio, 2026-07-25, with two photos): *"bordas craqueladas na pintura quando
/// muitas pinceladas são dadas repetidamente"*. Six repeated strokes across the feather, delivered at two
/// very different pointer rates, must produce the SAME picture — and in particular the contour where the
/// paint dies must sit in the same place.
///
/// It is the appearance restatement of the gate above and it earns its keep by not sharing its
/// arithmetic: it never computes `keep · free`, it just compares two renders of the same gesture.
///
/// **Mutation that must bleed** (the old per-batch door): the contour mean moved **4 px** and the paint
/// at half-keep went 0,886 → 0,992 — which is the crackle, because what is left visible is then only the
/// `keep ≈ 0` frontier, cut into rectangles by the batch regions.
#[test]
fn the_protection_is_a_fact_of_the_mask_not_of_the_polling_rate() {
    let (slow, keep) = paint_through_protection(4, 6, 30.0);
    let (fast, _) = paint_through_protection(60, 6, 30.0);
    // (a) the whole painted band, texel by texel.
    let mut worst = 0.0_f32;
    for y in 88..165usize {
        for x in 20..172usize {
            let i = y * G as usize + x;
            worst = worst.max((slow[i] - fast[i]).abs());
        }
    }
    // (b) and the contour the eye actually reads: where the ink drops through half, per scanline.
    let contour = |f: &[f32]| -> f32 {
        let xs: Vec<f32> = (88..165)
            .filter_map(|y| super::mask_probe::cross_x(f, G, y, 0.5))
            .collect();
        assert!(
            xs.len() > 40,
            "fixture: no contour to measure ({})",
            xs.len()
        );
        xs.iter().sum::<f32>() / xs.len() as f32
    };
    let (cs, cf) = (contour(&slow), contour(&fast));
    // The fixture must actually contain partial keep along that contour, else both halves are trivial.
    let partial = (88..165usize)
        .flat_map(|y| (20..172usize).map(move |x| y * G as usize + x))
        .filter(|&i| keep[i] > 0.02 && keep[i] < 0.98 && slow[i] > 0.05)
        .count();
    assert!(
        partial > 400,
        "fixture: only {partial} partially-protected texels carried paint"
    );
    assert!(
        worst <= 0.03,
        "the same six strokes painted at 4 and at 60 events differ by {worst:.3} — the strength of \
         the protection is following the mouse's report rate"
    );
    assert!(
        (cs - cf).abs() <= 1.0,
        "the paint's contour moved {:.2} px between 4 and 60 events/stroke ({cs:.2} vs {cf:.2})",
        (cs - cf).abs()
    );
}

/// The epoch is **born only when it is needed, SURVIVES the stroke, and dies when the thing it describes
/// changes** — the one question that replaced §13.7's 22 hand-maintained commit sites.
///
/// The absence half is the cost argument (a canvas-sized plane per ungated document would be a real
/// regression); the survival half is what makes the ceiling a ceiling; and the four death conditions are
/// the leak that got the épocha reverted, each asked of the witness instead of of a list.
#[test]
fn the_epoch_outlives_the_stroke_and_dies_with_the_protection() {
    let armed = || -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        t.set_brush_size_px(40.0);
        vstroke(&mut t, 96.0, 30.0, 162.0, 24);
        t.set_paint_tool_mode("brush");
        t.set_brush_size_px(16.0);
        t
    };
    let stroke = |t: &mut PainterTool| {
        t.on_canvas_pointer(cp([30.0, 96.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([90.0, 96.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([150.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
    };
    // (a) ungated: never allocated.
    let mut plain = PainterTool::default();
    plain.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    plain.set_brush_size_px(16.0);
    stroke(&mut plain);
    assert!(
        plain.gate.is_none(),
        "an ungated document must not allocate a canvas-sized free plane"
    );
    // (b) gated: alive after the pen is UP — this is the ceiling's whole mechanism.
    let mut t = armed();
    stroke(&mut t);
    assert!(
        t.gate.is_some(),
        "the epoch must OUTLIVE the stroke: one that dies with it lets 1 − (1−keep)^N through and the \
         protection erodes (§13.13)"
    );
    // (c) editing the PROTECTION ends it — and the oracle is the CONSEQUENCE, not the counter: with a
    //     stale epoch the free plane still holds every stroke at FULL strength, so lowering the protection
    //     would retroactively reveal history that was already buried under it (the §13.7 gate
    //     `lowering_protection_starts_a_new_epoch_not_a_replay_of_history`).
    let mut t = armed();
    t.set_brush_color_srgb8([0, 0, 0]);
    let buried = {
        let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
        (60..132usize)
            .map(|x| 96 * G as usize + x)
            .find(|&i| keep[i] < 0.15)
            .expect("fixture: the probe must start HEAVILY protected")
    };
    for _ in 0..6 {
        stroke(&mut t); // a lot of paint that the protection keeps almost entirely out
    }
    let hidden = 1.0 - f32::from(t.canvas_rgba[buried * 4]) / 255.0;
    assert!(
        hidden < 0.25,
        "fixture: the protection has to be HIDING most of the paint, got {hidden:.3}"
    );
    // Now UNPROTECT that area, then paint one more stroke over it.
    t.set_paint_tool_mode("mask");
    t.set_mask_brush(1); // Erase = unprotect
    vstroke(&mut t, 96.0, 30.0, 162.0, 24);
    t.set_mask_brush(0);
    t.set_paint_tool_mode("brush");
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.paint.brush.strength = 0.3;
    stroke(&mut t);
    let after = 1.0 - f32::from(t.canvas_rgba[buried * 4]) / 255.0;
    assert!(
        after < hidden + 0.5,
        "unprotecting REVEALED buried history: {hidden:.3} -> {after:.3} in one 30 %-strength stroke. \
         The epoch's free plane still held six strokes at full strength, so the new keep exposed them \
         all at once instead of letting the artist build up from what was on the canvas"
    );
    // (c2) …and the same for a whole-canvas **Modifier** on the mask (Invert / Blur / Clear), which is the
    //      path where the generation is the ONLY witness: `mask_canvas_op` never calls `mark_dirty`, so the
    //      pixel clock does not move (the mask STROKE above moves it, which made the first version of this
    //      gate survive its mutation — a layered defence needs a gate per layer).
    let mut t = armed();
    t.set_brush_color_srgb8([0, 0, 0]);
    let hidden_at = {
        let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
        (60..132usize)
            .map(|x| 96 * G as usize + x)
            .find(|&i| keep[i] < 0.15)
            .expect("fixture: the probe must start HEAVILY protected")
    };
    for _ in 0..6 {
        stroke(&mut t);
    }
    let before_invert = 1.0 - f32::from(t.canvas_rgba[hidden_at * 4]) / 255.0;
    t.mask_canvas_op(4); // Invert: what was protected is now open, and vice versa
    t.paint.brush.strength = 0.3;
    stroke(&mut t);
    let after_invert = 1.0 - f32::from(t.canvas_rgba[hidden_at * 4]) / 255.0;
    assert!(
        after_invert < before_invert + 0.5,
        "a mask Modifier left the epoch standing: {before_invert:.3} -> {after_invert:.3} in one \
         30 %-strength stroke. Inverting the protection dumped the free plane's six buried strokes onto \
         the canvas at once"
    );

    // (d) a FOREIGN canvas write ends it — the class that killed §13.7, caught by the witness rather
    //     than by remembering to call a commit at 22 places. ⚠️ The oracle is that the foreign work
    //     SURVIVES, not that the witness number moved: the number moves on our own writes too, so a gate
    //     that only watched it could not fail for the reason it states (measured — that version survived
    //     the mutation).
    let mut t = armed();
    stroke(&mut t);
    let probe = {
        let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
        (60..132usize)
            .map(|x| 120 * G as usize + x)
            .find(|&i| keep[i] < 0.15)
            .expect("fixture: the probe must be mostly PROTECTED, so `base` dominates what shows")
    };
    // A real member of the class: the Fill (ColorDrop) writes the canvas from its own snapshot, entirely
    // outside the gated stamp — one of the 22 sites §13.7 had to list by hand.
    t.set_brush_color_srgb8([255, 0, 0]);
    t.paint.paint_mode = PaintMode::Fill;
    t.fill_pointer(cp([30.0, 130.0], PointerPhase::Down));
    t.fill_pointer(cp([30.0, 130.0], PointerPhase::Up));
    t.paint.paint_mode = PaintMode::Paint;
    let filled = t.canvas_rgba[probe * 4 + 1]; // the fill is RED ⇒ its green channel is the fingerprint
    assert!(
        filled < 60,
        "fixture: the fill has to reach the probe (green {filled}, expected near 0)"
    );
    t.set_brush_color_srgb8([0, 0, 0]);
    // …and now a gated batch whose region covers the probe. Its projection reads `base`; if the epoch
    // was not re-seeded, `base` is the PRE-fill canvas and the fill is silently undone right here.
    t.on_canvas_pointer(cp([30.0, 120.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([150.0, 120.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([150.0, 120.0], PointerPhase::Up));
    let _ = t.take_preview_arc();
    let after = t.canvas_rgba[probe * 4 + 1];
    assert!(
        after < 90,
        "the foreign Fill was projected away (green {filled} -> {after}, white is 255): the epoch kept \
         a base from before it and blended the canvas back toward the era it remembers"
    );
    // (e) an undo ends it: its base describes a canvas that no longer exists.
    let mut t = armed();
    stroke(&mut t);
    assert!(t.undo_last());
    assert!(
        t.gate.is_none(),
        "an undo must drop the epoch: projecting against a base from the undone era blends the \
         restored pixels back toward what was just undone"
    );
}

/// **THE CEILING** (Enio, 2026-07-25 §13.13) — the law this wave exists for. A protection never erodes: no
/// matter how many strokes cross a half-protected texel, what shows converges on `keep` and never past it.
/// Before this, each stroke was scaled separately and the texel kept `1 − (1−keep)^N` — measured 0.522 at
/// one stroke, 0.949 at four, **1.000 at eight**: the mask stopped protecting anything, under exactly the
/// gesture Enio reported.
///
/// ⚠️ **Both halves, because a ceiling is not a WALL.** The brush must still build UP toward it (a low-flow
/// stroke lands short of `keep` and later strokes approach it) — a cap applied to the BRUSH instead would
/// make the second stroke a silent no-op, which is the shape of *"the brush stopped working"*.
///
/// **Mutation that must bleed:** end the epoch in `close_stroke` (the per-stroke lifetime) ⇒ the ink walks
/// 0.522 → 0.773 → … → 1.000 and half (a) fires.
#[test]
fn the_protection_never_erodes_no_matter_how_many_strokes_cross_it() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(40.0);
    vstroke(&mut t, 96.0, 30.0, 162.0, 24);
    let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
    let probe = (50..140usize)
        .map(|x| 96 * G as usize + x)
        .min_by(|&a, &b| {
            (keep[a] - 0.5)
                .abs()
                .partial_cmp(&(keep[b] - 0.5).abs())
                .unwrap()
        })
        .expect("fixture: the feather has to have a half-protected texel on the band");
    let k = keep[probe];
    assert!(
        (0.3..0.7).contains(&k),
        "fixture: the probe texel must be about half-protected, got {k:.3}"
    );
    // A brush that lands SHORT of the ceiling on the first stroke, so the build-up half is observable.
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(16.0);
    t.paint.brush.strength = 0.35;
    let ink = |t: &PainterTool| 1.0 - f32::from(t.canvas_rgba[probe * 4]) / 255.0;
    let mut seen = Vec::new();
    for _ in 0..12 {
        t.on_canvas_pointer(cp([20.0, 96.0], PointerPhase::Down));
        for i in 1..=8u8 {
            let x = 20.0 + 152.0 * f32::from(i) / 8.0;
            t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([172.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        seen.push(ink(&t));
    }
    // (a) it is a CEILING: nothing ever passes `keep`, however long the artist insists.
    let worst = seen.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        worst <= k + 0.02,
        "the protection eroded: {worst:.3} got through a keep of {k:.3} ({seen:?}) — each stroke is \
         being scaled separately, so N strokes let 1 - (1-keep)^N through"
    );
    // (b) …and NOT a wall: the paint still builds toward it.
    assert!(
        seen[0] < k - 0.05 && seen[11] > seen[0] + 0.05,
        "a ceiling must still let the brush build UP to it: {seen:?} against keep {k:.3} — a flat \
         sequence from the first stroke means the cap landed on the BRUSH instead of on the result"
    );
}

/// **The ceiling holds for a RE-STAMP method too** — Drag Dot, Line, and every shape editor.
///
/// It is a separate gate because those methods undo their own last batch every preview frame, through
/// `restore_region`, whose `mark_dirty` moves the pixel clock — the same signal the epoch uses to detect a
/// FOREIGN write. Until the epoch re-witnessed its own hand there, every preview frame re-seeded it: the
/// ceiling silently reverted to per-gesture for the whole shape family, and each frame paid a canvas
/// clone. A surviving mutation is what exposed it (the free plane could be restored from the wrong source
/// with no observable effect, because it was being thrown away and rebuilt anyway).
///
/// **Mutation that must bleed:** drop the re-witness at the end of `restore_region`.
#[test]
fn the_ceiling_holds_for_a_restamp_method_too() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(40.0);
    vstroke(&mut t, 96.0, 30.0, 162.0, 24);
    let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
    let probe = (50..140usize)
        .map(|x| 96 * G as usize + x)
        .min_by(|&a, &b| {
            (keep[a] - 0.5)
                .abs()
                .partial_cmp(&(keep[b] - 0.5).abs())
                .unwrap()
        })
        .expect("fixture: the feather has to have a half-protected texel");
    let k = keep[probe];
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(40.0);
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::DragDot;
    let mut seen = Vec::new();
    for _ in 0..10 {
        // A whole Drag Dot gesture, dragged (so `restore_region` runs) and committed.
        t.on_canvas_pointer(cp([60.0, 96.0], PointerPhase::Down));
        for x in [80.0, 96.0] {
            t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([96.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        seen.push(1.0 - f32::from(t.canvas_rgba[probe * 4]) / 255.0);
    }
    let worst = seen.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        worst <= k + 0.02,
        "ten Drag Dot gestures eroded the protection: {worst:.3} through a keep of {k:.3} ({seen:?}) — \
         the preview restore is re-seeding the epoch every frame, so the ceiling is per-gesture again"
    );
}

/// The appearance the ceiling buys, and the reason Enio still saw 15 %: with each stroke scaled separately,
/// the half-ink contour sat at a `keep` that depended on how many strokes covered that row (0.2929 for two,
/// 0.2063 for three), so the boundary came out **combed** — measured **1.68 px** peak-to-peak against an
/// arithmetic prediction of 1.64. Under the ceiling the contour is a pure function of `keep`, so the comb
/// collapses to the keep field's own ripple.
///
/// **Mutation that must bleed:** the per-stroke lifetime ⇒ the comb returns to ~1.7 px.
#[test]
fn the_boundary_of_repeated_strokes_is_the_keep_contour_not_a_comb() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(40.0);
    vstroke(&mut t, 96.0, 20.0, 176.0, 24);
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(16.0);
    // Six overlapping strokes: a row is covered by two or by three of them, which is what used to move the
    // contour. The fixture MUST have that variation or the gate cannot fail for the reason it states.
    for j in 0..6 {
        let y = 70.0 + j as f32 * 11.0;
        t.on_canvas_pointer(cp([20.0, y], PointerPhase::Down));
        for i in 1..=10u8 {
            let x = 20.0 + 152.0 * f32::from(i) / 10.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([172.0, y], PointerPhase::Up));
        let _ = t.take_preview_arc();
    }
    let ink: Vec<f32> = (0..(G as usize * G as usize))
        .map(|i| 1.0 - f32::from(t.canvas_rgba[i * 4]) / 255.0)
        .collect();
    let xs: Vec<f32> = (72..126)
        .filter_map(|y| super::mask_probe::cross_x(&ink, G, y, 0.5))
        .collect();
    assert!(
        xs.len() > 40,
        "fixture: no contour to measure ({})",
        xs.len()
    );
    let comb =
        xs.iter().copied().fold(f32::MIN, f32::max) - xs.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        comb <= 0.6,
        "the boundary is combed by {comb:.2} px — the contour is following how many strokes covered \
         each row (Δkeep between N=2 and N=3 is 0.0866; divided by the keep gradient that IS the comb)"
    );
}

/// **The cost of a gated stroke follows its FOOTPRINT, not the canvas** — the projection pass touches the
/// batch region and nothing else. Ratio first (immune to machine drift, and it catches the real failure
/// mode: some pass starting to walk the plane), then a wall-clock kill for everything else.
///
/// Measured in release, and the honest number is stated rather than hidden: a protected stroke's PEN-DOWN
/// is canvas-proportional and always will be — it allocates and fills a canvas-sized free plane, once.
/// 2048²: **7,43 ms** against 3,02 ungated · 4096²: **24,53 ms** against 11,26. The per-MOVE cost is
/// **1,20 / 1,13 ms** (flat) against 0,46 / 0,41 ungated. So one frame at the start of a protected stroke
/// on a 4K canvas is a dropped frame — and it was already one before this wave (the undo snapshot forces
/// its own canvas fork), so this makes it two. Named, measured, and NOT optimised here: the fix is a
/// tile-lazy seed plus a reused allocation, which is a perf wave with its own gates (doc 25 §13.12).
#[test]
fn the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas() {
    let per_move = |size: u32| -> f64 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let c = size as f32 * 0.5;
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        t.set_brush_size_px(120.0);
        vstroke(&mut t, c, c - 200.0, c + 200.0, 20);
        t.set_paint_tool_mode("brush");
        t.set_brush_size_px(120.0);
        t.on_canvas_pointer(cp([c - 200.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let n = 16;
        let t0 = std::time::Instant::now();
        for i in 1..=n {
            t.on_canvas_pointer(cp([c - 200.0 + i as f32 * 12.0, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let dt = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        t.on_canvas_pointer(cp([c + 200.0, c], PointerPhase::Up));
        dt
    };
    let small = per_move(1024);
    let big = per_move(2048);
    assert!(
        big < small * 1.6 + 0.15,
        "a gated move is bounded by the FOOTPRINT: {small:.2} ms @1024² vs {big:.2} ms @2048² \
         (a pass that walked the plane would give 4x)"
    );
    assert!(
        big < 8.0,
        "a gated move must fit in a frame with room: {big:.2} ms @2048² (kill 8 ms)"
    );
}

/// A **re-stamp** method (Drag Dot and every shape editor) puts the canvas back to pristine and re-stamps
/// the whole thing every preview frame. The free plane has to be put back with it — otherwise every
/// position the artist dragged THROUGH stays in it, and the projection keeps showing a fan of ghosts of
/// shapes that are no longer there.
///
/// Oracle: drag a preview across three positions and commit; the result must equal the same preview
/// stamped only at the LAST position. It is the relief channel's `reset_stroke_height` argument, one
/// plane over, and it is why the reset lives inside `restore_region` instead of at its five callers.
///
/// **Mutation that must bleed:** drop the `restore_gate_free()` call from `restore_region`.
#[test]
fn a_restamp_preview_leaves_no_ghost_in_the_free_plane() {
    let run = |through: bool| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        // A NARROW protection (radius 24 ⇒ it covers x 72..120), so the drag row has UNPROTECTED stretches
        // the re-stamp passes over — that is where an earlier stroke's paint can be observed at all.
        t.set_brush_size_px(24.0);
        vstroke(&mut t, 96.0, 30.0, 162.0, 24);
        t.set_paint_tool_mode("brush");
        t.set_brush_color_srgb8([0, 0, 0]);
        t.set_brush_size_px(24.0);
        // An EARLIER stroke in the same epoch, along the SAME row the drag will cross. The epoch outlives
        // strokes (§13.13), so putting the free plane back to `base` on a re-stamp would delete this —
        // which is why the reset restores the batch's own PATCH instead. It has to overlap the drag's
        // footprint or the mutation is invisible to the probe.
        // …and PARTIAL strength, so the earlier stroke's own contribution is a number and not a saturated
        // 1.0 that any later dab would hide.
        t.paint.brush.strength = 0.4;
        t.on_canvas_pointer(cp([40.0, 96.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([150.0, 96.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([150.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::DragDot;
        t.on_canvas_pointer(cp([58.0, 96.0], PointerPhase::Down));
        if through {
            // Dragged across the protection and BACK — the return leg is what makes a corrupted free plane
            // observable at all: the projection only rewrites the CURRENT batch's region, so damage done
            // under an earlier position stays hidden until the artist drags over it again.
            for x in [80.0, 110.0, 140.0, 110.0, 80.0] {
                t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
                let _ = t.take_preview_arc();
            }
        }
        t.on_canvas_pointer(cp([58.0, 96.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([58.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        t.canvas_rgba.as_ref().clone()
    };
    let dragged = run(true);
    let placed = run(false);
    let diff = dragged
        .chunks_exact(4)
        .zip(placed.chunks_exact(4))
        .filter(|(a, b)| a[0].abs_diff(b[0]) > 1)
        .count();
    assert_eq!(
        diff, 0,
        "{diff} texels remember a position the artist dragged THROUGH: the free plane kept the \
         previous preview's paint and the projection is showing ghosts"
    );
    // The fixture must have something to LOSE: an earlier stroke of the same epoch, under the drag's own
    // footprint. `diff == 0` above is what catches its deletion — resetting the free plane to `base`
    // instead of to the batch's patch wipes it, and the return leg of the drag then projects the hole.
    // The fixture must have something to LOSE: an earlier stroke of the same epoch, under the drag's own
    // footprint (measured 0.294 at the probe). Resetting the free plane to `base` instead of to the
    // batch's patch deletes it, and the return leg of the drag then projects the hole — which is what the
    // comparison above catches.
    let earlier = 1.0 - f32::from(placed[(96 * G as usize + 58) * 4]) / 255.0;
    assert!(
        earlier > 0.15,
        "fixture: the earlier stroke has to be visible at the probe (ink {earlier:.3})"
    );
}
