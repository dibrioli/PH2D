//! ⭐ **A barra da própria aba** (13 §4 itens 11 e 13b, decisão D3 do dono: «menu da própria aba»):
//! uma linha no topo do cartão do código com `File · Edit · Find · Go To · Run` à esquerda, o caminho
//! até o cursor (`player.luau › update › line 21`) à direita e a engrenagem do cartão «Script
//! options» no fim. As linhas de cada menu saem da tabela única de [`crate::commands`]; um clique
//! corre o comando pelo MESMO caminho do `Ctrl+K` e da tecla.

use crate::commands::{TabMenu, menu_rows};
use crate::docs::CodeDocs;
use crate::help::page_of;
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{folga_do_texto, paint_icon, paint_text, resolve};
use ph2d_editor_core::widget::{
    Popover, paint_popover, paint_popover_row_highlight, pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

const POPOVER_ID: NodeId = hash_node_id("code_tab_menu_popover");

/// O que a barra pintou no último quadro (o rato e os testes lêem-no).
#[derive(Clone, Debug, Default)]
pub struct BarPainted {
    pub header: Rect,
    pub titles: Vec<(TabMenu, Rect)>,
    /// As linhas do menu aberto: o comando e a caixa.
    pub rows: Vec<(NodeId, Rect)>,
    pub popup: Option<Rect>,
    /// A parte «função» do caminho: a caixa, a linha e a coluna (0-based) da definição.
    pub crumb_fn: Option<(Rect, usize, usize)>,
    pub gear: Option<Rect>,
}

/// O estado da barra: o menu aberto e a linha sob o rato.
#[derive(Clone, Debug, Default)]
pub struct Bar {
    pub open: Option<TabMenu>,
    pub hover: Option<NodeId>,
    pub painted: BarPainted,
}

fn font() -> f32 {
    TypeToken::Sm.px()
}

/// A altura da barra: a de uma linha de formulário.
#[must_use]
pub fn height() -> f32 {
    ph2d_tokens::row_pitch_px()
}

/// O cartão do código partido na barra (em cima) e no resto.
#[must_use]
pub fn split(doc_r: Rect) -> (Rect, Rect) {
    let h = height().min(doc_r.h);
    (
        Rect::new(doc_r.x, doc_r.y, doc_r.w, h),
        Rect::new(doc_r.x, doc_r.y + h, doc_r.w, (doc_r.h - h).max(0.0)),
    )
}

pub(crate) fn width_of(ts: &mut TextSystem, s: &str) -> f32 {
    ts.layout(s, font(), f32::INFINITY).width()
}

pub(crate) fn label(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    ink: ColorToken,
    theme: &Theme,
) {
    let y = r.y + folga_do_texto(r.h, font());
    paint_text(
        ts,
        scene,
        s,
        r.x,
        y,
        font(),
        r.w.max(0.0) + Spacing::Xs.px(),
        resolve(ink, *theme),
    );
}

/// O caminho até o cursor: o script, a função que o contém (nome, linha e coluna da definição) e a
/// linha do cursor (1-based).
pub struct Crumb {
    pub script: String,
    pub func: Option<(String, usize, usize)>,
    pub line: usize,
}

impl Bar {
    /// Pinta a barra em `r` (o fundo é o do cartão, pintado por quem chama).
    pub fn paint(
        &mut self,
        r: Rect,
        crumb: &Crumb,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let pad = crate::cards::well_pad();
        let sm = Spacing::Sm.px();
        self.painted = BarPainted {
            header: r,
            ..BarPainted::default()
        };
        let mut x = r.x + pad;
        for m in TabMenu::ALL {
            let w = width_of(ts, m.title()) + sm * 2.0;
            let tr = Rect::new(x, r.y, w, r.h);
            if self.open == Some(m) {
                paint_popover_row_highlight(scene, tr, *theme);
            }
            let ink = if self.open == Some(m) {
                ColorToken::AccentFg
            } else {
                ColorToken::Text1
            };
            label(
                ts,
                scene,
                m.title(),
                Rect::new(x + sm, r.y, w - sm * 2.0, r.h),
                ink,
                theme,
            );
            self.painted.titles.push((m, tr));
            x += w;
        }
        // A engrenagem no fim; o caminho encostado à esquerda dela.
        let side = r.h;
        let gear = Rect::new(r.x + r.w - pad - side, r.y, side, side);
        let inset = Spacing::Xs.px();
        let glyph = Rect::new(
            gear.x + inset,
            gear.y + inset,
            side - inset * 2.0,
            side - inset * 2.0,
        );
        paint_icon(
            scene,
            IconId::Settings,
            glyph,
            resolve(ColorToken::Text2, *theme),
            StrokeToken::Default.px(),
        );
        self.painted.gear = Some(gear);
        let sep = "\u{203a}";
        let line = ph2d_i18n::tr_with("code.crumb.line", &[("line", &crumb.line.to_string())]);
        let parts: Vec<(String, bool)> = [
            Some((crumb.script.clone(), false)),
            crumb.func.as_ref().map(|f| (f.0.clone(), true)),
            Some((line, false)),
        ]
        .into_iter()
        .flatten()
        .collect();
        let sep_pad = Spacing::Xs.px();
        let sep_w = width_of(ts, sep) + sep_pad * 2.0;
        let total: f32 = parts.iter().map(|p| width_of(ts, &p.0)).sum::<f32>()
            + sep_w * (parts.len() - 1) as f32;
        let mut cx = (gear.x - sm - total).max(x + sm);
        for (i, (s, is_fn)) in parts.iter().enumerate() {
            if i > 0 {
                label(
                    ts,
                    scene,
                    sep,
                    Rect::new(cx + sep_pad, r.y, sep_w - sep_pad * 2.0, r.h),
                    ColorToken::Text3,
                    theme,
                );
                cx += sep_w;
            }
            let w = width_of(ts, s);
            let pr = Rect::new(cx, r.y, w, r.h);
            let ink = if *is_fn {
                ColorToken::Accent
            } else {
                ColorToken::Text2
            };
            label(ts, scene, s, pr, ink, theme);
            if *is_fn && let Some(f) = &crumb.func {
                self.painted.crumb_fn = Some((pr, f.1, f.2));
            }
            cx += w;
        }
    }

    /// Pinta o menu aberto por baixo do título dele (por cima de tudo: chame-a por último).
    pub fn paint_popup(&mut self, scene: &mut VectorScene, ts: &mut TextSystem, theme: &Theme) {
        self.painted.rows.clear();
        self.painted.popup = None;
        let Some(m) = self.open else { return };
        let Some(&(_, title)) = self.painted.titles.iter().find(|t| t.0 == m) else {
            return;
        };
        let rows = menu_rows(m);
        let hint = (m == TabMenu::Run).then(|| ph2d_i18n::tr("code.menu.restart_hint"));
        let pad = Spacing::Sm.px();
        let gap = Spacing::Lg.px();
        let name_w = rows.iter().map(|r| width_of(ts, &r.1)).fold(0.0, f32::max);
        let keys_w = rows.iter().map(|r| width_of(ts, &r.2)).fold(0.0, f32::max);
        let hint_w = hint.map_or(0.0, |h| width_of(ts, h));
        let w = (name_w + gap + keys_w).max(hint_w) + pad * 4.0;
        let rh = height();
        let n = rows.len() + usize::from(hint.is_some());
        let popup = Rect::new(title.x, title.y + title.h, w, rh * n as f32 + pad);
        paint_popover(&Popover::new(POPOVER_ID), popup, scene, *theme);
        push_popover_clip(popup, scene);
        let mut y = popup.y + pad * 0.5;
        for (id, name, keys) in rows {
            let rr = Rect::new(
                popup.x + Spacing::Xs.px(),
                y,
                popup.w - Spacing::Xs.px() * 2.0,
                rh,
            );
            let hot = self.hover == Some(id);
            if hot {
                paint_popover_row_highlight(scene, rr, *theme);
            }
            let (ink, dim) = if hot {
                (ColorToken::AccentFg, ColorToken::AccentFg)
            } else {
                (ColorToken::Text1, ColorToken::Text3)
            };
            label(
                ts,
                scene,
                &name,
                Rect::new(popup.x + pad, y, name_w, rh),
                ink,
                theme,
            );
            let kw = width_of(ts, &keys);
            label(
                ts,
                scene,
                &keys,
                Rect::new(popup.x + popup.w - pad * 2.0 - kw, y, kw, rh),
                dim,
                theme,
            );
            self.painted.rows.push((id, rr));
            y += rh;
        }
        if let Some(h) = hint {
            label(
                ts,
                scene,
                h,
                Rect::new(popup.x + pad, y, hint_w, rh),
                ColorToken::Text3,
                theme,
            );
        }
        pop_popover_clip(scene);
        self.painted.popup = Some(popup);
    }

    fn title_at(&self, x: f32, y: f32) -> Option<TabMenu> {
        self.painted
            .titles
            .iter()
            .find(|t| t.1.contains(x, y))
            .map(|t| t.0)
    }

    fn row_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.painted
            .rows
            .iter()
            .find(|r| r.1.contains(x, y))
            .map(|r| r.0)
    }
}

/// O que um carregar na barra (ou no menu aberto) pede.
enum BarPress {
    Run(NodeId),
    Crumb(usize, usize),
    Gear,
    Taken,
}

impl CodeDocs {
    /// O cartão do código da aba `id` partido em barra e corpo (uma página de ajuda não tem barra).
    pub(crate) fn code_body(&self, id: u64, doc_r: Rect) -> Rect {
        if page_of(id).is_some() {
            doc_r
        } else {
            split(doc_r).1
        }
    }

    /// O caminho até o cursor da aba `id`.
    fn crumb(&mut self, id: u64) -> Option<Crumb> {
        let d = self.docs.iter_mut().find(|d| d.script.0 == id)?;
        let t = d.buffer.text();
        d.syntax.fresh(t);
        let line = t.line_of_byte(d.buffer.selection().primary().head);
        let folds = d.syntax.folds_in(t, 0..t.line_count());
        let func = d
            .syntax
            .symbols(t)
            .into_iter()
            .filter(|s| s.kind == ph2d_code_syntax::SymbolKind::Function)
            .filter_map(|s| {
                let l = t.line_of_byte(s.byte);
                let end = folds
                    .iter()
                    .find(|f| f.start_line == l)
                    .map_or(l, |f| f.end_line);
                let col = t.slice(t.byte_of_line(l)..s.byte).chars().count();
                (l <= line && line <= end).then_some((s.name, l, col))
            })
            .next_back();
        Some(Crumb {
            script: d.title.clone(),
            func,
            line: line + 1,
        })
    }

    /// Pinta a barra no topo de `doc_r` (código, nunca ajuda): o fundo de cartão desce um raio por
    /// baixo do cartão do código, que se lê como a mesma moldura.
    pub(crate) fn paint_bar(
        &mut self,
        id: u64,
        doc_r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let Some(crumb) = self.crumb(id) else { return };
        let (head, _) = split(doc_r);
        let back = Rect::new(
            head.x,
            head.y,
            head.w,
            (head.h + crate::cards::gap() * 2.0).min(doc_r.h),
        );
        crate::cards::open(scene, back, theme);
        self.bar.paint(head, &crumb, scene, ts, theme);
        crate::cards::close(scene);
    }

    /// O que flutua por cima da aba: o cartão das opções e o menu aberto (este por último).
    pub(crate) fn paint_over(
        &mut self,
        id: u64,
        doc_r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        if page_of(id).is_some() {
            return;
        }
        let state = self
            .docs
            .iter()
            .find(|d| d.script.0 == id)
            .map(|d| self.command_state(d));
        if let Some(state) = state {
            let assist = self.intel.assist();
            let under = self.code_body(id, doc_r);
            self.options.paint(under, state, assist, scene, ts, theme);
        }
        self.bar.paint_popup(scene, ts, theme);
    }

    /// Um carregar primário: o menu aberto, a barra, o cartão das opções. `Some(true)` = tomado.
    pub(crate) fn bar_press(&mut self, id: u64, x: f32, y: f32) -> Option<bool> {
        if page_of(id).is_some() {
            return None;
        }
        let press = if self.bar.open.is_some() {
            match (self.bar.row_at(x, y), self.bar.title_at(x, y)) {
                (Some(cmd), _) => BarPress::Run(cmd),
                (None, Some(m)) if self.bar.open != Some(m) => {
                    self.bar.open = Some(m);
                    BarPress::Taken
                }
                _ => {
                    self.bar.open = None;
                    BarPress::Taken
                }
            }
        } else if let Some(m) = self.bar.title_at(x, y) {
            self.bar.open = Some(m);
            self.bar.hover = None;
            BarPress::Taken
        } else if let Some((r, l, c)) = self.bar.painted.crumb_fn
            && r.contains(x, y)
        {
            BarPress::Crumb(l, c)
        } else if self.bar.painted.gear.is_some_and(|g| g.contains(x, y)) {
            BarPress::Gear
        } else if let Some(cmd) = self.options.press(x, y) {
            BarPress::Run(cmd)
        } else if self.options.contains(x, y) {
            BarPress::Taken
        } else {
            self.options.click_outside();
            return self.bar.painted.header.contains(x, y).then_some(true);
        };
        match press {
            BarPress::Run(cmd) => {
                self.bar.open = None;
                let _ = ph2d_editor_core::doc_surface::DocumentSurface::command(self, id, cmd);
            }
            BarPress::Crumb(l, c) => {
                let _ = self.tab_go_to(id, l, c);
            }
            BarPress::Gear => self.options.toggle(),
            BarPress::Taken => {}
        }
        Some(true)
    }

    /// O rato sem botão: a linha sob ele no menu aberto; passar a outro título troca de menu.
    pub(crate) fn bar_move(&mut self, x: f32, y: f32) -> bool {
        if self.bar.open.is_none() {
            return false;
        }
        if let Some(m) = self.bar.title_at(x, y) {
            self.bar.open = Some(m);
        }
        self.bar.hover = self.bar.row_at(x, y);
        true
    }

    /// O Esc fecha o menu aberto; sem menu, o cartão «Script options» sem alfinete. `true` = fechou algo.
    pub(crate) fn bar_escape(&mut self) -> bool {
        if self.bar.open.take().is_some() {
            return true;
        }
        if self.options.open && !self.options.pinned {
            self.options.open = false;
            return true;
        }
        false
    }

    /// O que a barra pintou (os testes clicam por aqui).
    #[must_use]
    pub fn bar_painted(&self) -> &BarPainted {
        &self.bar.painted
    }

    /// Abre o menu `m` (o passo da foto do smoke; o clique faz o mesmo).
    pub fn open_tab_menu(&mut self, m: Option<TabMenu>) {
        self.bar.open = m;
    }
}

impl CodeDocs {
    /// O cartão «Script options» (os testes e a sonda da foto lêem-no).
    #[must_use]
    pub fn options_card(&self) -> &crate::options_card::OptionsCard {
        &self.options
    }

    /// Abre/fecha o cartão «Script options» (o passo da foto do smoke; a engrenagem faz o mesmo).
    pub fn set_options_open(&mut self, open: bool) {
        self.options.open = open;
    }
}
