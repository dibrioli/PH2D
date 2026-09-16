//! **AS STRINGS DO NAVEGADOR DE ASSETS** — `panel.asset_browser.*`.
//!
//! ⚠️ **Um corte por ASSUNTO**, como os irmãos. Migrado em 2026-09-16 por
//! `scripts/migrar-texto-pintado.py` (mapa `docs/UI_New_and_Simple/ferramentas/seccoes_asset_browser.tsv`).
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave do navegador de assets, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "panel.asset_browser.title" => "Assets",
        "panel.asset_browser.panel.search_assets" => "Search assets\u{2026}",
        "panel.asset_browser.panel.loading_assets" => "Loading assets\u{2026}",
        "panel.asset_browser.panel.no_assets_yet" => {
            "No assets yet \u{2014} right-click an object and choose Make Prefab"
        }
        "panel.asset_browser.panel.this_asset_uses_nothing_else" => "This asset uses nothing else",
        "panel.asset_browser.panel.nothing_in_the_library_uses_this" => {
            "Nothing in the library uses this"
        }
        "panel.asset_browser.panel.nothing_matches_this_search" => "Nothing matches this search",
        "panel.asset_browser.panel.n_more" => {
            "+{beyond} more \u{2014} narrow the search to reach them"
        }
        "panel.asset_browser.catalog.all" => "All",
        "panel.asset_browser.catalog.unassigned" => "Unassigned",
        "panel.asset_browser.catalog.catalog" => "+ Catalog",
        "panel.asset_browser.related.this_asset" => "this asset",
        "panel.asset_browser.related.what_name_uses" => "What \u{201c}{name}\u{201d} uses",
        "panel.asset_browser.related.what_uses_name" => "What uses \u{201c}{name}\u{201d}",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
