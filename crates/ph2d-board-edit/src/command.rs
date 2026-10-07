//! As ordens do teclado fora do texto: apagar, duplicar, seleccionar tudo, sair, empurrar, desfazer,
//! copiar/cortar/colar, começar a escrever, trocar de ferramenta.

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, History, ShapeType};
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
                // A nota nasce da forma escolhida por último (quadrada ou larga).
                self.tool = match t {
                    Tool::Shape(ShapeType::Sticky | ShapeType::StickyWide) => {
                        Tool::Shape(self.notes.kind())
                    }
                    t => t,
                };
                true
            }
            Command::Mark(m) => self.toggle_mark(doc, history, m),
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
                // As setas que ficam e se prendiam ao que sai soltam-se onde estão (no MESMO passo).
                let gone = self.selection.clone();
                let mut ops = self.release_ends(doc, &gone);
                ops.extend(gone.iter().map(|id| BoardOp::Delete(*id)));
                self.selection.clear();
                let any = !gone.is_empty();
                history.apply(doc, ops);
                any
            }
            Command::Grow(dir) => {
                let [id] = self.selection.iter().copied().collect::<Vec<_>>()[..] else {
                    return false;
                };
                self.grow(doc, history, id, dir)
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
                        el.translate(d);
                        BoardOp::Put(el)
                    })
                    .collect();
                let any = !ops.is_empty();
                history.apply(doc, ops);
                any
            }
            Command::Duplicate => {
                let mut src: Vec<Element> = self.selected(doc).into_iter().cloned().collect();
                self.detach_outside(doc, &mut src);
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
        let mut els: Vec<Element> = self.selected(doc).into_iter().cloned().collect();
        self.detach_outside(doc, &mut els);
        // O texto deles (um por linha) vai também para fora do app — e é a marca que separa um
        // `Ctrl+V` destes elementos de um de fora.
        let text: Vec<&str> = els
            .iter()
            .filter_map(|el| match (el.shape(), el.connector()) {
                (Some(s), _) => Some(s.text.as_str()),
                (_, Some(c)) => Some(c.label.as_str()),
                _ => None,
            })
            .filter(|t| !t.is_empty())
            .collect();
        self.copied_text = Some(text.join("\n"));
        self.clipboard = els;
        self.pastes = 0;
    }

    /// O texto que a última cópia de elementos deu à área de transferência do sistema.
    #[must_use]
    pub fn copied_text(&self) -> Option<&str> {
        self.copied_text.as_deref()
    }

    /// ⭐ Um `Ctrl+V` com `system` (o texto da área de transferência do sistema) é uma colagem de
    /// FORA (uma planilha, um texto) — e não os elementos copiados no quadro? É, se há texto e não é
    /// o que a última cópia lá pôs (ou se não há elementos copiados).
    #[must_use]
    pub fn is_foreign_paste(&self, system: &str) -> bool {
        !system.trim().is_empty()
            && (self.clipboard.is_empty() || self.copied_text.as_deref() != Some(system))
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
        let mut map = std::collections::BTreeMap::new();
        let mut copies: Vec<Element> = src
            .iter()
            .map(|o| {
                let mut el = o.clone();
                el.id = doc.mint_id();
                map.insert(o.id, el.id);
                el.version = 0;
                el.translate([off, off]);
                ids.insert(el.id);
                el
            })
            .collect();
        crate::wire::remap(&mut copies, &map);
        let ops = copies
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
