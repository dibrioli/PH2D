//! ⭐ **A barra da própria aba** (13 §4 itens 11 e 13b, decisão D3 do dono: «menu da própria aba»):
//! uma linha no topo do cartão do código com `File · Edit · Find · Go To · Run` à esquerda, o caminho
//! até o cursor (`player.luau › update › line 21`) à direita e a engrenagem do cartão «Script
//! options» no fim. As linhas de cada menu saem da tabela única de [`crate::commands`]; um clique
//! corre o comando pelo MESMO caminho do `Ctrl+K` e da tecla.

use crate::commands::{CMD_RESTART_ASSIST, TabMenu, menu_rows, row_state};
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
    /// As dicas do rato: a engrenagem, a função do caminho e o alfinete do cartão das opções.
    pub tips: Vec<(Rect, String)>,
}

/// O estado da barra: o menu aberto e a linha sob o rato.
#[derive(Clone, Debug, Default)]
pub struct Bar {
    pub open: Option<TabMenu>,
    pub hover: Option<NodeId>,
    pub painted: BarPainted,
    /// O jogo parado: o menu Run apaga o que não vale e o F8 aparece numa linha só.
    pub stop: crate::debug::GameStop,
    /// A dica sob o rato (índice em `painted.tips`) e desde quando.
    pub tip: Option<(usize, u128)>,
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
    pub func: CrumbFunc,
    pub line: usize,
}

/// A função que contém o cursor: nome, linha e coluna da definição.
pub type CrumbFunc = Option<(String, usize, usize)>;

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
        let gear_tip = ph2d_i18n::tr("code.menu.options_tip").to_owned();
        self.painted.tips.push((gear, gear_tip));
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
                let tip = ph2d_i18n::tr_with("code.crumb.go_tip", &[("name", &f.0)]);
                self.painted.tips.push((pr, tip));
            }
            cx += w;
        }
    }

    /// Pinta o menu aberto por baixo do título dele (por cima de tudo: chame-a por último).
    pub fn paint_popup(
        &mut self,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
        now: u128,
    ) {
        self.painted.rows.clear();
        self.painted.popup = None;
        let Some(m) = self.open else {
            return self.paint_tip(scene, ts, theme, now);
        };
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
            let (live, show_keys) = row_state(id, self.stop);
            let keys = if show_keys { keys } else { String::new() };
            let rr = Rect::new(
                popup.x + Spacing::Xs.px(),
                y,
                popup.w - Spacing::Xs.px() * 2.0,
                rh,
            );
            let hot = live && self.hover == Some(id);
            if hot {
                paint_popover_row_highlight(scene, rr, *theme);
            }
            let (ink, dim) = if hot {
                (ColorToken::AccentFg, ColorToken::AccentFg)
            } else if !live {
                (ColorToken::Text3, ColorToken::Text3)
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
            if live {
                self.painted.rows.push((id, rr));
            }
            y += rh;
            // A nota do Restart Code Assistant fica debaixo dele.
            if let Some(h) = hint.filter(|_| id == CMD_RESTART_ASSIST) {
                let hr = Rect::new(popup.x + pad, y, hint_w, rh);
                label(ts, scene, h, hr, ColorToken::Text3, theme);
                y += rh;
            }
        }
        pop_popover_clip(scene);
        self.painted.popup = Some(popup);
    }

    /// A dica sob o rato, depois da pausa do balão (a do `hint`), por baixo da caixa dela.
    fn paint_tip(&self, scene: &mut VectorScene, ts: &mut TextSystem, theme: &Theme, now: u128) {
        let Some((i, since)) = self.tip else { return };
        let Some((r, msg)) = self.painted.tips.get(i) else {
            return;
        };
        if now.saturating_sub(since) < crate::hint::HOVER_PAUSE_NS {
            return;
        }
        let w = width_of(ts, msg) + Spacing::Sm.px() * 2.0;
        let x = r.x.min(r.x + r.w - w);
        let rect = Rect::new(x, r.y + r.h, w, height());
        let tip = ph2d_editor_core::widget::Tooltip::new(POPOVER_ID, msg.clone());
        ph2d_editor_core::widget::paint_tooltip(&tip, rect, scene, ts, *theme);
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

    /// O caminho até o cursor da aba `id`. É chrome de cada quadro: nunca reparsa (com a árvore
    /// velha fica a função da última árvore exacta até o worker a trocar) e só procura a função
    /// quando a árvore ou a linha mudam — com as dobras das funções candidatas, nunca as do ficheiro.
    fn crumb(&mut self, id: u64) -> Option<Crumb> {
        let d = self.docs.iter_mut().find(|d| d.script.0 == id)?;
        let t = d.buffer.text();
        let line = t.line_of_byte(d.buffer.selection().primary().head);
        let key = (d.syntax.generation(), line);
        let cached = d.crumb_func.as_ref();
        if !d.syntax.is_stale() && cached.is_none_or(|c| (c.0, c.1) != key) {
            let func = d
                .syntax
                .symbols(t)
                .into_iter()
                .filter(|s| s.kind == ph2d_code_syntax::SymbolKind::Function)
                .map(|s| (t.line_of_byte(s.byte), s))
                .filter(|(l, _)| *l <= line)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .find(|(l, _)| {
                    d.syntax.fold_at(t, *l).is_some_and(|f| line <= f.end_line) || *l == line
                })
                .map(|(l, s)| {
                    let col = t.slice(t.byte_of_line(l)..s.byte).chars().count();
                    (s.name, l, col)
                });
            d.crumb_func = Some((key.0, key.1, func));
        }
        Some(Crumb {
            script: d.title.clone(),
            func: d.crumb_func.as_ref().and_then(|c| c.2.clone()),
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
        if self.options.open
            && let Some(pin) = self.options.painted.pin
        {
            let tip = ph2d_i18n::tr("code.options.pin").to_owned();
            self.bar.painted.tips.push((pin, tip));
        }
        self.bar.stop = self.debug.game_stop;
        self.bar.paint_popup(scene, ts, theme, self.clock.0);
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
        let at = self
            .bar
            .painted
            .tips
            .iter()
            .position(|t| t.0.contains(x, y));
        self.bar.tip = at.map(|i| match self.bar.tip {
            Some((j, since)) if j == i => (i, since),
            _ => (i, self.clock.0),
        });
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
