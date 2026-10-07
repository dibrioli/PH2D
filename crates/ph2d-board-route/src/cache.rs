//! ⭐ **A cache de rotas** (plano §2: «rota em cache»). O conector vectorial refaz TODAS as rotas a
//! cada quadro (plano §0, o risco a escala); aqui:
//!
//! 1. o documento não mudou (a revisão da sessão é a mesma) ⇒ nada a fazer, O(1);
//! 2. mudou ⇒ a DIFERENÇA (as versões de cada elemento contra as da última vez, num passeio pela
//!    ordem de id) diz o que mudou; o índice espacial (as perguntas do gesto) troca só essas formas,
//!    e só se REVÊ a seta que mudou ou cuja forma de uma ponta mudou — nenhuma outra forma conta
//!    (ordem do dono, 06/10: as setas não se reajustam sozinhas);
//! 3. uma seta revista compara a sua CHAVE (a geometria das pontas e os pontos de ajuste) e só volta
//!    a ser desenhada se ela mudou.
//!
//! A chave é GEOMETRIA, não versões: uma cache que passa de um quadro a outro nunca confunde dois
//! elementos com o mesmo id (a diferença vê tudo mudado e revê tudo).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Anchor, BoardDoc, Element, ElementId, End, Route, ShapeType};
use ph2d_vec_connect::Aabb;

use crate::index::ShapeIndex;
use crate::{Resolved, Routed, SPREAD_STEP, aabb, anchor_for, compute, resolve};

/// O que decide a rota de uma ponta.
#[derive(Clone, Debug, PartialEq)]
enum EndKey {
    Point([f64; 2]),
    Shape {
        anchor: Anchor,
        /// `x, y, w, h, angle`
        geo: [f64; 5],
        kind: ShapeType,
        round: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct Key {
    route: Route,
    ends: [EndKey; 2],
    waypoints: Vec<[f64; 2]>,
    spread: f64,
    /// As hastes debaixo das pontas de seta (dependem das pontas e da espessura).
    stems: [f64; 2],
}

#[derive(Debug)]
struct Entry {
    key: Key,
    routed: Routed,
}

/// As rotas das setas de UM documento de cada vez (trocar de quadro refá-las).
#[derive(Debug, Default)]
pub struct RouteCache {
    rev: Option<u64>,
    /// `(id, versão)` de cada elemento vivo na última sincronização, por id.
    seen: Vec<(ElementId, u64)>,
    index: ShapeIndex,
    routes: BTreeMap<ElementId, Entry>,
    rerouted: usize,
    revisited: usize,
}

fn end_key(r: &Resolved<'_>) -> EndKey {
    match *r {
        Resolved::Point(p) => EndKey::Point(p),
        Resolved::Shape(el, anchor) => {
            let s = el.shape().expect("resolve só devolve formas");
            EndKey::Shape {
                anchor,
                geo: [el.x, el.y, el.w, el.h, el.angle],
                kind: s.kind,
                round: s.style.round,
            }
        }
    }
}

impl RouteCache {
    /// Põe as rotas em dia com `doc`. O(1) quando nada mudou.
    pub fn sync(&mut self, doc: &BoardDoc) {
        if self.rev == Some(doc.rev()) {
            return;
        }
        self.rev = Some(doc.rev());
        self.rerouted = 0;
        self.revisited = 0;
        let changed = self.diff(doc);

        let conns: Vec<&Element> = doc.live().filter(|el| el.connector().is_some()).collect();
        let spreads = spreads(&conns);
        let alive: BTreeSet<ElementId> = conns.iter().map(|el| el.id).collect();
        self.routes.retain(|id, _| alive.contains(id));
        for el in conns {
            let c = el.connector().expect("filtrado acima");
            let spread = spreads.get(&el.id).copied().unwrap_or(0.0);
            let stale = match self.routes.get(&el.id) {
                None => true,
                Some(e) => {
                    changed.contains(&el.id)
                        || c.targets().any(|t| changed.contains(&t))
                        || e.key.spread != spread
                }
            };
            if stale {
                self.revisit(doc, el, spread);
            }
        }
    }

    /// A diferença contra a última sincronização: os ids que mudaram (novos, mudados, apagados) —
    /// com o índice já em dia.
    fn diff(&mut self, doc: &BoardDoc) -> BTreeSet<ElementId> {
        let mut changed = BTreeSet::new();
        let mut now = Vec::with_capacity(self.seen.len());
        let mut old = self.seen.iter().peekable();
        for el in doc.live() {
            now.push((el.id, el.version));
            while let Some(&&(id, _)) = old.peek().filter(|(id, _)| *id < el.id) {
                old.next();
                changed.insert(id);
                self.index.remove(id);
            }
            let same = old
                .peek()
                .is_some_and(|&&(id, v)| id == el.id && v == el.version);
            if old.peek().is_some_and(|&&(id, _)| id == el.id) {
                old.next();
            }
            if same {
                continue;
            }
            changed.insert(el.id);
            if el.shape().is_some() {
                self.index.insert(el.id, aabb(el));
            } else {
                self.index.remove(el.id);
            }
        }
        for &(id, _) in old {
            changed.insert(id);
            self.index.remove(id);
        }
        self.seen = now;
        changed
    }

    /// Resolve as pontas e — se a chave mudou — refaz a rota.
    fn revisit(&mut self, doc: &BoardDoc, el: &Element, spread: f64) {
        self.revisited += 1;
        let c = el.connector().expect("só setas");
        let prev = self.routes.remove(&el.id);
        let was = prev.as_ref().map(|e| e.routed.ends());
        let mut ends = [
            resolve(doc, c.start, was.map(|w| w[0])),
            resolve(doc, c.end, was.map(|w| w[1])),
        ];
        // Uma forma que sumiu sem rota anterior: a ponta fica onde está a OUTRA.
        for i in 0..2 {
            if missing(doc, c.ends()[i]) && was.is_none() {
                ends[i] = Resolved::Point(ends[1 - i].center());
            }
        }
        let key = Key {
            route: c.route,
            ends: [end_key(&ends[0]), end_key(&ends[1])],
            waypoints: c.waypoints.clone(),
            spread,
            stems: crate::stems(c),
        };
        let entry = match prev {
            Some(e) if e.key == key => e,
            prev => {
                self.rerouted += 1;
                let sides = prev.map_or([None; 2], |e| e.routed.sides.map(Some));
                let ends = [&ends[0], &ends[1]];
                let routed = compute(ends, c.route, &c.waypoints, sides, spread, key.stems);
                Entry { key, routed }
            }
        };
        self.routes.insert(el.id, entry);
    }

    /// A rota da seta `id` (depois de [`Self::sync`]).
    #[must_use]
    pub fn get(&self, id: ElementId) -> Option<&Routed> {
        self.routes.get(&id).map(|e| &e.routed)
    }

    /// Quantas setas o último [`Self::sync`] mandou ao roteador (as réguas e os gates).
    #[must_use]
    pub fn rerouted(&self) -> usize {
        self.rerouted
    }

    /// Quantas setas o último [`Self::sync`] reviu (a chave comparada).
    #[must_use]
    pub fn revisited(&self) -> usize {
        self.revisited
    }

    /// A forma de CIMA sob `p` (o contorno, ou a menos de `tol` dele) — pelo índice: o passear do
    /// rato pergunta-o a cada movimento, e ordenar o quadro inteiro por z para isso custa O(N log N).
    pub fn shape_at(&mut self, doc: &BoardDoc, p: [f64; 2], tol: f64) -> Option<ElementId> {
        self.sync(doc);
        let mut cand = Vec::new();
        self.index.near(Aabb::new(p, p).inflate(tol), &mut cand);
        let ids: BTreeSet<ElementId> = cand.into_iter().map(|(id, _)| id).collect();
        ids.into_iter()
            .filter_map(|id| doc.get(id))
            .filter(|el| ph2d_board_geom::hit(el, p, tol))
            .max_by(|a, b| a.z.cmp(&b.z).then(a.id.cmp(&b.id)))
            .map(|el| el.id)
    }

    /// ⭐ **A forma onde uma ponta largada em `p` se prende** (a de cima, se há várias) e como —
    /// [`anchor_for`] com faixa `band` (mundo). `skip` = formas que não contam (a seta não se
    /// prende à forma de onde está a sair).
    pub fn bind_at(
        &mut self,
        doc: &BoardDoc,
        p: [f64; 2],
        band: f64,
        skip: &[ElementId],
    ) -> Option<(ElementId, Anchor)> {
        self.sync(doc);
        let mut cand = Vec::new();
        self.index.near(Aabb::new(p, p).inflate(band), &mut cand);
        let ids: BTreeSet<ElementId> = cand.into_iter().map(|(id, _)| id).collect();
        let mut els: Vec<&Element> = ids
            .into_iter()
            .filter(|id| !skip.contains(id))
            .filter_map(|id| doc.get(id))
            .collect();
        els.sort_by(|a, b| b.z.cmp(&a.z).then(b.id.cmp(&a.id)));
        els.into_iter()
            .find_map(|el| anchor_for(el, p, band).map(|a| (el.id, a)))
    }
}

fn missing(doc: &BoardDoc, end: End) -> bool {
    matches!(end, End::Bound { target, .. } if doc.get(target).and_then(Element::shape).is_none())
}

/// O afastamento de cada seta que liga o MESMO par de formas (sem ordem) pelo CENTRO: `(i −
/// (n−1)/2) · SPREAD_STEP`, por id — senão a segunda nasceria exactamente por baixo da primeira.
/// Uma ponta num ponto fixo sai sempre dali (o spread não a move), e não entra na conta: senão
/// empurrava a vizinha para fora do vértice sem razão (foto da cena 3, 06/10).
fn spreads(conns: &[&Element]) -> BTreeMap<ElementId, f64> {
    let mut pairs: BTreeMap<(ElementId, ElementId), Vec<ElementId>> = BTreeMap::new();
    for el in conns {
        let c = el.connector().expect("só setas");
        if let [
            End::Bound {
                target: a,
                anchor: Anchor::Center,
            },
            End::Bound {
                target: b,
                anchor: Anchor::Center,
            },
        ] = c.ends()
            && a != b
        {
            pairs.entry((a.min(b), a.max(b))).or_default().push(el.id);
        }
    }
    let mut out = BTreeMap::new();
    for ids in pairs.values().filter(|v| v.len() > 1) {
        let mid = (ids.len() - 1) as f64 / 2.0;
        for (i, id) in ids.iter().enumerate() {
            out.insert(*id, (i as f64 - mid) * SPREAD_STEP);
        }
    }
    out
}
