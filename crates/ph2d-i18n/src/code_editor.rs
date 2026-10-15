//! **O EDITOR DE SCRIPTS** (CodeEditor Onda 1, `docs/CodeEditor/02_onda1.md`) e as abas de documento
//! de família (`ph2d_editor_core::doc_surface`).

/// A tradução de uma chave `code.*` / `doc.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // A aba com alterações por salvar: o nome e o ponto.
        "doc.tab.dirty_title" => "{name} ●",
        // A barra de estado do editor (E4): o cursor, os problemas e o selo do último salvar.
        "code.status.position" => "Ln {line}, Col {col}",
        "code.status.no_problems" => "No problems",
        "code.status.problems" => "{errors} errors · {warnings} warnings",
        "code.status.reloaded" => "Reloaded · {count} objects",
        "code.status.reloaded_one" => "Reloaded · 1 object",
        "code.status.refused_line" => "Not applied: error on line {line} — the previous version keeps running",
        "code.status.refused" => "Not applied: the script has an error — the previous version keeps running",
        // O fim de uma linha com dobra fechada.
        "code.fold.ellipsis" => "…",
        // O nó de acessibilidade do texto do script (E12).
        "code.a11y.document" => "Script",
        _ => return None,
    })
}
