//! A pintura da tabela sobre a [`crate::Layout`]: listras → estado (rato, mudou) → texto, depois o
//! cabeçalho e as divisórias, e a barra de rolagem quando o corpo transborda.

use crate::{Align, Column, Ink, Layout, Row, TableState};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_icon, paint_text, paint_text_title,
    rect_to_vello, resolve, stroke_polyline,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme, TypeToken};
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
            Ink::Number | Ink::Accent => ColorToken::Accent,
            Ink::Text => ColorToken::Success,
            Ink::Boolean => ColorToken::Info,
            Ink::Changed | Ink::Warn => ColorToken::Warn,
            Ink::Danger => ColorToken::Danger,
        }
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
        let font = TypeToken::Sm.px();
        paint_text(
            ts,
            scene,
            text,
            x,
            y + (h - font) * 0.5,
            font,
            (right - x).max(0.0),
            resolve(Self::token(ink), self.theme),
        );
    }
}

/// ⭐ Pinta a tabela. `base` = o tom da superfície por baixo (a listra move-o de leve).
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn paint(
    lay: &Layout,
    cols: &[Column],
    rows: &[Row],
    st: &TableState,
    base: ColorToken,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &mut dyn CellText,
    theme: Theme,
) {
    let Some(m) = lay.m else { return };
    let ink = |t: ColorToken| resolve(t, theme);
    let hair = StrokeToken::Thin.px();
    let font = TypeToken::Sm.px();
    let ty = |y: f32, h: f32| y + (h - font) * 0.5;

    // O corpo, recortado.
    scene.push_clip(&rect_to_vello(lay.body));
    for span in &lay.rows {
        let Some(row) = rows.get(span.index) else {
            continue;
        };
        let r = span.rect;
        ph2d_editor_core::widget::paint_row_stripe(scene, r, theme, base, span.order);
        if row.changed {
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
            let x = first + m.pad + indent;
            chev(scene, x);
            let tx = x + m.chevron();
            let title = row.cells.first().map_or("", |c| c.text.as_str());
            let tw = ts.layout(title, font, f32::INFINITY).width();
            let right = r.x + r.w - m.pad;
            paint_text_title(
                ts,
                scene,
                title,
                tx,
                ty(r.y, r.h),
                font,
                (right - tx).max(0.0),
                ink(ColorToken::Text1),
            );
            let cx = tx + tw + m.pad;
            paint_text(
                ts,
                scene,
                count,
                cx,
                ty(r.y, r.h),
                font,
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
                cell.ink,
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

    // O cabeçalho.
    let h = lay.header;
    scene.push_clip(&rect_to_vello(h));
    for c in &lay.cols {
        let col = &cols[c.index];
        let right = c.x + c.w - m.pad;
        let tw = ts.layout(&col.title, font, f32::INFINITY).width();
        let x = match col.align {
            Align::Start => c.x + m.pad,
            Align::End => (right - tw).max(c.x + m.pad),
        };
        paint_text(
            ts,
            scene,
            &col.title,
            x,
            ty(h.y, h.h),
            font,
            (c.x + c.w - x).max(0.0),
            ink(ColorToken::Text3),
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

    // A barra de rolagem (só indica; a roda e o arrasto do corpo rolam).
    if lay.max_scroll() > 0.0 && lay.content_h > 0.0 {
        let b = lay.body;
        let bar = m.handle;
        let th = (b.h * b.h / lay.content_h).max(m.row_h);
        let ty = b.y + (b.h - th) * (lay.scroll / lay.max_scroll());
        fill_rounded_rect(
            scene,
            Rect::new(b.x + b.w - bar * 2.0, ty, bar, th),
            frame_radius(theme, ph2d_tokens::Radius::Full.px().min(bar * 0.5)),
            ink(ColorToken::BorderStrong),
        );
    }
}

/// O fundo de uma linha que mudou: o âmbar do tema à opacidade do realce de linha (o `WarnSoft`
/// é uma superfície neutra nalguns temas — medido na foto da cena 14: cinzento).
#[must_use]
pub fn changed_tint(theme: Theme) -> ph2d_vector::Color {
    resolve(ColorToken::Warn, theme).with_alpha(ph2d_tokens::ROW_SELECTED_ALPHA)
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
    let c = resolve(ColorToken::Accent, theme);
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
