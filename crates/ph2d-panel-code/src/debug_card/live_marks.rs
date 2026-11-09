//! O acabamento do «Live values» (smoke do dono 09/11): larguras que só crescem enquanto o Live dura
//! (os números vivos mudam de largura e nada mexe), a dica de rato nas células Line (vai à linha) e
//! Value (copia) da Sheet, e a dica mín · máx · actual de um traço.

use super::CardPainted;
use ph2d_code_debugcard::LiveRow;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr_with;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::{Hit, Layout, Row, Span};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

/// As células da Sheet ao vivo que respondem ao clique: Line leva à linha, Value copia.
const CLICKABLE: [&str; 2] = ["line", "value"];

/// Uma célula da Sheet ao vivo: `(chave da linha, coluna)`.
pub type CellAt = (String, &'static str);

/// ⭐ As maiores larguras vistas desde que o Live começou: a Sheet por coluna, a Strip pela largura
/// do valor de cada linha. Só crescem; zeram ao sair do Live.
#[derive(Clone, Debug, Default)]
pub struct Steady {
    cols: Vec<f32>,
    strip: BTreeMap<String, f32>,
}

impl Steady {
    /// As larguras naturais de agora, nunca abaixo das maiores já vistas.
    pub fn cols(&mut self, natural: Vec<f32>) -> Vec<f32> {
        if self.cols.len() != natural.len() {
            self.cols = vec![0.0; natural.len()];
        }
        for (seen, now) in self.cols.iter_mut().zip(&natural) {
            *seen = seen.max(*now);
        }
        self.cols.clone()
    }

    /// A largura do valor da linha `key` na Strip, nunca abaixo da maior já vista.
    pub fn strip(&mut self, key: &str, now: f32) -> f32 {
        let seen = self.strip.entry(key.to_owned()).or_default();
        *seen = seen.max(now);
        *seen
    }
}

/// A célula Line ou Value da Sheet ao vivo debaixo de `(x, y)`.
pub fn cell_at(p: &CardPainted, x: f32, y: f32) -> Option<CellAt> {
    let (lay, rows, cols) = p.table.as_ref()?;
    if !cols.iter().any(|c| c.key == "line") {
        return None;
    }
    match lay.hit(rows, x, y)? {
        Hit::Cell { row, col } if CLICKABLE.contains(&col) => {
            rows.get(row).map(|r| (r.key.clone(), col))
        }
        _ => None,
    }
}

/// A moldura da célula debaixo do rato (`hover`) e da premida (`pressed`, mais forte), por cima do
/// texto sem o tapar.
pub fn paint_cells(
    lay: &Layout,
    rows: &[Row],
    hover: Option<&CellAt>,
    pressed: Option<&CellAt>,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = frame_radius(theme, Radius::Sm.px());
    let at = |c: &CellAt| {
        let i = rows.iter().position(|r| r.key == c.0)?;
        let r = lay.cell_rect(i, c.1)?;
        let d = Spacing::Xxs.px();
        Some(Rect::new(
            r.x + d,
            r.y + d,
            (r.w - d * 2.0).max(0.0),
            (r.h - d * 2.0).max(0.0),
        ))
    };
    if let Some(r) = pressed.and_then(at) {
        let ink = resolve(ColorToken::Accent, theme);
        stroke_rounded_rect(scene, r, radius, StrokeToken::Thick.px(), ink);
    } else if let Some(r) = hover.and_then(at) {
        let ink = resolve(ColorToken::BorderStrong, theme);
        stroke_rounded_rect(scene, r, radius, StrokeToken::Thin.px(), ink);
    }
}

/// Um número curto para o gráfico e a dica (até duas casas, sem zeros à direita).
#[must_use]
pub fn short_number(v: f32) -> String {
    let s = format!("{v:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

/// «min 0.12 · max 3.42 · now 3.42» de `samples`.
#[must_use]
pub fn span_text(samples: &[f32]) -> Option<String> {
    let s = Span::of(samples)?;
    Some(tr_with(
        "code.debug.span",
        &[
            ("min", &short_number(s.min)),
            ("max", &short_number(s.max)),
            ("now", &short_number(s.now)),
        ],
    ))
}

/// A dica sobre um traço: mín · máx · actual, por cima dele (por baixo se não cabe), dentro do quadro.
pub(super) fn tip(
    r: &LiveRow,
    at: Rect,
    body: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let Some(s) = r.trail.as_deref().and_then(span_text) else {
        return;
    };
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let font = TypeToken::Xs.px();
    let w = ts.layout(&s, font, f32::INFINITY).width() + pad * 2.0;
    let x = (at.x + at.w - w).clamp(body.x, (body.x + body.w - w).max(body.x));
    let y = if at.y - rh >= body.y {
        at.y - rh
    } else {
        at.y + at.h
    };
    let r = Rect::new(x, y, w, rh);
    let radius = frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg3, theme));
    stroke_rounded_rect(
        scene,
        r,
        radius,
        StrokeToken::Thin.px(),
        resolve(ColorToken::BorderStrong, theme),
    );
    let tone = resolve(ColorToken::Text1, theme);
    paint_text(ts, scene, &s, x + pad, y + (rh - font) * 0.5, font, w, tone);
}
