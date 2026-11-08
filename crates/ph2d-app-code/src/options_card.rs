//! ⭐ **O cartão «Script options»** (13 §4 item 12): as três caixas (Strict types · Format on save ·
//! Type hints), a frase do assistente de código com «Restart», e o alfinete. Flutua no canto de cima à
//! direita do código; sem alfinete fecha-se com um clique fora dele (nunca tapa o código para
//! sempre). Cada caixa corre o MESMO comando que a paleta `Ctrl+K` lista pelo nome dela.

use crate::commands::{
    CMD_FORMAT_ON_SAVE, CMD_RESTART_ASSIST, CMD_STRICT, CMD_TYPE_HINTS, CommandState,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{folga_do_texto, paint_icon, paint_text, resolve};
use ph2d_editor_core::widget::{
    Button, Checkbox, CheckboxValue, Popover, paint_button, paint_checkbox, paint_popover,
    pop_popover_clip, push_popover_clip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::Assist;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

const CARD_ID: NodeId = hash_node_id("code_options_card");

/// O que o cartão pintou no último quadro.
#[derive(Clone, Debug, Default)]
pub struct OptionsPainted {
    pub card: Option<Rect>,
    /// As três caixas e o «Restart»: o comando e a caixa.
    pub rows: Vec<(NodeId, Rect)>,
    pub pin: Option<Rect>,
}

#[derive(Clone, Debug, Default)]
pub struct OptionsCard {
    pub open: bool,
    pub pinned: bool,
    pub painted: OptionsPainted,
}

fn font() -> f32 {
    TypeToken::Sm.px()
}

fn assist_sentence(assist: Option<Assist>) -> &'static str {
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

    /// Um carregar: o comando de uma caixa ou do «Restart»; o alfinete troca-se aqui.
    pub fn press(&mut self, x: f32, y: f32) -> Option<NodeId> {
        if !self.open {
            return None;
        }
        if self.painted.pin.is_some_and(|p| p.contains(x, y)) {
            self.pinned = !self.pinned;
            return None;
        }
        self.painted
            .rows
            .iter()
            .find(|r| r.1.contains(x, y))
            .map(|r| r.0)
    }

    /// Pinta o cartão no canto de cima à direita de `under` (o corpo do código).
    pub fn paint(
        &mut self,
        under: Rect,
        state: CommandState,
        assist: Option<Assist>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.painted = OptionsPainted::default();
        if !self.open {
            return;
        }
        let pad = ph2d_tokens::card_pad_px();
        let rh = ph2d_tokens::row_pitch_px();
        let sentence = assist_sentence(assist);
        let restart = ph2d_i18n::tr("code.options.restart");
        let width = |ts: &mut TextSystem, s: &str| ts.layout(s, font(), f32::INFINITY).width();
        let restart_w = width(ts, restart) + Spacing::Lg.px() * 2.0;
        let boxes = [
            (CMD_STRICT, "code.options.strict", state.strict),
            (
                CMD_FORMAT_ON_SAVE,
                "code.options.format_on_save",
                state.format_on_save,
            ),
            (CMD_TYPE_HINTS, "code.options.type_hints", state.type_hints),
        ];
        let mut label_w = width(ts, ph2d_i18n::tr("code.options.title"));
        for b in &boxes {
            label_w = label_w.max(width(ts, ph2d_i18n::tr(b.1)));
        }
        // A linha de formulário dá metade da caixa ao nome: o dobro do mais largo, mais a marca.
        label_w = label_w * 2.0 + rh * 2.0;
        let w = (width(ts, sentence) + Spacing::Xl.px() + restart_w)
            .max(label_w)
            .min(under.w)
            + pad * 2.0;
        let h = rh * 5.0 + pad * 2.0;
        let g = crate::cards::gap();
        let card = Rect::new(under.x + under.w - w - g, under.y + g, w, h.min(under.h));
        paint_popover(&Popover::new(CARD_ID), card, scene, *theme);
        push_popover_clip(card, scene);
        let inner = card.x + pad;
        let inner_w = card.w - pad * 2.0;
        let mut y = card.y + pad;
        let text = |ts: &mut TextSystem,
                    scene: &mut VectorScene,
                    s: &str,
                    x: f32,
                    y: f32,
                    w: f32,
                    ink: ColorToken| {
            paint_text(
                ts,
                scene,
                s,
                x,
                y + folga_do_texto(rh, font()),
                font(),
                w,
                resolve(ink, *theme),
            );
        };
        // O título e o alfinete.
        text(
            ts,
            scene,
            ph2d_i18n::tr("code.options.title"),
            inner,
            y,
            inner_w - rh,
            ColorToken::Text1,
        );
        let pin = Rect::new(inner + inner_w - rh, y, rh, rh);
        let inset = Spacing::Xs.px();
        let ink = if self.pinned {
            ColorToken::Accent
        } else {
            ColorToken::Text3
        };
        paint_icon(
            scene,
            IconId::CardPin,
            Rect::new(
                pin.x + inset,
                pin.y + inset,
                rh - inset * 2.0,
                rh - inset * 2.0,
            ),
            resolve(ink, *theme),
            StrokeToken::Default.px(),
        );
        self.painted.pin = Some(pin);
        y += rh;
        for (cmd, key, on) in boxes {
            let r = Rect::new(inner, y, inner_w, rh);
            let mut cb = Checkbox::new(cmd, ph2d_i18n::tr(key));
            cb.value = if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
            paint_checkbox(&cb, r, scene, ts, *theme);
            self.painted.rows.push((cmd, r));
            y += rh;
        }
        // A frase do assistente e o «Restart» (o mesmo comando do Run ▸ Restart Code Assistant).
        y += Spacing::Xs.px();
        text(
            ts,
            scene,
            sentence,
            inner,
            y,
            inner_w - restart_w - Spacing::Md.px(),
            ColorToken::Text2,
        );
        let br = Rect::new(
            inner + inner_w - restart_w,
            y,
            restart_w,
            ph2d_tokens::row_pitch_px() - ph2d_tokens::control_gap_px(),
        );
        paint_button(
            &Button::new(CMD_RESTART_ASSIST, restart),
            br,
            scene,
            ts,
            *theme,
        );
        self.painted.rows.push((CMD_RESTART_ASSIST, br));
        pop_popover_clip(scene);
        self.painted.card = Some(card);
    }
}

impl crate::docs::CodeDocs {
    /// «Format on save» ligado (o estado que a caixa do cartão mostra).
    #[must_use]
    pub fn format_on(&self) -> bool {
        self.format.on
    }
}
