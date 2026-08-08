//! Os parâmetros de **TEXTO** do painel Vector — irmão de `params.rs` pelo teto de 700 LOC
//! daquele arquivo. Tamanho, peso, entrelinha e tracking: cada um com a sua faixa e o par
//! de conversões que o slider e o chip compartilham.

/// Text glyph size in WORLD units — the Text-mode Size slider's range. The editor view is
/// ~10 world-units tall, so 1.0 reads comfortably. Shared by the panel (seed + chip mapping)
/// and the shell drain (track → size), like the Width family.
pub const TEXT_SIZE_MIN: f64 = 0.05;
pub const TEXT_SIZE_MAX: f64 = 8.0;
/// Default glyph size for a new text session (world units).
pub const DEFAULT_TEXT_SIZE: f64 = 1.0;

/// Affine slider mapping `size = track * SCALE + OFFSET` (track `0..=1`), consumed
/// by `WidgetStore::link_slider_number_mapped` so the size chip mirrors the slider.
pub const TEXT_SIZE_SLIDER_SCALE: f32 = (TEXT_SIZE_MAX - TEXT_SIZE_MIN) as f32;
pub const TEXT_SIZE_SLIDER_OFFSET: f32 = TEXT_SIZE_MIN as f32;

/// Normalized slider track `0..=1` → glyph size (world units) `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_size(track: f32) -> f64 {
    TEXT_SIZE_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_SIZE_MAX - TEXT_SIZE_MIN)
}

/// Glyph size (world units) → normalized slider track `0..=1` (inverse of
/// [`slider_to_text_size`]); seeds the knob from the shell's current size.
#[must_use]
pub fn text_size_to_slider(size: f64) -> f32 {
    (((size.clamp(TEXT_SIZE_MIN, TEXT_SIZE_MAX) - TEXT_SIZE_MIN) / (TEXT_SIZE_MAX - TEXT_SIZE_MIN))
        as f32)
        .clamp(0.0, 1.0)
}

/// Variable-font Weight (`wght` axis) — the Text-mode Weight slider's range. Inter
/// Variable spans 100..900 (default 400), the CSS/OpenType weight scale. Shared by the
/// panel (seed + chip mapping) and the shell drain (track → weight).
pub const TEXT_WEIGHT_MIN: f64 = 100.0;
pub const TEXT_WEIGHT_MAX: f64 = 900.0;
/// Default weight for a new text session (`wght` 400 = Regular).
pub const DEFAULT_TEXT_WEIGHT: f64 = 400.0;

/// Affine slider mapping `weight = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_WEIGHT_SLIDER_SCALE: f32 = (TEXT_WEIGHT_MAX - TEXT_WEIGHT_MIN) as f32;
pub const TEXT_WEIGHT_SLIDER_OFFSET: f32 = TEXT_WEIGHT_MIN as f32;

/// Normalized slider track `0..=1` → font weight `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_weight(track: f32) -> f64 {
    TEXT_WEIGHT_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_WEIGHT_MAX - TEXT_WEIGHT_MIN)
}

/// Font weight → normalized slider track `0..=1` (inverse of [`slider_to_text_weight`]);
/// seeds the knob from the shell's current weight.
#[must_use]
pub fn text_weight_to_slider(weight: f64) -> f32 {
    (((weight.clamp(TEXT_WEIGHT_MIN, TEXT_WEIGHT_MAX) - TEXT_WEIGHT_MIN)
        / (TEXT_WEIGHT_MAX - TEXT_WEIGHT_MIN)) as f32)
        .clamp(0.0, 1.0)
}

/// Text line height (leading) as a MULTIPLE of the glyph size. 1.2 is the common
/// default; the range spans tight (0.8) to airy (3.0). Shared by the panel (seed +
/// chip mapping) and the shell drain (track → line height).
pub const TEXT_LINE_HEIGHT_MIN: f64 = 0.8;
pub const TEXT_LINE_HEIGHT_MAX: f64 = 3.0;
/// Default line height for a new text session (1.2× the size).
pub const DEFAULT_TEXT_LINE_HEIGHT: f64 = 1.2;

/// Affine slider mapping `line_height = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_LINE_HEIGHT_SLIDER_SCALE: f32 = (TEXT_LINE_HEIGHT_MAX - TEXT_LINE_HEIGHT_MIN) as f32;
pub const TEXT_LINE_HEIGHT_SLIDER_OFFSET: f32 = TEXT_LINE_HEIGHT_MIN as f32;

/// Normalized slider track `0..=1` → line height `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_line_height(track: f32) -> f64 {
    TEXT_LINE_HEIGHT_MIN
        + f64::from(track.clamp(0.0, 1.0)) * (TEXT_LINE_HEIGHT_MAX - TEXT_LINE_HEIGHT_MIN)
}

/// Line height → normalized slider track `0..=1` (inverse of [`slider_to_text_line_height`]).
#[must_use]
pub fn text_line_height_to_slider(line_height: f64) -> f32 {
    (((line_height.clamp(TEXT_LINE_HEIGHT_MIN, TEXT_LINE_HEIGHT_MAX) - TEXT_LINE_HEIGHT_MIN)
        / (TEXT_LINE_HEIGHT_MAX - TEXT_LINE_HEIGHT_MIN)) as f32)
        .clamp(0.0, 1.0)
}

/// A largura da caixa a que o texto REFLUI, em unidades de MUNDO — a faixa do slider "Wrap
/// width", vivo só no modo `Fixed`.
///
/// ⚠️ **Isto é o ALCANCE do slider, não um teto do modelo** (§0): `wrap_width` é um
/// `Option<f64>` e não consome recurso nenhum — não há memória, banda nem precisão a proteger,
/// então não há número a MEDIR. O que a faixa diz é *onde arrastar deixa de ser útil*: o mínimo
/// é ~uma palavra curta no tamanho default (1.0), abaixo do qual toda palavra transborda para a
/// sua própria linha; o máximo é 4× a altura da vista do editor (~10 unidades), onde a caixa já
/// não cabe no ecrã. Uma caixa fora da faixa continua **exprimível** — o documento guarda um
/// `f64` — e é o smoke que decide se estes dois números são os certos.
pub const TEXT_WRAP_MIN: f64 = 1.0;
pub const TEXT_WRAP_MAX: f64 = 40.0;
/// A largura com que uma caixa nasce ao passar de `Auto` para `Fixed` — larga o bastante para
/// caber uma frase no tamanho default, estreita o bastante para o refluxo ser VISÍVEL no ecrã.
pub const DEFAULT_TEXT_WRAP: f64 = 12.0;

/// Affine slider mapping `wrap = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_WRAP_SLIDER_SCALE: f32 = (TEXT_WRAP_MAX - TEXT_WRAP_MIN) as f32;
pub const TEXT_WRAP_SLIDER_OFFSET: f32 = TEXT_WRAP_MIN as f32;

/// Normalized slider track `0..=1` → largura de refluxo (mundo) `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_wrap(track: f32) -> f64 {
    TEXT_WRAP_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_WRAP_MAX - TEXT_WRAP_MIN)
}

/// Largura de refluxo → normalized slider track `0..=1` (inverso de [`slider_to_text_wrap`]).
#[must_use]
pub fn text_wrap_to_slider(wrap: f64) -> f32 {
    (((wrap.clamp(TEXT_WRAP_MIN, TEXT_WRAP_MAX) - TEXT_WRAP_MIN) / (TEXT_WRAP_MAX - TEXT_WRAP_MIN))
        as f32)
        .clamp(0.0, 1.0)
}

/// Text letter-spacing (tracking) as a FRACTION of the glyph size (em), added between
/// glyphs. 0 = the font's native spacing; negative tightens, positive opens up.
pub const TEXT_TRACKING_MIN: f64 = -0.1;
pub const TEXT_TRACKING_MAX: f64 = 0.5;
/// Default tracking for a new text session (0 = native spacing).
pub const DEFAULT_TEXT_TRACKING: f64 = 0.0;

/// Affine slider mapping `tracking = track * SCALE + OFFSET` (track `0..=1`).
pub const TEXT_TRACKING_SLIDER_SCALE: f32 = (TEXT_TRACKING_MAX - TEXT_TRACKING_MIN) as f32;
pub const TEXT_TRACKING_SLIDER_OFFSET: f32 = TEXT_TRACKING_MIN as f32;

/// Normalized slider track `0..=1` → tracking (em fraction) `MIN..=MAX`.
#[must_use]
pub fn slider_to_text_tracking(track: f32) -> f64 {
    TEXT_TRACKING_MIN + f64::from(track.clamp(0.0, 1.0)) * (TEXT_TRACKING_MAX - TEXT_TRACKING_MIN)
}

/// Tracking (em fraction) → normalized slider track `0..=1` (inverse of
/// [`slider_to_text_tracking`]).
#[must_use]
pub fn text_tracking_to_slider(tracking: f64) -> f32 {
    (((tracking.clamp(TEXT_TRACKING_MIN, TEXT_TRACKING_MAX) - TEXT_TRACKING_MIN)
        / (TEXT_TRACKING_MAX - TEXT_TRACKING_MIN)) as f32)
        .clamp(0.0, 1.0)
}
