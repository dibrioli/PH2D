//! **Arch-gate: a alça do texto em caminho é DESENHADA e ARRASTADA pela `render_loop`.**
//!
//! A alça é uma bolinha pintada no shell. O gate de unidade (`vec_text_ride_tests`) prova o
//! MOTOR — onde a alça está, como um arrasto a move — mas não prova que o shell a **desenha** nem
//! que costura o press/drag no fluxo do ponteiro. Mutar o desenho ou esquecer a costura deixa
//! toda a workspace verde, porque nenhum teste de unidade alcança a `render_loop` (ela precisa de
//! janela e GPU).
//!
//! É o mesmo recurso que a `line/Painter` usou para o anel do pincel
//! (`the_brush_ring_wears_the_live_dab_rotor`) e que esta linha usou para o vínculo
//! (`every_text_on_path_id_is_consumed_by_the_render_loop`): quando o único consumidor é o
//! desenho do shell, a prova é sobre o FONTE.
//!
//! ⚠️ **Controle positivo:** o gate exige encontrar CADA costura, e nomeia qual faltou — um gate
//! que deixa de encontrar por uma mudança de forma passa a guardar nada, e um gate que não vê
//! nada passa sempre.

use std::fs;

fn shell(path: &str) -> String {
    fs::read_to_string(format!("{}/src/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("ler {path}: {e}"))
}

/// O DESENHO da alça está costurado no overlay, e gateado pela política de modo (Node-only).
#[test]
fn the_render_loop_draws_the_handle_gated_on_node_mode() {
    let rl = shell("render_loop/mod.rs");
    // Desenha pelo renderer dedicado…
    assert!(
        rl.contains("draw_text_handle"),
        "a `render_loop` não chama `draw_text_handle` — a alça existe no motor e não na tela"
    );
    // …atrás do flag de modo do `VecOverlayPlan` (Node-only, testado em `vec_overlay`)…
    assert!(
        rl.contains("overlay.textpath_handle"),
        "o desenho da alça não é gateado por `overlay.textpath_handle` — apareceria fora do Node"
    );
    // …e o ponto vem da porta única, não de uma re-derivação que divergiria do hit-test.
    assert!(
        rl.contains("vec_text_ride::handle::world"),
        "o desenho não lê o ponto da alça de `handle::world` — uma 2ª derivação divergiria do \
         hit-test, e a alça seria pintada num sítio e agarrada noutro"
    );
}

/// O GESTO da alça está costurado no ponteiro: press (arma), move (arrasta), release (limpa).
#[test]
fn the_render_loop_wires_the_handle_gesture() {
    let disp = shell("input_dispatch.rs");
    for (needle, what) in [
        (
            "vec_text_ride::handle::press",
            "o PRESS (armar) não está no dispatch do ponteiro",
        ),
        (
            "vec_textpath_handle_move",
            "o MOVE (arrastar) não está no dispatch do ponteiro",
        ),
        (
            "std::mem::take(&mut self.vec_textpath_handle_drag)",
            "o RELEASE não limpa o arrasto — a alça ficaria colada ao cursor após soltar",
        ),
    ] {
        assert!(disp.contains(needle), "{what}");
    }
    // ⚠️ E a ORDEM importa: o press da alça vem ANTES do do envelope e do pen. A geometria do
    // texto vinculado é COZIDA — deixar o pen agarrá-la primeiro dá um ponto que anda e volta.
    let handle_at = disp
        .find("vec_text_ride::handle::press")
        .expect("press da alça");
    // A CHAMADA (`self.vec_pen.on_press_node`), não a menção do comentário logo acima dela.
    let pen_at = disp
        .find("self.vec_pen.on_press_node")
        .expect("press do pen (on_press_node)");
    assert!(
        handle_at < pen_at,
        "o press da alça tem de ser hit-testado ANTES do pen (a geometria cozida do texto \
         reverteria uma âncora agarrada pelo pen)"
    );
}
