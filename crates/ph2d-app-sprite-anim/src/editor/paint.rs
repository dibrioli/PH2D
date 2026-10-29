//! **A pintura de uma aba do editor de animações** — lê a [`super::doc::Layout`] que o documento
//! acabou de calcular; nada aqui decide (os gestos estão no `doc`).

use super::doc::{AnimDoc, DragPaint, Field};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text, paint_text_centered, resolve, stroke_rect,
};
use ph2d_editor_core::widget::paint_card_ghost;
use ph2d_editor_core::widget::sheet_cell::paint_sheet_cell;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimCycleMode, InspectorAnimInfo};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{ImageQuality, VectorScene};

pub(crate) fn button(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    label: &str,
    on: bool,
) {
    let bg = if on {
        ColorToken::Accent
    } else {
        ColorToken::Bg2
    };
    let fg = if on {
        ColorToken::Bg0
    } else {
        ColorToken::Text1
    };
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, Radius::Sm.px()),
        resolve(bg, theme),
    );
    paint_text_centered(
        text,
        scene,
        label,
        r,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
}

pub(crate) fn field(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    shown: &str,
    editing: Option<&str>,
) {
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::Bg2, theme),
    );
    match editing {
        Some(buf) => {
            stroke_rect(
                scene,
                r,
                StrokeToken::Thin.px(),
                resolve(ColorToken::Accent, theme),
            );
            let s = tr_with("anim_editor.caret", &[("text", &buf)]);
            paint_text_centered(
                text,
                scene,
                &s,
                r,
                TypeToken::Sm.px(),
                resolve(ColorToken::Text1, theme),
            );
        }
        None => paint_text_centered(
            text,
            scene,
            shown,
            r,
            TypeToken::Sm.px(),
            resolve(ColorToken::Text1, theme),
        ),
    }
}

fn editing<'a>(doc: &'a AnimDoc, f: &Field) -> Option<&'a str> {
    doc.editing
        .as_ref()
        .filter(|(g, _)| g == f)
        .map(|(_, s)| s.as_str())
}

/// Pinta a aba inteira em `area`.
pub fn paint_doc(
    doc: &mut AnimDoc,
    area: Rect,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let l = doc.lay_out(area, text).clone();
    fill_rounded_rect(scene, area, 0.0, resolve(ColorToken::PanelBg, theme));
    let font = TypeToken::Sm.px();
    let info = doc.snap.info.clone();

    // ── A barra de cima ──
    let play = if doc.playing() {
        tr("anim_editor.pause")
    } else {
        tr("anim_editor.play")
    };
    button(scene, text, theme, l.play, play, doc.playing());
    let fps = doc.fps_label();
    field(scene, text, theme, l.fps, &fps, editing(doc, &Field::Fps));
    let mode = doc.mode();
    for (i, (r, key)) in l
        .modes
        .iter()
        .zip([
            "anim_editor.repeat",
            "anim_editor.once",
            "anim_editor.ping_pong",
        ])
        .enumerate()
    {
        button(
            scene,
            text,
            theme,
            *r,
            tr(key),
            mode == Some(AnimCycleMode::ALL[i]),
        );
    }
    let cols = doc.cols_label();
    field(
        scene,
        text,
        theme,
        l.cols,
        &cols,
        editing(doc, &Field::Cols),
    );
    let rows = doc.rows_label();
    field(
        scene,
        text,
        theme,
        l.rows,
        &rows,
        editing(doc, &Field::Rows),
    );
    if let (Some((t, use_)), Some(g)) = (l.banner, doc.proposal()) {
        fill_rounded_rect(
            scene,
            t,
            frame_radius(theme, Radius::Sm.px()),
            resolve(ColorToken::Bg2, theme),
        );
        let msg = tr_with(
            "anim_editor.looks_like",
            &[("cols", &g.cols), ("rows", &g.rows)],
        );
        paint_text(
            text,
            scene,
            &msg,
            t.x + Spacing::Sm.px(),
            t.y,
            font,
            t.w,
            resolve(ColorToken::Text1, theme),
        );
        button(scene, text, theme, use_, tr("anim_editor.use_grid"), true);
    }

    // ── A lista ──
    paint_list(doc, &l, info.as_ref(), scene, text, theme);

    // ── A folha ──
    let frames = doc.shown_frames();
    match doc.snap.sheet.clone() {
        Some(sheet) => {
            fill_rounded_rect(scene, l.sheet, 0.0, resolve(ColorToken::Bg0, theme));
            scene.draw_stable_image(
                &sheet.image,
                (
                    f64::from(l.sheet.x),
                    f64::from(l.sheet.y),
                    f64::from(l.sheet.x + l.sheet.w),
                    f64::from(l.sheet.y + l.sheet.h),
                ),
                ImageQuality::Medium,
            );
            let cells = doc.snap.hframes.max(1) * doc.snap.vframes.max(1);
            for c in 0..cells {
                stroke_rect(
                    scene,
                    doc.cell_rect(c),
                    StrokeToken::Hairline.px(),
                    resolve(ColorToken::Border, theme),
                );
            }
            for c in 0..cells {
                let order: Vec<String> = frames
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| **f == c)
                    .map(|(i, _)| (i + 1).to_string())
                    .collect();
                if order.is_empty() {
                    continue;
                }
                let r = doc.cell_rect(c);
                stroke_rect(
                    scene,
                    r,
                    StrokeToken::Thick.px(),
                    resolve(ColorToken::Accent, theme),
                );
                let s = order.join(tr("anim_editor.order_sep"));
                paint_text(
                    text,
                    scene,
                    &s,
                    r.x + Spacing::Sm.px(),
                    r.y,
                    font,
                    r.w,
                    resolve(ColorToken::Accent, theme),
                );
            }
        }
        None => paint_text(
            text,
            scene,
            tr("anim_editor.no_pixels"),
            l.sheet.x,
            l.sheet.y,
            font,
            l.sheet.w,
            resolve(ColorToken::Text3, theme),
        ),
    }

    // ── A faixa de quadros ──
    doc.painted = paint_strip(doc, &l, &frames, info.as_ref(), scene, text, theme);
}

fn paint_list(
    doc: &AnimDoc,
    l: &super::doc::Layout,
    info: Option<&InspectorAnimInfo>,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let font = TypeToken::Sm.px();
    fill_rounded_rect(
        scene,
        l.list,
        frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::Bg1, theme),
    );
    let rows = info.map(|i| i.rows.as_slice()).unwrap_or_default();
    if rows.is_empty() {
        paint_text(
            text,
            scene,
            tr("anim_editor.no_animations"),
            l.list.x + Spacing::Sm.px(),
            l.list.y,
            font,
            l.list.w,
            resolve(ColorToken::Text3, theme),
        );
    }
    for (i, (r, row)) in l.list_rows.iter().zip(rows).enumerate() {
        if i == doc.selected {
            fill_rounded_rect(
                scene,
                *r,
                frame_radius(theme, Radius::Sm.px()),
                resolve(ColorToken::Bg3, theme),
            );
        }
        if let Some(buf) = editing(doc, &Field::Name(i)) {
            field(scene, text, theme, *r, "", Some(buf));
            continue;
        }
        let playing = info.is_some_and(|x| x.playing && x.current == row.name);
        let label = if playing {
            tr_with("anim_editor.row_playing", &[("name", &row.name)])
        } else {
            row.name.clone()
        };
        let color = if playing {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        paint_text(
            text,
            scene,
            &label,
            r.x + Spacing::Sm.px(),
            r.y,
            font,
            r.w,
            resolve(color, theme),
        );
    }
    button(scene, text, theme, l.add, tr("anim_editor.add"), false);
    if !rows.is_empty() {
        button(
            scene,
            text,
            theme,
            l.remove,
            tr("anim_editor.remove"),
            false,
        );
    }
}

fn paint_strip(
    doc: &AnimDoc,
    l: &super::doc::Layout,
    frames: &[u32],
    info: Option<&InspectorAnimInfo>,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) -> DragPaint {
    let font = TypeToken::Sm.px();
    fill_rounded_rect(
        scene,
        l.strip,
        frame_radius(theme, Radius::Sm.px()),
        resolve(ColorToken::Bg1, theme),
    );
    if frames.is_empty() {
        paint_text(
            text,
            scene,
            tr("anim_editor.strip_hint"),
            l.strip.x + Spacing::Sm.px(),
            l.strip.y,
            font,
            l.strip.w,
            resolve(ColorToken::Text3, theme),
        );
        return DragPaint::default();
    }
    let drag = doc.frame_drag();
    let on_screen = info.is_some_and(|i| {
        i.rows
            .get(doc.selected)
            .is_some_and(|r| r.name == i.current)
    });
    let step = info.map(|i| i.card.step as usize);
    scene.push_clip(&ph2d_vector::Rect::new(
        f64::from(l.strip.x),
        f64::from(l.strip.y),
        f64::from(l.strip.x + l.strip.w),
        f64::from(l.strip.y + l.strip.h),
    ));
    for (i, ((thumb, ms, close), cell)) in l.frames.iter().zip(frames).enumerate() {
        fill_rounded_rect(
            scene,
            *thumb,
            frame_radius(theme, Radius::Sm.px()),
            resolve(ColorToken::Bg2, theme),
        );
        if let Some(sheet) = &doc.snap.sheet {
            paint_sheet_cell(
                scene,
                sheet,
                doc.snap.hframes,
                doc.snap.vframes,
                *cell,
                *thumb,
            );
        }
        if drag.is_some_and(|d| d.from == i) {
            // O lugar de onde o quadro saiu fica apagado enquanto ele anda.
            fill_rounded_rect(
                scene,
                *thumb,
                frame_radius(theme, Radius::Sm.px()),
                resolve(ColorToken::BgScrim, theme),
            );
        }
        if doc.frame_selected == Some(i) || (on_screen && step == Some(i)) {
            stroke_rect(
                scene,
                *thumb,
                StrokeToken::Thick.px(),
                resolve(ColorToken::Accent, theme),
            );
        }
        paint_text(
            text,
            scene,
            &(i + 1).to_string(),
            thumb.x + Spacing::Sm.px(),
            thumb.y,
            font,
            thumb.w,
            resolve(ColorToken::Text2, theme),
        );
        if frames.len() > 1 {
            paint_text_centered(
                text,
                scene,
                tr("anim_editor.remove_frame"),
                *close,
                font,
                resolve(ColorToken::Text2, theme),
            );
        }
        let shown = doc.ms_label(i);
        field(
            scene,
            text,
            theme,
            *ms,
            &shown,
            editing(doc, &Field::FrameMs(i)),
        );
    }
    scene.pop_layer();
    let mut painted = DragPaint::default();
    if let Some((gap, bar)) = l.marker {
        fill_rounded_rect(
            scene,
            bar,
            frame_radius(theme, Radius::Sm.px()),
            resolve(ColorToken::Accent, theme),
        );
        painted.marker = Some(gap);
    }
    if let Some(d) = drag
        && let (Some(f), Some(cell)) = (l.frames.get(d.from), frames.get(d.from))
    {
        let thumb = f.0;
        let mut content = VectorScene::new();
        if let Some(sheet) = &doc.snap.sheet {
            paint_sheet_cell(
                &mut content,
                sheet,
                doc.snap.hframes,
                doc.snap.vframes,
                *cell,
                thumb,
            );
        }
        // A borda separa o fantasma (meio transparente) da miniatura por cima da qual passa.
        stroke_rect(
            &mut content,
            thumb,
            StrokeToken::Thick.px(),
            resolve(ColorToken::Accent, theme),
        );
        paint_card_ghost(
            scene,
            &content,
            thumb,
            frame_radius(theme, Radius::Sm.px()),
            Some(resolve(ColorToken::Bg2, theme)),
            d.grab,
            d.at,
        );
        painted.ghost = Some(thumb);
    }
    painted
}
