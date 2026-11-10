//! ⭐ **O cartão «Script options»** (13 §4 item 12; plano 14 Q4 passo D): a faixa de cima (título, o
//! nome do script, o alfinete), uma linha por opção — título, interruptor à direita, e a frase do que
//! ela faz por baixo — e no pé a frase do assistente com «Restart». A linha inteira é o alvo (mão e
//! moldura); sem alfinete fecha-se com um clique fora. Os comandos e o estado são da família.

use crate::frame::Assist;
use ph2d_a11y::NodeId;
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    folga_do_texto, frame_radius, paint_icon, paint_text_elided, paint_text_title_elided, resolve,
    stroke_rounded_rect,
};
use ph2d_editor_core::widget::{
    Button, CHECKBOX_BOX_PX, Popover, Toggle, ToggleState, paint_button, paint_popover,
    paint_toggle, pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
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
    /// O interruptor de cada opção (a borda direita dele é a do «Restart»), e se mostra «ligado».
    pub marks: Vec<Rect>,
    pub on: Vec<bool>,
    /// A frase de cada opção: a caixa onde foi pintada.
    pub says: Vec<Rect>,
    pub pin: Option<Rect>,
}

#[derive(Clone, Debug, Default)]
pub struct OptionsCard {
    pub open: bool,
    pub pinned: bool,
    pub painted: OptionsPainted,
    /// A linha debaixo do rato e a carregada (a moldura de hover e a de clique).
    pub hover: Option<NodeId>,
    pub pressed: Option<NodeId>,
}

fn font() -> f32 {
    TypeToken::Sm.px()
}

fn small() -> f32 {
    TypeToken::Xs.px()
}

/// A altura da frase de uma opção, debaixo do título.
fn says_h() -> f32 {
    small() + Spacing::Sm.px()
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

    fn row_at(&self, x: f32, y: f32) -> Option<NodeId> {
        if !self.open {
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
        // Uma grelha (dono 04/11): os títulos à esquerda (`inner`); interruptores, alfinete e
        // «Restart» com a borda direita em `inner + inner_w`.
        let md = Spacing::Md.px();
        let pad = ph2d_tokens::card_pad_px() + md;
        let rh = ph2d_tokens::row_pitch_px();
        let sentence = assist_sentence(f.assist);
        let restart = ph2d_i18n::tr("code.options.restart");
        let width = |ts: &mut TextSystem, s: &str, px: f32| ts.layout(s, px, f32::INFINITY).width();
        let restart_w = width(ts, restart, font()) + Spacing::Lg.px() * 2.0;
        let (sw, sh) = (CHECKBOX_BOX_PX * 2.0, CHECKBOX_BOX_PX);
        let title = ph2d_i18n::tr("code.options.title");
        let mut need = width(ts, title, font()) + md + width(ts, f.script, font()) + md + rh;
        for o in f.rows {
            need = need.max(width(ts, o.title, font()) + md + sw);
            need = need.max(width(ts, o.says, small()));
        }
        need = need.max(width(ts, sentence, font()) + Spacing::Xl.px() + restart_w);
        let w = (need + pad * 2.0).min(under.w);
        let n = f.rows.len() as f32;
        let h = rh * (n + 2.0) + says_h() * n + Spacing::Xs.px() + pad * 2.0;
        let g = crate::cards::gap();
        let card = Rect::new(under.x + under.w - w - g, under.y + g, w, h.min(under.h));
        paint_popover(&Popover::new(f.id), card, scene, *theme);
        push_popover_clip(card, scene);
        let inner = card.x + pad;
        let inner_w = (card.w - pad * 2.0).max(0.0);
        let mut y = card.y + pad;
        self.paint_head(f.script, (inner, inner_w, y), scene, ts, theme);
        y += rh;
        for o in f.rows {
            let r = Rect::new(inner - md, y, inner_w + md * 2.0, rh + says_h());
            let state = if self.pressed == Some(o.cmd) {
                ToggleState::Pressed
            } else if self.hover == Some(o.cmd) {
                ToggleState::Hovered
            } else {
                ToggleState::Normal
            };
            let ink = |t| resolve(t, *theme);
            let ty = y + folga_do_texto(rh, font());
            let tw = inner_w - sw - md;
            paint_text_elided(
                ts,
                scene,
                o.title,
                inner,
                ty,
                font(),
                tw,
                ink(ColorToken::Text1),
            );
            let sy = y + rh + folga_do_texto(says_h(), small()) - Spacing::Xs.px();
            let says = Rect::new(inner, y + rh - Spacing::Xs.px(), inner_w, says_h());
            paint_text_elided(
                ts,
                scene,
                o.says,
                inner,
                sy,
                small(),
                inner_w,
                ink(ColorToken::Text3),
            );
            let mark = Rect::new(inner + inner_w - sw, y + (rh - sh) * 0.5, sw, sh);
            let t = Toggle::new(o.cmd, o.title).on(o.on).state(state);
            paint_toggle(&t, mark, scene, *theme);
            self.paint_frame(o.cmd, r, scene, theme);
            self.painted.marks.push(mark);
            self.painted.on.push(t.on);
            self.painted.says.push(says);
            self.painted.rows.push((o.cmd, r));
            y += rh + says_h();
        }
        // A frase do assistente e o «Restart».
        y += Spacing::Xs.px();
        let ink = resolve(ColorToken::Text2, *theme);
        let sx = inner_w - restart_w - md;
        paint_text_elided(
            ts,
            scene,
            sentence,
            inner,
            y + folga_do_texto(rh, font()),
            font(),
            sx,
            ink,
        );
        let bh = rh - ph2d_tokens::control_gap_px();
        let br = Rect::new(inner + inner_w - restart_w, y, restart_w, bh);
        paint_button(&Button::new(f.restart, restart), br, scene, ts, *theme);
        self.painted.rows.push((f.restart, br));
        pop_popover_clip(scene);
        self.painted.card = Some(card);
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
        paint_text_elided(
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

    /// A moldura da linha carregada (`Accent`, grossa) ou debaixo do rato (`BorderStrong`, fina).
    fn paint_frame(&self, cmd: NodeId, r: Rect, scene: &mut VectorScene, theme: &Theme) {
        let (token, stroke) = if self.pressed == Some(cmd) {
            (ColorToken::Accent, StrokeToken::Thick)
        } else if self.hover == Some(cmd) {
            (ColorToken::BorderStrong, StrokeToken::Thin)
        } else {
            return;
        };
        let d = Spacing::Xxs.px();
        let f = Rect::new(
            r.x + d,
            r.y + d,
            (r.w - d * 2.0).max(0.0),
            (r.h - d * 2.0).max(0.0),
        );
        let radius = frame_radius(*theme, Radius::Sm.px());
        stroke_rounded_rect(scene, f, radius, stroke.px(), resolve(token, *theme));
    }
}
