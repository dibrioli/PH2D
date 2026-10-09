//! **A HIERARQUIA além do objecto** — a raiz Cena, os grupos de conteúdo e os avisos de abrir uma
//! peça (`docs/ObjectsAndComponents/01_objeto_e_componente.md` §4–§7).

/// A tradução de uma chave das linhas da Hierarquia, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "hier_rows.scene" => "Scene",
        "hier_rows.content.layers" => "Layers ({n})",
        "hier_rows.content.clips" => "Clips ({n})",
        "hier_rows.content_gone" => "That piece is no longer there",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
