//! **O mapa mental** (W6): a árvore — um ponteiro do FILHO para o pai, como o `frame` —, o layout por
//! FAIXAS do Miro e a invariante: todo lote que toca um mapa deixa-o arrumado, no mesmo passo de
//! desfazer ([`crate::apply_batch`] chama [`settle`]). Leis e medidas:
//! `docs/MiroClone/04_planear_mapa_e_modelos.md` (§1.3 a imagem do Miro medida, §2 M1–M17).
//!
//! Um nó é uma forma com [`Mind`] na caixa de organizar ([`crate::Org`]). Recolher esconde os
//! descendentes SEM rearrumar (M10): o layout ignora o «recolhido», e o escondido sai de
//! [`BoardDoc::live`] — do desenho, do clique, do laço e da busca de uma vez.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{BoardDoc, BoardOp, Element, ElementId, FracKey, Rgba};

/// A letra de nascença de um nó (a omissão do Miro, REST: 14). As medidas abaixo são a esta letra e
/// escalam com a da raiz (o Miro: a letra muda-se só na raiz e escala o mapa todo).
pub const MIND_FONT: f64 = 14.0;
/// Vão entre níveis, de caixa a caixa, desde a borda de FORA do pai: 122 de texto a texto no Miro
/// menos as duas folgas de 9 (M3).
pub const MIND_LEVEL_GAP: f64 = 104.0;
/// Vão entre faixas vizinhas, a qualquer profundidade (M2): com a linha de 16,94 do nosso moldado a
/// 14 e as folgas de [`MIND_PAD`], o passo de um nó de uma linha é 53,94 — o Miro mede 54.
pub const MIND_SIBLING_GAP: f64 = 25.0;
/// A folga `[x, y]` entre o texto e a caixa de um nó (a linha do ramo encosta a ~9 do texto, §1.3).
pub const MIND_PAD: [f64; 2] = [9.0, 6.0];
/// A folga vertical da raiz: a pílula de 66 de alto com uma linha (§1.3); de lado, meia altura.
pub const MIND_ROOT_PAD_Y: f64 = 24.5;
/// O braço da curva de um ramo em fracção do VÃO no eixo dos níveis: 40,5 / 104 medido em cinco ramos
/// da imagem do Miro (rms ≤ 0,97 px) — não o das setas (`END_ARM`, da distância).
pub const MIND_ARM: f64 = 0.39;
/// A espessura de um ramo à letra de nascença.
pub const MIND_BRANCH_WIDTH: f64 = 1.5;
/// O contorno da raiz (lilás da imagem do Miro). Dado do documento, não da interface.
// LITERAL-COLOR-OK: medido na imagem oficial do Miro (04 §1.3).
pub const MIND_ROOT_STROKE: [u8; 4] = [0xaf, 0xa8, 0xfe, 0xff];
/// As cores dos ramos da raiz, pela ordem em que os filhos dela nascem (M14) — os píxeis mais
/// saturados de cada ramo da imagem do Miro. Dado do documento.
pub const MIND_COLORS: [[u8; 4]; 5] = [
    // LITERAL-COLOR-OK: paleta de documento medida na imagem oficial do Miro (04 §1.3).
    [0xff, 0xab, 0x00, 0xff],
    [0x7a, 0x28, 0xff, 0xff],
    [0xb7, 0x36, 0xff, 0xff],
    [0xfc, 0x66, 0xe7, 0xff],
    [0x74, 0xe6, 0xd2, 0xff],
];

/// De que lado da raiz vive um ramo: `After` = direita (ou baixo, no vertical).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    #[default]
    After,
    Before,
}

/// O que faz de uma forma um nó de mapa mental.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mind {
    /// O pai (`None` = a raiz do mapa).
    pub parent: Option<ElementId>,
    /// A ordem entre os irmãos.
    pub order: FracKey,
    /// O lado — só o de um filho da RAIZ conta; os de baixo herdam o do ramo.
    pub side: Side,
    /// A cor do ramo que chega a este nó.
    pub color: Rgba,
    /// Os descendentes estão escondidos (M10).
    pub collapsed: bool,
    /// Auto layout DESLIGADO neste pai: os filhos ficam onde a mão os pôs e andam com ele (M11).
    pub manual: bool,
    /// Só na raiz: o mapa cresce para cima e para baixo (M16).
    pub vertical: bool,
    /// Escondido por um antepassado recolhido — derivado, mantido pelas operações (não vai para o
    /// ficheiro).
    #[serde(skip)]
    pub(crate) hidden: bool,
}

impl Mind {
    /// Um nó novo debaixo de `parent`.
    #[must_use]
    pub fn new(parent: Option<ElementId>, order: FracKey, side: Side, color: Rgba) -> Self {
        Self {
            parent,
            order,
            side,
            color,
            collapsed: false,
            manual: false,
            vertical: false,
            hidden: false,
        }
    }
}

impl Element {
    /// O nó de mapa mental, se este elemento é um.
    #[must_use]
    pub fn mind(&self) -> Option<&Mind> {
        self.org.as_ref().and_then(|o| o.mind.as_ref())
    }

    /// O nó, mutável (`None` se não é um).
    pub fn mind_mut(&mut self) -> Option<&mut Mind> {
        self.org.as_mut().and_then(|o| o.mind.as_mut())
    }

    /// Faz deste elemento um nó (ou deixa de o ser, com `None`).
    pub fn set_mind(&mut self, m: Option<Mind>) {
        if m.is_some() || self.org.is_some() {
            self.org_mut().mind = m;
        }
    }

    /// Escondido por um ramo recolhido (fora do desenho, do clique e do laço).
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.mind().is_some_and(|m| m.hidden)
    }
}

/// Os filhos de cada pai e as raízes — o índice que as operações mantêm (não vai para o ficheiro).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MindIndex {
    children: BTreeMap<ElementId, BTreeSet<ElementId>>,
}

impl BoardDoc {
    /// O nó vivo `id`, se é de um mapa.
    fn node(&self, id: ElementId) -> Option<(&Element, &Mind)> {
        let el = self.get(id)?;
        Some((el, el.mind()?))
    }

    /// Os filhos de `id` pela ordem dos irmãos (os vivos).
    #[must_use]
    pub fn mind_children(&self, id: ElementId) -> Vec<ElementId> {
        let Some(set) = self.minds.children.get(&id) else {
            return Vec::new();
        };
        let mut v: Vec<(&FracKey, ElementId)> = set
            .iter()
            .filter_map(|c| self.node(*c).map(|(_, m)| (&m.order, *c)))
            .collect();
        v.sort();
        v.into_iter().map(|(_, c)| c).collect()
    }

    /// O pai de `id` se é um nó vivo de mapa (um órfão — pai apagado — é a raiz do seu ramo).
    #[must_use]
    pub fn mind_parent(&self, id: ElementId) -> Option<ElementId> {
        let p = self.node(id)?.1.parent?;
        self.node(p).map(|_| p)
    }

    /// A raiz do mapa de `id` (`None` se `id` não é um nó vivo). Um ciclo (de uma fusão estragada)
    /// pára no primeiro repetido.
    #[must_use]
    pub fn mind_root(&self, id: ElementId) -> Option<ElementId> {
        self.node(id)?;
        let mut seen = BTreeSet::new();
        let mut at = id;
        while let Some(p) = self.mind_parent(at) {
            if !seen.insert(at) {
                break;
            }
            at = p;
        }
        Some(at)
    }

    /// Todos os descendentes de `id` (escondidos incluídos), sem `id`.
    #[must_use]
    pub fn mind_descendants(&self, id: ElementId) -> BTreeSet<ElementId> {
        let mut out = BTreeSet::new();
        let mut stack = vec![id];
        while let Some(n) = stack.pop() {
            for c in self.mind_children(n) {
                if out.insert(c) {
                    stack.push(c);
                }
            }
        }
        out.remove(&id);
        out
    }

    /// A profundidade de `id` no mapa (0 = raiz).
    #[must_use]
    pub fn mind_depth(&self, id: ElementId) -> usize {
        let mut d = 0;
        let mut at = id;
        while let Some(p) = self.mind_parent(at) {
            d += 1;
            at = p;
            if d > self.elements.len() {
                break;
            }
        }
        d
    }

    /// O lado em que vive `id` (o do filho da raiz por onde se chega a ele).
    #[must_use]
    pub fn mind_side(&self, id: ElementId) -> Side {
        let mut at = id;
        let mut guard = 0;
        while let Some(p) = self.mind_parent(at) {
            if self.mind_parent(p).is_none() {
                break;
            }
            at = p;
            guard += 1;
            if guard > self.elements.len() {
                break;
            }
        }
        self.node(at).map_or(Side::After, |(_, m)| m.side)
    }

    /// Antes de `prev` sair do documento (ou ser substituído): tira-o do índice.
    pub(crate) fn mind_unlink(&mut self, prev: &Element) {
        if let Some(m) = prev.mind().filter(|_| !prev.deleted)
            && let Some(p) = m.parent
            && let Some(set) = self.minds.children.get_mut(&p)
        {
            set.remove(&prev.id);
            if set.is_empty() {
                self.minds.children.remove(&p);
            }
        }
    }

    /// Depois de `id` entrar (ou mudar): põe-no no índice e acerta o «escondido» dele e — se o véu
    /// dele mudou — o dos descendentes. `veil_before` = o véu (escondido ou recolhido) que tinha.
    pub(crate) fn mind_link(&mut self, id: ElementId, veil_before: Option<bool>) {
        let parent = match self.elements.get(&id) {
            Some(el) if !el.deleted => el.mind().map(|m| m.parent),
            _ => None,
        };
        if let Some(Some(p)) = parent {
            self.minds.children.entry(p).or_default().insert(id);
        }
        self.refresh_hidden(id, veil_before);
    }

    /// O véu de `id` (o que esconde os filhos dele): escondido, ou recolhido.
    fn veil(&self, id: ElementId) -> bool {
        self.node(id).is_some_and(|(_, m)| m.hidden || m.collapsed)
    }

    /// Acerta o «escondido» de `id` pelo véu do pai e desce enquanto o véu muda.
    fn refresh_hidden(&mut self, id: ElementId, veil_before: Option<bool>) {
        let mut stack = vec![(id, veil_before)];
        let mut steps = 0;
        while let Some((n, before)) = stack.pop() {
            steps += 1;
            if steps > self.elements.len() + 1 {
                break;
            }
            let hidden = self
                .elements
                .get(&n)
                .and_then(|el| el.mind())
                .and_then(|m| m.parent)
                .is_some_and(|p| self.veil(p));
            if let Some(m) = self
                .elements
                .get_mut(&n)
                .filter(|el| !el.deleted)
                .and_then(|el| el.mind_mut())
            {
                m.hidden = hidden;
            }
            let now = self.veil(n);
            if before == Some(now) {
                continue;
            }
            if let Some(kids) = self.minds.children.get(&n) {
                stack.extend(kids.iter().map(|c| (*c, None)));
            }
        }
    }

    /// Refaz o índice e os «escondidos» (ao ler de bytes).
    pub(crate) fn reindex_minds(&mut self) {
        self.minds = MindIndex::default();
        let ids: Vec<ElementId> = self
            .elements
            .values()
            .filter(|e| !e.deleted && e.mind().is_some())
            .map(|e| e.id)
            .collect();
        for id in &ids {
            if let Some(Some(p)) = self
                .elements
                .get(id)
                .and_then(|e| e.mind())
                .map(|m| m.parent)
            {
                self.minds.children.entry(p).or_default().insert(*id);
            }
        }
        let roots: Vec<ElementId> = ids
            .iter()
            .copied()
            .filter(|id| self.mind_parent(*id).is_none())
            .collect();
        for r in roots {
            self.refresh_hidden(r, None);
        }
    }
}

/// Os eixos de um mapa: no horizontal os níveis andam em x e as faixas em y; no vertical, trocados.
#[derive(Clone, Copy)]
struct Axes {
    vertical: bool,
}

impl Axes {
    /// `[nível, faixa]` de um ponto `[x, y]`.
    fn split(self, [x, y]: [f64; 2]) -> [f64; 2] {
        if self.vertical { [y, x] } else { [x, y] }
    }

    /// `[x, y]` de `[nível, faixa]`.
    fn join(self, [a, b]: [f64; 2]) -> [f64; 2] {
        if self.vertical { [b, a] } else { [a, b] }
    }
}

/// ⭐ **O layout de um mapa** (M1–M4, M11, M16): o canto superior esquerdo de cada nó que o layout
/// coloca, a partir da raiz (que não se move). A faixa de um nó é a soma das dos filhos (com o vão)
/// ou o tamanho dele, a maior; o pai fica no meio; cada lado da raiz centra-se nela por si; o filho
/// começa um vão depois da borda de FORA do pai. Num pai `manual` os filhos andam com ele, onde
/// estavam. O «recolhido» não entra (M10).
#[must_use]
pub fn layout(doc: &BoardDoc, root: ElementId) -> Vec<(ElementId, [f64; 2])> {
    let Some((r, rm)) = doc.node(root) else {
        return Vec::new();
    };
    let ax = Axes {
        vertical: rm.vertical,
    };
    let scale = r.style().font_size / MIND_FONT;
    let (level, gap) = (MIND_LEVEL_GAP * scale, MIND_SIBLING_GAP * scale);
    let size = |el: &Element| ax.split([el.w, el.h]);

    // As faixas, de baixo para cima (pós-ordem sem recursão: um ramo de 10 mil não estoura a pilha).
    let mut order: Vec<ElementId> = Vec::new();
    let mut kids: BTreeMap<ElementId, Vec<ElementId>> = BTreeMap::new();
    let mut seen = BTreeSet::from([root]);
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        order.push(n);
        let cs: Vec<ElementId> = doc
            .mind_children(n)
            .into_iter()
            .filter(|c| seen.insert(*c))
            .collect();
        stack.extend(cs.iter().copied());
        kids.insert(n, cs);
    }
    let mut band: BTreeMap<ElementId, f64> = BTreeMap::new();
    for n in order.iter().rev() {
        let Some((el, m)) = doc.node(*n) else {
            continue;
        };
        let own = size(el)[1];
        let cs = &kids[n];
        let b = if m.manual || cs.is_empty() || *n == root {
            own
        } else {
            let sum: f64 = cs.iter().map(|c| band.get(c).copied().unwrap_or(0.0)).sum();
            own.max(sum + gap * (cs.len() - 1) as f64)
        };
        band.insert(*n, b);
    }

    // As posições, de cima para baixo.
    let mut out = Vec::new();
    let mut at: BTreeMap<ElementId, [f64; 2]> = BTreeMap::from([(root, [r.x, r.y])]);
    let mut stack = vec![(root, Side::After)];
    while let Some((n, side)) = stack.pop() {
        let Some((el, m)) = doc.node(n) else {
            continue;
        };
        let p = at[&n];
        let cs = &kids[&n];
        if m.manual {
            let d = [p[0] - el.x, p[1] - el.y];
            for c in cs {
                let Some((ce, cm)) = doc.node(*c) else {
                    continue;
                };
                let q = [ce.x + d[0], ce.y + d[1]];
                at.insert(*c, q);
                out.push((*c, q));
                stack.push((*c, if n == root { cm.side } else { side }));
            }
            continue;
        }
        let [pl, pc] = ax.split(p);
        let [sl, sc] = size(el);
        let groups: Vec<(Side, Vec<ElementId>)> = if n == root {
            [Side::After, Side::Before]
                .into_iter()
                .map(|s| {
                    let g = cs
                        .iter()
                        .copied()
                        .filter(|c| doc.node(*c).is_some_and(|(_, cm)| cm.side == s))
                        .collect();
                    (s, g)
                })
                .collect()
        } else {
            vec![(side, cs.clone())]
        };
        for (s, g) in groups {
            if g.is_empty() {
                continue;
            }
            let total: f64 = g.iter().map(|c| band[c]).sum::<f64>() + gap * (g.len() - 1) as f64;
            let mut start = pc + sc / 2.0 - total / 2.0;
            for c in g {
                let Some((ce, _)) = doc.node(c) else {
                    continue;
                };
                let [cl, cc] = size(ce);
                let main = match s {
                    Side::After => pl + sl + level,
                    Side::Before => pl - level - cl,
                };
                let q = ax.join([main, start + band[&c] / 2.0 - cc / 2.0]);
                start += band[&c] + gap;
                at.insert(c, q);
                out.push((c, q));
                stack.push((c, s));
            }
        }
    }
    out
}

/// ⭐ **Arruma os mapas de `roots`** (M17): põe cada nó onde o [`layout`] o quer e devolve os
/// inversos, pela ordem de aplicação.
pub fn settle(doc: &mut BoardDoc, roots: &BTreeSet<ElementId>) -> Vec<BoardOp> {
    let mut inverse = Vec::new();
    for root in roots {
        if doc.mind_root(*root) != Some(*root) {
            continue;
        }
        for (id, [x, y]) in layout(doc, *root) {
            let Some(el) = doc.get(id) else {
                continue;
            };
            if el.x == x && el.y == y {
                continue;
            }
            let mut el = el.clone();
            el.translate([x - el.x, y - el.y]);
            inverse.extend(BoardOp::Put(el).apply(doc));
        }
    }
    inverse
}

/// A altura do selo de um ramo recolhido, em letras do mapa (M10: o número escondido, numa caixa da
/// cor do ramo do lado de fora do nó — nos GIFs do help do Miro, da altura de uma linha).
pub const MIND_BADGE_H: f64 = 1.25;
/// O vão entre o nó e o selo, e entre o nó e o «+», em letras do mapa.
pub const MIND_BADGE_GAP: f64 = 0.5;

/// O lado de FORA de `id` no seu mapa: `(nível, sinal)` — o eixo dos níveis (`true` = vertical) e
/// para onde o ramo cresce (`+1` depois, `-1` antes). A raiz cresce para os dois.
#[must_use]
pub fn outward(doc: &BoardDoc, id: ElementId) -> (bool, f64) {
    let vertical = doc
        .mind_root(id)
        .and_then(|r| doc.node(r))
        .is_some_and(|(_, m)| m.vertical);
    let sign = match doc.mind_side(id) {
        Side::After => 1.0,
        Side::Before => -1.0,
    };
    (vertical, sign)
}

/// ⭐ **O selo de um nó recolhido** `[x, y, w, h]` (mundo): do lado de fora, centrado na faixa, da
/// altura de [`MIND_BADGE_H`] letras e largo para `digits` algarismos — a MESMA conta para o desenho
/// e para o clique que expande.
#[must_use]
pub fn badge_rect(doc: &BoardDoc, id: ElementId, digits: usize) -> Option<[f64; 4]> {
    let el = doc.get(id)?;
    let font = el.style().font_size;
    let h = MIND_BADGE_H * font;
    let w = h.max(font * (0.4 + 0.6 * digits as f64));
    let gap = MIND_BADGE_GAP * font;
    let (vertical, sign) = outward(doc, id);
    let [cx, cy] = el.center();
    Some(match (vertical, sign > 0.0) {
        (false, true) => [el.x + el.w + gap, cy - h / 2.0, w, h],
        (false, false) => [el.x - gap - w, cy - h / 2.0, w, h],
        (true, true) => [cx - w / 2.0, el.y + el.h + gap, w, h],
        (true, false) => [cx - w / 2.0, el.y - gap - h, w, h],
    })
}

/// ⭐ **O ramo que chega a `child`** (M13): os quatro pontos da cúbica (mundo) — da borda de FORA do
/// pai à de DENTRO do filho, a meia altura, com as tangentes no eixo dos níveis e o braço de
/// [`MIND_ARM`] do vão. `None` = não há ramo (raiz, ou um dos dois escondido).
#[must_use]
pub fn branch(doc: &BoardDoc, child: ElementId) -> Option<[[f64; 2]; 4]> {
    let parent = doc.mind_parent(child)?;
    let (c, p) = (doc.get(child)?, doc.get(parent)?);
    if c.is_hidden() || p.is_hidden() {
        return None;
    }
    let (vertical, sign) = outward(doc, child);
    let ax = Axes { vertical };
    let ([pl, pc], [psl, psc]) = (ax.split([p.x, p.y]), ax.split([p.w, p.h]));
    let ([cl, cc], [csl, csc]) = (ax.split([c.x, c.y]), ax.split([c.w, c.h]));
    let (a, b) = if sign > 0.0 {
        (pl + psl, cl)
    } else {
        (pl, cl + csl)
    };
    let (ya, yb) = (pc + psc / 2.0, cc + csc / 2.0);
    let arm = MIND_ARM * (b - a);
    Some([
        ax.join([a, ya]),
        ax.join([a + arm, ya]),
        ax.join([b - arm, yb]),
        ax.join([b, yb]),
    ])
}

impl BoardDoc {
    /// Uma seta presa a um nó escondido por um ramo recolhido some com ele (desenho e clique).
    #[must_use]
    pub fn connector_hidden(&self, el: &Element) -> bool {
        el.connector().is_some_and(|c| {
            [&c.start, &c.end].into_iter().any(|e| match e {
                crate::End::Bound { target, .. } => {
                    self.elements.get(target).is_some_and(Element::is_hidden)
                }
                crate::End::Free(_) => false,
            })
        })
    }

    /// Há algum ramo de mapa mental (o desenho salta os ramos num quadro sem nenhum).
    #[must_use]
    pub fn has_mind_links(&self) -> bool {
        !self.minds.children.is_empty()
    }

    /// Os pares `(pai, filho)` dos mapas mentais (o índice: sem passar pelo quadro inteiro).
    pub fn mind_links(&self) -> impl Iterator<Item = (ElementId, ElementId)> + '_ {
        self.minds
            .children
            .iter()
            .flat_map(|(p, kids)| kids.iter().map(move |c| (*p, *c)))
    }
}

/// Quantos descendentes de `id` o recolher esconde.
#[must_use]
pub fn hidden_count(doc: &BoardDoc, id: ElementId) -> usize {
    doc.mind_descendants(id).len()
}

/// As raízes dos mapas a que `id` pertence AGORA (a dele e a dos filhos, se ficaram órfãos).
pub(crate) fn roots_of(doc: &BoardDoc, id: ElementId, into: &mut BTreeSet<ElementId>) {
    if let Some(r) = doc.mind_root(id) {
        into.insert(r);
    }
    if let Some(kids) = doc.minds.children.get(&id) {
        for c in kids {
            if let Some(r) = doc.mind_root(*c) {
                into.insert(r);
            }
        }
    }
}

#[cfg(test)]
#[path = "mind_tests.rs"]
mod tests;
