//! ⭐ **Arrastar sobre interruptores** (dono 01/11, o idioma dos olhos do Outliner do Blender).
//!
//! A lei, uma para todos os painéis: premir uma célula VIRA-A; o estado que ela ganhou é a
//! **tinta** do gesto; com o botão em baixo, cada outra célula do MESMO grupo em que o ponteiro
//! entra fica com a tinta (posta, nunca virada — a que já a tem não muda), e reentrar numa célula
//! não faz nada. Soltar acaba o gesto. O segmento entre dois `Move` é cortado contra as células, por
//! isso um rato rápido não salta nenhuma.
//!
//! O painel declara as células ao pintar ([`celula`]) com o estado que MOSTRA; o despacho emite um
//! `WidgetEvent::Click(id)` só para as células que têm de virar para chegar à tinta — o painel
//! continua a tratar um clique, pelo caminho de sempre.

use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

/// Uma célula pintada neste quadro: o grupo (o painel), o estado que se vê e o rect do clique.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Celula {
    grupo: &'static str,
    ligado: bool,
    rect: Rect,
}

thread_local! {
    static CELULAS: RefCell<BTreeMap<NodeId, Celula>> = const { RefCell::new(BTreeMap::new()) };
    static GESTO: RefCell<Option<Gesto>> = const { RefCell::new(None) };
}

/// ⭐ **A lei pura** — um gesto de pintar interruptores.
#[derive(Clone, Debug, PartialEq)]
pub struct Gesto {
    grupo: &'static str,
    tinta: bool,
    tocadas: BTreeSet<NodeId>,
    ultimo: (f32, f32),
}

impl Gesto {
    /// Premir `id`, que mostrava `mostrado`: ele vira, e o estado novo é a tinta.
    #[must_use]
    pub fn premir(id: NodeId, grupo: &'static str, mostrado: bool, x: f32, y: f32) -> Self {
        Self {
            grupo,
            tinta: !mostrado,
            tocadas: BTreeSet::from([id]),
            ultimo: (x, y),
        }
    }

    /// A tinta do gesto — o estado que cada célula tocada passa a ter.
    #[must_use]
    pub fn tinta(&self) -> bool {
        self.tinta
    }

    /// Entrar em `id`, que mostra `mostrado`: `true` = tem de virar para ficar com a tinta. Uma célula
    /// de outro grupo, ou já tocada neste gesto, nunca vira.
    pub fn entrar(&mut self, id: NodeId, grupo: &'static str, mostrado: bool) -> bool {
        if grupo != self.grupo || !self.tocadas.insert(id) {
            return false;
        }
        mostrado != self.tinta
    }
}

/// O intervalo `[t0, t1]` do segmento `a → b` dentro de `r` (Liang–Barsky), ou `None`.
fn corte(a: (f32, f32), b: (f32, f32), r: Rect) -> Option<(f32, f32)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let mut t0 = 0.0_f32;
    let mut t1 = 1.0_f32;
    for (p, q) in [
        (-dx, a.0 - r.x),
        (dx, r.x + r.w - a.0),
        (-dy, a.1 - r.y),
        (dy, r.y + r.h - a.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    (t0 <= t1).then_some((t0, t1))
}

/// As células de `grupo` que o segmento `a → b` atravessa, pela ordem em que o ponteiro entra nelas.
/// Uma célula tapada por outro registo (o topo do `HitIndex` no meio do troço não é ela) fica de fora.
fn atravessadas(
    hit_index: &HitIndex,
    celulas: &BTreeMap<NodeId, Celula>,
    grupo: &'static str,
    a: (f32, f32),
    b: (f32, f32),
) -> Vec<(NodeId, bool)> {
    let mut v: Vec<(f32, NodeId, bool)> = celulas
        .iter()
        .filter(|(_, c)| c.grupo == grupo)
        .filter_map(|(id, c)| {
            let (t0, t1) = corte(a, b, c.rect)?;
            let t = (t0 + t1) * 0.5;
            let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            (hit_index.hit_with_rect(x, y).map(|h| h.0) == Some(*id)).then_some((t0, *id, c.ligado))
        })
        .collect();
    v.sort_by(|p, q| p.0.total_cmp(&q.0));
    v.into_iter().map(|(_, id, on)| (id, on)).collect()
}

/// ⭐ **O pintor declara uma célula** — `ligado` é o estado que ela MOSTRA neste quadro.
pub fn celula(id: NodeId, grupo: &'static str, ligado: bool, rect: Rect) {
    CELULAS.with_borrow_mut(|c| {
        c.insert(
            id,
            Celula {
                grupo,
                ligado,
                rect,
            },
        );
    });
}

/// O pintor esquece as células de `grupo` antes de as pintar de novo (linhas que sumiram).
pub fn limpa_grupo(grupo: &'static str) {
    CELULAS.with_borrow_mut(|c| c.retain(|_, v| v.grupo != grupo));
}

/// Há um gesto em curso (entre o premir e o soltar).
#[must_use]
pub fn em_curso() -> bool {
    GESTO.with_borrow(Option::is_some)
}

/// O despacho: premir em `id`. `Some(id)` = é uma célula, o gesto começou e ela vira.
pub(crate) fn premir(id: NodeId, x: f32, y: f32) -> Option<NodeId> {
    let c = CELULAS.with_borrow(|c| c.get(&id).copied())?;
    GESTO.with_borrow_mut(|g| *g = Some(Gesto::premir(id, c.grupo, c.ligado, x, y)));
    Some(id)
}

/// O despacho: o ponteiro andou até `(x, y)`. Chama `virar` para cada célula atravessada que tem de
/// virar; devolve `true` se havia gesto (o `Move` é dele).
pub(crate) fn mover(hit_index: &HitIndex, x: f32, y: f32, mut virar: impl FnMut(NodeId)) -> bool {
    GESTO.with_borrow_mut(|g| {
        let Some(g) = g.as_mut() else {
            return false;
        };
        let cruzadas =
            CELULAS.with_borrow(|c| atravessadas(hit_index, c, g.grupo, g.ultimo, (x, y)));
        for (id, mostrado) in cruzadas {
            if g.entrar(id, g.grupo, mostrado) {
                virar(id);
            }
        }
        g.ultimo = (x, y);
        true
    })
}

/// O despacho: soltar acaba o gesto.
pub(crate) fn soltar() {
    GESTO.with_borrow_mut(|g| *g = None);
}

#[cfg(test)]
#[path = "toggle_paint_tests.rs"]
mod tests;
