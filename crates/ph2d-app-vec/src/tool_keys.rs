//! ⭐ **As teclas do vetor, APLICADAS** (D10, o padrão Illustrator). A TABELA é do painel
//! ([`ph2d_panel_vector::tool_keys`]); aqui mora o que ela faz quando uma tecla chega.
//!
//! ⭐⭐ **Uma tecla é um CLIQUE no mesmo id** que o botão da coluna ① (ou da ④) tem — entregue pela
//! mesma porta (`HeroScreen::apply_event`), como a paleta `Ctrl+K` já faz. Tecla, botão e paleta
//! fazem a MESMA coisa porque não há segundo caminho.
//!
//! ⭐ **A ferramenta TEMPORÁRIA** (estudo §2, traço 4 do Affinity): segurar a tecla dá a ferramenta,
//! soltar volta à anterior; um toque troca de vez. *Segurar* é o que o SISTEMA OPERATIVO já mede —
//! a tecla chegou a auto-repetir antes de ser solta. ⛔ Nenhum limiar de milissegundos inventado:
//! o do artista é o atraso de repetição que ele configurou.

use ph2d_editor_core::NodeId;
use ph2d_panel_vector::tool_keys::{Chord, KeyAction, action_for};
use ph2d_tool_vector::DrawMode;
use ph2d_tool_vector::ids;

/// O que a shell faz com uma tecla do vetor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyOutcome {
    /// Cliques nestes ids, por esta ordem (a mesma porta dos botões).
    Clicks(Vec<NodeId>),
    /// A booleana das duas últimas formas (a mesma ação que o `U/I/D/X` fazia, agora em `Ctrl+Alt`).
    Boolean(ph2d_vec_boolean::PathfinderOp),
    /// A tecla é do vetor mas não faz nada agora (a repetição de uma tecla segurada) — consome-a.
    Swallow,
}

/// Uma ferramenta segurada: a tecla, a ferramenta de antes e se a tecla chegou a repetir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hold {
    chord: Chord,
    prev: DrawMode,
    held: bool,
}

/// O acorde de uma tecla — `code` é o código ASCII maiúsculo da letra (o espaço de
/// `keymap::winit_to_input_keycode`); `None` para tudo o que não é letra ou para os acordes que não
/// são do vetor (`Ctrl` sozinho, `Alt` sozinho, `Super`: são de ficheiro e de sistema).
#[must_use]
pub fn chord(code: u32, shift: bool, ctrl: bool, alt: bool, super_key: bool) -> Option<Chord> {
    let letter = char::from_u32(code)
        .filter(char::is_ascii_uppercase)?
        .to_ascii_lowercase();
    if super_key || ctrl != alt {
        return None;
    }
    Some(Chord {
        letter,
        shift,
        ctrl_alt: ctrl && alt,
    })
}

fn rectangle_and_ellipse(kind: ph2d_vec_scene::ShapeKind) -> Option<NodeId> {
    ph2d_tool_vector::shapes::SHAPES
        .iter()
        .position(|d| d.kind == kind)
        .map(ids::vector_shape_id)
}

/// Os cliques de uma ação.
fn clicks(action: KeyAction) -> Option<KeyOutcome> {
    use ph2d_vec_boolean::PathfinderOp as Op;
    let out = match action {
        KeyAction::Mode(m) => KeyOutcome::Clicks(vec![ph2d_panel_vector::bars::mode_id(m)?]),
        KeyAction::ShapeRect | KeyAction::ShapeEllipse => {
            let kind = if action == KeyAction::ShapeRect {
                ph2d_vec_scene::ShapeKind::Rectangle
            } else {
                ph2d_vec_scene::ShapeKind::Ellipse
            };
            KeyOutcome::Clicks(vec![ids::VECTOR_MODE_SHAPE, rectangle_and_ellipse(kind)?])
        }
        KeyAction::ToggleFillStroke => KeyOutcome::Clicks(vec![ids::VECTOR_COLOUR_FOCUS]),
        KeyAction::SwapFillStroke => KeyOutcome::Clicks(vec![ids::VECTOR_COLOUR_SWAP]),
        KeyAction::DefaultColors => KeyOutcome::Clicks(vec![ids::VECTOR_COLOUR_DEFAULT]),
        KeyAction::Union => KeyOutcome::Boolean(Op::Union),
        KeyAction::Intersect => KeyOutcome::Boolean(Op::Intersect),
        KeyAction::Subtract => KeyOutcome::Boolean(Op::Subtract),
        KeyAction::Exclude => KeyOutcome::Boolean(Op::Exclude),
    };
    Some(out)
}

/// ⭐ **Uma tecla do vetor.** `pressed` falso = soltar; `repeat` = auto-repetição do sistema;
/// `mode` = a ferramenta na mão agora. Devolve `None` quando a tecla não é do vetor (segue a cadeia).
pub fn key(
    hold: &mut Option<Hold>,
    chord: Chord,
    pressed: bool,
    repeat: bool,
    mode: DrawMode,
) -> Option<KeyOutcome> {
    if !pressed {
        // Soltar: só a tecla segurada interessa, e só se chegou a repetir (segurada, não tocada).
        let h = hold.take_if(|h| h.chord.letter == chord.letter)?;
        if h.held && h.prev != mode {
            return Some(KeyOutcome::Clicks(vec![ph2d_panel_vector::bars::mode_id(
                h.prev,
            )?]));
        }
        return Some(KeyOutcome::Swallow);
    }
    let action = action_for(chord)?;
    if repeat {
        if let Some(h) = hold.as_mut().filter(|h| h.chord == chord) {
            h.held = true;
        }
        return Some(KeyOutcome::Swallow);
    }
    // Só as ferramentas são temporárias; a cor e as booleanas são verbos de um toque.
    if matches!(
        action,
        KeyAction::Mode(_) | KeyAction::ShapeRect | KeyAction::ShapeEllipse
    ) {
        *hold = Some(Hold {
            chord,
            prev: mode,
            held: false,
        });
    }
    clicks(action)
}

#[cfg(test)]
#[path = "tool_keys_tests.rs"]
mod tests;
