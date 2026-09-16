//! [`NumberInput`] — numeric value with up/down step buttons.
//!
//! Reuses the [`TextInput`](super::text_input) border palette to stay
//! visually consistent. Steppers are 16x16 chips painted on the right
//! edge; clicking them adjusts the value by `step`. Hit testing for
//! the steppers is shell-side via [`NumberInput::up_rect`] /
//! [`NumberInput::down_rect`].

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use crate::widget::text_input::TextInputState;
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct NumberInput {
    pub id: NodeId,
    pub label: String,
    pub value: f64,
    pub step: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub state: TextInputState,
    /// Quanto do hover está presente; ver [`super::TextInput::hover_t`].
    pub hover_t: f32,
    /// ⭐⭐⭐ **A UNIDADE, colada ao número, DENTRO da caixa.**
    ///
    /// ⛔⛔ **Ordem do dono, 2026-09-14:** a 1.ª entrega punha a unidade num CHIP com fundo
    /// próprio, encostado à borda direita do campo (`NumericInputWithUnit`) — *«não ficou legal.
    /// Melhor junto ao número dentro da caixa»*. E ele tem razão pela mesma lei que a caixa acabara
    /// de ganhar: **o campo passou a ser uma superfície afundada**, e um segundo rectângulo com
    /// outro fundo lá dentro lê-se como *duas* caixas.
    ///
    /// ⚠️ **Ela é pintada SÓ EM REPOUSO.** A escrever, o que está no campo é o que o artista
    /// escreveu — e o parser aceita o sufixo digitado (`"5m/s"`), então mostrá-lo durante a edição
    /// faria o texto discordar do que vai ser lido.
    ///
    /// ⚠️ **Cor de rótulo, não de valor** (`Text2`): ela diz o que o número SIGNIFICA e não é parte
    /// dele. Um `1.20 m` todo na mesma cor lê-se como um só campo de texto.
    pub suffix: Option<&'static str>,
}

impl NumberInput {
    pub fn new(id: NodeId, label: impl Into<String>, value: f64) -> Self {
        Self {
            id,
            label: label.into(),
            value,
            step: 1.0,
            min: None,
            max: None,
            state: TextInputState::Normal,
            hover_t: crate::motion::SETTLED,
            suffix: None,
        }
    }

    /// **A unidade colada ao número** — ver [`NumberInput::suffix`].
    #[must_use]
    pub fn suffix(mut self, suffix: Option<&'static str>) -> Self {
        self.suffix = suffix;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self.value = self.value.max(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self.value = self.value.min(max);
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    /// **O par que o store publica** — `(estado, quanto do hover está presente)`.
    /// Irmão exacto do [`super::TextInput::visual`].
    #[must_use]
    pub fn visual(self, v: (TextInputState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    /// Bump value by `+step`, clamped to `max`.
    pub fn increment(&mut self) {
        self.value = self.clamp(self.value + self.step);
    }

    /// Bump value by `-step`, clamped to `min`.
    pub fn decrement(&mut self) {
        self.value = self.clamp(self.value - self.step);
    }

    fn clamp(&self, v: f64) -> f64 {
        let mut v = v;
        if let Some(min) = self.min {
            v = v.max(min);
        }
        if let Some(max) = self.max {
            v = v.min(max);
        }
        v
    }

    pub fn up_rect(&self, host: Rect) -> Rect {
        stepper_up_rect(host)
    }

    pub fn down_rect(&self, host: Rect) -> Rect {
        stepper_down_rect(host)
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::NumberInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .numeric_value(self.value);
        if let Some(min) = self.min {
            builder = builder.numeric_value_min(min);
        }
        if let Some(max) = self.max {
            builder = builder.numeric_value_max(max);
        }
        builder.build()
    }
}

/// Canonical minimum width (px) for ANY number input — boxed
/// [`paint_number_input_with_buffer`] or chip
/// [`crate::widget::paint_number_chip`]. Sized to fit 7 digit-chars
/// at the canon Sm font (12 px) + canonical padding + stepper column.
///
/// **UI canon post-2026-05-24:** callers MUST reserve at least this
/// many px of horizontal space for a number input. Panel layouts that
/// scale chip width with available space (e.g. slider+chip composites)
/// must clamp the chip below this floor. User feedback:
/// "não permita que a caixa seja redimencionada para menor que isso".
pub const MIN_W_PX: f32 = 72.0; // LITERAL-PX-OK: ~3-4 digits at Sm + Lg.px() left pad + 22 px stepper column; reduced 96→72 (user 2026-05-24: chips were too dominant visually)

/// Width of the up/down stepper column carved out of the right edge
/// of every NumberInput / chip hit rect. Sized 60% of the host height,
/// clamped to 16-22 px so it stays clickable on dense rows but doesn't
/// dominate compact chips.
///
/// Exposed `pub` so `slider_with_chip::paint_number_chip` and the
/// stepper hit-test in `dispatch::number_input` use the SAME formula —
/// the chip canon (post-2026-05-24) paints arrows in this exact column,
/// and the dispatch's `apply_number_stepper_if_hit` carves this same
/// rect for the click→step affordance.
pub fn stepper_width(host: Rect) -> f32 {
    (host.h * 0.6).clamp(16.0, 22.0) // LITERAL-PX-OK: stepper column sized 60% of input height with min/max
}

/// Rect of the `up` arrow (top half of the stepper column). Standalone
/// fn so `paint_number_chip` can paint identical arrows without a
/// `NumberInput` instance.
pub fn stepper_up_rect(host: Rect) -> Rect {
    let chip_w = stepper_width(host);
    Rect::new(host.x + host.w - chip_w, host.y, chip_w, host.h * 0.5)
}

/// Rect of the `down` arrow (bottom half of the stepper column).
pub fn stepper_down_rect(host: Rect) -> Rect {
    let chip_w = stepper_width(host);
    Rect::new(
        host.x + host.w - chip_w,
        host.y + host.h * 0.5,
        chip_w,
        host.h * 0.5,
    )
}

pub fn paint_number_input(
    input: &NumberInput,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_number_input_with_buffer(input, None, 0, None, rect, scene, text_system, theme)
}

/// Like [`paint_number_input`] but renders the in-progress edit
/// buffer + caret line when the input is focused. Pass `Some(buffer)`
/// and a caret byte offset when reading live state from a
/// [`crate::interaction::WidgetStore`]; otherwise pass `None`.
/// `selection_anchor` paints a selection background when non-None
/// and the input is focused.
#[allow(clippy::too_many_arguments)]
pub fn paint_number_input_with_buffer(
    input: &NumberInput,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐⭐⭐ **A superfície é a PORTA** ([`crate::widget::paint_field_surface`]) — o raio pelo tema,
    //    o fundo pelo `field_fill` e a moldura com o eixo do hover. ⛔ As quatro linhas viviam aqui
    //    e a LINHA DE MARCAR passou a precisar delas (2026-09-15): *uma segunda cópia divergiria no
    //    primeiro caso especial*, que é o que este mesmo pintor já pagou ao escrever `Bg1` — a cor
    //    de um CARTÃO — e a deixar a caixa a `0/255` do que está por baixo (report do dono, 14/09).
    crate::widget::paint_field_surface(scene, rect, input.state, input.hover_t, theme);

    let chip_w = stepper_width(rect);
    let pad_x = crate::widget::field_pad_x();
    let value_text_owned;
    let value_text: &str = match buffer {
        Some(b) if input.state == TextInputState::Focused => b,
        _ => {
            value_text_owned = format_number(input.value);
            value_text_owned.as_str()
        }
    };
    // Sm (12 px) instead of Base (13 px) — user feedback 2026-05-24:
    // smaller font reads as "compact input field"; combined with
    // MIN_W_PX, fits 7 digits ("-1234.5" / "12345.67") without
    // overflow.
    let font_size = TypeToken::Sm.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5;
    // Right edge of text-area abuts the stepper column directly — no
    // extra padding on the right. User feedback 2026-05-24:
    // "deixe apenas as setas ocluírem o número" (the previous extra
    // `-pad_x` on the right created an unnecessary blank gap between
    // the cropped number and the arrows).
    let inner_w = (rect.w - pad_x - chip_w).max(0.0);
    let label_color = if input.state == TextInputState::Disabled {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    // Push a clip rect over the text area so long values (e.g.
    // "-141.881" at narrow chip widths) get cropped instead of
    // visually bleeding into the stepper column or beyond the
    // border. Parley wraps on word boundaries; numbers have none, so
    // without this clip the digits would render past `inner_x + inner_w`.
    // UI canon post-2026-05-24: numbers ALWAYS stay inside the box.
    let text_clip = ph2d_vector::Rect::new(
        inner_x as f64,
        rect.y as f64,
        (inner_x + inner_w) as f64,
        (rect.y + rect.h) as f64,
    );
    scene.push_clip(&text_clip);
    if input.state == TextInputState::Focused
        && buffer.is_some()
        && let Some(anchor) = selection_anchor
        && anchor != caret
    {
        let (sel_start, sel_end) = if anchor < caret {
            (anchor, caret)
        } else {
            (caret, anchor)
        };
        let sel_start = sel_start.min(value_text.len());
        let sel_end = sel_end.min(value_text.len());
        let prefix_w = text_system.prefix_width(&value_text[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&value_text[sel_start..sel_end], font_size)
        };
        let sel_x = (inner_x + prefix_w).min(inner_x + inner_w);
        let sel_w = mid_w.min(inner_x + inner_w - sel_x);
        if sel_w > 0.0 {
            let sel_top = rect.y + Spacing::Md.px();
            let sel_bot = rect.y + rect.h - Spacing::Md.px();
            let sel_rect = Rect::new(sel_x, sel_top, sel_w, (sel_bot - sel_top).max(2.0));
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }
    paint_text(
        text_system,
        scene,
        value_text,
        inner_x,
        inner_y,
        font_size,
        inner_w,
        resolve(label_color, theme),
    );
    // ⭐⭐ **A UNIDADE, colada ao número** — ver [`NumberInput::suffix`]. Ela entra aqui, DENTRO do
    //    mesmo recorte do valor, e não num chip à direita: com o campo afundado, um segundo
    //    rectângulo com fundo próprio lê-se como duas caixas (ordem do dono, 2026-09-14).
    //
    // ⚠️ **O `x` dela é medido, não reservado:** ela segue o fim do número, logo um valor curto
    //    não deixa um buraco e um valor comprido empurra-a para fora do recorte — que é o
    //    comportamento certo, porque o VALOR é o que não pode desaparecer.
    if let Some(suffix) = input
        .suffix
        .filter(|_| input.state != TextInputState::Focused)
    {
        let gap = Spacing::Xs.px();
        let num_w = text_system.prefix_width(value_text, font_size);
        let suffix_x = inner_x + num_w + gap;
        let suffix_w = (inner_x + inner_w - suffix_x).max(0.0);
        if suffix_w > 0.0 {
            paint_text(
                text_system,
                scene,
                suffix,
                suffix_x,
                inner_y,
                font_size,
                suffix_w,
                resolve(
                    if input.state == TextInputState::Disabled {
                        ColorToken::TextDisabled
                    } else {
                        ColorToken::Text2
                    },
                    theme,
                ),
            );
        }
    }

    if input.state == TextInputState::Focused && buffer.is_some() {
        let caret_clamped = caret.min(value_text.len());
        let prefix = &value_text[..caret_clamped];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (inner_x + prefix_w).min(inner_x + inner_w);
        let caret_top = rect.y + Spacing::Md.px();
        let caret_bot = rect.y + rect.h - Spacing::Md.px();
        let caret_rect = Rect::new(
            caret_x,
            caret_top,
            StrokeToken::Default.px(),
            (caret_bot - caret_top).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, resolve(ColorToken::Accent, theme)); // LITERAL-PX-OK: caret half-width radius
    }
    scene.pop_layer();

    let icon_color = resolve(ColorToken::Text2, theme);
    paint_icon(
        scene,
        IconId::ChevronUp,
        input.up_rect(rect),
        icon_color,
        StrokeToken::Default.px(),
    );
    paint_icon(
        scene,
        IconId::ChevronDown,
        input.down_rect(rect),
        icon_color,
        StrokeToken::Default.px(),
    );
}

pub fn format_number(v: f64) -> String {
    // An UNBOUNDED quantity reads as the infinity glyph, not a saturated `i64` or "inf". The
    // timeline's Dur box hands INFINITY here when a composition has no authored duration (0 =
    // infinite, Enio 2026-07-28); no other number input passes an infinite value. `\u{221E}`
    // keeps this source ASCII (the value is the `∞` char at runtime, past the tofu-glyph gate).
    if v.is_infinite() {
        return "\u{221E}".to_string();
    }
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v as i64)
    } else {
        format!("{v:.3}")
    }
}

#[cfg(test)]
mod tests;
