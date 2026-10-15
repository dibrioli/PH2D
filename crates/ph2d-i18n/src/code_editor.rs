//! **O EDITOR DE SCRIPTS** (CodeEditor Onda 1, `docs/CodeEditor/02_onda1.md`) e as abas de documento
//! de família (`ph2d_editor_core::doc_surface`).

/// A tradução de uma chave `code.*` / `doc.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // A aba com alterações por salvar: o nome e o ponto.
        "doc.tab.dirty_title" => "{name} ●",
        _ => return None,
    })
}
