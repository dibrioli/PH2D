//! Translate winit's [`KeyCode`] into the editor's canonical
//! `KEY_*` constants.
//!
//! Extracted from [`main`] (Track B2). Pure free fn — the editor
//! pipeline doesn't know about winit; this is the one + only place
//! the shell bridges between them.

use winit::keyboard::KeyCode;

/// Map a winit [`KeyCode`] into the editor's canonical KEY_*
/// constants (the values `dispatch_key` matches against). Returns
/// `None` for keys the editor pipeline doesn't currently consume.
pub fn winit_to_editor_keycode(code: KeyCode) -> Option<u32> {
    use ph2d_editor_core::interaction::{
        KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_DELETE,
        KEY_ENTER, KEY_ESCAPE, KEY_F2, KEY_KEY_A, KEY_KEY_C, KEY_KEY_D, KEY_KEY_F, KEY_KEY_G,
        KEY_KEY_H, KEY_KEY_I, KEY_KEY_K, KEY_KEY_L, KEY_KEY_P, KEY_KEY_V, KEY_KEY_X, KEY_SPACE,
        KEY_TAB,
    };
    Some(match code {
        KeyCode::Tab => KEY_TAB,
        KeyCode::Enter | KeyCode::NumpadEnter => KEY_ENTER,
        KeyCode::Space => KEY_SPACE,
        KeyCode::Escape => KEY_ESCAPE,
        KeyCode::Backspace => KEY_BACKSPACE,
        KeyCode::Delete => KEY_DELETE,
        // F2 — rename (doc 61). The graph's, and the one key the verb has everywhere else.
        KeyCode::F2 => KEY_F2,
        KeyCode::ArrowUp => KEY_ARROW_UP,
        KeyCode::ArrowDown => KEY_ARROW_DOWN,
        KeyCode::ArrowLeft => KEY_ARROW_LEFT,
        KeyCode::ArrowRight => KEY_ARROW_RIGHT,
        KeyCode::KeyA => KEY_KEY_A,
        KeyCode::KeyC => KEY_KEY_C,
        // I — Select Inverse in the graph (Ctrl+I); like the letters below, consumed only
        // while a graph surface has focus, otherwise it falls through unmapped.
        KeyCode::KeyI => KEY_KEY_I,
        // Motion Nodes M0.T3 — graph-editor shortcut letters (D duplicate,
        // F fit, K knife, P probe). Consumed only while a graph surface has
        // focus; otherwise they fall through unmapped as before.
        KeyCode::KeyD => KEY_KEY_D,
        KeyCode::KeyF => KEY_KEY_F,
        // G — Group / Ungroup a subgraph (doc 57), when the cursor is over the graph.
        KeyCode::KeyG => KEY_KEY_G,
        // H — Bypass / mute the selected node(s), when the cursor is over the graph.
        KeyCode::KeyH => KEY_KEY_H,
        KeyCode::KeyK => KEY_KEY_K,
        // L — Select Linked, when the cursor is over the graph (doc 63.5).
        KeyCode::KeyL => KEY_KEY_L,
        KeyCode::KeyP => KEY_KEY_P,
        KeyCode::KeyV => KEY_KEY_V,
        KeyCode::KeyX => KEY_KEY_X,
        _ => return None,
    })
}

/// **O NORMALIZADOR TOTAL — o que o Input Map usa**: mora na folha `ph2d-input-winit` (InputMap W1).
pub use ph2d_input_winit::winit_to_input_keycode;

/// A tecla LÓGICA vista pela composição de uma tecla morta ([`ph2d_editor_core::ime`]).
pub fn compose_key(k: &winit::keyboard::Key) -> ph2d_editor_core::ime::ComposeKey {
    use ph2d_editor_core::ime::ComposeKey;
    use winit::keyboard::{Key, NamedKey as N};
    match k {
        Key::Dead(Some(c)) => ComposeKey::Dead(*c),
        Key::Named(N::Escape) => ComposeKey::Escape,
        Key::Dead(None)
        | Key::Named(
            N::Shift | N::Control | N::Alt | N::AltGraph | N::Super | N::Meta | N::Hyper | N::Fn,
        )
        | Key::Named(N::CapsLock | N::NumLock | N::Symbol) => ComposeKey::Modifier,
        _ => ComposeKey::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::{GraphKey, graph_key_for};

    /// `´` + Shift + `A` = `Á`: o Shift não pode desfazer a tecla morta (sequência medida, T6).
    #[test]
    fn a_dead_key_survives_shift_and_dies_on_the_next_key() {
        use ph2d_editor_core::ime::ComposeKey as C;
        use winit::keyboard::{Key, NamedKey as N};
        assert_eq!(compose_key(&Key::Dead(Some('´'))), C::Dead('´'));
        assert_eq!(compose_key(&Key::Named(N::Shift)), C::Modifier);
        assert_eq!(compose_key(&Key::Named(N::Escape)), C::Escape);
        assert_eq!(compose_key(&Key::Character("á".into())), C::Other);
    }

    /// **Every verb the graph owns must survive the SHELL's normalizer.**
    ///
    /// This is the gate that would have caught the `Ctrl+G` bug before the smoke (Enio,
    /// 2026-07-13: it toggled the scene grid instead of grouping). The graph's keymap
    /// lives in editor-core; the shell's key router is what actually feeds it while the
    /// cursor is over the panel — and it can only feed a key it can NAME. `KeyG` was
    /// missing from this table, so the chord fell straight through to the global `G`.
    ///
    /// A verb the graph can express but the shell cannot spell is a verb that silently
    /// does something else.
    /// ⭐⭐ **OS DOIS NORMALIZADORES NUNCA DISCORDAM.**
    ///
    /// ⚠️ Há dois de propósito (o do editor devolve `None` para as teclas que ele deixa CAIR; o do
    /// Input Map mapeia tudo) — e dois leitores do mesmo vocabulário são, sem gate, duas verdades
    /// que divergem na primeira tecla que alguém acrescente a um só. Este gate torna a divergência
    /// impossível de aterrar em silêncio: **onde o do editor diz `Some(v)`, o do mapa diz o mesmo
    /// `v`**.
    ///
    /// ⚠️ E o CONTROLE POSITIVO vem junto: se a varredura não encontrar teclas mapeadas nenhumas,
    /// ela passaria a afirmar sobre um conjunto vazio.
    #[test]
    fn the_two_normalizers_never_disagree() {
        let every = [
            KeyCode::Tab,
            KeyCode::Enter,
            KeyCode::NumpadEnter,
            KeyCode::Space,
            KeyCode::Escape,
            KeyCode::Backspace,
            KeyCode::Delete,
            KeyCode::F2,
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            KeyCode::KeyA,
            KeyCode::KeyC,
            KeyCode::KeyD,
            KeyCode::KeyF,
            KeyCode::KeyG,
            KeyCode::KeyH,
            KeyCode::KeyI,
            KeyCode::KeyK,
            KeyCode::KeyL,
            KeyCode::KeyP,
            KeyCode::KeyV,
            KeyCode::KeyX,
        ];
        let mut checked = 0;
        for code in every {
            let Some(editor) = winit_to_editor_keycode(code) else {
                continue;
            };
            checked += 1;
            // ⭐ A única divergência DE PROPÓSITO (§13.1): o mapa dá ao Numpad Enter código próprio, e
            // ele continua a CONFIRMAR como o Enter do editor.
            if code == KeyCode::NumpadEnter {
                let k = winit_to_input_keycode(code).map(ph2d_input::Key);
                assert!(k.is_some_and(|k| k.is_enter_like() && k.0 != editor));
                continue;
            }
            assert_eq!(
                winit_to_input_keycode(code),
                Some(editor),
                "os dois normalizadores discordam sobre {code:?} -- uma accao ligada a essa tecla \
                 nunca dispararia, ou dispararia sobre a tecla errada"
            );
        }
        assert!(
            checked >= 20,
            "a varredura so' encontrou {checked} teclas mapeadas: este gate passou a afirmar sobre \
             quase nada"
        );
    }

    /// ⛔ **O do Input Map alcança o que o do editor deixa cair** — senão não haveria razão para
    /// existirem dois, e as teclas mais óbvias de um jogo (WASD) seriam inligáveis.
    #[test]
    fn the_input_normalizer_reaches_what_the_editor_drops() {
        for code in [
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyZ,
            KeyCode::KeyQ,
            KeyCode::ShiftLeft,
        ] {
            assert!(
                winit_to_editor_keycode(code).is_none(),
                "{code:?} passou a ser do editor -- este gate mede a diferenca entre os dois e a \
                 diferenca mudou"
            );
            assert!(
                winit_to_input_keycode(code).is_some(),
                "{code:?} e' inligavel no Input Map -- e e' uma das teclas mais obvias de um jogo"
            );
        }
    }

    #[test]
    fn every_graph_verb_is_reachable_through_the_shells_normalizer() {
        let cases = [
            (KeyCode::KeyG, true, false, GraphKey::Group),
            (KeyCode::KeyG, true, true, GraphKey::Ungroup),
            (KeyCode::KeyD, true, false, GraphKey::Duplicate),
            (KeyCode::KeyF, false, false, GraphKey::Fit),
            (KeyCode::KeyA, false, false, GraphKey::Add),
            (KeyCode::KeyA, true, false, GraphKey::SelectAll),
            (KeyCode::KeyC, true, false, GraphKey::Copy),
            (KeyCode::KeyV, true, false, GraphKey::Paste),
            (KeyCode::KeyX, true, false, GraphKey::Cut),
            (KeyCode::KeyL, true, false, GraphKey::SelectLinked),
            (KeyCode::KeyI, true, false, GraphKey::SelectInvert),
            (KeyCode::KeyH, false, false, GraphKey::Bypass),
            (KeyCode::KeyK, false, false, GraphKey::Knife),
            (KeyCode::KeyP, false, false, GraphKey::Probe),
            (KeyCode::Escape, false, false, GraphKey::Escape),
            (KeyCode::Delete, false, false, GraphKey::Delete),
            (KeyCode::Space, false, false, GraphKey::TogglePlay),
            (KeyCode::F2, false, false, GraphKey::Rename),
        ];
        for (code, cmd, alt, want) in cases {
            let kc = winit_to_editor_keycode(code)
                .unwrap_or_else(|| panic!("the shell cannot even NAME {code:?}"));
            assert_eq!(
                graph_key_for(kc, cmd, alt),
                Some(want),
                "{code:?} (cmd={cmd}, alt={alt}) must reach the graph as {want:?}"
            );
        }
    }

    /// A plain `G` is NOT a graph verb — it is the scene's grid toggle, and the graph
    /// must not eat a letter it does not own (that would be the same bug, mirrored).
    #[test]
    fn a_bare_g_is_not_a_graph_verb() {
        let kc = winit_to_editor_keycode(KeyCode::KeyG).unwrap();
        assert_eq!(graph_key_for(kc, false, false), None);
    }
}
