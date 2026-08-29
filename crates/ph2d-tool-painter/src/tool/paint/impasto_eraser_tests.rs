//! Gates do **ERASER do Impasto sob os métodos de FORMA** — o relevo que ele tira tem de ser função da
//! FIGURA, nunca de quantas vezes o motor a re-carimbou enquanto a mão a arrastava.
//!
//! A lei já estava escrita, uma família ao lado: o `stamp_drag_preview` explica que o sculpt *reescreve o
//! plano da camada AO VIVO, então tem de restaurá-lo* — sem isso *"uma Curve cavaria mais fundo a cada
//! movimento do ponteiro enquanto o artista apenas OLHAVA"*. O eraser do impasto reescreve o mesmo plano
//! pelo mesmo motivo (não há envelope próprio a descascar: ele esfrega o que já está commitado) e **não
//! tinha restauração nenhuma**.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::CanvasPaintTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_painter_brush::{Falloff, StrokeMethod};
use std::sync::Arc;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma camada que JÁ carrega massa — a única coisa sobre a qual um eraser de relevo tem o que dizer.
fn eraser_canvas(size: u32) -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 14.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_brush_impasto(true);
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("uma camada");
    let n = (size * size) as usize;
    t.heights.insert(layer, Arc::new(vec![0.6f32; n]));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();
    (t, layer)
}

fn heights_of(t: &PainterTool, layer: crate::tool::RtLayerId) -> Vec<f32> {
    t.heights
        .get(&layer)
        .map(|f| f.to_vec())
        .unwrap_or_default()
}

/// Borracha em mãos com o método LINE — o editor de forma que re-carimba a figura inteira por quadro.
fn arm_eraser_line(t: &mut PainterTool) {
    t.paint.eraser = true;
    let mut b = t.paint.brush;
    b.stroke_method = StrokeMethod::Line;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Paint.slot()] = b;
}

fn place(t: &mut PainterTool, p: [f32; 2]) {
    t.on_canvas_pointer(cp(p, PointerPhase::Down));
    t.on_canvas_pointer(cp(p, PointerPhase::Up));
}

/// **A massa que a borracha tira é função da FIGURA, não do caminho da mão até ela.**
///
/// Enio, 2026-08-07: *"no modo eraser de impasto, as shapes vivas do stroke estão marcando (reduzindo o
/// relevo) da massa antes de apertar apply/enter"*. O `erase_dab_height` faz `dst[i] *= scrub`, um
/// **PRODUTO** — e um editor de forma re-carimba a figura inteira a cada movimento do ponteiro, então o
/// produto passa a ser sobre QUADROS: arrastar o mesmo traço por um desvio come mais massa que desenhá-lo
/// direto, e não há volta antes do Apply.
///
/// O oráculo é o par de corridas, não um limiar: a MESMA linha, alcançada por dois caminhos, tem de
/// deixar o MESMO plano.
///
/// **Mutação que sangra:** tirar o `restamp_reset_erase()` dos DOIS sítios de re-carimbo — 1158 texels,
/// o pior 0,6000 mais fundo. ⚠️ **Tirar só um deixa este gate VERDE**, e isso está medido: para ESTE
/// gesto as duas devoluções são caminhos alternativos (a do rascunho corre durante o arrasto, a do
/// `stamp_drag_preview` no carimbo final) e a devolução é idempotente. Cada camada tem gate PRÓPRIO
/// abaixo — é o que impede uma delas de virar código morto sem ninguém ver
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn the_erased_relief_is_a_function_of_the_shape_not_of_how_it_was_dragged_there() {
    let size = 200u32;
    let (a, b) = ([60.0, 100.0], [140.0, 100.0]);
    let (c, d) = ([90.0, 60.0], [170.0, 150.0]);

    // Corrida 1 — a linha A→B, direta.
    let (mut t1, l1) = eraser_canvas(size);
    arm_eraser_line(&mut t1);
    place(&mut t1, a);
    place(&mut t1, b);
    let straight = heights_of(&t1, l1);
    let scrubbed = straight.iter().filter(|&&v| v < 0.6 - 1e-6).count();
    assert!(
        scrubbed > 200,
        "fixture: a borracha tirou massa de apenas {scrubbed} texels — o editor nunca carimbou, e a \
         comparação abaixo seria entre dois no-ops"
    );

    // Corrida 2 — a MESMA linha, com a ponta arrastada por um desvio antes de pousar em B.
    let (mut t2, l2) = eraser_canvas(size);
    arm_eraser_line(&mut t2);
    place(&mut t2, a);
    place(&mut t2, c);
    t2.on_canvas_pointer(cp(c, PointerPhase::Down));
    t2.on_canvas_pointer(cp(d, PointerPhase::Move));
    t2.on_canvas_pointer(cp(b, PointerPhase::Move));
    t2.on_canvas_pointer(cp(b, PointerPhase::Up));
    let dragged = heights_of(&t2, l2);

    let differing = straight
        .iter()
        .zip(&dragged)
        .filter(|(x, y)| x.to_bits() != y.to_bits())
        .count();
    let deepest = straight
        .iter()
        .zip(&dragged)
        .map(|(x, y)| x - y)
        .fold(0.0f32, f32::max);
    assert_eq!(
        differing, 0,
        "a mesma linha, alcançada por dois caminhos, deixou dois relevos ({differing} texels, o pior \
         {deepest:.4} mais fundo). A borracha está carregando a HISTÓRIA do arrasto: cada re-carimbo \
         esfrega o plano da camada de novo, e não há volta antes do Apply."
    );
}

/// **A massa sob uma figura de que a mão se AFASTOU volta.** A metade complementar, e a que o artista
/// vê primeiro: o gate acima compara duas corridas e ficaria verde se as duas ficassem igualmente fundas.
///
/// O texel medido saiu da própria repro (o primeiro em que os dois relevos discordavam): ele fica sob a
/// posição INTERMEDIÁRIA da linha, por onde a figura final não passa. Antes da cura ele lia **0,0** — a
/// massa comida até o zero por uma figura que já não existe.
///
/// **Mutação que sangra:** tirar as DUAS devoluções (o irmão acima explica por que uma só não basta).
#[test]
fn the_mass_under_a_shape_the_hand_moved_away_from_comes_back() {
    let size = 200u32;
    let (a, c, d, b) = (
        [60.0, 100.0],
        [90.0, 60.0], // …a figura intermediária mora aqui em cima…
        [170.0, 150.0],
        [140.0, 100.0], // …e a final, na horizontal, não passa por lá.
    );
    let (mut t, l) = eraser_canvas(size);
    arm_eraser_line(&mut t);
    place(&mut t, a);
    place(&mut t, c);
    t.on_canvas_pointer(cp(c, PointerPhase::Down));
    t.on_canvas_pointer(cp(d, PointerPhase::Move));
    t.on_canvas_pointer(cp(b, PointerPhase::Move));
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
    let h = heights_of(&t, l);
    let probe = ((48 * size) + 85) as usize;
    assert_eq!(
        h.get(probe).map(|v| v.to_bits()),
        Some(0.6f32.to_bits()),
        "a massa sob a figura ABANDONADA tem de voltar ao bit — ela leu {:?}",
        h.get(probe)
    );
}

/// **E apagar a figura devolve a massa que ela mordeu.** Um gesto diferente dos dois de cima — o
/// `delete_active_shape` tira UMA figura da mesa e deixa as outras de pé —, e ele já dizia a frase para o
/// relevo do depósito e para o sculpt: *"deixá-los de pé faria a luz iluminar uma crista sobre tinta que
/// não está mais lá"*. Para a borracha é o oposto exato e o mesmo erro: a massa comida por uma figura que
/// não existe mais.
///
/// **Mutação que sangra:** as duas devoluções. ⚠️ E este gate mediu uma coisa a mais: uma TERCEIRA
/// chamada, dentro do próprio `delete_active_shape`, foi escrita e **removida por ser código morto** — o
/// `restamp_shapes_preview(&[])` da linha seguinte já devolve. Tirá-la não sangrava nada.
#[test]
fn deleting_a_live_shape_gives_back_the_mass_it_had_eaten() {
    let size = 200u32;
    let (a, c) = ([60.0, 100.0], [90.0, 60.0]);
    let (mut t, l) = eraser_canvas(size);
    arm_eraser_line(&mut t);
    place(&mut t, a);
    place(&mut t, c);
    let probe = ((48 * size) + 85) as usize;
    let bitten = heights_of(&t, l);
    assert!(
        bitten[probe] < 0.6 - 1e-6,
        "fixture: a borracha nao mordeu o texel medido ({}), e o teste abaixo seria vacuo",
        bitten[probe]
    );
    assert!(t.delete_active_shape(), "a figura viva foi apagada");
    let after = heights_of(&t, l);
    assert_eq!(
        after.get(probe).map(|v| v.to_bits()),
        Some(0.6f32.to_bits()),
        "a massa da figura APAGADA tem de voltar ao bit — ela leu {:?}",
        after.get(probe)
    );
}

/// **E o DRAG DOT — o carimbo único que segue o cursor — não pode comer o caminho inteiro.**
///
/// Ele nunca passa pelo `restamp_shapes_preview` (não há figura a re-carimbar): o único sítio que o
/// alcança é o `stamp_drag_preview`. É a camada que os dois gates de cima não testam, e sem ela arrastar
/// a borracha de A para B deixa uma trilha de massa comida ao longo de TODO o percurso, permanente, com o
/// carimbo final por cima.
///
/// **Mutação que sangra:** tirar o `restamp_reset_erase()` do `stamp_preview::stamp_drag_preview`.
#[test]
fn a_drag_dot_eraser_eats_only_where_it_lands() {
    let size = 200u32;
    let (mut t, l) = eraser_canvas(size);
    t.paint.eraser = true;
    let mut b = t.paint.brush;
    b.stroke_method = StrokeMethod::DragDot;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Paint.slot()] = b;

    t.on_canvas_pointer(cp([60.0, 100.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([100.0, 100.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([140.0, 100.0], PointerPhase::Move));
    let h = heights_of(&t, l);
    let at = |x: u32, y: u32| h[((y * size) + x) as usize];
    assert!(
        at(140, 100) < 0.6 - 1e-6,
        "fixture: o carimbo final nao mordeu nada ({}) — o resto seria vacuo",
        at(140, 100)
    );
    assert_eq!(
        at(60, 100).to_bits(),
        0.6f32.to_bits(),
        "a massa de ONDE O CARIMBO ESTAVA tem de voltar: leu {}",
        at(60, 100)
    );
}

/// **E enquanto a mão AINDA arrasta, a massa já voltou.** A camada que os gates de estado final não
/// alcançam: sob a mão o gizmo É o preview (`restamp_shapes_preview` descasca e não re-carimba), então
/// nada devolveria a mordida até o artista soltar — e é literalmente o quadro que ele está olhando
/// quando diz que *"as shapes vivas estão marcando a massa"*.
///
/// **Mutação que sangra:** tirar o `restamp_reset_erase()` do ramo do RASCUNHO em
/// `stroke_multi::restamp_shapes_preview` — o `stamp_drag_preview` só a devolveria no Up.
#[test]
fn the_mass_is_back_while_the_hand_is_still_dragging() {
    let size = 200u32;
    let (a, c, d) = ([60.0, 100.0], [90.0, 60.0], [170.0, 150.0]);
    let (mut t, l) = eraser_canvas(size);
    arm_eraser_line(&mut t);
    place(&mut t, a);
    place(&mut t, c);
    let probe = ((48 * size) + 85) as usize;
    assert!(
        heights_of(&t, l)[probe] < 0.6 - 1e-6,
        "fixture: a figura pousada nao mordeu o texto medido"
    );
    t.on_canvas_pointer(cp(c, PointerPhase::Down)); // pega a ponta…
    t.on_canvas_pointer(cp(d, PointerPhase::Move)); // …e ARRASTA — sem soltar
    let mid = heights_of(&t, l);
    assert_eq!(
        mid.get(probe).map(|v| v.to_bits()),
        Some(0.6f32.to_bits()),
        "a massa tem de estar de volta NO MEIO do arrasto, nao so no Up — leu {:?}",
        mid.get(probe)
    );
}

/// **E uma mordida APLICADA é permanente — o gesto seguinte não a devolve.**
///
/// A metade oposta das três de cima, e a que o `commit_drag_preview` chama de arma carregada quando
/// descreve o sculpt: uma sessão que sobrevive ao seu gesto faz o PRIMEIRO quadro de preview do gesto
/// seguinte devolver massa que o artista apagou de verdade. Não é uma mordida a menos: é trabalho
/// desfeito sozinho.
///
/// **Mutação que sangra:** tirar o `drop_erase_session()` do `commit_drag_preview` **e** do
/// `close_stroke` — os dois, porque o segundo é no-op depois do primeiro (uma morte, não duas).
#[test]
fn an_applied_bite_survives_the_next_gesture() {
    let size = 200u32;
    let (a, b) = ([60.0, 100.0], [140.0, 100.0]);
    let (c, d, e) = ([40.0, 170.0], [60.0, 180.0], [150.0, 185.0]);
    let (mut t, l) = eraser_canvas(size);
    arm_eraser_line(&mut t);
    place(&mut t, a);
    place(&mut t, b);
    assert!(t.line_commit(), "a primeira linha foi APLICADA");
    let probe = ((100 * size) + 100) as usize;
    let applied = heights_of(&t, l)[probe];
    assert!(
        applied < 0.6 - 1e-6,
        "fixture: a primeira linha nao mordeu o texel medido ({applied})"
    );

    // Um SEGUNDO gesto, longe dali, com um arrasto — o quadro em que a sessão velha morderia de volta.
    place(&mut t, c);
    place(&mut t, d);
    t.on_canvas_pointer(cp(d, PointerPhase::Down));
    t.on_canvas_pointer(cp(e, PointerPhase::Move));
    t.on_canvas_pointer(cp(e, PointerPhase::Up));
    let after = heights_of(&t, l)[probe];
    assert_eq!(
        after.to_bits(),
        applied.to_bits(),
        "a mordida APLICADA voltou sozinha ({applied} -> {after}): a sessão da borracha sobreviveu ao \
         gesto que a criou"
    );
}
