//! **A pintura de uma aba do editor de animações** — lê a [`super::doc::Layout`] que o documento
//! acabou de calcular; nada aqui decide (os gestos estão no `doc`).

use super::bands::anim_color;
use super::doc::{AnimDoc, DragPaint, Field};
use super::paint_sheet::{paint_hover_list, paint_sheet};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, label_column_width, paint_text, paint_text_centered, resolve,
    stroke_rect,
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
    paint_text_centered(text, scene, label, r, TypeToken::Sm.px(), tom(fg, theme));
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
            paint_text_centered(
                text,
                scene,
                &s,
                r,
                TypeToken::Sm.px(),
                tom(ColorToken::Text1, theme),
            );
        }
        None => paint_text_centered(
            text,
            scene,
            shown,
            r,
            TypeToken::Sm.px(),
            tom(ColorToken::Text1, theme),
        ),
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
        frame_radius(theme, Radius::Sm.px()),
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
    let font = TypeToken::Sm.px();
    paint_text(text, scene, s, r.x, r.y, font, r.w, tom(c, theme));
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

    // ── A lista ──
    paint_list(doc, &l, info.as_ref(), scene, text, theme);

    // ── A folha ──
    let mut hint = tr("anim_editor.sheet_hint");
    if label_column_width(text, TypeToken::Sm.px(), [hint]) > l.sheet_title.w {
        hint = tr("anim_editor.sheet");
    }
    label(scene, text, theme, l.sheet_title, hint, ColorToken::Text2);
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
            tom(ColorToken::Bg2, theme),
        );
        let msg = tr_with(
            "anim_editor.looks_like",
            &[("cols", &g.cols), ("rows", &g.rows)],
        );
        let at = Rect::new(t.x + Spacing::Sm.px(), t.y, t.w, t.h);
        label(scene, text, theme, at, &msg, ColorToken::Text1);
        button(scene, text, theme, use_, tr("anim_editor.use_grid"), true);
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
    let Some(row) = info.and_then(|i| i.rows.get(doc.selected)) else {
        return;
    };
    let n = doc.shown_frames().len();
    let title = tr_with(
        "anim_editor.frames_title",
        &[("name", &row.name), ("n", &n)],
    );
    label(
        scene,
        text,
        theme,
        l.frames_title,
        &title,
        ColorToken::Text1,
    );
    let total = tr_with("anim_editor.total_ms", &[("ms", &doc.total_ms())]);
    let w = label_column_width(text, TypeToken::Sm.px(), [total.as_str()]);
    let r = l.frames_title;
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
    let fps_key = tr("panel.inspector.anim_card.fps");
    label(scene, text, theme, over(l.fps), fps_key, ColorToken::Text2);
    let fps = doc.fps_label();
    field(scene, text, theme, l.fps, &fps, editing(doc, &Field::Fps));
    let this = tr("anim_editor.this_frame");
    label(
        scene,
        text,
        theme,
        over(l.this_frame),
        this,
        ColorToken::Text2,
    );
    let shown = doc.this_frame_label();
    let open = doc
        .marked_frame()
        .and_then(|f| editing(doc, &Field::FrameMs(f)));
    field(scene, text, theme, l.this_frame, &shown, open);
    let marker = tr("anim_editor.marker");
    label(
        scene,
        text,
        theme,
        over(l.mark_bar),
        marker,
        ColorToken::Text2,
    );
    let mode = doc.mode();
    if let (Some(a), Some(b)) = (l.modes.first(), l.modes.last()) {
        let track = Rect::new(a.x, a.y, b.x + b.w - a.x, a.h);
        fill_rounded_rect(
            scene,
            track,
            frame_radius(theme, Radius::Sm.px()),
            tom(ColorToken::Bg2, theme),
        );
    }
    let keys = [
        "anim_editor.repeat",
        "anim_editor.once",
        "anim_editor.ping_pong",
    ];
    for (i, (r, key)) in l.modes.iter().zip(keys).enumerate() {
        button(
            scene,
            text,
            theme,
            *r,
            tr(key),
            mode == Some(AnimCycleMode::ALL[i]),
        );
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
    let font = TypeToken::Sm.px();
    let Some(frame) = doc.marked_frame() else {
        paint_text(
            text,
            scene,
            tr("anim_editor.marker_hint"),
            l.mark_bar.x,
            l.mark_bar.y,
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
    button(scene, text, theme, l.mark_toggle, rotulo, tem);
    let gravado = doc.marker_signal(frame);
    let shown = if gravado.is_empty() {
        tr("card.anim.marker_signal_placeholder")
    } else {
        gravado.as_str()
    };
    field(
        scene,
        text,
        theme,
        l.mark_signal,
        shown,
        editing(doc, &Field::Signal(frame)),
    );
    if let Some(step) = doc.marker_past_end(frame) {
        let x = l.mark_signal.x;
        let n = doc.authored_frames().len();
        paint_text(
            text,
            scene,
            &tr_with(
                "card.anim.marker_past_end",
                &[("frame", &(step + 1)), ("n", &n)],
            ),
            x,
            l.mark_bar.y - ROW_H_PX,
            font,
            (l.mark_bar.x + l.mark_bar.w - x).max(0.0),
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
    let font = TypeToken::Sm.px();
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
                frame_radius(theme, Radius::Sm.px()),
                tom(ColorToken::Bg3, theme),
            );
        }
        if let Some(buf) = editing(doc, &Field::Name(i)) {
            field(scene, text, theme, *r, "", Some(buf));
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
        // A amostra é a mesma cor das faixas da folha, e o número é quantos quadros a animação toca.
        let inset = Spacing::Sm.px();
        let side = Spacing::Md.px();
        let swatch = Rect::new(r.x + inset, r.y + (r.h - side) / 2.0, side, side);
        fill_rounded_rect(
            scene,
            swatch,
            frame_radius(theme, Radius::Sm.px()),
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
    button(scene, text, theme, l.add, tr("anim_editor.add"), false);
    if l.split.w > 0.0 {
        button(
            scene,
            text,
            theme,
            l.split,
            tr("anim_editor.split_by_rows"),
            false,
        );
    }
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
            frame_radius(theme, Radius::Sm.px()),
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
                frame_radius(theme, Radius::Sm.px()),
                tom(ColorToken::BgScrim, theme),
            );
        }
        if doc.frame_selected == Some(i) || (on_screen && step == Some(i)) {
            stroke_rect(
                scene,
                *thumb,
                StrokeToken::Thick.px(),
                tom(ColorToken::Accent, theme),
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
            tom(ColorToken::Text2, theme),
        );
        if frames.len() > 1 {
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
        // O tempo próprio só se mostra quando existe: este quadro dura outra coisa que os irmãos.
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
            frame_radius(theme, Radius::Sm.px()),
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
            frame_radius(theme, Radius::Sm.px()),
            Some(tom(ColorToken::Bg2, theme)),
            d.grab,
            d.at,
        );
        painted.ghost = Some(thumb);
    }
    painted
}
