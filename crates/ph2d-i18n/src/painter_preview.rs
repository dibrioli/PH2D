//! **As palavras do PINO da preview do pincel** (painel Painter, vista Brush): a dica do botão que fixa a
//! preview viva no topo enquanto o resto rola. Tabela irmã da [`super::painter_layers`] (ela está no tecto
//! de LOC) por isolamento (`CLAUDE.md` §0.2).

/// A tradução de uma chave `panel.painter_preview.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "panel.painter_preview.pin" => "Pin preview",
        "panel.painter_preview.unpin" => "Unpin preview",
        _ => return None,
    })
}
