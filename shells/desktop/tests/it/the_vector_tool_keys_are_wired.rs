//! ⭐⭐ **As teclas do vetor chegam à tabela do padrão Illustrator** (D10, 07/10).
//!
//! A TABELA é do painel (`ph2d_panel_vector::tool_keys`, com gates próprios: toda ferramenta com a
//! tecla dela, nenhuma tecla com dois donos) e a APLICAÇÃO da crate do vetor
//! (`ph2d_app_vec::tool_keys`: um clique no mesmo id, e a ferramenta temporária). Este gate prende a
//! COSTURA na shell: a tecla física só vira acorde, e o acorde vai à crate — ⛔ nenhuma letra do vetor
//! decidida aqui, e as booleanas já não moram nas letras soltas.
//!
//! Mutações que têm de sangrar: o ramo deixar de chamar o `tool_keys::key` ou o `tool_keys::chord`;
//! perder a cedência à timeline (o `K` voltaria a roubar a chave); um `KeyCode::KeyU` de volta.

#[test]
fn the_vector_letters_go_to_the_illustrator_table() {
    let src = crate::input_text::file("input_dispatch/keyboard_cadeia.rs");
    let at = src
        .find("ph2d_app_vec::tool_keys::key(")
        .expect("o ramo do vetor não chama a aplicação da tabela");
    let block = &src[src[..at].rfind("if self.vector_keys_live()").expect("a guarda do vetor")..at];
    assert!(
        block.contains("!self.cursor_over_timeline()"),
        "as letras do vetor deixaram de ceder à timeline (o `K` roubaria a chave)"
    );
    assert!(
        block.contains("ph2d_app_vec::tool_keys::chord("),
        "a tecla física não vira acorde pela porta da crate"
    );
    assert!(
        block.contains("winit_to_input_keycode"),
        "a letra não sai do normalizador da casa"
    );
    for velha in ["KeyCode::KeyU", "PathfinderOp::Union", "vec_text_toggle_mode"] {
        assert!(
            !src.contains(velha),
            "`{velha}` voltou à cadeia — a letra decide-se na tabela, e as booleanas moram em Ctrl+Alt"
        );
    }
}

/// A tecla segurada ESCUTA o soltar: o ramo não pode exigir `Pressed`, senão a ferramenta
/// temporária nunca volta (soltar a tecla não chegaria à crate).
#[test]
fn the_release_reaches_the_temporary_tool() {
    let src = crate::input_text::file("input_dispatch/keyboard_cadeia.rs");
    let at = src.find("ph2d_app_vec::tool_keys::key(").expect("o ramo");
    let start = src[..at].rfind("if self.vector_keys_live()").expect("a guarda");
    let guard = &src[start..at];
    assert!(
        !guard.contains("state == ElementState::Pressed\n"),
        "a guarda exige Pressed — o soltar da ferramenta temporária morre aqui"
    );
    assert!(
        guard.contains("let pressed = state == ElementState::Pressed;"),
        "o ramo não diz à crate se a tecla desceu ou subiu"
    );
}
