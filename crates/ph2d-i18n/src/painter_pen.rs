//! ⭐ **As palavras do Reset do cartão Pen** (dono 25/10: voltar ao neutro da app OU ao que o preset do pincel
//! traz). Tabela IRMÃ da [`super::painter_layers`] por isolamento (`CLAUDE.md` §0.2): outras linhas mexem naquela
//! ao mesmo tempo.

/// A tradução de uma chave `panel.painter_pen.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.painter_pen.reset_default" => "Reset to default",
        "panel.painter_pen.reset_preset" => "Reset to brush preset",
        "panel.painter_pen.reset_preset_none" => "Reset to brush preset (this brush has no preset)",
        _ => return None,
    })
}
