//! ⭐ **A aba de script como alvo de um arrasto** (item 8, `docs/CodeEditor/07_arrastar_para_o_codigo.md`
//! §3.11–12): por cima do texto ela mostra onde cairia (o cursor fino pela MESMA `hit` do clique) e o
//! balão das teclas; ao largar escreve pela lei pura ([`crate::dropping`]) num passo de undo.

use crate::doc::CodeDoc;
use crate::dropping::insertion;
use ph2d_code_buffer::{Range, Selection};
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

    /// Um arrasto passa (`Some`) ou saiu (`None`). `true` = largar aqui escreve.
    pub(crate) fn drop_hover(&mut self, at: Option<DropAt<'_>>) -> bool {
        self.dropping = at.and_then(|at| {
            let b = self.drop_byte(at.x, at.y)?;
            let Dropping::Objects(names) = at.what;
            let ins = insertion(
                self.buffer.text(),
                &self.syntax,
                b,
                names,
                at.mods.ctrl,
                &self.auto.unit(),
            );
            let top = self.syntax.at_top_level(self.buffer.text(), b);
            let ctrl = if top {
                "code.drop.ctrl_init"
            } else {
                "code.drop.ctrl_local"
            };
            Some(DropMark {
                at: ins.at,
                hint: vec![tr("code.drop.objects").to_owned(), tr(ctrl).to_owned()],
            })
        });
        self.dropping.is_some()
    }

    /// Largou-se: escreve num passo de undo e põe o cursor no fim do que escreveu. `true` = escreveu.
    pub(crate) fn drop(&mut self, at: DropAt<'_>) -> bool {
        self.dropping = None;
        let Some(b) = self.drop_byte(at.x, at.y) else {
            return false;
        };
        let Dropping::Objects(names) = at.what;
        if names.is_empty() {
            return false;
        }
        let ins = insertion(
            self.buffer.text(),
            &self.syntax,
            b,
            names,
            at.mods.ctrl,
            &self.auto.unit(),
        );
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
