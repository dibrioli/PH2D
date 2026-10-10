//! **Arrastar e largar um nó de mapa** (W6, M12) e os carregares que são do mapa (a ferramenta, o
//! «+», o selo): módulo irmão de [`crate::mindmap`], cortado dele pelo tecto de LOC.

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, FracKey, History, Side};
use ph2d_text::TextSystem;

use crate::mindmap::{across, order_of, settle_live};
use crate::{Editor, Pointer, Tool, live};

/// O nó arrumado pelo layout que um arrasto leva: o primeiro cujo pai fica (e tem auto layout).
pub(crate) fn lead(doc: &BoardDoc, moved: &BTreeSet<ElementId>) -> Option<ElementId> {
    moved.iter().copied().find(|id| {
        doc.mind_parent(*id).is_some_and(|p| {
            !moved.contains(&p)
                && doc
                    .get(p)
                    .and_then(Element::mind)
                    .is_some_and(|m| !m.manual)
        })
    })
}

/// O nó arrumado que o arrasto leva SÓ com o ramo dele (nada mais seleccionado): uma selecção
/// mista move-se como se move tudo, e o layout devolve os nós dele ao sítio (auditoria W6: a nota
/// que ia junto voltava para trás).
pub(crate) fn branch_lead(doc: &BoardDoc, moved: &BTreeSet<ElementId>) -> Option<ElementId> {
    let l = lead(doc, moved)?;
    let mut branch = doc.mind_descendants(l);
    branch.insert(l);
    moved.iter().all(|id| branch.contains(id)).then_some(l)
}

/// O nó (de qualquer mapa, à vista, fora de `moved`) onde largar faz de pai: o de cima sob `p`.
pub(crate) fn drop_target(
    doc: &BoardDoc,
    p: [f64; 2],
    moved: &BTreeSet<ElementId>,
) -> Option<ElementId> {
    doc.live_in_z_order()
        .into_iter()
        .rev()
        .find(|el| el.mind().is_some() && !moved.contains(&el.id) && el.box_contains(p))
        .map(|el| el.id)
}

impl Editor {
    /// Um carregar que é do mapa: a ferramenta «Mind map» põe uma raiz; com a selecção, o «+» cria o
    /// filho desse lado e o selo de um ramo recolhido expande-o. `true` = tomado.
    pub(crate) fn mind_pointer_down(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        p: Pointer,
    ) -> bool {
        match self.tool {
            Tool::MindMap => {
                self.mind_new_map(doc, history, ts, p.world);
                true
            }
            Tool::Select => {
                // O selo primeiro, e ele SELECCIONA o nó: o `Ctrl+↑` seguinte volta a fechar ESTE
                // ramo (a lição 4 da cena fechava o mapa inteiro — auditoria W6).
                if let Some(id) = self.mind_badge_at(doc, p.world) {
                    self.mind_collapse(doc, history, Some(id));
                    self.selection = BTreeSet::from([id]);
                    return true;
                }
                match self.mind_plus_at(doc, p) {
                    Some(side) => self.mind_child(doc, history, ts, Some(side)),
                    None => false,
                }
            }
            _ => false,
        }
    }

    /// O nó arrumado pelo layout que o arrasto leva (as cópias de um `Alt`, ou os originais).
    pub(crate) fn mind_lead_of(
        &self,
        doc: &BoardDoc,
        originals: &[Element],
        copies: &[ElementId],
    ) -> Option<ElementId> {
        let moved: BTreeSet<ElementId> = if copies.is_empty() {
            originals.iter().map(|o| o.id).collect()
        } else {
            copies.iter().copied().collect()
        };
        branch_lead(doc, &moved)
    }

    /// A meio do arrasto: o nó onde largar faria de pai (o realce). `true` = o arrasto leva um nó
    /// arrumado pelo layout — as guias calam-se (M12: quem decide o sítio é o layout).
    pub(crate) fn mind_drag_move(
        &mut self,
        doc: &BoardDoc,
        originals: &[Element],
        copies: &[ElementId],
        p: [f64; 2],
    ) -> bool {
        let moved: BTreeSet<ElementId> = if copies.is_empty() {
            originals.iter().map(|o| o.id).collect()
        } else {
            copies.iter().copied().collect()
        };
        let laid = branch_lead(doc, &moved).is_some();
        self.mind_drop = laid.then(|| drop_target(doc, p, &moved)).flatten();
        laid
    }

    /// ⭐ M12: largar o nó `lead` (e o ramo dele) — sobre outro nó, passa a ser o último filho dele
    /// (e toma a cor do ramo de lá); no vazio, muda de lugar entre os irmãos pela posição (e de lado,
    /// se cruzou a raiz). `Ctrl` = só reordena. O arrasto ao vivo desfaz-se e a mudança vira UM passo
    /// (o layout põe o ramo no sítio novo).
    pub(crate) fn mind_drop(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        lead: ElementId,
        originals: &[Element],
        copies: &[ElementId],
        p: Pointer,
    ) {
        self.mind_drop = None;
        let moved: BTreeSet<ElementId> = if copies.is_empty() {
            originals.iter().map(|o| o.id).collect()
        } else {
            copies.iter().copied().collect()
        };
        let target = if p.mods.ctrl {
            None
        } else {
            drop_target(doc, p.world, &moved)
        };
        let Some(now) = doc.get(lead).cloned() else {
            return;
        };
        let Some(mut m) = now.mind().cloned() else {
            return;
        };
        let centre = now.center();
        let parent = target.or(m.parent);
        let Some(parent) = parent else {
            return;
        };
        let root_parent = doc.mind_parent(parent).is_none();
        let (vertical, _) = ph2d_board_model::outward(doc, parent);
        if root_parent && let Some(r) = doc.get(parent) {
            let [rc, c] = [r.center(), centre];
            let k = usize::from(vertical);
            m.side = if c[k] < rc[k] {
                Side::Before
            } else {
                Side::After
            };
        }
        let siblings: Vec<ElementId> = doc
            .mind_children(parent)
            .into_iter()
            .filter(|c| !moved.contains(c))
            .filter(|c| {
                !root_parent
                    || doc
                        .get(*c)
                        .and_then(Element::mind)
                        .is_some_and(|cm| cm.side == m.side)
            })
            .collect();
        let (prev, next) = if target.is_some() {
            (siblings.last().copied(), None)
        } else {
            let at = across(&now, vertical);
            let i = siblings
                .iter()
                .take_while(|c| doc.get(**c).is_some_and(|e| across(e, vertical) < at))
                .count();
            (
                i.checked_sub(1).map(|j| siblings[j]),
                siblings.get(i).copied(),
            )
        };
        let lo = order_of(doc, prev.as_ref());
        let hi = order_of(doc, next.as_ref()).filter(|h| lo.as_ref().is_none_or(|l| h > l));
        m.order = doc.key_between(lo.as_ref(), hi.as_ref());
        let moved_home = m.parent != Some(parent);
        m.parent = Some(parent);
        // A cor do ramo: debaixo de um nó, a dele; na raiz (vindo de outro pai), a seguinte da paleta.
        let recolor = if !moved_home {
            None
        } else if root_parent {
            Some(crate::mindmap::next_color(doc, parent))
        } else {
            doc.get(parent).and_then(Element::mind).map(|pm| pm.color)
        };
        // M18: o ramo muda para a casa do pai novo.
        let home = doc.get(parent).map(|pe| pe.frame);
        // Um pai recolhido abre-se (como no `Tab`): largado lá dentro, o ramo não some.
        let open = target
            .and_then(|t| doc.get(t))
            .filter(|t| t.mind().is_some_and(|tm| tm.collapsed))
            .cloned()
            .map(|mut t| {
                if let Some(tm) = t.mind_mut() {
                    tm.collapsed = false;
                }
                t
            });
        if copies.is_empty() {
            for o in originals {
                live(doc, o.clone());
            }
        }
        let base = doc.get(lead).cloned().unwrap_or(now);
        let mut el = base;
        if let Some(c) = recolor {
            m.color = c;
        }
        el.set_mind(Some(m));
        if moved_home && let Some(h) = home {
            el.frame = h;
        }
        let mut ops: Vec<BoardOp> = open.into_iter().map(BoardOp::Put).collect();
        ops.push(BoardOp::Put(el));
        if moved_home {
            for d in doc.mind_descendants(lead) {
                if let Some(mut e) = doc.get(d).cloned() {
                    if let (Some(c), Some(dm)) = (recolor, e.mind_mut()) {
                        dm.color = c;
                    }
                    if let Some(h) = home {
                        e.frame = h;
                    }
                    ops.push(BoardOp::Put(e));
                }
            }
        }
        if copies.is_empty() {
            history.apply(doc, ops);
        } else {
            for op in ops {
                let _ = op.apply(doc);
            }
            settle_live(doc, lead);
            history.record(
                copies.iter().rev().map(|c| BoardOp::Delete(*c)).collect(),
                doc,
            );
        }
    }
}
