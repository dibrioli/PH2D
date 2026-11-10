//! **B · Keyboard** (R3): o teclado e o comando pintados, a barra do curso e quem lê cada acção (plano
//! `docs/InputMap/01_plano.md` §3). Por ora o cartão fica vazio: nenhum conteúdo de mentira.

use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::doc::InputMapDoc;

/// Pinta a vista dentro do cartão `body` (o cartão já está pintado).
pub fn paint(
    _doc: &InputMapDoc,
    _body: Rect,
    _scene: &mut VectorScene,
    _text: &mut TextSystem,
    _theme: Theme,
) {
}
