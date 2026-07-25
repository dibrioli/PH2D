//! Gates da **cobertura da máscara** (2026-07-25, doc 25 §13.10).
//!
//! ## A ordem que estes gates pinam
//!
//! *"A máscara deve pintar exatamente como o brush digital normal"* (Enio, depois do smoke). Ela não tem
//! lei própria: roda o MESMO pipeline de dabs, com o MESMO acúmulo per-dab, e a única diferença é o que a
//! cor significa (preto = proteger, branco = desproteger, e o destino é o scratch em vez da camada).
//!
//! ## Por que existiu uma lei própria por algumas horas, e por que ela morreu
//!
//! A borda da máscara endurece sob muitas passadas (o produto per-dab afia a cauda do falloff — medido:
//! band 3,53 px numa passada → 1,38 px em quinze). A cura tentada foi o **envelope do modo Wash do
//! Krita** (`max` por-traço em vez do produto), que de fato mata o endurecimento — **e foi REPROVADA na
//! tela**: sem a saturação do produto, a modulação por-dab do perfil fica visível e o traço sai em
//! **CONTAS** ao longo do ombro. Renderizado nas duas leis, com a mesma sonda
//! ([`super::mask_probe::probe_mask_beading_along_the_axis`]).
//!
//! ⚠️ **A lição de medição que isso deixou:** a modulação foi medida **no EIXO** do traço (6 níveis de
//! 255) e chamada de invisível. As contas não vivem no eixo — vivem no **OMBRO**, onde o perfil é íngreme,
//! e lá a mesma modulação é enorme na aparência. Um número no lugar errado disse o contrário do que a foto
//! dizia (`reference_topic_oracle_discipline`).
//!
//! Então: **as duas leis têm artefato**, e a cura do endurecimento — se voltar à mesa — não é a lei da
//! cobertura. Não reconstrua nenhuma das duas sem um render-and-look que mostre as contas ausentes.

use super::mask_probe::{band_px, coverage, cp, mask_tool, vstroke};
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_editor_core::tool::{
    CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool as _,
};

const S: u32 = 256;

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

/// The session is **born only when it is needed and dies with the stroke** — presence AND absence, both
/// asked mid-stroke, because after the pen is up the answer is `None` either way and a gate that only
/// asks then cannot fail.
///
/// The absence half is the cost argument (a canvas-sized plane per ungated stroke would be a real
/// regression, and an ungated document must be byte-untouched by all of this); the presence half is what
/// proves the plane is actually carrying the stroke rather than being re-seeded per batch.
#[test]
fn a_session_is_born_only_under_a_gate_and_dies_with_the_stroke() {
    // (a) ungated: never allocated.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    t.set_brush_size_px(16.0);
    t.on_canvas_pointer(cp([30.0, 96.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 96.0], PointerPhase::Move));
    assert!(
        t.gate.is_none(),
        "an ungated stroke must not allocate a canvas-sized free plane"
    );
    t.on_canvas_pointer(cp([90.0, 96.0], PointerPhase::Up));
    // (b) gated: alive across batches, gone at pen-up.
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
    t.on_canvas_pointer(cp([30.0, 96.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 96.0], PointerPhase::Move));
    assert!(
        t.gate.is_some(),
        "a stroke crossing the protection must carry a session between batches — without it the \
         keep factor is applied once per batch, which is the bug"
    );
    t.on_canvas_pointer(cp([150.0, 96.0], PointerPhase::Up));
    assert!(
        t.gate.is_none(),
        "the session must die with the stroke: one that outlives it turns protection into a \
         cross-stroke ceiling and caps the plain brush (the reverted epoch of doc 25 §13.7)"
    );
}

/// **The guard against re-introducing the époque of §13.7.** Repeated strokes over the same spot must
/// BUILD UP — which is what the plain digital brush does, and what the artist expects of paint. A session
/// that outlived its stroke would freeze `base` at the first pen-down, so every later stroke would blend
/// back toward it and the ink would be pinned at `keep` forever: protection silently becomes a
/// cross-stroke CEILING, which is exactly the reverted design (and how it capped ordinary paint).
///
/// The structural half of this is `a_session_is_born_only_under_a_gate_and_dies_with_the_stroke`; this is
/// the half an artist could see. **Mutation that must bleed:** drop `end_gate_session` from
/// `close_stroke`.
#[test]
fn repeated_strokes_through_the_feather_build_up_instead_of_converging() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(40.0);
    vstroke(&mut t, 96.0, 30.0, 162.0, 24);
    let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(16.0);
    // A texel the mask half-protects, on the paint band.
    let probe = (60..132usize)
        .map(|x| 96 * G as usize + x)
        .find(|&i| (0.3..0.7).contains(&keep[i]))
        .expect("fixture: the feather has to have a half-protected texel on the band");
    let ink = |t: &PainterTool| 1.0 - f32::from(t.canvas_rgba[probe * 4]) / 255.0;
    let mut seen = Vec::new();
    for _ in 0..3 {
        t.on_canvas_pointer(cp([20.0, 96.0], PointerPhase::Down));
        for i in 1..=8u8 {
            let x = 20.0 + 152.0 * f32::from(i) / 8.0;
            t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([172.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        seen.push(ink(&t));
    }
    assert!(
        seen[1] > seen[0] + 0.05 && seen[2] > seen[1] + 0.02,
        "painting the same protected texel again has to deepen it, as the plain brush does: {seen:?} \
         — a flat sequence means the session outlived its stroke and protection became a ceiling"
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
        t.set_brush_size_px(40.0);
        vstroke(&mut t, 96.0, 30.0, 162.0, 24);
        t.set_paint_tool_mode("brush");
        t.set_brush_color_srgb8([0, 0, 0]);
        t.set_brush_size_px(24.0);
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::DragDot;
        t.on_canvas_pointer(cp([60.0, 96.0], PointerPhase::Down));
        if through {
            // …dragged across the feather and back out the other side.
            for x in [70.0, 96.0, 120.0] {
                t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
                let _ = t.take_preview_arc();
            }
        }
        t.on_canvas_pointer(cp([140.0, 96.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([140.0, 96.0], PointerPhase::Up));
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
}

/// **A ORDEM, pinada.** A cobertura que um traço de máscara deixa tem de ser, **byte a byte**, o ALFA que
/// o mesmo traço do brush digital depositaria — mesma geometria, mesmo pincel, mesmo acúmulo.
///
/// O oráculo é o produto de verdade: pinta-se o MESMO traço duas vezes, uma em modo Mask (lendo o
/// scratch) e uma em modo Paint com tinta preta sobre branco (lendo o canvas). Se a máscara ganhar
/// qualquer lei própria — um envelope, um teto, um cap por-modo — os dois campos divergem e isto fica
/// vermelho, que é exactamente o que se quer: foi assim que a lei do canal foi construída, e é assim que
/// ela (ou outra) não volta em silêncio.
///
/// **Mutação que deve sangrar** (medida): forçar o buffer por-traço na máscara + a lei `max` (o envelope
/// Wash) faz **3020 texels divergirem, pior delta 120 de 255**.
#[test]
fn the_mask_lays_exactly_what_the_digital_brush_lays() {
    let stroke = |t: &mut PainterTool| vstroke(t, 128.0, 60.0, 200.0, 25);

    // (a) modo Mask: a cobertura é `1 − luma` do scratch.
    let mut m = mask_tool(S);
    stroke(&mut m);
    let mask_cov = coverage(&m, S);

    // (b) modo Paint, tinta PRETA sobre branco: a cobertura equivalente é `1 − luma` do canvas. O pincel
    //     é o do slot de máscara, copiado para todos os slots, para que a única diferença seja o MODO.
    let mut p = PainterTool::default();
    p.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    let mut mask_brush = m.paint.brush;
    mask_brush.color = [0.0, 0.0, 0.0];
    p.paint.brush = mask_brush;
    for slot in &mut p.paint.brush_by_mode {
        *slot = mask_brush;
    }
    stroke(&mut p); // (o `vstroke` já drena o preview; o campo pintado mora no canvas)
    let canvas = p.canvas_rgba.clone();

    let mut diff = 0usize;
    let mut worst = 0i32;
    for i in 0..(S as usize * S as usize) {
        let paint_cov = 255 - i32::from(canvas[i * 4]);
        let mask_val = (mask_cov[i] * 255.0).round() as i32;
        let d = (paint_cov - mask_val).abs();
        if d > 0 {
            diff += 1;
            worst = worst.max(d);
        }
    }
    assert_eq!(
        diff, 0,
        "a máscara tem de depositar o MESMO campo que o brush digital: {diff} texels diferem, pior \
         delta {worst} de 255"
    );
}

// ⚠️ **NÃO existe aqui um gate numérico das CONTAS**, e a razão é medida: sob o envelope reprovado o
// pico-a-pico da modulação por-dab é **5 níveis de 255**, contra **3** sob a lei do brush — os dois na
// mesma ordem, porque o que o olho vê não é a amplitude, é a ONDULAÇÃO PERIÓDICA sobre um campo
// quase-sólido (uma ripple de 2% de contraste é visível; um bar de pico-a-pico não a separa do ruído de
// quantização). Um gate com bar 4 seria um gate que não pode falhar pelo motivo que alega. O oráculo das
// contas é o RENDER (a sonda `probe_mask_beading_along_the_axis` + a foto no doc 25 §13.10), e o gate que
// de fato as impede é o de byte-identidade acima: o brush digital não faz contas, então a máscara não faz.

/// O cap de Accumulate segue armando exactamente onde armava, e **o MODO não entra na conta** — é isso
/// que "pinta como o brush digital" significa na porta.
#[test]
fn the_coverage_cap_arms_where_it_always_did() {
    let mut t = mask_tool(S);
    let base = ph2d_painter_brush::BrushSpec::default();
    assert!(
        !t.stroke_cover_wanted(&base),
        "Strength cheia + Accumulate OFF: o cap é inobservável, ninguém threada buffer"
    );
    assert!(
        t.stroke_cover_wanted(&ph2d_painter_brush::BrushSpec {
            strength: 0.5,
            accumulate: false,
            ..base
        }),
        "Strength < 1 + Accumulate OFF: o cap é observável e o buffer é threadado"
    );
    assert!(
        !t.stroke_cover_wanted(&ph2d_painter_brush::BrushSpec {
            strength: 0.5,
            accumulate: true,
            ..base
        }),
        "Accumulate ON não rastreia cobertura em modo nenhum"
    );
    let capped = ph2d_painter_brush::BrushSpec {
        strength: 0.5,
        ..base
    };
    t.paint.paint_mode = PaintMode::Paint;
    let in_paint = t.stroke_cover_wanted(&capped);
    t.paint.paint_mode = PaintMode::Mask;
    let in_mask = t.stroke_cover_wanted(&capped);
    assert_eq!(
        in_paint, in_mask,
        "a porta da cobertura não pode olhar o MODO: a máscara acumula como o brush digital"
    );
}

/// A máscara é um passo de undo, e o traço seguinte deposita normal — o ciclo de vida que qualquer lei de
/// cobertura tem de respeitar (o buffer por-traço é transiente, não estado de documento).
#[test]
fn a_mask_stroke_is_one_undo_step_and_the_next_stroke_starts_fresh() {
    let mut t = mask_tool(S);
    let blank = coverage(&t, S)[130 * S as usize + 128];
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let painted = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        painted > 0.9 && blank < 0.01,
        "fixture: o traço tem de proteger o miolo ({blank:.3} -> {painted:.3})"
    );
    assert!(t.undo_last(), "um traço de máscara é um passo de undo");
    let undone = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        undone < 0.01,
        "o undo tem de devolver a cobertura pré-traço, got {undone:.3}"
    );
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let again = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        (again - painted).abs() < 0.01,
        "re-pintar depois do undo tem de depositar o mesmo ({painted:.3} depois {again:.3})"
    );
}

/// **O custo é da PEGADA, não do canvas.** Quadruplicar a tela não pode mover o custo por movimento —
/// razão primeiro (imune à deriva da máquina), depois um kill de wall-clock generoso. Medido ~1,0× e
/// 0,9 ms médio / 2,5 ms pior nos dois perfis, contra um frame de 16,7 ms.
#[test]
fn the_mask_stroke_cost_does_not_follow_the_canvas() {
    let cost = |size: u32| -> f64 {
        let mut t = mask_tool(size);
        let c = size as f32 * 0.5;
        t.set_brush_size_px(120.0);
        t.on_canvas_pointer(cp([c - 100.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let n = 20;
        let t0 = std::time::Instant::now();
        for i in 1..=n {
            t.on_canvas_pointer(cp([c - 100.0 + i as f32 * 10.0, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let dt = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        t.on_canvas_pointer(cp([c + 100.0, c], PointerPhase::Up));
        dt
    };
    let small = cost(1024);
    let big = cost(2048);
    assert!(
        big < small * 1.6 + 0.15,
        "um move de máscara é limitado pela pegada: {small:.2} ms @1024² vs {big:.2} ms @2048² \
         (um passe que percorresse o plano daria 4×)"
    );
    assert!(
        big < 8.0,
        "um move de máscara cabe num frame com folga: {big:.2} ms @2048² (kill 8 ms)"
    );
}

/// A borda que a lei do brush deixa, **MEDIDA** — não uma asserção de que ela é boa. Existe para que o
/// número do endurecimento fique num teste executável e ninguém precise re-medir para saber do que se
/// fala: o traço nasce com ~3,5 px de rampa e ela aperta com as passadas. **É o defeito que segue ABERTO**
/// (doc 25 §13.10), e a cura não é a lei da cobertura — as duas leis foram tentadas.
#[test]
fn the_documented_hardening_is_still_there_and_this_is_its_number() {
    let mut t = mask_tool(S);
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let one = band_px(&coverage(&t, S), S, 130);
    for _ in 0..14 {
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    }
    let fifteen = band_px(&coverage(&t, S), S, 130);
    assert!(
        (one - 3.53).abs() < 0.5 && (fifteen - 1.38).abs() < 0.5,
        "o endurecimento documentado mudou de número (era 3.53 px numa passada e 1.38 em quinze): \
         got {one:.2} e {fifteen:.2}. Se foi de propósito, atualize o doc 25 §13.10 com a medição nova"
    );
}
