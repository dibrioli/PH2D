//! ⭐ **O quadro «Live values»** (plano 14 Q3) nas três aparências: a Sheet é a tabela da casa
//! Line · Name · Value · Last 4 s; os Cards, um mostrador por variável (nome, valor, traço); a Strip,
//! uma célula por variável com o traço ao lado do valor. Pairar sobre um traço diz mín · máx ·
//! actual; clicar abre o gráfico grande (por baixo da linha na Sheet, no pé nas outras).

use super::live_marks::tip;
pub use super::live_marks::{short_number, span_text};
use super::{CardBox, DebugCardView};
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_code_debugcard::{CardStyle, DebugCard, LIVE_WINDOW_S, LiveRow, shown_value, value_ink};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text, resolve, stroke_polyline, stroke_rounded_rect,
    token_to_vello,
};
use ph2d_editor_core::widget::scroll_area::OwnBar;
use ph2d_editor_core::widget::{ScrollbarState, paint_scrollbar, scrollbar_track_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::{Cell, CellText, Column, Ink, Metrics, Row, Span, natural_widths};
use ph2d_vector::VectorScene;

/// As linhas da tabela que o gráfico grande toma por baixo da sua.
pub const GRAPH_ROWS: usize = 4;
/// A marca das linhas do gráfico grande (`chave#graph0..`).
const GRAPH_MARK: &str = "#graph";
/// As linhas de um mostrador dos Cards: o nome, o valor, o traço.
const GAUGE_ROWS: f32 = 3.0; // LITERAL-PX-OK: uma contagem de linhas, não píxeis
/// As larguras de traço que um mostrador tem no mínimo.
const GAUGE_SPARKS: f32 = 2.0; // LITERAL-PX-OK: uma contagem de traços, não píxeis

/// As quatro colunas do Live values (o traço esconde-se pelo botão direito no cabeçalho).
#[must_use]
pub fn live_columns() -> Vec<Column> {
    let last = tr_with("code.debug.col.last", &[("seconds", &LIVE_WINDOW_S)]);
    vec![
        Column::new("line", tr("code.debug.col.line")).mono(),
        Column::new("name", tr("code.debug.col.name")).mono().flex(),
        Column::new("value", tr("code.debug.col.value"))
            .mono()
            .end(),
        Column::new("trail", last).spark().hideable(),
    ]
}

/// ⭐ As linhas da tabela ao vivo de `c`; por baixo da `open`, as [`GRAPH_ROWS`] do gráfico grande.
#[must_use]
pub fn live_sheet_rows(c: &DebugCard, open: Option<&str>) -> Vec<Row> {
    let mut out = Vec::with_capacity(c.live.len() + GRAPH_ROWS);
    for r in &c.live {
        out.push(Row {
            key: r.var.key.clone(),
            cells: vec![
                Cell::new(r.line.to_string(), Ink::Muted),
                Cell::new(&r.var.name, Ink::Name),
                // Ao vivo quase tudo muda a cada amostra: o âmbar fica no valor; o antigo é o traço.
                Cell::new(shown_value(&r.var), value_ink(&r.var)),
                Cell {
                    spark: r.trail.clone(),
                    ..Cell::default()
                },
            ],
            ..Row::default()
        });
        if open == Some(r.var.key.as_str()) && r.trail.is_some() {
            out.extend((0..GRAPH_ROWS).map(|i| Row {
                key: format!("{}{GRAPH_MARK}{i}", r.var.key),
                ..Row::default()
            }));
        }
    }
    out
}

/// A chave dona de uma linha do gráfico grande (`None` = uma linha de valor).
#[must_use]
pub fn graph_owner(key: &str) -> Option<&str> {
    key.find(GRAPH_MARK).map(|i| &key[..i])
}

/// A altura do corpo ao vivo na aparência `style` (o gráfico grande incluído), numa aba de `area_w`.
pub(super) fn body_height(st: &DebugCardView, c: &DebugCard, area_w: f32) -> f32 {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let graph = if open_row(st, c).is_some() {
        rh * GRAPH_ROWS as f32
    } else {
        0.0
    };
    match st.style {
        CardStyle::Sheet => {
            let rows = live_sheet_rows(c, st.graph.as_deref());
            ph2d_ui_table::natural_height(&rows, &st.table, &Metrics::standard()) + rh
        }
        CardStyle::Cards => {
            let (per, pitch) = gauge_grid(area_w);
            let lines = c.live.len().div_ceil(per).max(1);
            lines as f32 * pitch + Spacing::Md.px() + graph
        }
        CardStyle::Strip => rh * 2.0 + pad + graph,
    }
}

/// A linha cujo gráfico grande está aberto (se ainda é um número).
fn open_row<'a>(st: &DebugCardView, c: &'a DebugCard) -> Option<&'a LiveRow> {
    let key = st.graph.as_deref()?;
    c.live
        .iter()
        .find(|r| r.var.key == key && r.trail.is_some())
}

/// Quantos mostradores cabem numa fila de `w` e o passo vertical de uma fila.
fn gauge_grid(w: f32) -> (usize, f32) {
    let gap = Spacing::Md.px();
    let least = Metrics::standard().spark_w * GAUGE_SPARKS + super::inset() * 2.0;
    let per = (((w - gap) / (least + gap)).floor() as usize).max(1);
    (per, ph2d_tokens::row_pitch_px() * GAUGE_ROWS + gap)
}

/// ⭐ O corpo ao vivo de `c` em `body`, na aparência escolhida.
pub(super) fn paint(
    st: &mut DebugCardView,
    c: &DebugCard,
    body: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    if open_row(st, c).is_none() {
        st.graph = None;
    }
    let hover = st.spark_hover.take();
    let rh = ph2d_tokens::row_pitch_px();
    // Nos Cards e na Strip o gráfico vai ao pé e fica com o que sobra de uma fila (o quadro no tecto
    // não o põe por cima dos mostradores); com menos de duas linhas não se pinta.
    let least = match st.style {
        CardStyle::Sheet => body.h,
        CardStyle::Cards => gauge_grid(body.w).1 + Spacing::Md.px(),
        CardStyle::Strip => rh * 2.0 + Spacing::Sm.px(),
    };
    let room = ph2d_editor_core::math::safe_clamp(body.h - least, 0.0, rh * GRAPH_ROWS as f32);
    let graph_h = if st.graph.is_some() && room >= rh * 2.0 {
        room
    } else {
        0.0
    };
    let main = Rect::new(body.x, body.y, body.w, (body.h - graph_h).max(0.0));
    match st.style {
        CardStyle::Sheet => sheet(st, c, main, view, scene, ts, theme),
        CardStyle::Cards => gauges(st, c, main, view, scene, ts, theme),
        CardStyle::Strip => strip(st, c, main, view, scene, ts, theme),
    }
    if graph_h > 0.0
        && let Some(r) = open_row(st, c)
    {
        let at = Rect::new(
            body.x + super::inset(),
            main.y + main.h,
            body.w - super::inset() * 2.0,
            graph_h,
        );
        st.painted.sparks.push((r.var.key.clone(), at));
        graph(r, at, scene, ts, theme);
    }
    if let Some(key) = hover
        && let Some(r) = c.live.iter().find(|r| r.var.key == key)
        && let Some(&(_, at)) = st.painted.sparks.iter().find(|(k, _)| *k == key)
    {
        tip(r, at, body, scene, ts, theme);
    }
}

/// A Sheet ao vivo: a tabela a toda a largura, o gráfico grande por baixo da linha aberta, o pé.
fn sheet(
    st: &mut DebugCardView,
    c: &DebugCard,
    body: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let (cols, m) = (live_columns(), Metrics::standard());
    let rows = live_sheet_rows(c, st.graph.as_deref());
    let mut text = CodeCells { view, theme };
    let natural = natural_widths(&cols, &rows, &m, ts, &mut text);
    let natural = st.steady.cols.widths(&cols, &natural);
    let table_r = Rect::new(body.x, body.y, body.w, (body.h - rh).max(0.0));
    reveal_graph(st, &rows, table_r.h - m.header_h, &m);
    let lay = ph2d_ui_table::layout(&cols, &rows, &natural, &mut st.table, table_r, &m);
    let tones = (crate::cards::card_tone(), crate::cards::ground_tone());
    ph2d_ui_table::paint(
        &lay, &cols, &rows, &st.table, tones, scene, ts, &mut text, theme,
    );
    let hover = st.cell_hover.take();
    let pressed = st.cell_pressed.as_ref();
    let at = super::live_marks::pair;
    let (hover, pressed) = (hover.as_ref().map(at), pressed.map(at));
    ph2d_ui_table::paint_cell_frames(&lay, &rows, hover, pressed, scene, theme);
    for span in &lay.rows {
        let row = &rows[span.index];
        if row.cells.get(3).is_some_and(|c| c.spark.is_some())
            && let Some(r) = lay.cell_rect(span.index, "trail")
        {
            st.painted.sparks.push((row.key.clone(), r));
        }
    }
    // O gráfico grande por cima das linhas dele (as que estão à vista).
    let mut at: Option<Rect> = None;
    for span in &lay.rows {
        if graph_owner(&rows[span.index].key).is_some() {
            let b = span.rect;
            at = Some(at.map_or(b, |a| Rect::new(a.x, a.y, a.w, b.y + b.h - a.y)));
        }
    }
    if let (Some(at), Some(r)) = (at, open_row(st, c)) {
        let inner = Rect::new(
            at.x + super::inset(),
            at.y,
            (at.w - super::inset() * 2.0).max(0.0),
            at.h,
        );
        st.painted.sparks.push((r.var.key.clone(), inner));
        graph(r, inner, scene, ts, theme);
    }
    foot(body, scene, ts, theme);
    st.painted.table = Some((lay, rows, cols));
}

/// Abrir o gráfico grande rola até as linhas dele caberem (uma vez por abertura).
fn reveal_graph(st: &mut DebugCardView, rows: &[Row], body_h: f32, m: &Metrics) {
    let Some(key) = st.graph.clone() else { return };
    if st.revealed.as_deref() == Some(key.as_str()) {
        return;
    }
    let shown = st.table.visible_rows(rows);
    let Some(last) = shown
        .iter()
        .rposition(|&i| graph_owner(&rows[i].key) == Some(&key))
    else {
        return;
    };
    let first = last.saturating_sub(GRAPH_ROWS);
    let fits = (body_h / m.row_h).floor().max(1.0) as usize;
    if (last + 1) as f32 * m.row_h > st.table.scroll + body_h {
        st.table.scroll = (last + 1).saturating_sub(fits).min(first) as f32 * m.row_h;
    }
    st.revealed = Some(key);
}

/// O pé: o que o rato faz nos traços.
fn foot(body: Rect, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
    let rh = ph2d_tokens::row_pitch_px();
    let fy = body.y + body.h - rh;
    let hair = Rect::new(body.x, fy, body.w, StrokeToken::Thin.px());
    fill_rounded_rect(scene, hair, 0.0, resolve(ColorToken::Border, theme));
    let font = TypeToken::Xs.px();
    let x = body.x + super::inset();
    let w = (body.w - super::inset() * 2.0).max(0.0);
    let tone = resolve(ColorToken::Text3, theme);
    let s = tr("code.debug.live_foot");
    paint_text(ts, scene, s, x, fy + (rh - font) * 0.5, font, w, tone);
}

/// Os Cards ao vivo: um mostrador por variável (nome e linha, o valor numa caixa, o traço), em
/// filas que rolam inteiras.
fn gauges(
    st: &mut DebugCardView,
    c: &DebugCard,
    body: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let gap = Spacing::Md.px();
    let (per, pitch) = gauge_grid(body.w);
    let lines = c.live.len().div_ceil(per);
    let grid = Rect::new(
        body.x,
        body.y + gap * 0.5,
        body.w,
        (body.h - gap * 0.5).max(0.0),
    );
    let fits = ((grid.h / pitch).floor() as usize).max(1);
    let over = lines.saturating_sub(fits);
    let max = over as f32 * pitch;
    st.cards_scroll[0] = st.cards_scroll[0].min(max);
    let first = ((st.cards_scroll[0] / pitch).round() as usize).min(over);
    let state = if st.bar_dragging() {
        ScrollbarState::Dragging
    } else {
        ScrollbarState::Normal
    };
    let shown_h = fits.min(lines) as f32 * pitch;
    let view_r = Rect::new(grid.x, grid.y, grid.w, shown_h);
    let content = lines as f32 * pitch;
    let y0 = first as f32 * pitch;
    let bar = paint_scrollbar(view_r, y0, content, view_r.h, (state, 0.0), scene, theme);
    let track = OwnBar {
        body: view_r,
        content_h: content,
        visible_h: view_r.h,
    };
    st.painted.cards.push(CardBox {
        rect: grid,
        max,
        bar,
        track,
    });
    let right = bar.map_or(grid.x + grid.w, |_| scrollbar_track_rect(view_r).x);
    let cw = ((right - grid.x - gap * (per as f32 + 1.0)) / per as f32).max(0.0);
    let ch = pitch - gap;
    for (k, r) in c.live.iter().enumerate().skip(first * per).take(fits * per) {
        let (row, col) = (k / per - first, k % per);
        let card = Rect::new(
            grid.x + gap + col as f32 * (cw + gap),
            grid.y + row as f32 * pitch,
            cw,
            ch,
        );
        gauge(st, r, card, view, scene, ts, theme);
    }
}

/// Um mostrador: o nome e `ln N` em cima, o valor numa caixa (âmbar se mudou), o traço em baixo.
fn gauge(
    st: &mut DebugCardView,
    r: &LiveRow,
    card: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let gap = ph2d_tokens::control_gap_px();
    let font = TypeToken::Xs.px();
    let ln = tr_with("code.debug.stack_line", &[("line", &r.line)]);
    super::boxes::card_head(card, &r.var.name, &ln, scene, ts, theme);
    let radius = frame_radius(theme, Radius::Sm.px());
    let field = Rect::new(
        card.x + pad,
        card.y + rh + gap * 0.5,
        (card.w - pad * 2.0).max(0.0),
        rh - gap,
    );
    fill_rounded_rect(
        scene,
        field,
        radius,
        resolve(crate::cards::card_tone(), theme),
    );
    if r.var.old.is_some() {
        let edge = token_to_vello(ph2d_tokens::changed_ink(theme));
        // FRAME-RAW-OK: o contorno de «mudou» É a mensagem (o valor acabou de mudar).
        stroke_rounded_rect(scene, field, radius, StrokeToken::Thin.px(), edge);
    } else {
        let edge = resolve(ColorToken::Border, theme);
        ph2d_editor_core::paint::stroke_frame(
            scene,
            field,
            radius,
            theme,
            ph2d_tokens::visuals::Feel::Rest,
            StrokeToken::Thin.px(),
            edge,
        );
    }
    let text = shown_value(&r.var);
    let right = field.x + field.w - pad;
    let mut cells = CodeCells { view, theme };
    let vx = (right - cells.width(ts, &text, true)).max(field.x + pad);
    let y = card.y + rh;
    cells.paint(
        ts,
        scene,
        &text,
        value_ink(&r.var),
        true,
        vx,
        (y, rh),
        right,
    );
    st.painted.values.push((r.var.value.clone(), field));
    let spark = Rect::new(field.x, y + rh, field.w, rh);
    match &r.trail {
        Some(s) => {
            ph2d_ui_table::paint_spark(scene, s, spark, &Metrics::standard(), theme);
            st.painted.sparks.push((r.var.key.clone(), spark));
        }
        None => {
            let k = ph2d_code_debugcard::kind_name(r.var.kind);
            let tone = resolve(ColorToken::Text3, theme);
            paint_text(
                ts,
                scene,
                k,
                spark.x,
                spark.y + (rh - font) * 0.5,
                font,
                spark.w,
                tone,
            );
        }
    }
}

/// A Strip ao vivo: uma célula por variável — o nome e a linha pequenos em cima, o valor e o traço
/// em baixo; as que cabem, a sobra repartida.
fn strip(
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
    let small = TypeToken::Xs.px();
    let spark_w = Metrics::standard().spark_w;
    let mut cells: Vec<(&LiveRow, String, String, f32, f32)> = Vec::new();
    let mut used = 0.0;
    for r in &c.live {
        let key = tr_with(
            "code.debug.live_name",
            &[("name", &r.var.name), ("line", &r.line)],
        );
        let value = shown_value(&r.var);
        let kw = ts.layout(&key, small, f32::INFINITY).width();
        // A largura do valor só cresce enquanto o Live dura: o traço e a célula seguinte não mexem.
        let vw = st.steady.strip(
            &r.var.key,
            CodeCells { view, theme }.width(ts, &value, true),
        );
        let spark = if r.trail.is_some() {
            pad + spark_w
        } else {
            0.0
        };
        let cw = (vw + spark).max(kw) + super::inset() * 2.0;
        if used + cw > body.w && !cells.is_empty() {
            break;
        }
        used += cw;
        cells.push((r, key, value, cw, vw));
    }
    let extra = (body.w - used).max(0.0) / cells.len().max(1) as f32;
    let mut x = body.x;
    for (r, key, value, cw, vw) in cells {
        let cell = Rect::new(x, body.y, (cw + extra).min(body.x + body.w - x), body.h);
        if x > body.x {
            let hair = Rect::new(x, body.y, StrokeToken::Thin.px(), body.h);
            fill_rounded_rect(scene, hair, 0.0, resolve(ColorToken::Border, theme));
        }
        let tone = if r.var.old.is_some() {
            token_to_vello(ph2d_tokens::changed_ink(theme))
        } else {
            resolve(ColorToken::Text3, theme)
        };
        let kx = x + pad * 2.0;
        let room = (cell.w - super::inset() - pad).max(0.0);
        paint_text(ts, scene, &key, kx, body.y + pad, small, room, tone);
        let right = cell.x + cell.w - pad;
        let row = (body.y + pad + small, rh);
        let mut text = CodeCells { view, theme };
        // Encostado à direita da largura estável: os dígitos (mono) ficam no mesmo sítio.
        let vx = kx + vw - text.width(ts, &value, true);
        text.paint(ts, scene, &value, value_ink(&r.var), true, vx, row, right);
        if let Some(s) = &r.trail {
            let sx = kx + vw + pad;
            let at = Rect::new(sx, row.0, (right - sx).max(0.0).min(spark_w), rh);
            ph2d_ui_table::paint_spark(scene, s, at, &Metrics::standard(), theme);
            st.painted.sparks.push((r.var.key.clone(), at));
        }
        st.painted.values.push((r.var.value.clone(), cell));
        x += cell.w;
    }
}

/// ⭐ O gráfico grande de `r` em `at`: o máximo e o mínimo à esquerda com as suas linhas, o traço
/// com a última amostra marcada, e o tempo por baixo (−4 s … now).
fn graph(r: &LiveRow, at: Rect, scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme) {
    let Some(samples) = r.trail.as_deref() else {
        return;
    };
    let Some(span) = Span::of(samples) else {
        return;
    };
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let font = TypeToken::Xs.px();
    let radius = frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        scene,
        at,
        radius,
        resolve(crate::cards::ground_tone(), theme),
    );
    ph2d_editor_core::paint::stroke_frame(
        scene,
        at,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let (hi, lo) = (short_number(span.max), short_number(span.min));
    let label_w = [&hi, &lo, &r.var.name]
        .iter()
        .map(|s| ts.layout(s, font, f32::INFINITY).width())
        .fold(0.0f32, f32::max)
        + pad * 2.0;
    let plot = Rect::new(
        at.x + label_w,
        at.y + pad,
        (at.w - label_w - pad).max(0.0),
        (at.h - rh - pad).max(0.0),
    );
    let tone = resolve(ColorToken::Text3, theme);
    let hair = StrokeToken::Thin.px();
    let grid = resolve(ColorToken::Border, theme);
    for y in [plot.y, plot.y + plot.h] {
        stroke_polyline(scene, &[(plot.x, y), (plot.x + plot.w, y)], hair, grid);
    }
    let lx = at.x + pad;
    paint_text(ts, scene, &hi, lx, plot.y - font * 0.5, font, label_w, tone);
    paint_text(
        ts,
        scene,
        &lo,
        lx,
        plot.y + plot.h - font * 0.5,
        font,
        label_w,
        tone,
    );
    let name = resolve(ColorToken::Text2, theme);
    let my = plot.y + (plot.h - font) * 0.5;
    paint_text(ts, scene, &r.var.name, lx, my, font, label_w, name);
    let m = Metrics {
        pad: 0.0,
        ..Metrics::standard()
    };
    ph2d_ui_table::paint_spark(scene, samples, plot, &m, theme);
    let ty = at.y + at.h - rh + (rh - font) * 0.5;
    let ago = tr_with("code.debug.ago", &[("seconds", &LIVE_WINDOW_S)]);
    paint_text(ts, scene, &ago, plot.x, ty, font, plot.w, tone);
    let now = tr("code.debug.now");
    let nw = ts.layout(now, font, f32::INFINITY).width();
    paint_text(
        ts,
        scene,
        now,
        plot.x + plot.w - nw,
        ty,
        font,
        nw + pad,
        tone,
    );
}
