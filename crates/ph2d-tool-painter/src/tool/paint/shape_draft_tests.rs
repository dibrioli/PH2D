//! **O meio caro renderiza em REPOUSO** — os gates de [`super::super::shape_draft`].
//!
//! ⚠️ **Todos dirigem `on_canvas_pointer`**, a porta do artista, e não os `*_refill` por dentro: a lei
//! mora no roteador de ponteiro, e um gate que chamasse o refill direto ficaria VERDE com o roteador
//! desligado — provaria a ablação e não o produto.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

/// Quantos texels da tela deixaram de ser papel branco — *"há tinta na tela?"*.
///
/// ⚠️ **É o oráculo do que o ARTISTA vê**, e é por isso que a lei do repouso se mede por ele e não
/// pelo relógio: um bar de tempo mediria o perfil do build (a lição que esta linha já pagou).
fn painted(t: &crate::tool::PainterTool) -> usize {
    t.canvas_rgba
        .iter()
        .step_by(4)
        .zip(t.canvas_rgba.iter().skip(1).step_by(4))
        .filter(|(r, g)| **r != 255 || **g != 255)
        .count()
}

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

/// **A wave inteira se apoia nisto:** com a mão em movimento a tinta SOME, e ao soltar ela volta.
///
/// O relevo entra como corolário — sem carimbo não há corpo — e é ele que prova que o carimbo final
/// roda com o meio de VERDADE, e não numa versão desarmada.
#[test]
fn the_paint_is_gone_under_the_hand_and_back_at_rest() {
    let (mut t, up_at) = drag_an_ellipse(PaintMedia::Impasto);
    let under_the_hand = painted(&t);
    assert_eq!(
        under_the_hand, 0,
        "um gesto EM VOO nao pode deixar tinta na tela — o gizmo e o preview, e mediu {under_the_hand} texels"
    );
    assert_eq!(relief(&t), 0.0, "nem corpo");
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    assert!(
        painted(&t) > 0,
        "ao SOLTAR a tinta tem de voltar — o carimbo final nao rodou"
    );
    let at_rest = relief(&t);
    assert!(
        at_rest > 0.0,
        "o carimbo final rodou sem o meio de verdade (relevo {at_rest})"
    );
}

/// **A lei não é sobre o MEIO — ela vale para o Digital também**, porque o que ela pula é o composite
/// booleano, e ele custa o mesmo em qualquer meio (medido: 284 dos 308 ms, com o pincel quase
/// irrelevante).
///
/// ⚠️ Uma 1ª versão desta wave gateava o meio caro e deixava o Digital intacto — e o smoke reprovou
/// exatamente aí: *"mesmo o preview plano é extremamente custoso"*. Este gate é o que impede alguém
/// de re-estreitar a lei ao Impasto/Aquarela e reintroduzir o defeito reportado.
#[test]
fn the_plain_brush_disappears_under_the_hand_too() {
    let (mut t, up_at) = drag_an_ellipse(PaintMedia::Digital);
    assert_eq!(
        painted(&t),
        0,
        "o Digital continuou pintando sob a mao — a lei foi estreitada ao meio caro"
    );
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    assert!(
        painted(&t) > 0,
        "o Digital nao voltou ao soltar — a fixture nao pinta nada, ou o carimbo final nao roda"
    );
}

/// **O que a lei existe para pular: o composite BOOLEANO.** Com Operation Add um gesto em voo não
/// pode traçar contorno nenhum.
///
/// ⚠️ Este é o gate que fala do NÚMERO do report — 284 dos 308 ms de um move são rasterizar as
/// figuras e traçar os contornos, `O(área da união × SS²)`, refeito por quadro do arrasto para
/// produzir um contorno que a mão invalida no quadro seguinte. O contador do diag é o oráculo: ele
/// conta CHAMADAS do composite, não milissegundos.
#[test]
fn a_gesture_in_flight_runs_no_boolean_composite() {
    use crate::tool::paint::stroke_boolean::diag;
    let side = 256u32;
    let mut t = tool(side, PaintMedia::Digital, 12.0);
    t.set_stroke_op_mode(1); // Add — a figura entra no composite booleano
    t.paint.brush.stroke_method = StrokeMethod::Ellipse;
    #[allow(clippy::cast_precision_loss)]
    let c = (side / 2) as f32;
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    t.on_canvas_pointer(cp([c + 50.0, c], PointerPhase::Move));
    t.on_canvas_pointer(cp([c + 50.0, c], PointerPhase::Up));
    // O gesto de AJUSTE, com o contador zerado logo antes dos Moves.
    t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
    let _ = diag::take();
    t.on_canvas_pointer(cp([c + 8.0, c + 6.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([c + 14.0, c + 10.0], PointerPhase::Move));
    assert_eq!(
        diag::take().calls,
        0,
        "o composite booleano rodou sob a mao — sao 92% do custo do move na cena do report"
    );
    t.on_canvas_pointer(cp([c + 14.0, c + 10.0], PointerPhase::Up));
    assert!(
        diag::take().calls > 0,
        "o composite nao rodou ao soltar — a figura fica sem o contorno booleano"
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
    let under_the_hand = painted(&t);
    assert_eq!(
        under_the_hand, 0,
        "ajustar uma figura existente devia sumir tambem, e mediu {under_the_hand} texels"
    );
    t.on_canvas_pointer(cp(up_at, PointerPhase::Up));
    assert!(
        !t.paint.shape_stale,
        "o Up do AJUSTE deixou a tela DEVENDO — a figura fica invisivel ate o proximo evento"
    );
    assert!(
        painted(&t) > 0,
        "ao soltar o ajuste a figura tem de voltar a tela"
    );
    let at_rest = relief(&t);
    assert!(
        at_rest > 0.0,
        "…e com o corpo: o carimbo final rodou sem o meio de verdade (relevo {at_rest})"
    );
}

/// **Nenhum editor deixa a tela DEVENDO ao soltar** — a propriedade, perguntada aos quatro.
///
/// ⚠️ O `commit_shape_txn` assenta a tela antes de capturar, mas ele **sai cedo quando não há
/// transação aberta** (`stroke_undo == None`) — e aí a captura não acontece, logo a correção não é
/// devida, mas a TELA ainda está. Este gate varre os quatro editores porque *qual deles fecha o Up
/// sem transação* é detalhe interno que muda; a propriedade não.
#[test]
fn no_editor_leaves_the_canvas_owing_at_rest() {
    for method in [
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
        StrokeMethod::Line,
        StrokeMethod::Arc,
    ] {
        let side = 256u32;
        let mut t = tool(side, PaintMedia::Digital, 10.0);
        t.paint.brush.stroke_method = method;
        #[allow(clippy::cast_precision_loss)]
        let c = (side / 2) as f32;
        // Criação, e depois DOIS gestos de ajuste — o 2º pega onde o 1º largou.
        t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
        t.on_canvas_pointer(cp([c + 40.0, c], PointerPhase::Move));
        t.on_canvas_pointer(cp([c + 40.0, c + 20.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([c + 40.0, c + 20.0], PointerPhase::Up));
        assert!(
            !t.paint.shape_stale,
            "{method:?}: a criacao deixou a tela devendo"
        );
        for _ in 0..2 {
            t.on_canvas_pointer(cp([c, c], PointerPhase::Down));
            t.on_canvas_pointer(cp([c + 6.0, c + 4.0], PointerPhase::Move));
            t.on_canvas_pointer(cp([c + 12.0, c + 8.0], PointerPhase::Move));
            t.on_canvas_pointer(cp([c + 12.0, c + 8.0], PointerPhase::Up));
            assert!(
                !t.paint.shape_stale,
                "{method:?}: um gesto deixou a tela devendo — a figura fica invisivel ate o proximo evento"
            );
        }
    }
}
