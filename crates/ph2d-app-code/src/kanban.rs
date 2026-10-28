//! W11 do Kanban — os scripts que um cartão pediu para abrir (`kanban_links::open`): o editor de
//! scripts drena-os em [`crate::frame::frame`] e abre a aba de cada um.

use ph2d_editor_core::kanban_links::{Family, Target, take};
use ph2d_script::ScriptId;

/// Os scripts pedidos desde o último quadro, pela ordem (o último fica com o foco).
#[must_use]
pub(crate) fn scripts_asked() -> Vec<ScriptId> {
    take(Family::Script)
        .into_iter()
        .filter_map(|t| match t {
            Target::Script(s) => Some(ScriptId(s)),
            _ => None,
        })
        .collect()
}

/// ⭐ J3 — o texto salvo de `script`: se o código usa `ph2d.kanban` é um script de Kanban, e corre
/// sobre o projecto no quadro seguinte (quem tem os Kanbans e o desfazer serve o pedido —
/// `ph2d_kanban_hooks::take_runs`); os `ph2d.kanban.on` dele ficam activos. Deixar de usar
/// `ph2d.kanban` larga os handlers.
pub(crate) fn saved(script: ScriptId, name: &str, text: &str) {
    if ph2d_kanban_hooks::is_kanban_script(text) {
        ph2d_kanban_hooks::request_run(script.0, name, text);
    } else {
        ph2d_kanban_hooks::unload(script.0);
    }
}

/// O que os scripts de Kanban disseram (o `print`, os erros) vai para o console de cada um — o
/// mesmo console da cena; um erro abre-o.
pub(crate) fn hear(docs: &mut crate::docs::CodeDocs) {
    use ph2d_script::{ConsoleKind, ConsoleLine, ScriptDiagnostic};
    for l in ph2d_kanban_hooks::take_lines() {
        let script = ScriptId(l.key);
        let kind = if l.error {
            let mut d = ScriptDiagnostic::runtime(l.text.clone());
            d.line = l.line;
            ConsoleKind::RuntimeError(d)
        } else {
            ConsoleKind::Print
        };
        let line = ConsoleLine {
            script,
            entity: None,
            text: l.text,
            kind,
        };
        if docs.consoles.entry(script).or_default().push(&line)
            && let Some(d) = docs.docs.iter_mut().find(|d| d.script == script)
        {
            d.console_view.follow();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kanban_o_ctrl_s_de_um_script_que_usa_ph2d_kanban_pede_a_corrida_e_o_resto_larga() {
        let _ = ph2d_kanban_hooks::take_runs();
        saved(ScriptId(5), "k.luau", "local c = ph2d.kanban.get(\"x\")");
        saved(ScriptId(6), "scene.luau", "-- ph2d.kanban (só um comentário)\nprint(1)");
        let runs = ph2d_kanban_hooks::take_runs();
        assert_eq!(runs.len(), 1, "só o script de Kanban corre no projecto");
        assert_eq!((runs[0].key, runs[0].name.as_str()), (5, "k.luau"));
    }
}
