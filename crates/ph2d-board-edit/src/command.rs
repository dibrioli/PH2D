//! As ordens do teclado fora do texto: apagar, duplicar, seleccionar tudo, sair, empurrar, desfazer,
//! copiar/cortar/colar, começar a escrever, trocar de ferramenta.

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, History};
use ph2d_text::TextSystem;

use crate::{Command, Editor, Tool};

impl Editor {
    /// Executa `c`. `false` = não fez nada (quem chama pode deixar a tecla seguir).
    pub fn command(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        c: Command,
    ) -> bool {
        match c {
            Command::Undo | Command::Redo => {
                self.commit_text(doc, history);
                self.cancel_gesture(doc);
                let done = if c == Command::Undo {
                    history.undo(doc)
                } else {
                    history.redo(doc)
                };
                self.prune(doc);
                done
            }
            Command::Escape => {
                if self.cancel_gesture(doc) {
                    return true;
                }
                if !self.selection.is_empty() {
                    self.selection.clear();
                    return true;
                }
                if self.tool != Tool::Select {
                    self.tool = Tool::Select;
                    return true;
                }
                false
            }
            Command::Tool(t) => {
                self.tool = t;
                true
            }
            Command::SelectAll => {
                self.selection = doc.live_in_z_order().iter().map(|el| el.id).collect();
                true
            }
            Command::EditText => {
                let [id] = self.selection.iter().copied().collect::<Vec<_>>()[..] else {
                    return false;
                };
                self.begin_text(doc, ts, id, None)
            }
            Command::Delete => {
                let ops: Vec<BoardOp> = self
                    .selection
                    .iter()
                    .map(|id| BoardOp::Delete(*id))
                    .collect();
                self.selection.clear();
                let any = !ops.is_empty();
                history.apply(doc, ops);
                any
            }
            Command::Cut => {
                self.copy(doc);
                self.command(doc, history, ts, Command::Delete)
            }
            Command::Copy => {
                self.copy(doc);
                !self.clipboard.is_empty()
            }
            Command::Nudge(d) => {
                let ops: Vec<BoardOp> = self
                    .selected(doc)
                    .into_iter()
                    .map(|el| {
                        let mut el = el.clone();
                        el.x += d[0];
                        el.y += d[1];
                        BoardOp::Put(el)
                    })
                    .collect();
                let any = !ops.is_empty();
                history.apply(doc, ops);
                any
            }
            Command::Duplicate => {
                let src: Vec<Element> = self.selected(doc).into_iter().cloned().collect();
                self.place_copies(doc, history, &src, 1)
            }
            Command::Paste => {
                self.pastes += 1;
                let src = self.clipboard.clone();
                self.place_copies(doc, history, &src, self.pastes)
            }
        }
    }

    fn copy(&mut self, doc: &BoardDoc) {
        self.clipboard = self.selected(doc).into_iter().cloned().collect();
        self.pastes = 0;
    }

    /// Cópias de `src` desviadas `steps × paste_offset`, à frente de tudo, seleccionadas — UM passo.
    fn place_copies(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        src: &[Element],
        steps: u32,
    ) -> bool {
        if src.is_empty() {
            return false;
        }
        let off = self.metrics.paste_offset * f64::from(steps);
        let mut ids = BTreeSet::new();
        let ops = src
            .iter()
            .map(|o| {
                let mut el = o.clone();
                el.id = doc.mint_id();
                el.version = 0;
                el.x += off;
                el.y += off;
                ids.insert(el.id);
                el
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|mut el| {
                el.z = doc.z_on_top();
                // `z_on_top` só avança quando a op entra: aplica já, para a seguinte ficar acima.
                let _ = BoardOp::Put(el.clone()).apply(doc);
                el
            })
            .collect::<Vec<_>>();
        history.record(ops.iter().rev().map(|el| BoardOp::Delete(el.id)).collect());
        self.selection = ids;
        true
    }
}
