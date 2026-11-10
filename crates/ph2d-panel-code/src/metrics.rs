//! As medidas do editor, todas por tokens ou MEDIDAS na fonte: o tamanho é o [`CODE_TYPE`], a
//! altura da linha `LineHeight::Normal` no píxel, o avanço sai da mono moldada.

use crate::line_cache::LineCache;
use crate::style::Ink;
use parley::{FontData, Layout, PositionedLayoutItem};
use ph2d_editor_core::ui_scale::ao_pixel;
use ph2d_text::TextSystem;
use ph2d_tokens::{LineHeight, Spacing, StrokeToken, TypeToken};

const DIGITS: &str = "0123456789";

/// O corpo da mono do código: o do script, o do texto dos problemas, o de «alive» e de toda a
/// letra pintada pelas [`Metrics`] (dono 10/11: uma letra só).
pub const CODE_TYPE: TypeToken = TypeToken::Base;

/// O tamanho mostrado da mono do código (o [`CODE_TYPE`] com o tamanho da letra da interface).
#[must_use]
pub fn code_font_px() -> f32 {
    ph2d_text::displayed_font_px(CODE_TYPE.px())
}

/// Os dez dígitos da calha já moldados: os números pintam-se glifo a glifo, sem moldar nada.
pub(crate) struct DigitFont {
    pub font: FontData,
    pub size: f32,
    pub coords: Vec<i16>,
    pub glyphs: [u32; 10],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Metrics {
    pub font_px: f32,
    /// Altura de uma linha, no píxel.
    pub line_h: f32,
    /// O avanço de uma coluna na mono (medido).
    pub advance: f32,
    /// Do topo da linha ao topo da caixa do texto (centra a fonte na linha).
    pub text_dy: f32,
    /// Do topo da caixa do texto à linha de base.
    pub baseline: f32,
    pub digits: usize,
    /// Largura da coluna das marcas (problema, ponto de parada, seta).
    pub mark_w: f32,
    pub number_w: f32,
    /// Largura da coluna do marcador de dobra.
    pub fold_w: f32,
    pub gutter_w: f32,
    /// Folga entre a calha e o texto.
    pub text_pad: f32,
    pub status_h: f32,
    pub caret_w: f32,
}

impl Metrics {
    pub(crate) fn measure(
        cache: &mut LineCache,
        ts: &mut TextSystem,
        line_count: usize,
    ) -> (Self, Option<DigitFont>) {
        let font_px = code_font_px();
        let line_h = ao_pixel(font_px * LineHeight::Normal.ratio()).max(1.0);
        let layout = cache.shape_loose(ts, DIGITS, font_px);
        let advance = layout.width() / DIGITS.len() as f32;
        let (height, baseline) = layout.lines().next().map_or((font_px, font_px), |l| {
            (l.metrics().line_height, l.metrics().baseline)
        });
        let digits = count_digits(line_count).max(2);
        // Quadrada: o ponto de parada (≈ 60 % da linha) cabe com folga.
        let mark_w = line_h.max(Spacing::Md.px());
        let number_w = digits as f32 * advance;
        let fold_w = Spacing::Lg.px();
        let pad = Spacing::Sm.px();
        let gutter_w = ao_pixel(mark_w + number_w + pad + fold_w);
        let m = Self {
            font_px,
            line_h,
            advance,
            text_dy: ((line_h - height) * 0.5).max(0.0),
            baseline,
            digits,
            mark_w,
            number_w,
            fold_w,
            gutter_w,
            text_pad: Spacing::Xs.px(),
            status_h: ao_pixel(
                ph2d_text::displayed_font_px(TypeToken::Sm.px()) * LineHeight::Normal.ratio()
                    + Spacing::Sm.px(),
            ),
            caret_w: StrokeToken::Thick.px(),
        };
        (m, digit_font(&layout))
    }
}

fn digit_font(layout: &Layout<Ink>) -> Option<DigitFont> {
    let line = layout.lines().next()?;
    let PositionedLayoutItem::GlyphRun(gr) = line.items().next()? else {
        return None;
    };
    let run = gr.run();
    let ids: Vec<u32> = gr.glyphs().map(|g| g.id).collect();
    let glyphs: [u32; 10] = ids.try_into().ok()?;
    Some(DigitFont {
        font: run.font().clone(),
        size: run.font_size(),
        coords: ph2d_text::run_coord_bits(run),
        glyphs,
    })
}

pub(crate) fn count_digits(mut n: usize) -> usize {
    let mut d = 1;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

/// Do topo de `text` pintado a `px` à linha de base dele, medido no layout (a conta do
/// `debug_marks`).
pub(crate) fn baseline_px(ts: &mut TextSystem, text: &str, px: f32) -> f32 {
    let l = ts.layout(text, px, f32::INFINITY);
    l.lines().next().map_or(px, |l| l.metrics().baseline)
}

/// O topo de `small` (a `small_px`) posto na MESMA linha de base que `text` (a `px`) com topo em
/// `top` — a dica de tecla ao lado do rótulo de um botão (macOS, VS Code).
pub(crate) fn top_on_baseline(
    ts: &mut TextSystem,
    top: f32,
    (text, px): (&str, f32),
    (small, small_px): (&str, f32),
) -> f32 {
    top + baseline_px(ts, text, px) - baseline_px(ts, small, small_px)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O texto pequeno fica mais abaixo do topo do grande o quanto a base dele é mais curta; com o
    /// mesmo tamanho, o mesmo topo.
    #[test]
    fn a_tecla_pequena_desce_ate_a_linha_de_base_do_rotulo() {
        let mut ts = TextSystem::without_system_fonts();
        let (big, small) = (TypeToken::Sm.px(), TypeToken::Xs.px());
        let top = top_on_baseline(&mut ts, 10.0, ("Continue", big), ("F8", small));
        let (bb, bs) = (
            baseline_px(&mut ts, "Continue", big),
            baseline_px(&mut ts, "F8", small),
        );
        assert!(bb > bs, "{bb} {bs}");
        assert!((top + bs - (10.0 + bb)).abs() < 1e-4);
        let same = top_on_baseline(&mut ts, 10.0, ("F8", small), ("F8", small));
        assert!((same - 10.0).abs() < 1e-4);
    }
}
