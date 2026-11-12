//! Os pintores de texto CORTADO numa linha (a reticência de [`crate::text_elide`]): saíram de
//! `paint_text.rs` pelo tecto de 700 linhas; o caminho público continua `paint_text::…`.

use super::{Lines, paint_text_spans};
use crate::text_elide::elide_tracked;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::{Color, VectorScene};

// ⚠️ **Os três pintores de texto CORTADO vieram do `text_elide`** (auditoria A10, 2026-09-12): a lei
// da reticência ([`crate::text_elide`]) fica em baixo, e quem PINTA mora com os outros pintores de
// texto. Os corpos não mudaram uma linha — o `elide` passou a ser nomeado pelo módulo dele.

/// Paint `text` on **one line**, ellipsized when it does not fit `max_width`.
///
/// [`paint_text`] treats `max_width` as a *wrap* budget, so a label one pixel
/// too wide silently becomes two lines and spills into the row below. Anything
/// that must stay on its own line — list rows, track names — belongs here.
///
/// `max_width` too small for even the ellipsis paints nothing.
#[allow(clippy::too_many_arguments)]
#[track_caller] // o `elisao::Medido::onde` nomeia o PINTOR, nunca esta crate
pub fn paint_text_elided(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_elided_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::MEDIUM,
        0.0,
    );
}

/// [`paint_text_elided`] num peso escolhido (o corpo em `NORMAL`, à maquete): mede, corta e pinta com o MESMO
/// peso — quem mede chama `prefix_width_weighted` com ele.
#[track_caller]
pub fn paint_text_elided_weighted(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    [x, y]: [f32; 2],
    font_size: f32,
    max_width: f32,
    (color, weight): (Color, FontWeight),
) {
    paint_elided_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        weight,
        0.0,
    );
}

/// ⭐ [`paint_text_elided`] em **SemiBold** — a irmã de [`paint_text_title`], para o mesmo
/// motivo pelo qual ela existe.
///
/// ⚠️ Sem ela, cortar um TÍTULO obrigava a escolher entre duas regressões silenciosas: pintar
/// o corte em `Medium` (o título muda de peso e ninguém escreveu isso) ou medir em `Medium` e
/// pintar em `SemiBold` (o prefixo escolhido transborda ~3 %, exactamente na fronteira em que o
/// corte existe para não transbordar).
#[allow(clippy::too_many_arguments)]
#[track_caller] // o `elisao::Medido::onde` nomeia o PINTOR, nunca esta crate
pub fn paint_text_title_elided(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_elided_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::SEMI_BOLD,
        0.0,
    );
}

/// O peso de [`paint_text_title_elided_tracked`], o mesmo que [`title_tracked_width`] mede.
const TITLE_TRACKED_WEIGHT: FontWeight = FontWeight::SEMI_BOLD;

/// A largura de `text` inteiro como [`paint_text_title_elided_tracked`] o pinta (mesmo peso e
/// espaçamento, no regime de peso de quem chama): quem dá a largura mínima a um título.
#[must_use]
pub fn title_tracked_width(
    text_system: &mut TextSystem,
    text: &str,
    font_size: f32,
    tracking_em: f32,
) -> f32 {
    text_system.prefix_width_tracked(text, font_size, TITLE_TRACKED_WEIGHT, tracking_em)
}

/// [`paint_text_title_elided`] com espaçamento entre letras `tracking_em` (em `em`): mede e corta
/// com o MESMO valor com que pinta. `0.0` pinta o mesmo, ao bit.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn paint_text_title_elided_tracked(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    [x, y]: [f32; 2],
    font_size: f32,
    max_width: f32,
    color: Color,
    tracking_em: f32,
) {
    let (t, w) = (tracking_em, TITLE_TRACKED_WEIGHT);
    paint_elided_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        w,
        t,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_elided_weighted(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
    tracking_em: f32,
) {
    // ⚠️⚠️ **O peso ATRAVESSA, não é escolhido por um `if`.** A 1.ª redacção ramificava em
    // `weight == SEMI_BOLD`, e uma auditoria adversarial mostrou que era **um braço só**:
    // um terceiro peso seria MEDIDO nele e PINTADO em Medium, em silêncio — o defeito exacto
    // que este módulo existe para impedir. Três mutações sobreviveram a 1 100 testes por
    // causa dele. *Uma lista de pesos é uma lista que alguém esquece; um parâmetro não.*
    let paint = |ts: &mut TextSystem, sc: &mut VectorScene, t: &str| {
        let (inf, wrap) = (f32::INFINITY, Lines::Wrap);
        paint_text_spans(
            ts,
            sc,
            t,
            [x, y],
            font_size,
            inf,
            color,
            weight,
            wrap,
            &[],
            tracking_em,
        );
    };
    if max_width <= 0.0 {
        return;
    }
    // ⭐ **O BALÃO** — a área é a faixa que este pintor recebeu. Ver [`crate::text_elide::balao`].
    let _balao =
        crate::text_elide::balao::Ambito::nova(crate::zones::Rect::new(x, y, max_width, font_size));
    // ⚠️ **A pergunta vai pela porta** ([`crate::text_elide::coube`]) e não por uma comparação
    // escrita aqui: é ela que o censo das elisões ouve, e um pintor que a repita à mão fica
    // invisível ao gate que pergunta *«e quando alguém traduzir?»*.
    let tracked = (weight, tracking_em);
    if crate::text_elide::coube_tracked(text_system, text, font_size, max_width, tracked) {
        // `INFINITY`, not `max_width`: it fits, and passing the budget back would
        // let a sub-pixel measurement disagreement re-introduce the wrap.
        paint(text_system, scene, text);
        return;
    }
    let Some(elided) = elide_tracked(text_system, text, font_size, max_width, tracked) else {
        return;
    };
    paint(text_system, scene, &elided);
}
