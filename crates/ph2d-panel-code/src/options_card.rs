//! ⭐ **O cartão «Script options»** (13 §4 item 12; plano 14 Q4 passo D; dono 09/11): uma lista de
//! definições à macOS System Settings / VS Code Settings — a faixa de cima (título, o nome do script,
//! o alfinete), uma linha por opção (título, a frase por baixo que quebra mas nunca se corta, a
//! caixa de marcar do cartão à direita — dono 10/11: o padrão do app é caixa, não interruptor), um
//! fio entre as opções, e no pé a frase do assistente com «Restart». A linha inteira é o alvo (mão
//! e UM fundo de realce); sem alfinete fecha-se com um clique fora. Os comandos e o estado são da
//! família.

use crate::frame::Assist;
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, paint_icon, paint_text_block, paint_text_title_elided,
    resolve, text_rendering,
};
use ph2d_editor_core::widget::card_ink::{CardInk, card_body, tinta};
use ph2d_editor_core::widget::{
    Button, Popover, card_check_rect, paint_button, paint_card_check, paint_popover,
    pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Uma opção: o comando, o título, a frase do que faz, e se está ligada.
#[derive(Clone, Copy, Debug)]
pub struct OptionRow<'a> {
    pub cmd: NodeId,
    pub title: &'a str,
    pub says: &'a str,
    pub on: bool,
}

/// O que a família entrega ao cartão em cada quadro.
#[derive(Clone, Copy, Debug)]
pub struct OptionsFrame<'a> {
    pub id: NodeId,
    pub script: &'a str,
    pub rows: &'a [OptionRow<'a>],
    pub assist: Option<Assist>,
    /// O comando do «Restart» (o mesmo do Run ▸ Restart Code Assistant).
    pub restart: NodeId,
}

/// O que o cartão pintou no último quadro.
#[derive(Clone, Debug, Default)]
pub struct OptionsPainted {
    pub card: Option<Rect>,
    /// A linha inteira de cada opção (o alvo do clique) e o «Restart»: o comando e a caixa.
    pub rows: Vec<(NodeId, Rect)>,
    /// A caixa de marcar de cada opção (a borda direita dela é a do «Restart»), e se está marcada.
    pub marks: Vec<Rect>,
    pub on: Vec<bool>,
    /// A frase de cada opção: a caixa onde foi pintada (a altura é a do texto quebrado).
    pub says: Vec<Rect>,
    /// O único realce pintado: o fundo da linha debaixo do rato ou carregada, e a tinta dele.
    pub highlight: Option<(Rect, ph2d_vector::Color)>,
    pub pin: Option<Rect>,
}

#[derive(Clone, Debug, Default)]
pub struct OptionsCard {
    pub open: bool,
    pub pinned: bool,
    pub painted: OptionsPainted,
    /// A linha debaixo do rato e a carregada (o fundo de hover e o de clique).
    pub hover: Option<NodeId>,
    pub pressed: Option<NodeId>,
}

fn font() -> f32 {
    TypeToken::Sm.px()
}

fn small() -> f32 {
    TypeToken::Xs.px()
}

/// A largura de `s` numa linha só, e a altura dele quebrado em `w` — a mesma passada de layout que o
/// [`paint_text_block`] faz ao pintar.
fn measure(ts: &mut TextSystem, s: &str, px: f32, w: f32) -> (f32, f32) {
    let l = ts.layout_for_rendering(s, px, w, FontWeight::MEDIUM, text_rendering());
    (l.width(), l.height())
}

/// A frase do assistente de código no pé do cartão.
#[must_use]
pub fn assist_sentence(assist: Option<Assist>) -> &'static str {
    ph2d_i18n::tr(match assist {
        None => "code.options.assist_on",
        Some(Assist::Starting) => "code.status.assist_starting",
        Some(Assist::Restarting) => "code.status.assist_restarting",
        Some(Assist::Missing) => "code.status.assist_missing",
        Some(Assist::Stopped) => "code.status.assist_stopped",
    })
}

impl OptionsCard {
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Um clique fora do cartão: sem alfinete, ele fecha.
    pub fn click_outside(&mut self) {
        if !self.pinned {
            self.open = false;
        }
    }

    #[must_use]
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.open && self.painted.card.is_some_and(|c| c.contains(x, y))
    }

    /// Só dentro do cartão: num corpo baixo ele corta-se e as linhas de baixo não se vêem.
    fn row_at(&self, x: f32, y: f32) -> Option<NodeId> {
        if !self.contains(x, y) {
            return None;
        }
        let r = self.painted.rows.iter().find(|r| r.1.contains(x, y));
        r.map(|r| r.0)
    }

    /// Um carregar: o comando de uma linha ou do «Restart»; o alfinete troca-se aqui.
    pub fn press(&mut self, x: f32, y: f32) -> Option<NodeId> {
        if !self.open {
            return None;
        }
        if self.painted.pin.is_some_and(|p| p.contains(x, y)) {
            self.pinned = !self.pinned;
            return None;
        }
        self.pressed = self.row_at(x, y);
        self.pressed
    }

    /// Largar o botão. `true` = havia uma linha carregada.
    pub fn release(&mut self) -> bool {
        self.pressed.take().is_some()
    }

    /// O rato sem botão. `true` = a linha debaixo dele mudou (repintar).
    pub fn hover(&mut self, x: f32, y: f32) -> bool {
        let at = self.row_at(x, y);
        std::mem::replace(&mut self.hover, at) != at
    }

    /// A mão sobre o que se clica (as linhas, o «Restart», o alfinete); a seta no resto do cartão.
    #[must_use]
    pub fn look(&self, x: f32, y: f32) -> Option<PointerLook> {
        if !self.contains(x, y) {
            return None;
        }
        let pin = self.painted.pin.is_some_and(|p| p.contains(x, y));
        Some(if pin || self.row_at(x, y).is_some() {
            PointerLook::Hand
        } else {
            PointerLook::Arrow
        })
    }

    /// Pinta o cartão no canto de cima à direita de `under` (o corpo do código).
    pub fn paint(
        &mut self,
        under: Rect,
        f: &OptionsFrame<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.painted = OptionsPainted::default();
        if !self.open {
            return;
        }
        // Uma grelha: os textos começam em `inner`; caixas, alfinete e «Restart» acabam em
        // `inner + inner_w`. A largura é a que põe cada frase numa linha; sem ela, quebra.
        let (md, lg) = (Spacing::Md.px(), Spacing::Lg.px());
        let (side, rh) = (lg, ph2d_tokens::row_pitch_px());
        let mark0 = card_check_rect(0.0, 0.0, 0.0);
        let (sw, sh) = (mark0.w, mark0.h);
        let check_ink = card_body().unwrap_or(CardInk::of_list(*theme));
        let sentence = assist_sentence(f.assist);
        let restart = ph2d_i18n::tr("code.options.restart");
        let one = |ts: &mut TextSystem, s: &str, px: f32| measure(ts, s, px, f32::INFINITY).0;
        let restart_w = one(ts, restart, font()) + lg * 2.0;
        let title = ph2d_i18n::tr("code.options.title");
        let mut need = one(ts, title, font()) + lg + one(ts, f.script, font()) + md + rh;
        for o in f.rows {
            let text = one(ts, o.title, font()).max(one(ts, o.says, small()));
            need = need.max(text + lg + sw);
        }
        need = need.max(one(ts, sentence, font()) + lg + restart_w);
        let g = crate::cards::margin();
        let w = (need + side * 2.0).min((under.w - g * 2.0).max(0.0));
        let inner_w = (w - side * 2.0).max(0.0);
        let tw = (inner_w - sw - lg).max(0.0);
        let hair = StrokeToken::Thin.px();
        // As alturas, medidas na largura em que se pintam (a frase quebra, nunca se corta).
        let blocks: Vec<(f32, f32)> = f
            .rows
            .iter()
            .map(|o| {
                (
                    measure(ts, o.title, font(), tw).1,
                    measure(ts, o.says, small(), tw).1,
                )
            })
            .collect();
        let row_h = |b: (f32, f32)| (b.0 + Spacing::Xxs.px() + b.1 + md * 2.0).max(sh + md * 2.0);
        let foot_tw = (inner_w - restart_w - lg).max(0.0);
        let foot_h = measure(ts, sentence, font(), foot_tw).1.max(rh) + md * 2.0;
        let head_h = rh + md + Spacing::Xs.px();
        let rows_h: f32 = blocks.iter().map(|&b| row_h(b) + hair).sum();
        let h = head_h + rows_h + foot_h;
        let card = Rect::new(under.x + under.w - w - g, under.y + g, w, h.min(under.h));
        paint_popover(&Popover::new(f.id), card, scene, *theme);
        push_popover_clip(card, scene);
        let inner = card.x + side;
        self.paint_head(f.script, (inner, inner_w, card.y + md), scene, ts, theme);
        let mut y = card.y + head_h;
        for (o, &b) in f.rows.iter().zip(&blocks) {
            let r = Rect::new(card.x, y, card.w, row_h(b));
            self.paint_highlight(o.cmd, r, scene, theme);
            let ty = r.y + (r.h - b.0 - Spacing::Xxs.px() - b.1) * 0.5;
            let ink = |t| resolve(t, *theme);
            paint_text_block(
                ts,
                scene,
                o.title,
                inner,
                ty,
                font(),
                tw,
                ink(ColorToken::Text1),
            );
            let says = Rect::new(inner, ty + b.0 + Spacing::Xxs.px(), tw, b.1);
            paint_text_block(
                ts,
                scene,
                o.says,
                inner,
                says.y,
                small(),
                tw,
                ink(ColorToken::Text3),
            );
            let mark = card_check_rect(inner + inner_w - sw, r.y, r.h);
            let hover = self.hover == Some(o.cmd);
            paint_card_check(scene, check_ink, mark, o.on, hover);
            self.painted.marks.push(mark);
            self.painted.on.push(o.on);
            self.painted.says.push(says);
            self.painted.rows.push((o.cmd, r));
            y += r.h;
            let fio = Rect::new(inner, y, inner_w, hair);
            fill_rounded_rect(scene, fio, 0.0, resolve(ColorToken::Border, *theme));
            y += hair;
        }
        // O pé: a frase do assistente (quebra) e o «Restart», centrados um no outro.
        let bh = rh - ph2d_tokens::control_gap_px();
        let foot = Rect::new(inner, y + md, inner_w, foot_h - md * 2.0);
        let sentence_h = measure(ts, sentence, font(), foot_tw).1;
        let sy = foot.y + (foot.h - sentence_h) * 0.5;
        let ink = resolve(ColorToken::Text2, *theme);
        paint_text_block(ts, scene, sentence, inner, sy, font(), foot_tw, ink);
        let by = foot.y + (foot.h - bh) * 0.5;
        let br = Rect::new(inner + inner_w - restart_w, by, restart_w, bh);
        paint_button(&Button::new(f.restart, restart), br, scene, ts, *theme);
        self.painted.rows.push((f.restart, br));
        pop_popover_clip(scene);
        self.painted.card = Some(card);
    }

    /// O fundo da linha debaixo do rato ou carregada (mais forte): o ÚNICO realce da linha — sem
    /// moldura; a caixa só acende o fundo dela no hover. O hover é a camada do Material 3: a tinta do título, translúcida
    /// ([`ph2d_tokens::HOVER_LAYER_ALPHA`]) — clareia o cartão num tema escuro.
    fn paint_highlight(&mut self, cmd: NodeId, r: Rect, scene: &mut VectorScene, theme: &Theme) {
        let ink = if self.pressed == Some(cmd) {
            tinta(ColorToken::AccentSoft, *theme)
        } else if self.hover == Some(cmd) {
            resolve(ColorToken::Text1, *theme).with_alpha(ph2d_tokens::HOVER_LAYER_ALPHA)
        } else {
            return;
        };
        fill_rounded_rect(scene, r, 0.0, ink);
        self.painted.highlight = Some((r, ink));
    }

    /// A faixa de cima: «Script options» à esquerda, o nome do script à direita e o alfinete.
    fn paint_head(
        &mut self,
        script: &str,
        (inner, inner_w, y): (f32, f32, f32),
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        let rh = ph2d_tokens::row_pitch_px();
        let ty = y + folga_do_texto(rh, font());
        let title = ph2d_i18n::tr("code.options.title");
        let ink = |t| resolve(t, *theme);
        let tw = ts.layout(title, font(), f32::INFINITY).width();
        paint_text_title_elided(
            ts,
            scene,
            title,
            inner,
            ty,
            font(),
            inner_w - rh,
            ink(ColorToken::Text1),
        );
        let md = Spacing::Md.px();
        let nw = ts.layout(script, font(), f32::INFINITY).width();
        let room = (inner_w - rh - md - tw - md).max(0.0);
        let nx = inner + inner_w - rh - md - nw.min(room);
        ph2d_editor_core::paint::paint_text_elided(
            ts,
            scene,
            script,
            nx,
            ty,
            font(),
            room,
            ink(ColorToken::Text3),
        );
        let pin = Rect::new(inner + inner_w - rh, y, rh, rh);
        let inset = Spacing::Xs.px();
        let pin_ink = if self.pinned {
            ColorToken::Accent
        } else {
            ColorToken::Text3
        };
        let icon = Rect::new(
            pin.x + inset,
            pin.y + inset,
            rh - inset * 2.0,
            rh - inset * 2.0,
        );
        paint_icon(
            scene,
            IconId::CardPin,
            icon,
            ink(pin_ink),
            StrokeToken::Default.px(),
        );
        self.painted.pin = Some(pin);
    }
}
