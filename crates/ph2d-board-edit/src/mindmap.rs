//! **O mapa mental no editor** (W6): criar o mapa, `Tab` (filho) e `Enter` (irmão), as setas que
//! andam pela árvore, recolher, o «+» e o largar que muda de pai. O layout é do documento
//! (`ph2d_board_model::mind`, M17); aqui vive o gesto. Leis: `docs/MiroClone/04_planear_mapa_e_modelos.md` §2.

use std::collections::BTreeSet;

use ph2d_board_layout::TextEdit;
use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, FracKey, History, MIND_BRANCH_WIDTH,
    MIND_COLORS, MIND_FONT, MIND_PAD, MIND_ROOT_PAD_Y, MIND_ROOT_STROKE, Mind, Rgba, RichText,
    Shape, ShapeType, Side, Style,
};
use ph2d_text::TextSystem;

use crate::{Editor, Pointer, Tool, live};

/// A largura do texto de um nó VAZIO, em letras (o Miro mostra «Type something» até se escrever).
const EMPTY_EMS: f64 = 4.0;
/// A largura de quebra ao medir um nó: nenhuma — a linha nova é do `Shift+Enter` (M15).
const NO_WRAP: f32 = 1.0e7;

/// O tamanho `[w, h]` do texto de `el` sem quebra (um nó vazio: [`EMPTY_EMS`] letras de largo, uma
/// linha de alto).
pub(crate) fn text_size(ts: &mut TextSystem, el: &Element) -> [f64; 2] {
    let Some(s) = el.shape() else {
        return [el.w, el.h];
    };
    let font = s.style.font_size;
    let hand = crate::text::hand_lettered(el);
    let mut measure = |t: &RichText| {
        let mut e = TextEdit::new(t, font as f32, NO_WRAP).hand(hand);
        let l = e.layout(ts);
        [f64::from(l.width()), f64::from(l.height())]
    };
    if s.text.is_empty() {
        [EMPTY_EMS * font, measure(&RichText::plain("M"))[1]]
    } else {
        measure(&s.text)
    }
}

/// ⭐ M15: o tamanho de um nó para o texto `[tw, th]` — a raiz é uma pílula de 66 de alto com uma
/// linha (meia altura de cada lado do texto), os outros o texto com [`MIND_PAD`]; nunca menos do que
/// a forma precisa para o texto caber sem quebrar.
#[must_use]
pub fn node_size(el: &Element, [tw, th]: [f64; 2]) -> [f64; 2] {
    let Some(s) = el.shape() else {
        return [el.w, el.h];
    };
    let k = s.style.font_size / MIND_FONT;
    let root = el.mind().is_some_and(|m| m.parent.is_none());
    let pad_y = if root { MIND_ROOT_PAD_Y } else { MIND_PAD[1] } * k;
    let h = ph2d_board_geom::height_for_text(s.kind, th + 2.0 * pad_y, th);
    let w = if root { tw + h } else { tw + 2.0 * MIND_PAD[0] * k };
    [w.max(ph2d_board_geom::width_for_text(s.kind, tw)), h]
}

/// `el` com o tamanho que o texto pede; a raiz cresce à volta do centro (o mapa parte dela), os outros
/// o layout recoloca.
pub(crate) fn refit(ts: &mut TextSystem, el: &Element) -> Element {
    let [w, h] = node_size(el, text_size(ts, el));
    let mut out = el.clone();
    if el.mind().is_some_and(|m| m.parent.is_none()) {
        out.x -= (w - el.w) / 2.0;
        out.y -= (h - el.h) / 2.0;
    }
    out.w = w;
    out.h = h;
    out
}

/// Arruma AO VIVO o mapa de `id` (durante a escrita: o passo de desfazer guarda-se no fim, e desfazer
/// volta a arrumar — o layout é função do documento).
pub(crate) fn settle_live(doc: &mut BoardDoc, id: ElementId) {
    if let Some(r) = doc.mind_root(id) {
        let _ = ph2d_board_model::mind_settle(doc, &BTreeSet::from([r]));
    }
}

/// O nó arrumado pelo layout que um arrasto leva: o primeiro cujo pai fica (e tem auto layout).
pub(crate) fn lead(doc: &BoardDoc, moved: &BTreeSet<ElementId>) -> Option<ElementId> {
    moved.iter().copied().find(|id| {
        doc.mind_parent(*id).is_some_and(|p| {
            !moved.contains(&p) && doc.get(p).and_then(Element::mind).is_some_and(|m| !m.manual)
        })
    })
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

/// O centro no eixo das FAIXAS (y no horizontal, x no vertical).
fn across(el: &Element, vertical: bool) -> f64 {
    el.center()[usize::from(!vertical)]
}

/// O lado da raiz `root` mais LEVE (a extensão das faixas de cada lado); empate à direita (M5).
fn lighter_side(doc: &BoardDoc, root: ElementId) -> Side {
    let (vertical, _) = ph2d_board_model::outward(doc, root);
    let mut ext = [(f64::INFINITY, f64::NEG_INFINITY); 2];
    for id in doc.mind_descendants(root) {
        let Some(el) = doc.get(id) else {
            continue;
        };
        let (a, b) = if vertical {
            (el.x, el.x + el.w)
        } else {
            (el.y, el.y + el.h)
        };
        let e = &mut ext[usize::from(doc.mind_side(id) == Side::Before)];
        *e = (e.0.min(a), e.1.max(b));
    }
    let len = |(a, b): (f64, f64)| if a.is_finite() { b - a } else { 0.0 };
    if len(ext[1]) < len(ext[0]) {
        Side::Before
    } else {
        Side::After
    }
}

fn order_of(doc: &BoardDoc, id: Option<&ElementId>) -> Option<FracKey> {
    id.and_then(|c| doc.get(*c))
        .and_then(Element::mind)
        .map(|m| m.order.clone())
}

impl Editor {
    /// O nó de mapa em foco: o que se escreve, ou o único seleccionado.
    #[must_use]
    pub fn mind_focus(&self, doc: &BoardDoc) -> Option<ElementId> {
        let id = match &self.editing {
            Some(e) => e.id,
            None => match self.selection.iter().copied().collect::<Vec<_>>()[..] {
                [one] => one,
                _ => return None,
            },
        };
        doc.get(id)?.mind()?;
        Some(id)
    }

    /// A forma de um nó novo: a raiz uma pílula com o contorno lilás, os outros só texto (M15).
    fn mind_shape(&self, root: bool) -> Shape {
        let stroke = root.then_some(Rgba(MIND_ROOT_STROKE));
        let mut style = Style::new(None, stroke, self.style.text_color);
        style.font_size = MIND_FONT;
        style.stroke_width = MIND_BRANCH_WIDTH;
        Shape {
            kind: if root {
                ShapeType::Pill
            } else {
                ShapeType::Rectangle
            },
            style,
            text: RichText::default(),
        }
    }

    /// Um nó novo (do tamanho do texto vazio) centrado em `at`, à frente de tudo.
    pub(crate) fn mind_node(
        &self,
        doc: &mut BoardDoc,
        ts: &mut TextSystem,
        mind: Mind,
        at: [f64; 2],
        text: &str,
    ) -> Element {
        let mut shape = self.mind_shape(mind.parent.is_none());
        shape.text = RichText::plain(text);
        let mut el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [at[0], at[1], 0.0, 0.0]);
        el.set_mind(Some(mind));
        let [w, h] = node_size(&el, text_size(ts, &el));
        (el.x, el.y, el.w, el.h) = (at[0] - w / 2.0, at[1] - h / 2.0, w, h);
        el
    }

    /// Um mapa novo com a raiz em `c`, já a escrever (a ferramenta «Mind map»). UM passo.
    pub fn mind_new_map(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        c: [f64; 2],
    ) -> ElementId {
        self.commit_text(doc, history);
        let m = Mind::new(None, FracKey::between(None, None), Side::After, Rgba(MIND_ROOT_STROKE));
        let el = self.mind_node(doc, ts, m, c, "");
        let id = el.id;
        history.apply(doc, vec![BoardOp::Put(el)]);
        self.tool = Tool::Select;
        self.selection = BTreeSet::from([id]);
        self.begin_text(doc, ts, id, None);
        id
    }

    /// ⭐ M6 (`Tab`): o filho ÚLTIMO do nó em foco, seleccionado e a escrever. Na raiz nasce no lado
    /// `side` (o «+») ou no mais leve (M5); a cor é a seguinte da paleta na raiz, a do pai abaixo
    /// (M14). Um pai recolhido abre-se. UM passo.
    pub fn mind_child(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        side: Option<Side>,
    ) -> bool {
        let Some(parent) = self.mind_focus(doc) else {
            return false;
        };
        self.commit_text(doc, history);
        let Some(p) = doc.get(parent).cloned() else {
            return false;
        };
        let Some(pm) = p.mind().cloned() else {
            return false;
        };
        let kids = doc.mind_children(parent);
        let order = FracKey::between(order_of(doc, kids.last()).as_ref(), None);
        let is_root = doc.mind_parent(parent).is_none();
        let (side, color) = if is_root {
            let s = side.unwrap_or_else(|| lighter_side(doc, parent));
            (s, Rgba(MIND_COLORS[kids.len() % MIND_COLORS.len()]))
        } else {
            (doc.mind_side(parent), pm.color)
        };
        let node = self.mind_node(doc, ts, Mind::new(Some(parent), order, side, color), p.center(), "");
        let id = node.id;
        let mut ops = Vec::new();
        if pm.collapsed {
            let mut open = p.clone();
            if let Some(m) = open.mind_mut() {
                m.collapsed = false;
            }
            ops.push(BoardOp::Put(open));
        }
        ops.push(BoardOp::Put(node));
        history.apply(doc, ops);
        self.selection = BTreeSet::from([id]);
        self.begin_text(doc, ts, id, None);
        true
    }

    /// ⭐ M7 (`Enter` a escrever): o irmão LOGO DEPOIS do nó em foco, a escrever; na raiz, um filho.
    /// UM passo.
    pub fn mind_sibling(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
    ) -> bool {
        let Some(id) = self.mind_focus(doc) else {
            return false;
        };
        let Some(parent) = doc.mind_parent(id) else {
            return self.mind_child(doc, history, ts, None);
        };
        self.commit_text(doc, history);
        let Some(el) = doc.get(id).cloned() else {
            return false;
        };
        let Some(m) = el.mind().cloned() else {
            return false;
        };
        let kids = doc.mind_children(parent);
        let next = kids.iter().position(|c| *c == id).and_then(|i| kids.get(i + 1));
        let order = FracKey::between(Some(&m.order), order_of(doc, next).as_ref());
        let node =
            self.mind_node(doc, ts, Mind::new(Some(parent), order, m.side, m.color), el.center(), "");
        let nid = node.id;
        history.apply(doc, vec![BoardOp::Put(node)]);
        self.selection = BTreeSet::from([nid]);
        self.begin_text(doc, ts, nid, None);
        true
    }

    /// ⭐ M8: uma seta (seleccionado, não a escrever) anda pela árvore — para dentro o pai, para fora
    /// o filho mais perto do meio (na raiz, do lado da seta), ao longo das faixas o vizinho à vista
    /// à MESMA profundidade e do mesmo lado. `dir` = `(dx, dy)` unitário. `false` = não é um nó.
    pub fn mind_nav(&mut self, doc: &BoardDoc, dir: [f64; 2]) -> bool {
        if self.editing.is_some() {
            return false;
        }
        let Some(id) = self.mind_focus(doc) else {
            return false;
        };
        let (vertical, sign) = ph2d_board_model::outward(doc, id);
        let [along, side_step] = if vertical { [dir[1], dir[0]] } else { dir };
        let target = if along != 0.0 {
            match doc.mind_parent(id) {
                None => {
                    let s = if along > 0.0 { Side::After } else { Side::Before };
                    self.middle_child(doc, id, Some(s), vertical)
                }
                Some(_) if along * sign > 0.0 => self.middle_child(doc, id, None, vertical),
                Some(p) => Some(p),
            }
        } else {
            neighbour(doc, id, side_step, vertical)
        };
        if let Some(t) = target {
            self.selection = BTreeSet::from([t]);
        }
        true
    }

    /// O filho à vista de `id` (do lado `side`, na raiz) mais perto do meio dele.
    fn middle_child(
        &self,
        doc: &BoardDoc,
        id: ElementId,
        side: Option<Side>,
        vertical: bool,
    ) -> Option<ElementId> {
        let mid = across(doc.get(id)?, vertical);
        doc.mind_children(id)
            .into_iter()
            .filter_map(|c| doc.get(c))
            .filter(|c| !c.is_hidden())
            .filter(|c| side.is_none_or(|s| c.mind().is_some_and(|m| m.side == s)))
            .min_by(|a, b| {
                (across(a, vertical) - mid)
                    .abs()
                    .total_cmp(&(across(b, vertical) - mid).abs())
            })
            .map(|c| c.id)
    }

    /// ⭐ M10: recolhe ou expande `id` (ou o nó em foco). Os escondidos saem da selecção. UM passo.
    pub fn mind_collapse(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        id: Option<ElementId>,
    ) -> bool {
        let Some(id) = id.or_else(|| self.mind_focus(doc)) else {
            return false;
        };
        if doc.mind_children(id).is_empty() {
            return false;
        }
        self.commit_text(doc, history);
        let Some(mut el) = doc.get(id).cloned() else {
            return false;
        };
        if let Some(m) = el.mind_mut() {
            m.collapsed = !m.collapsed;
        }
        history.apply(doc, vec![BoardOp::Put(el)]);
        self.selection
            .retain(|s| doc.get(*s).is_some_and(|e| !e.is_hidden()));
        true
    }

    /// M11 / M16: liga ou desliga o auto layout do nó em foco (`vertical = false`), ou troca a
    /// direcção do mapa dele (`vertical = true`, na raiz). UM passo.
    pub fn mind_toggle(&mut self, doc: &mut BoardDoc, history: &mut History, vertical: bool) -> bool {
        let Some(id) = self.mind_focus(doc) else {
            return false;
        };
        let id = if vertical {
            doc.mind_root(id).unwrap_or(id)
        } else {
            id
        };
        self.commit_text(doc, history);
        let Some(mut el) = doc.get(id).cloned() else {
            return false;
        };
        if let Some(m) = el.mind_mut() {
            if vertical {
                m.vertical = !m.vertical;
            } else {
                m.manual = !m.manual;
            }
        }
        history.apply(doc, vec![BoardOp::Put(el)]);
        true
    }

    /// Os «+» do nó seleccionado (M5): o meio do lado de fora (mundo), a direcção para fora e o lado —
    /// a raiz tem dois. Quem desenha afasta-os `Metrics::dot` px do lado, como os pontos azuis.
    #[must_use]
    pub fn mind_plus(&self, doc: &BoardDoc) -> Vec<([f64; 2], [f64; 2], Side)> {
        if self.editing.is_some() || self.tool != Tool::Select {
            return Vec::new();
        }
        let Some(id) = self.mind_focus(doc) else {
            return Vec::new();
        };
        let Some(el) = doc.get(id) else {
            return Vec::new();
        };
        let (vertical, sign) = ph2d_board_model::outward(doc, id);
        let sides: &[Side] = if doc.mind_parent(id).is_none() {
            &[Side::After, Side::Before]
        } else if sign > 0.0 {
            &[Side::After]
        } else {
            &[Side::Before]
        };
        let [cx, cy] = el.center();
        sides
            .iter()
            .map(|s| {
                let k = if *s == Side::After { 1.0 } else { -1.0 };
                if vertical {
                    ([cx, cy + k * el.h / 2.0], [0.0, k], *s)
                } else {
                    ([cx + k * el.w / 2.0, cy], [k, 0.0], *s)
                }
            })
            .collect()
    }

    /// O «+» sob `p` (o lado), se há.
    pub(crate) fn mind_plus_at(&self, doc: &BoardDoc, p: Pointer) -> Option<Side> {
        let reach = self.metrics.handle * p.px;
        let off = self.metrics.dot * p.px;
        self.mind_plus(doc).into_iter().find_map(|(at, d, s)| {
            let pos = [at[0] + d[0] * off, at[1] + d[1] * off];
            ((pos[0] - p.world[0]).hypot(pos[1] - p.world[1]) <= reach).then_some(s)
        })
    }

    /// O selo de um ramo recolhido sob `p` (o nó dele), se há.
    pub(crate) fn mind_badge_at(&self, doc: &BoardDoc, p: [f64; 2]) -> Option<ElementId> {
        doc.live()
            .filter(|el| el.mind().is_some_and(|m| m.collapsed))
            .find(|el| {
                let n = ph2d_board_model::hidden_count(doc, el.id);
                n > 0
                    && ph2d_board_model::badge_rect(doc, el.id, digits(n)).is_some_and(
                        |[x, y, w, h]| p[0] >= x && p[0] <= x + w && p[1] >= y && p[1] <= y + h,
                    )
            })
            .map(|el| el.id)
    }

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
                if let Some(side) = self.mind_plus_at(doc, p) {
                    return self.mind_child(doc, history, ts, Some(side));
                }
                match self.mind_badge_at(doc, p.world) {
                    Some(id) => self.mind_collapse(doc, history, Some(id)),
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
        lead(doc, &moved)
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
        let laid = lead(doc, &moved).is_some();
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
            m.side = if c[k] < rc[k] { Side::Before } else { Side::After };
        }
        let siblings: Vec<ElementId> = doc
            .mind_children(parent)
            .into_iter()
            .filter(|c| !moved.contains(c))
            .filter(|c| !root_parent || doc.get(*c).and_then(Element::mind).is_some_and(|cm| cm.side == m.side))
            .collect();
        let (prev, next) = if target.is_some() {
            (siblings.last().copied(), None)
        } else {
            let at = across(&now, vertical);
            let i = siblings
                .iter()
                .take_while(|c| doc.get(**c).is_some_and(|e| across(e, vertical) < at))
                .count();
            (i.checked_sub(1).map(|j| siblings[j]), siblings.get(i).copied())
        };
        m.order = FracKey::between(order_of(doc, prev.as_ref()).as_ref(), order_of(doc, next.as_ref()).as_ref());
        m.parent = Some(parent);
        let recolor = (!root_parent && target.is_some())
            .then(|| doc.get(parent).and_then(Element::mind).map(|pm| pm.color))
            .flatten();
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
        let mut ops = vec![BoardOp::Put(el)];
        if let Some(c) = recolor {
            for d in doc.mind_descendants(lead) {
                if let Some(mut e) = doc.get(d).cloned()
                    && let Some(dm) = e.mind_mut()
                {
                    dm.color = c;
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
            history.record(copies.iter().rev().map(|c| BoardOp::Delete(*c)).collect());
        }
    }
}

/// O vizinho à vista de `id` à mesma profundidade e do mesmo lado, no sentido `step` das faixas.
fn neighbour(doc: &BoardDoc, id: ElementId, step: f64, vertical: bool) -> Option<ElementId> {
    if step == 0.0 {
        return None;
    }
    let root = doc.mind_root(id)?;
    let (depth, side) = (doc.mind_depth(id), doc.mind_side(id));
    let mut row: Vec<&Element> = doc
        .mind_descendants(root)
        .into_iter()
        .filter_map(|n| doc.get(n))
        .filter(|e| !e.is_hidden())
        .filter(|e| doc.mind_depth(e.id) == depth && doc.mind_side(e.id) == side)
        .collect();
    row.sort_by(|a, b| across(a, vertical).total_cmp(&across(b, vertical)));
    let i = row.iter().position(|e| e.id == id)?;
    let j = if step > 0.0 { i.checked_add(1)? } else { i.checked_sub(1)? };
    row.get(j).map(|e| e.id)
}

/// Quantos algarismos tem `n` (a largura do selo).
#[must_use]
pub fn digits(n: usize) -> usize {
    n.to_string().len()
}

#[cfg(test)]
#[path = "mindmap_tests.rs"]
mod tests;
