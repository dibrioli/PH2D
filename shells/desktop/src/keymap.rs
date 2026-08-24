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
    use ph2d_editor::interaction::{
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

/// **O NORMALIZADOR TOTAL — o que o Input Map usa** (plano 30 W5).
///
/// ⚠️ **Por que existe um segundo, e não um alargamento do primeiro:** o
/// [`winit_to_editor_keycode`] devolve `None` de propósito para as teclas que o editor **não**
/// consome, e é esse `None` que as deixa *cair* para os consumidores de baixo — alargá-lo faria o
/// editor passar a reclamar teclas que hoje ignora, e mudaria o comportamento de todo atalho que
/// depende dessa queda.
///
/// ⇒ este mapeia **tudo o que um artista pode querer ligar**, no **MESMO espaço `u32`**. As duas
/// funções são dois leitores de um vocabulário só, e o gate
/// `the_two_normalizers_never_disagree` torna a divergência impossível de aterrar em silêncio:
/// onde o do editor diz `Some(v)`, este tem de dizer o mesmo `v`.
///
/// ⛔ **Não é um enum novo.** Letras e dígitos vão para o ASCII maiúsculo (`A` = `0x41`), que é
/// exactamente onde as constantes `KEY_KEY_*` do editor já estão — e é por isso que os dois podem
/// concordar por construção em vez de por promessa.
pub fn winit_to_input_keycode(code: KeyCode) -> Option<u32> {
    /// O primeiro código da faixa de função no espaço NSEvent que o editor já usa para as setas.
    const FN_BASE: u32 = 0xF704;
    let v = match code {
        // Letras: ASCII maiúsculo, a mesma casa das `KEY_KEY_*`.
        KeyCode::KeyA => 0x41,
        KeyCode::KeyB => 0x42,
        KeyCode::KeyC => 0x43,
        KeyCode::KeyD => 0x44,
        KeyCode::KeyE => 0x45,
        KeyCode::KeyF => 0x46,
        KeyCode::KeyG => 0x47,
        KeyCode::KeyH => 0x48,
        KeyCode::KeyI => 0x49,
        KeyCode::KeyJ => 0x4A,
        KeyCode::KeyK => 0x4B,
        KeyCode::KeyL => 0x4C,
        KeyCode::KeyM => 0x4D,
        KeyCode::KeyN => 0x4E,
        KeyCode::KeyO => 0x4F,
        KeyCode::KeyP => 0x50,
        KeyCode::KeyQ => 0x51,
        KeyCode::KeyR => 0x52,
        KeyCode::KeyS => 0x53,
        KeyCode::KeyT => 0x54,
        KeyCode::KeyU => 0x55,
        KeyCode::KeyV => 0x56,
        KeyCode::KeyW => 0x57,
        KeyCode::KeyX => 0x58,
        KeyCode::KeyY => 0x59,
        KeyCode::KeyZ => 0x5A,
        // Dígitos da fileira de cima: ASCII.
        KeyCode::Digit0 => 0x30,
        KeyCode::Digit1 => 0x31,
        KeyCode::Digit2 => 0x32,
        KeyCode::Digit3 => 0x33,
        KeyCode::Digit4 => 0x34,
        KeyCode::Digit5 => 0x35,
        KeyCode::Digit6 => 0x36,
        KeyCode::Digit7 => 0x37,
        KeyCode::Digit8 => 0x38,
        KeyCode::Digit9 => 0x39,
        // Os controlos, nos MESMOS valores que o editor já usa.
        KeyCode::Tab => 0x09,
        KeyCode::Enter | KeyCode::NumpadEnter => 0x0D,
        KeyCode::Space => 0x20,
        KeyCode::Escape => 0x1B,
        KeyCode::Backspace => 0x08,
        KeyCode::Delete => 0xF728,
        KeyCode::ArrowUp => 0xF700,
        KeyCode::ArrowDown => 0xF701,
        KeyCode::ArrowLeft => 0xF702,
        KeyCode::ArrowRight => 0xF703,
        // F1..F12, contíguas a partir da mesma faixa das setas (o `F2` do editor é `0xF705`).
        KeyCode::F1 => FN_BASE,
        KeyCode::F2 => FN_BASE + 1,
        KeyCode::F3 => FN_BASE + 2,
        KeyCode::F4 => FN_BASE + 3,
        KeyCode::F5 => FN_BASE + 4,
        KeyCode::F6 => FN_BASE + 5,
        KeyCode::F7 => FN_BASE + 6,
        KeyCode::F8 => FN_BASE + 7,
        KeyCode::F9 => FN_BASE + 8,
        KeyCode::F10 => FN_BASE + 9,
        KeyCode::F11 => FN_BASE + 10,
        KeyCode::F12 => FN_BASE + 11,
        // Os modificadores: um jogo liga-lhes acções (agachar no Ctrl, correr no Shift).
        KeyCode::ShiftLeft => 0xF710,
        KeyCode::ShiftRight => 0xF711,
        KeyCode::ControlLeft => 0xF712,
        KeyCode::ControlRight => 0xF713,
        KeyCode::AltLeft => 0xF714,
        KeyCode::AltRight => 0xF715,
        // ⛔ **As cinco que a auditoria de 2026-08-24 achou INLIGÁVEIS.** Elas têm atalho de editor
        // e não estavam em normalizador nenhum: o artista carregava e a acção nunca aparecia, sem
        // um aviso. *Uma tecla que o mapa não sabe soletrar é uma tecla que o artista não pode
        // usar* — a irmã exacta do `KeyG` que faltava e fazia o `Ctrl+G` do grafo cair no atalho
        // errado (Enio, 2026-07-13).
        KeyCode::BracketLeft => 0x5B,
        KeyCode::BracketRight => 0x5D,
        KeyCode::Comma => 0x2C,
        KeyCode::Period => 0x2E,
        KeyCode::Home => 0xF729,
        _ => return None,
    };
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor::interaction::{GraphKey, graph_key_for};

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
            KeyCode::Tab, KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space, KeyCode::Escape,
            KeyCode::Backspace, KeyCode::Delete, KeyCode::F2, KeyCode::ArrowUp, KeyCode::ArrowDown,
            KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::KeyA, KeyCode::KeyC, KeyCode::KeyD,
            KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyK,
            KeyCode::KeyL, KeyCode::KeyP, KeyCode::KeyV, KeyCode::KeyX,
        ];
        let mut checked = 0;
        for code in every {
            let Some(editor) = winit_to_editor_keycode(code) else {
                continue;
            };
            checked += 1;
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
        for code in [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyZ, KeyCode::KeyQ, KeyCode::ShiftLeft]
        {
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
