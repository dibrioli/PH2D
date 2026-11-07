//! **A pintura de uma aba do editor de animações** — lê a [`super::doc::Layout`] que o documento
//! acabou de calcular; nada aqui decide (os gestos estão no `doc`).

use super::bands::anim_color;
use super::doc::{AnimDoc, DragPaint, Field};
use super::paint_sheet::{paint_hover_list, paint_sheet};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, label_column_width, paint_text, paint_text_centered,
    paint_text_title, resolve, stroke_rect, stroke_rounded_rect,
};
use ph2d_editor_core::paint_shapes::fill_diamond;
use ph2d_editor_core::widget::paint_card_ghost;
use ph2d_editor_core::widget::sheet_cell::paint_sheet_cell;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimCycleMode, InspectorAnimInfo};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// A amostra de cor da lista, em relação à letra (a maquete, 9 de 11).
const SWATCH_OVER_FONT: f32 = 0.8; // LITERAL-PX-OK: razão adimensional
/// O corpo do texto do editor: o dos campos dos cartões.
const FONT_TOKEN: TypeToken = TypeToken::Base;

/// A cor do editor: em cartões, o tom da lista; `PH2D_CARDS=0`, o de antes.
pub fn tom(t: ColorToken, theme: Theme) -> Color {
    if !ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        return resolve(t, theme);
    }
    let cartao = match t {
        ColorToken::PanelBg => ColorToken::KanbanList,
        ColorToken::Bg1 => ColorToken::KanbanCard,
        ColorToken::Bg2 => ColorToken::CardField,
        ColorToken::Bg3 => ColorToken::KanbanCardRaised,
        ColorToken::Text1 => ColorToken::KanbanCardFg,
        ColorToken::Text2 | ColorToken::Text3 => ColorToken::CardLabel,
        o => return resolve(o, theme),
    };
    ph2d_editor_core::widget::card_ink::CardInk::of_list(theme)
        .color(cartao)
        .with_alpha(1.0)
}

pub(crate) fn button(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    label: &str,
    on: bool,
) {
    button_sized(scene, text, theme, r, label, on, TypeToken::Sm.px());
}

fn button_sized(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    label: &str,
    on: bool,
    font: f32,
) {
    // Em cartões o ligado é o segmento levantado, como «Keys» na Timeline; o acento fica para a seleção.
    let cartao = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    let bg = match (on, cartao) {
        (true, true) => ColorToken::Bg3,
        (true, false) => ColorToken::Accent,
        (false, _) => ColorToken::Bg2,
    };
    let fg = if on && !cartao {
        ColorToken::Bg0
    } else {
        ColorToken::Text1
    };
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, Radius::Sm.px()),
        tom(bg, theme),
    );
    paint_text_centered(text, scene, label, r, font, tom(fg, theme));
}

pub(crate) fn field(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    shown: &str,
    editing: Option<&str>,
) {
    field_sized(scene, text, theme, r, shown, editing, TypeToken::Sm.px());
}

fn field_sized(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    shown: &str,
    editing: Option<&str>,
    font: f32,
) {
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, m::FIELD_RADIUS),
        tom(ColorToken::Bg2, theme),
    );
    match editing {
        Some(buf) => {
            stroke_rect(
                scene,
                r,
                StrokeToken::Thin.px(),
                tom(ColorToken::Accent, theme),
            );
            let s = tr_with("anim_editor.caret", &[("text", &buf)]);
            paint_text_centered(text, scene, &s, r, font, tom(ColorToken::Text1, theme));
        }
        None => paint_text_centered(text, scene, shown, r, font, tom(ColorToken::Text1, theme)),
    }
}

fn editing<'a>(doc: &'a AnimDoc, f: &Field) -> Option<&'a str> {
    doc.editing
        .as_ref()
        .filter(|(g, _)| g == f)
        .map(|(_, s)| s.as_str())
}

/// Um cartão do editor: o fundo arredondado.
fn card(scene: &mut VectorScene, theme: Theme, r: Rect) {
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, Radius::Lg.px()),
        tom(ColorToken::Bg1, theme),
    );
}

/// Um título ou rótulo do cartão.
fn label(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    s: &str,
    c: ColorToken,
) {
    let font = FONT_TOKEN.px();
    let y = r.y + (r.h - ROW_H_PX).max(0.0) / 2.0;
    paint_text(text, scene, s, r.x, y, font, r.w, tom(c, theme));
}

/// Um campo dentro da moldura, um degrau acima dela: a letra apagada à esquerda e o valor junto.
fn glyph_field(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    r: Rect,
    glyph: &str,
    shown: &str,
    open: Option<&str>,
) {
    fill_rounded_rect(
        scene,
        r,
        frame_radius(theme, m::FIELD_RADIUS),
        tom(ColorToken::Bg3, theme),
    );
    if open.is_some() {
        stroke_rect(
            scene,
            r,
            StrokeToken::Thin.px(),
            tom(ColorToken::Accent, theme),
        );
    }
    let font = FONT_TOKEN.px();
    let x = r.x + Spacing::Md.px();
    let glyph_w = label_column_width(text, font, [glyph]);
    let apagado = tom(ColorToken::Text3, theme);
    let y = r.y + (r.h - ROW_H_PX) / 2.0;
    paint_text(text, scene, glyph, x, y, font, glyph_w, apagado);
    let value_x = x + glyph_w + Spacing::Md.px();
    let caret;
    let value = match open {
        Some(buf) => {
            caret = tr_with("anim_editor.caret", &[("text", &buf)]);
            caret.as_str()
        }
        None => shown,
    };
    let w = (r.x + r.w - value_x).max(0.0);
    let c = tom(ColorToken::Text1, theme);
    paint_text(text, scene, value, value_x, y, font, w, c);
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
    fill_rounded_rect(scene, area, 0.0, tom(ColorToken::PanelBg, theme));
    let info = doc.snap.info.clone();
    for r in [l.list, l.sheet_card, l.frames_card] {
        card(scene, theme, r);
    }
    paint_list(doc, &l, info.as_ref(), scene, text, theme);
    // ── A folha ──
    let mut hint = tr("anim_editor.sheet_hint");
    if label_column_width(text, FONT_TOKEN.px(), [hint]) > l.sheet_title.w {
        hint = tr("anim_editor.sheet");
    }
    label(scene, text, theme, l.sheet_title, hint, ColorToken::Text2);
    if let (Some((t, use_)), Some(g)) = (l.banner, doc.proposal()) {
        fill_rounded_rect(
            scene,
            t,
            frame_radius(theme, m::FIELD_RADIUS),
            tom(ColorToken::Bg2, theme),
        );
        let msg = tr_with(
            "anim_editor.looks_like",
            &[("cols", &g.cols), ("rows", &g.rows)],
        );
        let at = Rect::new(t.x + Spacing::Sm.px(), t.y, t.w, t.h);
        label(scene, text, theme, at, &msg, ColorToken::Text1);
        button_sized(
            scene,
            text,
            theme,
            use_,
            tr("anim_editor.use_grid"),
            true,
            FONT_TOKEN.px(),
        );
    }
    let users = doc.cell_users();
    doc.painted_cells = paint_sheet(doc, &l, &users, scene, text, theme);

    // ── A animação aberta ──
    paint_frames_title(doc, &l, info.as_ref(), scene, text, theme);
    let frames = doc.shown_frames();
    doc.painted = paint_strip(doc, &l, &frames, info.as_ref(), scene, text, theme);
    doc.painted_marks = (0..l.frames.len())
        .filter(|i| doc.marker_at(*i).is_some())
        .collect();
    paint_fields(doc, &l, scene, text, theme);
    paint_mark_bar(doc, &l, scene, text, theme);

    let names = doc.hovered_names(&users);
    paint_hover_list(doc, &l, &names, scene, text, theme);
}

/// «run · 8 frames», com a duração de um ciclo à direita.
fn paint_frames_title(
    doc: &AnimDoc,
    l: &super::doc::Layout,
    info: Option<&InspectorAnimInfo>,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    if info.and_then(|i| i.rows.get(doc.selected)).is_none() {
        return;
    }
    let font = FONT_TOKEN.px();
    let r = l.frames_title;
    let c = tom(ColorToken::Text1, theme);
    let title = doc.frames_title_text();
    let y = r.y + (r.h - ROW_H_PX) / 2.0;
    paint_text_title(text, scene, &title, r.x, y, font, r.w, c);
    let total = doc.total_text();
    let w = label_column_width(text, font, [total.as_str()]);
    let at = Rect::new(r.x + r.w - w, r.y, w, r.h);
    label(scene, text, theme, at, &total, ColorToken::Text2);
}

/// Frames/s e This frame, com o rótulo em cima, e o segmentado do ciclo.
fn paint_fields(
    doc: &AnimDoc,
    l: &super::doc::Layout,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let over = |r: Rect| Rect::new(r.x, r.y - ROW_H_PX, r.w, ROW_H_PX);
    // A moldura como a do cartão Image: um degrau abaixo do cartão.
    for g in [l.time_group, l.grid_group] {
        fill_rounded_rect(
            scene,
            g,
            frame_radius(theme, Radius::Sm.px()),
            tom(ColorToken::Bg2, theme),
        );
    }
    let fps_key = tr("panel.inspector.anim_card.fps");
    label(scene, text, theme, over(l.fps), fps_key, ColorToken::Text2);
    let fps = doc.fps_label();
    let glyph = tr("anim_editor.glyph_factor");
    let open = editing(doc, &Field::Fps);
    glyph_field(scene, text, theme, l.fps, glyph, &fps, open);
    let this = tr("anim_editor.this_frame");
    let this_label = over(l.this_frame);
    label(scene, text, theme, this_label, this, ColorToken::Text2);
    let shown = doc.this_frame_label();
    let open = doc
        .marked_frame()
        .and_then(|f| editing(doc, &Field::FrameMs(f)));
    glyph_field(scene, text, theme, l.this_frame, glyph, &shown, open);
    for (r, key, glyph, shown, f) in [
        (
            l.cols,
            "anim_editor.cols_title",
            "anim_editor.glyph_cols",
            doc.cols_label(),
            Field::Cols,
        ),
        (
            l.rows,
            "anim_editor.rows_title",
            "anim_editor.glyph_rows",
            doc.rows_label(),
            Field::Rows,
        ),
    ] {
        label(scene, text, theme, over(r), tr(key), ColorToken::Text2);
        glyph_field(scene, text, theme, r, tr(glyph), &shown, editing(doc, &f));
    }
    let mode = doc.mode();
    if let (Some(a), Some(b)) = (l.modes.first(), l.modes.last()) {
        let track = Rect::new(a.x, a.y, b.x + b.w - a.x, a.h);
        fill_rounded_rect(
            scene,
            track,
            frame_radius(theme, m::FIELD_RADIUS),
            tom(ColorToken::Bg2, theme),
        );
    }
    let keys = [
        "anim_editor.repeat",
        "anim_editor.once",
        "anim_editor.ping_pong",
    ];
    for (i, (r, key)) in l.modes.iter().zip(keys).enumerate() {
        if mode == Some(AnimCycleMode::ALL[i]) {
            let inset = Spacing::Xxs.px();
            let raised = Rect::new(
                r.x + inset,
                r.y + inset,
                r.w - 2.0 * inset, // LITERAL-PX-OK: as duas margens
                r.h - 2.0 * inset, // LITERAL-PX-OK: as duas margens
            );
            button_sized(scene, text, theme, raised, tr(key), true, FONT_TOKEN.px());
        } else {
            let c = tom(ColorToken::Text1, theme);
            paint_text_centered(text, scene, tr(key), *r, FONT_TOKEN.px(), c);
        }
    }
}

/// A barra do ◆: sem quadro escolhido, a dica; com ele, pôr ou tirar, o nome do sinal e o aviso de marcador mudo.
fn paint_mark_bar(
    doc: &AnimDoc,
    l: &super::doc::Layout,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let font = FONT_TOKEN.px();
    let Some(frame) = doc.marked_frame() else {
        let hint_w = label_column_width(text, font, [tr("anim_editor.marker_hint")]);
        paint_text(
            text,
            scene,
            tr("anim_editor.marker_hint"),
            (l.mark_bar.x + l.mark_bar.w - hint_w).max(l.mark_bar.x),
            l.mark_bar.y + (l.mark_bar.h - ROW_H_PX) / 2.0,
            font,
            l.mark_bar.w,
            tom(ColorToken::Text3, theme),
        );
        return;
    };
    let tem = doc.marker_at(frame).is_some();
    let rotulo = tr(if tem {
        "anim_editor.marker_remove"
    } else {
        "anim_editor.marker_add"
    });
    button_sized(
        scene,
        text,
        theme,
        l.mark_toggle,
        rotulo,
        tem,
        FONT_TOKEN.px(),
    );
    let gravado = doc.marker_signal(frame);
    let shown = if gravado.is_empty() {
        tr("card.anim.marker_signal_placeholder")
    } else {
        gravado.as_str()
    };
    field_sized(
        scene,
        text,
        theme,
        l.mark_signal,
        shown,
        editing(doc, &Field::Signal(frame)),
        FONT_TOKEN.px(),
    );
    if let Some(step) = doc.marker_past_end(frame) {
        let x = l.this_frame.x + l.this_frame.w / 2.0;
        let n = doc.authored_frames().len();
        paint_text(
            text,
            scene,
            &tr_with(
                "card.anim.marker_past_end",
                &[("frame", &(step + 1)), ("n", &n)],
            ),
            x,
            l.this_frame.y - ROW_H_PX,
            font,
            (l.this_frame.x + l.this_frame.w - x).max(0.0),
            tom(ColorToken::Warn, theme),
        );
    }
}

fn paint_list(
    doc: &AnimDoc,
    l: &super::doc::Layout,
    info: Option<&InspectorAnimInfo>,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let font = FONT_TOKEN.px();
    let title = tr("anim_editor.animations");
    label(scene, text, theme, l.list_title, title, ColorToken::Text2);
    let rows = info.map(|i| i.rows.as_slice()).unwrap_or_default();
    if rows.is_empty() {
        paint_text(
            text,
            scene,
            tr("anim_editor.no_animations"),
            l.list_title.x,
            l.list_title.y + l.list_title.h,
            font,
            l.list_title.w,
            tom(ColorToken::Text3, theme),
        );
    }
    for (i, (r, row)) in l.list_rows.iter().zip(rows).enumerate() {
        if i == doc.selected {
            fill_rounded_rect(
                scene,
                *r,
                frame_radius(theme, m::FIELD_RADIUS),
                tom(ColorToken::Bg3, theme),
            );
        }
        if let Some(buf) = editing(doc, &Field::Name(i)) {
            field_sized(scene, text, theme, *r, "", Some(buf), FONT_TOKEN.px());
            continue;
        }
        let playing = info.is_some_and(|x| x.playing && x.current == row.name);
        let name = if playing {
            tr_with("anim_editor.row_playing", &[("name", &row.name)])
        } else {
            row.name.clone()
        };
        let color = if playing {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        let inset = Spacing::Sm.px();
        let side = font * SWATCH_OVER_FONT;
        let swatch = Rect::new(r.x + inset, r.y + (r.h - side) / 2.0, side, side);
        fill_rounded_rect(
            scene,
            swatch,
            Spacing::Xxs.px(),
            tom(anim_color(row.color), theme),
        );
        let count = row.steps.to_string();
        let count_w = label_column_width(text, font, [count.as_str()]);
        let name_x = swatch.x + side + inset;
        paint_text(
            text,
            scene,
            &name,
            name_x,
            r.y,
            font,
            (r.x + r.w - count_w - inset - name_x).max(0.0),
            tom(color, theme),
        );
        paint_text(
            text,
            scene,
            &count,
            r.x + r.w - count_w - inset,
            r.y,
            font,
            count_w,
            tom(ColorToken::Text2, theme),
        );
    }
    for (r, key, visible) in [
        (l.add, "anim_editor.add", true),
        (l.split, "anim_editor.split_by_rows", l.split.w > 0.0),
        (l.remove, "anim_editor.remove", !rows.is_empty()),
    ] {
        if visible {
            button_sized(scene, text, theme, r, tr(key), true, font);
        }
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
    let font = FONT_TOKEN.px();
    if frames.is_empty() {
        paint_text(
            text,
            scene,
            tr("anim_editor.strip_hint"),
            l.strip.x + Spacing::Sm.px(),
            l.strip.y,
            font,
            l.strip.w,
            tom(ColorToken::Text3, theme),
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
    for (i, ((thumb, close), cell)) in l.frames.iter().zip(frames).enumerate() {
        fill_rounded_rect(
            scene,
            *thumb,
            frame_radius(theme, m::FIELD_RADIUS),
            tom(ColorToken::Bg2, theme),
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
                frame_radius(theme, m::FIELD_RADIUS),
                tom(ColorToken::BgScrim, theme),
            );
        }
        if doc.frame_selected == Some(i) || (on_screen && step == Some(i)) {
            let radius = frame_radius(theme, m::FIELD_RADIUS);
            let c = tom(ColorToken::Accent, theme);
            stroke_rounded_rect(scene, *thumb, radius, StrokeToken::Thick.px(), c);
        }
        let n = (i + 1).to_string();
        let c = tom(ColorToken::Text2, theme);
        paint_text(
            text,
            scene,
            &n,
            thumb.x + Spacing::Sm.px(),
            thumb.y,
            font,
            thumb.w,
            c,
        );
        let hovered = doc.hover.is_some_and(|(x, y)| {
            x >= thumb.x && x < thumb.x + thumb.w && y >= thumb.y && y < thumb.y + thumb.h
        });
        if frames.len() > 1 && hovered {
            paint_text_centered(
                text,
                scene,
                tr("anim_editor.remove_frame"),
                *close,
                font,
                tom(ColorToken::Text2, theme),
            );
        }
        if doc.marker_at(i).is_some() {
            let cor = if doc.marker_past_end(i).is_some() {
                ColorToken::Warn
            } else {
                ColorToken::Accent
            };
            let r = font * 0.5; // LITERAL-PX-OK: o ◆ tem a altura da letra
            fill_diamond(
                scene,
                thumb.x + Spacing::Sm.px() + r,
                thumb.y + thumb.h - Spacing::Sm.px() - r,
                r,
                tom(cor, theme),
            );
        }
        if (doc.frame_factor_at(i) - 1.0).abs() > f32::EPSILON {
            let badge = Rect::new(thumb.x, thumb.y + thumb.h - ROW_H_PX, thumb.w, ROW_H_PX);
            paint_text_centered(
                text,
                scene,
                &doc.ms_label(i),
                badge,
                font,
                tom(ColorToken::Text1, theme),
            );
        }
    }
    scene.pop_layer();
    let mut painted = DragPaint::default();
    if let Some((gap, bar)) = l.marker {
        fill_rounded_rect(
            scene,
            bar,
            frame_radius(theme, m::FIELD_RADIUS),
            tom(ColorToken::Accent, theme),
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
            tom(ColorToken::Accent, theme),
        );
        paint_card_ghost(
            scene,
            &content,
            thumb,
            frame_radius(theme, m::FIELD_RADIUS),
            Some(tom(ColorToken::Bg2, theme)),
            d.grab,
            d.at,
        );
        painted.ghost = Some(thumb);
    }
    painted
}
