//! **O meio caro renderiza em REPOUSO** — os gates de [`super::super::shape_draft`].
//!
//! ⚠️ **Todos dirigem `on_canvas_pointer`**, a porta do artista, e não os `*_refill` por dentro: a lei
//! mora no roteador de ponteiro, e um gate que chamasse o refill direto ficaria VERDE com o roteador
//! desligado — provaria a ablação e não o produto.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// Quanto CORPO de tinta a figura VIVA carrega.
///
/// ⚠️ **O envelope por-traço, não o plano commitado** (`heights`): o depósito acumula o relevo em
/// `relief.stroke_height` e só o funde na camada no COMMIT — que para um shape editor é o Apply/Enter,
/// não o Up do ponteiro. A 1ª versão deste gate lia o plano e nascia VERMELHA na metade *"em
/// repouso"* com o produto correto: a fixture não continha o fenômeno, ela media outro. É a MESMA
/// grandeza que o `impasto_visible` consulta para decidir se acende a luz.
fn relief(t: &crate::tool::PainterTool) -> f32 {
    t.paint.relief.stroke_height.iter().map(|v| v.abs()).sum()
}

/// Desenha uma elipse com um arrasto REAL e devolve o tool com o gesto ainda ABERTO no último Move.
fn drag_an_ellipse(media: PaintMedia) -> (crate::tool::PainterTool, [f32; 2]) {
    let side = 256u32;
    let mut t = tool(side, media, 12.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    #[allow(clippy::cast_precision_loss)]
    let c = (side / 2) as f32;
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c + 40.0, c], PointerPhase::Move));
    t.on_canvas_pointer(cp([c + 60.0, c], PointerPhase::Move));
    (t, [c + 60.0, c])
}

/// A elipse já EXISTE e o artista a está **movendo** — o gesto que o report descreve.
///
/// ⚠️ **Não é o mesmo caminho da criação, e a diferença é o que o gate 5 mede:** o `ellipse_up`
/// re-carimba no ramo de CRIAÇÃO e sai por `return true` no ramo de EDIÇÃO (`ed.editing`), então só
/// esta fixture alcança o fallback do roteador. Uma bateria que só criasse figuras deixaria a mutação
/// *"tire o fallback"* passar — e o produto ficaria com a figura plana depois de todo arrasto de
/// ajuste, que é literalmente o gesto reportado.
fn move_an_existing_ellipse(media: PaintMedia) -> (crate::tool::PainterTool, [f32; 2]) {
    let side = 256u32;
    let mut t = tool(side, media, 12.0);
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    #[allow(clippy::cast_precision_loss)]
    let c = (side / 2) as f32;
    // Criação, fechada com o Up — daqui em diante `editing` é verdadeiro.
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c + 50.0, c], PointerPhase::Move));
    t.on_canvas_pointer(cp([c + 50.0, c], PointerPhase::Up));
    // O gesto de AJUSTE: pega o centro e arrasta.
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c + 8.0, c + 6.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([c + 14.0, c + 10.0], PointerPhase::Move));
    (t, [c + 14.0, c + 10.0])
}

/// **A wave inteira se apoia nisto:** com a mão em movimento a figura sai PLANA, e ao soltar ela
/// ganha o corpo.
///
/// ⚠️ **O oráculo é o RELEVO, não o relógio** — um gate de tempo mediria o perfil do build (a lição
/// que esta linha já pagou), enquanto *"há corpo de tinta na camada?"* é o fato que o artista vê.
#[test]
fn the_expensive_medium_renders_at_rest_not_under_the_hand() {
    let (mut t, up_at) = drag_an_ellipse(PaintMedia::Impasto);
    let under_the_hand = relief(&t);
    assert_eq!(
        under_the_hand, 0.0,
        "um gesto EM VOO nao pode depositar corpo — o re-carimbo devia sair rascunhado, e mediu {under_the_hand}"
    );
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    let at_rest = relief(&t);
    assert!(
        at_rest > 0.0,
        "ao SOLTAR a figura tem de ganhar o corpo — o carimbo final nao rodou (relevo {at_rest})"
    );
}

/// **O CONTROLE:** o Digital não tem meio caro para desarmar, então o rascunho não pode movê-lo.
///
/// ⚠️ Sem esta metade a wave passaria com uma ablação larga demais (algo que apagasse o depósito
/// inteiro durante o gesto), e os pixels do pincel comum sumiriam sob a mão sem nenhum gate reclamar.
/// A afirmação é **byte a byte**: o que o Move desenha é o que o Up desenha.
#[test]
fn the_plain_brush_draws_the_same_under_the_hand_and_at_rest() {
    let (mut t, up_at) = drag_an_ellipse(PaintMedia::Digital);
    let under_the_hand = t.canvas_rgba.as_ref().clone();
    let painted = under_the_hand
        .iter()
        .step_by(4)
        .filter(|a| **a != 255)
        .count();
    assert!(
        painted > 0,
        "a fixture nao pintou nada — o gate ficaria verde sobre duas telas em branco"
    );
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    assert_eq!(
        under_the_hand,
        t.canvas_rgba.as_ref().clone(),
        "o pincel comum mudou ao soltar — o rascunho vazou para um meio que nao tem meio caro"
    );
}

/// **O PAINEL não pisca.** `watercolor_active` responde *"que meio o artista escolheu?"*, e essa
/// resposta não pode depender de a mão estar em movimento.
///
/// ⚠️ É o gate que separa as duas perguntas: se o snapshot voltar a ler `watercolor_render_active`, a
/// row **Accumulate** aparece e some a cada arrasto de figura.
#[test]
fn the_watercolor_chip_does_not_flicker_while_the_shape_is_dragged() {
    let (mut t, up_at) = drag_an_ellipse(PaintMedia::Watercolor);
    assert!(
        t.brush_settings().watercolor_active,
        "o painel perdeu a aquarela com a mao em movimento — a UI segue uma decisao de RENDER"
    );
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    assert!(
        t.brush_settings().watercolor_active,
        "o painel perdeu a aquarela em repouso"
    );
}

/// **AJUSTAR uma figura que já existe** — o gesto que o report descreve, e o que exercita o fallback.
///
/// ⚠️ O ramo `editing` do `ellipse_up` **não** re-carimba (ele fecha a transação de undo e sai), então
/// sem o fallback do roteador a figura ficaria com a cara do rascunho até o próximo evento — plana,
/// depois de todo arrasto de ajuste. É por isto que o contador existe em vez de uma lista de ramos.
#[test]
fn adjusting_an_existing_shape_also_ends_at_rest() {
    let (mut t, up_at) = move_an_existing_ellipse(PaintMedia::Impasto);
    let under_the_hand = relief(&t);
    assert_eq!(
        under_the_hand, 0.0,
        "ajustar uma figura existente devia rascunhar tambem, e mediu {under_the_hand}"
    );
    let before = t.paint.restamp_seq;
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    assert!(
        t.paint.restamp_seq > before,
        "o Up do AJUSTE nao re-carimbou — a figura fica plana ate o proximo evento"
    );
    let at_rest = relief(&t);
    assert!(
        at_rest > 0.0,
        "ao soltar o ajuste a figura tem de recuperar o corpo (relevo {at_rest})"
    );
}
