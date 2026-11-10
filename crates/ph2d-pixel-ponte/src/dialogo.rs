//! **Os diálogos MODAIS do Pixel Art** (saído de `ph2d-app-pixel::compose`, 09/11): quando um está
//! aberto, toda tecla vai só para ele.

use ph2d_editor_core::interaction::{ContextMenuKind, WidgetStore};

/// Um diálogo do Pixel Art (*New Sprite*, *Color Mode*) está aberto — MODAL: toda tecla vai só para
/// ele.
#[must_use]
pub fn dialog_open(store: &WidgetStore) -> bool {
    store.context_menu().is_some_and(|r| {
        matches!(
            r.kind,
            ContextMenuKind::NewSpriteDialog
                | ContextMenuKind::NewImageDialog
                | ContextMenuKind::PixelColorModeDialog
                | ContextMenuKind::PixelEditDialog
        )
    })
}

/// Os caracteres do texto digitado que vão para o campo do diálogo (sem controlo, sem os atalhos
/// `Cmd`/`Ctrl` + letra).
pub fn dialog_text(text: Option<&str>, cmd: bool) -> impl Iterator<Item = char> + '_ {
    text.unwrap_or_default()
        .chars()
        .filter(move |ch| !ch.is_control() && !(cmd && ch.is_ascii_alphabetic()))
}
