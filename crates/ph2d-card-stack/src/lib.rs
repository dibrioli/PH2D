//! ⭐ **A lista de cartões com mola** — o arrasto do Kanban (TrelloClone) tirado da família para que
//! todo arrastar-reordenar voe igual (memória `feedback_every_drag_reorder_flies_like_the_kanban_card`,
//! `docs/CartoesUI/02_especificacao.md` §4).
//!
//! As peças soltas ([`edge_depth`], [`slot_index`], [`Tilt`], [`flight_pivot`]) servem o Kanban, que
//! tem a sua própria disposição em duas dimensões; o [`Stack`] junta-as numa lista VERTICAL com chave
//! (a de um painel: secção = cartão). A lei da mola é a do chamador (`UiMotion::law(Role::Travel)`;
//! `None` = movimento reduzido, tudo chega no quadro).

use ph2d_spring::{Follow, Spring};
use std::collections::BTreeMap;

/// O que voa inclina 4° (o Trello) e roda em torno de `(50 %, 60 %)`.
pub const FLIGHT_TILT_DEG: f64 = 4.0;
pub const FLIGHT_PIVOT_Y: f64 = 0.6;
/// A faixa junto à borda que rola durante um arrasto, e a velocidade no fundo dela.
pub const EDGE_SCROLL_BAND: f32 = 64.0;
pub const EDGE_SCROLL_MAX_PX_S: f32 = 900.0;

pub mod metrics;

/// As sombras do cartão: `(dy, desfoque)` de cada camada (as cores são de quem pinta).
pub mod shadow {
    pub const SHADOW_NEAR: (f64, f64) = (1.0, 1.0);
    pub const SHADOW_RIM: (f64, f64) = (0.0, 1.0);
    /// A sombra de pairar no claro (no escuro é um anel de 1 px).
    pub const SHADOW_HOVER_LIGHT: (f64, f64) = (4.0, 8.0);
    pub const SHADOW_FLIGHT_DARK: (f64, f64) = (14.0, 30.0);
    pub const SHADOW_FLIGHT_LIGHT: (f64, f64) = (16.0, 32.0);
    pub const SHADOW_FLIGHT_NEAR: (f64, f64) = (3.0, 8.0);
    pub const HOVER_RING: f32 = 1.0;
}

/// A fundura `0 → 1` de `p` na faixa junto à borda `[lo, hi]` (negativa = a de cima/esquerda).
#[must_use]
pub fn edge_depth(p: f32, lo: f32, hi: f32) -> f32 {
    let band = EDGE_SCROLL_BAND.min((hi - lo) / 3.0).max(1.0);
    if p < lo + band {
        -((lo + band - p) / band).min(1.0)
    } else if p > hi - band {
        ((p - (hi - band)) / band).min(1.0)
    } else {
        0.0
    }
}

/// Px a rolar neste quadro para uma fundura `depth`.
#[must_use]
pub fn edge_scroll_px(depth: f32, dt: f64) -> f64 {
    f64::from(depth) * (f64::from(EDGE_SCROLL_MAX_PX_S) * dt)
}

/// ⭐ Onde cai o que voa: o índice entre os vizinhos (`extents`, SEM o que voa e sem o vão — senão o
/// vão empurra a fronteira e o alvo oscila) cujo meio o centro `center` já passou.
#[must_use]
pub fn slot_index(extents: impl IntoIterator<Item = f32>, gap: f32, center: f32) -> usize {
    let mut acc = 0.0_f32;
    let mut index = 0usize;
    for e in extents {
        if acc + e / 2.0 > center {
            break;
        }
        acc += e + gap;
        index += 1;
    }
    index
}

/// O pivô da inclinação de um retângulo `w × h` (desde o canto dele).
#[must_use]
pub fn flight_pivot(w: f32, h: f32) -> (f64, f64) {
    (f64::from(w) / 2.0, f64::from(h) * FLIGHT_PIVOT_Y)
}

/// A inclinação de quem voa (graus), com mola: sobe a [`FLIGHT_TILT_DEG`] ao levantar e volta a 0.
#[derive(Clone, Copy, Debug, Default)]
pub struct Tilt {
    pub spring: Follow,
}

impl Tilt {
    /// Mira (voa ou não) e avança. `true` = ainda se mexe.
    pub fn update(&mut self, flying: bool, dt: f64, law: Option<Spring>) -> bool {
        // Nasce a 0 (o 1.º alvo de uma mola é a posição dela), e só então mira.
        if !self.spring.seeded {
            self.spring.aim(0.0);
        }
        self.spring.aim(if flying { FLIGHT_TILT_DEG } else { 0.0 });
        self.spring.advance(dt, law)
    }

    #[must_use]
    pub fn deg(&self) -> f64 {
        self.spring.pos
    }
}

/// Um cartão de altura `h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item<K> {
    pub key: K,
    pub h: f32,
}

/// O cartão em voo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flight<K> {
    pub key: K,
    /// O ponteiro (ecrã) e onde agarrou, dentro do cartão.
    pub at: (f32, f32),
    pub grab: (f32, f32),
    pub size: (f32, f32),
    /// O índice onde cai (entre os outros); `None` = fora da lista: o vão fecha-se.
    pub to: Option<usize>,
}

impl<K> Flight<K> {
    /// O canto do cartão no ecrã: a mão menos a pega.
    #[must_use]
    pub fn ghost(&self) -> (f32, f32) {
        (self.at.0 - self.grab.0, self.at.1 - self.grab.1)
    }
}

/// A disposição: `y` de cada cartão desde o topo da lista, o vão e a altura do conteúdo.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed<K> {
    pub ys: Vec<(K, f32)>,
    /// O vão `(y, h)` onde o que voa cairia.
    pub slot: Option<(f32, f32)>,
    pub content_h: f32,
}

/// ⭐ A disposição-ALVO (sem mola): os cartões pela ordem, sem o que voa, com o vão dele aberto.
#[must_use]
pub fn targets<K: Copy + PartialEq>(
    items: &[Item<K>],
    gap: f32,
    flight: Option<&Flight<K>>,
) -> Placed<K> {
    let lifted = flight.map(|f| f.key);
    let drop_at = flight.and_then(|f| f.to.map(|i| (i, f.size.1)));
    let mut y = 0.0_f32;
    let mut ys = Vec::with_capacity(items.len());
    let mut slot = None;
    let mut shown = 0usize;
    let mut open = |y: &mut f32, h: f32| {
        slot = Some((*y, h));
        *y += h + gap;
    };
    for it in items {
        if Some(it.key) == lifted {
            continue;
        }
        if let Some((i, h)) = drop_at
            && i == shown
        {
            open(&mut y, h);
        }
        ys.push((it.key, y));
        y += it.h + gap;
        shown += 1;
    }
    if let Some((i, h)) = drop_at
        && i >= shown
    {
        open(&mut y, h);
    }
    Placed {
        ys,
        slot,
        content_h: (y - gap).max(0.0),
    }
}

/// ⭐ Uma lista vertical de cartões com mola: o vão que abre, o voo inclinado, o assentar.
#[derive(Clone, Debug)]
pub struct Stack<K: Ord> {
    springs: BTreeMap<K, Follow>,
    tilt: Tilt,
    flight: Option<Flight<K>>,
    /// O cartão largado agora: parte de onde a mão o deixou (`y` desde o topo da lista).
    landed: Option<(K, f32)>,
    /// T41: o cartão que REGRESSA de fora da coluna (o canvas): o desvio em `x` até à lista, com mola.
    arriving: BTreeMap<K, Follow>,
    /// T41: a altura VISTA de cada cartão (dobrar, desdobrar, cartão ↔ fantasma) segue a medida.
    heights: BTreeMap<K, Follow>,
    heights_moving: bool,
    moving: bool,
}

// À mão: o `derive` pediria `K: Default`, e uma chave (um `NodeId`) não tem omissão.
impl<K: Ord> Default for Stack<K> {
    fn default() -> Self {
        Self {
            springs: BTreeMap::new(),
            tilt: Tilt::default(),
            flight: None,
            landed: None,
            arriving: BTreeMap::new(),
            heights: BTreeMap::new(),
            heights_moving: false,
            moving: false,
        }
    }
}

impl<K: Ord + Copy> Stack<K> {
    #[must_use]
    pub fn flight(&self) -> Option<&Flight<K>> {
        self.flight.as_ref()
    }

    /// A inclinação de quem voa (e de quem acabou de assentar, a voltar a 0).
    #[must_use]
    pub fn tilt_deg(&self) -> f64 {
        self.tilt.deg()
    }

    /// Ainda há mola a mexer-se (quem pinta pede outro quadro).
    #[must_use]
    pub fn moving(&self) -> bool {
        self.moving || self.flight.is_some()
    }

    /// Levanta `key` (o vão abre de início onde ele estava: nada se mexe no 1.º quadro).
    pub fn lift(
        &mut self,
        key: K,
        at: (f32, f32),
        grab: (f32, f32),
        size: (f32, f32),
        index: usize,
    ) {
        self.flight = Some(Flight {
            key,
            at,
            grab,
            size,
            to: Some(index),
        });
    }

    /// A mão anda. `top` = o topo da lista no ecrã, ou `None` com o ponteiro fora dela.
    pub fn drag(&mut self, items: &[Item<K>], gap: f32, at: (f32, f32), top: Option<f32>) {
        let Some(f) = self.flight.as_mut() else {
            return;
        };
        f.at = at;
        f.to = top.map(|top| {
            let center = at.1 - f.grab.1 + f.size.1 / 2.0 - top;
            let key = f.key;
            slot_index(
                items.iter().filter(|it| it.key != key).map(|it| it.h),
                gap,
                center,
            )
        });
    }

    /// Larga: devolve `(chave, índice novo)` se caiu na lista; o cartão assenta a partir da mão.
    pub fn land(&mut self, top: f32) -> Option<(K, usize)> {
        let f = self.flight.take()?;
        let to = f.to?;
        self.landed = Some((f.key, f.ghost().1 - top));
        Some((f.key, to))
    }

    /// ⭐ C5: larga FORA da lista — devolve o voo sem assentar (o cartão passa a solto no canvas).
    pub fn detach(&mut self) -> Option<Flight<K>> {
        let f = self.flight.take()?;
        self.springs.remove(&f.key);
        Some(f)
    }

    /// ⭐ T41: `key` regressa à lista de `from` (`x` desde a coluna, `y` desde o topo): as duas molas
    /// partem de lá e assentam no lugar dele — nada salta.
    pub fn arrive(&mut self, key: K, from: (f32, f32)) {
        self.landed = Some((key, from.1));
        let mut x = Follow::default();
        x.jump(f64::from(from.0));
        x.aim(0.0);
        self.arriving.insert(key, x);
    }

    /// O desvio em `x` de quem ainda regressa (`None` = está na coluna).
    #[must_use]
    pub fn arriving_dx(&self, key: K) -> Option<f32> {
        #[allow(clippy::cast_possible_truncation)]
        self.arriving.get(&key).map(|f| f.pos as f32)
    }

    /// T41: `key` ENTRA na lista com a altura vista a partir de `from` (a 1.ª altura deixa de chegar
    /// sem animar — o cartão abre-se em vez de aparecer).
    pub fn enter(&mut self, key: K, from: f32) {
        self.heights.entry(key).or_default().jump(f64::from(from));
    }

    /// ⭐ T41: troca a altura medida de cada item pela altura com mola (a 1.ª chega sem animar) e
    /// devolve as chaves que ainda crescem ou encolhem — quem pinta recorta-as à altura vista.
    pub fn smooth_heights(
        &mut self,
        items: &mut [Item<K>],
        dt: f64,
        law: Option<Spring>,
    ) -> Vec<K> {
        let mut vivas = Vec::new();
        for it in items.iter_mut() {
            let s = self.heights.entry(it.key).or_default();
            s.aim(f64::from(it.h));
            if s.advance(dt, law) {
                vivas.push(it.key);
            }
            #[allow(clippy::cast_possible_truncation)]
            {
                it.h = s.pos as f32;
            }
        }
        self.heights
            .retain(|k, _| items.iter().any(|it| it.key == *k));
        self.heights_moving = !vivas.is_empty();
        vivas
    }

    /// Cancela o voo (Esc): o cartão volta ao seu lugar com mola, a partir da mão.
    pub fn cancel(&mut self, top: f32) {
        if let Some(f) = self.flight.take() {
            self.landed = Some((f.key, f.ghost().1 - top));
        }
    }

    /// ⭐ A disposição VISTA: mira as molas nos alvos, avança `dt` e devolve os `y` com mola.
    pub fn place(
        &mut self,
        items: &[Item<K>],
        gap: f32,
        dt: f64,
        law: Option<Spring>,
    ) -> Placed<K> {
        let mut target = targets(items, gap, self.flight.as_ref());
        let landed = self.landed.take();
        let mut moving = false;
        for (key, y) in &mut target.ys {
            let s = self.springs.entry(*key).or_default();
            if let Some((k, from)) = landed
                && k == *key
            {
                s.jump(f64::from(from));
            }
            s.aim(f64::from(*y));
            moving |= s.advance(dt, law);
            #[allow(clippy::cast_possible_truncation)]
            {
                *y = s.pos as f32;
            }
        }
        self.arriving.retain(|_, x| {
            let m = x.advance(dt, law);
            moving |= m;
            m
        });
        let lifted = self.flight.map(|f| f.key);
        self.springs
            .retain(|k, _| Some(*k) == lifted || items.iter().any(|it| it.key == *k));
        moving |= self.tilt.update(self.flight.is_some(), dt, law);
        self.moving = moving || self.heights_moving;
        target
    }
}

/// Move `key` para `to` (índice entre os OUTROS) numa ordem.
pub fn reorder<K: PartialEq>(order: &mut Vec<K>, key: &K, to: usize) {
    if let Some(from) = order.iter().position(|k| k == key) {
        let k = order.remove(from);
        order.insert(to.min(order.len()), k);
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
