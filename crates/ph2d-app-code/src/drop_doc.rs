//! ⭐ **A aba de script como alvo de um arrasto** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.11–12): por cima do texto ela mostra onde cairia (o cursor fino pela MESMA `hit` do clique) e o
//! balão das teclas; ao largar escreve pela lei pura ([`ph2d_code_syntax::dropping`]) num passo de undo.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Range, Selection};
use ph2d_code_syntax::dropping::{Insertion, insertion, name_insertion, property_insertion};
use ph2d_editor_core::doc_surface::{DropAt, Dropping};
use ph2d_i18n::tr;
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
                hint: hint(&[what, ctrl]),
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
}
