//! Texto vetorial — o CONVERTER puro: transforma o contorno de um glyph (skrifa via
//! `ph2d-vector-font`) em UM `VecPath` compound do motor novo. O contorno externo vai
//! em `verts`, os furos (miolo do `o`, `e`, `a`…) em `subpaths`, com
//! `FillRule::NonZero` — a canônica OT/SVG que resolve buracos. Assim um glyph é uma
//! forma vetorial como qualquer outra: herda render, gizmo, snap, undo e Hierarquia.
//! A sessão de edição que consome isto vive no módulo irmão [`crate::vec_text`].
//!
//! **Coordenadas.** Unidades de design são y-up e o world do editor TAMBÉM é y-up
//! (quem inverte pra tela é a câmera, `world_to_screen_affine` com `scale(k, −k)`).
//! Então mapeia `(x, y) → (x·scale + origin.x, y·scale + origin.y)`, SEM flip — flipar
//! aqui somaria com o da câmera e o texto sairia de cabeça pra baixo.
//!
//! **Handles.** O `VecVertex` guarda handles ABSOLUTOS (estilo Rive `CubicVertex`),
//! não vetores-tangente. Quad `S–Q–E` sobe pra cúbica com controles absolutos
//! `S + ⅔(Q−S)` e `E + ⅔(Q−E)`; a cúbica usa `c1`/`c2` diretos.

use ph2d_tool_vector::TextAlign;
use ph2d_vec_edit::PenStyle;
use ph2d_vec_scene::{Contour, FillRule, Paint, StrokeSpec, VecPath};
use ph2d_vector_font::{AxisTag, VariableFont};

/// Os knobs de layout de um bloco de texto (tudo o que NÃO é fonte/eixo/cor). Agrupa
/// tamanho + tipografia num só parâmetro para o converter não estourar o número de
/// argumentos. `line_height` é múltiplo do tamanho; `tracking` é fração do tamanho (em)
/// somada entre glyphs; `align` posiciona cada linha em relação à origem (o clique).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TextLayout {
    pub size: f64,
    pub line_height: f64,
    pub tracking: f64,
    pub align: TextAlign,
}

/// Resolve o Style do Pen (fill/stroke/width em px) no par (fill, stroke) que cada
/// glyph-path recebe — mesma regra das formas (`shape.rs`): sem preenchimento quando
/// alpha 0; traço sempre presente (o render pula width/alpha 0). Assim o texto herda
/// Fill/Stroke/Width/Cap/Join do painel do vetor como qualquer forma.
#[must_use]
pub(crate) fn resolve_style(
    style: &PenStyle,
    px_to_world: f64,
) -> (Option<Paint>, Option<StrokeSpec>) {
    let fill = (style.fill.a != 0).then(|| Paint::solid(style.fill));
    let stroke = Some(style.stroke_spec(style.stroke_w_px * px_to_world));
    (fill, stroke)
}

/// Layout de uma string em `VecPath`s — um por glyph com contorno. Cada linha (`\n`
/// separa) é medida e deslocada pelo alinhamento; `tracking` abre/fecha o espaço entre
/// glyphs; `line_height` (× tamanho) é a entrelinha. `axes` (ex. `wght`) valem no
/// contorno E no avanço (a métrica muda com o peso). Advance-only, sem shaping complexo.
#[must_use]
pub(crate) fn text_to_vec_paths(
    font: &VariableFont,
    text: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
    origin: [f64; 2],
    fill: &Option<Paint>,
    stroke: &Option<StrokeSpec>,
) -> Vec<VecPath> {
    let scale = layout.size / f64::from(font.units_per_em().max(1));
    let line_h = layout.size * layout.line_height;
    let track_px = layout.size * layout.tracking;
    let mut out = Vec::new();
    let mut pen_y = 0.0;
    for line in text.split('\n') {
        let width = line_advance(font, line, scale, track_px, axes);
        let mut pen_x = align_offset(layout.align, width);
        for ch in line.chars() {
            let Some(gid) = font.glyph_for_char(ch) else {
                continue;
            };
            let advance = f64::from(font.advance(gid, axes).unwrap_or(0.0)) * scale;
            if let Ok(outline) = font.outline(gid, axes)
                && let Some(path) = glyph_to_vec_path(
                    &outline,
                    scale,
                    [origin[0] + pen_x, origin[1] + pen_y],
                    fill.clone(),
                    *stroke,
                )
            {
                out.push(path);
            }
            pen_x += advance + track_px;
        }
        pen_y -= line_h; // linhas descem = y menor (world y-up)
    }
    out
}

/// O texto inteiro como UM `VecPath` compound (todos os contornos de todos os glyphs
/// — externos + furos — num só path, `NonZero`). É a geometria do texto VIVO: um
/// objeto, um pick, um gizmo. `None` se não houver glyph com área (string vazia/só
/// espaços). Para "Convert to Curves", use [`text_to_vec_paths`] (um path por glyph).
#[must_use]
pub(crate) fn text_to_compound_path(
    font: &VariableFont,
    text: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
    origin: [f64; 2],
    fill: &Option<Paint>,
    stroke: &Option<StrokeSpec>,
) -> Option<VecPath> {
    let glyphs = text_to_vec_paths(font, text, layout, axes, origin, &None, &None);
    // Concatena o contorno externo + os furos de cada glyph num único compound.
    let mut contours: Vec<Contour> = Vec::new();
    for g in glyphs {
        contours.push(Contour {
            verts: g.verts,
            closed: g.closed,
        });
        contours.extend(g.subpaths);
    }
    let mut iter = contours.into_iter();
    let first = iter.next()?;
    Some(VecPath {
        verts: first.verts,
        closed: true,
        fill: fill.clone(),
        stroke: *stroke,
        subpaths: iter.collect(),
        fill_rule: FillRule::NonZero,
        ..Default::default()
    })
}

/// Centro da bbox (âncoras + alças) de um `VecPath` — o ponto que vira o pivô da
/// forma viva (Live Shapes: a geometria nasce centrada no local 0). `[0,0]` se vazio.
#[must_use]
pub(crate) fn path_center(path: &VecPath) -> [f64; 2] {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            lo[0] = lo[0].min(p[0]);
            lo[1] = lo[1].min(p[1]);
            hi[0] = hi[0].max(p[0]);
            hi[1] = hi[1].max(p[1]);
        }
    }
    if lo[0].is_finite() {
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
    } else {
        [0.0, 0.0]
    }
}

/// Desloca toda a geometria de `path` (âncoras + alças, em todos os contornos) por `d`.
pub(crate) fn offset_path(path: &mut VecPath, d: [f64; 2]) {
    path.for_each_vert_mut(|v| {
        v.anchor = [v.anchor[0] + d[0], v.anchor[1] + d[1]];
        v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
        v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
    });
}

/// Largura visual de UMA linha em world: soma dos avanços + `tracking` entre glyphs
/// (`n−1` gaps). `axes` no mesmo `location` do layout (o avanço muda com o peso).
fn line_advance(
    font: &VariableFont,
    line: &str,
    scale: f64,
    track_px: f64,
    axes: &[(AxisTag, f32)],
) -> f64 {
    let mut width = 0.0;
    let mut glyphs = 0usize;
    for ch in line.chars() {
        if let Some(g) = font.glyph_for_char(ch) {
            width += f64::from(font.advance(g, axes).unwrap_or(0.0)) * scale;
            glyphs += 1;
        }
    }
    width + track_px * glyphs.saturating_sub(1) as f64
}

/// Deslocamento em x da 1ª coluna de uma linha de largura `width`, dado o alinhamento:
/// esquerda começa na origem, centro a centraliza, direita a termina nela.
fn align_offset(align: TextAlign, width: f64) -> f64 {
    match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => -width / 2.0,
        TextAlign::Right => -width,
    }
}

/// Deslocamento em x (a partir de `origin.x`) do CURSOR na ponta de `last_line`: o
/// offset de alinhamento + a largura da linha (com tracking). Usado pelo caret.
#[must_use]
pub(crate) fn caret_x_offset(
    font: &VariableFont,
    last_line: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
) -> f64 {
    let scale = layout.size / f64::from(font.units_per_em().max(1));
    let track_px = layout.size * layout.tracking;
    let width = line_advance(font, last_line, scale, track_px, axes);
    align_offset(layout.align, width) + width
}

/// O builder de glyph (contorno → `VecPath` compound) vive no módulo irmão —
/// re-exportado para o layout acima (e a shell) seguirem chamando `vec_glyph::`.
pub(crate) use crate::vec_glyph_build::glyph_to_vec_path;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::Rgba8;

    fn black() -> Paint {
        Paint::solid(Rgba8::new(0, 0, 0, 255))
    }

    /// Multi-linha: a 2ª linha desce (world y-up → y negativo abaixo da baseline).
    #[test]
    fn a_second_line_sits_below_the_first() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let font = &font;
        let min_y = |paths: &[VecPath]| {
            paths
                .iter()
                .flat_map(|p| p.verts.iter())
                .map(|v| v.anchor[1])
                .fold(f64::INFINITY, f64::min)
        };
        let lay = TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
        };
        let one = text_to_vec_paths(font, "A", &lay, &[], [0.0, 0.0], &Some(black()), &None);
        let two = text_to_vec_paths(font, "A\nA", &lay, &[], [0.0, 0.0], &Some(black()), &None);
        assert!(
            min_y(&one) >= -1e-6,
            "linha única: baseline em 0, sem descer"
        );
        assert!(
            min_y(&two) < -0.5,
            "a 2ª linha desce abaixo da baseline da 1ª"
        );
    }

    /// Centralizar uma linha a desloca para a ESQUERDA da origem (o bloco fica
    /// centrado no ponto de clique); alinhar à direita a termina na origem.
    #[test]
    fn alignment_shifts_the_line_horizontally() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let min_x = |paths: &[VecPath]| {
            paths
                .iter()
                .flat_map(|p| p.verts.iter())
                .map(|v| v.anchor[0])
                .fold(f64::INFINITY, f64::min)
        };
        let lay = |align| TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align,
        };
        let left = text_to_vec_paths(
            &font,
            "AA",
            &lay(TextAlign::Left),
            &[],
            [0.0, 0.0],
            &Some(black()),
            &None,
        );
        let center = text_to_vec_paths(
            &font,
            "AA",
            &lay(TextAlign::Center),
            &[],
            [0.0, 0.0],
            &Some(black()),
            &None,
        );
        let right = text_to_vec_paths(
            &font,
            "AA",
            &lay(TextAlign::Right),
            &[],
            [0.0, 0.0],
            &Some(black()),
            &None,
        );
        assert!(
            min_x(&center) < min_x(&left),
            "centralizado começa à esquerda do alinhado à esquerda"
        );
        assert!(
            min_x(&right) < min_x(&center),
            "à direita começa ainda mais à esquerda (termina na origem)"
        );
    }

    /// O texto vivo é UM path compound com todos os contornos: "Hi" = H (1) + ponto e
    /// haste do i (2) → 1 verts + ≥2 subpaths, um objeto só.
    #[test]
    fn compound_path_merges_all_glyph_contours() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let lay = TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
        };
        let one = text_to_compound_path(&font, "Hi", &lay, &[], [0.0, 0.0], &Some(black()), &None)
            .unwrap();
        assert!(
            !one.subpaths.is_empty(),
            "vários glyphs/furos viram subpaths do mesmo path"
        );
        assert!(
            text_to_compound_path(&font, "   ", &lay, &[], [0.0, 0.0], &Some(black()), &None)
                .is_none(),
            "só espaços = sem geometria"
        );
    }

    /// Tracking positivo abre o espaço entre glyphs, então a mesma string ocupa mais
    /// largura (o cursor avança mais).
    #[test]
    fn positive_tracking_widens_the_line() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let base = TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
        };
        let wide = TextLayout {
            tracking: 0.3,
            ..base
        };
        let narrow = caret_x_offset(&font, "AAA", &base, &[]);
        let opened = caret_x_offset(&font, "AAA", &wide, &[]);
        assert!(
            opened > narrow + 0.5,
            "tracking abre a linha (0.3·size × 2 gaps)"
        );
    }
}
