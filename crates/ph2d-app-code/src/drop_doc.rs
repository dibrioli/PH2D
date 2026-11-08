//! ⭐ **A aba de script como alvo de um arrasto** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.11–12): por cima do texto ela mostra onde cairia (o cursor fino pela MESMA `hit` do clique) e o
//! balão das teclas; ao largar escreve pela lei pura ([`ph2d_code_syntax::dropping`]) num passo de undo.

use crate::actions::{Action, Then};
use crate::doc::CodeDoc;
use ph2d_code_buffer::{Range, Selection};
use ph2d_code_syntax::dropping::{
    Insertion, insertion, name_insertion, object_property_insertion, property_insertion,
};
use ph2d_editor_core::doc_surface::{DropAt, Dropping};
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_code::Hit;

/// O que a aba mostra durante o arrasto: o byte onde cairia e as linhas do balão.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DropMark {
    pub at: usize,
    pub hint: Vec<String>,
}

impl CodeDoc {
    /// O byte de texto sob `(x, y)`, se o ponto cai no TEXTO (não na calha, na barra, num balão).
    fn drop_byte(&self, x: f32, y: f32) -> Option<usize> {
        if self.view.hint_hover_contains(x, y) {
            return None;
        }
        match self.view.hit(&self.buffer, x, y) {
            Hit::Text(b) => Some(b),
            _ => None,
        }
    }

    /// O que a soltura em `b` escreve, pela lei do que chega.
    /// Uma linha do Inspector com propriedade de OUTRO script não tem o que escrever aqui.
    fn welcomes(&self, what: Dropping<'_>) -> bool {
        match what {
            Dropping::Property {
                script: Some(id), ..
            } => id == self.script.0,
            _ => true,
        }
    }

    fn drop_insertion(&self, b: usize, at: DropAt<'_>) -> Insertion {
        let text = self.buffer.text();
        match at.what {
            Dropping::Objects(names) if at.mods.alt => {
                object_property_insertion(text, &self.syntax, b, names)
            }
            Dropping::Objects(names) => insertion(
                text,
                &self.syntax,
                b,
                names,
                at.mods.ctrl,
                &self.auto.unit(),
            ),
            Dropping::Names(names) => name_insertion(text, b, names, at.mods.ctrl),
            Dropping::Property { names, script } => property_insertion(b, names, script.is_some()),
        }
    }

    /// Um arrasto passa (`Some`) ou saiu (`None`). `true` = largar aqui escreve.
    pub(crate) fn drop_hover(&mut self, at: Option<DropAt<'_>>) -> bool {
        self.dropping = at.filter(|a| self.welcomes(a.what)).and_then(|at| {
            let b = self.drop_byte(at.x, at.y)?;
            let ins = self.drop_insertion(b, at);
            let hint = |keys: &[&str]| keys.iter().map(|k| tr(k).to_owned()).collect();
            let (what, ctrl) = match at.what {
                Dropping::Property { script, .. } => {
                    let key = match script {
                        Some(_) => "code.drop.property_self",
                        None => "code.drop.property",
                    };
                    return Some(DropMark {
                        at: ins.at,
                        hint: hint(&[key]),
                    });
                }
                Dropping::Names(_) => ("code.drop.names", "code.drop.ctrl_const"),
                Dropping::Objects(_) if self.syntax.at_top_level(self.buffer.text(), b) => {
                    ("code.drop.objects", "code.drop.ctrl_init")
                }
                Dropping::Objects(_) => ("code.drop.objects", "code.drop.ctrl_local"),
            };
            Some(DropMark {
                at: ins.at,
                hint: match at.what {
                    Dropping::Objects(_) => hint(&[what, ctrl, "code.drop.alt_property"]),
                    _ => hint(&[what, ctrl]),
                },
            })
        });
        self.dropping.is_some()
    }

    /// Largou-se: escreve num passo de undo e põe o cursor no fim do que escreveu. `true` = escreveu.
    pub(crate) fn drop(&mut self, at: DropAt<'_>) -> bool {
        self.dropping = None;
        let Some(b) = self
            .drop_byte(at.x, at.y)
            .filter(|_| self.welcomes(at.what))
        else {
            return false;
        };
        let (Dropping::Objects(names) | Dropping::Names(names) | Dropping::Property { names, .. }) =
            at.what;
        if names.is_empty() {
            return false;
        }
        if matches!(at.what, Dropping::Objects(_)) && !at.mods.ctrl && !at.mods.alt {
            self.offer_drop(b, at, names);
            return true;
        }
        let ins = self.drop_insertion(b, at);
        self.buffer.seal_history();
        self.buffer
            .set_selection(Selection::single(Range::new(ins.at, ins.at)));
        let edits = self.buffer.insert(&ins.text);
        self.edited(&edits, true);
        self.buffer
            .set_selection(Selection::single(Range::new(ins.caret, ins.caret)));
        self.buffer.seal_history();
        self.reveal();
        true
    }

    /// ⭐ Item 15 (doc 13 §4): a soltura SEM tecla pergunta — Name · Variable · Inspector field, cada
    /// linha com o que escreve. Cada escolha é a MESMA inserção da tecla (sem tecla · Ctrl · Alt), num
    /// passo de undo; Esc ou um clique fora não escrevem nada. Hoje nenhum contexto tira uma das três
    /// (a lei de `dropping` responde as três em qualquer byte de código), por isso a pergunta é sempre.
    fn offer_drop(&mut self, b: usize, at: DropAt<'_>, names: &[String]) {
        let with = |ctrl, alt| DropAt {
            mods: ph2d_host::Modifiers {
                ctrl,
                alt,
                ..ph2d_host::Modifiers::default()
            },
            ..at
        };
        let first = ph2d_code_syntax::dropping::reference(&names[0]);
        let rows = [
            (false, false, "code.drop.choice_name"),
            (true, false, "code.drop.choice_variable"),
            (false, true, "code.drop.choice_field"),
        ];
        let list = rows
            .into_iter()
            .map(|(ctrl, alt, key)| {
                let ins = self.drop_insertion(b, with(ctrl, alt));
                // O que se lê é a linha que nomeia o 1.º objecto (com Ctrl no topo, a do `init`).
                let code = ins
                    .text
                    .lines()
                    .find(|l| l.contains(&first))
                    .unwrap_or(&ins.text)
                    .trim()
                    .to_owned();
                let more = if names.len() > 1 && !code.contains(", ") {
                    " \u{2026}"
                } else {
                    ""
                };
                Action {
                    title: tr_with(key, &[("code", &format!("{code}{more}"))]),
                    edits: vec![(ins.at..ins.at, ins.text)],
                    then: Then::Caret(ins.caret),
                }
            })
            .collect();
        let title = tr_with("code.drop.choose_title", &[("name", &names[0])]);
        let note = format!("{title}\n\n{}", tr("code.drop.choose_skip"));
        self.offer(b, list, note);
        self.reveal();
    }
}
