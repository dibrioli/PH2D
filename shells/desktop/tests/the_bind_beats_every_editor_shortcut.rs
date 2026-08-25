//! ⛔⛔ **ARCH-GATE do report do Enio (2026-08-24): *"Os atalhos de editor estão em conflito com o
//! Bind"*.**
//!
//! # O defeito, e por que ele passou por seis gates verdes
//!
//! A guarda da escuta foi posta como o **primeiro ramo do `dispatch_key`** — o primeiro *dentro do
//! editor-core* — e havia gate a prová-lo (`a_captured_key_never_fires_the_editors_shortcut`, com o
//! `Tab`). Ele estava certo e era **insuficiente**: o `key_input` da shell tem uma vintena de
//! `return` **antes** de chamar o `dispatch_key`, e as teclas mais óbvias de um jogo estão entre
//! elas — o `P` do menu radial, o `W` do painel de mundo, o Espaço do transporte, o peek do Flip.
//!
//! Nenhuma delas chegava ao editor-core. Carregar nelas durante o `Bind…` executava o atalho e
//! **não ligava nada**.
//!
//! ⇒ *A ordem é a feature — e ela tem de estar no topo da cadeia REAL, não no topo de um pedaço
//! dela.* Este ficheiro mede a posição.

use std::fs;

fn key_input() -> String {
    fs::read_to_string("src/input_dispatch/keyboard.rs").expect("o key_input da shell")
}

/// ⚠️ **O irmão para onde a guarda foi cortada** (teto de LOC, HR-18). Este gate re-ancorou-se no
/// dia em que o corte aconteceu — a lei é a mesma, mudou o endereço.
fn bind_capture() -> String {
    fs::read_to_string("src/input_dispatch/keyboard_bind_capture.rs").expect("o irmao da guarda")
}

/// **(1) A guarda está ACIMA de todo `return` da função.**
///
/// ⚠️ **A propriedade é a POSIÇÃO, e não a presença.** Um gate que só perguntasse *"a guarda
/// existe?"* teria ficado verde durante todo o tempo em que ela existia **no sítio errado** — que é
/// exactamente o que aconteceu.
#[test]
fn the_listening_guard_sits_above_every_early_return() {
    let src = key_input();
    let body = src
        .split_once("pub(crate) fn key_input(")
        .expect("a funcao existe")
        .1;
    // ⚠️ A agulha é a CHAMADA do irmão, e não a lei: a guarda foi cortada para
    // `keyboard_bind_capture.rs` pelo teto de LOC, e a propriedade que importa continua a ser
    // **onde ela é chamada** dentro desta função.
    let guard = body
        .find("self.capture_binding_if_listening(")
        .expect("a guarda da escuta tem de ser chamada no `key_input` da shell");
    let first_return = body.find("\n            return;").unwrap_or(body.len());
    assert!(
        guard < first_return,
        "a guarda da escuta esta' DEPOIS do primeiro `return` da funcao ({guard} vs \
         {first_return}): as teclas que a shell consome antes dela (P do radial, W do mundo, \
         Espaco do transporte) executam o atalho em vez de ligarem"
    );
}

/// **(2) A shell e o editor chamam a MESMA lei.**
///
/// ⚠️ Duas implementações da mesma pergunta divergiriam na primeira tecla que alguém tratasse só
/// num dos lados — e o sintoma seria *"esta tecla liga, aquela não"*, sem padrão visível.
#[test]
fn the_shell_and_the_editor_call_one_law() {
    assert!(
        key_input().contains("self.capture_binding_if_listening("),
        "o `key_input` tem de chamar a guarda"
    );
    assert!(
        bind_capture().contains("ph2d_editor::interaction::capture_if_listening("),
        "o irmao da guarda tem de chamar a LEI, e nao uma copia dela"
    );
    let editor =
        fs::read_to_string("../../crates/ph2d-editor-core/src/interaction/dispatch/key.rs")
            .expect("o dispatch do editor");
    assert!(
        editor.contains("capture_if_listening(store, event.keycode)"),
        "o despacho do editor tem de chamar a MESMA lei, e nao uma copia dela"
    );
    // O CONTROLE POSITIVO: a lei existe num sítio só.
    let law =
        fs::read_to_string("../../crates/ph2d-editor-core/src/interaction/state/input_map_ops.rs")
            .expect("a lei");
    assert!(
        law.contains("pub fn capture_if_listening("),
        "a lei mudou de casa -- este gate esta' a medir dois chamadores de uma porta que sumiu"
    );
}

/// **(3) A guarda usa o normalizador TOTAL.**
///
/// ⚠️ Com o do editor, as teclas mais óbvias de um jogo (`W`, `S`, `Z`, `Q`) chegariam como `None`
/// e cairiam para os atalhos — o defeito reportado, com outra causa e o mesmo sintoma.
#[test]
fn the_guard_normalises_with_the_total_map() {
    let src = bind_capture();
    let at = src
        .find("capture_if_listening(")
        .expect("a guarda chama a lei");
    let before = &src[..at];
    assert!(
        before.contains("winit_to_input_keycode"),
        "a guarda esta' a normalizar com o mapa do EDITOR: W/S/Z/Q chegariam como `None` e \
         cairiam para os atalhos"
    );
}
