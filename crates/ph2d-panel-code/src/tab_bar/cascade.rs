//! A posição dos menus na janela e o rato sobre os N níveis da cascata ([`super::menu`]).

use super::menu::{AIM_REST_NS, MenuItem, MenuPress};
use super::{TabBar, height};
use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

/// `h` de altura a partir de `y`, dentro de `b`: encosta ao fundo; mais alto que `b`, ocupa-o (e rola).
fn fit_y(x: f32, y: f32, w: f32, h: f32, b: Rect) -> Rect {
    let h = h.min(b.h);
    Rect::new(x, y.min(b.y + b.h - h).max(b.y), w, h)
}

/// O menu de um título: por baixo dele, dentro de `b` (encostado à direita e ao fundo).
pub(super) fn below(title: Rect, w: f32, h: f32, b: Rect) -> Rect {
    let x = title.x.min(b.x + b.w - w).max(b.x);
    fit_y(x, title.y + title.h, w, h, b)
}

/// Um submenu à altura da linha-mãe `parent`: à direita do `menu`, senão à esquerda; sem lugar de
/// nenhum lado, por baixo do menu, encostado à borda (nunca por cima das linhas da mãe).
pub(super) fn beside(menu: Rect, parent: Rect, w: f32, h: f32, b: Rect) -> Rect {
    let (right, top) = (b.x + b.w, parent.y - Spacing::Sm.px() * 0.5);
    let (x, y) = if menu.x + menu.w + w <= right {
        (menu.x + menu.w, top)
    } else if menu.x - w >= b.x {
        (menu.x - w, top)
    } else {
        ((right - w).max(b.x), menu.y + menu.h)
    };
    fit_y(x, y, w, h, b)
}

/// Um nível aberto como o rato o vê.
struct Level {
    popup: Rect,
    rows: Vec<(NodeId, Rect)>,
    parents: Vec<NodeId>,
    arrows: Option<(Rect, Rect)>,
}

impl<M: Copy + PartialEq> TabBar<M> {
    /// Onde os menus podem ir: a janela; sem ela, a largura da barra e nenhum limite na vertical.
    pub(super) fn bounds(&self) -> Rect {
        let h = self.painted.header;
        self.window
            .unwrap_or(Rect::new(h.x, f32::MIN, h.w, f32::INFINITY))
    }

    /// As linhas-mãe abertas, do 1.º submenu ao mais fundo.
    #[must_use]
    pub fn chain(&self) -> Vec<NodeId> {
        self.sub
            .into_iter()
            .chain(self.deeper.iter().copied())
            .collect()
    }

    fn set_chain(&mut self, mut c: Vec<NodeId>) {
        self.scroll.truncate(c.len() + 1);
        self.sub = (!c.is_empty()).then(|| c.remove(0));
        self.deeper = c;
    }

    /// Abre, no nível `level`, o submenu da linha `id` (fecha os mais fundos).
    fn open_sub(&mut self, level: usize, id: NodeId) {
        let mut c = self.chain();
        c.truncate(level);
        if c.len() == level {
            c.push(id);
        }
        self.set_chain(c);
    }

    /// Corta a cadeia no primeiro elo que já não é um submenu das linhas `items`.
    pub(super) fn prune(&mut self, items: &[MenuItem]) {
        let (mut ok, mut cur) = (Vec::new(), items);
        for id in self.chain() {
            let Some(rows) = cur.iter().find_map(|i| match i {
                MenuItem::Sub { id: s, rows, .. } if *s == id => Some(rows),
                _ => None,
            }) else {
                break;
            };
            ok.push(id);
            cur = rows;
        }
        if ok.len() != self.chain().len() {
            self.set_chain(ok);
        }
    }

    /// A rolagem do nível `level`, presa a `0..=max` (e guardada assim).
    pub(super) fn scroll_to(&mut self, level: usize, max: f32) -> f32 {
        if self.scroll.len() <= level {
            self.scroll.resize(level + 1, 0.0);
        }
        let s = self.scroll[level].clamp(0.0, max.max(0.0));
        self.scroll[level] = s;
        s
    }

    fn levels(&self) -> Vec<Level> {
        let p = &self.painted;
        let top = p.popup.map(|popup| Level {
            popup,
            rows: p.rows.clone(),
            parents: p.sub_parents.clone(),
            arrows: p.arrows,
        });
        let subs = p.sub.iter().chain(&p.deeper).map(|s| Level {
            popup: s.popup,
            rows: s.rows.clone(),
            parents: s.parents.clone(),
            arrows: s.arrows,
        });
        top.into_iter().chain(subs).collect()
    }

    /// O nível mais fundo (o de cima) sob `(x, y)`.
    fn level_at(&self, x: f32, y: f32) -> Option<(usize, Level)> {
        let mut ls: Vec<_> = self.levels().into_iter().enumerate().collect();
        ls.reverse();
        ls.into_iter().find(|l| l.1.popup.contains(x, y))
    }

    /// Um menu aberto tem um nível em `(x, y)`: o rato ali é da barra, por cima do que estiver por baixo.
    #[must_use]
    pub fn floats_at(&self, x: f32, y: f32) -> bool {
        self.is_open() && self.level_at(x, y).is_some()
    }

    /// Um menu aberto: o de um título ou o de transbordo.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open.is_some() || self.more
    }

    /// O título sob o menu aberto.
    pub(super) fn anchor(&self) -> Option<Rect> {
        if self.more {
            return self.painted.more;
        }
        let m = self.open?;
        self.painted.titles.iter().find(|t| t.0 == m).map(|t| t.1)
    }

    /// O transbordo `…` está em `(x, y)`?
    fn on_more(&self, x: f32, y: f32) -> bool {
        self.painted.more.is_some_and(|r| r.contains(x, y))
    }

    /// Abre o menu do título `m`, ou o transbordo com `None`.
    fn open_at(&mut self, m: Option<M>) {
        self.more = m.is_none();
        self.open = m;
        self.set_chain(Vec::new());
        self.scroll.clear();
    }

    fn close(&mut self) {
        self.open_at(None);
        self.more = false;
        self.hover = None;
    }

    /// Um carregar primário: `None` = não foi da barra (o menu está fechado e não é um título).
    pub fn press(&mut self, x: f32, y: f32) -> Option<MenuPress> {
        if !self.is_open() {
            if self.on_more(x, y) {
                self.open_at(None);
            } else {
                self.open_at(Some(self.title_at(x, y)?));
            }
            self.hover = None;
            return Some(MenuPress::Taken);
        }
        if let Some((k, l)) = self.level_at(x, y) {
            if let Some((up, down)) = l.arrows
                && (up.contains(x, y) || down.contains(x, y))
            {
                let step = if up.contains(x, y) {
                    -height()
                } else {
                    height()
                };
                self.scroll_to(k, f32::INFINITY);
                self.scroll[k] += step;
                return Some(MenuPress::Taken);
            }
            return Some(match l.rows.iter().find(|r| r.1.contains(x, y)) {
                Some(&(id, _)) if l.parents.contains(&id) => {
                    self.open_sub(k, id);
                    MenuPress::Taken
                }
                Some(&(id, _)) => {
                    self.close();
                    MenuPress::Run(id)
                }
                None => MenuPress::Taken,
            });
        }
        match self.title_at(x, y) {
            Some(m) if self.open != Some(m) => self.open_at(Some(m)),
            None if !self.more && self.on_more(x, y) => self.open_at(None),
            _ => self.close(),
        }
        Some(MenuPress::Taken)
    }

    /// A roda sobre um nível aberto: rola o que rola; `true` = era do menu.
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) -> bool {
        let Some((k, l)) = self.level_at(x, y).filter(|_| self.is_open()) else {
            return false;
        };
        if l.arrows.is_some() {
            self.scroll_to(k, f32::INFINITY);
            self.scroll[k] -= dy;
        }
        true
    }

    /// O rato sem botão com um menu aberto: pairar acende a linha; a linha de um submenu abre-o, outra
    /// linha do mesmo nível fecha os mais fundos; outro título troca de menu. `false` = o menu está
    /// fechado (o rato é de outrem).
    pub fn hover_move(&mut self, x: f32, y: f32) -> bool {
        self.hover_move_at(x, y, 0)
    }

    /// [`Self::hover_move`] no relógio `now` (ns): a mira até ao submenu parte da âncora (o último ponto
    /// fora dela) e acaba se o rato parar mais de [`AIM_REST_NS`] a caminho.
    pub fn hover_move_at(&mut self, x: f32, y: f32, now: u128) -> bool {
        if !self.is_open() {
            return false;
        }
        let rested = now.saturating_sub(std::mem::replace(&mut self.moved_at, now)) > AIM_REST_NS;
        if let Some(m) = self.title_at(x, y)
            && self.open != Some(m)
        {
            self.open_at(Some(m));
        } else if !self.more && self.on_more(x, y) {
            self.open_at(None);
        }
        let at = self.level_at(x, y);
        let levels = self.levels();
        // A mira vai ao nível aberto a seguir ao do rato (fora de todos: ao mais fundo).
        let target = at
            .as_ref()
            .map_or(levels.len().saturating_sub(1), |a| a.0 + 1);
        if target >= 1
            && let (Some(p), Some(t)) = (self.last, levels.get(target))
            && !rested
            && aims_at(p, (x, y), t.popup)
        {
            return true;
        }
        self.last = Some((x, y));
        self.hover = None;
        if let Some((k, l)) = at {
            self.hover = l.rows.iter().find(|r| r.1.contains(x, y)).map(|r| r.0);
            if let Some(h) = self.hover {
                if l.parents.contains(&h) {
                    self.open_sub(k, h);
                } else {
                    let mut c = self.chain();
                    c.truncate(k);
                    self.set_chain(c);
                }
            }
        }
        true
    }

    /// O Esc fecha um nível: o submenu mais fundo, senão o menu. `true` = fechou algo.
    pub fn escape(&mut self) -> bool {
        let mut c = self.chain();
        if c.pop().is_some() {
            self.set_chain(c);
            self.hover = None;
            return true;
        }
        std::mem::take(&mut self.more) | self.open.take().is_some()
    }
}

/// `to` está no triângulo entre `from` e a borda de `sub` virada para ele (a mira até ao submenu)?
fn aims_at(from: (f32, f32), to: (f32, f32), sub: Rect) -> bool {
    let edge = if from.0 <= sub.x {
        sub.x
    } else {
        sub.x + sub.w
    };
    let slack = height() * 0.5;
    let (a, b) = ((edge, sub.y - slack), (edge, sub.y + sub.h + slack));
    let side = |p: (f32, f32), q: (f32, f32), r: (f32, f32)| {
        (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0)
    };
    let (d1, d2, d3) = (side(from, a, to), side(a, b, to), side(b, from, to));
    let moved = (to.0 - from.0).abs() + (to.1 - from.1).abs() > 0.0;
    moved && ((d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0) || (d1 <= 0.0 && d2 <= 0.0 && d3 <= 0.0))
}
