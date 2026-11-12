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
//! nem assim, por baixo do menu, encostado à borda (fica dentro da aba, onde o rato chega, e nunca
//! tapa a mãe).

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

/// Uma linha do menu aberto (o de um título e o submenu pintam-se pela MESMA função).
#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    /// Um comando: o id, o nome, o atalho (vazio = nenhum) e se vale agora (senão a cinzento e
    /// não se clica).
    Cmd {
        id: NodeId,
        name: String,
        keys: String,
        live: bool,
    },
    /// Uma escolha um-entre-N: a em vigor leva o `✓` na coluna da esquerda.
    Choice { id: NodeId, name: String, on: bool },
    /// Uma nota na letra das dicas, debaixo da linha anterior (não se clica).
    Note(String),
    /// Uma linha que abre um submenu com as linhas `rows`.
    Sub {
        id: NodeId,
        name: String,
        rows: Vec<MenuItem>,
    },
}

impl MenuItem {
    /// As escolhas `(id, nome, em vigor)` como linhas de um submenu.
    #[must_use]
    pub fn choices(rows: impl IntoIterator<Item = (NodeId, String, bool)>) -> Vec<Self> {
        rows.into_iter()
            .map(|(id, name, on)| Self::Choice { id, name, on })
            .collect()
    }

    /// As linhas que se escolhem, pela ordem: os comandos que valem e, no lugar de um submenu, as
    /// escolhas dele com a mãe (`Some(mãe)`).
    #[must_use]
    pub fn leaves(items: &[Self]) -> Vec<(NodeId, Option<NodeId>)> {
        let mut out = Vec::new();
        for it in items {
            match it {
                Self::Cmd { id, live: true, .. } | Self::Choice { id, .. } => out.push((*id, None)),
                Self::Sub { id: p, rows, .. } => {
                    out.extend(Self::leaves(rows).into_iter().map(|r| (r.0, Some(*p))));
                }
                _ => {}
            }
        }
        out
    }
}

/// Uma linha como saiu pintada (os testes comparam o menu de um título com o submenu do `…`).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// O comando, a escolha ou a mãe de um submenu; `None` = uma nota.
    pub id: Option<NodeId>,
    pub name: String,
    /// O atalho pintado (vazio = nenhum).
    pub keys: String,
    /// A distância da borda direita do atalho à do menu (o atalho alinha-se à direita).
    pub keys_inset: f32,
    /// Clica-se (corre, escolhe, abre)?
    pub live: bool,
    /// Uma escolha: se leva o `✓`.
    pub check: Option<bool>,
}

/// O submenu pintado: a linha-mãe, a caixa e as escolhas.
#[derive(Clone, Debug, PartialEq)]
pub struct SubPainted {
    pub parent: NodeId,
    pub popup: Rect,
    /// As linhas que se clicam e a caixa delas.
    pub rows: Vec<(NodeId, Rect)>,
    /// As escolhas com o `✓` (a em vigor).
    pub checked: Vec<NodeId>,
    /// Todas as linhas como saíram (as inactivas e as notas incluídas).
    pub lines: Vec<Line>,
}

/// O que um carregar no menu pede a quem o usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuPress {
    /// Correr o comando (o menu já fechou).
    Run(NodeId),
    /// O carregar foi do menu (abrir, trocar, fechar); nada a correr.
    Taken,
}

/// A linha do transbordo `…` que abre o submenu do `i`-ésimo título.
#[must_use]
pub fn more_row(i: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("code_tab_menu_more_{i}"))
}

/// O menu do transbordo `…` (a barra estreita, o idioma do VS Code): cada título escondido
/// `(linha, nome, linhas do menu dele)` vira um submenu com as MESMAS linhas do menu desse título
/// (atalhos, inactivas a cinzento, notas); um submenu dele (o 3.º nível) entra como a nota com o nome
/// e as escolhas por baixo.
pub fn more_menu(hidden: impl Iterator<Item = (NodeId, String, Vec<MenuItem>)>) -> Vec<MenuItem> {
    let inline = |it: MenuItem| match it {
        MenuItem::Sub { name, rows, .. } => {
            std::iter::once(MenuItem::Note(name)).chain(rows).collect()
        }
        it => vec![it],
    };
    hidden
        .map(|(id, name, items)| MenuItem::Sub {
            id,
            name,
            rows: items.into_iter().flat_map(inline).collect(),
        })
        .collect()
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

/// A largura de um menu com `items` e a da coluna do `✓` (zero sem escolhas).
fn measure(items: &[MenuItem], ts: &mut TextSystem) -> (f32, f32) {
    let (pad, gap, xs) = (Spacing::Sm.px(), Spacing::Lg.px(), Spacing::Xs.px());
    let (mut name_w, mut keys_w, mut note_w): (f32, f32, f32) = (0.0, 0.0, 0.0);
    let mut check_w: f32 = 0.0;
    for it in items {
        match it {
            MenuItem::Cmd { name, keys, .. } => {
                name_w = name_w.max(mono_label::width(ts, name));
                keys_w = keys_w.max(width_of(ts, keys));
            }
            MenuItem::Choice { name, .. } => {
                name_w = name_w.max(mono_label::width(ts, name));
                check_w = INLINE_ICON_PX + xs;
            }
            MenuItem::Sub { name, .. } => {
                name_w = name_w.max(mono_label::width(ts, name));
                keys_w = keys_w.max(INLINE_ICON_PX);
            }
            MenuItem::Note(n) => note_w = note_w.max(width_of(ts, n)),
        }
    }
    let right = if keys_w > 0.0 { gap + keys_w } else { 0.0 };
    let w = (check_w + name_w + right).max(note_w) + pad * 4.0; // LITERAL-PX-OK: o respiro do realce e do texto, dos dois lados
    (w, check_w)
}

/// O que [`TabBar::paint_lines`] pintou.
#[derive(Default)]
struct Laid {
    rows: Vec<(NodeId, Rect)>,
    parents: Vec<NodeId>,
    checked: Vec<NodeId>,
    lines: Vec<Line>,
    /// A linha do submenu aberto: o índice em `items` e a caixa.
    open: Option<(usize, Rect)>,
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
        self.painted.lines.clear();
        self.painted.popup = None;
        self.painted.sub = None;
        if !self.is_open() {
            return self.paint_tip(scene, ts, theme, now, pause_ns);
        }
        let Some(title) = self.anchor() else {
            return;
        };
        if self.sub.is_some_and(|s| {
            !items
                .iter()
                .any(|i| matches!(i, MenuItem::Sub { id, .. } if *id == s))
        }) {
            self.sub = None;
        }
        let (w, check_w) = measure(items, ts);
        let h = height() * items.len() as f32 + Spacing::Sm.px();
        let popup = Rect::new(title.x, title.y + title.h, w, h);
        let laid = self.paint_lines(items, popup, check_w, true, scene, ts, theme);
        self.painted.rows = laid.rows;
        self.painted.sub_parents = laid.parents;
        self.painted.lines = laid.lines;
        self.painted.popup = Some(popup);
        if let Some((i, parent)) = laid.open
            && let MenuItem::Sub { id, rows, .. } = &items[i]
        {
            self.paint_sub(*id, popup, parent, rows, scene, ts, theme);
        }
    }

    /// Pinta `items` no popover `popup` (o menu de um título, ou o submenu com `top = false`, onde uma
    /// linha de submenu não abre nada): o `✓` na coluna `check_w`, o nome, o atalho à direita; a
    /// linha inactiva a cinzento, sem realce e fora das que se clicam.
    #[allow(clippy::too_many_arguments)]
    fn paint_lines(
        &self,
        items: &[MenuItem],
        popup: Rect,
        check_w: f32,
        top: bool,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) -> Laid {
        let (pad, xs, rh) = (Spacing::Sm.px(), Spacing::Xs.px(), height());
        let id = if top { POPOVER_ID } else { SUB_POPOVER_ID };
        paint_popover(&Popover::new(id), popup, scene, *theme);
        push_popover_clip(popup, scene);
        let mut laid = Laid::default();
        let mut y = popup.y + pad * 0.5;
        let right = popup.x + popup.w - pad * 2.0;
        for (i, it) in items.iter().enumerate() {
            let rr = Rect::new(popup.x + xs, y, popup.w - xs * 2.0, rh);
            let nr = Rect::new(popup.x + pad + check_w, y, f32::INFINITY, rh);
            let (rid, name, keys, live, check, opens) = match it {
                MenuItem::Note(n) => {
                    let hr = Rect::new(popup.x + pad, y, width_of(ts, n), rh);
                    label(ts, scene, n, hr, ColorToken::Text3, theme);
                    laid.lines.push(Line {
                        id: None,
                        name: n.clone(),
                        keys: String::new(),
                        keys_inset: 0.0,
                        live: false,
                        check: None,
                    });
                    y += rh;
                    continue;
                }
                MenuItem::Cmd {
                    id,
                    name,
                    keys,
                    live,
                } => (*id, name, keys.as_str(), *live, None, false),
                MenuItem::Choice { id, name, on } => (*id, name, "", true, Some(*on), false),
                MenuItem::Sub { id, name, .. } => (*id, name, "", top, None, true),
            };
            let hot = live && (self.hover == Some(rid) || (opens && self.sub == Some(rid)));
            if hot {
                paint_popover_row_highlight(scene, rr, *theme);
            }
            let (ink, dim) = row_inks(hot, live);
            if check == Some(true) {
                let s = INLINE_ICON_PX;
                let cr = Rect::new(popup.x + pad, y + (rh - s) * 0.5, s, s);
                let mark = if hot {
                    ColorToken::AccentFg
                } else {
                    ColorToken::Accent
                };
                let stroke = StrokeToken::Default.px();
                paint_icon(scene, IconId::Check, cr, resolve(mark, *theme), stroke);
                laid.checked.push(rid);
            }
            let c = resolve(ink, *theme);
            mono_label::paint_on(ts, scene, *theme, name, c, row_bg(hot, theme), nr);
            let kw = width_of(ts, keys);
            if opens {
                chevron(scene, right, y, dim, theme);
            } else {
                label(
                    ts,
                    scene,
                    keys,
                    Rect::new(right - kw, y, kw, rh),
                    dim,
                    theme,
                );
            }
            laid.lines.push(Line {
                id: Some(rid),
                name: name.clone(),
                keys: keys.to_owned(),
                keys_inset: popup.x + popup.w - right,
                live,
                check,
            });
            if live {
                laid.rows.push((rid, rr));
            }
            if opens && top {
                laid.parents.push(rid);
                if self.sub == Some(rid) {
                    laid.open = Some((i, rr));
                }
            }
            y += rh;
        }
        pop_popover_clip(scene);
        laid
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_sub(
        &mut self,
        id: NodeId,
        menu: Rect,
        parent: Rect,
        rows: &[MenuItem],
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let pad = Spacing::Sm.px();
        let (w, check_w) = measure(rows, ts);
        let h = height() * rows.len() as f32 + pad;
        let edge = self.painted.header.x + self.painted.header.w;
        let left = self.painted.header.x;
        // Sem lugar de nenhum lado (o transbordo numa coluna estreita), abre por baixo do menu,
        // encostado à borda: nunca por cima das linhas da mãe.
        let (x, y) = if menu.x + menu.w + w <= edge {
            (menu.x + menu.w, parent.y - pad * 0.5)
        } else if menu.x - w >= left {
            (menu.x - w, parent.y - pad * 0.5)
        } else {
            ((edge - w).max(left), menu.y + menu.h)
        };
        let popup = Rect::new(x, y, w, h);
        let laid = self.paint_lines(rows, popup, check_w, false, scene, ts, theme);
        self.painted.sub = Some(SubPainted {
            parent: id,
            popup,
            rows: laid.rows,
            checked: laid.checked,
            lines: laid.lines,
        });
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

    /// Um menu aberto: o de um título ou o de transbordo.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open.is_some() || self.more
    }

    /// O título sob o menu aberto.
    fn anchor(&self) -> Option<Rect> {
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
        self.sub = None;
    }

    fn close(&mut self) {
        self.more = false;
        self.open = None;
        self.sub = None;
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
            Some(m) if self.open != Some(m) => self.open_at(Some(m)),
            None if !self.more && self.on_more(x, y) => self.open_at(None),
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
