//! **A lista das animações** do editor: nome, cor, quadros, o alerta dos vazios e a escolhida com moldura de acento.
//! Filho de [`super`] pelo tecto de linhas.

use super::*;

pub(super) fn paint_list(
    doc: &AnimDoc,
    l: &crate::doc::Layout,
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
    // A escolhida tem fundo e moldura de acento, também em alerta, cujo fundo tapa o dela (dono 08/11).
    let outline = |scene: &mut VectorScene, r: Rect| {
        let (rad, w) = (
            frame_radius(theme, m::FIELD_RADIUS),
            StrokeToken::Thick.px(),
        );
        stroke_rounded_rect(scene, r, rad, w, tom(ColorToken::Accent, theme));
    };
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
        if doc.sheet_ui.paint_alert_row(scene, text, theme, *r, i, row) {
            if i == doc.selected {
                outline(scene, *r);
            }
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
        if i == doc.selected {
            outline(scene, *r);
        }
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
