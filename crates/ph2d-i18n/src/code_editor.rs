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
        // As contagens têm a forma de UM à parte (`_one`): «1 errors» era o que a barra dizia.
        "code.status.problems" => "{errors} · {warnings}",
        "code.status.errors" => "{count} errors",
        "code.status.errors_one" => "1 error",
        "code.status.warnings" => "{count} warnings",
        "code.status.warnings_one" => "1 warning",
        "code.status.reloaded" => "Reloaded · {count} objects",
        "code.status.reloaded_one" => "Reloaded · 1 object",
        "code.status.refused_line" => {
            "Not applied: error on line {line} — the previous version keeps running"
        }
        "code.status.refused" => {
            "Not applied: the script has an error — the previous version keeps running"
        }
        "code.status.saved_unused" => "Saved · no object uses this script",
        // A barra de buscar/substituir e o «Go to line» (E9).
        "code.find.placeholder" => "Find",
        "code.find.replace_placeholder" => "Replace",
        "code.find.count" => "{current} of {total}",
        "code.find.total" => "{total} results",
        "code.find.total_one" => "1 result",
        "code.find.none" => "No results",
        "code.find.case" => "Aa",
        "code.find.word" => "Word",
        "code.find.replace_one" => "Replace",
        "code.find.replace_all" => "Replace all",
        "code.goto.label" => "Go to line",
        "code.goto.placeholder" => "Line number",
        // O console da aba (E10).
        "code.console.title" => "Console",
        "code.console.title_errors" => "Console · {count} errors",
        "code.console.title_errors_one" => "Console · 1 error",
        "code.console.clear" => "Clear",
        "code.console.empty" => "Nothing printed yet — print() from the script shows here",
        "code.console.runtime_error" => "Line {line}: {msg}",
        "code.console.runtime_error_noline" => "Error: {msg}",
        "code.console.refused" => "Not applied — line {line}: {msg}",
        "code.console.refused_noline" => "Not applied: {msg}",
        // Os problemas vivos do texto (E7), enquanto se escreve.
        "code.problem.syntax" => "Syntax error",
        "code.problem.missing" => "Missing {sym}",
        // O fim de uma linha com dobra fechada.
        "code.fold.ellipsis" => "…",
        // Outro projecto aberto com textos por gravar: ficaram em ficheiros (nunca em silêncio).
        "code.recovery.kept" => {
            "{count} unsaved scripts of the previous project were kept in {dir}"
        }
        "code.recovery.kept_one" => "1 unsaved script of the previous project was kept in {dir}",
        "code.recovery.failed" => {
            "Could not keep {count} unsaved scripts of the previous project: {err}"
        }
        // O nó de acessibilidade do texto do script (E12).
        "code.a11y.document" => "Script",
        // E13: os comandos da aba nos menus da barra e no Ctrl+K (o atalho no rótulo, como o File).
        "code.cmd.save" => "Save Script \u{00b7} Cmd+S",
        "code.cmd.find" => "Find\u{2026} \u{00b7} Cmd+F",
        "code.cmd.replace" => "Replace\u{2026} \u{00b7} Cmd+H",
        "code.cmd.goto" => "Go to Line\u{2026} \u{00b7} Cmd+G",
        "code.cmd.fold_all" => "Fold All",
        "code.cmd.unfold_all" => "Unfold All",
        // E11: a lista de scripts do projeto, à esquerda dentro da aba.
        "code.list.title" => "Scripts",
        "code.list.filter" => "Filter",
        "code.list.new" => "New script",
        "code.list.rename" => "Rename",
        "code.list.delete" => "Delete",
        "code.list.no_match" => "No script matches",
        "code.list.users" => "{count} objects",
        "code.list.users_one" => "1 object",
        "code.list.in_use" => {
            "\u{201c}{name}\u{201d} is used by {count} object(s) \u{2014} it was not deleted"
        }
        "code.list.show" => "Show the script list",
        "code.list.hide" => "Hide the script list",
        _ => return None,
    })
}
