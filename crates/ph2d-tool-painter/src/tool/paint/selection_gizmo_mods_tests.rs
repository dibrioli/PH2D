//! Gates das **TECLAS ACESSÓRIAS do gizmo de seleção** (Enio 2026-08-07: *"as seleções ainda não têm
//! aquele sistema de teclas acessórias (shift e ctrl) para escalonar a partir do centro e manter as
//! proporções"*).
//!
//! ⚠️ **A lei não é nova, e é isso que decide o desenho:** o gizmo de sprite já a declara em
//! [`ph2d_editor_core::GizmoModifiers`] — *Shift trava a razão de aspecto numa quina; Ctrl/Cmd troca a
//! âncora do escalonamento para o CENTRO, e o **default é a quina oposta***. O gizmo de seleção escalava
//! SEMPRE pelo centro, então a tecla que o Enio pediu para escalar pelo centro não teria o que ligar: o
//! default é que estava do lado errado da lei.

use super::*;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::CanvasPaintTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma seleção elíptica centrada em (32,32) com raio 16, em modo de edição — o gizmo na tela.
fn ellipse_selection() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_tool_mode("selection");
    t.set_selection_mode(3); // Ellipse
    t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([48.0, 48.0], PointerPhase::Up));
    t.toggle_selection_edit();
    t
}

/// `(centro, hx, hy)` lidos do PRÓPRIO gizmo — o oráculo é o que o artista vê e agarra, não um campo
/// interno da forma.
fn frame_of(t: &PainterTool) -> ([f32; 2], f32, f32) {
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

fn drag_handle(t: &mut PainterTool, handle: usize, to: [f32; 2]) {
    let from = t.selection_gizmos()[0].scale_handles[handle];
    t.on_canvas_pointer(cp(from, PointerPhase::Down));
    t.on_canvas_pointer(cp(to, PointerPhase::Move));
    t.on_canvas_pointer(cp(to, PointerPhase::Up));
}

/// **Sem tecla, a quina OPOSTA fica onde está.** O default de todo editor, e a única coisa que dá sentido
/// a uma tecla "escale pelo centro": um gizmo que já escala pelo centro não tem para onde a tecla o levar.
///
/// **Mutação que sangra:** devolver a âncora ao centro (o `mods.ctrl` sempre verdadeiro no
/// `apply_gizmo_drag`) — a quina oposta passa a fugir do lugar.
#[test]
fn a_scale_drag_pins_the_opposite_corner() {
    let mut t = ellipse_selection();
    let before = t.selection_gizmos()[0].scale_handles[3]; // BL, a quina oposta à TR
    drag_handle(&mut t, 1, [60.0, 60.0]); // agarra a quina e puxa para FORA nos dois eixos
    let after = t.selection_gizmos()[0].scale_handles[3];
    let moved = ((after[0] - before[0]).powi(2) + (after[1] - before[1]).powi(2)).sqrt();
    assert!(
        moved < 0.01,
        "a quina oposta andou {moved:.3} px — ela é a ÂNCORA e tem de ficar onde está \
         ({before:?} -> {after:?})"
    );
    let (_, hx, hy) = frame_of(&t);
    assert!(hx > 16.0 && hy > 16.0, "e a seleção cresceu ({hx}, {hy})");
}

/// **Com Ctrl, a âncora é o CENTRO** — a metade que o Enio pediu, e a que o gizmo de sprite chama de
/// *center-anchor*: os dois lados crescem juntos e o centro não sai do lugar.
///
/// **Mutação que sangra:** ignorar o `mods.ctrl` na escolha da âncora — o centro passa a andar.
#[test]
fn ctrl_scales_from_the_centre() {
    let mut t = ellipse_selection();
    let (c0, hx0, _) = frame_of(&t);
    t.set_gizmo_modifiers(false, true, false);
    drag_handle(&mut t, 1, [56.0, 8.0]);
    let (c1, hx1, _) = frame_of(&t);
    let moved = ((c1[0] - c0[0]).powi(2) + (c1[1] - c0[1]).powi(2)).sqrt();
    assert!(
        moved < 0.01,
        "com Ctrl o CENTRO é a âncora e não pode andar — andou {moved:.3} px ({c0:?} -> {c1:?})"
    );
    assert!(hx1 > hx0, "e a seleção cresceu ({hx0} -> {hx1})");
}

/// **Com Shift, a razão de aspecto sobrevive à quina.** A fixture puxa MUITO num eixo e quase nada no
/// outro, que é onde uma escala livre deforma: sem a trava a razão sai de 1,0 e vai longe.
///
/// **Mutação que sangra:** apagar o ramo do `mods.shift` — a razão medida sai de 1,000 para **11,000**
/// (a fixture puxa 22 num eixo e 2 no outro).
#[test]
fn shift_keeps_the_proportions_on_a_corner() {
    let mut t = ellipse_selection();
    let (_, hx0, hy0) = frame_of(&t);
    assert!(
        (hx0 - hy0).abs() < 0.01,
        "fixture: a elipse nasce redonda ({hx0}, {hy0})"
    );
    t.set_gizmo_modifiers(true, false, false);
    drag_handle(&mut t, 1, [60.0, 12.0]); // muito em x, pouco em y
    let (_, hx1, hy1) = frame_of(&t);
    let ratio = hx1 / hy1;
    assert!(
        (ratio - 1.0).abs() < 0.02,
        "Shift tem de manter a proporção: a razão saiu {ratio:.3} ({hx1}, {hy1})"
    );
    assert!(hx1 > hx0, "e ainda assim escalou ({hx0} -> {hx1})");
}

/// **E Shift numa ARESTA não muda nada.** Uma alça de aresta segura UM eixo; travar a razão ali faria a
/// tecla mexer no eixo que a mão não está segurando — que é a definição de um controle que surpreende.
///
/// **Mutação que sangra:** tirar o `handle <= 3` da guarda do Shift — a alça da direita passa a mexer na
/// altura também, e os dois arrastos deixam de ser iguais ao bit.
#[test]
fn shift_is_inert_on_an_edge_handle() {
    let plain = {
        let mut t = ellipse_selection();
        drag_handle(&mut t, 4, [56.0, 32.0]);
        frame_of(&t)
    };
    let shifted = {
        let mut t = ellipse_selection();
        t.set_gizmo_modifiers(true, false, false);
        drag_handle(&mut t, 4, [56.0, 32.0]);
        frame_of(&t)
    };
    assert_eq!(
        (plain.1.to_bits(), plain.2.to_bits()),
        (shifted.1.to_bits(), shifted.2.to_bits()),
        "numa aresta o Shift tem de ser inerte: {plain:?} contra {shifted:?}"
    );
}
