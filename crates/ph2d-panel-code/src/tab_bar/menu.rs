//! ⭐ **O menu aberto da barra e o submenu em cascata** (dono 10/11: `View ▸ Text rendering ▸ 4 modos`).
//!
//! A mecânica é a dos menus de sistema (macOS, GTK, Windows): a linha com submenu tem a seta `›` à
//! direita; **pairar** sobre ela abre o submenu ao lado (fica acesa enquanto ele está aberto);
//! pairar noutra linha do menu fecha-o; o rato pode sair dos dois sem os fechar. As escolhas são
//! um-entre-N: a em vigor leva o `✓` na coluna da esquerda (em acento). A caminho do submenu aberto
//! (dentro do triângulo entre a ÂNCORA — o último ponto fora da mira, que fica parada a caminho — e a
//! borda dele, alargada meia linha) o rato passa por cima de outras linhas sem o fechar — a «mira»
//! dos menus de sistema; parar mais de [`AIM_REST_NS`] a caminho acaba-a. Escolher fecha tudo; o Esc
//! fecha **um** nível. O submenu abre à direita; se não cabe até à borda da barra, à esquerda; se
//! nem assim, encosta à borda (fica sempre dentro da aba, onde o rato chega).

use super::{TabBar, height, label, width_of};
use crate::mono_label;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{paint_icon, resolve};
use ph2d_editor_core::widget::{
    Popover, paint_popover, paint_popover_row_highlight, pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, Spacing, StrokeToken, Theme};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

/// O popover do menu aberto (e o das dicas).
pub const POPOVER_ID: NodeId = hash_node_id("code_tab_menu_popover");
const SUB_POPOVER_ID: NodeId = hash_node_id("code_tab_submenu_popover");

/// Parado mais do que isto a caminho do submenu, o rato deixa de mirar: a linha onde está ganha (o
/// atraso dos menus de sistema, ~300 ms no macOS e no GTK).
pub const AIM_REST_NS: u128 = 300_000_000;

/// Uma linha do menu aberto.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    /// Um comando: o id, o nome, o atalho (vazio = nenhum) e se vale agora.
    Cmd {
        id: NodeId,
        name: String,
        keys: String,
        live: bool,
    },
    /// Uma nota na letra das dicas, debaixo da linha anterior (não se clica).
    Note(String),
    /// Uma linha que abre um submenu: o id da linha, o nome e as escolhas `(id, nome, em vigor)`.
    Sub {
        id: NodeId,
        name: String,
        rows: Vec<(NodeId, String, bool)>,
    },
}

/// O submenu pintado: a linha-mãe, a caixa e as escolhas.
#[derive(Clone, Debug, PartialEq)]
pub struct SubPainted {
    pub parent: NodeId,
    pub popup: Rect,
    pub rows: Vec<(NodeId, Rect)>,
    /// As escolhas com o `✓` (a em vigor).
    pub checked: Vec<NodeId>,
}

/// O que um carregar no menu pede a quem o usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuPress {
    /// Correr o comando (o menu já fechou).
    Run(NodeId),
    /// O carregar foi do menu (abrir, trocar, fechar); nada a correr.
    Taken,
}

fn row_inks(hot: bool, live: bool) -> (ColorToken, ColorToken) {
    if hot {
        (ColorToken::AccentFg, ColorToken::AccentFg)
    } else if !live {
        (ColorToken::Text3, ColorToken::Text3)
    } else {
        (ColorToken::Text1, ColorToken::Text3)
    }
}

/// O fundo sob uma linha: o acento se acesa, senão o do popover (o escurecimento lê-o).
pub fn row_bg(hot: bool, theme: &Theme) -> ph2d_vector::Color {
    if hot {
        resolve(ColorToken::Accent, *theme)
    } else {
        ph2d_editor_core::widget::Superficie::de(*theme).fundo()
    }
}

fn chevron(scene: &mut VectorScene, right: f32, y: f32, ink: ColorToken, theme: &Theme) {
    let s = INLINE_ICON_PX;
    let r = Rect::new(right - s, y + (height() - s) * 0.5, s, s);
    paint_icon(
        scene,
        IconId::ChevronRight,
        r,
        resolve(ink, *theme),
        StrokeToken::Default.px(),
    );
}

impl<M: Copy + PartialEq> TabBar<M> {
    /// Pinta o menu aberto por baixo do título dele, com `items`, e o submenu aberto ao lado (por cima
    /// de tudo: chame-a por último). Sem menu aberto, a dica sob o rato depois de `pause_ns`.
    pub fn paint_popup(
        &mut self,
        items: &[MenuItem],
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
        (now, pause_ns): (u128, u128),
    ) {
        self.painted.rows.clear();
        self.painted.sub_parents.clear();
        self.painted.popup = None;
        self.painted.sub = None;
        let Some(m) = self.open else {
            return self.paint_tip(scene, ts, theme, now, pause_ns);
        };
        let Some(&(_, title)) = self.painted.titles.iter().find(|t| t.0 == m) else {
            return;
        };
        let (pad, gap, xs) = (Spacing::Sm.px(), Spacing::Lg.px(), Spacing::Xs.px());
        let mut name_w: f32 = 0.0;
        let (mut keys_w, mut note_w): (f32, f32) = (0.0, 0.0);
        for it in items {
            match it {
                MenuItem::Cmd { name, keys, .. } => {
                    name_w = name_w.max(mono_label::width(ts, name));
                    keys_w = keys_w.max(width_of(ts, keys));
                }
                MenuItem::Sub { name, .. } => {
                    name_w = name_w.max(mono_label::width(ts, name));
                    keys_w = keys_w.max(INLINE_ICON_PX);
                }
                MenuItem::Note(n) => note_w = note_w.max(width_of(ts, n)),
            }
        }
        let w = (name_w + gap + keys_w).max(note_w) + pad * 4.0; // LITERAL-PX-OK: o respiro do realce e do texto, dos dois lados
        let rh = height();
        let popup = Rect::new(title.x, title.y + title.h, w, rh * items.len() as f32 + pad);
        paint_popover(&Popover::new(POPOVER_ID), popup, scene, *theme);
        push_popover_clip(popup, scene);
        let mut y = popup.y + pad * 0.5;
        let mut sub_at = None;
        if self.sub.is_some_and(|s| {
            !items
                .iter()
                .any(|i| matches!(i, MenuItem::Sub { id, .. } if *id == s))
        }) {
            self.sub = None;
        }
        for it in items {
            let rr = Rect::new(popup.x + xs, y, popup.w - xs * 2.0, rh);
            let nr = Rect::new(popup.x + pad, y, f32::INFINITY, rh);
            match it {
                MenuItem::Note(n) => {
                    let hr = Rect::new(popup.x + pad, y, note_w, rh);
                    label(ts, scene, n, hr, ColorToken::Text3, theme);
                }
                MenuItem::Cmd {
                    id,
                    name,
                    keys,
                    live,
                } => {
                    let hot = *live && self.hover == Some(*id);
                    if hot {
                        paint_popover_row_highlight(scene, rr, *theme);
                    }
                    let (ink, dim) = row_inks(hot, *live);
                    let c = resolve(ink, *theme);
                    mono_label::paint_on(ts, scene, *theme, name, c, row_bg(hot, theme), nr);
                    let kw = width_of(ts, keys);
                    let kr = Rect::new(popup.x + popup.w - pad * 2.0 - kw, y, kw, rh);
                    label(ts, scene, keys, kr, dim, theme);
                    if *live {
                        self.painted.rows.push((*id, rr));
                    }
                }
                MenuItem::Sub { id, name, rows } => {
                    let hot = self.hover == Some(*id) || self.sub == Some(*id);
                    if hot {
                        paint_popover_row_highlight(scene, rr, *theme);
                    }
                    let (ink, dim) = row_inks(hot, true);
                    let c = resolve(ink, *theme);
                    mono_label::paint_on(ts, scene, *theme, name, c, row_bg(hot, theme), nr);
                    chevron(scene, popup.x + popup.w - pad * 2.0, y, dim, theme);
                    self.painted.rows.push((*id, rr));
                    self.painted.sub_parents.push(*id);
                    if self.sub == Some(*id) {
                        sub_at = Some((*id, rr, rows));
                    }
                }
            }
            y += rh;
        }
        pop_popover_clip(scene);
        self.painted.popup = Some(popup);
        if let Some((id, parent, rows)) = sub_at {
            self.paint_sub(id, popup, parent, rows, scene, ts, theme);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_sub(
        &mut self,
        id: NodeId,
        menu: Rect,
        parent: Rect,
        rows: &[(NodeId, String, bool)],
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let (pad, xs, rh) = (Spacing::Sm.px(), Spacing::Xs.px(), height());
        let check_w = INLINE_ICON_PX + xs;
        let name_w = rows
            .iter()
            .map(|r| mono_label::width(ts, &r.1))
            .fold(0.0, f32::max);
        let w = check_w + name_w + pad * 4.0; // LITERAL-PX-OK: o respiro do realce e do texto, dos dois lados
        let h = rh * rows.len() as f32 + pad;
        let edge = self.painted.header.x + self.painted.header.w;
        let left = self.painted.header.x;
        let x = if menu.x + menu.w + w <= edge {
            menu.x + menu.w
        } else if menu.x - w >= left {
            menu.x - w
        } else {
            (edge - w).max(left)
        };
        let popup = Rect::new(x, parent.y - pad * 0.5, w, h);
        paint_popover(&Popover::new(SUB_POPOVER_ID), popup, scene, *theme);
        push_popover_clip(popup, scene);
        let mut sub = SubPainted {
            parent: id,
            popup,
            rows: Vec::new(),
            checked: Vec::new(),
        };
        let mut y = popup.y + pad * 0.5;
        for (rid, name, on) in rows {
            let rr = Rect::new(popup.x + xs, y, popup.w - xs * 2.0, rh);
            let hot = self.hover == Some(*rid);
            if hot {
                paint_popover_row_highlight(scene, rr, *theme);
            }
            if *on {
                let s = INLINE_ICON_PX;
                let cr = Rect::new(popup.x + pad, y + (rh - s) * 0.5, s, s);
                let ink = if hot {
                    ColorToken::AccentFg
                } else {
                    ColorToken::Accent
                };
                paint_icon(
                    scene,
                    IconId::Check,
                    cr,
                    resolve(ink, *theme),
                    StrokeToken::Default.px(),
                );
                sub.checked.push(*rid);
            }
            let (ink, _) = row_inks(hot, true);
            let nr = Rect::new(popup.x + pad + check_w, y, f32::INFINITY, rh);
            let c = resolve(ink, *theme);
            mono_label::paint_on(ts, scene, *theme, name, c, row_bg(hot, theme), nr);
            sub.rows.push((*rid, rr));
            y += rh;
        }
        pop_popover_clip(scene);
        self.painted.sub = Some(sub);
    }

    fn row_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.painted
            .rows
            .iter()
            .find(|r| r.1.contains(x, y))
            .map(|r| r.0)
    }

    fn sub_row_at(&self, x: f32, y: f32) -> Option<Option<NodeId>> {
        let s = self.painted.sub.as_ref()?;
        s.popup
            .contains(x, y)
            .then(|| s.rows.iter().find(|r| r.1.contains(x, y)).map(|r| r.0))
    }

    fn close(&mut self) {
        self.open = None;
        self.sub = None;
        self.hover = None;
    }

    /// Um carregar primário: `None` = não foi da barra (o menu está fechado e não é um título).
    pub fn press(&mut self, x: f32, y: f32) -> Option<MenuPress> {
        if self.open.is_none() {
            let m = self.title_at(x, y)?;
            self.open = Some(m);
            self.hover = None;
            self.sub = None;
            return Some(MenuPress::Taken);
        }
        if let Some(at) = self.sub_row_at(x, y) {
            return Some(match at {
                Some(id) => {
                    self.close();
                    MenuPress::Run(id)
                }
                None => MenuPress::Taken,
            });
        }
        if let Some(id) = self.row_at(x, y) {
            if self.painted.sub_parents.contains(&id) {
                self.sub = Some(id);
                return Some(MenuPress::Taken);
            }
            self.close();
            return Some(MenuPress::Run(id));
        }
        match self.title_at(x, y) {
            Some(m) if self.open != Some(m) => {
                self.open = Some(m);
                self.sub = None;
            }
            _ => self.close(),
        }
        Some(MenuPress::Taken)
    }

    /// O rato sem botão com um menu aberto: pairar acende a linha; a linha de um submenu abre-o, outra
    /// linha fecha-o; dentro do submenu acende a escolha; outro título troca de menu. `false` = o
    /// menu está fechado (o rato é de outrem).
    pub fn hover_move(&mut self, x: f32, y: f32) -> bool {
        self.hover_move_at(x, y, 0)
    }

    /// [`Self::hover_move`] no relógio `now` (ns): a mira até ao submenu parte da âncora (o último ponto
    /// fora dela) e acaba se o rato parar mais de [`AIM_REST_NS`] a caminho.
    pub fn hover_move_at(&mut self, x: f32, y: f32, now: u128) -> bool {
        if self.open.is_none() {
            return false;
        }
        let rested = now.saturating_sub(std::mem::replace(&mut self.moved_at, now)) > AIM_REST_NS;
        if let Some(m) = self.title_at(x, y)
            && self.open != Some(m)
        {
            self.open = Some(m);
            self.sub = None;
        }
        if let Some(at) = self.sub_row_at(x, y) {
            self.hover = at;
            self.last = Some((x, y));
            return true;
        }
        if let (Some(p), Some(s)) = (self.last, self.painted.sub.as_ref())
            && !rested
            && aims_at(p, (x, y), s.popup)
        {
            return true;
        }
        self.last = Some((x, y));
        self.hover = self.row_at(x, y);
        if let Some(h) = self.hover {
            self.sub = self.painted.sub_parents.contains(&h).then_some(h);
        }
        true
    }

    /// O Esc fecha um nível: o submenu, senão o menu. `true` = fechou algo.
    pub fn escape(&mut self) -> bool {
        if self.sub.take().is_some() {
            self.hover = None;
            return true;
        }
        self.open.take().is_some()
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
