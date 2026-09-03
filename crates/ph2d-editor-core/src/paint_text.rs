//! **A família do TEXTO** — layout parley → `draw_glyphs`, e a única metade do
//! `paint` que sabe o que é uma fonte.
//!
//! Módulo irmão do [`super::paint`] por RESPONSABILIDADE e não por tamanho: o
//! resto daquele arquivo emite geometria (retângulos, círculos, ícones, polilinhas)
//! e não olha para um glifo; esta metade shapea, quebra linha e emite runs. As
//! duas crescem por motivos diferentes — a de lá quando chega uma primitiva nova,
//! esta quando chega uma pergunta nova sobre texto (foi o `paint_text_block`, que
//! devolve a altura, que estourou o teto congelado do `paint.rs`).
//!
//! ⚠️ **Os caminhos dos chamadores não mudam:** `paint.rs` re-exporta tudo, então
//! `ph2d_editor_core::paint::paint_text` continua sendo o endereço. Um split que
//! obrigasse ~200 sítios a reescrever o `use` seria churn puro pelo mesmo
//! resultado.

use super::{snap_x_apply, text_rendering};
use ph2d_text::{FontWeight, PositionedLayoutItem, TextSystem};
use ph2d_vector::{Affine, Color, Diagonal2, Fill, FontEmbolden, Glyph, VectorScene};

/// Lay out `text` via parley + emit a glyph run for each parley
/// [`PositionedLayoutItem::GlyphRun`] at `(x, y)` (top-left origin).
/// `font_size` is in device-independent pixels; `max_width` is the
/// wrap budget (pass `f32::INFINITY` for single-line).
#[allow(clippy::too_many_arguments)]
pub fn paint_text(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::MEDIUM,
    );
}

/// [`paint_text`] que devolve a **ALTURA que o texto de fato ocupou**, já com a
/// quebra de linha aplicada.
///
/// ⚠️ **Pintar e medir são a MESMA passada de layout, de propósito.** Uma função
/// de medição separada faria parley duas vezes e as duas respostas poderiam
/// divergir — e a forma que essa divergência toma é a que o painel de física
/// mostrou: uma dica de duas linhas avançando `ROW_H_PX` e escrevendo por cima da
/// linha seguinte. Quem empilha texto de comprimento variável tem de perguntar
/// ao pintor quanto ele gastou, não estimar.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_block(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) -> f32 {
    paint_text_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::MEDIUM,
    )
}

/// SemiBold (600) variant of [`paint_text`] for panel titles and
/// other prominent headings. See [`TextSystem::layout_with_weight`]
/// for why titles need the extra weight: diagonals in glyphs like
/// "y" hint poorly at small sizes without LCD subpixel AA, and the
/// extra pen mass closes the perceptual gap with vertical-stem
/// letters in the same word.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_title(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::SEMI_BOLD,
    );
}

/// ⚠️ `pub(crate)` desde 2026-08-30: o [`crate::text_elide`] precisa de pintar no MESMO peso
/// em que mediu, e a alternativa era um `if weight == SEMI_BOLD { … } else { … }` lá —
/// **um `if` de um braço só** (§5.0), em que um terceiro peso seria medido num e pintado
/// noutro, em silêncio. *Uma lista de pesos é uma lista que alguém esquece; um parâmetro não.*
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_text_weighted(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) -> f32 {
    let rendering = text_rendering();
    let layout = text_system.layout_for_rendering(text, font_size, max_width, weight, rendering);
    let height = layout.height();
    let inner = scene.inner_mut();
    // Snap the text origin to integer pixels: hinting snaps stems to the
    // glyph's local pixel grid, but if the *baseline* lands at a
    // fractional Y the snapped grid is itself offset → soft. Callers
    // routinely produce fractional Y from vertical centering math like
    // `rect.y + (rect.h - font_size) * 0.5`. Rounding here makes every
    // caller crisp without each one having to remember to align.
    let translate = Affine::translate((x.round() as f64, y.round() as f64));
    let params = rendering.params();
    let snap_x = params.snap_x;
    let hint = params.hint;
    let embolden = params.embolden_px;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let font = run.font();
            let run_font_size = run.font_size();
            inner
                .draw_glyphs(font)
                .font_size(run_font_size)
                // ⭐⭐ **A dilatação sintética da outline** (Vello 0.10) — o que o eixo
                // `wght` não sabe fazer: engrossar **só** o X. Os três presets
                // históricos declaram `(0.0, 0.0)`, e aí o Vello nem entra no caminho da
                // expansão (`glyph_cache.rs` compara contra `Diagonal2::new(0,0)`) ⇒ eles
                // ficam **byte-idênticos** ao que shipava.
                .font_embolden(FontEmbolden::new(Diagonal2::new(
                    f64::from(embolden.0),
                    f64::from(embolden.1),
                )))
                // Hint per preset. `true` snapa stems ao pixel grid
                // (crisp); `false` deixa o eixo wght variable fluir
                // sem quantização (necessário para CrispHeavy ficar
                // visualmente distinto de Crisp a 11-12 px).
                .hint(hint)
                // **Critical**: forward parley's per-run variation
                // coordinates (already includes the wght axis we
                // pushed in `layout_for_rendering`). Without this,
                // Vello rasterizes glyphs with the font's default
                // axis values — Inter Variable falls back to ~Regular
                // 400 regardless of which weight stop was selected,
                // so Crisp Heavy looks identical to Crisp. The slice
                // is `&[i16]` on both sides (parley + vello typedef
                // NormalizedCoord = i16), so no conversion needed.
                .normalized_coords(run.normalized_coords())
                .brush(color)
                .transform(translate)
                .draw(
                    Fill::NonZero,
                    glyph_run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        // Snap glyph Y to integer to keep the baseline
                        // pixel-aligned per glyph. X snap depends on
                        // the current `TextRendering` preset's
                        // `SnapX` strategy (None / Half / Full).
                        //
                        // ⚠️ **`g.y` é Y-DOWN, e nós NÃO o negamos** — nem antes
                        // nem depois da subida do parley 0.6 → 0.11. A prova está
                        // a jusante, no `vello_encoding::resolve`: o transform
                        // por-glifo é `matrix: [1,0,0,-1]` com
                        // `translation: [g.x, g.y]`. O `-1` inverte o CONTORNO
                        // (que vem da fonte em Y-up); a translação compõe-se como
                        // `M·p + t`, logo `t` vive no espaço de SAÍDA, que é o da
                        // tela, Y para baixo — o mesmo em que `Affine::translate`
                        // acima põe o topo da caixa de texto.
                        //
                        // ⚠️ O que a parley 0.8 inverteu foi só o `y_offset` que o
                        // shaper devolve (font space, Y-up) antes de o somar à
                        // baseline; a baseline em si é Y-down nas duas versões e
                        // `positioned_glyphs()` (`g.y += baseline`) é linha a
                        // linha idêntica. ⇒ a 0.6 injetava uma grandeza Y-up num
                        // campo Y-down: um acento posicionado por
                        // mark-attachment do GPOS descia em vez de subir. A subida
                        // CURA esse defeito nosso; compensar o sinal aqui seria
                        // reintroduzi-lo. Para o latino precomposto que o chrome
                        // usa, `y_offset` é 0 e a saída não muda um bit.
                        x: snap_x_apply(g.x, snap_x),
                        y: g.y.round(),
                    }),
                );
        }
    }
    height
}

/// Like [`paint_text`] but rotates the layout 90° counter-clockwise
/// so the text reads bottom-to-top. The anchor `(anchor_x, anchor_y)`
/// is where the rotated baseline's left edge lands — visually this is
/// the BOTTOM-left of the painted text. `max_width` constrains the
/// pre-rotation layout width (i.e. the visual HEIGHT after rotation).
///
/// Used by the LeftRail to paint per-button sub-labels in the column
/// to the left of the chips.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_rotated_ccw(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    anchor_x: f32,
    anchor_y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    // Apply the same TextRendering strategy as the straight painter
    // — `layout_for_rendering` bumps the FontWeight per the preset's
    // tier and the rotated glyph loop honors snap-X (pre-rotation
    // coords; the 90° rotation is axis-aligned so post-rotation pixel
    // alignment is preserved by the snap).
    let rendering = text_rendering();
    let layout =
        text_system.layout_for_rendering(text, font_size, max_width, FontWeight::MEDIUM, rendering);
    let inner = scene.inner_mut();
    // Rotate 90° CCW around the anchor, then translate to it.
    let transform = Affine::translate((anchor_x as f64, anchor_y as f64))
        * Affine::rotate(-std::f64::consts::FRAC_PI_2);
    let params = rendering.params();
    let snap_x = params.snap_x;
    let hint = params.hint;
    let embolden = params.embolden_px;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let font = run.font();
            let run_font_size = run.font_size();
            inner
                .draw_glyphs(font)
                .font_size(run_font_size)
                // ⭐⭐ **A dilatação sintética da outline** (Vello 0.10) — o que o eixo
                // `wght` não sabe fazer: engrossar **só** o X. Os três presets
                // históricos declaram `(0.0, 0.0)`, e aí o Vello nem entra no caminho da
                // expansão (`glyph_cache.rs` compara contra `Diagonal2::new(0,0)`) ⇒ eles
                // ficam **byte-idênticos** ao que shipava.
                .font_embolden(FontEmbolden::new(Diagonal2::new(
                    f64::from(embolden.0),
                    f64::from(embolden.1),
                )))
                // Hint per preset (same as the straight painter — see
                // `paint_text_weighted`). Hinting under a 90° rotation:
                // skrifa snaps to the *layout* pixel grid pre-rotation,
                // axis-aligned so post-rotation grid alignment is 1:1.
                .hint(hint)
                // Forward parley's per-run variation coords (wght +
                // opsz). See `paint_text_weighted` for why this is
                // critical — without it Vello ignores the weight stop.
                .normalized_coords(run.normalized_coords())
                .brush(color)
                .transform(transform)
                .draw(
                    Fill::NonZero,
                    glyph_run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        // Snap X pre-rotation in Crisp; rotation
                        // turns this into snap-Y in screen space —
                        // which aligns rotated stems to columns.
                        //
                        // ⚠️ `g.y` passa CRU, sem negação — a convenção é a mesma
                        // do `paint_text_weighted` (lá está o mecanismo inteiro):
                        // `vello::Glyph::y` é Y-down, e a inversão de sinal da
                        // parley 0.8 corrigiu o `y_offset` do shaper, não a
                        // baseline. ⚠️ Diferença com o pintor reto: aqui NÃO há
                        // `.round()`, então este caminho vê o deslocamento
                        // sub-pixel do GPOS por inteiro — se algum dia um rótulo
                        // girado usar marcas combinantes, é aqui que a mudança
                        // aparece primeiro.
                        x: snap_x_apply(g.x, snap_x),
                        y: g.y,
                    }),
                );
        }
    }
}
