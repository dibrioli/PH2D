//! A Sheet (padrão): a tabela da casa à esquerda; Call stack e Waiting here à direita; a frase do pé.

use super::DebugCardView;
use super::split::{self, Split};
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_code_debugcard::{DebugCard, columns, sheet_rows};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_in_weight, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::{Metrics, Row, natural_widths};
use ph2d_vector::VectorScene;

/// Um texto numa linha `rh` a começar em `(x, y)`: `Text1`/`Text2` no corpo, o resto (títulos,
/// notas, `ln N`) um degrau abaixo, como a maquete 14.
#[allow(clippy::too_many_arguments)]
pub(super) fn line(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    (x, y, w): (f32, f32, f32),
    tone: ColorToken,
    theme: Theme,
) {
    line_in(ts, scene, s, (x, y, w), (tone, crate::peso::CORPO), theme);
}

/// O [`line`] num peso dado: os títulos do lado (`h4`) são `500`, medidos no mesmo peso.
pub(super) fn line_in(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    (x, y, w): (f32, f32, f32),
    (tone, weight): (ColorToken, FontWeight),
    theme: Theme,
) {
    let font = match tone {
        ColorToken::Text1 | ColorToken::Text2 => TypeToken::Sm.px(),
        _ => TypeToken::Xs.px(),
    };
    let rh = ph2d_tokens::row_pitch_px();
    let at = [x, y + (rh - font) * 0.5];
    let ink = resolve(tone, theme);
    paint_text_in_weight(ts, scene, s, at, font, w.max(0.0), ink, weight);
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
    let (cols, rows, m) = (columns(), sheet_rows(c), crate::peso::metricas());
    let mut text = CodeCells { view, theme };
    let natural = natural_widths(&cols, &rows, &m, ts, &mut text);
    // A coluna do lado nunca esconde o título «Call stack»; a tabela nunca corta Value nem Type.
    let title_w = crate::peso::largura_em(
        ts,
        tr("code.debug.stack"),
        TypeToken::Xs.px(),
        crate::peso::BOTAO,
    );
    let table_min = ph2d_ui_table::min_width(&cols, &natural, &st.table, &m);
    let s = Split::measure(
        st.side_w,
        body.w,
        table_min,
        title_w + super::inset() * 2.0 + pad,
    );
    let tw = (body.w - s.side_w - split::band()).max(0.0);
    let table_r = Rect::new(body.x, body.y, tw, (body.h - rh).max(0.0));
    reveal_changed(st, c, &rows, table_r.h - m.header_h, &m);
    let lay = ph2d_ui_table::layout(&cols, &rows, &natural, &mut st.table, table_r, &m);
    let tones = (crate::cards::card_tone(), crate::cards::ground_tone());
    ph2d_ui_table::paint(
        &lay, &cols, &rows, &st.table, tones, scene, ts, &mut text, theme,
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
    split::paint(st, s, (sx, body.y, table_r.h), scene, theme);
    let side_x = sx + split::band();
    super::side::paint(
        st,
        c,
        Rect::new(side_x, body.y, s.side_w, table_r.h),
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
