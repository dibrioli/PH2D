//! **Os cliques da facilitação** (W7): o `+`/`−` de votar (a sessão, `vote.rs`) e as fichas dos
//! carimbos (`docs/MiroClone/05_facilitacao.md` §2 V2, S2–S3). Ganham ao clique da selecção: estão
//! pintados por cima de tudo.

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History, Stamp};

use crate::{Editor, Pointer};

/// V2 — o `−` de tirar um voto fica à esquerda do `+`, a esta fracção do Ø (centro a centro).
pub const VOTE_MINUS_STEP: f64 = 1.15;

/// Os círculos de votar de um objecto em sessão, no MUNDO: o `+`, o `−` (só com votos seus) e os
/// votos da pessoa ao ecrã nele.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoteMark {
    pub id: ElementId,
    pub plus: ([f64; 2], f64),
    pub minus: Option<([f64; 2], f64)>,
    pub mine: u32,
}

/// O ponto `local` do referencial da caixa de `el` no mundo.
fn world(el: &Element, local: [f64; 2]) -> [f64; 2] {
    el.rotate([el.x + local[0], el.y + local[1]])
}

/// `p` (mundo) no referencial da caixa de `el`.
fn local(el: &Element, p: [f64; 2]) -> [f64; 2] {
    let q = el.unrotate(p);
    [q[0] - el.x, q[1] - el.y]
}

fn inside(p: [f64; 2], (c, d): ([f64; 2], f64)) -> bool {
    (p[0] - c[0]).hypot(p[1] - c[1]) <= d / 2.0
}

impl Editor {
    /// V2 — os círculos de votar de cada votável da sessão, de trás para a frente.
    #[must_use]
    pub fn vote_marks(&self, doc: &BoardDoc) -> Vec<VoteMark> {
        let Some(s) = &self.vote else {
            return Vec::new();
        };
        // Um nó escondido por um ramo recolhido não mostra o `+` (nem o aceita): volta ao expandir.
        // W12 — nem uma nota cujo texto não leio (volta ao revelar), salvo se já lá tenho votos: o
        // `−` tira-os (um voto novo nela é recusado).
        let mut els: Vec<&Element> = s
            .targets()
            .iter()
            .filter(|id| !self.unread.contains(id) || s.mine(**id) > 0)
            .filter_map(|id| doc.get(*id))
            .filter(|e| !e.is_hidden())
            .collect();
        els.sort_by(|a, b| a.z.cmp(&b.z));
        els.into_iter()
            .map(|el| {
                let (c, d) = ph2d_board_model::vote_plus(el);
                let mine = s.mine(el.id);
                let minus = (mine > 0).then(|| {
                    let m = [c[0] - d * VOTE_MINUS_STEP, c[1]];
                    (world(el, m), d)
                });
                VoteMark {
                    id: el.id,
                    plus: (world(el, c), d),
                    minus,
                    mine,
                }
            })
            .collect()
    }

    /// S2 — um carimbo `s` em cada nota seleccionada (as travadas não): UM passo.
    pub fn stamp(&mut self, doc: &mut BoardDoc, history: &mut History, s: Stamp) -> bool {
        let ops: Vec<BoardOp> = self
            .editable(doc)
            .into_iter()
            .filter(|el| el.shape().is_some_and(|sh| sh.kind.is_note()))
            .map(|el| {
                let mut el = el.clone();
                el.add_stamp(s, 1);
                BoardOp::Put(el)
            })
            .collect();
        let any = !ops.is_empty();
        history.apply(doc, ops);
        any
    }

    /// O clique da facilitação em `p`: o `−`/`+` de votar, depois a ficha de um carimbo (`Shift` tira
    /// um). `true` = tomou o clique.
    pub(crate) fn facilitate_down(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        p: Pointer,
    ) -> bool {
        let marks = self.vote_marks(doc);
        for m in marks.iter().rev() {
            if m.minus.is_some_and(|c| inside(p.world, c)) {
                self.vote_click(m.id, true);
                return true;
            }
            if inside(p.world, m.plus) {
                self.vote_click(m.id, false);
                return true;
            }
        }
        let Some((id, stamp)) = chip_at(doc, p.world) else {
            return false;
        };
        let Some(mut el) = doc.get(id).cloned() else {
            return false;
        };
        if el.locked() {
            self.notice = Some(crate::Notice::Locked);
            return true;
        }
        el.add_stamp(stamp, if p.mods.shift { -1 } else { 1 });
        history.apply(doc, vec![BoardOp::Put(el)]);
        true
    }
}

/// S3 — a ficha de carimbo debaixo de `p` (a do elemento de cima).
#[must_use]
pub fn chip_at(doc: &BoardDoc, p: [f64; 2]) -> Option<(ElementId, Stamp)> {
    let mut els: Vec<&Element> = doc.live().filter(|e| !e.stamps().is_empty()).collect();
    els.sort_by(|a, b| b.z.cmp(&a.z));
    els.into_iter().find_map(|el| {
        let q = local(el, p);
        ph2d_board_model::stamp_chips(el)
            .into_iter()
            .find(|(_, r)| q[0] >= r[0] && q[0] <= r[2] && q[1] >= r[1] && q[1] <= r[3])
            .map(|(c, _)| (el.id, c.stamp))
    })
}
