//! **A aquarela sob o RESTO da ferramenta.** O wash através do editor de forma, do blend mode do
//! pincel, da seleção e da proteção, do alpha-lock, do ladrilho, da textura e da ponta do pincel — as
//! costuras em que o meio caro encontra uma feature que não sabe que ele existe.

use super::*;

#[test]
fn editing_the_paper_re_renders_the_wet_wash_with_the_new_paper() {
    // Sweep finding (2026-07-12), found INDEPENDENTLY by two lenses. The live-editable wash (2026-07-11)
    // re-renders the committed pool when a Grain/Paper param moves while the paper is still wet —
    // `rerender_editable_wash`'s own doc says it reconstructs "with the CURRENT brush texture". But the
    // paper-tooth memo (`wet_substrate`) is only NaN-reset at PEN-DOWN, and `fill_substrate_cache` fills
    // only the NaN misses — so every pixel of the pool keeps the paper height computed for the OLD paper.
    // The field's doc-comment asserts "the paper cannot change mid-stroke, so there is no in-stroke
    // invalidation to get wrong" — the live-edit feature made that premise false, and defeated ITSELF for
    // the Paper slot (the Grain works, which is why the smoke passed).
    // RED without the fix: the canvas is byte-identical after moving Paper Size.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        granulation: 0.9, // the substrate has to WEIGH on the bake, else nothing is observable
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::Voronoi; // a lattice paper: Size genuinely changes the tooth
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    t.paint.brush.paper_depth = 1.0;
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let memo_before: Vec<f32> = t.paint.wet_substrate.clone();
    let memoised = memo_before.iter().filter(|v| !v.is_nan()).count();
    assert!(
        memoised > 0,
        "the wash memoised the paper tooth under its footprint"
    );
    // The paper is still wet. The user drags Paper Size — the live-edit feature's whole reason to exist.
    t.set_brush_paper_size(0, 24.0);
    t.set_brush_paper_size(1, 24.0);
    t.paint_tick(0.016); // the heartbeat → rerender_editable_wash
    // THE ORACLE IS THE MEMO, not the pixels. A pixel-level `assert_ne!` goes GREEN for the wrong reason:
    // the re-render forces a bigger dirty region, so freshly-filled (NaN) pixels move bytes around even
    // while every ALREADY-memoised pixel keeps the old paper. Compare only the pixels that were already
    // memoised — those are the ones the staleness hides in.
    let stale = t
        .paint
        .wet_substrate
        .iter()
        .zip(memo_before.iter())
        .filter(|(now, was)| !was.is_nan() && now.to_bits() == was.to_bits())
        .count();
    assert_eq!(
        stale, 0,
        "every memoised paper-tooth sample must be rebuilt for the new Paper          ({stale}/{memoised} still hold the OLD paper's tooth)"
    );
}

#[test]
fn paper_depth_and_granulation_re_render_the_wet_wash() {
    // Sweep (2026-07-12): the live-editable wash's change detector was `(Grain, Paper)` `TextureSettings`
    // only. But `apply_watercolor` also reads **Paper Depth** and **Granulation**, which live on
    // `BrushSpec`, NOT inside `TextureSettings`. So dragging Paper *Size* re-rendered the wet pool and
    // dragging Paper *Depth* — the slider right next to it — did nothing: the same gesture, two different
    // behaviours, side by side. (Swapping the Paper/Grain IMAGE while keeping `kind: Image` was invisible
    // too: no setting changes, only the pixel version.)
    // RED without the fix: the canvas is byte-identical after moving Paper Depth.
    use ph2d_painter_brush::{TextureKind, TextureMapping};
    let mut t = white_canvas(64, 10.0);
    t.paint.brush = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.85, 0.1, 0.1],
        space_attenuation: false,
        watercolor: true,
        fill: 0.5,
        depth: 1.5,
        granulation: 0.9,
        paper_depth: 1.0,
        ..Default::default()
    };
    t.paint.brush.paper.kind = TextureKind::Voronoi;
    t.paint.brush.paper.mapping = TextureMapping::Tiled;
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let before = (*t.canvas_rgba).clone();
    // The paper is still wet. The user drags Paper Depth — nothing else moves.
    t.set_brush_paper_depth(0.0);
    t.paint_tick(0.016); // the heartbeat → rerender_editable_wash
    assert_ne!(
        before,
        (*t.canvas_rgba).clone(),
        "Paper Depth is read by the composite — it must re-render the wet pool like Paper Size does"
    );
}

#[test]
fn grain_rake_is_inert_under_the_wash() {
    // Enio asked to close the sweep's open findings. This one is the Grain twin of the Paper Rake removal.
    // With Watercolor on, the Grain slot IS the granulation map — a CANVAS-ANCHORED substrate saying where
    // pigment settles — not a stamp the dab carries. The composite samples it through
    // `angle_basis(texture.angle_deg)`: no `d.dir`. So "Rake" (follow the stroke) has nothing to rotate,
    // and the same checkbox meant two different things depending on the Watercolor tick — one being "nothing".
    // This pins the deadness that justifies hiding it (a hidden knob must be PROVABLY inert), and it fails
    // the day someone wires per-dab Grain frames into the wash — at which point the panel must show it again.
    // (The per-slot "Random Angle" was retired 2026-07-19 — the Stroke Jitter Rotate covers a random spin.)
    use ph2d_painter_brush::{TextureKind, TextureSettings};
    let wash = |rake: bool| -> Vec<u8> {
        let mut t = white_canvas(64, 9.0);
        t.paint.brush = BrushSpec {
            radius_px: 9.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            depth: 1.5,
            granulation: 0.9, // the Grain has to WEIGH on the bake, else the test proves nothing
            texture: TextureSettings {
                kind: TextureKind::Noise,
                size: [6.0, 6.0],
                rake,
                ..Default::default()
            },
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down));
        for i in 1..10u16 {
            t.on_canvas_pointer(cp([16.0 + 4.0 * f32::from(i), 32.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([52.0, 32.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    let plain = wash(false);
    assert!(
        plain
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] != 255 || p[1] != 255),
        "the wash actually painted (guard against a fixture that proves nothing)"
    );
    assert_eq!(plain, wash(true), "Grain Rake is inert under the wash");
}

/// **Sob a lavagem, NEM o Accumulate NEM a Strength alcançam a tinta** — e as duas metades chegaram
/// aqui por caminhos diferentes, que é a razão de o gate dizer as duas.
///
/// * **Accumulate — INERTE por construção** (Enio, 2026-07-12: *"no modo aquarela, faz sentido ter
///   Strength e Accumulate no painel?"*). Ele é lido só pelo `accumulate_cap`, dentro do roteamento
///   do stamp, e a lavagem desvia ANTES disso (`stamp_dabs`). É redundante de qualquer forma: a
///   cobertura da lavagem é **max-blended** (um envelope), que já É *"sem build-up dentro de um
///   traço"*.
/// * **Strength — inerte por DECISÃO** (Enio, 2026-08-12: *"Strength não é adequado para watercolor.
///   Tire essa ligação e esconda o slider"*). ⚠️ **Este gate afirmava o CONTRÁRIO até hoje** — o nome
///   dele era `under_the_wash_accumulate_is_inert_but_strength_is_not` e a asserção era um
///   `assert_ne!` com o texto *"the slider must STAY"*. Ela estava **certa sobre o mundo** (a
///   Strength era o pico do depósito, e a medição de 12/08 confirma: 1029 bytes diferiam, pior delta
///   202) e respondia à pergunta *"ela faz alguma coisa?"*. A pergunta desta vez é outra — *"ela DEVE
///   fazer?"* — e é de produto. A cerca caiu **junto com o corte**
///   ([`super::watercolor_accum::WASH_DEPOSIT_PEAK`]), nunca deixada verde por acidente.
///
/// ⚠️ **Eram TRÊS consumidores e o gate cobre os três**, senão a metade esquecida seria um knob
/// invisível ainda governando a lavagem: o splat da COBERTURA · o splat da COR · e o `amount` do
/// SMUDGE, que não deposita nada — ele diz *quanto a água arrasta*.
///
/// ⚠️ E o **CONTROLE** é o que torna os `assert_eq!` legíveis: um traço que não pintasse nada
/// satisfaria os três por vácuo.
///
/// Isto também resolve que TODO método de traço lava: os shape editors correm a óptica pelo
/// `stamp_drag_preview_watercolor` (doc 13 #3), então não há método onde o Accumulate volte.
#[test]
fn under_the_wash_neither_accumulate_nor_strength_reaches_the_paint() {
    let wash = |strength: f32, accumulate: bool| -> Vec<u8> {
        let mut t = white_canvas(64, 10.0);
        t.paint.brush = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.85, 0.1, 0.1],
            space_attenuation: false,
            watercolor: true,
            fill: 0.5,
            depth: 1.5,
            strength,
            accumulate,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([36.0, 36.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([36.0, 36.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    // CONTROLE: o traço tem de PINTAR, senão os três `assert_eq!` abaixo passam por vácuo.
    let painted = wash(0.9, true)
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] != 255 || p[1] != 255 || p[2] != 255)
        .count();
    assert!(
        painted > 100,
        "a fixture tem de conter uma lavagem de verdade (so {painted} px sairam do branco)"
    );
    assert_eq!(
        wash(0.6, true),
        wash(0.6, false),
        "Accumulate is INERT under the wash — hiding the checkbox removes nothing"
    );
    assert_eq!(
        wash(0.35, true),
        wash(0.9, true),
        "a Strength nao pode alcancar a lavagem (WASH_DEPOSIT_PEAK) — 2026-08-12, ordem do Enio"
    );

    // E a 3a metade, a que se esquece: o SMUDGE. Ele nao deposita — diz quanto a agua ARRASTA.
    //
    // ⚠️ A fixture e a do `watercolor_smudge_true_smears_the_painted_paint`, VERBATIM na forma, e as
    // tres escolhas dela sao load-bearing — duas versoes minhas passaram por VACUO antes disto:
    //   * a tinta a arrastar vem do SOURCE (papel branco nao tem o que arrastar);
    //   * o falloff e o MACIO do motor (com `Constant` em forca cheia a esfregada degenera numa
    //     translacao rigida — o disco sobrescreve tudo o que cruza);
    //   * a lavagem e LEVE (`fill 0.3`), senao o wash cobre a base esfregada e nada se ve.
    // A 2a versao ainda errava numa 4a: os dois tracos tinham a MESMA cor, e arrastar vermelho para
    // dentro de vermelho e invisivel.
    let smear = |strength: f32, smudge: f32| -> Vec<u8> {
        let size = 128u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let p = if (40..70).contains(&x) {
                    [217u8, 13, 13, 255] // banda vermelha no meio
                } else {
                    [255u8, 255, 255, 255]
                };
                src[i..i + 4].copy_from_slice(&p);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 6.0,
            color: [0.1, 0.2, 0.85],
            space_attenuation: false,
            watercolor: true,
            edge_gain: 0.0,
            granulation: 0.0,
            warp: 0.0,
            fill: 0.3,
            depth: 1.0,
            wet_smudge: smudge,
            wet_rewet: 0.0,
            strength,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        t.on_canvas_pointer(cp([16.0, 64.0], PointerPhase::Down));
        let mut x = 16.0f32;
        while x < 96.0 {
            x += 3.0;
            t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up));
        (*t.canvas_rgba).clone()
    };
    // CONTROLE do smudge: a esfregada tem de ser VISIVEL nesta fixture, senao o `assert_eq!` abaixo
    // passa por vacuo — foi assim que as duas primeiras versoes deste gate deixaram a mutacao viver.
    assert_ne!(
        smear(1.0, 0.0),
        smear(1.0, 0.9),
        "a fixture tem de conter a esfregada (com e sem Smudge tem de DIFERIR)"
    );
    assert_eq!(
        smear(0.35, 0.9),
        smear(0.9, 0.9),
        "a Strength ainda governava quanto o SMUDGE arrasta — a metade que nao deposita e a que se esquece"
    );
}

/// **Shape-editor bake runs the watercolor wash (doc 13 #3).** A shape editor (here a Line) committed
/// with a Watercolor brush must bake the OPTICAL wash (frozen base + rim / Ragged-Edge warp) — not the
/// plain source-over deposit. RED before #3: the shape editors stamp WITHOUT the stroke lifecycle, so no
/// base is frozen (`watercolor_base` stays `None`), the `stamp_dabs` watercolor gate is false, and the
/// bake is BYTE-IDENTICAL to the same shape drawn with Watercolor OFF. GREEN: the optics diverge it.
#[test]
fn watercolor_shape_editor_bake_runs_the_wash() {
    fn line_brush(t: &mut PainterTool, watercolor: bool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            warp: 3.0, // Ragged Edge on — a signature only the wash produces
            stroke_method: StrokeMethod::Line,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    fn draw_and_commit(t: &mut PainterTool) {
        t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down)); // corner 1
        t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
        t.on_canvas_pointer(cp([8.0, 44.0], PointerPhase::Down)); // corner 2 (press)
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move)); // drag to the final spot
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
        assert!(t.commit_open_shape(), "Apply baked the open line");
    }
    let size = 64u32;
    let differs = |a: &PainterTool, b: &PainterTool| {
        (0..size * size).any(|i| px(a, size, i % size, i / size) != px(b, size, i % size, i / size))
    };
    let blank = white_canvas(size, 8.0);

    let mut plain = white_canvas(size, 8.0);
    line_brush(&mut plain, false);
    draw_and_commit(&mut plain);

    let mut wet = white_canvas(size, 8.0);
    line_brush(&mut wet, true);
    draw_and_commit(&mut wet);

    assert!(
        differs(&plain, &blank),
        "the line committed paint (control)"
    );
    assert!(
        differs(&wet, &plain),
        "watercolor optics ran on the shape bake (was byte-identical to plain before #3)"
    );
}

/// **A watercolor shape preview leaves no trail (doc 13 #3).** The moving/resizing shape preview
/// re-composites the wash each frame over the restored-pristine canvas — including the rim/warp that
/// reach BEYOND the dab bbox. If the save/restore footprint didn't cover that reach, a wrong drag would
/// leave a rim trail. Proof: a line dragged straight to its final corner and the SAME line dragged
/// through two wrong spots first must bake BYTE-IDENTICALLY (the wobble peels clean).
#[test]
fn watercolor_shape_preview_leaves_no_trail() {
    fn line_brush(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            warp: 3.0,
            stroke_method: StrokeMethod::Line,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    let size = 64u32;

    // Straight to the final corner.
    let mut direct = white_canvas(size, 8.0);
    line_brush(&mut direct);
    direct.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    direct.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    direct.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Down));
    direct.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    assert!(direct.commit_open_shape(), "direct line baked");

    // Same final corner, but dragged through two WRONG spots first.
    let mut wobbled = white_canvas(size, 8.0);
    line_brush(&mut wobbled);
    wobbled.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
    wobbled.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Up));
    wobbled.on_canvas_pointer(cp([30.0, 8.0], PointerPhase::Down)); // wrong 1
    wobbled.on_canvas_pointer(cp([44.0, 56.0], PointerPhase::Move)); // wrong 2
    wobbled.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move)); // final
    wobbled.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
    assert!(wobbled.commit_open_shape(), "wobbled line baked");

    for i in 0..size * size {
        let (x, y) = (i % size, i / size);
        assert_eq!(
            px(&wobbled, size, x, y),
            px(&direct, size, x, y),
            "wobble left a wash trail at ({x},{y}) — the preview footprint missed the rim reach"
        );
    }
}

/// **The watercolor wash ignores the Brush Blend mode (doc 13 #4).** The optical deposit is source-over +
/// Beer–Lambert optics — `BrushBlend` is never read on the wash path, so the Brush Blend dropdown is
/// INERT in watercolor mode (why the panel hides it there). Two washes identical but for `brush.blend`
/// bake byte-for-byte. Refutable: wiring blend into the wash turns this RED (and would un-justify the hide).
#[test]
fn watercolor_wash_ignores_the_brush_blend_mode() {
    fn wash(t: &mut PainterTool, blend: ph2d_painter_brush::BrushBlend) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            warp: 3.0,
            blend, // the wash must ignore this entirely
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
        assert!(t.on_canvas_pointer(cp([16.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([48.0, 32.0], PointerPhase::Up));
    }
    let size = 64u32;
    let mut mix = white_canvas(size, 8.0);
    wash(&mut mix, ph2d_painter_brush::BrushBlend::Mix);
    let mut mult = white_canvas(size, 8.0);
    wash(&mut mult, ph2d_painter_brush::BrushBlend::Multiply);
    for i in 0..size * size {
        let (x, y) = (i % size, i / size);
        assert_eq!(
            px(&mix, size, x, y),
            px(&mult, size, x, y),
            "brush Blend changed the wash at ({x},{y}) — it must be inert in watercolor"
        );
    }
}

/// **Watercolor respects the Selection + protection-mask gates** (the audit hole, Enio 2026-07-07):
/// the optical path used to short-circuit BEFORE the canvas gates in `stamp_dabs`, so a watercolor
/// stroke painted straight through an active selection and the Sculpt-style protection scratch.
/// Now the wash never FORMS on gated-out texels (splat gates) AND the composite keep-lerps the final
/// bytes toward the frozen base (restore semantics — warp-proof: this stroke runs Ragged Edge > 0,
/// whose displaced sampling used to be the leak vector).
#[test]
fn watercolor_respects_selection_and_protection_masks() {
    fn wet_brush(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            warp: 4.0, // Ragged Edge ON: proves the composite gate stops the warped-sampling leak
            wet_rewet: 1.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    let size = 64u32;

    // ── Selection: left half selected; a stroke straddling x=32 must clip at the border. ──
    let mut t = white_canvas(size, 8.0);
    wet_brush(&mut t);
    t.set_rect_selection(0, 0, 32, 64);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    assert_ne!(
        px(&t, size, 28, 32),
        [255, 255, 255, 255],
        "inside the selection the wash painted"
    );
    for x in [38u32, 44, 50] {
        assert_eq!(
            px(&t, size, x, 32),
            [255, 255, 255, 255],
            "outside the selection stays pristine (x={x}) — the watercolor gate hole"
        );
    }

    // ── Protection scratch: right half painted black (= frozen); the wash must not land there. ──
    let mut t = white_canvas(size, 8.0);
    wet_brush(&mut t);
    t.ensure_mask_scratch();
    assert!(t.mask_scratch_active(), "scratch installed on the layer");
    {
        let scratch = Arc::make_mut(&mut t.paint.mask_scratch_rgba);
        for y in 0..size {
            for x in 32..size {
                let i = ((y * size + x) * 4) as usize;
                scratch[i] = 0; // black = protect (mask_value 0 = frozen)
                scratch[i + 1] = 0;
                scratch[i + 2] = 0;
                scratch[i + 3] = 255;
            }
        }
    }
    assert!(t.mask_protection_active(), "protection gate armed");
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    assert_ne!(
        px(&t, size, 28, 32),
        [255, 255, 255, 255],
        "unprotected side painted"
    );
    for x in [38u32, 44, 50] {
        assert_eq!(
            px(&t, size, x, 32),
            [255, 255, 255, 255],
            "protected texels stay frozen (x={x})"
        );
    }
}

/// **Alpha-lock (doc 13 #8) — the watercolor wash paints only into EXISTING alpha.**
/// Canvas = left half opaque white (α=255), right half transparent (α=0); alpha-lock ON; a wet stroke
/// (Ragged Edge on, so the warped sampling can REACH the transparent side) straddles the α boundary.
/// The opaque side takes the wash with its alpha preserved; the transparent side stays fully
/// transparent — the layer's silhouette is frozen, exactly like the non-wc dab (`acc[3] = pre_alpha`).
/// RED before the fix: the composite deposits `cov_a` alpha wherever coverage reaches, transparent or
/// not (`out_a = ab + (1−ab)·cov_a` with `ab = 0` ⇒ `out_a = cov_a > 0`).
#[test]
fn watercolor_alpha_lock_paints_only_into_existing_alpha() {
    let size = 64u32;
    let mut src = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size / 2 {
            let i = ((y * size + x) * 4) as usize;
            src[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, size, size);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        warp: 4.0, // Ragged Edge ON: the warped sampling reaches the transparent side (composite gate)
        wet_rewet: 1.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    let active = t.layers.active().expect("active layer");
    t.layers.get_mut(active).expect("layer").alpha_locked = true;

    // Stroke centred on the α boundary (x=32), radius 8 ⇒ the disc covers x∈[24,40].
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));

    // Opaque side: the wash landed (colour moved) and the alpha is still fully opaque.
    let inside = px(&t, size, 26, 32);
    assert_ne!(
        [inside[0], inside[1], inside[2]],
        [255, 255, 255],
        "opaque side took the watercolor wash"
    );
    assert_eq!(inside[3], 255, "opaque side alpha preserved");

    // Transparent side (inside the disc at x=36/38, and warp-reach at x=44): alpha-lock froze it.
    for x in [36u32, 38, 44] {
        assert_eq!(
            px(&t, size, x, 32)[3],
            0,
            "alpha-lock kept the transparent side transparent (x={x})"
        );
    }
}

/// **Tiling (doc 13 #2) — the watercolor wash wraps seamlessly across the sprite seam.** A wet dab
/// hard against the right edge (radius crosses x=64) with X-tiling on must ALSO deposit the wrapped
/// part on the left edge, so the painted texture tiles. RED before the fix: the watercolor route
/// short-circuits `stamp_dabs` BEFORE `tiled_dabs`, so only the original (un-wrapped) dab forms.
#[test]
fn watercolor_tiling_wraps_the_wash_across_the_seam() {
    let size = 64u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.paint.tiling = [true, false]; // seamless wrap on X

    // Dab at x=62 (r=8 ⇒ footprint [54,70] crosses the far edge at x=64).
    assert!(t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Up));

    assert_ne!(
        px(&t, size, 61, 32),
        [255, 255, 255, 255],
        "the wash landed on the right edge"
    );
    // The wrapped copy (shifted −64 ⇒ centre −2, footprint [−10,6]) paints x∈[0,6] on the left edge —
    // unreachable from x=62 without the wrap (distance 60 ≫ radius 8), so any paint here IS the tile.
    assert_ne!(
        px(&t, size, 2, 32),
        [255, 255, 255, 255],
        "tiling wrapped the wash onto the left edge (seamless seam)"
    );
}

/// **A dynamic SHAPE's wash crosses the Tiling seam (Enio 2026-07-11).** The shape editors re-stamp
/// through `stamp_drag_preview_watercolor` (a re-stamp preview, NOT the stroke lifecycle), which took the
/// RAW dabs — so with seamless Tiling a shape crossing the border was cut there instead of wrapping. RED
/// before the fix: the left (wrapped) edge stays pristine. GREEN: the tiled dabs form the wash on the
/// opposite edge too, matching the plain stroke (`stamp_dabs`). Control: tiling OFF leaves it pristine.
#[test]
fn watercolor_shape_wash_crosses_the_tiling_seam() {
    fn line_brush(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.15, 0.25, 0.75],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.5,
            edge_spread: 4.0,
            stroke_method: StrokeMethod::Line,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    // A vertical Line at x=62 (r=8 ⇒ footprint [54,70] crosses the far edge at x=64): two clicks place the
    // start + end anchors, then Apply bakes.
    fn draw_and_commit(t: &mut PainterTool) {
        t.on_canvas_pointer(cp([62.0, 20.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([62.0, 20.0], PointerPhase::Up));
        t.on_canvas_pointer(cp([62.0, 44.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([62.0, 44.0], PointerPhase::Up));
        assert!(t.commit_open_shape(), "Apply baked the open line");
    }
    let size = 64u32;

    // Tiling ON (X): the wrapped copy (centre −2, footprint [−10,6]) paints x∈[0,6] on the LEFT edge —
    // unreachable from x=62 without the wrap (distance 60 ≫ radius 8), so any paint here IS the tile.
    let mut tiled = white_canvas(size, 8.0);
    line_brush(&mut tiled);
    tiled.paint.tiling = [true, false];
    draw_and_commit(&mut tiled);
    assert_ne!(
        px(&tiled, size, 61, 32),
        [255, 255, 255, 255],
        "the shape wash landed on the right edge"
    );
    assert_ne!(
        px(&tiled, size, 2, 32),
        [255, 255, 255, 255],
        "the shape wash wrapped across the seam onto the left edge (was cut before the fix)"
    );

    // Control: tiling OFF ⇒ the left edge stays pristine (proves the wrap is what paints it).
    let mut plain = white_canvas(size, 8.0);
    line_brush(&mut plain);
    draw_and_commit(&mut plain);
    assert_eq!(
        px(&plain, size, 2, 32),
        [255, 255, 255, 255],
        "without tiling the shape wash never reaches the far edge (control)"
    );
}

/// **A texture-param change re-renders the still-wet wash — central AND every Tiling copy (Enio
/// 2026-07-11).** After pen-up the wash bakes, but the last wash stays re-renderable while the paper is
/// wet: changing the Grain Size re-renders the whole committed wash (not just the next stroke). The setter
/// alone is inert (only stores the value); the paint tick applies it. With Tiling on, the WRAPPED copy
/// re-renders too. RED before the feature: the baked wash never reacts to a Size change.
#[test]
fn watercolor_texture_size_rerenders_the_wet_wash_and_all_tiles() {
    let size = 64u32;
    let mut t = white_canvas(size, 8.0);
    t.paint.brush = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        granulation: 1.0, // granulation ON so the Grain texture modulates the wash visibly
        texture: ph2d_painter_brush::TextureSettings {
            kind: ph2d_painter_brush::TextureKind::Noise,
            size: [1.0, 1.0],
            ..Default::default()
        },
        ..Default::default()
    };
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.paint.tiling = [true, true];

    // Paint a wash at x=62 (footprint [54,70] crosses the far edge ⇒ a wrapped copy paints x∈[0,6]); lift.
    assert!(t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([62.0, 32.0], PointerPhase::Up));

    let central_before = px(&t, size, 60, 32);
    let wrapped_before = px(&t, size, 2, 32);
    assert_ne!(
        central_before,
        [255, 255, 255, 255],
        "the wash baked at the right edge"
    );
    assert_ne!(
        wrapped_before,
        [255, 255, 255, 255],
        "tiling wrapped the wash to the left edge"
    );

    // The setter alone only STORES the value — the baked canvas is untouched until the tick applies it.
    t.set_brush_texture_size(0, 6.0);
    assert_eq!(
        px(&t, size, 60, 32),
        central_before,
        "the Size setter must not touch the canvas by itself"
    );

    // The paint tick re-renders the still-wet wash with the new Grain Size.
    t.paint_tick(0.016);
    let central_after = px(&t, size, 60, 32);
    let wrapped_after = px(&t, size, 2, 32);
    assert_ne!(
        central_before, central_after,
        "the wet wash re-rendered centrally with the new Grain Size"
    );
    assert_ne!(
        wrapped_before, wrapped_after,
        "the WRAPPED Tiling copy re-rendered too (all tiles update together)"
    );

    // Once the session dries the wash is permanent — a further Size change no longer re-renders it.
    t.dry_session_now();
    let dry_before = px(&t, size, 60, 32);
    t.set_brush_texture_size(0, 12.0);
    t.paint_tick(0.016);
    assert_eq!(
        px(&t, size, 60, 32),
        dry_before,
        "a dried wash is permanent — texture edits no longer re-render it"
    );
}

/// **Alpha-lock is a no-op where the layer is fully opaque (byte-identical, §0.6).** On an opaque
/// canvas every texel has `ka = 1` ⇒ the splat gate is `1.0` and the composite's α-pin re-writes the
/// already-opaque α — so a locked stroke must be byte-for-byte the same as the unlocked one.
#[test]
fn watercolor_alpha_lock_is_a_noop_on_fully_opaque() {
    fn wet(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            warp: 4.0,
            wet_rewet: 1.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    fn stroke(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    }
    let size = 64u32;

    let mut unlocked = white_canvas(size, 8.0);
    wet(&mut unlocked);
    stroke(&mut unlocked);

    let mut locked = white_canvas(size, 8.0);
    wet(&mut locked);
    let active = locked.layers.active().expect("active layer");
    locked.layers.get_mut(active).expect("layer").alpha_locked = true;
    stroke(&mut locked);

    for y in 0..size {
        for x in 0..size {
            assert_eq!(
                px(&locked, size, x, y),
                px(&unlocked, size, x, y),
                "alpha-lock changed a fully-opaque pixel at ({x},{y})"
            );
        }
    }
}

/// **Shape "Automatic" (doc 13 #1) — the continuity + capability contract.**
/// (a) CONTINUITY: unchecking Automatic (which auto-selects the `Falloff::Watercolor` preset — the
/// built-in feather as a curve) paints a stroke BYTE-IDENTICAL to Automatic: the manual path with the
/// default knobs is the same stamp, so the checkbox transition never pops. (b) CAPABILITY: with a
/// half-blank Shape image the manual stamp is ASYMMETRIC (the image drives the watercolor silhouette),
/// which Automatic's round feather can never produce.
#[test]
fn watercolor_shape_automatic_continuity_and_image_silhouette() {
    fn stroke(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([24.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 32.0], PointerPhase::Up));
    }
    fn wet(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 8.0,
            hardness: 0.0,
            falloff: Falloff::Smooth,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    let size = 64u32;

    // (a) Automatic ON (default) …
    let mut auto_t = white_canvas(size, 8.0);
    wet(&mut auto_t);
    stroke(&mut auto_t);
    // … vs the panel toggle OFF (routes through the real seam: also auto-selects Falloff::Watercolor).
    let mut manual_t = white_canvas(size, 8.0);
    wet(&mut manual_t);
    manual_t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
        ph2d_editor_core::ids::PAINTER_SHAPE_WATERCOLOR_AUTO,
    ));
    let b = manual_t.brush_settings();
    assert!(!b.watercolor_shape_auto, "toggle turned Automatic off");
    assert_eq!(
        manual_t.paint.brush.falloff,
        Falloff::Watercolor,
        "unchecking auto-selects the Watercolor falloff preset (continuity)"
    );
    manual_t.paint.brush_by_mode.fill(manual_t.paint.brush);
    stroke(&mut manual_t);
    assert_eq!(
        auto_t.canvas_rgba.as_slice(),
        manual_t.canvas_rgba.as_slice(),
        "Automatic OFF + Watercolor falloff must paint BYTE-IDENTICAL to Automatic ON"
    );

    // (b) Manual + a Shape image whose RIGHT half is blank → the wash silhouette goes asymmetric.
    let mut img_t = white_canvas(size, 8.0);
    wet(&mut img_t);
    img_t.paint.brush.watercolor_shape_auto = false;
    img_t.paint.brush.falloff = Falloff::Watercolor;
    let mut lum = vec![255u8; 16 * 16];
    for y in 0..16 {
        for x in 8..16 {
            lum[y * 16 + x] = 0; // right half of the tip: no coverage
        }
    }
    img_t.set_brush_shape_image(lum, 16, 16);
    img_t.paint.brush_by_mode.fill(img_t.paint.brush);
    assert!(t_dab_paints_asymmetric(&mut img_t, size));
}

/// Helper for the image-silhouette assertion: one dab at the canvas centre; returns whether the
/// painted result differs left-vs-right of the centre column (the half-blank tip must show).
fn t_dab_paints_asymmetric(t: &mut PainterTool, size: u32) -> bool {
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    let mut left_painted = 0u32;
    let mut right_painted = 0u32;
    for y in 24..40u32 {
        for dx in 1..8u32 {
            if px(t, size, 32 - dx, y) != [255, 255, 255, 255] {
                left_painted += 1;
            }
            if px(t, size, 32 + dx, y) != [255, 255, 255, 255] {
                right_painted += 1;
            }
        }
    }
    assert!(
        left_painted > 0,
        "the covered half of the tip painted (left {left_painted})"
    );
    assert!(
        left_painted > right_painted * 2,
        "the blank half must paint far less (left {left_painted} vs right {right_painted})"
    );
    true
}

/// **Manual Shape stamp honours Flatten + Rotate + grey-tip normalisation** (Enio 2026-07-07,
/// smoke round 2): (a) `dab_flatten` squeezes the watercolor footprint into an ellipse and
/// `dab_angle_deg` orients it — they flowed through `footprint_deform` on the plain dab but the
/// watercolor envelope used the raw round distance; (b) a GREY tip image must paint the same wash
/// as a WHITE one (the per-stroke max-luminance normaliser: coverage is wetness geometry that must
/// saturate — a raw grey tip starved the optics: pale centre, dead rim).
#[test]
fn watercolor_manual_shape_flatten_rotate_and_grey_tip_normalise() {
    fn wet_manual(t: &mut PainterTool) {
        t.paint.brush = BrushSpec {
            radius_px: 12.0,
            hardness: 0.0,
            falloff: Falloff::Watercolor,
            color: [0.1, 0.2, 0.7],
            space_attenuation: false,
            watercolor: true,
            watercolor_shape_auto: false,
            fill: 0.6,
            depth: 2.0,
            edge_gain: 2.0,
            edge_spread: 4.0,
            warp: 0.0, // no organic boundary noise — the extents measure the FOOTPRINT
            granulation: 0.0, // no mottle either
            ..Default::default()
        };
        t.paint.brush_by_mode.fill(t.paint.brush);
    }
    fn dab(t: &mut PainterTool) {
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));
    }
    /// Painted extent along x and y through the centre row/column.
    fn extent(t: &PainterTool, size: u32) -> (u32, u32) {
        let (mut ex, mut ey) = (0u32, 0u32);
        for d in 0..16u32 {
            if px(t, size, 32 + d, 32) != [255, 255, 255, 255] {
                ex = d;
            }
            if px(t, size, 32, 32 + d) != [255, 255, 255, 255] {
                ey = d;
            }
        }
        (ex, ey)
    }
    let size = 64u32;

    // (a) Flatten 0.8: the footprint squeezes the minor (y) axis at angle 0…
    let mut t = white_canvas(size, 12.0);
    wet_manual(&mut t);
    t.paint.brush.dab_flatten = 0.8;
    t.paint.brush_by_mode.fill(t.paint.brush);
    dab(&mut t);
    let (ex, ey) = extent(&t, size);
    assert!(
        ex >= ey + 3,
        "Flatten must squeeze the watercolor footprint (x extent {ex} vs y {ey})"
    );
    // … and Rotate 90° swaps the axes.
    let mut t = white_canvas(size, 12.0);
    wet_manual(&mut t);
    t.paint.brush.dab_flatten = 0.8;
    t.paint.brush.dab_angle_deg = 90;
    t.paint.brush_by_mode.fill(t.paint.brush);
    dab(&mut t);
    let (ex, ey) = extent(&t, size);
    assert!(
        ey >= ex + 3,
        "Rotate 90° must re-orient the flattened footprint (x {ex} vs y {ey})"
    );

    // (b) Grey tip == white tip byte-for-byte (the normaliser rescales 128/255 → 1.0).
    let mut white_tip = white_canvas(size, 12.0);
    wet_manual(&mut white_tip);
    white_tip.set_brush_shape_image(vec![255u8; 16 * 16], 16, 16);
    white_tip.paint.brush_by_mode.fill(white_tip.paint.brush);
    dab(&mut white_tip);
    let mut grey_tip = white_canvas(size, 12.0);
    wet_manual(&mut grey_tip);
    grey_tip.set_brush_shape_image(vec![128u8; 16 * 16], 16, 16);
    grey_tip.paint.brush_by_mode.fill(grey_tip.paint.brush);
    dab(&mut grey_tip);
    assert_eq!(
        white_tip.canvas_rgba.as_slice(),
        grey_tip.canvas_rgba.as_slice(),
        "a uniformly grey tip must paint the SAME wash as a white one (normalised wetness)"
    );
}

/// **A TEXTURED tip keeps the typical watercolor** (Enio 2026-07-07: "não tem como o algoritmo que
/// faz a aquarela típica funcionar com textura no slot shape?"): the tip's texture must NOT hole the
/// wash — water fills the tip's outer silhouette (saturated coverage → body + rim at the OUTER
/// boundary) while the texture becomes pigment DENSITY within (`stroke_density` × the fill term).
/// A streaky tip therefore paints a fully-wet wash whose interior VARIES with the streaks instead of
/// showing white gaps.
#[test]
fn watercolor_textured_tip_keeps_typical_wash_with_density_variation() {
    let size = 64u32;
    let mut t = white_canvas(size, 12.0);
    t.paint.brush = BrushSpec {
        radius_px: 12.0,
        hardness: 0.0,
        falloff: Falloff::Watercolor,
        color: [0.1, 0.2, 0.7],
        space_attenuation: false,
        watercolor: true,
        watercolor_shape_auto: false,
        fill: 0.6,
        depth: 2.0,
        edge_gain: 2.0,
        edge_spread: 4.0,
        warp: 0.0,        // measure the footprint, not the organic noise
        granulation: 0.0, // no mottle — the only interior variation is the tip density
        ..Default::default()
    };
    // Streaky tip: 4-px columns alternating white (255) / mid (100) — bristle-like texture.
    let mut lum = vec![255u8; 32 * 32];
    for y in 0..32 {
        for x in 0..32 {
            if (x / 4) % 2 == 1 {
                lum[y * 32 + x] = 100;
            }
        }
    }
    t.set_brush_shape_image(lum, 32, 32);
    t.paint.brush_by_mode.fill(t.paint.brush);
    assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
    t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Up));

    // (a) TYPICAL WASH: every pixel in the core (radius/2) is painted — no white holes from the
    //     mid-tone streaks (they are fully WET; only their pigment density differs).
    let mut holes = 0u32;
    for y in 27..38u32 {
        for x in 27..38u32 {
            if px(&t, size, x, y) == [255, 255, 255, 255] {
                holes += 1;
            }
        }
    }
    assert_eq!(
        holes, 0,
        "mid-tone streaks must stay WET (no white holes in the core)"
    );

    // (b) TEXTURE AS DENSITY: the streak pattern shows as intensity variation — the painted core's
    //     green channel is not uniform (min/max spread beyond rounding noise).
    let (mut lo, mut hi) = (255u8, 0u8);
    for y in 30..35u32 {
        for x in 27..38u32 {
            let g = px(&t, size, x, y)[1];
            lo = lo.min(g);
            hi = hi.max(g);
        }
    }
    assert!(
        hi - lo >= 8,
        "the tip texture must read as pigment-density variation (green spread {lo}..{hi})"
    );
}
