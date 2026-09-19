//! **AS DUAS FILEIRAS QUE SÃO UM PARAM DE TEXTO COM CARA DE SELECTOR** — o picker de CANAL e o
//! picker de FONTE.
//!
//! ⚠️ **Irmãs por RESPONSABILIDADE do [`super`]** (tecto de `600` linhas por ficheiro de painel,
//! 2026-09-19): lá moram as fileiras cujo widget É o param (um número, uma cor, um interruptor);
//! aqui as duas cujo doc já dizia o mesmo de si — *«pure UI sugar over `TextRow`»*, *«a
//! artist-facing face of a TEXT param»*. O substrato das duas é intocado: o param de texto
//! continua a ser a fonte da verdade, e o que elas acrescentam é uma maneira de o escolher.
//!
//! ⛔ *A cura de um tecto é o CORTE, nunca uma entrada numa lista de isenção.*

use super::*;
use crate::{ChannelsRow, SourceRow};
use ph2d_editor_core::paint::paint_text_elided;
use ph2d_editor_core::widget::panel_chrome::paint_segmented_button_in_group;
use ph2d_editor_core::widget::{block_cells, grid_height};
use ph2d_i18n::tr;

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
    paint_text_elided(
        text_system,
        scene,
        &row.label,
        inner_x,
        y,
        TypeToken::Sm.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Sm.px() + ph2d_tokens::control_gap_px();
    let n = row.channels.len(); // Custom is the n-th button
    let k = (n + 1).min(MAX_ENUM_OPTIONS);
    let cols = k.clamp(1, 4); // CLAMP-OK: segmented column count, not a UI metric
    let _gap = Spacing::Xs.px();
    let grows: Vec<usize> = (0..k.div_ceil(cols))
        .map(|r| cols.min(k - r * cols))
        .collect();
    let gblock = block_cells(Rect::new(inner_x, y, inner_w, 0.0), &grows, ROW_H_PX);
    for opt in 0..k {
        let caption = if opt < n {
            row.channels[opt].0
        } else {
            tr("panel.motion_params.rows.custom")
        };
        let bid = param_enum_id(i, opt);
        let (brect, cell) = gblock[opt / cols][opt % cols];
        let bstate = store.button_visual(bid);
        paint_segmented_button_in_group(
            brect,
            caption,
            opt == row.selected,
            bstate,
            scene,
            text_system,
            theme,
            cell,
        );
        hit_index.register(bid, brect);
    }
    y += grid_height(grows.len(), ROW_H_PX) + row_gap;
    // Custom selected: the live-column picker (the roadmap's *dropdown populated at
    // runtime*) + the raw text field as the escape.
    if row.selected >= n {
        // Chips for the columns the UPSTREAM stream actually carries: the artist clicks a
        // REAL name instead of guessing. Ids live in a range above the curated segments
        // (`CHANNELS_EXTRA_BASE`).
        let ext = row.extra.len().min(MAX_ENUM_OPTIONS);
        if ext > 0 {
            paint_text_elided(
                text_system,
                scene,
                tr("panel.motion_params.rows.from_stream"),
                inner_x,
                y,
                TypeToken::Sm.px(),
                inner_w,
                resolve(ColorToken::Text2, theme),
            );
            y += TypeToken::Sm.px() + ph2d_tokens::control_gap_px();
            let ecols = ext.clamp(1, 4); // CLAMP-OK: segmented column count
            let egap = Spacing::Sm.px();
            let _ew = ((inner_w - egap * (ecols as f32 - 1.0)) / ecols as f32).max(1.0);
            let erows: Vec<usize> = (0..ext.div_ceil(ecols))
                .map(|r| ecols.min(ext - r * ecols))
                .collect();
            let eblock = block_cells(Rect::new(inner_x, y, inner_w, 0.0), &erows, ROW_H_PX);
            for j in 0..ext {
                let bid = param_enum_id(i, CHANNELS_EXTRA_BASE + j);
                let (brect, cell) = eblock[j / ecols][j % ecols];
                let bstate = store.button_visual(bid);
                paint_segmented_button_in_group(
                    brect,
                    &row.extra[j],
                    row.extra[j] == row.custom,
                    bstate,
                    scene,
                    text_system,
                    theme,
                    cell,
                );
                hit_index.register(bid, brect);
            }
            y += grid_height(erows.len(), ROW_H_PX) + ph2d_tokens::control_gap_px();
        }
        // The raw text field for anything not listed (honest placeholder, never "e.g. sin(t)").
        let used = paint_text_row(
            Rect::new(inner_x, y, inner_w, ROW_H_PX),
            tr("panel.motion_params.rows.column"),
            tr("panel.motion_params.rows.e_g_inv_mass_id"),
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
    paint_text_elided(
        text_system,
        scene,
        &row.label,
        inner_x,
        y,
        TypeToken::Sm.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Sm.px() + ph2d_tokens::control_gap_px();
    let n = row.options.len().min(MAX_ENUM_OPTIONS);
    if n > 0 {
        paint_text_elided(
            text_system,
            scene,
            tr("panel.motion_params.rows.drawn_shapes"),
            inner_x,
            y,
            TypeToken::Sm.px(),
            inner_w,
            resolve(ColorToken::Text2, theme),
        );
        y += TypeToken::Sm.px() + ph2d_tokens::control_gap_px();
        let cols = n.clamp(1, 4); // CLAMP-OK: segmented column count, not a UI metric
        let gap = Spacing::Xs.px();
        let jrows: Vec<usize> = (0..n.div_ceil(cols))
            .map(|r| cols.min(n - r * cols))
            .collect();
        let jblock = block_cells(Rect::new(inner_x, y, inner_w, 0.0), &jrows, ROW_H_PX);
        for j in 0..n {
            let bid = param_enum_id(i, j);
            let (brect, cell) = jblock[j / cols][j % cols];
            let bstate = store.button_visual(bid);
            paint_segmented_button_in_group(
                brect,
                &row.options[j],
                row.options[j] == row.current,
                bstate,
                scene,
                text_system,
                theme,
                cell,
            );
            hit_index.register(bid, brect);
        }
        let seg_rows = n.div_ceil(cols) as f32;
        y += seg_rows * ROW_H_PX + (seg_rows - 1.0) * gap + ph2d_tokens::control_gap_px();
    }
    // The raw text field for a name not (yet) in the list — the honest escape.
    let used = paint_text_row(
        Rect::new(inner_x, y, inner_w, ROW_H_PX),
        tr("panel.motion_params.rows.name"),
        tr("panel.motion_params.rows.e_g_a_drawn_shape"),
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
