//! **Arch-gate: as alças de PARÂMETRO da forma (F9) são DESENHADAS e ARRASTADAS pela shell.**
//!
//! Irmão das alças do texto e do padrão. A lei e o mundo têm gates de comportamento no motor
//! (`shape_handles_tests`, `shape_handles_live_tests`, a cena 15); o que nenhum deles alcança é a
//! COSTURA da shell (precisa de janela): que ela desenha, e que o press, o arrasto, o soltar e o
//! `Esc` estão no fluxo do ponteiro e do teclado, na ordem certa.

/// O desenho: pela porta do motor, com a política de modo (`overlay.shape_handles`, só no modo Forma
/// desde a F10) como argumento — F11: o portão passou para dentro do `draw` (o `shown` dele) —,
/// FORA do bloco `if overlay.edit`, depois das alças do conector.
#[test]
fn the_render_loop_draws_the_shape_handles_by_the_mode_policy() {
    let rl = crate::frame_text::render_frame();
    let draw_at = rl.find("ph2d_app_vec::shape_handles_live::draw(").expect(
        "o desenho não passa pela porta do motor — uma 2ª derivação divergiria do hit-test",
    );
    let args = &rl[draw_at..rl[draw_at..].find(");").map_or(rl.len(), |e| draw_at + e)];
    assert!(
        args.contains("overlay.shape_handles"),
        "o desenho das alças da forma não recebe a política de modo: {args}"
    );
    let conn_at = rl
        .find("draw_connector_handles")
        .expect("desenho das alças do conector");
    assert!(
        draw_at > conn_at,
        "o desenho das alças da forma está ANTES do conector — dentro do `overlay.edit`?"
    );
}

/// O gesto (F10): o press no cluster de alças sobre o canvas, FORA do portão de modo do Select
/// (as alças da forma são do modo Forma — a lei do modo e do latch é do motor) e ANTES do ramo
/// vetorial (o Shift-arm e o desenho de uma forma nova) e do gizmo; o arrasto no mover; o soltar e
/// o `Esc` libertam-no.
#[test]
fn the_pointer_and_the_keyboard_wire_the_shape_handle_gesture() {
    let disp = crate::input_text::dispatch();
    let press = disp
        .find("&& (self.vec_shape_handle_down(w)")
        .expect("o PRESS das alças da forma (cluster das alças sobre o canvas)");
    let head = disp[..press]
        .rfind("if self.vector_tool_active()")
        .expect("o if do cluster");
    assert!(
        !disp[head..press].contains("DrawMode::Select"),
        "o press das alças da forma voltou para trás do portão do Select"
    );
    let generic = disp
        .find("no modo **Select** a ferramenta não captura")
        .expect("o guard ADR-0112 do ramo vetorial");
    let draw = disp
        .find("self.vec.shape.on_press(")
        .expect("o press do desenho de uma forma nova");
    assert!(
        press < generic && press < draw,
        "o press das alças da forma tem de vir ANTES do ramo vetorial — senão o clique numa alça \
         desenharia uma forma nova (ou abriria o Shift-arm) em vez de a agarrar"
    );
    assert!(
        disp.contains("h.gizmo.drag.is_some())"),
        "o ramo vetorial não cede a um arrasto da caixa — o Up dele engoliria o fim do arrasto"
    );
    for (needle, what) in [
        (
            "|| self.vec_shape_handle_move(",
            "o ARRASTO não está no mover do ponteiro",
        ),
        (
            "self.vec.shape_handles.release()",
            "o SOLTAR não liberta o arrasto — a alça ficaria colada ao cursor",
        ),
    ] {
        assert!(disp.contains(needle), "{what}");
    }
    let esc = crate::input_text::file("input_dispatch/keyboard_escapes.rs");
    assert!(
        esc.contains("ph2d_app_vec::shape_handles_live::on::escape("),
        "o Esc a meio do arrasto não repõe os valores do press"
    );
}
