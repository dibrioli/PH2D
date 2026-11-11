//! O texto rodado 90° à esquerda: a coluna de legendas do rail. Irmão de `paint_text.rs` pelo tecto de LOC.

use super::*;

/// Like [`paint_text`] but rotates the layout 90° counter-clockwise
/// so the text reads bottom-to-top. The anchor `(anchor_x, anchor_y)`
/// is where the rotated baseline's left edge lands — visually this is
/// the BOTTOM-left of the painted text. `max_width` constrains the
/// pre-rotation layout width (i.e. the visual HEIGHT after rotation).
///
/// Used by the LeftRail to paint per-button sub-labels in the column
/// to the left of the chips.
#[allow(clippy::too_many_arguments)]
#[track_caller] // o `elisao::Medido::onde` nomeia o PINTOR, nunca esta crate
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
    // ⚠️ Girado 90° para a esquerda, o «para baixo» do texto é o +x do ecrã: é o x que centra.
    let anchor_x = centrado(anchor_x, font_size);
    // Rotate 90° CCW around the anchor, then translate to it.
    let transform = Affine::translate((anchor_x as f64, anchor_y as f64))
        * Affine::rotate(-std::f64::consts::FRAC_PI_2);
    let params = rendering.params();
    let snap_x = params.snap_x;
    let hint = params.hint;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let font = run.font();
            let run_font_size = run.font_size();
            let coords = ph2d_text::run_coord_bits(run);
            inner
                .draw_glyphs(font)
                .font_size(run_font_size)
                // Hint per preset (same as the straight painter — see
                // `paint_text_weighted`). Hinting under a 90° rotation:
                // skrifa snaps to the *layout* pixel grid pre-rotation,
                // axis-aligned so post-rotation grid alignment is 1:1.
                .hint(hint)
                .font_embolden(crate::paint::font_embolden_for(run_font_size, color))
                // Forward parley's per-run variation coords (wght +
                // opsz). See `paint_text_weighted` for why this is
                // critical — without it Vello ignores the weight stop.
                .normalized_coords(&coords)
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
