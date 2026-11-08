//! **O EDITOR DE SCRIPTS** (CodeEditor Onda 1, `docs/CodeEditor/02_onda1.md`) e as abas de documento
//! de família (`ph2d_editor_core::doc_surface`).

/// A tradução de uma chave `code.*` / `doc.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // A aba com alterações por salvar: o nome e o ponto.
        "doc.tab.dirty_title" => "{name} ●",
        // A barra de estado do editor (E4): o cursor, os problemas e o selo do último salvar.
        "code.status.position" => "Ln {line}, Col {col}",
        "code.complete.scene_object" => "object in the scene",
        "code.complete.scene_master" => "master",
        "code.drop.objects" => "Drop: choose what to write here.",
        "code.drop.ctrl_local" => "Hold Ctrl: a local variable on this line.",
        "code.drop.ctrl_init" => "Hold Ctrl: kept in self, inside init.",
        "code.drop.alt_property" => "Hold Alt: an object property, set in the Inspector.",
        "code.drop.names" => "Drop: writes the name, in quotes, here.",
        "code.drop.ctrl_const" => "Hold Ctrl: a local constant on this line.",
        "code.drop.property" => "Drop: writes the property name, in quotes, here.",
        "code.drop.property_self" => "Drop: writes self.<name>, a property of this script.",
        // Onda C item 15 (doc 13 §4): a soltura sem tecla pergunta o que escrever.
        "code.drop.choose_title" => "{name} \u{2014} what to write?",
        "code.drop.choose_skip" => "Hold Ctrl or Alt while dropping to skip this.",
        "code.drop.choice_name" => "{code} \u{2014} Name",
        "code.drop.choice_variable" => "{code} \u{2014} Variable",
        "code.drop.choice_field" => "{code} \u{2014} Inspector field",
        // Onda C item 16: o número no corpo de uma função vira um campo do Inspector.
        "code.action.make_property" => "Make it a property (Inspector field)",
        // O script tem `--!strict` na 1.ª linha (I14).
        "code.status.strict" => "Strict types",
        // O contador de problemas (ícones e números) ao leitor de ecrã: o molde do VS Code 1.142.
        "code.status.counter" => "Errors: {errors}, Warnings: {warnings}",
        "code.status.reloaded" => "Reloaded · {count} {count?object|objects}",
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
        // Diz o que se PERDE (curto: a barra é partilhada); os esqueletos locais ainda abrem a lista.
        "code.status.assist_missing" => {
            "Code assistant missing — no smart suggestions, help or rename"
        }
        "code.status.assist_stopped" => {
            "Code assistant stopped — editing works; Run ▸ Restart Code Assistant"
        }
        // A barra de buscar/substituir e o «Go to line» (E9).
        "code.find.placeholder" => "Find",
        "code.find.replace_placeholder" => "Replace",
        "code.find.count" => "{current} of {total}",
        "code.find.total" => "{total} {total?result|results}",
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
        "code.console.clear" => "Clear",
        "code.console.empty" => "Nothing printed yet — print() from the script shows here",
        // A lista de problemas, o 2.º separador da faixa de baixo (acabamento, Onda 2 item 4).
        "code.band.problems" => "Problems",
        "code.band.problems_count" => "Problems ({count})",
        "code.problems.empty" => "No problems in this script.",
        "code.problems.line" => "Line {line}",
        // Onda A (doc 13 §4.1): um problema do Play — os objectos onde aconteceu e quantas vezes.
        "code.problems.on_objects" => "{text} — on {who}",
        "code.problems.on_objects_times" => "{text} — on {who} · ×{count}",
        "code.problems.times" => "{text} · ×{count}",
        "code.problems.on_objects_in_play" => "{text} — on {who} in Play",
        "code.problems.on_objects_times_in_play" => "{text} — on {who} · ×{count} in Play",
        "code.problems.in_play" => "{text} — in Play",
        "code.problems.times_in_play" => "{text} — ×{count} in Play",
        "code.problems.more" => "{names} +{count} more",
        // Onda A (doc 13 §4.2): quem disse o `print` e quando (o relógio do jogo).
        "code.console.tag" => "{who} · {time}",
        "code.console.time" => "{min}:{sec}",
        "code.console.object" => "Object {id}",
        // O leitor de ecrã: a lista e cada linha (o molde do VS Code 1.142, `oraculo_acabamento`).
        "code.problems.a11y_list" => "Problems: {count}",
        "code.problems.a11y_error" => "Error: {text}, line {line}",
        "code.problems.a11y_warning" => "Warning: {text}, line {line}",
        "code.problems.a11y_info" => "Info: {text}, line {line}",
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
        // Onda A (doc 13 §4, itens 3–4): comentar, dobrar o bloco do cursor, a contagem da lista.
        "code.cmd.toggle_comment" => "Toggle Comment \u{00b7} Cmd+/",
        "code.cmd.toggle_fold" => "Toggle Fold \u{00b7} Alt+F",
        "code.list.unused" => "unused",
        "code.list.used_by" => "Used by {count} {count?object|objects}",
        "code.list.used_by_none" => "No object uses this script",
        "code.cmd.restart_assistant" => "Restart Code Assistant",
        "code.cmd.rename" => "Rename Symbol \u{00b7} F2",
        "code.cmd.references" => "Find References \u{00b7} Shift+F12",
        // As três caixas do cartão «Script options» (13 §4 item 12); na paleta, o NOME da caixa e o
        // estado dela (escolher troca-o).
        "code.cmd.type_hints" => "Type Hints: Off",
        "code.cmd.type_hints_off" => "Type Hints: On",
        "code.cmd.strict" => "Strict Types (--!strict): Off",
        "code.cmd.strict_off" => "Strict Types (--!strict): On",
        "code.cmd.format_on_save" => "Format on Save: Off",
        "code.cmd.format_on_save_off" => "Format on Save: On",
        // 13 §4 item 11: o menu PRÓPRIO da aba (File · Edit · Find · Go To · Run).
        "code.menu.file" => "File",
        "code.menu.edit" => "Edit",
        "code.menu.find" => "Find",
        "code.menu.go_to" => "Go To",
        "code.menu.run" => "Run",
        "code.cmd.new_script" => "New Script",
        "code.cmd.save_as" => "Save As\u{2026} \u{00b7} Cmd+Shift+S",
        "code.cmd.close_tab" => "Close Tab \u{00b7} Cmd+W",
        "code.cmd.undo" => "Undo \u{00b7} Cmd+Z",
        "code.cmd.redo" => "Redo \u{00b7} Cmd+Shift+Z",
        "code.cmd.go_to_definition" => "Go to Definition \u{00b7} F12",
        "code.cmd.back" => "Back \u{00b7} Alt+\u{2190}",
        "code.cmd.forward" => "Forward \u{00b7} Alt+\u{2192}",
        "code.cmd.next_problem" => "Next Problem \u{00b7} F8",
        "code.cmd.script_options" => "Script Options\u{2026}",
        "code.menu.restart_hint" => "Only needed if suggestions stop working",
        "code.menu.options_tip" => "Script options",
        // 13 §4 item 13: o caminho até o cursor, à direita da barra da aba.
        "code.crumb.line" => "line {line}",
        "code.crumb.go_tip" => "Go to {name}",
        // 13 §4 item 14: o menu do botão direito no texto.
        "code.ctx.help_for" => "Help for {name}",
        "code.ctx.fix_keys" => "Alt+Enter",
        // 13 §4 item 12: o cartão «Script options».
        "code.options.title" => "Script options",
        "code.options.strict" => "Strict types",
        "code.options.format_on_save" => "Format on save",
        "code.options.type_hints" => "Type hints",
        "code.options.assist_on" => "Code assistant: on",
        "code.options.restart" => "Restart",
        "code.options.pin" => "Keep open",
        // I13: o formatador (StyLua) ao salvar.
        "code.format.missing" => {
            "The formatter (StyLua) is missing, so scripts are saved without formatting. {err}"
        }
        "code.format.would_change" => {
            "Not formatted: the formatter would change what the code does. Saved as written."
        }
        "code.format.failed" => "Not formatted: {err}",
        // E11: a lista de scripts do projeto, à esquerda dentro da aba.
        "code.list.title" => "Scripts",
        "code.list.filter" => "Filter",
        "code.list.new" => "New script",
        "code.list.rename" => "Rename",
        "code.list.delete" => "Delete",
        "code.list.no_match" => "No script matches",
        "code.list.users" => "{count} {count?object|objects}",
        "code.list.users_one" => "1 object",
        "code.list.in_use" => {
            "\u{201c}{name}\u{201d} is used by {count} {count?object|objects} \u{2014} it was not deleted"
        }
        "code.list.show" => "Show the script list",
        "code.list.hide" => "Hide the script list",
        // Onda 2, item 5: a página de ajuda (o cabeçalho, o índice e o botão de cada exemplo).
        "code.help.back" => "Back",
        "code.help.forward" => "Forward",
        "code.help.copy" => "Copy example",
        "code.help.sections" => "On this page",
        // As secções da página (`ph2d-app-code/src/help_doc.rs`), como as do Godot 4.7.2.
        "code.help.page.description" => "Description",
        "code.help.page.functions" => "Functions",
        "code.help.page.hooks" => "Hooks",
        "code.help.page.types" => "Types",
        "code.help.page.constants" => "Constants",
        "code.help.page.function_descriptions" => "Function descriptions",
        "code.help.page.hook_descriptions" => "Hook descriptions",
        "code.help.page.type_descriptions" => "Type descriptions",
        "code.help.page.constant_descriptions" => "Constant descriptions",
        "code.help.page.parameters" => "Parameters",
        "code.help.page.returns" => "Returns",
        "code.help.page.example" => "Example",
        // Entre o nome de um argumento (ou «Returns») e o texto dele.
        "code.help.page.dash" => " \u{2014} ",
        _ => return None,
    })
}
