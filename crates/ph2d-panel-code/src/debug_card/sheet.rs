//! A Sheet (padrão): a tabela da casa à esquerda; Call stack e Waiting here à direita; a frase do pé.

use super::DebugCardView;
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_code_debugcard::{DebugCard, columns, sheet_rows};
use ph2d_editor_core::paint::{fill_rounded_rect, frame_radius, paint_text, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, PANEL_MIN_W_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::{Metrics, Row, natural_widths};
use ph2d_vector::VectorScene;

/// A parte da largura que a coluna do lado (Call stack, Waiting here) pode tomar.
const SIDE_SHARE: f32 = 0.4; // LITERAL-PX-OK: fração da largura do quadro, não píxeis

/// Um texto pequeno numa linha `rh` a começar em `(x, y)`.
#[allow(clippy::too_many_arguments)]
fn line(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    (x, y, w): (f32, f32, f32),
    tone: ColorToken,
    theme: Theme,
) {
    let font = TypeToken::Sm.px();
    let rh = ph2d_tokens::row_pitch_px();
    paint_text(
        ts,
        scene,
        s,
        x,
        y + (rh - font) * 0.5,
        font,
        w.max(0.0),
        resolve(tone, theme),
    );
}

/// ⭐ A Sheet de `c` em `body`.
pub(super) fn paint(
    st: &mut DebugCardView,
    c: &DebugCard,
    body: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let side_w = PANEL_MIN_W_PX.min(body.w * SIDE_SHARE);
    let table_r = Rect::new(
        body.x,
        body.y,
        (body.w - side_w).max(0.0),
        (body.h - rh).max(0.0),
    );
    let (cols, rows, m) = (columns(), sheet_rows(c), Metrics::standard());
    let mut text = CodeCells { view, theme };
    let natural = natural_widths(&cols, &rows, &m, ts, &mut text);
    reveal_changed(st, c, &rows, table_r.h - m.header_h, &m);
    let lay = ph2d_ui_table::layout(&cols, &rows, &natural, &mut st.table, table_r, &m);
    let base = crate::cards::card_tone();
    ph2d_ui_table::paint(
        &lay, &cols, &rows, &st.table, base, scene, ts, &mut text, theme,
    );
    if rows.is_empty() {
        let at = (table_r.x + pad * 2.0, lay.body.y, table_r.w);
        line(
            ts,
            scene,
            tr("code.debug.no_vars"),
            at,
            ColorToken::Text3,
            theme,
        );
    }
    let sx = table_r.x + table_r.w;
    let hair = StrokeToken::Thin.px();
    let border = resolve(ColorToken::Border, theme);
    fill_rounded_rect(scene, Rect::new(sx, body.y, hair, table_r.h), 0.0, border);
    side(
        st,
        c,
        Rect::new(sx, body.y, side_w, table_r.h),
        scene,
        ts,
        theme,
    );
    // O pé.
    let fy = body.y + body.h - rh;
    fill_rounded_rect(scene, Rect::new(body.x, fy, body.w, hair), 0.0, border);
    let at = (body.x + super::inset(), fy, body.w - super::inset() * 2.0);
    line(
        ts,
        scene,
        tr("code.debug.foot"),
        at,
        ColorToken::Text3,
        theme,
    );
    st.painted.table = Some((lay, rows, cols));
}

/// Um passo novo com uma linha âmbar fora de vista rola a tabela até ela (ao meio do corpo).
fn reveal_changed(st: &mut DebugCardView, c: &DebugCard, rows: &[Row], body_h: f32, m: &Metrics) {
    let shown = st.table.visible_rows(rows);
    let Some((order, key)) = shown
        .iter()
        .enumerate()
        .find(|(_, i)| rows[**i].changed)
        .map(|(o, &i)| (o, format!("{}:{}", c.line, rows[i].key)))
    else {
        return;
    };
    if st.revealed.as_deref() == Some(key.as_str()) {
        return;
    }
    let y = order as f32 * m.row_h;
    if y < st.table.scroll || y + m.row_h > st.table.scroll + body_h {
        st.table.scroll = (y - (body_h - m.row_h) * 0.5).max(0.0);
    }
    st.revealed = Some(key);
}

/// A coluna do lado: Call stack (a do topo acesa; clicar vai à linha) e Waiting here.
fn side(
    st: &mut DebugCardView,
    c: &DebugCard,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let x = r.x + super::inset();
    let w = (r.w - super::inset() - pad).max(0.0);
    let mut y = r.y + pad;
    line(
        ts,
        scene,
        tr("code.debug.stack"),
        (x, y, w),
        ColorToken::Text3,
        theme,
    );
    y += rh;
    for (i, s) in c.stack.iter().enumerate() {
        let rr = Rect::new(x - pad, y, w + pad, rh);
        if i == 0 {
            let radius = frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, rr, radius, resolve(ColorToken::AccentSoft, theme));
        }
        let name = if s.function.is_empty() {
            tr("code.debug.top_level").to_owned()
        } else {
            s.function.clone()
        };
        let ln = tr_with("code.debug.stack_line", &[("line", &s.line)]);
        let lw = ts.layout(&ln, font, f32::INFINITY).width();
        line(
            ts,
            scene,
            &name,
            (x, y, w - lw - pad),
            ColorToken::Text1,
            theme,
        );
        line(
            ts,
            scene,
            &ln,
            (x + w - lw - pad, y, lw + pad),
            ColorToken::Text3,
            theme,
        );
        st.painted.stack.push((s.line, rr));
        y += rh;
    }
    if c.waiting.is_empty() {
        return;
    }
    y += pad;
    let title = tr_with("code.debug.waiting_here", &[("count", &c.waiting.len())]);
    line(ts, scene, &title, (x, y, w), ColorToken::Text3, theme);
    y += rh;
    for who in &c.waiting {
        super::head::who_plain(who, Rect::new(x, y, w, rh), scene, ts, theme);
        y += rh;
    }
}
