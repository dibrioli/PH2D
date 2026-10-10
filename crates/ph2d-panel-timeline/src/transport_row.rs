//! A fileira do transporte `|< < >/|| > >|` — irmã do `transport.rs`, que estava no tecto de 600 LOC.
//! É o transporte de EDIÇÃO (toca o que a aba mostra); a cena a CORRER é o Play da barra superior
//! (onda 4, item 1).

use super::*;

/// A largura da fileira: os cinco botões num corpo, com um fio entre peças.
pub(super) fn width() -> f32 {
    let n = TRANSPORT_BTNS as f32;
    BTN_W * n + ph2d_editor_core::widget::SEGMENT_HAIRLINE * (n - 1.0)
}

/// `|< < >/|| > >|` — jump to start, step back, play/pause, step forward, jump to end, as
/// ONE body (report do dono de 2026-09-30). Saiu do [`paint_item`] pelo tecto de 200 LOC por
/// função: é um assunto só, a fileira de transporte.
pub(super) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    snap: &TimelineViewSnapshot,
    view: BarView,
) {
    // |< < >/|| > >| — jump to start, step back, play/pause, step forward,
    // jump to end. The skip glyphs bracket the frame-steppers, as every
    // transport does.
    // ⭐ UM corpo (report do dono de 2026-09-30): as cinco peças ENCOSTAM a um fio e só
    //    as pontas de fora arredondam — o que separa duas peças do mesmo controlo é a
    //    quina, não um vão. `width` mede com o MESMO fio.
    let fio = ph2d_editor_core::widget::SEGMENT_HAIRLINE;
    let pos = |i: usize| ph2d_editor_core::widget::GroupCell {
        col: ph2d_editor_core::widget::GroupPos::of(i, TRANSPORT_BTNS),
        row: ph2d_editor_core::widget::GroupPos::Only,
    };
    let mut x = widgets::icon_button_in(
        ctx,
        theme,
        x,
        y,
        TIMELINE_GO_START,
        IconId::SkipBack,
        pos(0),
    ) + fio;
    x = widgets::icon_button_in(
        ctx,
        theme,
        x,
        y,
        TIMELINE_PREV_FRAME,
        IconId::ChevronLeft,
        pos(1),
    ) + fio;
    let play_glyph = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    // **The Containers LIST has no playback mode** (Enio, 2026-07-22): the
    // list is a library of assets, not a view of time, so play/pause paints
    // DEAD there — disabled AND unhittable, because a dimmed control that
    // still dispatches lies ([[feedback_widget_is_done_when_a_test_clicks_it]]).
    // It comes back inside a container, on Keys and on Arrange, where the
    // same door (`tab::rows`) stops answering `Containers`.
    x = if crate::tab::rows(view.tab, snap) == crate::tab::Rows::Containers {
        widgets::dead_icon_button(ctx, theme, x, y, play_glyph, pos(2))
    } else {
        widgets::icon_button_in(ctx, theme, x, y, TIMELINE_PLAY, play_glyph, pos(2))
    } + fio;
    x = widgets::icon_button_in(
        ctx,
        theme,
        x,
        y,
        TIMELINE_NEXT_FRAME,
        IconId::ChevronRight,
        pos(3),
    ) + fio;
    widgets::icon_button_in(
        ctx,
        theme,
        x,
        y,
        TIMELINE_GO_END,
        IconId::SkipForward,
        pos(4),
    );
}
