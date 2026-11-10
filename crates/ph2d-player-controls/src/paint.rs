//! A pintura do painel «Controls» — PURA: o mesmo painel no Play e na prévia da aba Input Map.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_centered, paint_text_elided, paint_text_title,
    paint_text_title_elided, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

use crate::layout::Layout;
use crate::screen::{ControlsScreen, Row};

/// Pinta o painel em `area` (a vista do jogo). Devolve a disposição, para o clique usar a MESMA.
pub fn paint(
    area: Rect,
    rows: &[Row],
    screen: &ControlsScreen,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) -> Layout {
    let lay = Layout::scrolled(area, rows, screen.selected(), screen.top());
    let c = |t: ColorToken| resolve(t, theme);
    let line = StrokeToken::Hairline.px();
    fill_rounded_rect(scene, lay.panel, m::CARD_RADIUS, c(ColorToken::Bg1));
    stroke_rounded_rect(
        scene,
        lay.panel,
        m::CARD_RADIUS,
        line,
        c(ColorToken::Border),
    );
    // O título e os nomes em negrito, como no molde C·4.
    let title = tr("player_controls.title");
    let tw = text.prefix_width_weighted(title, m::CARD_TITLE_PX, FontWeight::SEMI_BOLD);
    paint_text_title(
        text,
        scene,
        title,
        lay.title.x + ((lay.title.w - tw) * 0.5).max(0.0),
        lay.title.y + (lay.title.h - m::CARD_TITLE_PX) * 0.5,
        m::CARD_TITLE_PX,
        lay.title.w,
        c(ColorToken::Text1),
    );
    for ((i, row), rl) in rows.iter().enumerate().skip(lay.first).zip(&lay.rows) {
        let selected = i == screen.selected();
        if selected {
            fill_rounded_rect(scene, rl.band, m::FIELD_RADIUS, c(ColorToken::Bg2));
        }
        let ty = rl.name.y + (rl.name.h - m::VALUE_PX) * 0.5;
        paint_text_title_elided(
            text,
            scene,
            &row.label,
            rl.name.x,
            ty,
            m::VALUE_PX,
            rl.name.w,
            c(ColorToken::Text1),
        );
        for (s, r) in rl.slots.iter().enumerate() {
            let here = selected && s == screen.slot();
            let key = row.keys.get(s).copied();
            paint_slot(scene, text, *r, key, here, screen.listening(), theme);
            if let (Some((slot, _)), Some(note)) = (&row.also, row.note())
                && *slot == s
            {
                stroke_rounded_rect(scene, *r, m::FIELD_RADIUS, line, c(ColorToken::Warn));
                let ny = rl.note.y + (rl.note.h - m::LABEL_PX) * 0.5;
                paint_text_elided(
                    text,
                    scene,
                    &note,
                    rl.note.x,
                    ny,
                    m::LABEL_PX,
                    rl.note.w,
                    c(ColorToken::Warn),
                );
            }
        }
    }
    if let Some((track, thumb)) = lay.bar {
        let r = track.w * 0.5;
        fill_rounded_rect(scene, track, r, c(ColorToken::Bg2));
        fill_rounded_rect(scene, thumb, r, c(ColorToken::Text3));
    }
    let on_reset = screen.selected() >= rows.len();
    if on_reset {
        fill_rounded_rect(scene, lay.reset, m::FIELD_RADIUS, c(ColorToken::Bg2));
    }
    paint_text_centered(
        text,
        scene,
        &tr("player_controls.reset"),
        lay.reset,
        m::VALUE_PX,
        c(if on_reset {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        }),
    );
    let hint = if rows.is_empty() {
        "player_controls.none"
    } else if screen.listening() {
        "player_controls.hint_listening"
    } else {
        "player_controls.hint_choose"
    };
    paint_text_centered(
        text,
        scene,
        &tr(hint),
        lay.hint,
        m::LABEL_PX,
        c(ColorToken::Text2),
    );
    lay
}

/// Uma caixa de tecla: o nome inteiro da tecla; à escuta, «Press a key» na cor de aviso do molde.
fn paint_slot(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    r: Rect,
    key: Option<ph2d_input::Key>,
    here: bool,
    listening: bool,
    theme: Theme,
) {
    let c = |t: ColorToken| resolve(t, theme);
    let radius = m::FIELD_RADIUS;
    if here && listening {
        fill_rounded_rect(scene, r, radius, c(ColorToken::Warn));
        paint_text_centered(
            text,
            scene,
            &tr("player_controls.listening"),
            r,
            m::VALUE_PX,
            c(ColorToken::Bg0),
        );
        return;
    }
    let (label, ink) = match key {
        Some(k) => (k.label(), ColorToken::Text1),
        None => (tr("player_controls.add").to_owned(), ColorToken::Text3),
    };
    fill_rounded_rect(scene, r, radius, c(ColorToken::Bg3));
    if here {
        stroke_rounded_rect(
            scene,
            r,
            radius,
            StrokeToken::Thick.px(),
            c(ColorToken::FocusRing),
        );
    }
    paint_text_centered(text, scene, &label, r, m::VALUE_PX, c(ink));
}
