//! Gates for **W4 — the advective family**: the warp carries the impasto planes with the pixels.
//!
//! The paint the fixtures warp is laid by the REAL deposit (relief + coverage + material + colour from one
//! impasto stroke), and the warp is driven through the REAL pointer route — the fixture cannot drift from
//! the product. Every gate names the mutation that must make it bleed.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};
use std::sync::Arc;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A tool holding one REAL impasto stroke (relief + coverage + material + colour), parked in Deform
/// Reshape with a firm Push brush — the state every gate here starts from.
fn deformable_relief_tool() -> (PainterTool, crate::tool::RtLayerId) {
    let size = 160u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 14.0,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("a layer");
    t.on_canvas_pointer(cp([50.0, 80.0], PointerPhase::Down));
    let mut x = 54.0;
    while x <= 110.0 {
        t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Move));
        x += 4.0;
    }
    t.on_canvas_pointer(cp([x, 80.0], PointerPhase::Up));
    assert!(
        t.heights
            .get(&layer)
            .is_some_and(|h| h.iter().any(|v| *v > 0.5)),
        "fixture: the deposit laid no relief"
    );

    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_RESHAPE);
    t.set_deform_mode(0); // Push
    t.set_deform_size_norm(0.5);
    t.set_deform_pressure(1.0);
    (t, layer)
}

/// One Push stroke dragging rightward through the paint's middle.
fn push_right(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([70.0, 80.0], PointerPhase::Down));
    for i in 1..=6 {
        t.on_canvas_pointer(cp([70.0 + (i as f32) * 6.0, 80.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([106.0, 80.0], PointerPhase::Up));
}

fn heights_of(t: &PainterTool, l: crate::tool::RtLayerId) -> Vec<f32> {
    t.heights.get(&l).map(|h| (**h).clone()).unwrap_or_default()
}
fn covers_of(t: &PainterTool, l: crate::tool::RtLayerId) -> Vec<u8> {
    t.covers.get(&l).map(|c| (**c).clone()).unwrap_or_default()
}
fn mats_of(
    t: &PainterTool,
    l: crate::tool::RtLayerId,
) -> Vec<ph2d_painter_brush::material::MaterialBytes> {
    t.mats.get(&l).map(|m| (**m).clone()).unwrap_or_default()
}

/// The mass centroid's x of a non-negative field — the "where is the body" number a rigid drag moves.
fn centroid_x_f32(field: &[f32], w: u32) -> f64 {
    let (mut sum, mut wsum) = (0.0f64, 0.0f64);
    for (i, v) in field.iter().enumerate() {
        let v = f64::from(v.max(0.0));
        sum += v * ((i as u32 % w) as f64);
        wsum += v;
    }
    if wsum <= 0.0 { 0.0 } else { sum / wsum }
}
fn centroid_x_u8(field: &[u8], w: u32) -> f64 {
    let (mut sum, mut wsum) = (0.0f64, 0.0f64);
    for (i, v) in field.iter().enumerate() {
        let v = f64::from(*v);
        sum += v * ((i as u32 % w) as f64);
        wsum += v;
    }
    if wsum <= 0.0 { 0.0 } else { sum / wsum }
}

/// **The warp carries the BODY with the colour — heights, coverage and material ride the same drag.**
///
/// This is W4's whole sentence (doc 18 §5's exception): paint is a substance, and a Grab that slid the
/// colour out from under its own thickness would leave the light shading a ridge of paint that is no
/// longer there. The oracle is the artist's: drag the paint right, and *where the paint is* — its relief,
/// its coverage, its material — moves right with it.
///
/// **Mutation that must bleed:** delete the `warp_render_relief(bbox)` call in `warp/apply.rs` — the
/// pixels move, the body stays, and all three centroids stand still.
#[test]
fn the_warp_carries_the_body_with_the_colour() {
    let (mut t, layer) = deformable_relief_tool();
    let (w, _) = t.source_size;
    let h0 = heights_of(&t, layer);
    let c0 = covers_of(&t, layer);
    let m0 = mats_of(&t, layer);
    assert!(
        t.paint.warp.affect_relief,
        "fixture: the toggle defaults ON"
    );

    push_right(&mut t);

    let h1 = heights_of(&t, layer);
    let c1 = covers_of(&t, layer);
    let m1 = mats_of(&t, layer);

    let dh = centroid_x_f32(&h1, w) - centroid_x_f32(&h0, w);
    let dc = centroid_x_u8(&c1, w) - centroid_x_u8(&c0, w);
    assert!(
        dh > 1.0,
        "the relief's centroid moved {dh:.2} px under a firm rightward Push; the paint's pixels moved and \
         its BODY did not — the light now shades a ridge of paint that is no longer there. The warp must \
         advect `heights` along the same displacement as the colour."
    );
    assert!(
        dc > 0.5,
        "the coverage's centroid moved {dc:.2} px — the paint's silhouette stayed while its pixels moved"
    );
    let mats_moved = m0.iter().zip(&m1).filter(|(a, b)| a != b).count();
    assert!(
        mats_moved > 50,
        "the material plane changed at only {mats_moved} texels — a dragged fosco/brilhante boundary must \
         travel with the paint it describes"
    );
}

/// **Toggle OFF: the body is BYTE-IDENTICAL — the warp touches the colour only.**
///
/// The escape half of the toggle: warping a texture without disturbing its relief is a real workflow, and
/// "off" must mean off to the bit, not "mostly off".
///
/// **Mutation that must bleed:** make `warp_render_relief` ignore `affect_relief` — the planes move and
/// the byte-identity breaks.
#[test]
fn the_toggle_off_leaves_the_body_byte_identical() {
    let (mut t, layer) = deformable_relief_tool();
    t.toggle_deform_relief();
    assert!(!t.paint.warp.affect_relief, "fixture: toggled OFF");
    let h0 = heights_of(&t, layer);
    let c0 = covers_of(&t, layer);
    let m0 = mats_of(&t, layer);

    push_right(&mut t);

    let h1 = heights_of(&t, layer);
    let same_h =
        h0.len() == h1.len() && h0.iter().zip(&h1).all(|(a, b)| a.to_bits() == b.to_bits());
    assert!(
        same_h,
        "Affect Relief is OFF and the heights still moved — the toggle is cosmetic, which is worse than \
         no toggle (a control that lies about what it controls)"
    );
    assert_eq!(c0, covers_of(&t, layer), "…and the coverage moved");
    assert_eq!(m0, mats_of(&t, layer), "…and the material moved");
    // The presence sibling: the PIXELS did move (else the three identities above are vacuous).
    let pre = (*t.paint.warp.pre).clone();
    assert_ne!(
        pre, *t.canvas_rgba,
        "fixture: the warp moved no pixels at all, so the byte-identities above prove nothing"
    );
}

/// **Reset returns the body WITH the pixels — all four planes, one baseline.**
///
/// Reset's contract is "the session never happened". Giving back the colour and keeping the warped body
/// would leave the light shading a canvas the artist just asked to un-exist.
///
/// **Mutation that must bleed:** delete the `deform_restore_relief_planes()` call in `deform_reset` — the
/// pixels return, the relief stays warped.
#[test]
fn reset_returns_the_body_with_the_pixels() {
    let (mut t, layer) = deformable_relief_tool();
    let h0 = heights_of(&t, layer);
    let c0 = covers_of(&t, layer);
    let m0 = mats_of(&t, layer);
    let rgba0 = (*t.canvas_rgba).clone();

    push_right(&mut t);
    assert!(
        heights_of(&t, layer)
            .iter()
            .zip(&h0)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "fixture: the warp moved no relief, so the restore below proves nothing"
    );

    t.deform_reset();

    let same_h = heights_of(&t, layer)
        .iter()
        .zip(&h0)
        .all(|(a, b)| a.to_bits() == b.to_bits());
    assert!(
        same_h,
        "Reset restored the pixels but left the RELIEF warped — the session's body baseline was not \
         restored, and the light now disagrees with the colour about where the paint is"
    );
    assert_eq!(c0, covers_of(&t, layer), "…the coverage did not come back");
    assert_eq!(m0, mats_of(&t, layer), "…the material did not come back");
    assert_eq!(rgba0, *t.canvas_rgba, "…and the pixels themselves");
}

/// **Apply & Keep rebases the body's baseline with the pixels' — Reset then returns to the KEPT state.**
///
/// **Mutation that must bleed:** skip the relief rebase in `deform_apply_keep` — the later Reset walks the
/// body back past the bank, to a state the pixels can no longer reach.
#[test]
fn apply_and_keep_rebases_the_bodys_baseline() {
    let (mut t, layer) = deformable_relief_tool();
    let h_orig = heights_of(&t, layer);

    push_right(&mut t);
    let h_warped = heights_of(&t, layer);
    assert!(
        h_warped
            .iter()
            .zip(&h_orig)
            .any(|(a, b)| a.to_bits() != b.to_bits()),
        "fixture: no relief moved"
    );

    t.deform_apply_keep();
    t.deform_reset();

    let h_after = heights_of(&t, layer);
    let back_to_kept = h_after
        .iter()
        .zip(&h_warped)
        .all(|(a, b)| a.to_bits() == b.to_bits());
    assert!(
        back_to_kept,
        "Reset after Apply & Keep did not return the relief to the BANKED state — the body's baseline \
         was not rebased with the pixels', so the two now roll back to different canvases"
    );
}

/// **Reconstruct slides the body back with the colour — a real un-warp of all four planes.**
///
/// **Mutation that must bleed:** delete the `warp_render_relief(bbox)` call in `warp/reconstruct.rs` —
/// the pixels slide home and the relief stays where the Push left it.
#[test]
fn reconstruct_slides_the_body_back() {
    let (mut t, layer) = deformable_relief_tool();
    let h0 = heights_of(&t, layer);
    let (w, _) = t.source_size;

    push_right(&mut t);
    let pushed = centroid_x_f32(&heights_of(&t, layer), w) - centroid_x_f32(&h0, w);
    assert!(pushed > 1.0, "fixture: the Push moved no relief");

    t.set_deform_mode(5); // Reconstruct
    for _ in 0..6 {
        // Dwell over the warped area — each pass shrinks the displacement toward zero.
        t.on_canvas_pointer(cp([88.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp([58.0 + (i as f32) * 8.0, 80.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([122.0, 80.0], PointerPhase::Up));
    }

    let residue = centroid_x_f32(&heights_of(&t, layer), w) - centroid_x_f32(&h0, w);
    assert!(
        residue.abs() < pushed * 0.35,
        "after six Reconstruct passes the relief's centroid is still {residue:.2} px displaced (the Push \
         had moved it {pushed:.2}) — the un-warp slid the colour home and left the body behind"
    );
}

/// **Undo rolls the warp session back with the body — the frozen planes ride the snapshot.**
///
/// The same lesson `deform_disp` carries (Enio 2026-07-04), one plane over: an undo mid-session must
/// leave Reconstruct able to un-warp what remains — of the body as much as of the colour. The layer
/// planes themselves are already in `ModelSnapshot`; what this pins is the SESSION's frozen baselines
/// (`WarpSnap.pre_h` …) riding beside them.
///
/// **Mutation that must bleed:** stop carrying `pre_h` through `deform_for_snapshot`/`restore_deform`
/// (hand back an empty Arc) — after the undo the session has relief amnesia, and the next Reconstruct
/// dab can no longer move the body (the render's size guard refuses the empty plane).
#[test]
fn undo_rolls_the_warp_session_back_with_the_body() {
    let (mut t, layer) = deformable_relief_tool();
    let h0 = heights_of(&t, layer);
    let (w, _) = t.source_size;

    push_right(&mut t); // stroke 1 — commits one structural entry
    let mid = t.snapshot_model();
    push_right(&mut t); // stroke 2 — pushes further

    // Undo stroke 2 (the tool-level restore the painter's own Ctrl+Z takes).
    t.restore_model(mid);
    let after_undo = heights_of(&t, layer);
    assert!(
        t.paint.warp.pre_h.len() == after_undo.len(),
        "the undo dropped the session's frozen relief baseline — Reconstruct now has relief amnesia"
    );

    // …and Reconstruct must still be able to slide the REMAINING warp's body home.
    t.set_deform_mode(5);
    for _ in 0..6 {
        t.on_canvas_pointer(cp([88.0, 80.0], PointerPhase::Down));
        for i in 1..=8 {
            t.on_canvas_pointer(cp([58.0 + (i as f32) * 8.0, 80.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([122.0, 80.0], PointerPhase::Up));
    }
    let displaced_now = centroid_x_f32(&heights_of(&t, layer), w) - centroid_x_f32(&h0, w);
    let displaced_mid = centroid_x_f32(&after_undo, w) - centroid_x_f32(&h0, w);
    assert!(
        displaced_now.abs() < displaced_mid.abs() * 0.5 + 0.05,
        "Reconstruct after an undo left the body {displaced_now:.2} px displaced (it was \
         {displaced_mid:.2} after the undo) — the session snapshot did not carry the frozen relief, so \
         the un-warp has nothing to slide the body back FROM"
    );
}

/// **A session on a relief-bare layer stays relief-bare — no plane is invented.**
///
/// The guard side: Deform on plain paint (no impasto) must not conjure `heights`/`covers`/`mats` entries,
/// and the render must be a clean no-op through the empty-plane guards.
///
/// **Mutation that must bleed:** unconditionally insert planes in `ensure_deform_session` (freeze
/// `vec![0.0; n]` for a bare layer) — the layer gains a relief entry it never earned and
/// `sync_relief_flags` starts announcing relief on unpainted documents.
#[test]
fn a_bare_layer_gains_no_planes_from_a_warp() {
    let size = 96u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let layer = t.layers.active().expect("a layer");
    // Plain (non-impasto) paint so the warp has pixels to move but no body exists.
    let b = BrushSpec {
        radius_px: 10.0,
        color: [0.2, 0.6, 0.9],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.on_canvas_pointer(cp([30.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 48.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 48.0], PointerPhase::Up));

    t.set_paint_tool_mode("liquify");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_RESHAPE);
    t.set_deform_pressure(1.0);
    t.on_canvas_pointer(cp([45.0, 48.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([60.0, 48.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([60.0, 48.0], PointerPhase::Up));

    assert!(
        !t.heights.contains_key(&layer),
        "warping a layer with no impasto invented a heights plane for it"
    );
    assert!(
        t.paint.warp.pre_h.is_empty(),
        "…and froze a baseline for it"
    );
    assert!(t.paint.warp.relief_layer.is_none());
    let _ = Arc::clone(&t.canvas_rgba); // the pixels exist and moved; the body correctly never did
}

/// **O corpo viaja com a cor no TRANSFORM também** — o report do Enio (2026-08-08): *"as ferramentas do
/// painel Transform não atuam sobre o relevo de impasto. Em Liquify funciona corretamente"*.
///
/// ⚠️ **A W4 cobriu METADE do Deform.** Ela deu ao warp a advecção de `h`/`covers`/`mats` pelo `disp`
/// da sessão, e o `disp` só existe na metade das PINCELADAS (Push/Twist/Pinch/Wrinkle/Fold/
/// Reconstruct). A metade do GIZMO não tem `disp`: ela LEVANTA um patch e o compõe sobre a base, e os
/// quatro arquivos dela não citam `heights`/`covers`/`mats` uma única vez. A doutrina do `relief.rs` —
/// *corpo e cor não podem divergir sobre "para onde foi"* — valia para um render e não para o outro.
///
/// O oráculo é a própria doutrina: os dois **CENTROIDES** (o da tinta e o do relevo) têm de andar
/// juntos. Comparar posições absolutas seria frágil; comparar o DESLOCAMENTO de um contra o do outro é
/// a pergunta que a frase faz.
///
/// **Mutação que tem de sangrar:** tirar a chamada ao relevo do `composite_transform`.
#[test]
fn the_transform_carries_the_body_where_it_carries_the_colour() {
    let (mut t, layer) = deformable_relief_tool();
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);

    /// Centroide em x, pesado — `None` quando não há nada a pesar.
    fn centroid_x(vals: &[f32], w: u32, keep: impl Fn(f32) -> bool) -> Option<f32> {
        let (mut sum, mut wsum) = (0.0f64, 0.0f64);
        for (i, v) in vals.iter().enumerate() {
            if keep(*v) {
                let x = (i % (w as usize)) as f64;
                sum += x * f64::from(*v);
                wsum += f64::from(*v);
            }
        }
        (wsum > 0.0).then(|| (sum / wsum) as f32)
    }
    // ⚠️ **O peso da tinta é o PIGMENTO, não o alfa.** A fixture pinta sobre branco OPACO, então o alfa
    // é 255 na tela inteira e o centroide dele descreve a MOLDURA, não o traço: um Transform de camada
    // inteira que anda 20 px deixa uma faixa vazia de 20 px e move esse centroide ~10. Medir alfa
    // contra altura seria comparar duas populações diferentes — o erro que fez este gate acusar um
    // produto correto de mover o corpo "demais". O traço é vermelho sobre branco, então o déficit de
    // VERDE é exatamente a tinta que a pincelada depositou.
    let ink = |t: &PainterTool| {
        let g: Vec<f32> = t
            .canvas_rgba
            .chunks_exact(4)
            .map(|p| 255.0 - f32::from(p[1]))
            .collect();
        centroid_x(&g, 160, |v| v > 8.0).expect("fixture: a tinta existe")
    };
    let body = |t: &PainterTool| {
        centroid_x(&heights_of(t, layer), 160, |v| v > 0.5).expect("fixture: o relevo existe")
    };

    let (ink0, body0) = (ink(&t), body(&t));
    // Pega a alça central e arrasta o patch 20 px para a direita.
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Up));
    let (ink1, body1) = (ink(&t), body(&t));

    let ink_moved = ink1 - ink0;
    let body_moved = body1 - body0;
    assert!(
        ink_moved > 5.0,
        "fixture: o Transform não moveu a TINTA ({ink_moved:.2} px), então nada abaixo diz respeito ao \
         relevo"
    );
    assert!(
        (body_moved - ink_moved).abs() < 3.0,
        "o Transform moveu a tinta {ink_moved:.2} px e o CORPO {body_moved:.2} px — a luz vai sombrear \
         uma crista de tinta que não está mais lá"
    );
}

/// **E o relevo DEIXA o lugar de onde saiu** — a outra metade da mesma frase.
///
/// Mover o corpo para o destino é só metade de *"para onde foi"*: se o antigo lugar mantém a crista, a
/// luz desenha a tinta em DOIS sítios e o artista vê um fantasma que nenhum verbo alcança (a doença que
/// o §5 do plano 18 já pagou no `covers` write-once do Inflate). Aqui o buraco não é código extra: fora
/// da `affected` o composite escreve a BASE, e num Transform de camada inteira a base **não tem
/// cobertura nenhuma** (`c_base: None`) — o buraco é o que sobra quando ninguém inventa nada.
///
/// **Mutação que tem de sangrar:** escrever só onde `total > 0` (deixar o texel quieto quando não sobra
/// tinta) — a faixa vaga guarda a crista antiga e o gate a encontra.
#[test]
fn the_transform_leaves_no_ghost_ridge_where_the_body_used_to_be() {
    let (mut t, layer) = deformable_relief_tool();
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);

    // A faixa que a pincelada ocupava e que um arrasto de 20 px para a direita VAGA.
    let band = |t: &PainterTool| {
        let h = heights_of(t, layer);
        (30..50)
            .map(|x| h[(80 * 160 + x) as usize])
            .fold(0.0f32, f32::max)
    };
    let before = band(&t);
    assert!(
        before > 0.5,
        "fixture: a faixa 30..50 não tinha relevo ({before:.2}), então esvaziá-la não prova nada"
    );

    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Up));

    let after = band(&t);
    assert!(
        after < 0.5,
        "o corpo foi para o destino e FICOU também na origem (pico {after:.2} na faixa vaga) — a luz \
         sombreia uma crista de tinta que não está mais lá"
    );
}

/// **Pegar a ferramenta e devolvê-la ao lugar não custa tinta** — a reconstituição do par `over`.
///
/// O levante SEPARA os planos (patch × base) e o composite os RECONSTITUI. Duas perguntas, e este gate
/// faz as duas: **entrar** não pode escrever (o composite nem roda sem movimento — medido com uma sonda
/// que força `out_c = 42`), e a **IDA E VOLTA** é o único gesto que roda o composite com `minv`
/// identidade, porque o `recomposite_transform` re-renderiza sempre dos planos CONGELADOS.
///
/// ⚠️ **A barra da altura é MEDIDA, não escolhida, e duas versões deste gate reprovaram produto
/// correto antes desta.** Na ida e volta a COR e a COBERTURA voltam **byte-exatas** (0 bytes, 0 texels)
/// e a altura volta a **2 ulps (0,000002) onde a cobertura é CHEIA**. O que desvia até **0,43** são
/// **2881 texels do RIM**, e são de duas espécies, ambas sem consequência na tela: metade são **órfãos**
/// (`cover == 0` com altura ≠ 0), que o composite normaliza a zero **de propósito** — o próprio produto
/// declara que *cobertura zero é "não há tinta aqui"* —, e o resto é o rim de cobertura parcial. **A luz
/// pesa por COBERTURA**, então altura sobre cobertura ~0 contribui ~0 para a imagem: exigir byte-
/// identidade ali seria pinar um número que ninguém pode ver, sobre um produto que está certo.
///
/// **Mutação que tem de sangrar:** trocar o `Arc::clone(covers)` do levante de camada inteira por um
/// plano recomputado (`c·254/255`) — a cobertura deixa de voltar exata.
#[test]
fn a_transform_round_trip_gives_the_body_back() {
    let (mut t, layer) = deformable_relief_tool();
    let h0 = heights_of(&t, layer);
    let c0 = covers_of(&t, layer);
    let m0 = mats_of(&t, layer);
    let px0 = (*t.canvas_rgba).clone();
    assert!(
        h0.iter().any(|v| *v > 0.5),
        "fixture: há relevo a preservar"
    );

    // Metade 1 — ENTRAR não escreve.
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    assert!(
        t.deform_gizmo().is_some(),
        "fixture: o Transform não levantou patch nenhum, então não há reconstituição a julgar"
    );
    assert!(
        h0.iter()
            .zip(&heights_of(&t, layer))
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "levantar o patch — sem arrastar — já mexeu nas alturas; pegar a ferramenta para olhar custa tinta"
    );
    assert_eq!(c0, covers_of(&t, layer), "…e mexeu na cobertura");
    assert_eq!(m0, mats_of(&t, layer), "…e mexeu no material");

    // Metade 2 — IDA E VOLTA: o composite roda com `minv` identidade e tem de devolver o original.
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Up));

    assert_eq!(
        c0,
        covers_of(&t, layer),
        "a ida e volta não devolveu a COBERTURA — e é ela que a luz pesa, então isto o artista vê"
    );
    assert_eq!(px0, *t.canvas_rgba, "…e não devolveu a COR");
    let h1 = heights_of(&t, layer);
    let worst = c0
        .iter()
        .zip(h0.iter().zip(&h1))
        .filter(|(c, _)| **c > 250)
        .map(|(_, (a, b))| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-4,
        "a ida e volta moveu a ALTURA em {worst:.6} onde a tinta é opaca — ali `cp = 1` e o `over` tem \
         de reduzir à identidade; um desvio deste tamanho é o par levante/composite discordando"
    );
}

/// **`Affect Relief` desligado vale para o TRANSFORM também** — o toggle é da sessão de warp, não da
/// metade das pinceladas.
///
/// O irmão `the_toggle_off_leaves_the_body_byte_identical` prova isto para o Liquify. Sem este, a
/// metade do gizmo poderia honrar o toggle ou ignorá-lo e a suíte ficaria verde das duas maneiras — que
/// é exatamente como a advecção do W4 passou a existir só numa das metades.
///
/// **Mutação que tem de sangrar:** tirar o early-return de `affect_relief` do
/// `composite_transform_relief` — o corpo viaja com o toggle desligado.
#[test]
fn the_toggle_off_holds_for_the_transform_half_too() {
    let (mut t, layer) = deformable_relief_tool();
    t.toggle_deform_relief();
    assert!(!t.paint.warp.affect_relief, "fixture: toggled OFF");
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);
    let h0 = heights_of(&t, layer);
    let c0 = covers_of(&t, layer);
    let pre = (*t.canvas_rgba).clone();

    t.on_canvas_pointer(cp([80.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([100.0, 80.0], PointerPhase::Up));

    let h1 = heights_of(&t, layer);
    assert!(
        h0.len() == h1.len() && h0.iter().zip(&h1).all(|(a, b)| a.to_bits() == b.to_bits()),
        "Affect Relief está OFF e o Transform moveu as alturas — o toggle governa a sessão, e uma \
         metade que o ignora é um controle que mente sobre o que controla"
    );
    assert_eq!(c0, covers_of(&t, layer), "…e moveu a cobertura");
    // O irmão de PRESENÇA: a COR moveu, senão as identidades acima são vácuo.
    assert_ne!(
        pre, *t.canvas_rgba,
        "fixture: o Transform não moveu pixel nenhum, então as identidades acima não provam nada"
    );
}

/// **Um Transform de SELEÇÃO leva o corpo e deixa o buraco** — o irmão de relevo do
/// `deform_transform_lifts_the_selection_and_leaves_a_hole`, e a única rota que exercita a partição.
///
/// ⚠️ **Sem seleção o levante do corpo é um `Arc::clone` e `c_base` é `None`** (o buraco é total, e não
/// há nada a repartir); é só aqui que `lift_transform_relief` de fato PARTE a cobertura em `c·m` e
/// `c·(1−m)`, que é a metade do código que os outros gates desta família não alcançam — uma mutação no
/// braço `Some(mask)` passa por todos eles.
///
/// A seleção pega o fim da pincelada e o arrasta para **fora** dela: o destino tinha relevo ZERO, então
/// as duas metades da frase (*saiu de lá* · *chegou aqui*) são medíveis sem ambiguidade.
///
/// **Mutação que tem de sangrar:** dar à base a cobertura CHEIA (`Some(Arc::clone(covers))` no lugar de
/// `c·(1−m)`) — o fundo passa a mentir sobre o que ficou para trás e o buraco não abre.
#[test]
fn a_selected_transform_carries_the_body_and_leaves_the_hole() {
    let (mut t, layer) = deformable_relief_tool();
    t.set_shape_grab_tol_px(8.0);
    // O fim da pincelada (ela corre x≈36..124 em y=80), para que o destino caia em tela nua.
    t.set_rect_selection(95, 66, 20, 28);
    t.set_deform_temperament(super::super::DEFORM_TEMPERAMENT_TRANSFORM);

    let at = |t: &PainterTool, x: u32| heights_of(t, layer)[(80 * 160 + x) as usize];
    let (origin0, dest0) = (at(&t, 105), at(&t, 135));
    assert!(
        origin0 > 0.5 && dest0 < 0.5,
        "fixture: a seleção tem de sair de relevo ({origin0:.2}) para tela nua ({dest0:.2}), senão as \
         duas metades abaixo não distinguem nada"
    );

    // Pega o centro da seleção (105, 80) e arrasta +30 px.
    t.on_canvas_pointer(cp([105.0, 80.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([135.0, 80.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([135.0, 80.0], PointerPhase::Up));

    let (origin1, dest1) = (at(&t, 105), at(&t, 135));
    assert!(
        dest1 > 0.5,
        "o corpo não CHEGOU ao destino (altura {dest1:.2} onde a tinta agora está) — a cor viajou \
         sozinha e a luz não tem o que sombrear"
    );
    assert!(
        origin1 < origin0 * 0.5,
        "o corpo não SAIU da origem ({origin0:.2} → {origin1:.2}) — a seleção levou os pixels e deixou \
         a crista, que é a luz sombrando tinta que não está mais lá"
    );
}
