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
        KEY_KEY_H, KEY_KEY_K, KEY_KEY_L, KEY_KEY_P, KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB,
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
    #[test]
    fn every_graph_verb_is_reachable_through_the_shells_normalizer() {
        let cases = [
            (KeyCode::KeyG, true, false, GraphKey::Group),
            (KeyCode::KeyG, true, true, GraphKey::Ungroup),
            (KeyCode::KeyD, true, false, GraphKey::Duplicate),
            (KeyCode::KeyF, false, false, GraphKey::Fit),
            (KeyCode::KeyA, false, false, GraphKey::Add),
            (KeyCode::KeyA, true, false, GraphKey::SelectAll),
            (KeyCode::KeyL, true, false, GraphKey::SelectLinked),
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
