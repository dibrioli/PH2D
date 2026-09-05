//! [`TextInput`] — single-line text field.
//!
//! v1 paints layout-only: caret stays static at `caret_pos`, no IME
//! composing, no selection range. Real input handling lands when the
//! shell wires `winit::Event::KeyboardInput` into the editor (post
//! M13). The data shape here exists so widgets that compose
//! TextInput (NumberInput, Combobox) have a stable contract.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the caret bar. The viewport reserves it on the right so a caret at
/// the very end of a scrolled line stays inside the clip instead of landing on
/// its boundary — where it would be trimmed away exactly while you are typing.
const CARET_W: f32 = 1.0; // LITERAL-PX-OK: a hairline caret

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextInputState {
    #[default]
    Normal,
    Hovered,
    Focused,
    Disabled,
    /// Validation failed; border switches to `Danger`.
    Error,
}

#[derive(Clone, Debug)]
pub struct TextInput {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
    /// Byte offset of the caret within `value`. Out-of-range values
    /// are clamped at paint time. v1 draws the caret only when
    /// `state == Focused`.
    pub caret_byte: usize,
    /// Quanto do hover está presente. [`crate::motion::SETTLED`] = assente no estado que o campo
    /// diz ter, que é o mundo pré-UI-viva byte a byte.
    pub hover_t: f32,
}

impl TextInput {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
            caret_byte: 0,
            hover_t: crate::motion::SETTLED,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.caret_byte = self.value.len();
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    /// **O par que o store publica** — `(estado, quanto do hover está presente)`, numa pergunta só.
    /// Irmão exacto do [`crate::widget::Button::visual`].
    #[must_use]
    pub fn visual(self, v: (TextInputState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// Border tokens chosen by state. Promoted to a free function so
/// `text_area`/`number_input` can reuse the same palette.
pub(crate) fn border_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Border,
        TextInputState::Hovered => ColorToken::BorderEmph,
        TextInputState::Focused => ColorToken::Accent,
        TextInputState::Error => ColorToken::Danger,
        TextInputState::Normal => ColorToken::Border,
    }
}

pub(crate) fn fill_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Bg2,
        _ => ColorToken::Bg1,
    }
}

/// **A cor da borda de um campo, já com o eixo do hover** — a porta ÚNICA dos três pintores da
/// família (`text_input` · `number_input` · `text_area`), pelo mesmo motivo que o
/// [`border_token`] é livre: eles partilham a paleta, e uma segunda cópia da mistura divergiria
/// no primeiro caso especial.
///
/// ⚠️ **Só o par `Normal ⇄ Hovered` interpola.** `Focused`, `Error` e `Disabled` nomeiam um
/// ESTADO, não uma quantidade — um `Accent` a meio caminho leria como *meio-focado*, e meia
/// desactivação não quer dizer nada. É a mesma cerca que o `Button::bg_color` planta.
///
/// ⚠️ **Quem escolhe a cor no eixo é o ESCALAR, não o estado**, e é isso que faz a SAÍDA
/// funcionar: no quadro em que o rato sai, o estado já voltou a `Normal`, então se ele decidisse
/// não haveria nada entre a cor de agora e a de repouso.
pub(crate) fn border_color(
    state: TextInputState,
    hover_t: f32,
    theme: Theme,
) -> ph2d_vector::Color {
    let soft = matches!(state, TextInputState::Normal | TextInputState::Hovered);
    crate::motion::hover_axis(
        soft,
        hover_t,
        Some(ColorToken::Border.resolve(theme)),
        Some(ColorToken::BorderEmph.resolve(theme)),
    )
    .map_or_else(
        || resolve(border_token(state), theme),
        crate::paint::token_to_vello,
    )
}

pub fn paint_text_input(
    input: &TextInput,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_text_input_with_buffer(input, None, None, None, rect, scene, text_system, theme)
}

/// Like [`paint_text_input`] but draws an override `buffer` and
/// caret offset when the caller has a live
/// [`crate::interaction::WidgetStore`] entry for the input. Reading
/// from the store avoids per-frame allocations that would happen if
/// the caller copied `store.text(id)` into `TextInput.value`.
/// `selection_anchor` is the other end of an active selection (for
/// double-click "select all" + Shift+Arrow); when None, no selection
/// is drawn.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_input_with_buffer(
    input: &TextInput,
    buffer: Option<&str>,
    caret: Option<usize>,
    selection_anchor: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ O cromo do campo é do TEMA (`ph2d_tokens::visuals::Chrome`): no clássico o `Radius::Sm`,
    //    o `Bg1` e a moldura permanente de sempre; num tema moderno o raio `4`, um fundo um degrau
    //    abaixo do painel, e **moldura só no foco** (o `LineEdit` do Godot) — ou no erro.
    let chrome = ph2d_tokens::visuals::Chrome::of(theme);
    let radius = chrome.field_radius;
    let fill = if chrome.field_border.is_visible() {
        resolve(fill_token(input.state), theme)
    } else {
        crate::paint::token_to_vello(chrome.field_fill)
    };
    fill_rounded_rect(scene, rect, radius, fill);
    if chrome.field_border.is_visible() {
        let stroke_w = if input.state == TextInputState::Focused {
            chrome.field_focus.width
        } else {
            chrome.field_border.width
        };
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            stroke_w,
            border_color(input.state, input.hover_t, theme),
        );
    } else {
        match input.state {
            TextInputState::Focused => stroke_rounded_rect(
                scene,
                rect,
                radius,
                chrome.field_focus.width,
                crate::paint::token_to_vello(chrome.field_focus.color),
            ),
            TextInputState::Error => {
                stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Danger, theme))
            }
            _ => {}
        }
    }

    let pad_x = Spacing::Lg.px();
    let pad_y = Spacing::Md.px();
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5 - pad_y * 0.0;
    let inner_w = (rect.w - pad_x * 2.0).max(0.0);

    let displayed: &str = buffer.unwrap_or(input.value.as_str());
    let displayed_caret = caret.unwrap_or(input.caret_byte);

    // **A text field is one LINE, so it SCROLLS — it does not wrap.** `paint_text`
    // breaks the run at `max_width`, so a name longer than the box grew a second
    // line and spilled out of the field, over whatever sat below it (Enio,
    // 2026-07-16). The line is laid out unbounded (`f32::INFINITY` — the same "do
    // not wrap" value `prefix_width` itself passes), clipped to the inner box, and
    // slid left just far enough to keep the caret inside: the viewport every text
    // field in every toolkit has.
    //
    // Only a FOCUSED field scrolls. With no caret to follow there is nothing to
    // chase, and a reader looking at an unfocused field wants the BEGINNING of the
    // name — scrolling it to the end would hide the part that identifies it.
    let focused = input.state == TextInputState::Focused;
    let caret_w = focused.then(|| {
        text_system.prefix_width(
            &displayed[..displayed_caret.min(displayed.len())],
            font_size,
        )
    });
    let text_x = inner_x - caret_scroll(focused, inner_w, caret_w.unwrap_or(0.0));
    scene.push_clip(&crate::paint::rect_to_vello(Rect::new(
        inner_x, rect.y, inner_w, rect.h,
    )));

    if input.state == TextInputState::Focused
        && let Some(anchor) = selection_anchor
        && anchor != displayed_caret
    {
        let (sel_start, sel_end) = if anchor < displayed_caret {
            (anchor, displayed_caret)
        } else {
            (displayed_caret, anchor)
        };
        let sel_start = sel_start.min(displayed.len());
        let sel_end = sel_end.min(displayed.len());
        let prefix_w = text_system.prefix_width(&displayed[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&displayed[sel_start..sel_end], font_size)
        };
        // The highlight rides the same viewport as the glyphs it covers; the clip
        // trims whatever runs past the box, so it needs no clamp of its own.
        let sel_x = text_x + prefix_w;
        let sel_w = mid_w;
        if sel_w > 0.0 {
            // ⚠️ O `pad_y` é fixo e a altura do host é variável: num campo mais baixo que dois
            // paddings a subtração fica NEGATIVA — um retângulo que se estende para CIMA do
            // próprio topo.
            //
            // ⚠️ **MEDIDO, e o piso é HIGIENE, não a cura de um defeito visível:** o `push_clip`
            // acima já apara a seleção à caixa interna, então um retângulo invertido é recortado
            // antes de chegar à tela — um gate escrito contra a CENA não consegue distinguir as
            // duas versões, e o que eu tinha escrito passava com o defeito reinstalado. Ele fica
            // porque geometria malformada é lida errada pelo próximo consumidor do retângulo (um
            // hit-test, um recorte diferente), e custa uma chamada.
            let sel_rect = Rect::new(
                sel_x,
                rect.y + pad_y,
                sel_w,
                (rect.h - pad_y * 2.0).max(0.0),
            );
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }

    if displayed.is_empty() && !input.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &input.placeholder,
            inner_x,
            inner_y,
            font_size,
            f32::INFINITY,
            resolve(ColorToken::Text3, theme),
        );
    } else if !displayed.is_empty() {
        let color = if input.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        paint_text(
            text_system,
            scene,
            displayed,
            text_x,
            inner_y,
            font_size,
            f32::INFINITY,
            resolve(color, theme),
        );
    }

    if let Some(caret_w) = caret_w {
        // ⚠️ Mesma aritmética da seleção acima, o mesmo piso e a MESMA medição: o recorte apara,
        // então isto é higiene de geometria, não a cura de algo que se vê.
        let caret_rect = Rect::new(
            text_x + caret_w,
            rect.y + pad_y,
            CARET_W,
            (rect.h - pad_y * 2.0).max(0.0),
        );
        scene.fill_rect(
            crate::paint::rect_to_vello(caret_rect),
            resolve(ColorToken::Accent, theme),
        );
    }
    scene.pop_layer();
}

/// How far the single line is slid LEFT so the caret stays inside the box.
///
/// Pure, so the rule can be stated and tested without a scene. Exactly as far as
/// the caret overhangs, plus the caret's own width — a caret parked ON the right
/// boundary is trimmed by the clip precisely while you are typing at the end of
/// the name, which is the one moment you need to see it. An unfocused field never
/// scrolls: there is no caret to follow, and the reader wants the name's start.
fn caret_scroll(focused: bool, inner_w: f32, caret_w: f32) -> f32 {
    if focused {
        (caret_w - (inner_w - CARET_W)).max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests;
