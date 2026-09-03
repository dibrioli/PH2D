//! **O CAMPO NUMÉRICO de uma linha** — o chip: texto, cursor, selecção, recorte e as **setinhas**.
//!
//! ⚠️ **Saiu do [`super`] por TETO DE LOC** (594 > 500, 2026-09-02), e o corte é de **assunto**: o
//! ficheiro-mãe responde *como uma linha de propriedade se compõe*; este responde *como um número
//! se edita*. As duas perguntas crescem por motivos diferentes — a primeira com o desenho da
//! caixa, a segunda com o que um campo de texto tem de saber.
//!
//! ⛔ **O canon das SETINHAS fica** (`architecture_no_chip_without_steppers`): todo chip numérico
//! do app pinta setas, e o despacho recorta a mesma coluna para o clique→passo. É por isso que a
//! caixa única as leva DENTRO em vez de as apagar — *apagar o desenho de um alvo que continua
//! clicável é fabricar um controlo invisível.*

use super::*;

/// Paint the canonical numeric chip — background, optional focus border,
/// centered text, caret, selection highlight, **plus the up/down stepper
/// arrows** carved from the right edge of the rect.
///
/// **Canon (post-2026-05-24):** every numeric chip in the app paints
/// arrows. The dispatch's [`apply_number_stepper_if_hit`] already carves
/// the same right-edge column for click→step, and
/// [`crate::widget::number_input::stepper_width`] is the single source
/// of truth for that column's width — chip and dispatch always agree.
///
/// Pre-2026-05-24 there was a "pill" variant (no arrows) used by
/// slider+chip composites; that variant produced "chip looks like
/// plain text" affordance and forced every panel to remember
/// [`crate::interaction::WidgetStore::mark_chip_no_stepper`] to avoid
/// a phantom-stepper hold bug. Both problems went away by always
/// painting arrows (and dropping the no-stepper opt-out in
/// [`crate::interaction::WidgetStore::link_slider_number`]).
///
/// Used by [`paint_slider_with_chip`] and callable directly when a chip
/// needs to live somewhere a slider row layout doesn't fit.
///
/// `display_override` wins over `value` when present (e.g. for chips
/// that display engineering units while the linked slider still drives
/// a 0..1 normalised value).
///
/// [`apply_number_stepper_if_hit`]: crate::interaction::dispatch
#[allow(clippy::too_many_arguments)]
pub fn paint_number_chip(
    rect: Rect,
    state: TextInputState,
    value: f64,
    display_override: Option<&str>,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_number_chip_inner(
        rect,
        state,
        value,
        display_override,
        buffer,
        caret,
        selection_anchor,
        scene,
        text_system,
        theme,
        true,
    );
}

/// ⭐⭐ **O MESMO chip, SEM a superfície própria** — para quando ele vive DENTRO de outra caixa.
///
/// É a metade que a [`super::property_box`] precisa: a caixa única já pinta o fundo, o
/// preenchimento e a moldura da linha, e um segundo pil dentro dela seria a caixa-dentro-da-caixa
/// que o redesenho existe para apagar.
///
/// ⚠️ **Irmão, não cópia.** As duas portas chamam o mesmo corpo, e por isso o valor, o cursor, a
/// selecção, o recorte do texto **e as SETINHAS** comportam-se igual nos dois sítios. ⛔ Reescrever
/// um campo de texto ao lado do que existe é como o `slider_with_chip` nasceu (*"o slider do painel
/// X parece diferente do do painel Y"*), um nível abaixo.
#[allow(clippy::too_many_arguments)]
pub fn paint_number_chip_flat(
    rect: Rect,
    state: TextInputState,
    value: f64,
    display_override: Option<&str>,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_number_chip_inner(
        rect,
        state,
        value,
        display_override,
        buffer,
        caret,
        selection_anchor,
        scene,
        text_system,
        theme,
        false,
    );
}

/// O corpo partilhado. `draw_surface` é a ÚNICA diferença entre as duas portas.
#[allow(clippy::too_many_arguments)]
fn paint_number_chip_inner(
    rect: Rect,
    state: TextInputState,
    value: f64,
    display_override: Option<&str>,
    buffer: Option<&str>,
    caret: usize,
    selection_anchor: Option<usize>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    draw_surface: bool,
) {
    let focused = state == TextInputState::Focused;
    let radius = Radius::Xs.px();
    if draw_surface {
        let bg = if focused {
            ColorToken::Bg2
        } else {
            ColorToken::Bg3
        };
        fill_rounded_rect(scene, rect, radius, resolve(bg, theme));
        if focused {
            stroke_rounded_rect(scene, rect, radius, 2.0, resolve(ColorToken::Accent, theme));
        }
    }
    let display_owned;
    let display = match (focused, buffer, display_override) {
        (true, Some(b), _) => b,
        (_, _, Some(s)) => s,
        _ => {
            display_owned = crate::interaction::format_number(value);
            display_owned.as_str()
        }
    };
    // Reserve the right-edge column for the stepper arrows and center the
    // text in what's left. Without this, long values would slide into the
    // arrow column visually.
    let chip_stepper_w = stepper_width(rect);
    let text_area_w = (rect.w - chip_stepper_w).max(0.0);
    let text_area_right = rect.x + text_area_w;
    let font_size = TypeToken::Xs.px();
    let total_w = if display.is_empty() {
        0.0
    } else {
        text_system
            .layout(display, font_size, f32::INFINITY)
            .width()
    };
    let text_start = rect.x + (text_area_w - total_w) * 0.5;
    // Clip the text-area rect so long values (e.g. "-141.881" at
    // narrow chip widths) crop at the chip border instead of bleeding
    // into the stepper column. UI canon post-2026-05-24: numbers
    // ALWAYS stay inside the box, regardless of digit length.
    let text_clip = ph2d_vector::Rect::new(
        rect.x as f64,
        rect.y as f64,
        text_area_right as f64,
        (rect.y + rect.h) as f64,
    );
    scene.push_clip(&text_clip);
    if focused
        && let Some(anchor) = selection_anchor
        && anchor != caret
    {
        let (sel_start, sel_end) = if anchor < caret {
            (anchor, caret)
        } else {
            (caret, anchor)
        };
        let sel_start = sel_start.min(display.len());
        let sel_end = sel_end.min(display.len());
        let prefix_w = text_system.prefix_width(&display[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&display[sel_start..sel_end], font_size)
        };
        let sel_top = rect.y + Spacing::Xs.px();
        let sel_bot = rect.y + rect.h - Spacing::Xs.px();
        let sel_x = (text_start + prefix_w).clamp(rect.x + 2.0, text_area_right - 2.0); // CLAMP-OK: rect-bound text-selection clamp (bounds well-formed by construction)
        let sel_w = mid_w.min(text_area_right - 2.0 - sel_x);
        if sel_w > 0.0 {
            let sel_rect = Rect::new(sel_x, sel_top, sel_w, (sel_bot - sel_top).max(2.0));
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }
    // Centered text inside the reduced text-area rect (not the full chip).
    let text_area_rect = Rect::new(rect.x, rect.y, text_area_w, rect.h);
    paint_text_centered(
        text_system,
        scene,
        display,
        text_area_rect,
        font_size,
        resolve(ColorToken::Text1, theme),
    );
    if focused {
        let caret_clamped = caret.min(display.len());
        let prefix = &display[..caret_clamped];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (text_start + prefix_w).clamp(rect.x + 2.0, text_area_right - 2.0); // CLAMP-OK: rect-bound caret clamp (bounds well-formed by construction)
        let caret_top = rect.y + Spacing::Xs.px();
        let caret_bot = rect.y + rect.h - Spacing::Xs.px();
        let caret_rect = Rect::new(
            caret_x,
            caret_top,
            StrokeToken::Default.px(),
            (caret_bot - caret_top).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, resolve(ColorToken::Accent, theme)); // LITERAL-PX-OK: caret half-width radius
    }
    scene.pop_layer();
    // Stepper arrows (up + down) on the right edge — same column the
    // dispatch carves for `apply_number_stepper_if_hit`. Color tracks the
    // chip text color (Text2) — dimmer than the value text to keep the
    // chip from looking busy.
    let icon_color = resolve(ColorToken::Text2, theme);
    paint_icon(
        scene,
        IconId::ChevronUp,
        stepper_up_rect(rect),
        icon_color,
        StrokeToken::Default.px(),
    );
    paint_icon(
        scene,
        IconId::ChevronDown,
        stepper_down_rect(rect),
        icon_color,
        StrokeToken::Default.px(),
    );
}
