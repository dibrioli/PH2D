//! **Arch-gate: as alças de PARÂMETRO da forma (F9) são DESENHADAS e ARRASTADAS pela shell.**
//!
//! Irmão das alças do texto e do padrão. A lei e o mundo têm gates de comportamento no motor
//! (`shape_handles_tests`, `shape_handles_live_tests`, a cena 15); o que nenhum deles alcança é a
//! COSTURA da shell (precisa de janela): que ela desenha, e que o press, o arrasto, o soltar e o
//! `Esc` estão no fluxo do ponteiro e do teclado, na ordem certa.

/// O desenho: gateado pela política de modo (`overlay.shape_handles`, Select-only), FORA do bloco
/// `if overlay.edit` (falso no Select) — depois das alças do conector, como as do padrão.
#[test]
fn the_render_loop_draws_the_shape_handles_in_select_mode() {
    let rl = crate::frame_text::render_frame();
    let draw_at = rl
        .find("if overlay.shape_handles {")
        .expect("o desenho das alças da forma, gateado pelo modo");
    assert!(
        rl[draw_at..].contains("ph2d_app_vec::shape_handles_live::draw("),
        "o desenho não passa pela porta do motor — uma 2ª derivação divergiria do hit-test"
    );
    let conn_at = rl
        .find("draw_connector_handles")
        .expect("desenho das alças do conector");
    assert!(
        draw_at > conn_at,
        "o desenho das alças da forma está ANTES do conector — dentro do `overlay.edit`?"
    );
}

/// O gesto: o press no cluster de alças do Select, ANTES do picking/gizmo genérico; o arrasto no
/// mover; o soltar e o `Esc` libertam-no.
#[test]
fn the_pointer_and_the_keyboard_wire_the_shape_handle_gesture() {
    let disp = crate::input_text::dispatch();
    let press = disp
        .find("|| self.vec_shape_handle_down(w))")
        .expect("o PRESS das alças da forma (cluster do Select)");
    let generic = disp
        .find("no modo **Select** a ferramenta não captura")
        .expect("o guard ADR-0112 do dispatch genérico");
    assert!(
        press < generic,
        "o press das alças da forma tem de vir ANTES do picking/gizmo — senão o clique selecionaria \
         a forma (ou a escalaria pela caixa) em vez de agarrar a alça"
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
