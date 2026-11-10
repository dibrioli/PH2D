//! A pintura da tabela sobre a [`crate::Layout`]: listras → estado (rato, mudou) → texto, depois o
//! cabeçalho e as divisórias, e a barra de rolagem quando o corpo transborda.

use crate::{Align, Cell, ColSpan, Column, Ink, Layout, Row, TableState};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_icon, paint_text, paint_text_title,
    rect_to_vello, resolve, stroke_polyline, token_to_vello,
};
use ph2d_editor_core::widget::{RowHighlight, paint_row_highlight};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O texto das células: o anfitrião mede e pinta (o editor de scripts pinta a mono com a cor do
/// realce do código; o resto da casa usa [`UiText`]).
pub trait CellText {
    fn width(&mut self, ts: &mut TextSystem, text: &str, mono: bool) -> f32;
    /// Pinta `text` a começar em `x`, centrado na linha `(y, h)`, cortado em `right`.
    #[allow(clippy::too_many_arguments)]
    fn paint(
        &mut self,
        ts: &mut TextSystem,
        scene: &mut VectorScene,
        text: &str,
        ink: Ink,
        mono: bool,
        x: f32,
        row: (f32, f32),
        right: f32,
    );
}

/// O texto da interface (sem mono própria): os tons dos tokens.
pub struct UiText {
    pub theme: Theme,
}

impl UiText {
    #[must_use]
    pub fn token(ink: Ink) -> ColorToken {
        match ink {
            Ink::Plain | Ink::Function => ColorToken::Text1,
            Ink::Muted | Ink::Nil => ColorToken::Text3,
            Ink::Name | Ink::Table => ColorToken::Text2,
            Ink::Number => ColorToken::Info,
            Ink::Accent => ColorToken::Accent,
            Ink::Text => ColorToken::Success,
            Ink::Boolean => ColorToken::Info,
            Ink::Changed | Ink::Warn => ColorToken::Warn,
            Ink::Danger => ColorToken::Danger,
        }
    }
}

/// A tinta de um tom: o «mudou» é o âmbar canónico ([`ph2d_tokens::changed_ink`]); o resto, o
/// token de [`UiText::token`].
#[must_use]
pub fn ink_color(ink: Ink, theme: Theme) -> ph2d_vector::Color {
    match ink {
        Ink::Changed => token_to_vello(ph2d_tokens::changed_ink(theme)),
        Ink::Number => token_to_vello(ph2d_tokens::value_ink(theme)),
        _ => resolve(UiText::token(ink), theme),
    }
}

/// O corpo do texto da interface numa célula: o secundário (tipo, notas) desce um degrau.
#[must_use]
pub fn ui_font(ink: Ink) -> f32 {
    match ink {
        Ink::Muted | Ink::Nil => TypeToken::Xs.px(),
        _ => TypeToken::Sm.px(),
    }
}

impl CellText for UiText {
    fn width(&mut self, ts: &mut TextSystem, text: &str, _mono: bool) -> f32 {
        ts.layout(text, TypeToken::Sm.px(), f32::INFINITY).width()
    }

    fn paint(
        &mut self,
        ts: &mut TextSystem,
        scene: &mut VectorScene,
        text: &str,
        ink: Ink,
        _mono: bool,
        x: f32,
        (y, h): (f32, f32),
        right: f32,
    ) {
        let font = ui_font(ink);
        // Centrado pela caixa que o texto OCUPA (a linha da letra), como a mono ao lado.
        let th = ts.layout(text, font, f32::INFINITY).height();
        paint_text(
            ts,
            scene,
            text,
            x,
            y + (h - th) * 0.5,
            font,
            (right - x).max(0.0),
            ink_color(ink, self.theme),
        );
    }
}

/// ⭐ Pinta a tabela. `base` = o tom da superfície por baixo (a listra move-o de leve); `head` = o
/// tom do cabeçalho e das linhas de grupo (um degrau abaixo, como a maquete 14).
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn paint(
    lay: &Layout,
    cols: &[Column],
    rows: &[Row],
    st: &TableState,
    (base, head): (ColorToken, ColorToken),
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &mut dyn CellText,
    theme: Theme,
) {
    let Some(m) = lay.m else { return };
    let ink = |t: ColorToken| resolve(t, theme);
    let hair = StrokeToken::Thin.px();
    let small = TypeToken::Xs.px();
    let ty_small = |y: f32, h: f32| y + (h - small) * 0.5;

    // O corpo, recortado.
    scene.push_clip(&rect_to_vello(lay.body));
    for span in &lay.rows {
        let Some(row) = rows.get(span.index) else {
            continue;
        };
        let r = span.rect;
        if row.group.is_some() {
            fill_rounded_rect(scene, r, 0.0, ink(head));
        } else {
            ph2d_editor_core::widget::paint_row_stripe(scene, r, theme, base, span.order);
        }
        // O fio de baixo de cada linha (a grelha da maquete: subtil, as colunas mais fortes).
        let floor = Rect::new(r.x, r.y + r.h - hair, r.w, hair);
        fill_rounded_rect(scene, floor, 0.0, ink(head));
        // A selecção vence o «mudou» e o hover (é um facto do documento, não do rato).
        if st.selected.as_deref() == Some(row.key.as_str()) {
            paint_row_highlight(scene, r, theme, RowHighlight::Selected, 0.0);
        } else if row.changed {
            fill_rounded_rect(scene, r, 0.0, changed_tint(theme));
        } else if st.hover.as_deref() == Some(row.key.as_str()) {
            fill_rounded_rect(scene, r, 0.0, ink(ColorToken::AccentSoft));
        }
        let first = lay.cols.first().map_or(r.x, |c| c.x);
        let indent = f32::from(row.depth) * m.indent;
        let chev = |scene: &mut VectorScene, x: f32| {
            let side = m.row_h * 0.5;
            let icon = if st.is_open(row) {
                IconId::ChevronDown
            } else {
                IconId::ChevronRight
            };
            paint_icon(
                scene,
                icon,
                Rect::new(x, r.y + (r.h - side) * 0.5, side, side),
                ink(ColorToken::Text3),
                StrokeToken::Default.px(),
            );
        };
        if let Some(count) = &row.group {
            // Um grupo: ▾ Título · contagem, a linha inteira.
            let x = first + m.pad * 0.5 + indent;
            chev(scene, x);
            let tx = x + m.chevron();
            let title = row.cells.first().map_or("", |c| c.text.as_str());
            let tw = ts.layout(title, small, f32::INFINITY).width();
            let right = r.x + r.w - m.pad;
            paint_text_title(
                ts,
                scene,
                title,
                tx,
                ty_small(r.y, r.h),
                small,
                (right - tx).max(0.0),
                ink(ColorToken::Text2),
            );
            let cx = tx + tw + m.pad * 0.5;
            paint_text(
                ts,
                scene,
                count,
                cx,
                ty_small(r.y, r.h),
                small,
                (right - cx).max(0.0),
                ink(ColorToken::Text3),
            );
            continue;
        }
        for (k, c) in lay.cols.iter().enumerate() {
            let Some(cell) = row.cells.get(c.index) else {
                continue;
            };
            let col = &cols[c.index];
            paint_cell_marks(scene, c, r, cell, st, theme);
            let mut x = c.x + m.pad;
            let right = c.x + c.w - m.pad;
            if k == 0 {
                x += indent;
                if row.expandable {
                    chev(scene, x);
                }
                x += m.chevron();
            }
            if let Some(dot) = cell.dot {
                let rad = m.row_h * 0.15;
                fill_circle(scene, x + rad, r.y + r.h * 0.5, rad, ink(dot));
                x += m.row_h * 0.5;
            }
            if let Some(sw) = cell.swatch {
                let side = crate::swatch_side();
                let sx = match col.align {
                    Align::Start => x,
                    Align::End => (right - side).max(x),
                };
                let q = Rect::new(sx, r.y + (r.h - side) * 0.5, side, side);
                fill_rounded_rect(scene, q, Spacing::Xxs.px(), ph2d_vector::Color::new(sw));
                continue;
            }
            if let Some(s) = &cell.spark {
                paint_spark(
                    scene,
                    s,
                    Rect::new(x, r.y, (right - x).max(0.0), r.h),
                    &m,
                    theme,
                );
                continue;
            }
            let w = text.width(ts, &cell.text, col.mono);
            let tx = match col.align {
                Align::Start => x,
                Align::End => (right - w).max(x),
            };
            if let Some(old) = &cell.old {
                // O antigo riscado, encostado à esquerda do novo.
                let ow = text.width(ts, old, col.mono);
                let ox = (tx - m.pad - ow).max(x);
                text.paint(ts, scene, old, Ink::Muted, col.mono, ox, (r.y, r.h), tx);
                let my = r.y + r.h * 0.5;
                stroke_polyline(
                    scene,
                    &[(ox, my), ((ox + ow).min(tx), my)],
                    hair,
                    ink(ColorToken::Text3),
                );
            }
            text.paint(
                ts,
                scene,
                &cell.text,
                if cell.changed { Ink::Changed } else { cell.ink },
                col.mono,
                tx,
                (r.y, r.h),
                // O corte é a borda da coluna: o texto que cabe à justa não perde a última letra
                // por arredondamento (o alinhamento continua dentro do recuo).
                c.x + c.w,
            );
        }
    }
    // As divisórias das colunas, pelo corpo.
    for c in lay.cols.iter().skip(1) {
        stroke_polyline(
            scene,
            &[(c.x, lay.body.y), (c.x, lay.body.y + lay.body.h)],
            hair,
            ink(ColorToken::Border),
        );
    }
    scene.pop_layer();

    paint_header(lay, cols, st, (base, head), scene, ts, theme);

    // A barra de rolagem da casa: agarra-se no polegar ou na trilha (`Hit::Bar`).
    let bar = lay.bar();
    let state = if st.bar_dragging() {
        ph2d_editor_core::widget::ScrollbarState::Dragging
    } else {
        ph2d_editor_core::widget::ScrollbarState::Normal
    };
    let (body, content, visible) = (bar.body, bar.content_h, bar.visible_h);
    let visual = (state, ph2d_editor_core::motion::SETTLED);
    let _ = ph2d_editor_core::widget::paint_scrollbar(
        body, lay.scroll, content, visible, visual, scene, theme,
    );
}

/// O cabeçalho: opaco e pintado DEPOIS do corpo (nada do corpo se lê por baixo dele).
#[allow(clippy::too_many_arguments)]
fn paint_header(
    lay: &Layout,
    cols: &[Column],
    st: &TableState,
    (base, head): (ColorToken, ColorToken),
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let Some(m) = lay.m else { return };
    let ink = |t: ColorToken| resolve(t, theme);
    let hair = StrokeToken::Thin.px();
    let small = TypeToken::Xs.px();
    let ty_small = |y: f32, h: f32| y + (h - small) * 0.5;
    let h = lay.header;
    // `Metrics.header_h = 0`: uma tabela sem cabeçalho (nem a linha de baixo dele).
    if h.h <= 0.0 {
        return;
    }
    scene.push_clip(&rect_to_vello(h));
    // Opaco e DEPOIS do corpo: nada do corpo se lê por baixo do cabeçalho.
    fill_rounded_rect(scene, h, 0.0, ink(base));
    fill_rounded_rect(scene, h, 0.0, ink(head));
    for c in &lay.cols {
        let col = &cols[c.index];
        let right = c.x + c.w - m.pad;
        let tw = ts.layout(&col.title, small, f32::INFINITY).width();
        let x = match col.align {
            Align::Start => c.x + m.pad,
            Align::End => (right - tw).max(c.x + m.pad),
        };
        paint_text(
            ts,
            scene,
            &col.title,
            x,
            ty_small(h.y, h.h),
            small,
            (c.x + c.w - x).max(0.0),
            ink(if st.lit_col.as_deref() == Some(&*c.key) {
                ColorToken::Text1
            } else {
                ColorToken::Text3
            }),
        );
        if c.x > lay.area.x {
            stroke_polyline(
                scene,
                &[(c.x, h.y), (c.x, h.y + h.h)],
                hair,
                ink(ColorToken::Border),
            );
        }
    }
    scene.pop_layer();
    stroke_polyline(
        scene,
        &[(h.x, h.y + h.h), (h.x + h.w, h.y + h.h)],
        hair,
        ink(ColorToken::Border),
    );
}

/// As marcas de uma célula, depois da listra e antes do texto: a banda da coluna acesa
/// (`AccentSoft`) ou da leve (o hover das listas), e a da célula mudada (âmbar, recuada).
fn paint_cell_marks(
    scene: &mut VectorScene,
    c: &ColSpan,
    r: Rect,
    cell: &Cell,
    st: &TableState,
    theme: Theme,
) {
    let band = Rect::new(c.x, r.y, c.w, r.h);
    if st.lit_col.as_deref() == Some(&*c.key) {
        fill_rounded_rect(scene, band, 0.0, resolve(ColorToken::AccentSoft, theme));
    } else if st.soft_col.as_deref() == Some(&*c.key) {
        paint_row_highlight(scene, band, theme, RowHighlight::Hovered, 0.0);
    }
    if cell.changed {
        let radius = frame_radius(theme, Radius::Sm.px());
        let at = crate::marks::inset(band, Spacing::Xxs.px());
        fill_rounded_rect(scene, at, radius, changed_tint(theme));
    }
}

/// O fundo de uma linha que mudou: a banda âmbar opaca ([`ph2d_tokens::changed_band`]; o `WarnSoft`
/// dos temas do Godot é uma superfície neutra — medido na foto da cena 14: cinzento).
#[must_use]
pub fn changed_tint(theme: Theme) -> ph2d_vector::Color {
    token_to_vello(ph2d_tokens::changed_band(theme))
}

/// O traço de `samples` em `r`: a escala é a do próprio traço (mín → fundo, máx → topo), o último
/// ponto marcado.
pub fn paint_spark(
    scene: &mut VectorScene,
    samples: &[f32],
    r: Rect,
    m: &crate::Metrics,
    theme: Theme,
) {
    let pts = spark_points(samples, r, m);
    let c = ink_color(Ink::Number, theme);
    if pts.len() >= 2 {
        stroke_polyline(scene, &pts, StrokeToken::Default.px(), c);
    }
    if let Some(&(x, y)) = pts.last() {
        fill_circle(scene, x, y, StrokeToken::Default.px() * 1.5, c);
    }
}

/// Os pontos do traço em `r` (recuado `m.pad / 2` em cima e em baixo); uma série plana fica a meio.
#[must_use]
pub fn spark_points(samples: &[f32], r: Rect, m: &crate::Metrics) -> Vec<(f32, f32)> {
    let finite: Vec<f32> = samples.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() || r.w <= 0.0 {
        return Vec::new();
    }
    let lo = finite.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = finite.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let inset = m.pad * 0.5;
    let top = r.y + inset;
    let span_h = (r.h - inset * 2.0).max(0.0);
    let n = finite.len();
    let step = if n > 1 { r.w / (n - 1) as f32 } else { 0.0 };
    finite
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let t = if hi > lo { (v - lo) / (hi - lo) } else { 0.5 };
            (r.x + i as f32 * step, top + span_h * (1.0 - t))
        })
        .collect()
}
