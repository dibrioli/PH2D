//! Gates das **TECLAS ACESSÓRIAS ao DESENHAR uma seleção** — a outra metade do pedido do Enio
//! (2026-08-07: *"as seleções ainda não têm aquele sistema de teclas acessórias (shift e ctrl) para
//! escalonar a partir do centro e manter as proporções"*), e a que ele testou primeiro.
//!
//! ⚠️ **O `selection_canvas_pointer` tem DOIS donos** e a primeira rodada só tocou um: `selection_edit_mode`
//! roteia ao gizmo (a lei foi para lá) e o **senão** roteia ao `selection_down`/`_move`/`_up`, que é o gesto
//! de *desenhar* a marquee — o que se faz **antes de existir um gizmo para editar**. Com o diagnóstico
//! ligado o shell falava e o tool calava, que é exatamente a assinatura de *"o gesto não é nenhum dos que
//! têm lei"*.
//!
//! A lei é a mesma dos dois lados ([`ph2d_editor_core::GizmoModifiers`]): **Shift** trava a razão de
//! aspecto, **Ctrl/Cmd** ancora no CENTRO, default = o canto onde a mão pousou.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::CanvasPaintTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

const RECT: u8 = 2;
const ELLIPSE: u8 = 3;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Desenha UMA marquee com as teclas dadas e devolve `(centro, hx, hy)` lidos do **gizmo** — o oráculo é o
/// que o artista vê e agarra depois de soltar, não um campo interno.
fn draw(mode: u8, anchor: [f32; 2], to: [f32; 2], shift: bool, ctrl: bool) -> ([f32; 2], f32, f32) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 128 * 128 * 4], 128, 128);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(mode);
    t.set_gizmo_modifiers(shift, ctrl, false);
    t.on_canvas_pointer(cp(anchor, PointerPhase::Down));
    t.on_canvas_pointer(cp(to, PointerPhase::Move));
    t.on_canvas_pointer(cp(to, PointerPhase::Up));
    t.toggle_selection_edit();
    let g = &t.selection_gizmos()[0];
    let c = g.center;
    let r = g.scale_handles[4]; // meio do lado DIREITO
    let top = g.scale_handles[5]; // meio do lado de CIMA
    (
        c,
        ((r[0] - c[0]).powi(2) + (r[1] - c[1]).powi(2)).sqrt(),
        ((top[0] - c[0]).powi(2) + (top[1] - c[1]).powi(2)).sqrt(),
    )
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.05
}

/// **O CONTROLE, e ele vem primeiro:** sem tecla nenhuma o gesto é o que sempre foi. Um arrasto de
/// (16,16) a (56,32) numa elipse dá centro (36,24) e raios 20 × 8 — a mesma aritmética que shipava antes
/// desta wave, e é ela que torna a `marquee_corners` uma adição em vez de uma mudança de comportamento.
///
/// **Mutação que sangra:** trocar o ramo sem-tecla por qualquer um dos outros (por exemplo devolver
/// sempre o par espelhado do Ctrl) — o centro salta para a âncora.
#[test]
fn without_a_key_the_marquee_is_what_it_always_was() {
    let (c, hx, hy) = draw(ELLIPSE, [16.0, 16.0], [56.0, 32.0], false, false);
    assert!(
        near(c[0], 36.0) && near(c[1], 24.0) && near(hx, 20.0) && near(hy, 8.0),
        "sem tecla o gesto não pode mudar: centro {c:?}, raios ({hx}, {hy})"
    );
}

/// **Shift dá um CÍRCULO PERFEITO** de um arrasto torto. O lado vem do eixo que o cursor puxou mais (40
/// contra 16), então a quina continua sob o dedo — que é o que separa esta lei de um "quadre pelo menor",
/// onde a caixa foge da mão.
///
/// **Mutação que sangra:** apagar o ramo do `mods.shift` — os raios voltam a 20 × 8, razão 2,5.
#[test]
fn shift_draws_a_perfect_circle() {
    let (_, hx, hy) = draw(ELLIPSE, [16.0, 16.0], [56.0, 32.0], true, false);
    assert!(
        near(hx, hy),
        "com Shift a razão tem de ser 1: raios ({hx}, {hy})"
    );
    assert!(near(hx, 20.0), "e o lado é o eixo MAIOR (40/2): {hx}");
}

/// **Ctrl ancora no CENTRO** — o ponto onde a mão pousou vira o centro da elipse, e os dois lados crescem
/// juntos. É literalmente a frase do Enio (*"escalonar a partir do centro"*).
///
/// **Mutação que sangra:** ignorar o `mods.ctrl` — o centro cai no meio do arrasto, a 8,9 px da âncora.
#[test]
fn ctrl_draws_from_the_centre() {
    let a = [32.0, 32.0];
    let (c, hx, hy) = draw(ELLIPSE, a, [48.0, 40.0], false, true);
    let moved = ((c[0] - a[0]).powi(2) + (c[1] - a[1]).powi(2)).sqrt();
    assert!(
        moved < 0.05,
        "com Ctrl a âncora É o centro — saiu {moved:.3} px ({c:?})"
    );
    assert!(
        near(hx, 16.0) && near(hy, 8.0),
        "e o arrasto vira o raio inteiro, não a metade: ({hx}, {hy})"
    );
}

/// **E as duas juntas compõem:** o Shift quadra o delta, o Ctrl espelha o delta JÁ quadrado ⇒ círculo
/// perfeito centrado onde a mão pousou. A ordem é a lei; invertê-la quadraria um delta já espelhado e daria
/// o mesmo número aqui — por isso o gate mede as DUAS propriedades, e não só a razão.
///
/// **Mutação que sangra:** qualquer uma das duas isolada (sem Shift a razão vai a 2,0; sem Ctrl o centro
/// anda 11,3 px).
#[test]
fn shift_and_ctrl_draw_a_circle_from_the_centre() {
    let a = [32.0, 32.0];
    let (c, hx, hy) = draw(ELLIPSE, a, [48.0, 40.0], true, true);
    let moved = ((c[0] - a[0]).powi(2) + (c[1] - a[1]).powi(2)).sqrt();
    assert!(moved < 0.05, "o centro é a âncora — saiu {moved:.3} px");
    assert!(near(hx, hy) && near(hx, 16.0), "e é redondo: ({hx}, {hy})");
}

/// **O RETÂNGULO passa pela mesma porta.** Ele é um `Polygon` de 4 lados cujos raios carregam um fator
/// `√2` (os vértices caem nas quinas da caixa desenhada), então o oráculo não é o número — é a RAZÃO, que
/// tem de ser 1 tanto no quadrado quanto no círculo.
///
/// **Mutação que sangra:** resolver os cantos só no `selection_move` e deixar o `selection_up` com os
/// crus — o preview sai quadrado e a forma COMMITADA sai retangular, que é a falha de duas-portas na sua
/// forma mais cruel (o artista vê uma coisa e recebe outra).
#[test]
fn shift_squares_the_rectangle_too() {
    let (_, hx, hy) = draw(RECT, [16.0, 16.0], [56.0, 32.0], true, false);
    assert!(
        near(hx, hy),
        "o retângulo com Shift é um QUADRADO: ({hx}, {hy})"
    );
}

/// **A caixa cresce para onde a mão foi.** Um arrasto para CIMA e para a ESQUERDA com Shift tem de dar um
/// quadrado acima e à esquerda da âncora — não um espelhado para baixo e para a direita.
///
/// **Mutação que sangra:** trocar o `copysign` por um `s` nu nos dois eixos — o centro medido vai para
/// **(84,84)** em vez de (44,44), ou seja a caixa salta para o lado OPOSTO ao do cursor.
#[test]
fn the_squared_box_follows_the_hand() {
    let (c, _, _) = draw(ELLIPSE, [64.0, 64.0], [24.0, 48.0], true, false);
    assert!(
        c[0] < 64.0 && c[1] < 64.0,
        "arrastando para cima e para a esquerda o centro tem de ficar ANTES da âncora: {c:?}"
    );
    assert!(
        near(c[0], 44.0) && near(c[1], 44.0),
        "quadrado de lado 40 a partir de (64,64) para trás ⇒ centro (44,44): {c:?}"
    );
}
