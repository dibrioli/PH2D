//! O teclado nos menus abertos (o idioma de qualquer app de secretária e do VS Code): ↑/↓ movem o
//! realce pelas linhas que valem (saltam as inactivas e as notas, dão a volta), Home/End vão à
//! primeira/última, → abre o submenu da linha realçada (e realça a 1.ª linha dele), ← fecha-o; no
//! menu de topo ←/→ passam ao título vizinho (o `…` incluído). Enter/Espaço corre a linha (ou abre o
//! submenu); Esc fecha um nível. O realce é o MESMO do rato ([`TabBar::hover`]); num nível que rola,
//! a linha realçada fica à vista.

use super::menu::{MenuItem, MenuPress};
use super::{TabBar, height};
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::Key;

/// As linhas que se realçam: (índice em `items`, id, abre um submenu).
fn selectable(items: &[MenuItem]) -> Vec<(usize, NodeId, bool)> {
    items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| match it {
            MenuItem::Cmd { id, live: true, .. } | MenuItem::Choice { id, .. } => {
                Some((i, *id, false))
            }
            MenuItem::Sub { id, .. } => Some((i, *id, true)),
            _ => None,
        })
        .collect()
}

fn sub_rows(items: &[MenuItem], id: NodeId) -> Option<&[MenuItem]> {
    items.iter().find_map(|i| match i {
        MenuItem::Sub { id: s, rows, .. } if *s == id => Some(rows.as_slice()),
        _ => None,
    })
}

impl<M: Copy + PartialEq> TabBar<M> {
    /// As linhas de cada nível aberto, do menu ao submenu mais fundo.
    fn level_items<'a>(&self, items: &'a [MenuItem]) -> Vec<&'a [MenuItem]> {
        let mut out = vec![items];
        for id in self.chain() {
            let Some(rows) = out.last().and_then(|l| sub_rows(l, id)) else {
                break;
            };
            out.push(rows);
        }
        out
    }

    /// Uma tecla com um menu aberto (`items` = as linhas dele). `None` = a tecla não é do menu (o menu
    /// está fechado, ou não é uma tecla de menu): segue para quem vem depois.
    pub fn key(&mut self, items: &[MenuItem], key: Key) -> Option<MenuPress> {
        if !self.is_open() {
            return None;
        }
        let levels = self.level_items(items);
        let in_level = |l: &[MenuItem], h| selectable(l).iter().any(|r| Some(r.1) == h);
        let k = (0..levels.len())
            .rev()
            .find(|&k| in_level(levels[k], self.hover))
            .unwrap_or(levels.len() - 1);
        let rows = selectable(levels[k]);
        let at = rows.iter().position(|r| Some(r.1) == self.hover);
        let hot = at.map(|i| rows[i]);
        match key {
            Key::Up | Key::Down | Key::Home | Key::End if !rows.is_empty() => {
                let n = rows.len();
                let i = match (key, at) {
                    (Key::Home, _) | (Key::Down, None) => 0,
                    (_, None) | (Key::End, _) => n - 1,
                    (Key::Down, Some(i)) => (i + 1) % n,
                    (_, Some(i)) => (i + n - 1) % n,
                };
                let mut c = self.chain();
                c.truncate(k);
                self.set_chain(c);
                self.hover = Some(rows[i].1);
                self.show_row(k, rows[i].0);
            }
            Key::Right | Key::Enter | Key::Space if hot.is_some_and(|h| h.2) => {
                let Some((_, id, _)) = hot else { return None };
                self.open_sub(k, id);
                self.hover =
                    sub_rows(levels[k], id).and_then(|r| selectable(r).first().map(|f| f.1));
            }
            Key::Enter | Key::Space => {
                if let Some((_, id, _)) = hot {
                    self.close();
                    return Some(MenuPress::Run(id));
                }
            }
            Key::Left if k > 0 => {
                let mut c = self.chain();
                c.truncate(k);
                self.hover = c.pop();
                self.set_chain(c);
            }
            Key::Left | Key::Right => self.next_title(key == Key::Right),
            Key::Escape => {
                let parent = self.chain().pop();
                self.escape();
                self.hover = parent;
            }
            Key::Up | Key::Down | Key::Home | Key::End => {}
            _ => return None,
        }
        Some(MenuPress::Taken)
    }

    /// Abre o título vizinho (`…` no fim da fila) e realça a 1.ª linha dele no próximo quadro.
    fn next_title(&mut self, fwd: bool) {
        let mut order: Vec<Option<M>> = self.painted.titles.iter().map(|t| Some(t.0)).collect();
        if self.painted.more.is_some() {
            order.push(None);
        }
        let cur = if self.more { None } else { self.open };
        let n = order.len();
        let Some(i) = order.iter().position(|o| *o == cur).filter(|_| n > 1) else {
            return;
        };
        let j = if fwd { (i + 1) % n } else { (i + n - 1) % n };
        self.open_at(order[j]);
        self.hover = None;
        self.first_pending = true;
    }

    /// [`Self::next_title`] pediu a 1.ª linha do menu novo: realça-a (quem pinta chama com as linhas).
    pub(super) fn take_first(&mut self, items: &[MenuItem]) {
        if std::mem::take(&mut self.first_pending) {
            self.hover = selectable(items).first().map(|r| r.1);
        }
    }

    /// Rola o nível `level` (se rola) até a linha `i` (índice nas linhas dele) ficar à vista.
    fn show_row(&mut self, level: usize, i: usize) {
        let p = &self.painted;
        let geo = if level == 0 {
            p.popup.zip(p.arrows)
        } else {
            let s = p.sub.iter().chain(&p.deeper).nth(level - 1);
            s.and_then(|s| s.arrows.map(|a| (s.popup, a)))
        };
        let Some((popup, _)) = geo else {
            return;
        };
        let rh = height();
        let (top, view) = (rh * i as f32, popup.h - rh * 2.0); // LITERAL-PX-OK: as duas setas ▲▼
        self.scroll_to(level, f32::INFINITY);
        let s = &mut self.scroll[level];
        *s = s.min(top).max(top + rh - view);
    }
}
