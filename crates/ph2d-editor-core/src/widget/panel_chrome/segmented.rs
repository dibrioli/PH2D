//! O SELECTOR SEGMENTADO — o botão de grupo e os dois painters que o arranjam em fila.
//!
//! ⚠️ **Saiu do `panel_chrome.rs` porque a pergunta é OUTRA:** o pai responde *que forma tem um
//! painel* (superfície, cabeçalho, fechar, redimensionar, clamp) e este responde *como um grupo de
//! 2-3 opções se desenha*. O corte é o que a entrada do `FILE_OVERAGE_OK` prometia desde a Wave 11
//! (*"sub-folder split is a follow-up"*) — e é o `command_palette.rs` + `command_palette/layout.rs`
//! outra vez: arquivo irmão sob o mesmo módulo, então nenhum caminho de chamada muda.

use crate::interaction::HitIndex;
use crate::paint::{paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;
/// Canonical segmented / toggle-group button — **the single source of
/// truth for grouped 2–3-way selectors** (Mode pickers, render-strategy
/// switchers, etc.). Always draws a discrete outline so an UNSELECTED
/// segment still reads as a button; the bare `paint_button(Default)`
/// ghost look failed that (inactive halves looked like plain text).
///
/// Selected → `Bg2` fill + `Accent` outline + `Text1`. Unselected →
/// `Bg3` fill + `Border` outline + `Text2`. Caller registers the hit
/// rect against the segment's `NodeId`.
pub fn paint_segmented_button(
    rect: Rect,
    label: &str,
    selected: bool,
    visual: (crate::widget::ButtonState, f32),
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_segmented_button_in_group(
        rect,
        label,
        selected,
        visual,
        scene,
        text_system,
        theme,
        crate::widget::GroupCell {
            col: crate::widget::GroupPos::Only,
            row: crate::widget::GroupPos::Only,
        },
    );
}

/// ⭐⭐ **O mesmo segmento, sabendo ONDE está no grupo** — a lei do Blender que o dono apontou:
/// numa fileira de vizinhos só as bordas de FORA arredondam.
#[allow(clippy::too_many_arguments)]
pub fn paint_segmented_button_in_group(
    rect: Rect,
    label: &str,
    selected: bool,
    visual: (crate::widget::ButtonState, f32),
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    pos: crate::widget::GroupCell,
) {
    use crate::widget::ButtonState;
    let (state, hover_t) = visual;
    // ⭐ Raio e moldura pela porta do TEMA (`paint::frame_radius` / `paint::stroke_frame`): no
    //    clássico é o `Sm` e a moldura de sempre; num tema moderno é o `4` do Godot e nenhuma
    //    moldura — um segmento seleccionado diz-se pelo fundo e pelo texto, não por um contorno.
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let feel = match state {
        _ if selected || state == ButtonState::Pressed => ph2d_tokens::visuals::Feel::Active,
        ButtonState::Hovered => ph2d_tokens::visuals::Feel::Hovered,
        ButtonState::Focused => ph2d_tokens::visuals::Feel::Focused,
        ButtonState::Disabled => ph2d_tokens::visuals::Feel::Disabled,
        _ => ph2d_tokens::visuals::Feel::Rest,
    };
    // Hover/press feedback (Enio 2026-07-04): segmented buttons used to render ONLY selected-vs-not, so
    // hovering / pressing showed nothing. Read the widget `state` the dispatcher sets and deepen the fill.
    let (bg, fg, border) = if selected {
        // Selected: accent-outlined; a press deepens the fill for feedback.
        let bg = if state == ButtonState::Pressed {
            ColorToken::AccentSoft
        } else {
            ColorToken::Bg2
        };
        (bg, ColorToken::Text1, ColorToken::Accent)
    } else {
        // Unselected: flat surface — Normal → Bg3, Hovered → BgElev, Pressed → AccentSoft.
        let bg = match state {
            ButtonState::Pressed => ColorToken::AccentSoft,
            ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
            _ => ColorToken::Bg3,
        };
        (bg, ColorToken::Text2, ColorToken::Border)
    };
    // ⚠️ **O REPOUSO mistura com o HOVER, e é o `t` que escolhe** — a mesma expressão do
    //    [`crate::widget::Button::bg_color`], pela mesma razão: se o *estado* escolhesse, sair do
    //    hover seria instantâneo (no quadro em que o rato sai, `state` já é `Normal` e a cor de
    //    repouso já é a de destino, então não sobra nada para animar). `Pressed`/`Focused` seguem
    //    **duros**: não são uma *quantidade* de nada.
    let fill = if hover_t < 1.0
        && !selected
        && matches!(state, ButtonState::Normal | ButtonState::Hovered)
    {
        // ⚠️ A mistura acontece em `ph2d_tokens::Color` (o token RESOLVIDO), nunca na cor de
        //    pintura: é a moeda que o `blend_token_color` fala, e é a mesma ponte que o
        //    `paint_button` atravessa uma linha antes de encher o retângulo.
        crate::motion::blend_token_color(
            Some(ColorToken::Bg3.resolve(theme)),
            Some(ColorToken::BgElev.resolve(theme)),
            hover_t,
        )
        .map(|c| ph2d_vector::Color::from_rgba8(c.r, c.g, c.b, c.a)) // LITERAL-COLOR-OK: token-bridge
        .unwrap_or_else(|| resolve(bg, theme))
    } else {
        resolve(bg, theme)
    };
    // ⭐⭐ **Só as bordas de FORA do grupo arredondam** — a lei do Blender (report do dono,
    //    2026-09-06). Um segmento sozinho continua a arredondar os quatro cantos.
    crate::paint::fill_rounded_rect_radii(scene, rect, pos.radii(radius), fill);
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        feel,
        StrokeToken::Default.px(),
        resolve(border, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
}

/// Canonical horizontal gap (px) between the segments of a
/// toggle/segmented group. **Single source of truth** — both
/// [`paint_segmented_group`] and the typed `RadioGroup` segmented
/// painter read this so the spacing never diverges (some groups, e.g.
/// the Widget Gallery's Low/Mid/High, previously had zero gap).
pub fn segmented_gap() -> f32 {
    crate::widget::SEGMENT_HAIRLINE
}

/// Canonical segmented / toggle GROUP: lays out `segments` as N
/// equal-width buttons across `rect` with [`segmented_gap`] between
/// them, paints each via [`paint_segmented_button`], and registers each
/// segment's hit rect. **The single source of truth for segmented-group
/// layout** — call sites pass `(label, selected, node_id)` and never
/// compute per-segment rects or gaps inline.
pub fn paint_segmented_group(
    rect: Rect,
    segments: &[(&str, bool, NodeId)],
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &crate::interaction::WidgetStore,
    hit_index: &mut HitIndex,
) {
    let n = segments.len();
    if n == 0 {
        return;
    }
    // ⭐⭐ **Uma fileira de irmãos ENCOSTA, e só as pontas de fora arredondam** — a lei do Blender.
    for (i, (rect_i, cell)) in crate::widget::segment_rects(rect, n)
        .into_iter()
        .enumerate()
    {
        let (label, selected, id) = segments[i];
        let visual = store.button_visual(id);
        paint_segmented_button_in_group(
            rect_i,
            label,
            selected,
            visual,
            scene,
            text_system,
            theme,
            cell,
        );
        hit_index.register(id, rect_i);
    }
}

/// Like [`paint_segmented_group`] but **adaptive**: when the labels
/// don't fit in `rect.w` at their natural width, demotes buttons
/// from the END of the list to NEW ROWS below — each demoted button
/// takes the full row width. Returns the total height used (≥ `rect.h`).
///
/// UI canon post-2026-05-24: instead of letting a button's label
/// wrap inside the button (which the canonical button painter does
/// not gracefully support — the `Hand-\npacked` artifact in the
/// 2026-05-24 screenshot), the group reflows by stacking the
/// overflow buttons vertically. Callers advance `cur_y` by the
/// returned height instead of `rect.h`.
///
/// Demotion order: last button first, then next-to-last, etc., so
/// the visual "primary" buttons (left) stay on the top row.
pub fn paint_segmented_group_adaptive(
    rect: Rect,
    segments: &[(&str, bool, NodeId)],
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &crate::interaction::WidgetStore,
    hit_index: &mut HitIndex,
) -> f32 {
    let seg_state = |id: NodeId| store.button_visual(id);
    let n = segments.len();
    if n == 0 {
        return 0.0;
    }
    let _gap = segmented_gap();
    let labels: Vec<&str> = segments.iter().map(|(l, _, _)| *l).collect();
    let widths = crate::widget::segmented_adaptive::segmented_natural_widths(&labels, text_system);
    // ONE answer to "how does this group wrap", shared with the measurer. See `segmented_row_counts`:
    // greedy flow, so a long list packs every row instead of stacking one button per line.
    let rows = crate::widget::segmented_adaptive::segmented_row_counts(rect.w, &widths);

    let row_h = rect.h;
    // ⭐⭐⭐ **Um grupo que QUEBRA continua a ser UM corpo** — as fileiras encostam entre si como
    //    as peças encostam dentro de cada uma, e só os quatro cantos do BLOCO arredondam. É a lei
    //    do Blender nas duas direcções (`block_cells`), e é o que impede uma escolha entre irmãos
    //    de se ler como dois controlos por ter mudado de linha.
    let block = crate::widget::block_cells(Rect::new(rect.x, rect.y, rect.w, 0.0), &rows, row_h);
    let mut i = 0usize;
    for (r, count) in rows.iter().enumerate() {
        for k in 0..*count {
            let (label, selected, id) = segments[i + k];
            let (seg, cell) = block[r][k];
            paint_segmented_button_in_group(
                seg,
                label,
                selected,
                seg_state(id),
                scene,
                text_system,
                theme,
                cell,
            );
            hit_index.register(id, seg);
        }
        i += count;
    }

    crate::widget::grid_height(rows.len(), row_h)
}
