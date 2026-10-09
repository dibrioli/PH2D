//! **O mapa mental no editor** (W6): criar o mapa, `Tab` (filho) e `Enter` (irmão), as setas que
//! andam pela árvore, recolher, o «+» e o largar que muda de pai. O layout é do documento
//! (`ph2d_board_model::mind`, M17); aqui vive o gesto. Leis: `docs/MiroClone/04_planear_mapa_e_modelos.md` §2.

use std::collections::BTreeSet;

use ph2d_board_layout::TextEdit;
use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, FracKey, History, MIND_BRANCH_WIDTH, MIND_COLORS,
    MIND_FONT, MIND_PAD, MIND_ROOT_PAD_Y, MIND_ROOT_STROKE, Mind, Rgba, RichText, Shape, ShapeType,
    Side, Style,
};
use ph2d_text::TextSystem;

use crate::{Editor, Pointer, Tool};

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
    let w = if root {
        tw + h
    } else {
        tw + 2.0 * MIND_PAD[0] * k
    };
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

/// O centro no eixo das FAIXAS (y no horizontal, x no vertical).
pub(crate) fn across(el: &Element, vertical: bool) -> f64 {
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

/// Arruma AO VIVO os mapas a que `ids` pertencem — o fim de um gesto que mexeu em nós sem ser o
/// largar de um ramo (um arrasto misto, redimensionar, rodar): o layout devolve os arrumados ao
/// sítio (M17; auditoria W6).
pub(crate) fn settle_moved(doc: &mut BoardDoc, ids: impl IntoIterator<Item = ElementId>) {
    let roots: BTreeSet<ElementId> = ids.into_iter().filter_map(|id| doc.mind_root(id)).collect();
    if !roots.is_empty() {
        let _ = ph2d_board_model::mind_settle(doc, &roots);
    }
}

/// O nó é arrumado pelo layout (tem pai, e o pai tem auto layout).
pub(crate) fn laid(doc: &BoardDoc, id: ElementId) -> bool {
    doc.mind_parent(id)
        .and_then(|p| doc.get(p))
        .and_then(Element::mind)
        .is_some_and(|m| !m.manual)
}

/// A chave do primeiro irmão (filho de `parent`) ESTRITAMENTE depois de `key` — o vizinho de cima
/// de um «logo depois», mesmo num documento com chaves repetidas.
pub(crate) fn after(doc: &BoardDoc, parent: ElementId, key: &FracKey) -> Option<FracKey> {
    doc.mind_children(parent)
        .into_iter()
        .filter_map(|c| doc.get(c)?.mind().map(|m| m.order.clone()))
        .filter(|o| o > key)
        .min()
}

/// M14: a cor do filho novo da raiz — a seguinte da paleta depois da do ÚLTIMO filho (depois de
/// apagar um do meio, a contagem repetia a do vizinho — auditoria W6).
pub(crate) fn next_color(doc: &BoardDoc, root: ElementId) -> Rgba {
    let last = doc
        .mind_children(root)
        .last()
        .and_then(|c| doc.get(*c))
        .and_then(Element::mind)
        .and_then(|m| MIND_COLORS.iter().position(|c| Rgba(*c) == m.color));
    Rgba(MIND_COLORS[last.map_or(0, |i| (i + 1) % MIND_COLORS.len())])
}

/// Onde nasce o filho de um pai SEM auto layout (M11): um vão de nível depois da borda de fora
/// dele, logo abaixo do último filho (ou à altura do pai) — nunca por cima do pai.
fn beside_last(doc: &BoardDoc, parent: ElementId, kids: &[ElementId], side: Side) -> [f64; 2] {
    let Some(p) = doc.get(parent) else {
        return [0.0, 0.0];
    };
    let (vertical, _) = ph2d_board_model::outward(doc, parent);
    let k = p.style().font_size / ph2d_board_model::MIND_FONT;
    let (level, gap) = (
        ph2d_board_model::MIND_LEVEL_GAP * k,
        ph2d_board_model::MIND_SIBLING_GAP * k,
    );
    let sign = if side == Side::After { 1.0 } else { -1.0 };
    let last = kids.last().and_then(|c| doc.get(*c));
    let [cx, cy] = p.center();
    if vertical {
        let y = cy + sign * (p.h / 2.0 + level);
        let x = last.map_or(cx, |l| l.x + l.w + gap + l.w / 2.0);
        [x, y]
    } else {
        // O filho novo nasce vazio: meia largura de um nó vazio além do vão.
        let half = (EMPTY_EMS + 2.0 * MIND_PAD[0] / ph2d_board_model::MIND_FONT)
            * p.style().font_size
            / 2.0;
        let x = cx + sign * (p.w / 2.0 + level + half);
        let y = last.map_or(cy, |l| l.y + l.h + gap + l.h / 2.0);
        [x, y]
    }
}

/// Uma CÓPIA de nó (colar, duplicar, `Alt`): com o pai copiado, fica debaixo da cópia dele; sem
/// ele, é irmã do original LOGO DEPOIS dele, com chave própria; se o pai já não existe, é a raiz de
/// um mapa novo (com a forma de raiz). `true` = virou raiz (o tamanho acerta-se no próximo
/// overlay).
pub(crate) fn rehome_copy(
    doc: &BoardDoc,
    el: &mut Element,
    map: &std::collections::BTreeMap<ElementId, ElementId>,
) -> bool {
    let Some(m) = el.mind().cloned() else {
        return false;
    };
    let Some(p) = m.parent else {
        return false;
    };
    if let Some(np) = map.get(&p) {
        if let Some(mm) = el.mind_mut() {
            mm.parent = Some(*np);
        }
        return false;
    }
    if doc.get(p).and_then(Element::mind).is_some() {
        let order = FracKey::between(Some(&m.order), after(doc, p, &m.order).as_ref());
        if let Some(mm) = el.mind_mut() {
            mm.order = order;
        }
        return false;
    }
    if let Some(mm) = el.mind_mut() {
        mm.parent = None;
        mm.color = Rgba(MIND_ROOT_STROKE);
    }
    if let Some(s) = el.shape_mut() {
        s.kind = ShapeType::Pill;
        s.style.stroke = Some(Rgba(MIND_ROOT_STROKE));
    }
    true
}

/// A casa (o frame) de um nó novo: a do pai; a raiz, a do frame onde nasce.
fn home_of(doc: &BoardDoc, mind: &Mind, at: [f64; 2]) -> Option<ElementId> {
    match mind.parent.and_then(|p| doc.get(p)) {
        Some(p) => p.frame,
        None => doc.deepest_frame_at(at, &BTreeSet::new()),
    }
}

pub(crate) fn order_of(doc: &BoardDoc, id: Option<&ElementId>) -> Option<FracKey> {
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
        let mut el = Element::new_shape(
            doc.mint_id(),
            doc.z_on_top(),
            shape,
            [at[0], at[1], 0.0, 0.0],
        );
        el.frame = home_of(doc, &mind, at);
        el.set_mind(Some(mind));
        let [w, h] = node_size(&el, text_size(ts, &el));
        (el.x, el.y, el.w, el.h) = (at[0] - w / 2.0, at[1] - h / 2.0, w, h);
        el
    }

    /// Um nó novo SEM o moldador à mão (um botão da barra): o tamanho é uma estimativa e o próximo
    /// [`Editor::overlay`] acerta-o ao texto (o caminho das notas nascidas fora do teclado).
    pub(crate) fn mind_node_unfitted(
        &mut self,
        doc: &mut BoardDoc,
        mind: Mind,
        at: [f64; 2],
        text: &str,
    ) -> Element {
        let mut shape = self.mind_shape(mind.parent.is_none());
        shape.text = RichText::plain(text);
        let font = shape.style.font_size;
        let mut el = Element::new_shape(
            doc.mint_id(),
            doc.z_on_top(),
            shape,
            [at[0], at[1], 0.0, 0.0],
        );
        el.frame = home_of(doc, &mind, at);
        el.set_mind(Some(mind));
        let guess = [
            (EMPTY_EMS * font).max(text.chars().count() as f64 * font / 2.0),
            font,
        ];
        let [w, h] = node_size(&el, guess);
        (el.x, el.y, el.w, el.h) = (at[0] - w / 2.0, at[1] - h / 2.0, w, h);
        self.unfitted.push(el.id);
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
        let m = Mind::new(
            None,
            FracKey::between(None, None),
            Side::After,
            Rgba(MIND_ROOT_STROKE),
        );
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
            (s, next_color(doc, parent))
        } else {
            (doc.mind_side(parent), pm.color)
        };
        let at = if pm.manual {
            beside_last(doc, parent, &kids, side)
        } else {
            p.center()
        };
        let node = self.mind_node(doc, ts, Mind::new(Some(parent), order, side, color), at, "");
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
        let order = FracKey::between(Some(&m.order), after(doc, parent, &m.order).as_ref());
        let node = self.mind_node(
            doc,
            ts,
            Mind::new(Some(parent), order, m.side, m.color),
            el.center(),
            "",
        );
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
                    let s = if along > 0.0 {
                        Side::After
                    } else {
                        Side::Before
                    };
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
    pub fn mind_toggle(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        vertical: bool,
    ) -> bool {
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
        // Recolhido, o lado de fora é do selo (o número): o «+» dele tapava-o e roubava-lhe o clique
        // a zoom alto (auditoria W6) — expande-se primeiro.
        let badge = el.mind().filter(|m| m.collapsed).map(|_| {
            if sign > 0.0 {
                Side::After
            } else {
                Side::Before
            }
        });
        sides
            .iter()
            .filter(|s| Some(**s) != badge)
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
    let j = if step > 0.0 {
        i.checked_add(1)?
    } else {
        i.checked_sub(1)?
    };
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
