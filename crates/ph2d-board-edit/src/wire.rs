//! **As setas no editor** (W2): criar arrastando (a forma por baixo realça-se e liga; `Ctrl` solta),
//! arrastar uma ponta de uma seta seleccionada, os pontos azuis que criam a forma seguinte já
//! ligada (e `Ctrl+seta`), e o que apagar, copiar e duplicar fazem às pontas presas.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, Connector, Element, ElementId, End, History, Style,
};
use ph2d_board_route::{Routed, fixed_world};

use crate::gesture::Gesture;
use crate::{Editor, Frame, NEXT_GAP, Pointer, Target, live};

/// Que ponta de que seta.
pub(crate) type EndRef = (ElementId, usize);

impl Gesture {
    /// O realce do alvo de uma ligação em curso.
    pub(crate) fn target(&self, doc: &BoardDoc) -> Option<Target> {
        let (id, anchor) = match self {
            Gesture::Connect { target, .. } | Gesture::EndDrag { target, .. } => (*target)?,
            _ => return None,
        };
        let el = doc.get(id)?;
        Some(Target {
            element: id,
            fixed: match anchor {
                Anchor::Center => None,
                Anchor::Fixed(uv) => Some(fixed_world(el, uv)),
            },
        })
    }
}

impl Editor {
    /// O estilo de uma seta nova: o actual, sem preenchimento, e com traço mesmo que a forma não o
    /// tenha (uma seta sem traço não se veria).
    pub(crate) fn arrow_style(&self) -> Style {
        let mut s = self.style.clone();
        s.fill = None;
        s.stroke = s.stroke.or(Some(s.text_color));
        s
    }

    /// A ponta que o ponteiro em `p` pede: presa à forma por baixo (realçada), ou solta com `Ctrl`
    /// (ou sobre o vazio). `skip` = formas que não contam.
    pub(crate) fn end_at(
        &mut self,
        doc: &BoardDoc,
        p: Pointer,
        skip: &[ElementId],
    ) -> (End, Option<(ElementId, Anchor)>) {
        if p.mods.ctrl {
            return (End::Free(p.world), None);
        }
        let band = self.metrics.bind * p.px;
        match self.routes.bind_at(doc, p.world, band, skip) {
            Some((target, anchor)) => (End::Bound { target, anchor }, Some((target, anchor))),
            None => (End::Free(p.world), None),
        }
    }

    /// Carregar com a ferramenta Seta (ou num ponto azul): a ponta de início prende-se já.
    pub(crate) fn begin_connect(
        &mut self,
        doc: &BoardDoc,
        p: Pointer,
        origin: Option<ElementId>,
        dot: Option<[f64; 2]>,
    ) -> Gesture {
        let (start, _) = match origin {
            Some(id) => (
                End::Bound {
                    target: id,
                    anchor: Anchor::Center,
                },
                None,
            ),
            None => self.end_at(doc, p, &[]),
        };
        let origin = origin.or(match start {
            End::Bound { target, .. } => Some(target),
            End::Free(_) => None,
        });
        Gesture::Connect {
            start,
            from: p.world,
            id: None,
            target: None,
            origin,
            dot,
        }
    }

    /// O arrasto de uma seta nova: a ponta de fim segue o ponteiro e prende-se ao que está por baixo.
    pub(crate) fn connect_move(&mut self, doc: &mut BoardDoc, g: &mut Gesture, p: Pointer) {
        let Gesture::Connect {
            start,
            from,
            id,
            target,
            origin,
            ..
        } = g
        else {
            return;
        };
        if id.is_none() && dist(*from, p.world) <= self.metrics.drag * p.px {
            return;
        }
        let skip: Vec<ElementId> = origin.iter().copied().collect();
        let (end, t) = self.end_at(doc, p, &skip);
        *target = t;
        let el = match *id {
            Some(existing) => doc.get(existing).cloned().map(|mut el| {
                if let Some(c) = el.connector_mut() {
                    c.end = end;
                }
                el
            }),
            None => {
                let mut c = Connector::new(*start, end, self.route, self.arrow_style());
                c.heads = self.heads;
                let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
                *id = Some(el.id);
                Some(el)
            }
        };
        if let Some(el) = el {
            live(doc, el);
        }
    }

    /// Largar uma seta nova. Sem arrastar: num ponto azul cria a forma seguinte; com a ferramenta,
    /// não faz nada (a seta nasce de um arrasto).
    pub(crate) fn connect_up(&mut self, doc: &mut BoardDoc, history: &mut History, g: Gesture) {
        let Gesture::Connect {
            id, origin, dot, ..
        } = g
        else {
            return;
        };
        match (id, dot, origin) {
            (Some(id), _, _) => {
                history.record(vec![BoardOp::Delete(id)]);
                self.selection = BTreeSet::from([id]);
                self.tool = crate::Tool::Select;
            }
            (None, Some(dir), Some(from)) => {
                self.grow(doc, history, from, dir);
            }
            _ => {}
        }
    }

    /// A ponta `which` de uma seta seleccionada sob `p` (a pega dela).
    pub(crate) fn end_handle_at(&mut self, doc: &BoardDoc, p: Pointer) -> Option<EndRef> {
        self.routes.sync(doc);
        let reach = self.metrics.handle * p.px;
        self.selection
            .iter()
            .filter(|id| doc.get(**id).and_then(Element::connector).is_some())
            .find_map(|id| {
                let r = self.routes.get(*id)?;
                (0..2)
                    .find(|&i| dist(r.ends()[i], p.world) <= reach)
                    .map(|i| (*id, i))
            })
    }

    /// Arrastar a ponta `which` da seta `id`: ela prende-se ao que está por baixo, nunca à forma da
    /// OUTRA ponta (um laço por acidente).
    pub(crate) fn end_drag_move(&mut self, doc: &mut BoardDoc, g: &mut Gesture, p: Pointer) {
        let Gesture::EndDrag {
            id, which, target, ..
        } = g
        else {
            return;
        };
        let Some(mut el) = doc.get(*id).cloned() else {
            return;
        };
        let Some(c) = el.connector() else {
            return;
        };
        let skip: Vec<ElementId> = match c.ends()[1 - *which] {
            End::Bound { target, .. } => vec![target],
            End::Free(_) => Vec::new(),
        };
        let (end, t) = self.end_at(doc, p, &skip);
        *target = t;
        if let Some(c) = el.connector_mut() {
            *c.end_mut(*which) = end;
        }
        live(doc, el);
    }

    /// ⭐ **Os pontos azuis**: a única forma seleccionada (ou a que está sob o ponteiro, com a
    /// ferramenta de seleccionar) mostra um ponto em cada lado — `(meio do lado, para fora)`.
    pub(crate) fn dots(&self, doc: &BoardDoc) -> Vec<([f64; 2], [f64; 2])> {
        if self.tool != crate::Tool::Select {
            return Vec::new();
        }
        let sel: Vec<&Element> = self.selected(doc);
        let el = match sel.as_slice() {
            [one] if one.shape().is_some() => *one,
            [] => match self.hover.and_then(|h| doc.get(h)) {
                Some(h) => h,
                None => return Vec::new(),
            },
            _ => return Vec::new(),
        };
        let f = Frame::of(el);
        [[0.0, -1.0], [1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]]
            .into_iter()
            .map(|[sx, sy]| {
                let at = f.point([sx * f.w / 2.0, sy * f.h / 2.0]);
                let c = f.point([0.0, 0.0]);
                let out = f.point([sx, sy]);
                (at, [out[0] - c[0], out[1] - c[1]])
            })
            .collect()
    }

    /// O ponto azul sob `p`: a forma e a direcção dele.
    pub(crate) fn dot_at(&self, doc: &BoardDoc, p: Pointer) -> Option<(ElementId, [f64; 2])> {
        let dots = self.dots(doc);
        let owner = match self.selected(doc).as_slice() {
            [one] => one.id,
            _ => self.hover?,
        };
        let reach = self.metrics.handle * p.px;
        dots.into_iter()
            .find(|(at, dir)| {
                let pos = [
                    at[0] + dir[0] * self.metrics.dot * p.px,
                    at[1] + dir[1] * self.metrics.dot * p.px,
                ];
                dist(pos, p.world) <= reach
            })
            .map(|(_, dir)| (owner, dir))
    }

    /// O ponteiro passeia (sem botão): a forma por baixo mostra os pontos azuis. Fica a mesma
    /// enquanto o ponteiro está na zona dos pontos dela — senão eles fugiam antes do clique.
    pub fn hover(&mut self, doc: &BoardDoc, p: Pointer) {
        if self.gesture.is_some() || self.tool != crate::Tool::Select {
            self.hover = None;
            return;
        }
        let keep = self.hover.and_then(|h| doc.get(h)).is_some_and(|el| {
            let f = Frame::of(el);
            let l = f.local(p.world);
            let pad = (self.metrics.dot + self.metrics.handle) * p.px;
            l[0].abs() <= f.w / 2.0 + pad && l[1].abs() <= f.h / 2.0 + pad
        });
        if !keep {
            self.hover = self
                .hit(doc, p)
                .filter(|id| doc.get(*id).is_some_and(|el| el.shape().is_some()));
        }
    }

    /// ⭐ **A forma seguinte, já ligada** — o ponto azul clicado e o `Ctrl+seta`: uma cópia da forma
    /// `from` (tipo, estilo, tamanho, rotação; sem texto) do lado `dir`, a [`NEXT_GAP`] dela —
    /// mais longe se o sítio está ocupado —, a seta entre as duas, e a nova seleccionada. UM passo.
    pub(crate) fn grow(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        from: ElementId,
        dir: [f64; 2],
    ) -> bool {
        let Some(src) = doc.get(from).filter(|el| el.shape().is_some()).cloned() else {
            return false;
        };
        let f = Frame::of(&src);
        let (s, c) = f.angle.sin_cos();
        let reach = (dir[0] * c + dir[1] * s).abs() * f.w / 2.0
            + (-dir[0] * s + dir[1] * c).abs() * f.h / 2.0;
        let step = 2.0 * reach + NEXT_GAP;
        let mut next = src.clone();
        next.id = doc.mint_id();
        next.z = doc.z_on_top();
        next.version = 0;
        if let Some(sh) = next.shape_mut() {
            sh.text.clear();
        }
        let free = |el: &Element, doc: &BoardDoc| {
            let b = el.aabb();
            !doc.live().filter(|o| o.shape().is_some()).any(|o| {
                let a = o.aabb();
                a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
            })
        };
        let shapes = doc.live().filter(|o| o.shape().is_some()).count();
        for k in 1..=shapes + 1 {
            next.x = src.x + dir[0] * step * k as f64;
            next.y = src.y + dir[1] * step * k as f64;
            if free(&next, doc) {
                break;
            }
        }
        let id = next.id;
        let center = |target| End::Bound {
            target,
            anchor: Anchor::Center,
        };
        let mut wire = Connector::new(center(from), center(id), self.route, self.arrow_style());
        wire.heads = self.heads;
        // `z_on_top` só avança quando a forma entra: a seta nasce depois, por cima dela.
        let _ = BoardOp::Put(next).apply(doc);
        let w = Element::new_connector(doc.mint_id(), doc.z_on_top(), wire);
        let _ = BoardOp::Put(w.clone()).apply(doc);
        history.record(vec![BoardOp::Delete(w.id), BoardOp::Delete(id)]);
        self.selection = BTreeSet::from([id]);
        true
    }

    /// As setas que ficam e se prendiam a uma forma de `gone`: as pontas soltam-se ONDE ESTÃO (as
    /// operações a juntar ao lote de apagar — desfazer devolve-as presas).
    pub(crate) fn release_ends(
        &mut self,
        doc: &BoardDoc,
        gone: &BTreeSet<ElementId>,
    ) -> Vec<BoardOp> {
        self.routes.sync(doc);
        doc.live()
            .filter(|el| !gone.contains(&el.id))
            .filter_map(|el| {
                let c = el.connector()?;
                let hits = c.targets().any(|t| gone.contains(&t));
                if !hits {
                    return None;
                }
                let r = self.routes.get(el.id)?;
                let mut el = el.clone();
                let c = el.connector_mut()?;
                for (i, at) in r.ends().into_iter().enumerate() {
                    if matches!(c.ends()[i], End::Bound { target, .. } if gone.contains(&target)) {
                        *c.end_mut(i) = End::Free(at);
                    }
                }
                Some(BoardOp::Put(el))
            })
            .collect()
    }

    /// Antes de copiar `els`: as pontas presas a formas que NÃO vão na cópia soltam-se onde estão
    /// (a cópia de uma seta sozinha não fica presa às formas do original).
    pub(crate) fn detach_outside(&mut self, doc: &BoardDoc, els: &mut [Element]) {
        self.routes.sync(doc);
        let inside: BTreeSet<ElementId> = els.iter().map(|e| e.id).collect();
        for el in els.iter_mut() {
            let id = el.id;
            let Some(c) = el.connector_mut() else {
                continue;
            };
            let ends = self.routes.get(id).map(Routed::ends);
            for i in 0..2 {
                if let End::Bound { target, .. } = c.ends()[i]
                    && !inside.contains(&target)
                {
                    let at = ends.map_or([0.0, 0.0], |e| e[i]);
                    *c.end_mut(i) = End::Free(at);
                }
            }
        }
    }
}

/// As cópias de setas passam a prender-se às CÓPIAS das formas (`map`: id original → id da cópia).
pub(crate) fn remap(copies: &mut [Element], map: &BTreeMap<ElementId, ElementId>) {
    for el in copies {
        let Some(c) = el.connector_mut() else {
            continue;
        };
        for i in 0..2 {
            if let End::Bound { target, anchor } = c.ends()[i]
                && let Some(&t) = map.get(&target)
            {
                *c.end_mut(i) = End::Bound { target: t, anchor };
            }
        }
    }
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
