//! **A pintura de uma aba do editor de animações** — lê a [`super::doc::Layout`] que o documento
//! acabou de calcular; nada aqui decide (os gestos estão no `doc`).

use super::doc::{AnimDoc, Field, format_fps};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rect,
};
use ph2d_editor_core::widget::sheet_cell::paint_sheet_cell;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimCycleMode, InspectorAnimInfo};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme, TypeToken};
use ph2d_vector::{ImageQuality, VectorScene};

fn button(
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
    fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(bg, theme));
    paint_text_centered(
        text,
        scene,
        label,
        r,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
}

fn field(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    shown: &str,
    editing: Option<&str>,
) {
    fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
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
    let l = doc.lay_out(area).clone();
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
    let fps = info
        .as_ref()
        .and_then(|i| i.rows.get(doc.selected))
        .map(|r| {
            tr_with(
                "anim_editor.fps",
                &[("fps", &format_fps(ph2d_ecs::fps_of_ms(r.frame_ms)))],
            )
        })
        .unwrap_or_default();
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
    let cols = tr_with("anim_editor.cols", &[("n", &doc.snap.hframes)]);
    field(
        scene,
        text,
        theme,
        l.cols,
        &cols,
        editing(doc, &Field::Cols),
    );
    let rows = tr_with("anim_editor.rows", &[("n", &doc.snap.vframes)]);
    field(
        scene,
        text,
        theme,
        l.rows,
        &rows,
        editing(doc, &Field::Rows),
    );
    if let (Some((t, use_)), Some(g)) = (l.banner, doc.proposal()) {
        fill_rounded_rect(scene, t, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
        let msg = tr_with(
            "anim_editor.looks_like",
            &[("cols", &g.cols), ("rows", &g.rows)],
        );
        paint_text(
            text,
            scene,
            &msg,
            t.x + Radius::Sm.px(),
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
                    r.x + Radius::Sm.px(),
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
    paint_strip(doc, &l, &frames, info.as_ref(), scene, text, theme);
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
        Radius::Sm.px(),
        resolve(ColorToken::Bg1, theme),
    );
    let rows = info.map(|i| i.rows.as_slice()).unwrap_or_default();
    if rows.is_empty() {
        paint_text(
            text,
            scene,
            tr("anim_editor.no_animations"),
            l.list.x + Radius::Sm.px(),
            l.list.y,
            font,
            l.list.w,
            resolve(ColorToken::Text3, theme),
        );
    }
    for (i, (r, row)) in l.list_rows.iter().zip(rows).enumerate() {
        if i == doc.selected {
            fill_rounded_rect(scene, *r, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
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
            r.x + Radius::Sm.px(),
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
) {
    let font = TypeToken::Sm.px();
    fill_rounded_rect(
        scene,
        l.strip,
        Radius::Sm.px(),
        resolve(ColorToken::Bg1, theme),
    );
    if frames.is_empty() {
        paint_text(
            text,
            scene,
            tr("anim_editor.strip_hint"),
            l.strip.x + Radius::Sm.px(),
            l.strip.y,
            font,
            l.strip.w,
            resolve(ColorToken::Text3, theme),
        );
        return;
    }
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
            Radius::Sm.px(),
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
            thumb.x + Radius::Sm.px(),
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
        let shown = tr_with("anim_editor.ms", &[("ms", &doc.frame_ms_at(i))]);
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
}
