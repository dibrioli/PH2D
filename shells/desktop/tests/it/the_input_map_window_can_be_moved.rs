//! **ARCH-GATE do fio do ARRASTO da janela do Input Map.**
//!
//! ⚠️ **Por que é um gate de FIO e não de comportamento:** o arrasto é uma máquina de estados do
//! shell entre um `Down` e um `Up`, e os dois chegam por `winit` — que não se constrói fora do
//! winit. É a mesma parede que fez o irmão do Fill modal ter um gate desta forma.
//!
//! Três formas de o fio nascer morto, e uma afirmação para cada.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// **(1) Os TRÊS sítios existem.** Um arrasto precisa de armar, mover e largar; faltando qualquer
/// um deles, ou a janela não se mexe, ou ela **cola-se ao cursor para sempre**.
#[test]
fn the_drag_is_wired_at_all_three_points() {
    let src = read("src/input_dispatch.rs");
    for (needle, what) in [
        (
            "self.arm_input_map_drag_if_on_handle(",
            "ARMAR no Primary Down",
        ),
        ("self.input_map_drag_move(", "MOVER no CursorMoved"),
        ("self.input_map_drag_up();", "LARGAR no Primary Up"),
    ] {
        assert!(
            src.contains(needle),
            "falta o sitio de {what} no fio do arrasto -- sem ele a janela {}",
            if needle.ends_with("up();") {
                "cola-se ao cursor para sempre"
            } else {
                "nao se mexe"
            }
        );
    }
}

/// **(2) O arrasto CONSOME o movimento.**
///
/// ⚠️ Sem o early-return, arrastar a janela **também** panaria a câmara ou conduziria um gizmo por
/// baixo dela — e o artista veria o canvas a fugir enquanto arruma a janela.
#[test]
fn dragging_the_window_consumes_the_motion() {
    let src = read("src/input_dispatch.rs");
    let at = src
        .find("if self.input_map_drag_move(")
        .expect("o CursorMoved consulta o arrasto");
    let tail = &src[at..];
    let block = &tail[..tail.find('}').map_or(tail.len(), |i| i + 1)];
    assert!(
        block.contains("return"),
        "o arrasto tem de CONSUMIR o movimento -- senao arrastar a janela pana a camara por baixo. \
         Bloco:\n{block}"
    );
}

/// **(3) A janela é armada ANTES do Fill modal.**
///
/// ⚠️ Dois cartões flutuantes que reclamam o mesmo `Down` é como um deles fica impossível de
/// agarrar. A ordem é a decisão, e ela tem de estar escrita.
#[test]
fn the_input_map_claims_the_down_before_the_fill_modal() {
    let src = read("src/input_dispatch.rs");
    let ours = src
        .find("self.arm_input_map_drag_if_on_handle(")
        .expect("o nosso arma");
    let theirs = src
        .find("self.arm_fill_modal_drag_if_on_handle(")
        .expect("o do Fill arma");
    assert!(
        ours < theirs,
        "a ordem dos dois cartoes flutuantes inverteu-se: {ours} vs {theirs}"
    );
}

/// **(4) A porta do movimento recebe um DELTA.**
///
/// ⚠️ A irmã (`move_fill_modal`) também. Uma que recebesse a posição absoluta faria a janela
/// **saltar** para debaixo do cursor no primeiro pixel de arrasto, em vez de acompanhar a mão.
#[test]
fn the_move_door_takes_a_delta_not_a_position() {
    let src = read("../../crates/ph2d-editor-core/src/interaction/state/input_map_ops.rs");
    assert!(
        src.contains("pub fn move_input_map(&mut self, dx: f32, dy: f32)"),
        "a porta do movimento deixou de receber um delta -- a janela vai saltar para debaixo do \
         cursor no primeiro pixel"
    );
    assert!(
        src.contains("Some((x + dx, y + dy))"),
        "a porta recebe um delta e nao o SOMA -- ou ela nao anda, ou anda para o sitio errado"
    );
}
