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
        // O script tem `--!strict` na 1.ª linha (I14).
        "code.status.strict" => "Strict types",
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
        // O Ctrl+S da aba nunca abre diálogo: sem ficheiro de projecto, aplica e di-lo.
        "code.status.project_unsaved" => "Script applied · project not saved yet — File ▸ Save As…",
        "code.status.project_unsaved_after" => "{badge} · project not saved yet — File ▸ Save As…",
        // O assistente de código (`luau-lsp`) quando não está pronto: o editor continua a editar.
        "code.status.assist_starting" => "Starting code assistant…",
        "code.status.assist_restarting" => "Code assistant stopped — restarting it…",
        "code.status.assist_missing" => {
            "Code assistant not installed — editing works, without suggestions"
        }
        "code.status.assist_stopped" => {
            "Code assistant stopped — editing works; Edit ▸ Restart Code Assistant"
        }
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
        // A lista de completação (I4): o detalhe de uma propriedade `ph2d.property` depois de
        // `self.`, e o nome da lista para o leitor de ecrã.
        "code.complete.property" => "property",
        // Os esqueletos (I12): o detalhe de um gancho do motor e o de um modelo de chamada.
        "code.complete.hook" => "engine hook",
        "code.complete.skeleton" => "skeleton",
        "code.a11y.completion" => "Suggestions",
        // As acções de contexto (I7, Alt+Enter): o «você quis dizer», a dica de tipo inserida, e a
        // lista vazia. (O aviso do campo do `self` é `code.explain.unknown_field`, I8.)
        "code.action.did_you_mean" => "Did you mean {name}?",
        "code.action.add_type" => "Add type: {type}",
        "code.action.none" => "No quick fixes here",
        "code.a11y.actions" => "Quick fixes",
        // Renomear (I10, F2): a ajuda por baixo do campo e porque um nome não serve; o desfazer de um
        // rename de propriedade, que mora no texto e na cena, recusado por inteiro.
        "code.rename.hint" => "Enter to rename · Esc to cancel",
        "code.rename.hint_objects" => "Enter to rename · keeps the value of {count} objects",
        "code.rename.hint_objects_one" => "Enter to rename · keeps the value of 1 object",
        "code.rename.nothing" => "Nothing to rename here",
        "code.rename.member" => "“{name}” belongs to another table — rename it where it is made",
        "code.rename.hook" => "The engine calls “{name}” by this name",
        "code.rename.escapes" => "self is passed around in this script — rename “{name}” by hand",
        "code.rename.invalid" => "“{name}” is not a valid name",
        "code.rename.reserved" => "“{name}” is a reserved word",
        "code.rename.in_use" => "“{name}” is already used in this script",
        "code.rename.taken" => "A property “{name}” already exists",
        "code.rename.offline" => "The code assistant is not running",
        "code.rename.failed" => "The code assistant could not rename this",
        "code.a11y.rename" => "Rename: {note}",
        // Shift+F12 (I10): uma linha da lista dos sítios de um nome, e o nome da lista.
        "code.refs.row" => "{line}: {code}",
        "code.a11y.references" => "References",
        "code.undo.family_ahead" => {
            "Undo the script edits made after the rename first — it changed the script and the scene together"
        }
        "code.undo.global_ahead" => {
            "Undo the scene changes made after the rename first — it changed the script and the scene together"
        }
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
        "code.cmd.restart_assistant" => "Restart Code Assistant",
        "code.cmd.rename" => "Rename Symbol \u{00b7} F2",
        "code.cmd.references" => "Find References \u{00b7} Shift+F12",
        // Ligar/desligar (I9, I14): o rótulo diz o que o comando FAZ agora.
        "code.cmd.type_hints" => "Show Type Hints",
        "code.cmd.type_hints_off" => "Hide Type Hints",
        "code.cmd.strict" => "Turn On Strict Types (--!strict)",
        "code.cmd.strict_off" => "Turn Off Strict Types (--!strict)",
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
