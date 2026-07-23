//! **Arch-gate: as duas alças do Pattern on Path são DESENHADAS e ARRASTADAS pela `render_loop`.**
//!
//! Irmão do `the_textpath_handle_is_drawn_and_dragged`. As alças são fichas pintadas no shell (W4).
//! O gate de unidade (`pattern_live_tests`) prova o MOTOR — onde as fichas estão, como um arrasto
//! as move — mas não que o shell as **desenha** nem que costura o press/drag no fluxo do ponteiro.
//! Mutar o desenho ou esquecer a costura deixa toda a workspace verde, porque nenhum teste de
//! unidade alcança a `render_loop` (ela precisa de janela e GPU). Quando o único consumidor é o
//! desenho do shell, a prova é sobre o FONTE.

use std::fs;

fn shell(path: &str) -> String {
    fs::read_to_string(format!("{}/src/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("ler {path}: {e}"))
}

/// O DESENHO das fichas está costurado no overlay, gateado pela política de modo (Select-only), e
/// FORA do bloco `if overlay.edit` (falso no Select) — provado pela ordem contra as alças do
/// conector, a mesma técnica do gate do texto.
#[test]
fn the_render_loop_draws_the_handles_gated_on_select_mode() {
    let rl = shell("render_loop/mod.rs");
    assert!(
        rl.contains("overlay.patternpath_handles"),
        "o desenho das alças não é gateado por `overlay.patternpath_handles` — apareceriam fora do Select"
    );
    assert!(
        rl.contains("pattern_live::handle::world"),
        "o desenho não lê os pontos das fichas de `handle::world` — uma 2ª derivação divergiria do hit-test"
    );
    // As fichas são desenhadas DEPOIS do `draw_connector_handles` (a região comprovadamente fora
    // do `overlay.edit`, que é FALSO no Select) — se alguém as puser no bloco de edição, elas
    // precederiam o conector e este gate fica VERMELHO.
    let draw_at = rl
        .find("overlay.patternpath_handles")
        .expect("desenho das alças do pattern");
    let conn_at = rl
        .find("draw_connector_handles")
        .expect("desenho das alças do conector");
    assert!(
        draw_at > conn_at,
        "o desenho das alças do pattern está ANTES do `draw_connector_handles` — provavelmente \
         dentro do `if overlay.edit`, onde nunca desenha no Select"
    );
}

/// O GESTO das alças está costurado no ponteiro: press (arma qual ficha), move (arrasta), release
/// (limpa). O press é irmão do `vec_textpath_handle_down` e vem ANTES do picking/gizmo genérico.
#[test]
fn the_render_loop_wires_the_handle_gesture() {
    let disp = shell("input_dispatch.rs");
    for (needle, what) in [
        (
            "vec_patternpath_handle_down",
            "o PRESS (armar) não está no dispatch do ponteiro (modo Select)",
        ),
        (
            "vec_patternpath_handle_move",
            "o MOVE (arrastar) não está no dispatch do ponteiro",
        ),
        (
            "self.vec_patternpath_handle = None",
            "o RELEASE não limpa o arrasto — a ficha ficaria colada ao cursor após soltar",
        ),
    ] {
        assert!(disp.contains(needle), "{what}");
    }
    // A ORDEM importa: no Select a tool não captura o canvas, então o press da ficha tem de ser
    // hit-testado ANTES do picking/gizmo genérico (o guard ADR-0112), senão o clique selecionaria
    // a forma atrás dela. A mesma razão da alça do conector e da do texto.
    let handle_at = disp
        .find("self.vec_patternpath_handle_down(w)")
        .expect("press das fichas (Select)");
    let generic_dispatch_at = disp
        .find("no modo **Select** a ferramenta não captura")
        .expect("o guard ADR-0112 do dispatch genérico");
    assert!(
        handle_at < generic_dispatch_at,
        "o press das fichas (Select) tem de ser hit-testado ANTES do picking/gizmo genérico"
    );
    // E é irmão do press do texto — ambos no mesmo cluster de early-return do Select.
    let textpath_at = disp
        .find("self.vec_textpath_handle_down(w)")
        .expect("press da alça do texto");
    assert!(
        (handle_at as isize - textpath_at as isize).unsigned_abs() < 1200,
        "as fichas do pattern e a alça do texto deviam estar no MESMO cluster de Select"
    );
}
