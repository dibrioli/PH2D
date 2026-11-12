//! ⭐ **O menu aberto da barra e os submenus em cascata** (dono 10/11: `View ▸ Text rendering ▸ 4 modos`).
//!
//! A mecânica é a dos menus de sistema (macOS, GTK, Windows): a linha com submenu tem a seta `›` à
//! direita; **pairar** sobre ela abre o submenu ao lado (fica acesa enquanto ele está aberto);
//! pairar noutra linha do mesmo nível fecha-o; o rato pode sair de todos sem os fechar. Os níveis são
//! N (`… ▸ View ▸ Text rendering ▸`): cada submenu pinta-se pela MESMA [`TabBar::paint_lines`]. As
//! escolhas são um-entre-N: a em vigor leva o `✓` na coluna da esquerda (em acento). A caminho de um
//! submenu aberto (dentro do triângulo entre a ÂNCORA — o último ponto fora da mira, que fica parada
//! a caminho — e a borda dele, alargada meia linha) o rato passa por cima de outras linhas sem o
//! fechar — a «mira» dos menus de sistema; parar mais de [`AIM_REST_NS`] a caminho acaba-a. Escolher
//! fecha tudo; o Esc fecha **um** nível.
//!
//! **Os menus flutuam por cima de TUDO, presos à JANELA** ([`TabBar::window`]; a aba pinta-os no fim
//! do quadro): o menu abre por baixo do título e, sem lugar, encosta ao fundo da janela; o submenu
//! abre à direita, senão à esquerda, senão por baixo do menu; um nível mais alto que a janela ocupa-a
//! e rola (a roda e as setas ▲▼ nas pontas). Sem janela (os testes da barra sozinha) a borda é a da
//! barra na horizontal e nada limita a vertical. A posição e o rato estão em [`super::cascade`].

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

/// Uma linha do menu aberto (o de um título e os submenus pintam-se pela MESMA função).
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
    /// Uma linha que abre um submenu com as linhas `rows` (que podem ter submenus delas).
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
    /// escolhas dele (de qualquer nível) com a mãe de topo (`Some(mãe)`).
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

/// Um submenu pintado: a linha-mãe, a caixa e as linhas.
#[derive(Clone, Debug, PartialEq)]
pub struct SubPainted {
    pub parent: NodeId,
    pub popup: Rect,
    /// As linhas que se clicam (à vista) e a caixa delas.
    pub rows: Vec<(NodeId, Rect)>,
    /// As escolhas com o `✓` (a em vigor).
    pub checked: Vec<NodeId>,
    /// Todas as linhas como saíram (as inactivas e as notas incluídas).
    pub lines: Vec<Line>,
    /// As linhas que abrem um submenu (o nível seguinte).
    pub parents: Vec<NodeId>,
    /// As setas ▲▼ das pontas, quando o nível é mais alto que a janela e rola.
    pub arrows: Option<(Rect, Rect)>,
}

/// O que um carregar no menu pede a quem o usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuPress {
    /// Correr o comando (o menu já fechou).
    Run(NodeId),
    /// O carregar foi do menu (abrir, trocar, fechar, rolar); nada a correr.
    Taken,
}

/// A linha do transbordo `…` que abre o submenu do `i`-ésimo título.
#[must_use]
pub fn more_row(i: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("code_tab_menu_more_{i}"))
}

/// O menu do transbordo `…` (a barra estreita, o idioma do VS Code): cada título escondido
/// `(linha, nome, linhas do menu dele)` vira um submenu com as MESMAS linhas do menu desse título
/// (atalhos, inactivas a cinzento, notas, e os submenus dele como submenus: o 3.º nível).
pub fn more_menu(hidden: impl Iterator<Item = (NodeId, String, Vec<MenuItem>)>) -> Vec<MenuItem> {
    hidden
        .map(|(id, name, rows)| MenuItem::Sub { id, name, rows })
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

fn icon_at(scene: &mut VectorScene, icon: IconId, r: Rect, ink: ColorToken, theme: &Theme) {
    let stroke = StrokeToken::Default.px();
    paint_icon(scene, icon, r, resolve(ink, *theme), stroke);
}

fn chevron(scene: &mut VectorScene, right: f32, y: f32, ink: ColorToken, theme: &Theme) {
    let s = INLINE_ICON_PX;
    let r = Rect::new(right - s, y + (height() - s) * 0.5, s, s);
    icon_at(scene, IconId::ChevronRight, r, ink, theme);
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

/// A altura de um menu de `n` linhas inteiro (sem rolar).
fn content_h(n: usize) -> f32 {
    height() * n as f32 + Spacing::Sm.px()
}

/// O que [`TabBar::paint_lines`] pintou.
#[derive(Default)]
struct Laid {
    rows: Vec<(NodeId, Rect)>,
    parents: Vec<NodeId>,
    checked: Vec<NodeId>,
    lines: Vec<Line>,
    /// A caixa da linha cujo submenu está aberto (se à vista).
    open: Option<Rect>,
    arrows: Option<(Rect, Rect)>,
}

impl<M: Copy + PartialEq> TabBar<M> {
    /// As linhas do menu aberto: as do título (`rows`), as do transbordo (cada título escondido como
    /// um submenu `more(m)` = (linha, nome) com as linhas dele), ou nenhuma.
    pub fn open_items(
        &self,
        rows: impl Fn(M) -> Vec<MenuItem>,
        more: impl Fn(M) -> (NodeId, String),
    ) -> Vec<MenuItem> {
        if self.more {
            let sub = |&m: &M| {
                let (id, name) = more(m);
                (id, name, rows(m))
            };
            return more_menu(self.painted.hidden.iter().map(sub));
        }
        self.open.map(rows).unwrap_or_default()
    }

    /// Pinta o menu aberto por baixo do título dele, com `items`, e os submenus abertos ao lado (por
    /// cima de tudo: chame-a por último, depois de [`Self::window`]). Sem menu aberto, a dica sob o
    /// rato depois de `pause_ns`.
    pub fn paint_popup(
        &mut self,
        items: &[MenuItem],
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
        (now, pause_ns): (u128, u128),
    ) {
        let p = &mut self.painted;
        (p.popup, p.sub, p.arrows) = (None, None, None);
        p.rows.clear();
        p.sub_parents.clear();
        p.lines.clear();
        p.deeper.clear();
        if !self.is_open() {
            return self.paint_tip(scene, ts, theme, now, pause_ns);
        }
        let Some(title) = self.anchor() else {
            return;
        };
        self.prune(items);
        self.take_first(items);
        let (w, check_w) = measure(items, ts);
        // Sem janela, o menu fica debaixo do título, sem limite (a barra sozinha dos testes).
        let all = Rect::new(f32::MIN, f32::MIN, f32::INFINITY, f32::INFINITY);
        let b = self.window.unwrap_or(all);
        let popup = super::cascade::below(title, w, content_h(items.len()), b);
        let laid = self.paint_lines(items, popup, check_w, 0, scene, ts, theme);
        let p = &mut self.painted;
        (p.rows, p.sub_parents, p.lines) = (laid.rows, laid.parents, laid.lines);
        (p.popup, p.arrows) = (Some(popup), laid.arrows);
        let (mut menu, mut row, mut cur) = (popup, laid.open, items);
        for (k, id) in self.chain().into_iter().enumerate() {
            let Some(rows) = cur.iter().find_map(|i| match i {
                MenuItem::Sub { id: s, rows, .. } if *s == id => Some(rows),
                _ => None,
            }) else {
                break;
            };
            let Some(parent) = row else { break };
            let (w, check_w) = measure(rows, ts);
            let b = self.bounds();
            let popup = super::cascade::beside(menu, parent, w, content_h(rows.len()), b);
            let laid = self.paint_lines(rows, popup, check_w, k + 1, scene, ts, theme);
            let sub = SubPainted {
                parent: id,
                popup,
                rows: laid.rows,
                checked: laid.checked,
                lines: laid.lines,
                parents: laid.parents,
                arrows: laid.arrows,
            };
            if k == 0 {
                self.painted.sub = Some(sub);
            } else {
                self.painted.deeper.push(sub);
            }
            (menu, row, cur) = (popup, laid.open, rows);
        }
    }

    /// Pinta `items` no popover `popup` do nível `level` (0 = o menu do título): o `✓` na coluna
    /// `check_w`, o nome, o atalho à direita (ou a `›` de um submenu); a linha inactiva a cinzento,
    /// sem realce e fora das que se clicam. Mais alto que `popup`, rola entre as setas ▲▼.
    #[allow(clippy::too_many_arguments)]
    fn paint_lines(
        &mut self,
        items: &[MenuItem],
        popup: Rect,
        check_w: f32,
        level: usize,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) -> Laid {
        let (pad, xs, rh) = (Spacing::Sm.px(), Spacing::Xs.px(), height());
        let id = if level == 0 {
            POPOVER_ID
        } else {
            SUB_POPOVER_ID
        };
        paint_popover(&Popover::new(id), popup, scene, *theme);
        let mut laid = Laid::default();
        let rolls = popup.h < content_h(items.len());
        let (view, mut y) = if rolls {
            let view = Rect::new(popup.x, popup.y + rh, popup.w, popup.h - rh * 2.0);
            let max = rh * items.len() as f32 - view.h;
            let s = self.scroll_to(level, max);
            let up = Rect::new(popup.x, popup.y, popup.w, rh);
            let down = Rect::new(popup.x, view.y + view.h, popup.w, rh);
            laid.arrows = Some((up, down));
            self.paint_arrows((up, down), s > 0.0, s < max, scene, theme);
            (view, view.y - s)
        } else {
            self.scroll_to(level, 0.0);
            (popup, popup.y + pad * 0.5)
        };
        push_popover_clip(view, scene);
        let chain = self.chain();
        let right = popup.x + popup.w - pad * 2.0;
        for it in items {
            let rr = Rect::new(popup.x + xs, y, popup.w - xs * 2.0, rh);
            let seen = (view.y..=view.y + view.h).contains(&(y + rh * 0.5));
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
                MenuItem::Sub { id, name, .. } => (*id, name, "", true, None, true),
            };
            let open = opens && chain.get(level) == Some(&rid);
            let hot = live && (self.hover == Some(rid) || open);
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
                icon_at(scene, IconId::Check, cr, mark, theme);
                laid.checked.push(rid);
            }
            let c = resolve(ink, *theme);
            mono_label::paint_on(ts, scene, *theme, name, c, row_bg(hot, theme), nr);
            let kw = width_of(ts, keys);
            if opens {
                chevron(scene, right, y, dim, theme);
            } else {
                let kr = Rect::new(right - kw, y, kw, rh);
                label(ts, scene, keys, kr, dim, theme);
            }
            laid.lines.push(Line {
                id: Some(rid),
                name: name.clone(),
                keys: keys.to_owned(),
                keys_inset: popup.x + popup.w - right,
                live,
                check,
            });
            if live && seen {
                laid.rows.push((rid, rr));
            }
            if opens && seen {
                laid.parents.push(rid);
                if open {
                    laid.open = Some(rr);
                }
            }
            y += rh;
        }
        pop_popover_clip(scene);
        laid
    }

    /// As setas ▲▼ das pontas de um nível que rola (a cinzento quando já não há para onde ir).
    fn paint_arrows(
        &self,
        (up, down): (Rect, Rect),
        more_up: bool,
        more_down: bool,
        scene: &mut VectorScene,
        theme: &Theme,
    ) {
        let s = INLINE_ICON_PX;
        let at = |r: Rect| Rect::new(r.x + (r.w - s) * 0.5, r.y + (r.h - s) * 0.5, s, s);
        let ink = |on: bool| {
            if on {
                ColorToken::Text2
            } else {
                ColorToken::Text3
            }
        };
        icon_at(scene, IconId::ChevronUp, at(up), ink(more_up), theme);
        icon_at(scene, IconId::ChevronDown, at(down), ink(more_down), theme);
    }
}
