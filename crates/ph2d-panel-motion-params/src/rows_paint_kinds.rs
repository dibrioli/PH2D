//! The per-`ParamRow`-kind painters, split out of `rows_paint.rs` so the `paint_rows`
//! dispatcher stays under the 200-LOC fn cap AND `rows_paint.rs` under the 600-LOC file
//! cap (extracting them INTO `rows_paint.rs` traded one cap for the other). `use super::*`
//! is `rows_paint`'s import scope; the row structs come from the crate root.

use super::*;
use crate::{ChannelsRow, ColorRow, EnumRow, ScalarRow, SourceRow, ToggleRow};

/// **A row DIRIGIDA (doc 58) — o único caso que não é um widget.** O fio decide o número,
/// então não há nada a registrar: sem hit rect, sem arrasto, sem id no store. É por isso que
/// ela sai do laço em vez de virar mais um braço parecido com os outros — os oito braços
/// restantes registram e despacham; este só *mostra*, e o acento diz que o valor vem de fora.
/// (Extraída para o `paint_rows` caber no teto de 200 LOC de fn de painel, HR-18.)
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_driven_row(
    row: &ScalarRow,
    inner_x: f32,
    inner_w: f32,
    y: f32,
    label_font: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let mid = y + (ROW_H_PX - label_font) * 0.5;
    paint_text(
        text_system,
        scene,
        &row.label,
        inner_x,
        mid,
        label_font,
        DEFAULT_LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
    let display = row_value(
        normalized_track(row.value, row.min, (row.max - row.min).max(f64::EPSILON)),
        row.min,
        row.max,
        row.integer,
    );
    paint_text(
        text_system,
        scene,
        // The SAME formatter the chip uses — a second one would show the same
        // number with two faces.
        &ph2d_editor_core::widget::format_number(display),
        inner_x + DEFAULT_LABEL_W,
        mid,
        label_font,
        inner_w - DEFAULT_LABEL_W,
        // The accent says it: this number is coming from somewhere else.
        resolve(ColorToken::Accent, theme),
    );
}

/// The **Scalar** row — the ordinary slider+chip, the common case. Returns the advanced
/// `y`. (The driven Scalar is `paint_driven_row`; this is the knob the artist turns.)
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_scalar_row(
    row: &ScalarRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    chip_w: f32,
    row_gap: f32,
    mut y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let slider_id = param_slider_id(i);
    let chip_id = param_chip_id(i);
    let span = (row.max - row.min).max(f64::EPSILON);
    let track = store
        .slider(slider_id)
        .map(|(_, v)| v)
        .unwrap_or(normalized_track(row.value, row.min, span));
    let display = row_value(track, row.min, row.max, row.integer);
    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(inner_x, y, inner_w, ROW_H_PX),
        &row.label,
        track,
        display,
        None,
        slider_id,
        chip_id,
        DEFAULT_LABEL_W,
        chip_w,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += used + row_gap;
    y
}

/// The **Color** row — a label + right-aligned OKLCH swatch (the Down opens the shared
/// picker; the swatch id is anchor-keyed by channel). Returns the advanced `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_color_row(
    row: &ColorRow,
    inner_x: f32,
    inner_w: f32,
    row_gap: f32,
    mut y: f32,
    label_font: f32,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let swatch_w = SwatchSize::Md.px();
    paint_text(
        text_system,
        scene,
        &row.label,
        inner_x,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        DEFAULT_LABEL_W,
        resolve(ColorToken::Text1, theme),
    );
    let swatch_id = param_swatch_id(row.channels[0]);
    let srect = Rect::new(inner_x + inner_w - swatch_w, y, swatch_w, ROW_H_PX);
    let sw = ColorSwatch::new(swatch_id, "Color", row.srgb).size(SwatchSize::Md);
    paint_color_swatch(&sw, srect, scene, theme);
    hit_index.register(swatch_id, srect);
    y += ROW_H_PX + row_gap;
    y
}

/// The **Enum** row — a named segmented selector (label line + option buttons), the mirror
/// of the Vector panel's Cap/Join/Draw rows. Returns the advanced `y`. (Extracted so
/// `paint_rows` stays under the 200-LOC fn cap, HR-18.)
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_enum_row(
    row: &EnumRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    row_gap: f32,
    mut y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    paint_text(
        text_system,
        scene,
        &row.label,
        inner_x,
        y,
        TypeToken::Sm.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Sm.px() + Spacing::Xs.px();
    let k = row.labels.len().min(MAX_ENUM_OPTIONS);
    // Up to 4 buttons across, then wrap; a single option → 1.
    let cols = k.clamp(1, 4); // CLAMP-OK: segmented column count (option-count layout, not a UI metric)
    let gap = Spacing::Sm.px();
    let seg_w = ((inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
    for (opt, caption) in row.labels.iter().enumerate().take(k) {
        let bid = param_enum_id(i, opt);
        let rx = inner_x + (opt % cols) as f32 * (seg_w + gap);
        let ry = y + (opt / cols) as f32 * (ROW_H_PX + gap);
        let brect = Rect::new(rx, ry, seg_w, ROW_H_PX);
        let bstate = store.button_state(bid).unwrap_or(ButtonState::Normal);
        paint_segmented_button(
            brect,
            caption,
            opt == row.selected,
            bstate,
            scene,
            text_system,
            theme,
        );
        hit_index.register(bid, brect);
    }
    let seg_rows = k.div_ceil(cols) as f32;
    y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + row_gap;
    y
}

/// The **Channels** row — a named-channel picker (plan §1.1): the channel LABELS as
/// segmented buttons + a trailing "Custom" (the artist reads "Speed", not a column name),
/// and Custom reveals the live-column chips + a raw text escape. Returns the advanced `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_channels_row(
    row: &ChannelsRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    row_gap: f32,
    mut y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    paint_text(
        text_system,
        scene,
        &row.label,
        inner_x,
        y,
        TypeToken::Sm.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Sm.px() + Spacing::Xs.px();
    let n = row.channels.len(); // Custom is the n-th button
    let k = (n + 1).min(MAX_ENUM_OPTIONS);
    let cols = k.clamp(1, 4); // CLAMP-OK: segmented column count, not a UI metric
    let gap = Spacing::Sm.px();
    let seg_w = ((inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
    for opt in 0..k {
        let caption = if opt < n {
            row.channels[opt].0
        } else {
            "Custom"
        };
        let bid = param_enum_id(i, opt);
        let rx = inner_x + (opt % cols) as f32 * (seg_w + gap);
        let ry = y + (opt / cols) as f32 * (ROW_H_PX + gap);
        let brect = Rect::new(rx, ry, seg_w, ROW_H_PX);
        let bstate = store.button_state(bid).unwrap_or(ButtonState::Normal);
        paint_segmented_button(
            brect,
            caption,
            opt == row.selected,
            bstate,
            scene,
            text_system,
            theme,
        );
        hit_index.register(bid, brect);
    }
    let seg_rows = k.div_ceil(cols) as f32;
    y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + row_gap;
    // Custom selected: the live-column picker (the roadmap's *dropdown populated at
    // runtime*) + the raw text field as the escape.
    if row.selected >= n {
        // Chips for the columns the UPSTREAM stream actually carries: the artist clicks a
        // REAL name instead of guessing. Ids live in a range above the curated segments
        // (`CHANNELS_EXTRA_BASE`).
        let ext = row.extra.len().min(MAX_ENUM_OPTIONS);
        if ext > 0 {
            paint_text(
                text_system,
                scene,
                "From stream",
                inner_x,
                y,
                TypeToken::Sm.px(),
                inner_w,
                resolve(ColorToken::Text2, theme),
            );
            y += TypeToken::Sm.px() + Spacing::Xs.px();
            let ecols = ext.clamp(1, 4); // CLAMP-OK: segmented column count
            let egap = Spacing::Sm.px();
            let ew = ((inner_w - egap * (ecols as f32 - 1.0)) / ecols as f32).max(1.0);
            for j in 0..ext {
                let bid = param_enum_id(i, CHANNELS_EXTRA_BASE + j);
                let rx = inner_x + (j % ecols) as f32 * (ew + egap);
                let ry = y + (j / ecols) as f32 * (ROW_H_PX + egap);
                let brect = Rect::new(rx, ry, ew, ROW_H_PX);
                let bstate = store.button_state(bid).unwrap_or(ButtonState::Normal);
                paint_segmented_button(
                    brect,
                    &row.extra[j],
                    row.extra[j] == row.custom,
                    bstate,
                    scene,
                    text_system,
                    theme,
                );
                hit_index.register(bid, brect);
            }
            let erows = ext.div_ceil(ecols) as f32;
            y += erows * ROW_H_PX + (erows - 1.0) * egap + Spacing::Xs.px();
        }
        // The raw text field for anything not listed (honest placeholder, never "e.g. sin(t)").
        let used = paint_text_row(
            Rect::new(inner_x, y, inner_w, ROW_H_PX),
            "Column",
            "e.g. inv_mass, id",
            param_text_id(i),
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += used + row_gap;
    }
    y
}

/// The **Source** row — a source picker (doc 65): the names the app published (drawn
/// shapes) as chips + a text field for a name not yet drawn. Returns the advanced `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_source_row(
    row: &SourceRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    row_gap: f32,
    mut y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    paint_text(
        text_system,
        scene,
        &row.label,
        inner_x,
        y,
        TypeToken::Sm.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Sm.px() + Spacing::Xs.px();
    let n = row.options.len().min(MAX_ENUM_OPTIONS);
    if n > 0 {
        paint_text(
            text_system,
            scene,
            "Drawn shapes",
            inner_x,
            y,
            TypeToken::Sm.px(),
            inner_w,
            resolve(ColorToken::Text2, theme),
        );
        y += TypeToken::Sm.px() + Spacing::Xs.px();
        let cols = n.clamp(1, 4); // CLAMP-OK: segmented column count, not a UI metric
        let gap = Spacing::Sm.px();
        let seg_w = ((inner_w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
        for j in 0..n {
            let bid = param_enum_id(i, j);
            let rx = inner_x + (j % cols) as f32 * (seg_w + gap);
            let ry = y + (j / cols) as f32 * (ROW_H_PX + gap);
            let brect = Rect::new(rx, ry, seg_w, ROW_H_PX);
            let bstate = store.button_state(bid).unwrap_or(ButtonState::Normal);
            paint_segmented_button(
                brect,
                &row.options[j],
                row.options[j] == row.current,
                bstate,
                scene,
                text_system,
                theme,
            );
            hit_index.register(bid, brect);
        }
        let seg_rows = n.div_ceil(cols) as f32;
        y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + Spacing::Xs.px();
    }
    // The raw text field for a name not (yet) in the list — the honest escape.
    let used = paint_text_row(
        Rect::new(inner_x, y, inner_w, ROW_H_PX),
        "Name",
        "e.g. a drawn shape",
        param_text_id(i),
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += used + row_gap;
    y
}

/// The **Toggle** row — a real checkbox (label + box on the left; the box owns the click,
/// and the dispatch flips its value + fires Toggled). Returns the advanced `y`.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_toggle_row(
    row: &ToggleRow,
    i: usize,
    inner_x: f32,
    inner_w: f32,
    row_gap: f32,
    mut y: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let cb_id = param_checkbox_id(i);
    let (cb_state, _) = store
        .checkbox(cb_id)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let value = if row.on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let cb = Checkbox::new(cb_id, row.label.clone())
        .state(cb_state)
        .value(value);
    let crect = Rect::new(inner_x, y, inner_w, ROW_H_PX);
    paint_checkbox(&cb, crect, scene, text_system, theme);
    hit_index.register(cb_id, crect);
    y += ROW_H_PX + row_gap;
    y
}
