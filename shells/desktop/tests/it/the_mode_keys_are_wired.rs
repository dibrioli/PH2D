//! **Arch-gate: as teclas do MODO** (spec/06 §3.2; dono, 03/10 e 05/10) — `Tab` = Object ↔ o modo,
//! `Ctrl+Tab` = a lista (as duas por `mode_drive::mode_key`, com gate na fundação) e `Ctrl+Space` =
//! o zen. O despacho do teclado só se alcança com a `App`; o que se afirma é o FONTE do território
//! do input inteiro.
//!
//! ⛔ O defeito que ele apanhou (05/10): no modo Node do vetor um braço `KeyCode::Tab` percorria os
//! nós e CONSUMIA a tecla — em Edit com o Node, o `Tab` não saía do modo. ⇒ o `Tab` tem UM dono.

/// ⚠️ A ÚNICA excepção é a tradução do QUADRO (`keyboard_board.rs`, MiroClone): com uma aba de quadro
/// activa a cena está escondida e o teclado é do quadro (o `Tab` lá é a nota seguinte, como no Miro);
/// sem quadro activo essa porta devolve `false` antes de ler a tecla.
///
/// Mutação que tem de sangrar: um segundo braço `KeyCode::Tab` em qualquer outro ficheiro do input;
/// o quadro tomar o teclado sem quadro activo; ou o braço do modo deixar de chamar o `mode_key`.
#[test]
fn only_the_mode_owns_the_tab() {
    const BOARD: &str = "input_dispatch/keyboard_board.rs";
    let all = crate::input_text::territory();
    let mut arms: Vec<(&str, usize)> = Vec::new();
    let mut board = "";
    for chunk in all.split("// [[ficheiro ").skip(1) {
        let (name, body) = chunk.split_once("]]\n").expect("o marcador do ficheiro");
        if name == BOARD {
            board = body;
        }
        let n = body.matches("KeyCode::Tab").count();
        if n > 0 {
            arms.push((name, n));
        }
    }
    let scene: Vec<_> = arms.iter().filter(|(f, _)| *f != BOARD).collect();
    assert!(
        scene.len() == 1 && scene[0].1 == 1,
        "o `Tab` tem mais de um dono no input (ou nenhum): {arms:?}"
    );
    assert!(
        board.contains("if hero.documents.active().is_none() {\n            return false;"),
        "o quadro toma o teclado (e o `Tab`) sem um quadro activo"
    );
    let handlers = crate::input_text::handlers();
    let arm = handlers
        .find("KeyCode::Tab =>")
        .expect("o braco do `Tab` nao esta no input_handlers");
    let body = &handlers[arm..arm + 240];
    assert!(
        body.contains("mode_drive::mode_key(hero, cmd_chord)"),
        "o `Tab` nao chega ao modo (o `Ctrl+Tab` e a lista pelo mesmo `cmd_chord`)"
    );
}

/// Mutação que tem de sangrar: o `Ctrl+Space` deixar de alternar o zen.
#[test]
fn ctrl_space_toggles_the_zen() {
    let handlers = crate::input_text::handlers();
    let arm = handlers
        .find("KeyCode::Space if cmd_chord =>")
        .expect("o braco do `Ctrl+Space` sumiu");
    assert!(
        handlers[arm..arm + 120].contains("gfx.zen.try_toggle()"),
        "o `Ctrl+Space` nao alterna o zen"
    );
}
