//! As ordens do teclado fora do texto: apagar, duplicar, seleccionar tudo, sair, empurrar, desfazer,
//! copiar/cortar/colar, começar a escrever, trocar de ferramenta.

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History, ShapeType};
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
                if std::mem::take(&mut self.linking) {
                    return true;
                }
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
                // Os travados ficam de fora (o Miro e o oráculo E).
                self.selection = doc
                    .live()
                    .filter(|el| !el.locked())
                    .map(|el| el.id)
                    .collect();
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
                // Os travados ficam; um frame leva tudo o que vive dentro dele (menos o travado, que
                // fica solto onde está).
                let free: Vec<ElementId> = self.movable(doc).iter().map(|el| el.id).collect();
                if free.len() < self.selection.len() {
                    self.notice = Some(crate::Notice::Locked);
                }
                let gone: BTreeSet<ElementId> = crate::organize::with_descendants(doc, &free)
                    .into_iter()
                    .filter(|id| doc.get(*id).is_some_and(|el| !el.locked()))
                    .collect();
                let mut ops = self.release_ends(doc, &gone);
                ops.extend(gone.iter().map(|id| BoardOp::Delete(*id)));
                // M9: apagar um nó de mapa passa a selecção ao pai — se saiu alguma coisa.
                let parent = self.mind_focus(doc).and_then(|n| doc.mind_parent(n));
                let any = !gone.is_empty();
                if any {
                    self.selection = parent.into_iter().collect();
                }
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
                // Um nó que o layout arruma não se empurra (o layout devolvia-o no mesmo passo e
                // ficava um passo de desfazer vazio — auditoria W6).
                let free: Vec<ElementId> = self
                    .movable(doc)
                    .iter()
                    .filter(|el| !crate::mindmap::laid(doc, el.id))
                    .map(|el| el.id)
                    .collect();
                let ops: Vec<BoardOp> = crate::organize::with_descendants(doc, &free)
                    .into_iter()
                    .filter_map(|id| doc.get(id))
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
                let mut src = self.selected_with_contents(doc);
                self.detach_outside(doc, &mut src);
                self.place_copies(doc, history, &src, 1)
            }
            Command::Group
            | Command::Ungroup
            | Command::Lock
            | Command::Order(_)
            | Command::Align(_)
            | Command::Distribute(_)
            | Command::Link
            | Command::MindNav(_)
            | Command::MindCollapse
            | Command::MindAuto
            | Command::MindVertical => self.organize(doc, history, c),
            Command::MindChild => self.mind_child(doc, history, ts, None),
            Command::MindSibling => self.mind_sibling(doc, history, ts),
            Command::Paste => {
                self.pastes += 1;
                let src = self.clipboard.clone();
                self.place_copies(doc, history, &src, self.pastes)
            }
        }
    }

    fn copy(&mut self, doc: &BoardDoc) {
        let mut els = self.selected_with_contents(doc);
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
                // Uma cópia treme de outra maneira (a semente importada fica no original, W9).
                el.set_imported_seed(None);
                el.translate([off, off]);
                ids.insert(el.id);
                el
            })
            .collect();
        crate::wire::remap(&mut copies, &map);
        let roots = remap_organize(doc, &mut copies, &map);
        self.unfitted.extend(roots);
        let ops = copies
            .into_iter()
            .map(|mut el| {
                el.z = doc.z_on_top();
                // `z_on_top` só avança quando a op entra: aplica já, para a seguinte ficar acima.
                let _ = BoardOp::Put(el.clone()).apply(doc);
                el
            })
            .collect::<Vec<_>>();
        history.record(
            ops.iter().rev().map(|el| BoardOp::Delete(el.id)).collect(),
            doc,
        );
        for el in &ops {
            crate::mindmap::settle_live(doc, el.id);
        }
        // Os escondidos de um ramo recolhido copiado não ficam seleccionados (auditoria W6).
        self.selection = ids
            .into_iter()
            .filter(|id| doc.get(*id).is_some_and(|e| !e.is_hidden()))
            .collect();
        true
    }
}

impl Editor {
    /// As ordens de ORGANIZAR (W5) — sem o moldador de texto, o que um botão da barra pode pedir.
    /// `false` = não fez nada (ou não é uma delas).
    pub fn organize(&mut self, doc: &mut BoardDoc, history: &mut History, c: Command) -> bool {
        match c {
            Command::Group => self.group(doc, history),
            Command::Ungroup => self.ungroup(doc, history),
            Command::Lock => self.toggle_lock(doc, history),
            Command::Order(o) => self.reorder(doc, history, o),
            Command::Align(a) => self.align(doc, history, a),
            Command::Distribute(ax) => self.distribute(doc, history, ax),
            Command::MindNav(d) => self.mind_nav(doc, d),
            Command::MindCollapse => self.mind_collapse(doc, history, None),
            Command::MindAuto => self.mind_toggle(doc, history, false),
            Command::MindVertical => self.mind_toggle(doc, history, true),
            Command::Link => self.link_command(doc, history),
            _ => false,
        }
    }

    /// Os seleccionados e o que vive dentro dos frames entre eles, em ordem de z (o que uma cópia
    /// leva: copiar um frame copia o conteúdo).
    fn selected_with_contents(&self, doc: &BoardDoc) -> Vec<Element> {
        let ids: Vec<ElementId> = self.selection.iter().copied().collect();
        let mut v: Vec<Element> = crate::organize::with_descendants(doc, &ids)
            .into_iter()
            .filter_map(|id| doc.get(id).cloned())
            .collect();
        v.sort_by(|a, b| a.z.cmp(&b.z));
        v
    }
}

/// As cópias ficam nos frames COPIADOS (ou na casa do original, se o frame dele não veio) e em
/// grupos NOVOS — uma cópia de um grupo é outro grupo; o link para um copiado aponta a cópia.
pub(crate) fn remap_organize(
    doc: &mut BoardDoc,
    copies: &mut [Element],
    map: &std::collections::BTreeMap<ElementId, ElementId>,
) -> Vec<ElementId> {
    let mut groups = std::collections::BTreeMap::new();
    let mut roots = Vec::new();
    for el in copies.iter_mut() {
        if let Some(f) = el.frame.and_then(|f| map.get(&f)) {
            el.frame = Some(*f);
        }
        if let Some(l) = el.link().and_then(|l| map.get(&l)) {
            el.set_link(Some(*l));
        }
        // Um nó copiado com o pai fica debaixo da cópia dele; sem o pai, é irmão do original — LOGO
        // DEPOIS dele, com chave própria (duas chaves iguais faziam o `Enter` seguinte entrar em
        // pânico, auditoria W6); e se o pai já não existe, a cópia é a raiz de um mapa novo.
        if crate::mindmap::rehome_copy(doc, el, map) {
            roots.push(el.id);
        }
        for g in el.groups_mut() {
            *g = *groups.entry(*g).or_insert_with(|| doc.mint_group());
        }
    }
    roots
}
