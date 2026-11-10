//! A pintura do painel «Controls» — PURA: o mesmo painel no Play e na prévia da aba Input Map.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, paint_text_centered, paint_text_elided, paint_text_title,
    paint_text_title_elided, resolve, stroke_rounded_rect, token_to_vello,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, VectorScene};

use crate::layout::{FOOT_PX, Layout, TITLE_PX, foot_lines, slot_label};
use crate::screen::{ControlsScreen, Row};

/// A faixa da linha escolhida, o fundo do botão «Reset to defaults» (o das caixas de tecla) e a dica (texto
/// secundário).
pub const ROW_INK: ColorToken = ColorToken::AccentSoft;
pub const RESET_INK: ColorToken = ColorToken::Bg3;
pub const HINT_INK: ColorToken = ColorToken::Text2;

/// Pinta o painel em `area` (a vista do jogo). Devolve a disposição, para o clique usar a MESMA.
pub fn paint(
    area: Rect,
    rows: &[Row],
    screen: &ControlsScreen,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) -> Layout {
    let lay = Layout::scrolled(area, rows, screen, text);
    paint_with(&lay, rows, screen, scene, text, theme, false);
    lay
}

/// **A prévia** (aba Input Map, vista C): o painel do jogador INTEIRO — no tamanho natural, com todas as linhas, o
/// «Reset to defaults» e a dica, sem indicador de rolagem — ESCALADO para caber em `area` (nunca aumentado) e
/// centrado nela. Sem linha escolhida e sem anel de foco: ninguém está a jogar, por isso nada acende. Devolve onde
/// o painel escalado assentou.
pub fn paint_preview(
    area: Rect,
    rows: &[Row],
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) -> Rect {
    let mut local = VectorScene::new();
    let lay = paint_preview_natural(rows, &mut local, text, theme);
    let fit = Rect::new(
        area.x + m::CARD_PAD_X,
        area.y + m::CARD_PAD_X,
        (area.w - m::CARD_PAD_X * 2.0).max(0.0),
        (area.h - m::CARD_PAD_X * 2.0).max(0.0),
    );
    let k = (fit.w / lay.panel.w).min(fit.h / lay.panel.h).min(1.0);
    let (w, h) = (lay.panel.w * k, lay.panel.h * k);
    let at = Rect::new(
        area.x + (area.w - w) * 0.5,
        area.y + (area.h - h) * 0.5,
        w,
        h,
    );
    let t = Affine::translate((f64::from(at.x), f64::from(at.y))) * Affine::scale(f64::from(k));
    scene.append_transformed(&local, t);
    at
}

/// O painel da prévia no tamanho natural, com o canto em `(0, 0)` — o que [`paint_preview`] escala.
pub(crate) fn paint_preview_natural(
    rows: &[Row],
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) -> Layout {
    let lay = Layout::natural(0.0, 0.0, rows, text);
    let screen = ControlsScreen::default();
    paint_with(&lay, rows, &screen, scene, text, theme, true);
    lay
}

pub(crate) fn paint_with(
    lay: &Layout,
    rows: &[Row],
    screen: &ControlsScreen,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    preview: bool,
) {
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
    // O título e os nomes em negrito, como no molde C·4; o título um degrau acima.
    let title = tr("player_controls.title");
    let tw = text.prefix_width_weighted(title, TITLE_PX, FontWeight::SEMI_BOLD);
    paint_text_title(
        text,
        scene,
        title,
        lay.title.x + ((lay.title.w - tw) * 0.5).max(0.0),
        lay.title.y + (lay.title.h - TITLE_PX) * 0.5,
        TITLE_PX,
        lay.title.w,
        c(ColorToken::Text1),
    );
    for ((i, row), rl) in rows.iter().enumerate().skip(lay.first).zip(&lay.rows) {
        let selected = !preview && i == screen.selected();
        // A linha escolhida acende no acento suave, também à escuta (a caixa amarela diz qual das teclas).
        if selected {
            fill_rounded_rect(scene, rl.band, m::FIELD_RADIUS, c(ROW_INK));
        }
        let ty = rl.name.y + folga_do_texto(rl.name.h, m::VALUE_PX);
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
            let listening = here && screen.listening();
            let label = slot_label(row, s, listening);
            let empty = row.keys.get(s).is_none();
            paint_slot(scene, text, *r, &label, (here, listening, empty), theme);
            if let (Some((slot, _)), Some(note)) = (&row.also, row.note())
                && *slot == s
            {
                stroke_rounded_rect(scene, *r, m::FIELD_RADIUS, line, c(ColorToken::Warn));
                let ny = rl.note.y + folga_do_texto(rl.note.h, m::LABEL_PX);
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
    // O pé: um fio, e por baixo o BOTÃO «Reset to defaults» (a caixa das teclas, o anel de foco quando escolhido) e a
    // dica no texto secundário.
    fill_rounded_rect(scene, lay.rule, 0.0, c(ColorToken::Border));
    let on_reset = !preview && screen.selected() >= rows.len();
    fill_rounded_rect(scene, lay.reset, m::FIELD_RADIUS, c(RESET_INK));
    if on_reset {
        stroke_rounded_rect(
            scene,
            lay.reset,
            m::FIELD_RADIUS,
            StrokeToken::Default.px(),
            c(ColorToken::FocusRing),
        );
    }
    paint_text_centered(
        text,
        scene,
        tr("player_controls.reset"),
        lay.reset,
        FOOT_PX,
        c(ColorToken::Text1),
    );
    let hint = crate::layout::hint_key(rows, screen);
    let line_h = lay.reset.h;
    for (i, l) in foot_lines(text, tr(hint), lay.hint.w).iter().enumerate() {
        let r = Rect::new(
            lay.hint.x,
            lay.hint.y + line_h * i as f32,
            lay.hint.w,
            line_h,
        );
        paint_text_centered(text, scene, l, r, FOOT_PX, c(HINT_INK));
    }
}

/// Uma caixa de tecla do tamanho do nome; à escuta, «Press a key» no amarelo do Input Map com a tinta escura.
fn paint_slot(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    r: Rect,
    label: &str,
    (here, listening, empty): (bool, bool, bool),
    theme: Theme,
) {
    let c = |t: ColorToken| resolve(t, theme);
    let radius = m::FIELD_RADIUS;
    if listening {
        let under = ColorToken::InputMapHighlight.resolve(theme);
        let ink = token_to_vello(ph2d_tokens::skin_body::readable_on(under, theme));
        fill_rounded_rect(scene, r, radius, c(ColorToken::InputMapHighlight));
        paint_text_centered(text, scene, label, r, m::VALUE_PX, ink);
        return;
    }
    fill_rounded_rect(scene, r, radius, c(ColorToken::Bg3));
    if here {
        // O anel de foco no MESMO traço do da aba (`interact::paint_focus_ring`): um anel, uma espessura.
        stroke_rounded_rect(
            scene,
            r,
            radius,
            StrokeToken::Default.px(),
            c(ColorToken::FocusRing),
        );
    }
    let ink = if empty {
        ColorToken::Text3
    } else {
        ColorToken::Text1
    };
    paint_text_centered(text, scene, label, r, m::VALUE_PX, c(ink));
}
